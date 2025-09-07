//! Terminal-based user interface for the agentic development environment.
//!
//! This module provides an interactive terminal interface with block-based
//! input/output, customizable keybindings, and real-time agent monitoring.

pub mod blocks;
pub mod input;
pub mod keybindings;
pub mod notifications;
pub mod panels;
pub mod themes;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::collections::HashMap;

/// Main UI application state
#[derive(Debug)]
pub struct Application {
    pub running: bool,
    pub current_mode: AppMode,
    pub input_handler: input::InputHandler,
    pub notification_panel: notifications::NotificationPanel,
    pub agent_panel: panels::AgentPanel,
    pub output_blocks: Vec<blocks::OutputBlock>,
    pub theme: themes::Theme,
    pub keybindings: keybindings::KeyBindings,
}

/// Application modes
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Input,
    Command,
    AgentView,
    Settings,
}

/// UI events that can occur
#[derive(Debug, Clone)]
pub enum UIEvent {
    KeyPress(KeyEvent),
    AgentUpdate(String, String), // agent_id, status
    NotificationReceived(notifications::Notification),
    OutputReceived(blocks::OutputBlock),
    Resize(u16, u16),
    Quit,
}

/// Configuration for UI appearance and behavior
#[derive(Debug, Clone)]
pub struct UIConfig {
    pub theme: themes::Theme,
    pub keybindings: keybindings::KeyBindings,
    pub auto_scroll: bool,
    pub show_timestamps: bool,
    pub max_output_blocks: usize,
    pub notification_timeout: u64,
}

/// Errors that can occur in the UI
#[derive(Debug, thiserror::Error)]
pub enum UIError {
    #[error(\"Terminal setup failed: {0}\")]
    TerminalSetupFailed(String),
    
    #[error(\"Rendering failed: {0}\")]
    RenderingFailed(String),
    
    #[error(\"Input handling failed: {0}\")]
    InputHandlingFailed(String),
    
    #[error(\"Theme loading failed: {0}\")]
    ThemeLoadingFailed(String),
}

impl Application {
    /// Create a new application instance
    pub fn new(config: UIConfig) -> Result<Self, UIError> {
        Ok(Self {
            running: true,
            current_mode: AppMode::Normal,
            input_handler: input::InputHandler::new(config.keybindings.clone()),
            notification_panel: notifications::NotificationPanel::new(config.notification_timeout),
            agent_panel: panels::AgentPanel::new(),
            output_blocks: Vec::new(),
            theme: config.theme,
            keybindings: config.keybindings,
        })
    }
    
    /// Handle UI events
    pub async fn handle_event(&mut self, event: UIEvent) -> Result<(), UIError> {
        match event {
            UIEvent::KeyPress(key_event) => {
                self.handle_key_event(key_event).await?;
            },
            UIEvent::AgentUpdate(agent_id, status) => {
                self.agent_panel.update_agent_status(agent_id, status);
            },
            UIEvent::NotificationReceived(notification) => {
                self.notification_panel.add_notification(notification);
            },
            UIEvent::OutputReceived(output_block) => {
                self.add_output_block(output_block);
            },
            UIEvent::Resize(width, height) => {
                // Handle terminal resize if needed
            },
            UIEvent::Quit => {
                self.running = false;
            },
        }
        Ok(())
    }
    
    /// Handle keyboard input
    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<(), UIError> {
        match self.current_mode {
            AppMode::Normal => {
                match key_event.code {
                    KeyCode::Char('q') => self.running = false,
                    KeyCode::Char('i') => self.current_mode = AppMode::Input,
                    KeyCode::Char(':') => self.current_mode = AppMode::Command,
                    KeyCode::Char('a') => self.current_mode = AppMode::AgentView,
                    KeyCode::Char('s') => self.current_mode = AppMode::Settings,
                    _ => {},
                }
            },
            AppMode::Input => {
                match key_event.code {
                    KeyCode::Esc => self.current_mode = AppMode::Normal,
                    KeyCode::Enter => {
                        let input = self.input_handler.get_current_input();
                        self.process_input(input).await?;
                        self.input_handler.clear_input();
                        self.current_mode = AppMode::Normal;
                    },
                    _ => {
                        self.input_handler.handle_key_event(key_event)?;
                    },
                }
            },
            AppMode::Command => {
                match key_event.code {
                    KeyCode::Esc => self.current_mode = AppMode::Normal,
                    KeyCode::Enter => {
                        let command = self.input_handler.get_current_input();
                        self.execute_command(command).await?;
                        self.input_handler.clear_input();
                        self.current_mode = AppMode::Normal;
                    },
                    _ => {
                        self.input_handler.handle_key_event(key_event)?;
                    },
                }
            },
            _ => {
                match key_event.code {
                    KeyCode::Esc => self.current_mode = AppMode::Normal,
                    _ => {},
                }
            },
        }
        Ok(())
    }
    
    /// Process natural language input
    async fn process_input(&mut self, input: String) -> Result<(), UIError> {
        // This would integrate with the agent system to process natural language input
        let output = blocks::OutputBlock {
            id: uuid::Uuid::new_v4().to_string(),
            content: format!(\"Processing: {}\", input),
            block_type: blocks::BlockType::UserInput,
            timestamp: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        };
        self.add_output_block(output);
        Ok(())
    }
    
    /// Execute a command
    async fn execute_command(&mut self, command: String) -> Result<(), UIError> {
        // This would integrate with the shell system to execute commands
        let output = blocks::OutputBlock {
            id: uuid::Uuid::new_v4().to_string(),
            content: format!(\"Executing command: {}\", command),
            block_type: blocks::BlockType::Command,
            timestamp: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        };
        self.add_output_block(output);
        Ok(())
    }
    
    /// Add an output block to the display
    fn add_output_block(&mut self, block: blocks::OutputBlock) {
        self.output_blocks.push(block);
        
        // Limit the number of output blocks to prevent memory issues
        let max_blocks = 1000; // This could be configurable
        if self.output_blocks.len() > max_blocks {
            self.output_blocks.drain(0..self.output_blocks.len() - max_blocks);
        }
    }
    
    /// Render the UI
    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>) {
        let size = f.size();
        
        // Create the main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),     // Status bar
                Constraint::Min(10),       // Main content
                Constraint::Length(3),     // Input area
                Constraint::Length(5),     // Notification panel
            ])
            .split(size);
        
        // Render status bar
        self.render_status_bar(f, chunks[0]);
        
        // Create horizontal split for main content
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70),  // Output area
                Constraint::Percentage(30),  // Agent panel
            ])
            .split(chunks[1]);
        
        // Render main content areas
        self.render_output_area(f, main_chunks[0]);
        self.render_agent_panel(f, main_chunks[1]);
        
        // Render input area
        self.render_input_area(f, chunks[2]);
        
        // Render notification panel
        self.render_notification_panel(f, chunks[3]);
    }
    
    /// Render the status bar
    fn render_status_bar<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let status_text = format!(
            \"Mode: {:?} | Press 'q' to quit | 'i' for input | ':' for commands\",
            self.current_mode
        );
        
        let status = Paragraph::new(status_text)
            .style(self.theme.status_bar_style());
        
        f.render_widget(status, area);
    }
    
    /// Render the output area
    fn render_output_area<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let items: Vec<ListItem> = self.output_blocks
            .iter()
            .rev()
            .take(area.height as usize - 2)
            .map(|block| {
                ListItem::new(block.render(&self.theme))
            })
            .collect();
        
        let output_list = List::new(items)
            .block(Block::default()
                .title(\"Output\")
                .borders(Borders::ALL)
                .style(self.theme.output_area_style()));
        
        f.render_widget(output_list, area);
    }
    
    /// Render the agent panel
    fn render_agent_panel<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        self.agent_panel.render(f, area, &self.theme);
    }
    
    /// Render the input area
    fn render_input_area<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        self.input_handler.render(f, area, &self.theme, &self.current_mode);
    }
    
    /// Render the notification panel
    fn render_notification_panel<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        self.notification_panel.render(f, area, &self.theme);
    }
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            theme: themes::Theme::default(),
            keybindings: keybindings::KeyBindings::default(),
            auto_scroll: true,
            show_timestamps: true,
            max_output_blocks: 1000,
            notification_timeout: 5000, // 5 seconds
        }
    }
}
