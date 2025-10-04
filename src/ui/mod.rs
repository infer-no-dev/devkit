//! User interface components for the terminal application.

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
    Frame, Terminal,
};
use std::io::{stdout, Stdout};
use std::time::{Duration, Instant};
use tokio::time::interval;
pub mod blocks;
pub mod input;
pub mod keybindings;
pub mod notifications;
pub mod panels;
pub mod syntax;
pub mod themes;

use crate::agents::{AgentStatus, TaskPriority};
use input::InputHandler;
use keybindings::{Action, KeyBindings, KeyContext, KeybindingManager};
use notifications::Notification;
use panels::{AgentDisplayInfo, PanelManager, PanelType};
use themes::{Theme, ThemeManager};

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
    SetLayout(String),
    ShowCompletions(Vec<String>),
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
        enable_raw_mode()
            .map_err(|e| UIError::SetupFailed(format!("Failed to enable raw mode: {}", e)))?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen).map_err(|e| {
            UIError::SetupFailed(format!("Failed to enter alternate screen: {}", e))
        })?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)
            .map_err(|e| UIError::SetupFailed(format!("Failed to create terminal: {}", e)))?;

        let mut theme_manager = ThemeManager::new();
        if !theme_manager.set_theme(&config.theme) {
            eprintln!(
                "Warning: Failed to set theme '{}'. Using default theme.",
                config.theme
            );
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
            self.terminal
                .draw(|f| {
                    Self::render_frame(f, &theme, panel_manager, input_handler);
                })
                .map_err(|e| UIError::RenderError(format!("Render failed: {}", e)))?;
        }

        self.cleanup()?;
        Ok(())
    }

    /// Handle key events
    fn handle_key_event(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<(), UIError> {
        // Create KeyEvent for input handler
        let key_event = KeyEvent::new(key, modifiers);

        // Get current input context
        let current_context = *self.input_handler.current_context();

        // In input or command mode, handle input keys first
        if current_context == KeyContext::Input || current_context == KeyContext::Command {
            if let Ok(result) = self.input_handler.handle_key_event(key_event) {
                match result {
                    input::InputResult::Command(cmd) => {
                        // Process command
                        self.process_command(cmd);
                        // Switch back to normal mode after command
                        self.input_handler
                            .set_context(keybindings::KeyContext::Normal);
                        return Ok(());
                    }
                    input::InputResult::Input(input) => {
                        // Process input
                        self.process_command(input);
                        // Switch back to normal mode after input
                        self.input_handler
                            .set_context(keybindings::KeyContext::Normal);
                        return Ok(());
                    }
                    input::InputResult::Action(action) => {
                        // Handle action from input
                        match action {
                            Action::SwitchToNormalMode => {
                                self.input_handler
                                    .set_context(keybindings::KeyContext::Normal);
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                    input::InputResult::Consumed => {
                        // Key was consumed by input handler
                        return Ok(());
                    }
                    _ => {
                        // Fall through to global key handling
                    }
                }
            }
        }

        // Global key handling (only if not in input/command mode, or key wasn't consumed)

        // Check for global keybindings
        if let Some(action) =
            self.keybinding_manager
                .get_action(key, modifiers, &keybindings::Context::Global)
        {
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

        // Check for common key shortcuts
        match key {
            KeyCode::Char('i') if modifiers.is_empty() && current_context == KeyContext::Normal => {
                // Switch to input mode
                self.input_handler
                    .set_context(keybindings::KeyContext::Input);
                return Ok(());
            }
            KeyCode::Char(':') if modifiers.is_empty() && current_context == KeyContext::Normal => {
                // Switch to command mode
                self.input_handler
                    .set_context(keybindings::KeyContext::Command);
                return Ok(());
            }
            KeyCode::F(1) => {
                // F1 shows help
                self.panel_manager.toggle_help();
                return Ok(());
            }
            KeyCode::Char('?') if modifiers.is_empty() && current_context == KeyContext::Normal => {
                // '?' shows help
                self.panel_manager.toggle_help();
                return Ok(());
            }
            KeyCode::Char('q') if modifiers.is_empty() && current_context == KeyContext::Normal => {
                // 'q' quits
                self.running = false;
                return Ok(());
            }
            KeyCode::Tab if modifiers.is_empty() => {
                // Tab cycles focus
                self.cycle_panel_focus();
                return Ok(());
            }
            _ => {}
        }

        // Handle panel-specific keys (only in normal mode)
        if current_context == KeyContext::Normal {
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
            self.panel_manager
                .output_blocks()
                .add_agent_response(&response);

            // Add notification
            let notification = Notification::info(
                "Command Processed".to_string(),
                format!("Executed: {}", command),
            );
            self.panel_manager
                .notification_panel()
                .add_notification(notification);
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

        // Create main layout with status bar at bottom
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Min(3),    // Main content area
                Constraint::Length(1), // Status bar
            ])
            .split(size);

        let main_area = main_chunks[0];
        let status_area = main_chunks[1];

        // Calculate layout constraints based on visible panels
        let constraints = panel_manager.calculate_layout_constraints();

        // Create main layout for panels
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(constraints)
            .split(main_area);

        let mut chunk_idx = 0;

        // Render agent status panel if visible
        if let Some(layout) = panel_manager.get_panel_layout(&PanelType::AgentStatus) {
            if layout.visible && chunk_idx < chunks.len() {
                panel_manager
                    .agent_panel()
                    .render(f, chunks[chunk_idx], theme);
                chunk_idx += 1;
            }
        }

        // Render notifications panel if visible
        if let Some(layout) = panel_manager.get_panel_layout(&PanelType::Notifications) {
            if layout.visible && chunk_idx < chunks.len() {
                panel_manager
                    .notification_panel()
                    .render(f, chunks[chunk_idx], theme);
                chunk_idx += 1;
            }
        }

        // Render output panel if visible
        if let Some(layout) = panel_manager.get_panel_layout(&PanelType::Output) {
            if layout.visible && chunk_idx < chunks.len() {
                panel_manager
                    .output_blocks()
                    .render(f, chunks[chunk_idx], theme);
                chunk_idx += 1;
            }
        }

        // Render input panel if visible
        if let Some(layout) = panel_manager.get_panel_layout(&PanelType::Input) {
            if layout.visible && chunk_idx < chunks.len() {
                input_handler.render(f, chunks[chunk_idx], theme, input_handler.current_context());
            }
        }

        // Render status bar
        Self::render_status_bar(f, status_area, theme, input_handler);

        // Render help overlay if visible (use full size, not main_area)
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
            UIEvent::AgentStatusUpdate {
                agent_name,
                status,
                task,
                priority,
                progress,
            } => {
                let mut info = AgentDisplayInfo::new(agent_name.clone(), "Agent".to_string());
                info.update_status(status);
                if let Some(task) = task {
                    info.set_current_task(Some(task), priority);
                }
                if let Some(progress) = progress {
                    info.update_progress(progress);
                }
                self.panel_manager
                    .agent_panel()
                    .update_agent(agent_name, info);
            }
            UIEvent::Notification(notification) => {
                self.panel_manager
                    .notification_panel()
                    .add_notification(notification);
            }
            UIEvent::Output {
                content,
                block_type,
            } => match block_type.as_str() {
                "user" => self.panel_manager.output_blocks().add_user_input(&content),
                "agent" => self
                    .panel_manager
                    .output_blocks()
                    .add_agent_response(&content),
                "system" => self
                    .panel_manager
                    .output_blocks()
                    .add_system_message(&content),
                "error" => self.panel_manager.output_blocks().add_error(&content),
                _ => self
                    .panel_manager
                    .output_blocks()
                    .add_system_message(&content),
            },
            UIEvent::ToggleHelp => {
                self.panel_manager.toggle_help();
            },
            UIEvent::SetLayout(layout_name) => {
                // TODO: Implement layout switching
                eprintln!("Layout switching not yet implemented: {}", layout_name);
            },
            UIEvent::ShowCompletions(completions) => {
                // TODO: Implement completion display
                eprintln!("Completion display not yet implemented: {} completions", completions.len());
            },
            UIEvent::SwitchTheme(theme_name) => {
                if !self.theme_manager.set_theme(&theme_name) {
                    let notification = Notification::error(
                        "Theme Error".to_string(),
                        format!("Failed to switch to theme '{}'", theme_name),
                    );
                    self.panel_manager
                        .notification_panel()
                        .add_notification(notification);
                }
            }
        }
    }

    /// Cleanup terminal state
    fn cleanup(&mut self) -> Result<(), UIError> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
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
        self.panel_manager
            .notification_panel()
            .add_notification(notification);
    }

    /// Add agent status update
    pub fn update_agent_status(
        &mut self,
        agent_name: String,
        status: AgentStatus,
        task: Option<String>,
        priority: Option<TaskPriority>,
        progress: Option<f64>,
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
        let event = UIEvent::Output {
            content,
            block_type,
        };
        self.handle_event(event);
    }

    /// Set command sender for external command processing
    pub fn set_command_sender(&mut self, sender: tokio::sync::mpsc::UnboundedSender<String>) {
        self.command_sender = Some(sender);
    }

    /// Cycle panel focus
    fn cycle_panel_focus(&mut self) {
        let panels = vec![
            panels::PanelType::Input,
            panels::PanelType::Output,
            panels::PanelType::AgentStatus,
            panels::PanelType::Notifications,
        ];

        let current = self.panel_manager.get_focus();
        let next_index = if let Some(current_type) = current {
            panels
                .iter()
                .position(|p| p == current_type)
                .map(|i| (i + 1) % panels.len())
                .unwrap_or(0)
        } else {
            0
        };

        self.panel_manager
            .set_focus(Some(panels[next_index].clone()));
    }

    /// Render the status bar
    fn render_status_bar(f: &mut Frame, area: Rect, theme: &Theme, input_handler: &InputHandler) {
        use ratatui::style::{Modifier, Style};
        use ratatui::text::{Line, Span};
        use ratatui::widgets::Paragraph;

        // Get current input context
        let context = input_handler.current_context();

        // Create status text based on context
        let status_text = match context {
            keybindings::KeyContext::Input => {
                "INPUT MODE: Type your command and press Enter | ESC: Cancel | ?: Help | q: Quit"
            }
            keybindings::KeyContext::Command => {
                "COMMAND MODE: Type /command and press Enter | ESC: Cancel | ?: Help | q: Quit"
            }
            _ => "i: Input | :: Command | Tab: Cycle Panels | ?: Help | q: Quit | Ctrl+C: Exit",
        };

        let status_line = Line::from(vec![Span::styled(
            status_text,
            Style::default()
                .fg(theme
                    .primary_style()
                    .fg
                    .unwrap_or(ratatui::style::Color::White))
                .add_modifier(Modifier::DIM),
        )]);

        let status_paragraph = Paragraph::new(vec![status_line]).style(
            Style::default().bg(theme
                .border_style()
                .bg
                .unwrap_or(ratatui::style::Color::Black)),
        );

        f.render_widget(status_paragraph, area);
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
