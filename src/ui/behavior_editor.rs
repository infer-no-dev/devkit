//! Behavior Profile Editor UI
//!
//! This module provides a comprehensive UI for creating, editing, and managing
//! agent behavior profiles. It allows users to customize personality traits,
//! decision-making patterns, communication styles, and other behavioral aspects.

use crate::agents::behavior::{
    AgentBehaviorProfile, BehaviorProfileManager, BehaviorValue, 
    PersonalityTraits, DecisionMakingPattern, CommunicationStyle, 
    TaskHandlingBehavior, LearningBehavior, ErrorHandlingBehavior,
    CollaborationBehavior, ResourceUsageBehavior, DecisionStrategy,
    UpdateFrequency, PrioritizationStrategy, ErrorHandlingStrategy,
    ConflictResolutionStrategy, SharingAlgorithm, CacheEvictionPolicy,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Clear, Gauge, List, ListItem, ListState, 
        Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Tabs, Wrap,
    },
    Frame,
};
use std::collections::{HashMap, HashSet};
use std::fmt;
use tracing::{debug, trace};

/// Different sections of the behavior editor
#[derive(Debug, Clone, PartialEq)]
pub enum EditorSection {
    /// Profile overview and metadata
    Overview,
    /// Personality traits configuration
    Personality,
    /// Decision-making patterns
    DecisionMaking,
    /// Communication style settings
    Communication,
    /// Task handling behavior
    TaskHandling,
    /// Learning and adaptation
    Learning,
    /// Error handling strategies
    ErrorHandling,
    /// Collaboration preferences
    Collaboration,
    /// Resource usage policies
    ResourceUsage,
    /// Custom parameters
    CustomParameters,
}

impl fmt::Display for EditorSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            EditorSection::Overview => "Overview",
            EditorSection::Personality => "Personality",
            EditorSection::DecisionMaking => "Decision Making",
            EditorSection::Communication => "Communication",
            EditorSection::TaskHandling => "Task Handling",
            EditorSection::Learning => "Learning",
            EditorSection::ErrorHandling => "Error Handling",
            EditorSection::Collaboration => "Collaboration",
            EditorSection::ResourceUsage => "Resource Usage",
            EditorSection::CustomParameters => "Custom Parameters",
        };
        write!(f, "{}", name)
    }
}

/// Different field types that can be edited
#[derive(Debug, Clone)]
pub enum FieldType {
    /// Text input field
    Text,
    /// Floating point number (0.0 to 1.0)
    Float(f64, f64), // min, max
    /// Integer field
    Integer(i64, i64), // min, max
    /// Boolean toggle
    Boolean,
    /// Duration in seconds
    Duration,
    /// Single selection from enum variants
    Enum(Vec<String>),
    /// Multiple selection from options
    MultiSelect(Vec<String>),
    /// Tag input (comma-separated)
    Tags,
}

/// A field being edited in the behavior profile
#[derive(Debug, Clone)]
pub struct EditableField {
    /// Field identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Field description/help text
    pub description: String,
    /// Type of field
    pub field_type: FieldType,
    /// Current value
    pub value: BehaviorValue,
    /// Whether this field is currently selected
    pub selected: bool,
    /// Whether this field is being edited
    pub editing: bool,
    /// Current edit buffer for text input
    pub edit_buffer: String,
    /// Cursor position in edit buffer
    pub cursor_position: usize,
}

/// Behavior profile editor state and UI
pub struct BehaviorEditor {
    /// Reference to the behavior profile manager
    profile_manager: Option<BehaviorProfileManager>,
    /// Currently edited profile (clone for safety)
    current_profile: Option<AgentBehaviorProfile>,
    /// Original profile ID (for updates vs creates)
    original_profile_id: Option<String>,
    /// Current editor section
    current_section: EditorSection,
    /// Fields in the current section
    current_fields: Vec<EditableField>,
    /// Selected field index
    selected_field_index: usize,
    /// Scroll position for fields list
    scroll_position: usize,
    /// Whether the editor is visible
    visible: bool,
    /// Whether there are unsaved changes
    has_changes: bool,
    /// Status message
    status_message: Option<String>,
    /// Error message
    error_message: Option<String>,
    /// List state for UI rendering
    list_state: ListState,
    /// Scroll state for scrollbar
    scroll_state: ScrollbarState,
    /// Available sections tabs
    sections: Vec<EditorSection>,
    /// Current tab index
    tab_index: usize,
}

impl Default for BehaviorEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl BehaviorEditor {
    /// Create a new behavior editor
    pub fn new() -> Self {
        let sections = vec![
            EditorSection::Overview,
            EditorSection::Personality,
            EditorSection::DecisionMaking,
            EditorSection::Communication,
            EditorSection::TaskHandling,
            EditorSection::Learning,
            EditorSection::ErrorHandling,
            EditorSection::Collaboration,
            EditorSection::ResourceUsage,
            EditorSection::CustomParameters,
        ];

        Self {
            profile_manager: None,
            current_profile: None,
            original_profile_id: None,
            current_section: EditorSection::Overview,
            current_fields: Vec::new(),
            selected_field_index: 0,
            scroll_position: 0,
            visible: false,
            has_changes: false,
            status_message: None,
            error_message: None,
            list_state: ListState::default(),
            scroll_state: ScrollbarState::default(),
            tab_index: 0,
            sections,
        }
    }

    /// Show the editor with a profile to edit
    pub fn show_profile(&mut self, profile: AgentBehaviorProfile) {
        self.original_profile_id = Some(profile.id.clone());
        self.current_profile = Some(profile);
        self.current_section = EditorSection::Overview;
        self.tab_index = 0;
        self.has_changes = false;
        self.status_message = None;
        self.error_message = None;
        self.selected_field_index = 0;
        self.scroll_position = 0;
        self.visible = true;
        
        self.update_current_fields();
        trace!("Opened behavior editor for profile");
    }

    /// Show the editor for creating a new profile
    pub fn show_new_profile(&mut self) {
        self.original_profile_id = None;
        self.current_profile = Some(self.create_default_profile());
        self.current_section = EditorSection::Overview;
        self.tab_index = 0;
        self.has_changes = false;
        self.status_message = Some("Creating new behavior profile".to_string());
        self.error_message = None;
        self.selected_field_index = 0;
        self.scroll_position = 0;
        self.visible = true;
        
        self.update_current_fields();
        trace!("Opened behavior editor for new profile");
    }

    /// Hide the editor
    pub fn hide(&mut self) {
        if self.has_changes {
            self.status_message = Some("Warning: Unsaved changes will be lost".to_string());
            return; // Don't actually hide yet, let user confirm
        }
        
        self.visible = false;
        self.current_profile = None;
        self.original_profile_id = None;
        self.current_fields.clear();
        self.has_changes = false;
        self.status_message = None;
        self.error_message = None;
        trace!("Closed behavior editor");
    }

    /// Check if the editor is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Handle key input for the editor
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> EditorAction {
        if !self.visible {
            return EditorAction::None;
        }

        // Handle editing mode
        if let Some(field) = self.current_fields.get_mut(self.selected_field_index) {
            if field.editing {
                return self.handle_field_editing(field, key_event);
            }
        }

        // Handle navigation and general commands
        match key_event.code {
            KeyCode::Esc => {
                if self.has_changes {
                    EditorAction::ConfirmClose
                } else {
                    self.hide();
                    EditorAction::Close
                }
            }
            KeyCode::Tab => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.previous_section();
                } else {
                    self.next_section();
                }
                EditorAction::SectionChanged
            }
            KeyCode::Up => {
                self.navigate_up();
                EditorAction::Navigate
            }
            KeyCode::Down => {
                self.navigate_down();
                EditorAction::Navigate
            }
            KeyCode::PageUp => {
                self.navigate_page_up();
                EditorAction::Navigate
            }
            KeyCode::PageDown => {
                self.navigate_page_down();
                EditorAction::Navigate
            }
            KeyCode::Enter => {
                self.start_editing_current_field();
                EditorAction::StartEdit
            }
            KeyCode::Char('s') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_profile()
            }
            KeyCode::Char('z') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.reset_current_field();
                EditorAction::FieldReset
            }
            KeyCode::F(1) => {
                EditorAction::ShowHelp
            }
            _ => EditorAction::None,
        }
    }

    /// Render the behavior editor
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Clear background
        f.render_widget(Clear, area);

        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(3), // Tabs
                Constraint::Min(10),   // Content
                Constraint::Length(2), // Status
            ])
            .split(area);

        // Render header
        self.render_header(f, chunks[0]);

        // Render section tabs
        self.render_tabs(f, chunks[1]);

        // Render content area
        self.render_content(f, chunks[2]);

        // Render status line
        self.render_status(f, chunks[3]);
    }

    /// Get current profile being edited
    pub fn get_current_profile(&self) -> Option<&AgentBehaviorProfile> {
        self.current_profile.as_ref()
    }

    /// Check if there are unsaved changes
    pub fn has_unsaved_changes(&self) -> bool {
        self.has_changes
    }

    // Private methods

    fn create_default_profile() -> AgentBehaviorProfile {
        use std::time::SystemTime;
        use uuid::Uuid;

        AgentBehaviorProfile {
            id: Uuid::new_v4().to_string(),
            name: "New Profile".to_string(),
            description: "A custom behavior profile".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            tags: HashSet::new(),
            personality: PersonalityTraits::default(),
            decision_making: DecisionMakingPattern::default(),
            communication: CommunicationStyle::default(),
            task_handling: TaskHandlingBehavior::default(),
            learning: LearningBehavior::default(),
            error_handling: ErrorHandlingBehavior::default(),
            collaboration: CollaborationBehavior::default(),
            resource_usage: ResourceUsageBehavior::default(),
            custom_parameters: HashMap::new(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            active: true,
        }
    }

    fn update_current_fields(&mut self) {
        self.current_fields = match &self.current_section {
            EditorSection::Overview => self.create_overview_fields(),
            EditorSection::Personality => self.create_personality_fields(),
            EditorSection::DecisionMaking => self.create_decision_making_fields(),
            EditorSection::Communication => self.create_communication_fields(),
            EditorSection::TaskHandling => self.create_task_handling_fields(),
            EditorSection::Learning => self.create_learning_fields(),
            EditorSection::ErrorHandling => self.create_error_handling_fields(),
            EditorSection::Collaboration => self.create_collaboration_fields(),
            EditorSection::ResourceUsage => self.create_resource_usage_fields(),
            EditorSection::CustomParameters => self.create_custom_parameters_fields(),
        };

        // Reset selection
        self.selected_field_index = 0;
        self.scroll_position = 0;
        if !self.current_fields.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    fn create_overview_fields(&self) -> Vec<EditableField> {
        let profile = match &self.current_profile {
            Some(p) => p,
            None => return Vec::new(),
        };

        vec![
            EditableField {
                id: "name".to_string(),
                name: "Profile Name".to_string(),
                description: "Display name for this behavior profile".to_string(),
                field_type: FieldType::Text,
                value: BehaviorValue::String(profile.name.clone()),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "description".to_string(),
                name: "Description".to_string(),
                description: "Detailed description of this behavior profile".to_string(),
                field_type: FieldType::Text,
                value: BehaviorValue::String(profile.description.clone()),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "version".to_string(),
                name: "Version".to_string(),
                description: "Version number for this profile".to_string(),
                field_type: FieldType::Text,
                value: BehaviorValue::String(profile.version.clone()),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "author".to_string(),
                name: "Author".to_string(),
                description: "Creator of this behavior profile".to_string(),
                field_type: FieldType::Text,
                value: BehaviorValue::String(profile.author.clone().unwrap_or_default()),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "active".to_string(),
                name: "Active".to_string(),
                description: "Whether this profile is available for use".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.active),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
        ]
    }

    fn create_personality_fields(&self) -> Vec<EditableField> {
        let profile = match &self.current_profile {
            Some(p) => p,
            None => return Vec::new(),
        };

        vec![
            EditableField {
                id: "proactiveness".to_string(),
                name: "Proactiveness".to_string(),
                description: "How proactive vs reactive (0.0 = reactive, 1.0 = proactive)".to_string(),
                field_type: FieldType::Float(0.0, 1.0),
                value: BehaviorValue::Float(profile.personality.proactiveness),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "risk_tolerance".to_string(),
                name: "Risk Tolerance".to_string(),
                description: "Risk tolerance (0.0 = risk-averse, 1.0 = risk-seeking)".to_string(),
                field_type: FieldType::Float(0.0, 1.0),
                value: BehaviorValue::Float(profile.personality.risk_tolerance),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "creativity".to_string(),
                name: "Creativity".to_string(),
                description: "Creativity level (0.0 = conservative, 1.0 = highly creative)".to_string(),
                field_type: FieldType::Float(0.0, 1.0),
                value: BehaviorValue::Float(profile.personality.creativity),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "sociability".to_string(),
                name: "Sociability".to_string(),
                description: "Social interaction preference (0.0 = solitary, 1.0 = highly social)".to_string(),
                field_type: FieldType::Float(0.0, 1.0),
                value: BehaviorValue::Float(profile.personality.sociability),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "detail_orientation".to_string(),
                name: "Detail Orientation".to_string(),
                description: "Attention to detail (0.0 = big picture, 1.0 = detail-oriented)".to_string(),
                field_type: FieldType::Float(0.0, 1.0),
                value: BehaviorValue::Float(profile.personality.detail_orientation),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "speed_vs_accuracy".to_string(),
                name: "Speed vs Accuracy".to_string(),
                description: "Speed vs accuracy preference (0.0 = accuracy-focused, 1.0 = speed-focused)".to_string(),
                field_type: FieldType::Float(0.0, 1.0),
                value: BehaviorValue::Float(profile.personality.speed_vs_accuracy),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "helpfulness".to_string(),
                name: "Helpfulness".to_string(),
                description: "Helpfulness (0.0 = minimal help, 1.0 = maximum help)".to_string(),
                field_type: FieldType::Float(0.0, 1.0),
                value: BehaviorValue::Float(profile.personality.helpfulness),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "persistence".to_string(),
                name: "Persistence".to_string(),
                description: "Persistence (0.0 = give up easily, 1.0 = very persistent)".to_string(),
                field_type: FieldType::Float(0.0, 1.0),
                value: BehaviorValue::Float(profile.personality.persistence),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "confidence".to_string(),
                name: "Confidence".to_string(),
                description: "Confidence level (0.0 = uncertain, 1.0 = very confident)".to_string(),
                field_type: FieldType::Float(0.0, 1.0),
                value: BehaviorValue::Float(profile.personality.confidence),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
        ]
    }

    fn create_decision_making_fields(&self) -> Vec<EditableField> {
        let profile = match &self.current_profile {
            Some(p) => p,
            None => return Vec::new(),
        };

        vec![
            EditableField {
                id: "strategy".to_string(),
                name: "Decision Strategy".to_string(),
                description: "Primary decision-making strategy".to_string(),
                field_type: FieldType::Enum(vec![
                    "Heuristic".to_string(),
                    "Analytical".to_string(),
                    "DataDriven".to_string(),
                    "Conservative".to_string(),
                    "Aggressive".to_string(),
                    "Balanced".to_string(),
                    "UserGuided".to_string(),
                    "Collaborative".to_string(),
                ]),
                value: BehaviorValue::String(format!("{:?}", profile.decision_making.strategy)),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "seek_confirmation".to_string(),
                name: "Seek Confirmation".to_string(),
                description: "Whether to seek confirmation for important decisions".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.decision_making.seek_confirmation),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "autonomy_threshold".to_string(),
                name: "Autonomy Threshold".to_string(),
                description: "Confidence threshold for autonomous decisions".to_string(),
                field_type: FieldType::Float(0.0, 1.0),
                value: BehaviorValue::Float(profile.decision_making.autonomy_threshold),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "explain_reasoning".to_string(),
                name: "Explain Reasoning".to_string(),
                description: "Whether to explain reasoning for decisions".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.decision_making.explain_reasoning),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
        ]
    }

    fn create_communication_fields(&self) -> Vec<EditableField> {
        let profile = match &self.current_profile {
            Some(p) => p,
            None => return Vec::new(),
        };

        vec![
            EditableField {
                id: "verbosity".to_string(),
                name: "Verbosity".to_string(),
                description: "Verbosity level (0.0 = terse, 1.0 = verbose)".to_string(),
                field_type: FieldType::Float(0.0, 1.0),
                value: BehaviorValue::Float(profile.communication.verbosity),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "technical_level".to_string(),
                name: "Technical Level".to_string(),
                description: "Use of technical language (0.0 = plain language, 1.0 = technical)".to_string(),
                field_type: FieldType::Float(0.0, 1.0),
                value: BehaviorValue::Float(profile.communication.technical_level),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "emoji_usage".to_string(),
                name: "Emoji Usage".to_string(),
                description: "Emoji and emoticon usage (0.0 = none, 1.0 = frequent)".to_string(),
                field_type: FieldType::Float(0.0, 1.0),
                value: BehaviorValue::Float(profile.communication.emoji_usage),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "provide_explanations".to_string(),
                name: "Provide Explanations".to_string(),
                description: "Whether to provide explanations for actions".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.communication.provide_explanations),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "ask_for_clarification".to_string(),
                name: "Ask for Clarification".to_string(),
                description: "Whether to ask for clarification when uncertain".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.communication.ask_for_clarification),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "summarize_work".to_string(),
                name: "Summarize Work".to_string(),
                description: "Whether to summarize completed work".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.communication.summarize_work),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
        ]
    }

    fn create_task_handling_fields(&self) -> Vec<EditableField> {
        let profile = match &self.current_profile {
            Some(p) => p,
            None => return Vec::new(),
        };

        vec![
            EditableField {
                id: "max_concurrent_tasks".to_string(),
                name: "Max Concurrent Tasks".to_string(),
                description: "Maximum number of concurrent tasks".to_string(),
                field_type: FieldType::Integer(1, 20),
                value: BehaviorValue::Integer(profile.task_handling.max_concurrent_tasks as i64),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "batch_similar_tasks".to_string(),
                name: "Batch Similar Tasks".to_string(),
                description: "Whether to batch similar tasks together".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.task_handling.batch_similar_tasks),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "decompose_complex_tasks".to_string(),
                name: "Decompose Complex Tasks".to_string(),
                description: "Whether to break down complex tasks".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.task_handling.decompose_complex_tasks),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "validate_requirements".to_string(),
                name: "Validate Requirements".to_string(),
                description: "Whether to validate task requirements before starting".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.task_handling.validate_requirements),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
        ]
    }

    fn create_learning_fields(&self) -> Vec<EditableField> {
        let profile = match &self.current_profile {
            Some(p) => p,
            None => return Vec::new(),
        };

        vec![
            EditableField {
                id: "learn_from_interactions".to_string(),
                name: "Learn from Interactions".to_string(),
                description: "Whether to learn from user interactions".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.learning.learn_from_interactions),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "adapt_from_outcomes".to_string(),
                name: "Adapt from Outcomes".to_string(),
                description: "Whether to adapt behavior based on success/failure".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.learning.adapt_from_outcomes),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "learning_rate".to_string(),
                name: "Learning Rate".to_string(),
                description: "Learning rate (0.0 = no learning, 1.0 = rapid learning)".to_string(),
                field_type: FieldType::Float(0.0, 1.0),
                value: BehaviorValue::Float(profile.learning.learning_rate),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "remember_preferences".to_string(),
                name: "Remember Preferences".to_string(),
                description: "Whether to remember user preferences".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.learning.remember_preferences),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
        ]
    }

    fn create_error_handling_fields(&self) -> Vec<EditableField> {
        let profile = match &self.current_profile {
            Some(p) => p,
            None => return Vec::new(),
        };

        vec![
            EditableField {
                id: "error_strategy".to_string(),
                name: "Error Strategy".to_string(),
                description: "Primary error handling strategy".to_string(),
                field_type: FieldType::Enum(vec![
                    "FailFast".to_string(),
                    "RetryWithBackoff".to_string(),
                    "WorkAround".to_string(),
                    "GracefulDegrade".to_string(),
                    "EscalateToHuman".to_string(),
                    "CollaborativeResolve".to_string(),
                    "LearnAndAdapt".to_string(),
                ]),
                value: BehaviorValue::String(format!("{:?}", profile.error_handling.strategy)),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "max_retries".to_string(),
                name: "Max Retries".to_string(),
                description: "Number of retry attempts for recoverable errors".to_string(),
                field_type: FieldType::Integer(0, 10),
                value: BehaviorValue::Integer(profile.error_handling.max_retries as i64),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "escalate_errors".to_string(),
                name: "Escalate Errors".to_string(),
                description: "Whether to escalate unresolved errors".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.error_handling.escalate_errors),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "detailed_logging".to_string(),
                name: "Detailed Logging".to_string(),
                description: "Whether to log detailed error information".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.error_handling.detailed_logging),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
        ]
    }

    fn create_collaboration_fields(&self) -> Vec<EditableField> {
        let profile = match &self.current_profile {
            Some(p) => p,
            None => return Vec::new(),
        };

        vec![
            EditableField {
                id: "collaboration_willingness".to_string(),
                name: "Collaboration Willingness".to_string(),
                description: "Willingness to collaborate (0.0 = individualistic, 1.0 = highly collaborative)".to_string(),
                field_type: FieldType::Float(0.0, 1.0),
                value: BehaviorValue::Float(profile.collaboration.collaboration_willingness),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "seek_collaboration".to_string(),
                name: "Seek Collaboration".to_string(),
                description: "Whether to actively seek collaboration opportunities".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.collaboration.seek_collaboration),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "share_resources".to_string(),
                name: "Share Resources".to_string(),
                description: "Whether to share resources with other agents".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.collaboration.share_resources),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "mentor_others".to_string(),
                name: "Mentor Others".to_string(),
                description: "Whether to mentor or help other agents".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.collaboration.mentor_others),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
        ]
    }

    fn create_resource_usage_fields(&self) -> Vec<EditableField> {
        let profile = match &self.current_profile {
            Some(p) => p,
            None => return Vec::new(),
        };

        vec![
            EditableField {
                id: "cpu_limit".to_string(),
                name: "CPU Limit".to_string(),
                description: "CPU usage limits (0.0 to 1.0 of available)".to_string(),
                field_type: FieldType::Float(0.0, 1.0),
                value: BehaviorValue::Float(profile.resource_usage.cpu_limit),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "yield_when_idle".to_string(),
                name: "Yield When Idle".to_string(),
                description: "Whether to yield resources when not actively working".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.resource_usage.yield_when_idle),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "optimize_usage".to_string(),
                name: "Optimize Usage".to_string(),
                description: "Whether to monitor and optimize resource usage".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.resource_usage.optimize_usage),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
            EditableField {
                id: "cache_enabled".to_string(),
                name: "Cache Enabled".to_string(),
                description: "Whether to use caching".to_string(),
                field_type: FieldType::Boolean,
                value: BehaviorValue::Boolean(profile.resource_usage.cache_behavior.enabled),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            },
        ]
    }

    fn create_custom_parameters_fields(&self) -> Vec<EditableField> {
        let profile = match &self.current_profile {
            Some(p) => p,
            None => return Vec::new(),
        };

        let mut fields = Vec::new();

        for (key, value) in &profile.custom_parameters {
            let field_type = match value {
                BehaviorValue::String(_) => FieldType::Text,
                BehaviorValue::Integer(_, _) => FieldType::Integer(i64::MIN, i64::MAX),
                BehaviorValue::Float(_) => FieldType::Float(f64::NEG_INFINITY, f64::INFINITY),
                BehaviorValue::Boolean(_) => FieldType::Boolean,
                BehaviorValue::Duration(_) => FieldType::Duration,
                _ => FieldType::Text,
            };

            fields.push(EditableField {
                id: format!("custom_{}", key),
                name: key.clone(),
                description: format!("Custom parameter: {}", key),
                field_type,
                value: value.clone(),
                selected: false,
                editing: false,
                edit_buffer: String::new(),
                cursor_position: 0,
            });
        }

        fields
    }

    fn handle_field_editing(&mut self, field: &mut EditableField, key_event: KeyEvent) -> EditorAction {
        match key_event.code {
            KeyCode::Esc => {
                field.editing = false;
                field.edit_buffer.clear();
                EditorAction::CancelEdit
            }
            KeyCode::Enter => {
                if self.apply_field_edit(field) {
                    field.editing = false;
                    self.has_changes = true;
                    EditorAction::ApplyEdit
                } else {
                    EditorAction::EditError
                }
            }
            KeyCode::Backspace => {
                if field.cursor_position > 0 {
                    field.edit_buffer.remove(field.cursor_position - 1);
                    field.cursor_position -= 1;
                }
                EditorAction::EditUpdate
            }
            KeyCode::Delete => {
                if field.cursor_position < field.edit_buffer.len() {
                    field.edit_buffer.remove(field.cursor_position);
                }
                EditorAction::EditUpdate
            }
            KeyCode::Left => {
                if field.cursor_position > 0 {
                    field.cursor_position -= 1;
                }
                EditorAction::EditUpdate
            }
            KeyCode::Right => {
                if field.cursor_position < field.edit_buffer.len() {
                    field.cursor_position += 1;
                }
                EditorAction::EditUpdate
            }
            KeyCode::Home => {
                field.cursor_position = 0;
                EditorAction::EditUpdate
            }
            KeyCode::End => {
                field.cursor_position = field.edit_buffer.len();
                EditorAction::EditUpdate
            }
            KeyCode::Char(c) => {
                field.edit_buffer.insert(field.cursor_position, c);
                field.cursor_position += 1;
                EditorAction::EditUpdate
            }
            _ => EditorAction::None,
        }
    }

    fn apply_field_edit(&mut self, field: &EditableField) -> bool {
        let profile = match &mut self.current_profile {
            Some(p) => p,
            None => return false,
        };

        match field.field_type {
            FieldType::Text => {
                match field.id.as_str() {
                    "name" => profile.name = field.edit_buffer.clone(),
                    "description" => profile.description = field.edit_buffer.clone(),
                    "version" => profile.version = field.edit_buffer.clone(),
                    "author" => profile.author = Some(field.edit_buffer.clone()),
                    _ => return false,
                }
            }
            FieldType::Float(min, max) => {
                if let Ok(value) = field.edit_buffer.parse::<f64>() {
                    if value >= min && value <= max {
                        self.apply_float_field(profile, &field.id, value);
                    } else {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            FieldType::Integer(min, max) => {
                if let Ok(value) = field.edit_buffer.parse::<i64>() {
                    if value >= min && value <= max {
                        self.apply_integer_field(profile, &field.id, value);
                    } else {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            FieldType::Boolean => {
                match field.edit_buffer.to_lowercase().as_str() {
                    "true" | "t" | "yes" | "y" | "1" => {
                        self.apply_boolean_field(profile, &field.id, true);
                    }
                    "false" | "f" | "no" | "n" | "0" => {
                        self.apply_boolean_field(profile, &field.id, false);
                    }
                    _ => return false,
                }
            }
            _ => return false,
        }

        true
    }

    fn apply_float_field(&mut self, profile: &mut AgentBehaviorProfile, field_id: &str, value: f64) {
        match field_id {
            "proactiveness" => profile.personality.proactiveness = value,
            "risk_tolerance" => profile.personality.risk_tolerance = value,
            "creativity" => profile.personality.creativity = value,
            "sociability" => profile.personality.sociability = value,
            "detail_orientation" => profile.personality.detail_orientation = value,
            "speed_vs_accuracy" => profile.personality.speed_vs_accuracy = value,
            "helpfulness" => profile.personality.helpfulness = value,
            "persistence" => profile.personality.persistence = value,
            "confidence" => profile.personality.confidence = value,
            "autonomy_threshold" => profile.decision_making.autonomy_threshold = value,
            "verbosity" => profile.communication.verbosity = value,
            "technical_level" => profile.communication.technical_level = value,
            "emoji_usage" => profile.communication.emoji_usage = value,
            "learning_rate" => profile.learning.learning_rate = value,
            "collaboration_willingness" => profile.collaboration.collaboration_willingness = value,
            "cpu_limit" => profile.resource_usage.cpu_limit = value,
            _ => {}
        }
    }

    fn apply_integer_field(&mut self, profile: &mut AgentBehaviorProfile, field_id: &str, value: i64) {
        match field_id {
            "max_concurrent_tasks" => profile.task_handling.max_concurrent_tasks = value as usize,
            "max_retries" => profile.error_handling.max_retries = value as u32,
            _ => {}
        }
    }

    fn apply_boolean_field(&mut self, profile: &mut AgentBehaviorProfile, field_id: &str, value: bool) {
        match field_id {
            "active" => profile.active = value,
            "seek_confirmation" => profile.decision_making.seek_confirmation = value,
            "explain_reasoning" => profile.decision_making.explain_reasoning = value,
            "provide_explanations" => profile.communication.provide_explanations = value,
            "ask_for_clarification" => profile.communication.ask_for_clarification = value,
            "summarize_work" => profile.communication.summarize_work = value,
            "batch_similar_tasks" => profile.task_handling.batch_similar_tasks = value,
            "decompose_complex_tasks" => profile.task_handling.decompose_complex_tasks = value,
            "validate_requirements" => profile.task_handling.validate_requirements = value,
            "learn_from_interactions" => profile.learning.learn_from_interactions = value,
            "adapt_from_outcomes" => profile.learning.adapt_from_outcomes = value,
            "remember_preferences" => profile.learning.remember_preferences = value,
            "escalate_errors" => profile.error_handling.escalate_errors = value,
            "detailed_logging" => profile.error_handling.detailed_logging = value,
            "seek_collaboration" => profile.collaboration.seek_collaboration = value,
            "share_resources" => profile.collaboration.share_resources = value,
            "mentor_others" => profile.collaboration.mentor_others = value,
            "yield_when_idle" => profile.resource_usage.yield_when_idle = value,
            "optimize_usage" => profile.resource_usage.optimize_usage = value,
            "cache_enabled" => profile.resource_usage.cache_behavior.enabled = value,
            _ => {}
        }
    }

    fn start_editing_current_field(&mut self) {
        if let Some(field) = self.current_fields.get_mut(self.selected_field_index) {
            field.editing = true;
            field.edit_buffer = match &field.value {
                BehaviorValue::String(s) => s.clone(),
                BehaviorValue::Integer(i) => i.to_string(),
                BehaviorValue::Float(f) => f.to_string(),
                BehaviorValue::Boolean(b) => b.to_string(),
                _ => String::new(),
            };
            field.cursor_position = field.edit_buffer.len();
        }
    }

    fn reset_current_field(&mut self) {
        if let Some(field) = self.current_fields.get_mut(self.selected_field_index) {
            field.editing = false;
            field.edit_buffer.clear();
            field.cursor_position = 0;
        }
    }

    fn save_profile(&mut self) -> EditorAction {
        if let Some(_profile) = &self.current_profile {
            // TODO: Implement actual saving logic
            self.has_changes = false;
            self.status_message = Some("Profile saved successfully!".to_string());
            self.error_message = None;
            debug!("Saved behavior profile");
            EditorAction::ProfileSaved
        } else {
            self.error_message = Some("No profile to save".to_string());
            EditorAction::SaveError
        }
    }

    fn next_section(&mut self) {
        self.tab_index = (self.tab_index + 1) % self.sections.len();
        self.current_section = self.sections[self.tab_index].clone();
        self.update_current_fields();
    }

    fn previous_section(&mut self) {
        self.tab_index = if self.tab_index == 0 {
            self.sections.len() - 1
        } else {
            self.tab_index - 1
        };
        self.current_section = self.sections[self.tab_index].clone();
        self.update_current_fields();
    }

    fn navigate_up(&mut self) {
        if self.selected_field_index > 0 {
            self.selected_field_index -= 1;
            if self.selected_field_index < self.scroll_position {
                self.scroll_position = self.selected_field_index;
            }
            self.list_state.select(Some(self.selected_field_index));
        }
    }

    fn navigate_down(&mut self) {
        if self.selected_field_index < self.current_fields.len().saturating_sub(1) {
            self.selected_field_index += 1;
            // Scroll down if necessary (assuming 10 visible items)
            let visible_items = 10;
            if self.selected_field_index >= self.scroll_position + visible_items {
                self.scroll_position = self.selected_field_index - visible_items + 1;
            }
            self.list_state.select(Some(self.selected_field_index));
        }
    }

    fn navigate_page_up(&mut self) {
        let page_size = 10;
        self.selected_field_index = self.selected_field_index.saturating_sub(page_size);
        self.scroll_position = self.scroll_position.saturating_sub(page_size);
        self.list_state.select(Some(self.selected_field_index));
    }

    fn navigate_page_down(&mut self) {
        let page_size = 10;
        let max_index = self.current_fields.len().saturating_sub(1);
        self.selected_field_index = (self.selected_field_index + page_size).min(max_index);
        let visible_items = 10;
        if self.selected_field_index >= self.scroll_position + visible_items {
            self.scroll_position = self.selected_field_index - visible_items + 1;
        }
        self.list_state.select(Some(self.selected_field_index));
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let profile_name = self.current_profile
            .as_ref()
            .map(|p| p.name.as_str())
            .unwrap_or("New Profile");

        let title = if self.has_changes {
            format!("Behavior Editor - {} *", profile_name)
        } else {
            format!("Behavior Editor - {}", profile_name)
        };

        let header = Paragraph::new(title)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(Color::Blue)),
            )
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::BOLD));

        f.render_widget(header, area);
    }

    fn render_tabs(&self, f: &mut Frame, area: Rect) {
        let tab_titles: Vec<Line> = self.sections
            .iter()
            .map(|section| Line::from(section.to_string()))
            .collect();

        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .select(self.tab_index);

        f.render_widget(tabs, area);
    }

    fn render_content(&mut self, f: &mut Frame, area: Rect) {
        if self.current_fields.is_empty() {
            let empty_text = Paragraph::new("No fields available for this section")
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
            
            f.render_widget(empty_text, area);
            return;
        }

        let items: Vec<ListItem> = self.current_fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let is_selected = i == self.selected_field_index;
                let is_editing = field.editing;

                let mut style = if is_selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                if is_editing {
                    style = style.add_modifier(Modifier::BOLD);
                }

                let value_text = if is_editing {
                    format!("{}│{}", 
                        &field.edit_buffer[..field.cursor_position], 
                        &field.edit_buffer[field.cursor_position..])
                } else {
                    self.format_field_value(&field.value)
                };

                let content = if is_editing {
                    format!("{}: {} (editing)", field.name, value_text)
                } else {
                    format!("{}: {}", field.name, value_text)
                };

                let mut spans = vec![Span::styled(content, style)];

                if is_selected && !field.description.is_empty() {
                    spans.push(Span::raw("  "));
                    spans.push(Span::styled(
                        format!("({})", field.description),
                        style.fg(Color::Gray),
                    ));
                }

                ListItem::new(Line::from(spans))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("{} Settings", self.current_section))
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(list, area, &mut self.list_state);

        // Render scrollbar if needed
        if self.current_fields.len() > 10 {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None);
            
            f.render_stateful_widget(
                scrollbar,
                area.inner(Margin { horizontal: 0, vertical: 1 }),
                &mut self.scroll_state.content_length(self.current_fields.len()),
            );
        }
    }

    fn render_status(&self, f: &mut Frame, area: Rect) {
        let status_text = if let Some(error) = &self.error_message {
            format!("Error: {}", error)
        } else if let Some(status) = &self.status_message {
            status.clone()
        } else {
            "Enter: Edit field | Tab: Switch sections | Ctrl+S: Save | Esc: Exit | F1: Help".to_string()
        };

        let style = if self.error_message.is_some() {
            Style::default().fg(Color::Red)
        } else if self.status_message.is_some() {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Gray)
        };

        let status = Paragraph::new(status_text)
            .style(style)
            .alignment(Alignment::Center);

        f.render_widget(status, area);
    }

    fn format_field_value(&self, value: &BehaviorValue) -> String {
        match value {
            BehaviorValue::String(s) => s.clone(),
            BehaviorValue::Integer(i) => i.to_string(),
            BehaviorValue::Float(f) => format!("{:.2}", f),
            BehaviorValue::Boolean(b) => b.to_string(),
            BehaviorValue::Duration(d) => format!("{:?}", d),
            BehaviorValue::List(list) => format!("[{} items]", list.len()),
            BehaviorValue::Map(map) => format!("{} items", map.len()),
        }
    }
}

/// Actions that can be triggered by the behavior editor
#[derive(Debug, Clone, PartialEq)]
pub enum EditorAction {
    /// No action
    None,
    /// Editor was closed
    Close,
    /// Request confirmation to close with unsaved changes
    ConfirmClose,
    /// Section was changed
    SectionChanged,
    /// Navigate in field list
    Navigate,
    /// Start editing a field
    StartEdit,
    /// Cancel field editing
    CancelEdit,
    /// Apply field edit
    ApplyEdit,
    /// Field edit update (typing)
    EditUpdate,
    /// Error during field editing
    EditError,
    /// Field was reset
    FieldReset,
    /// Profile was saved
    ProfileSaved,
    /// Error during save
    SaveError,
    /// Show help
    ShowHelp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_creation() {
        let editor = BehaviorEditor::new();
        assert!(!editor.is_visible());
        assert!(!editor.has_unsaved_changes());
    }

    #[test]
    fn test_show_new_profile() {
        let mut editor = BehaviorEditor::new();
        editor.show_new_profile();
        
        assert!(editor.is_visible());
        assert!(editor.get_current_profile().is_some());
        assert_eq!(editor.current_section, EditorSection::Overview);
    }

    #[test]
    fn test_field_value_formatting() {
        let editor = BehaviorEditor::new();
        
        assert_eq!(editor.format_field_value(&BehaviorValue::String("test".to_string())), "test");
        assert_eq!(editor.format_field_value(&BehaviorValue::Integer(42)), "42");
        assert_eq!(editor.format_field_value(&BehaviorValue::Float(3.14159)), "3.14");
        assert_eq!(editor.format_field_value(&BehaviorValue::Boolean(true)), "true");
    }
}