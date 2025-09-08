//! User interface components for the terminal application.

use std::io::{stdout, Stdout};
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Direction, Layout},
    Terminal, Frame,
};
use tokio::time::{interval, Instant};

// Submodules
pub mod blocks;
pub mod input;
pub mod keybindings;
pub mod notifications;
pub mod panels;
pub mod themes;

use input::InputHandler;
use keybindings::{KeybindingManager, KeyBindings, KeyContext, Action};
use notifications::Notification;
use panels::{PanelManager, PanelType, AgentDisplayInfo};
use themes::{Theme, ThemeManager};
use crate::agents::{AgentStatus, TaskPriority};

/// UI configuration
#[derive(Debug, Clone)]
pub struct UIConfig {
    pub auto_scroll: bool,
    pub show_timestamps: bool,
    pub theme: String,
    pub tick_rate: Duration,
}

/// UI errors
#[derive(Debug, thiserror::Error)]
pub enum UIError {
    #[error("Setup failed: {0}")]
    SetupFailed(String),
    #[error("Render error: {0}")]
    RenderError(String),
    #[error("Terminal error: {0}")]
    TerminalError(#[from] std::io::Error),
}

/// Main UI application state
#[derive(Debug)]
pub struct Application {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    theme_manager: ThemeManager,
    panel_manager: PanelManager,
    input_handler: InputHandler,
    keybinding_manager: KeybindingManager,
    config: UIConfig,
    running: bool,
    last_tick: Instant,
    command_sender: Option<tokio::sync::mpsc::UnboundedSender<String>>,
}

/// UI events that can be sent to the application
#[derive(Debug, Clone)]
pub enum UIEvent {
    Quit,
    Input(String),
    AgentStatusUpdate {
        agent_name: String,
        status: AgentStatus,
        task: Option<String>,
        priority: Option<TaskPriority>,
        progress: Option<f64>,
    },
    Notification(Notification),
    Output {
        content: String,
        block_type: String,
    },
    ToggleHelp,
    SwitchTheme(String),
}

impl UIConfig {
    /// Create a new UI configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set theme
    pub fn with_theme(mut self, theme: String) -> Self {
        self.theme = theme;
        self
    }
    
    /// Set tick rate
    pub fn with_tick_rate(mut self, tick_rate: Duration) -> Self {
        self.tick_rate = tick_rate;
        self
    }
}

impl Application {
    /// Create a new UI application
    pub fn new(config: UIConfig) -> Result<Self, UIError> {
        // Setup terminal
        enable_raw_mode().map_err(|e| UIError::SetupFailed(format!("Failed to enable raw mode: {}", e)))?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)
            .map_err(|e| UIError::SetupFailed(format!("Failed to enter alternate screen: {}", e)))?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)
            .map_err(|e| UIError::SetupFailed(format!("Failed to create terminal: {}", e)))?;

        let mut theme_manager = ThemeManager::new();
        if !theme_manager.set_theme(&config.theme) {
            eprintln!("Warning: Failed to set theme '{}'. Using default theme.", config.theme);
        }

        Ok(Self {
            terminal,
            theme_manager,
            panel_manager: PanelManager::new(),
            input_handler: InputHandler::new(KeyBindings::default()),
            keybinding_manager: KeybindingManager::new(),
            config,
            running: true,
            last_tick: Instant::now(),
            command_sender: None,
        })
    }

    /// Run the UI event loop
    pub async fn run(&mut self) -> Result<(), UIError> {
        let mut tick_interval = interval(self.config.tick_rate);
        
        while self.running {
            // Handle input events
            if event::poll(Duration::from_millis(0))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key.code, key.modifiers)?;
                }
            }
            
            // Tick for animations and updates
            tick_interval.tick().await;
            self.tick();
            
            // Render the UI
            let theme = self.theme_manager.current_theme().clone();
            let panel_manager = &mut self.panel_manager;
            let input_handler = &mut self.input_handler;
            self.terminal.draw(|f| {
                Self::render_frame(f, &theme, panel_manager, input_handler);
            })
                .map_err(|e| UIError::RenderError(format!("Render failed: {}", e)))?;
        }
        
        self.cleanup()?;
        Ok(())
    }
    
    /// Handle key events
    fn handle_key_event(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<(), UIError> {
        // Check for global keybindings first
        if let Some(action) = self.keybinding_manager.get_action(key, modifiers, &keybindings::Context::Global) {
            match action {
                Action::Quit => {
                    self.running = false;
                    return Ok(());
                }
                Action::ToggleHelp => {
                    self.panel_manager.toggle_help();
                    return Ok(());
                }
                Action::SwitchTheme => {
                    self.theme_manager.cycle_theme();
                    return Ok(());
                }
                _ => {}
            }
        }
        
        // Check if we should handle input directly
        match key {
            KeyCode::Char('i') if modifiers.is_empty() => {
                // Switch to input mode
                self.input_handler.set_context(keybindings::KeyContext::Input);
                return Ok(());
            }
            KeyCode::Char(':') if modifiers.is_empty() => {
                // Switch to command mode
                self.input_handler.set_context(keybindings::KeyContext::Command);
                return Ok(());
            }
            _ => {}
        }
        
        // Handle input with KeyEvent
        let key_event = KeyEvent::new(key, modifiers);
        if let Ok(result) = self.input_handler.handle_key_event(key_event) {
            match result {
                input::InputResult::Command(cmd) => {
                    // Process command
                    self.process_command(cmd);
                    // Switch back to normal mode after command
                    self.input_handler.set_context(keybindings::KeyContext::Normal);
                }
                input::InputResult::Input(input) => {
                    // Process input
                    self.process_command(input);
                    // Switch back to normal mode after input
                    self.input_handler.set_context(keybindings::KeyContext::Normal);
                }
                input::InputResult::Action(action) => {
                    // Handle action from input
                    match action {
                        Action::SwitchToNormalMode => {
                            self.input_handler.set_context(keybindings::KeyContext::Normal);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        
        // Handle panel-specific keys
        match self.panel_manager.get_focus() {
            Some(PanelType::AgentStatus) => {
                match key {
                    KeyCode::Enter => {
                        // Toggle expanded agent view (would need selected agent)
                    }
                    KeyCode::Char('s') => {
                        // Cycle sort method
                    }
                    KeyCode::Char('i') => {
                        self.panel_manager.agent_panel().toggle_show_inactive();
                    }
                    _ => {}
                }
            }
            Some(PanelType::Notifications) => {
                match key {
                    KeyCode::Char('c') => {
                        self.panel_manager.notification_panel().clear_dismissible();
                    }
                    KeyCode::Char('d') => {
                        // Dismiss selected notification (would need selection)
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Process a command
    fn process_command(&mut self, command: String) {
        // Send command to external processor if available
        if let Some(sender) = &self.command_sender {
            let _ = sender.send(command.clone());
        } else {
            // Fallback: Add command output
            self.panel_manager.output_blocks().add_user_input(&command);
            
            // Process the command (placeholder)
            let response = format!("Processing command: {}", command);
            self.panel_manager.output_blocks().add_agent_response(&response);
            
            // Add notification
            let notification = Notification::info(
                "Command Processed".to_string(),
                format!("Executed: {}", command),
            );
            self.panel_manager.notification_panel().add_notification(notification);
        }
    }
    
    /// Update UI state (called on each tick)
    fn tick(&mut self) {
        self.last_tick = Instant::now();
        
        // Cleanup expired notifications
        self.panel_manager.notification_panel().cleanup_expired();
        
        // Update input handler
        self.input_handler.tick();
    }
    
    /// Static render function to avoid borrow checker issues
    fn render_frame(
        f: &mut Frame,
        theme: &Theme,
        panel_manager: &mut PanelManager,
        input_handler: &mut InputHandler,
    ) {
        let size = f.area();
        
        // Calculate layout constraints based on visible panels
        let constraints = panel_manager.calculate_layout_constraints();
        
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(constraints)
            .split(size);
        
        let mut chunk_idx = 0;
        
        // Render agent status panel if visible
        if let Some(layout) = panel_manager.get_panel_layout(&PanelType::AgentStatus) {
            if layout.visible && chunk_idx < chunks.len() {
                panel_manager.agent_panel().render(f, chunks[chunk_idx], theme);
                chunk_idx += 1;
            }
        }
        
        // Render notifications panel if visible
        if let Some(layout) = panel_manager.get_panel_layout(&PanelType::Notifications) {
            if layout.visible && chunk_idx < chunks.len() {
                panel_manager.notification_panel().render(f, chunks[chunk_idx], theme);
                chunk_idx += 1;
            }
        }
        
        // Render output panel if visible
        if let Some(layout) = panel_manager.get_panel_layout(&PanelType::Output) {
            if layout.visible && chunk_idx < chunks.len() {
                panel_manager.output_blocks().render(f, chunks[chunk_idx], theme);
                chunk_idx += 1;
            }
        }
        
        // Render input panel if visible
        if let Some(layout) = panel_manager.get_panel_layout(&PanelType::Input) {
            if layout.visible && chunk_idx < chunks.len() {
                input_handler.render(f, chunks[chunk_idx], theme, &KeyContext::Global);
            }
        }
        
        // Render help overlay if visible
        panel_manager.render_help_overlay(f, size, theme);
    }
    
    /// Handle UI events from external sources
    pub fn handle_event(&mut self, event: UIEvent) {
        match event {
            UIEvent::Quit => {
                self.running = false;
            }
            UIEvent::Input(input) => {
                self.input_handler.set_input(input);
            }
            UIEvent::AgentStatusUpdate { agent_name, status, task, priority, progress } => {
                let mut info = AgentDisplayInfo::new(agent_name.clone(), "Agent".to_string());
                info.update_status(status);
                if let Some(task) = task {
                    info.set_current_task(Some(task), priority);
                }
                if let Some(progress) = progress {
                    info.update_progress(progress);
                }
                self.panel_manager.agent_panel().update_agent(agent_name, info);
            }
            UIEvent::Notification(notification) => {
                self.panel_manager.notification_panel().add_notification(notification);
            }
            UIEvent::Output { content, block_type } => {
                match block_type.as_str() {
                    "user" => self.panel_manager.output_blocks().add_user_input(&content),
                    "agent" => self.panel_manager.output_blocks().add_agent_response(&content),
                    "system" => self.panel_manager.output_blocks().add_system_message(&content),
                    "error" => self.panel_manager.output_blocks().add_error(&content),
                    _ => self.panel_manager.output_blocks().add_system_message(&content),
                }
            }
            UIEvent::ToggleHelp => {
                self.panel_manager.toggle_help();
            }
            UIEvent::SwitchTheme(theme_name) => {
                if !self.theme_manager.set_theme(&theme_name) {
                    let notification = Notification::error(
                        "Theme Error".to_string(),
                        format!("Failed to switch to theme '{}'", theme_name),
                    );
                    self.panel_manager.notification_panel().add_notification(notification);
                }
            }
        }
    }
    
    /// Cleanup terminal state
    fn cleanup(&mut self) -> Result<(), UIError> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen
        )?;
        Ok(())
    }
    
    /// Get the current theme
    pub fn theme(&self) -> &Theme {
        self.theme_manager.current_theme()
    }
    
    /// Get mutable access to panel manager
    pub fn panel_manager(&mut self) -> &mut PanelManager {
        &mut self.panel_manager
    }
    
    /// Get mutable access to theme manager
    pub fn theme_manager(&mut self) -> &mut ThemeManager {
        &mut self.theme_manager
    }
    
    /// Check if UI is still running
    pub fn is_running(&self) -> bool {
        self.running
    }
    
    /// Force quit the UI
    pub fn quit(&mut self) {
        self.running = false;
    }
    
    /// Add a notification
    pub fn add_notification(&mut self, notification: Notification) {
        self.panel_manager.notification_panel().add_notification(notification);
    }
    
    /// Add agent status update
    pub fn update_agent_status(
        &mut self, 
        agent_name: String, 
        status: AgentStatus,
        task: Option<String>,
        priority: Option<TaskPriority>,
        progress: Option<f64>
    ) {
        let event = UIEvent::AgentStatusUpdate {
            agent_name,
            status,
            task,
            priority,
            progress,
        };
        self.handle_event(event);
    }
    
    /// Add output to display
    pub fn add_output(&mut self, content: String, block_type: String) {
        let event = UIEvent::Output { content, block_type };
        self.handle_event(event);
    }
    
    /// Set command sender for external command processing
    pub fn set_command_sender(&mut self, sender: tokio::sync::mpsc::UnboundedSender<String>) {
        self.command_sender = Some(sender);
    }
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            auto_scroll: true,
            show_timestamps: true,
            theme: "Dark".to_string(),
            tick_rate: Duration::from_millis(50),
        }
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
