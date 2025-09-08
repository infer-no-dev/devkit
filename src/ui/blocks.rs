//! Output blocks for displaying various types of content in the UI.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use ratatui::text::{Line, Span, Text};
use crate::ui::themes::Theme;

/// Types of output blocks that can be displayed
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlockType {
    UserInput,
    AgentResponse,
    Command,
    CommandOutput,
    Error,
    Warning,
    Info,
    Success,
    CodeGeneration,
    Analysis,
    Notification,
    System,
}

/// An output block containing content and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputBlock {
    pub id: String,
    pub content: String,
    pub block_type: BlockType,
    pub timestamp: std::time::SystemTime,
    pub metadata: HashMap<String, String>,
}

/// Block formatting options
#[derive(Debug, Clone)]
pub struct BlockFormat {
    pub show_timestamp: bool,
    pub show_type_indicator: bool,
    pub wrap_content: bool,
    pub max_width: Option<u16>,
    pub indent_level: u16,
}

impl OutputBlock {
    /// Create a new output block
    pub fn new(
        content: String,
        block_type: BlockType,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            block_type,
            timestamp: std::time::SystemTime::now(),
            metadata,
        }
    }
    
    /// Create a user input block
    pub fn user_input(content: String) -> Self {
        Self::new(content, BlockType::UserInput, HashMap::new())
    }
    
    /// Create an agent response block
    pub fn agent_response(content: String, agent_name: String) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("agent".to_string(), agent_name);
        Self::new(content, BlockType::AgentResponse, metadata)
    }
    
    /// Create a command execution block
    pub fn command(content: String, exit_code: i32) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("exit_code".to_string(), exit_code.to_string());
        Self::new(content, BlockType::Command, metadata)
    }
    
    /// Create an error block
    pub fn error(content: String) -> Self {
        Self::new(content, BlockType::Error, HashMap::new())
    }
    
    /// Create a code generation block
    pub fn code_generation(content: String, language: String) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("language".to_string(), language);
        Self::new(content, BlockType::CodeGeneration, metadata)
    }
    
    /// Render the block as formatted text for the terminal
    pub fn render(&self, theme: &Theme) -> Text<'_> {
        let mut lines = Vec::new();
        
        // Add timestamp and type indicator if needed
        let header = self.format_header(theme);
        if !header.spans.is_empty() {
            lines.push(header);
        }
        
        // Add the main content
        let content_lines = self.format_content(theme);
        lines.extend(content_lines);
        
        // Add metadata if present
        let metadata_lines = self.format_metadata(theme);
        lines.extend(metadata_lines);
        
        Text::from(lines)
    }
    
    /// Format the block header (timestamp, type, etc.)
    fn format_header(&self, theme: &Theme) -> Line<'_> {
        let mut spans = Vec::new();
        
        // Add timestamp
        if let Ok(duration) = self.timestamp.duration_since(std::time::UNIX_EPOCH) {
            let timestamp = duration.as_secs();
            let time_str = format!("[{}] ", 
                chrono::DateTime::from_timestamp(timestamp as i64, 0)
                    .unwrap_or_default()
                    .format("%H:%M:%S")
            );
            spans.push(Span::styled(time_str, theme.timestamp_style()));
        }
        
        // Add type indicator
        let (type_str, type_style) = match self.block_type {
            BlockType::UserInput => ("→ ", theme.user_input_style()),
            BlockType::AgentResponse => ("🤖 ", theme.agent_response_style()),
            BlockType::Command => ("$ ", theme.command_style()),
            BlockType::CommandOutput => ("", theme.command_output_style()),
            BlockType::Error => ("❌ ", theme.error_style()),
            BlockType::Warning => ("⚠️  ", theme.warning_style()),
            BlockType::Info => ("ℹ️  ", theme.info_style()),
            BlockType::Success => ("✅ ", theme.success_style()),
            BlockType::CodeGeneration => ("🔧 ", theme.code_generation_style()),
            BlockType::Analysis => ("🔍 ", theme.analysis_style()),
            BlockType::Notification => ("📢 ", theme.notification_style()),
            BlockType::System => ("⚙️  ", theme.system_style()),
        };
        
        if !type_str.is_empty() {
            spans.push(Span::styled(type_str.to_string(), type_style));
        }
        
        // Add agent name if present
        if let Some(agent_name) = self.metadata.get("agent") {
            spans.push(Span::styled(
                format!("[{}] ", agent_name),
                theme.agent_name_style(),
            ));
        }
        
        Line::from(spans)
    }
    
    /// Format the main content
    fn format_content(&self, theme: &Theme) -> Vec<Line<'_>> {
        let content_style = match self.block_type {
            BlockType::UserInput => theme.user_input_content_style(),
            BlockType::AgentResponse => theme.agent_response_content_style(),
            BlockType::Command => theme.command_content_style(),
            BlockType::CommandOutput => theme.command_output_content_style(),
            BlockType::Error => theme.error_content_style(),
            BlockType::Warning => theme.warning_content_style(),
            BlockType::Info => theme.info_content_style(),
            BlockType::Success => theme.success_content_style(),
            BlockType::CodeGeneration => theme.code_generation_content_style(),
            BlockType::Analysis => theme.analysis_content_style(),
            BlockType::Notification => theme.notification_content_style(),
            BlockType::System => theme.system_content_style(),
        };
        
        // Handle multi-line content
        self.content
            .lines()
            .map(|line| {
                Line::from(vec![Span::styled(line.to_string(), content_style)])
            })
            .collect()
    }
    
    /// Format metadata information
    fn format_metadata(&self, theme: &Theme) -> Vec<Line<'_>> {
        let mut lines = Vec::new();
        
        // Show exit code for commands
        if let Some(exit_code) = self.metadata.get("exit_code") {
            if exit_code != "0" {
                let line = Line::from(vec![
                    Span::styled("  Exit code: ", theme.metadata_key_style()),
                    Span::styled(exit_code.clone(), theme.metadata_value_style()),
                ]);
                lines.push(line);
            }
        }
        
        // Show language for code generation
        if let Some(language) = self.metadata.get("language") {
            let line = Line::from(vec![
                Span::styled("  Language: ", theme.metadata_key_style()),
                Span::styled(language.clone(), theme.metadata_value_style()),
            ]);
            lines.push(line);
        }
        
        lines
    }
    
    /// Get a short summary of the block for display
    pub fn summary(&self) -> String {
        let preview_length = 50;
        let content_preview = if self.content.len() > preview_length {
            format!("{}...", &self.content[..preview_length])
        } else {
            self.content.clone()
        };
        
        format!("{:?}: {}", self.block_type, content_preview)
    }
    
    /// Check if this block matches a filter
    pub fn matches_filter(&self, filter: &BlockFilter) -> bool {
        // Check block type filter
        if let Some(types) = &filter.block_types {
            if !types.contains(&self.block_type) {
                return false;
            }
        }
        
        // Check content filter
        if let Some(content_filter) = &filter.content_contains {
            if !self.content.to_lowercase().contains(&content_filter.to_lowercase()) {
                return false;
            }
        }
        
        // Check agent filter
        if let Some(agent_filter) = &filter.agent_name {
            if let Some(agent_name) = self.metadata.get("agent") {
                if agent_name != agent_filter {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // Check time filter
        if let Some(after) = filter.after_time {
            if self.timestamp < after {
                return false;
            }
        }
        
        true
    }
}

/// Filter for selecting specific blocks
#[derive(Debug, Clone, Default)]
pub struct BlockFilter {
    pub block_types: Option<Vec<BlockType>>,
    pub content_contains: Option<String>,
    pub agent_name: Option<String>,
    pub after_time: Option<std::time::SystemTime>,
}

/// Collection of output blocks with management capabilities
#[derive(Debug, Clone)]
pub struct BlockCollection {
    blocks: Vec<OutputBlock>,
    max_blocks: usize,
}

/// Type alias for backward compatibility
pub type OutputBlockCollection = BlockCollection;

impl BlockCollection {
    /// Create a new block collection
    pub fn new(max_blocks: usize) -> Self {
        Self {
            blocks: Vec::new(),
            max_blocks,
        }
    }
    
    /// Add a block to the collection
    pub fn add_block(&mut self, block: OutputBlock) {
        self.blocks.push(block);
        
        // Maintain size limit
        if self.blocks.len() > self.max_blocks {
            let excess = self.blocks.len() - self.max_blocks;
            self.blocks.drain(0..excess);
        }
    }
    
    /// Get all blocks
    pub fn get_blocks(&self) -> &[OutputBlock] {
        &self.blocks
    }
    
    /// Get filtered blocks
    pub fn get_filtered_blocks(&self, filter: &BlockFilter) -> Vec<&OutputBlock> {
        self.blocks
            .iter()
            .filter(|block| block.matches_filter(filter))
            .collect()
    }
    
    /// Get recent blocks (last n)
    pub fn get_recent_blocks(&self, count: usize) -> Vec<&OutputBlock> {
        self.blocks
            .iter()
            .rev()
            .take(count)
            .collect()
    }
    
    /// Clear all blocks
    pub fn clear(&mut self) {
        self.blocks.clear();
    }
    
    /// Get block count
    pub fn len(&self) -> usize {
        self.blocks.len()
    }
    
    /// Check if collection is empty
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
    
    /// Add a user input block
    pub fn add_user_input(&mut self, content: &str) {
        let block = OutputBlock::new(
            content.to_string(),
            BlockType::UserInput,
            HashMap::new(),
        );
        self.add_block(block);
    }
    
    /// Add an agent response block
    pub fn add_agent_response(&mut self, content: &str) {
        let block = OutputBlock::new(
            content.to_string(),
            BlockType::AgentResponse,
            HashMap::new(),
        );
        self.add_block(block);
    }
    
    /// Add a system message block
    pub fn add_system_message(&mut self, content: &str) {
        let block = OutputBlock::new(
            content.to_string(),
            BlockType::System,
            HashMap::new(),
        );
        self.add_block(block);
    }
    
    /// Add an error block
    pub fn add_error(&mut self, content: &str) {
        let mut metadata = HashMap::new();
        metadata.insert("severity".to_string(), "error".to_string());
        let block = OutputBlock::new(
            content.to_string(),
            BlockType::Error,
            metadata,
        );
        self.add_block(block);
    }
    
    /// Render the block collection (placeholder)
    pub fn render(&self, f: &mut ratatui::Frame, area: ratatui::layout::Rect, theme: &crate::ui::themes::Theme) {
        use ratatui::widgets::{Block, Borders, List, ListItem};
        use ratatui::text::{Line, Span};
        
        let items: Vec<ListItem> = self.blocks
            .iter()
            .map(|block| {
                let content = if block.content.len() > 50 {
                    format!("{}...", &block.content[..47])
                } else {
                    block.content.clone()
                };
                
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{:?}: ", block.block_type), theme.primary_style()),
                    Span::styled(content, theme.secondary_style()),
                ]))
            })
            .collect();
        
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Output")
                    .style(theme.border_style())
            )
            .style(theme.secondary_style());
        
        f.render_widget(list, area);
    }
}

impl Default for BlockFormat {
    fn default() -> Self {
        Self {
            show_timestamp: true,
            show_type_indicator: true,
            wrap_content: true,
            max_width: None,
            indent_level: 0,
        }
    }
}
