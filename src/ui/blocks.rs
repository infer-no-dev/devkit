//! Output blocks for displaying various types of content in the UI.

use crate::ui::themes::Theme;
use ratatui::text::{Line, Span, Text};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub fn new(content: String, block_type: BlockType, metadata: HashMap<String, String>) -> Self {
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
            let time_str = format!(
                "[{}] ",
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
            .map(|line| Line::from(vec![Span::styled(line.to_string(), content_style)]))
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
            if !self
                .content
                .to_lowercase()
                .contains(&content_filter.to_lowercase())
            {
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
    scroll_offset: usize,
    auto_scroll: bool,
}

/// Type alias for backward compatibility
pub type OutputBlockCollection = BlockCollection;

impl BlockCollection {
    /// Create a new block collection
    pub fn new(max_blocks: usize) -> Self {
        Self {
            blocks: Vec::new(),
            max_blocks,
            scroll_offset: 0,
            auto_scroll: true,
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

        // Auto-scroll to bottom when new content is added
        if self.auto_scroll {
            // For auto-scroll, we'll just let the render method handle positioning
            // by using the total line count as the scroll offset
            let total_lines = self.get_total_display_lines();
            self.scroll_offset = total_lines;
        }
    }

    /// Scroll up by the specified number of lines
    pub fn scroll_up(&mut self, lines: usize) {
        if self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(lines);
            self.auto_scroll = false; // Disable auto-scroll when manually scrolling
        }
    }

    /// Scroll down by the specified number of lines
    pub fn scroll_down(&mut self, lines: usize) {
        let total_lines = self.get_total_display_lines();
        if self.scroll_offset < total_lines {
            self.scroll_offset += lines;
            // Check if we've scrolled to the bottom
            if self.scroll_offset >= total_lines {
                self.scroll_to_bottom();
            }
        }
    }

    /// Scroll to the top
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
        self.auto_scroll = false;
    }

    /// Scroll to the bottom
    pub fn scroll_to_bottom(&mut self) {
        let total_lines = self.get_total_display_lines();
        // Ensure we don't scroll past the content
        self.scroll_offset = total_lines.saturating_sub(1);
        self.auto_scroll = true;
    }

    /// Page up (scroll up by a page)
    pub fn page_up(&mut self, page_size: usize) {
        self.scroll_up(page_size);
    }

    /// Page down (scroll down by a page)
    pub fn page_down(&mut self, page_size: usize) {
        self.scroll_down(page_size);
    }

    /// Toggle auto-scroll mode
    pub fn toggle_auto_scroll(&mut self) {
        self.auto_scroll = !self.auto_scroll;
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    /// Get current scroll offset
    pub fn get_scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Check if auto-scroll is enabled
    pub fn is_auto_scroll(&self) -> bool {
        self.auto_scroll
    }

    /// Calculate total number of display lines for all blocks
    fn get_total_display_lines(&self) -> usize {
        self.blocks.iter().map(|block| {
            // Header line + content lines + metadata lines + separator
            1 + block.content.lines().count() + 
            block.metadata.len() + 1 // separator line
        }).sum()
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
        self.blocks.iter().rev().take(count).collect()
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
        let block = OutputBlock::new(content.to_string(), BlockType::UserInput, HashMap::new());
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
        let block = OutputBlock::new(content.to_string(), BlockType::System, HashMap::new());
        self.add_block(block);
    }

    /// Add an error block
    pub fn add_error(&mut self, content: &str) {
        let mut metadata = HashMap::new();
        metadata.insert("severity".to_string(), "error".to_string());
        let block = OutputBlock::new(content.to_string(), BlockType::Error, metadata);
        self.add_block(block);
    }

    /// Render the block collection with proper formatting and scrolling support
    pub fn render(
        &self,
        f: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        theme: &crate::ui::themes::Theme,
        is_focused: bool,
    ) {
        use ratatui::widgets::{Block, Borders, Paragraph, Wrap, Scrollbar, ScrollbarOrientation, ScrollbarState};
        use ratatui::text::Text;
        use ratatui::layout::{Constraint, Direction, Layout};

        // Calculate available height for content (subtract borders)
        let content_height = area.height.saturating_sub(2) as usize; // 2 for top and bottom borders
        
        // Collect all rendered blocks into lines
        let mut all_lines = Vec::new();
        
        for block in &self.blocks {
            let block_text = block.render(theme);
            all_lines.extend(block_text.lines);
            // Add empty line between blocks
            all_lines.push(Line::from(""));
        }
        
        // If no blocks, show helpful message
        if self.blocks.is_empty() {
            all_lines.push(Line::from("No output yet. Press 'i' to enter input mode and start typing commands."));
        }

        // Apply scrolling - take lines from scroll_offset to scroll_offset + content_height
        let total_lines = all_lines.len();
        
        // Ensure valid scroll position
        let start_line = if total_lines == 0 {
            0
        } else if self.auto_scroll && total_lines > content_height {
            // For auto-scroll, show the last content_height lines
            total_lines.saturating_sub(content_height)
        } else {
            // For manual scroll, use scroll_offset but ensure it's valid
            self.scroll_offset.min(total_lines.saturating_sub(content_height).max(0))
        };
        
        let end_line = (start_line + content_height).min(total_lines);
        
        let visible_lines = if total_lines > 0 && start_line < total_lines {
            all_lines[start_line..end_line].to_vec()
        } else {
            all_lines
        };

        let text = Text::from(visible_lines);
        
        // Create title with scroll information and focus indicator
        let scroll_info = if total_lines > content_height {
            format!(" ({}/{}) {}", 
                start_line + 1, 
                total_lines, 
                if self.auto_scroll { "[AUTO]" } else { "[MANUAL]" }
            )
        } else {
            String::new()
        };
        
        let focus_indicator = if is_focused { " [FOCUSED]" } else { "" };
        let title = format!("Output ({}){}{}", self.blocks.len(), scroll_info, focus_indicator);
        
        // Split area to make room for scrollbar if needed
        let show_scrollbar = total_lines > content_height;
        let (paragraph_area, scrollbar_area) = if show_scrollbar {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(0), Constraint::Length(1)])
                .split(area);
            (chunks[0], chunks[1])
        } else {
            (area, ratatui::layout::Rect::default())
        };
        
        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .style(theme.panel_border_style(is_focused)),
            )
            .wrap(Wrap { trim: true })
            .style(theme.primary_style());

        f.render_widget(paragraph, paragraph_area);
        
        // Render scrollbar if needed
        if show_scrollbar {
            let mut scrollbar_state = ScrollbarState::default()
                .content_length(total_lines)
                .viewport_content_length(content_height)
                .position(start_line);
                
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
                
            f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        }
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
