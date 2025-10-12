//! Enhanced Keyboard Shortcuts System
//!
//! This module provides comprehensive keyboard shortcut management with customizable
//! keybindings, context-aware shortcuts, help system, and multi-modal interaction.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use tracing::{debug, trace, warn};

/// Maximum number of keys in a chord sequence
const MAX_CHORD_LENGTH: usize = 4;

/// A keyboard shortcut definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct KeyShortcut {
    /// The key combination or chord sequence
    pub keys: Vec<KeyCombination>,
    /// Description of what this shortcut does
    pub description: String,
    /// Context where this shortcut is active
    pub context: ShortcutContext,
    /// Whether this shortcut can be customized by users
    pub customizable: bool,
    /// Category for organization
    pub category: String,
}

/// A single key combination (key + modifiers)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct KeyCombination {
    /// The key code
    pub key: KeyCode,
    /// Modifier keys (Ctrl, Alt, Shift)
    pub modifiers: KeyModifiers,
}

/// Context where a shortcut is active
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ShortcutContext {
    /// Global shortcuts (active everywhere)
    Global,
    /// Main application interface
    Main,
    /// Artifact viewer
    ArtifactViewer,
    /// Conversation history browser
    ConversationHistory,
    /// Search interface
    Search,
    /// Command palette
    CommandPalette,
    /// Text input fields
    TextInput,
    /// File browser
    FileBrowser,
    /// Settings/configuration
    Settings,
    /// Help system
    Help,
    /// Custom context (plugin-defined)
    Custom(String),
}

/// Action that a shortcut can trigger
#[derive(Debug, Clone, PartialEq)]
pub enum ShortcutAction {
    // Navigation
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    NavigateToTop,
    NavigateToBottom,
    NavigatePageUp,
    NavigatePageDown,
    NavigateBack,
    NavigateForward,
    
    // Selection and editing
    Select,
    SelectAll,
    Copy,
    Cut,
    Paste,
    Delete,
    Undo,
    Redo,
    
    // Application control
    OpenCommandPalette,
    OpenSearch,
    OpenSettings,
    OpenHelp,
    ToggleBookmark,
    ToggleFocus,
    SwitchTab,
    CloseTab,
    NewTab,
    
    // Artifact management
    CreateArtifact,
    ViewArtifact,
    EditArtifact,
    DeleteArtifact,
    ExportArtifact,
    CompareArtifacts,
    
    // Conversation management
    NewConversation,
    SaveConversation,
    LoadConversation,
    SearchConversations,
    BookmarkConversation,
    TagConversation,
    
    // View controls
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ToggleFullscreen,
    ToggleSidebar,
    ToggleStatusbar,
    SwitchViewMode,
    
    // System
    Quit,
    Save,
    Reload,
    Refresh,
    
    // Custom action (plugin-defined)
    Custom(String),
    
    // Easter egg: Vim escape sequences that actually work
    VimEscape(String),
}

/// Keyboard shortcut manager
pub struct ShortcutManager {
    /// All registered shortcuts
    shortcuts: HashMap<KeyShortcut, ShortcutAction>,
    /// Context-specific shortcut mappings
    context_shortcuts: HashMap<ShortcutContext, HashMap<Vec<KeyCombination>, ShortcutAction>>,
    /// Current active context
    active_context: ShortcutContext,
    /// Chord sequence in progress
    chord_sequence: Vec<KeyCombination>,
    /// Chord timeout in milliseconds
    chord_timeout_ms: u64,
    /// Last key press timestamp
    last_key_time: std::time::Instant,
    /// Whether shortcuts are enabled
    enabled: bool,
    /// User customizations
    custom_shortcuts: HashMap<KeyShortcut, ShortcutAction>,
    /// Vim sequence tracking for easter egg
    vim_sequence: Vec<char>,
}

/// Help information for shortcuts
#[derive(Debug, Clone)]
pub struct ShortcutHelp {
    /// Shortcuts organized by category
    pub categories: HashMap<String, Vec<ShortcutHelpItem>>,
    /// Context-specific help
    pub context_help: HashMap<ShortcutContext, Vec<ShortcutHelpItem>>,
}

/// Individual help item for a shortcut
#[derive(Debug, Clone)]
pub struct ShortcutHelpItem {
    /// Key combination display
    pub keys_display: String,
    /// Action description
    pub description: String,
    /// Action name
    pub action: ShortcutAction,
    /// Whether it's customizable
    pub customizable: bool,
}

/// Result of processing a key event
#[derive(Debug, Clone)]
pub enum ShortcutResult {
    /// No action triggered
    None,
    /// Action triggered immediately
    Action(ShortcutAction),
    /// Chord sequence in progress, waiting for more keys
    ChordInProgress(Vec<KeyCombination>),
    /// Chord sequence timed out
    ChordTimeout,
    /// Ambiguous shortcut (multiple matches)
    Ambiguous(Vec<ShortcutAction>),
}

impl ShortcutManager {
    /// Create a new shortcut manager with default shortcuts
    pub fn new() -> Self {
        let mut manager = Self {
            shortcuts: HashMap::new(),
            context_shortcuts: HashMap::new(),
            active_context: ShortcutContext::Main,
            chord_sequence: Vec::new(),
            chord_timeout_ms: 1000, // 1 second timeout
            last_key_time: std::time::Instant::now(),
            enabled: true,
            custom_shortcuts: HashMap::new(),
            vim_sequence: Vec::new(),
        };

        manager.register_default_shortcuts();
        manager.rebuild_context_mappings();
        manager
    }

    /// Process a key event and return the corresponding action
    pub fn process_key_event(&mut self, key_event: KeyEvent) -> ShortcutResult {
        if !self.enabled {
            return ShortcutResult::None;
        }

        // Easter egg: Check for Vim escape sequences first!
        if let Some(vim_action) = self.check_vim_escape(&key_event) {
            return ShortcutResult::Action(vim_action);
        }

        let now = std::time::Instant::now();
        let key_combo = KeyCombination {
            key: key_event.code,
            modifiers: key_event.modifiers,
        };

        // Check if chord sequence timed out
        if !self.chord_sequence.is_empty() {
            let elapsed = now.duration_since(self.last_key_time).as_millis() as u64;
            if elapsed > self.chord_timeout_ms {
                self.chord_sequence.clear();
                return ShortcutResult::ChordTimeout;
            }
        }

        // Add to chord sequence
        self.chord_sequence.push(key_combo);
        self.last_key_time = now;

        // Look for exact matches
        if let Some(context_map) = self.context_shortcuts.get(&self.active_context) {
            if let Some(action) = context_map.get(&self.chord_sequence) {
                let result_action = action.clone();
                self.chord_sequence.clear();
                trace!("Triggered shortcut action: {:?}", result_action);
                return ShortcutResult::Action(result_action);
            }
        }

        // Also check global context
        if self.active_context != ShortcutContext::Global {
            if let Some(global_map) = self.context_shortcuts.get(&ShortcutContext::Global) {
                if let Some(action) = global_map.get(&self.chord_sequence) {
                    let result_action = action.clone();
                    self.chord_sequence.clear();
                    trace!("Triggered global shortcut action: {:?}", result_action);
                    return ShortcutResult::Action(result_action);
                }
            }
        }

        // Check for partial matches (chord in progress)
        let partial_matches = self.find_partial_matches(&self.chord_sequence);
        if !partial_matches.is_empty() {
            if self.chord_sequence.len() >= MAX_CHORD_LENGTH {
                // Too many keys, reset
                self.chord_sequence.clear();
                return ShortcutResult::None;
            }
            return ShortcutResult::ChordInProgress(self.chord_sequence.clone());
        }

        // No matches, reset chord sequence
        self.chord_sequence.clear();
        ShortcutResult::None
    }

    /// Set the active context for shortcuts
    pub fn set_context(&mut self, context: ShortcutContext) {
        if self.active_context != context {
            self.active_context = context;
            self.chord_sequence.clear(); // Reset any ongoing chord
            debug!("Switched shortcut context to: {:?}", self.active_context);
        }
    }

    /// Get the current active context
    pub fn get_context(&self) -> &ShortcutContext {
        &self.active_context
    }

    /// Enable or disable shortcuts
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.chord_sequence.clear();
        }
    }

    /// Register a new shortcut
    pub fn register_shortcut(
        &mut self,
        shortcut: KeyShortcut,
        action: ShortcutAction,
    ) -> Result<(), String> {
        // Validate shortcut
        if shortcut.keys.is_empty() {
            return Err("Shortcut must have at least one key combination".to_string());
        }

        if shortcut.keys.len() > MAX_CHORD_LENGTH {
            return Err(format!("Shortcut cannot have more than {} keys", MAX_CHORD_LENGTH));
        }

        // Check for conflicts
        if let Some(existing_action) = self.shortcuts.get(&shortcut) {
            if *existing_action != action {
                return Err(format!("Shortcut conflicts with existing binding for: {:?}", existing_action));
            }
        }

        // Register shortcut
        self.shortcuts.insert(shortcut, action);
        self.rebuild_context_mappings();
        
        Ok(())
    }

    /// Easter egg: Check for Vim escape sequences and provide helpful messages
    fn check_vim_escape(&mut self, key_event: &KeyEvent) -> Option<ShortcutAction> {
        // Only track non-control characters for vim sequences
        if !key_event.modifiers.contains(KeyModifiers::CONTROL) {
            if let KeyCode::Char(c) = key_event.code {
                self.vim_sequence.push(c);
                
                // Keep only the last 10 characters to prevent memory issues
                if self.vim_sequence.len() > 10 {
                    self.vim_sequence.drain(0..1);
                }
                
                let sequence: String = self.vim_sequence.iter().collect();
                
                // Check for various vim escape sequences
                if sequence.ends_with(":q!") {
                    self.vim_sequence.clear();
                    return Some(ShortcutAction::VimEscape(
                        "🎉 Finally escaping Vim after 7 years! Welcome to DevKit where Ctrl+Q actually works!".to_string()
                    ));
                } else if sequence.ends_with(":wq") {
                    self.vim_sequence.clear();
                    return Some(ShortcutAction::VimEscape(
                        "💾 Saving and quitting like a Vim pro! But you're in DevKit now - try Ctrl+S instead! 😄".to_string()
                    ));
                } else if sequence.ends_with("ZZ") {
                    self.vim_sequence.clear();
                    return Some(ShortcutAction::VimEscape(
                        "🎩 Fancy Vim exit detected! DevKit respects the classics (but makes them actually usable)".to_string()
                    ));
                } else if sequence.ends_with(":x") {
                    self.vim_sequence.clear();
                    return Some(ShortcutAction::VimEscape(
                        "⚡ Quick Vim exit! In DevKit, everything is quick - no cryptic commands needed!".to_string()
                    ));
                } else if sequence.ends_with(":help") {
                    self.vim_sequence.clear();
                    return Some(ShortcutAction::VimEscape(
                        "📚 Looking for help? In DevKit, just press Ctrl+H - no ':' required!".to_string()
                    ));
                }
            }
        }
        None
    }

    /// Unregister a shortcut
    pub fn unregister_shortcut(&mut self, shortcut: &KeyShortcut) -> bool {
        let removed = self.shortcuts.remove(shortcut).is_some();
        if removed {
            self.rebuild_context_mappings();
        }
        removed
    }

    /// Customize an existing shortcut
    pub fn customize_shortcut(
        &mut self,
        old_shortcut: &KeyShortcut,
        new_keys: Vec<KeyCombination>,
    ) -> Result<(), String> {
        // Find the action for the old shortcut
        let action = self.shortcuts.get(old_shortcut)
            .ok_or("Shortcut not found")?
            .clone();

        // Check if the old shortcut is customizable
        if !old_shortcut.customizable {
            return Err("Shortcut is not customizable".to_string());
        }

        // Create new shortcut
        let mut new_shortcut = old_shortcut.clone();
        new_shortcut.keys = new_keys;

        // Remove old binding
        self.shortcuts.remove(old_shortcut);

        // Add new binding
        self.shortcuts.insert(new_shortcut.clone(), action.clone());
        self.custom_shortcuts.insert(new_shortcut, action);

        self.rebuild_context_mappings();
        Ok(())
    }

    /// Get help information for shortcuts
    pub fn get_help(&self) -> ShortcutHelp {
        let mut categories: HashMap<String, Vec<ShortcutHelpItem>> = HashMap::new();
        let mut context_help: HashMap<ShortcutContext, Vec<ShortcutHelpItem>> = HashMap::new();

        for (shortcut, action) in &self.shortcuts {
            let help_item = ShortcutHelpItem {
                keys_display: self.format_key_sequence(&shortcut.keys),
                description: shortcut.description.clone(),
                action: action.clone(),
                customizable: shortcut.customizable,
            };

            // Add to category
            categories.entry(shortcut.category.clone())
                .or_insert_with(Vec::new)
                .push(help_item.clone());

            // Add to context
            context_help.entry(shortcut.context.clone())
                .or_insert_with(Vec::new)
                .push(help_item);
        }

        ShortcutHelp {
            categories,
            context_help,
        }
    }

    /// Get shortcuts for a specific context
    pub fn get_context_shortcuts(&self, context: &ShortcutContext) -> Vec<(Vec<KeyCombination>, ShortcutAction)> {
        self.context_shortcuts.get(context)
            .map(|map| map.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default()
    }

    /// Format key sequence for display
    pub fn format_key_sequence(&self, keys: &[KeyCombination]) -> String {
        keys.iter()
            .map(|combo| self.format_key_combination(combo))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Format a single key combination for display
    pub fn format_key_combination(&self, combo: &KeyCombination) -> String {
        let mut parts = Vec::new();

        if combo.modifiers.contains(KeyModifiers::CONTROL) {
            parts.push("Ctrl");
        }
        if combo.modifiers.contains(KeyModifiers::ALT) {
            parts.push("Alt");
        }
        if combo.modifiers.contains(KeyModifiers::SHIFT) {
            parts.push("Shift");
        }

        parts.push(&self.format_key_code(&combo.key));
        parts.join("+")
    }

    /// Format a key code for display
    fn format_key_code(&self, key: &KeyCode) -> String {
        match key {
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::PageUp => "PgUp".to_string(),
            KeyCode::PageDown => "PgDn".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Delete => "Del".to_string(),
            KeyCode::Insert => "Ins".to_string(),
            KeyCode::F(n) => format!("F{}", n),
            KeyCode::Char(c) => c.to_uppercase().to_string(),
            KeyCode::Esc => "Esc".to_string(),
            _ => format!("{:?}", key),
        }
    }

    /// Register default shortcuts
    fn register_default_shortcuts(&mut self) {
        let shortcuts = [
            // Global shortcuts
            (
                KeyShortcut {
                    keys: vec![KeyCombination {
                        key: KeyCode::Char('q'),
                        modifiers: KeyModifiers::CONTROL,
                    }],
                    description: "Quit application".to_string(),
                    context: ShortcutContext::Global,
                    customizable: false,
                    category: "Application".to_string(),
                },
                ShortcutAction::Quit,
            ),
            (
                KeyShortcut {
                    keys: vec![KeyCombination {
                        key: KeyCode::Char('p'),
                        modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                    }],
                    description: "Open command palette".to_string(),
                    context: ShortcutContext::Global,
                    customizable: true,
                    category: "Navigation".to_string(),
                },
                ShortcutAction::OpenCommandPalette,
            ),
            (
                KeyShortcut {
                    keys: vec![KeyCombination {
                        key: KeyCode::Char('f'),
                        modifiers: KeyModifiers::CONTROL,
                    }],
                    description: "Open search".to_string(),
                    context: ShortcutContext::Global,
                    customizable: true,
                    category: "Search".to_string(),
                },
                ShortcutAction::OpenSearch,
            ),
            (
                KeyShortcut {
                    keys: vec![KeyCombination {
                        key: KeyCode::F(1),
                        modifiers: KeyModifiers::NONE,
                    }],
                    description: "Show help".to_string(),
                    context: ShortcutContext::Global,
                    customizable: true,
                    category: "Help".to_string(),
                },
                ShortcutAction::OpenHelp,
            ),

            // Navigation shortcuts
            (
                KeyShortcut {
                    keys: vec![KeyCombination {
                        key: KeyCode::Up,
                        modifiers: KeyModifiers::NONE,
                    }],
                    description: "Navigate up".to_string(),
                    context: ShortcutContext::Main,
                    customizable: true,
                    category: "Navigation".to_string(),
                },
                ShortcutAction::NavigateUp,
            ),
            (
                KeyShortcut {
                    keys: vec![KeyCombination {
                        key: KeyCode::Down,
                        modifiers: KeyModifiers::NONE,
                    }],
                    description: "Navigate down".to_string(),
                    context: ShortcutContext::Main,
                    customizable: true,
                    category: "Navigation".to_string(),
                },
                ShortcutAction::NavigateDown,
            ),
            (
                KeyShortcut {
                    keys: vec![KeyCombination {
                        key: KeyCode::Left,
                        modifiers: KeyModifiers::NONE,
                    }],
                    description: "Navigate left".to_string(),
                    context: ShortcutContext::Main,
                    customizable: true,
                    category: "Navigation".to_string(),
                },
                ShortcutAction::NavigateLeft,
            ),
            (
                KeyShortcut {
                    keys: vec![KeyCombination {
                        key: KeyCode::Right,
                        modifiers: KeyModifiers::NONE,
                    }],
                    description: "Navigate right".to_string(),
                    context: ShortcutContext::Main,
                    customizable: true,
                    category: "Navigation".to_string(),
                },
                ShortcutAction::NavigateRight,
            ),

            // Editing shortcuts
            (
                KeyShortcut {
                    keys: vec![KeyCombination {
                        key: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                    }],
                    description: "Copy".to_string(),
                    context: ShortcutContext::Main,
                    customizable: true,
                    category: "Edit".to_string(),
                },
                ShortcutAction::Copy,
            ),
            (
                KeyShortcut {
                    keys: vec![KeyCombination {
                        key: KeyCode::Char('v'),
                        modifiers: KeyModifiers::CONTROL,
                    }],
                    description: "Paste".to_string(),
                    context: ShortcutContext::Main,
                    customizable: true,
                    category: "Edit".to_string(),
                },
                ShortcutAction::Paste,
            ),
            (
                KeyShortcut {
                    keys: vec![KeyCombination {
                        key: KeyCode::Char('z'),
                        modifiers: KeyModifiers::CONTROL,
                    }],
                    description: "Undo".to_string(),
                    context: ShortcutContext::Main,
                    customizable: true,
                    category: "Edit".to_string(),
                },
                ShortcutAction::Undo,
            ),

            // Artifact management shortcuts
            (
                KeyShortcut {
                    keys: vec![KeyCombination {
                        key: KeyCode::Char('n'),
                        modifiers: KeyModifiers::CONTROL,
                    }],
                    description: "Create new artifact".to_string(),
                    context: ShortcutContext::ArtifactViewer,
                    customizable: true,
                    category: "Artifacts".to_string(),
                },
                ShortcutAction::CreateArtifact,
            ),
            (
                KeyShortcut {
                    keys: vec![KeyCombination {
                        key: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE,
                    }],
                    description: "View artifact".to_string(),
                    context: ShortcutContext::ArtifactViewer,
                    customizable: true,
                    category: "Artifacts".to_string(),
                },
                ShortcutAction::ViewArtifact,
            ),
            (
                KeyShortcut {
                    keys: vec![KeyCombination {
                        key: KeyCode::Char('e'),
                        modifiers: KeyModifiers::NONE,
                    }],
                    description: "Edit artifact".to_string(),
                    context: ShortcutContext::ArtifactViewer,
                    customizable: true,
                    category: "Artifacts".to_string(),
                },
                ShortcutAction::EditArtifact,
            ),
            (
                KeyShortcut {
                    keys: vec![KeyCombination {
                        key: KeyCode::Delete,
                        modifiers: KeyModifiers::NONE,
                    }],
                    description: "Delete artifact".to_string(),
                    context: ShortcutContext::ArtifactViewer,
                    customizable: true,
                    category: "Artifacts".to_string(),
                },
                ShortcutAction::DeleteArtifact,
            ),

            // Conversation management shortcuts
            (
                KeyShortcut {
                    keys: vec![KeyCombination {
                        key: KeyCode::Char('b'),
                        modifiers: KeyModifiers::CONTROL,
                    }],
                    description: "Bookmark conversation".to_string(),
                    context: ShortcutContext::ConversationHistory,
                    customizable: true,
                    category: "Conversations".to_string(),
                },
                ShortcutAction::BookmarkConversation,
            ),
            (
                KeyShortcut {
                    keys: vec![KeyCombination {
                        key: KeyCode::Char('t'),
                        modifiers: KeyModifiers::CONTROL,
                    }],
                    description: "Tag conversation".to_string(),
                    context: ShortcutContext::ConversationHistory,
                    customizable: true,
                    category: "Conversations".to_string(),
                },
                ShortcutAction::TagConversation,
            ),

            // View control shortcuts
            (
                KeyShortcut {
                    keys: vec![KeyCombination {
                        key: KeyCode::Tab,
                        modifiers: KeyModifiers::NONE,
                    }],
                    description: "Switch tab".to_string(),
                    context: ShortcutContext::Main,
                    customizable: true,
                    category: "View".to_string(),
                },
                ShortcutAction::SwitchTab,
            ),
            (
                KeyShortcut {
                    keys: vec![KeyCombination {
                        key: KeyCode::F(11),
                        modifiers: KeyModifiers::NONE,
                    }],
                    description: "Toggle fullscreen".to_string(),
                    context: ShortcutContext::Global,
                    customizable: true,
                    category: "View".to_string(),
                },
                ShortcutAction::ToggleFullscreen,
            ),

            // Multi-key chord examples
            (
                KeyShortcut {
                    keys: vec![
                        KeyCombination {
                            key: KeyCode::Char('g'),
                            modifiers: KeyModifiers::NONE,
                        },
                        KeyCombination {
                            key: KeyCode::Char('g'),
                            modifiers: KeyModifiers::NONE,
                        },
                    ],
                    description: "Go to top".to_string(),
                    context: ShortcutContext::Main,
                    customizable: true,
                    category: "Navigation".to_string(),
                },
                ShortcutAction::NavigateToTop,
            ),
            (
                KeyShortcut {
                    keys: vec![
                        KeyCombination {
                            key: KeyCode::Char('g'),
                            modifiers: KeyModifiers::NONE,
                        },
                        KeyCombination {
                            key: KeyCode::Char('e'),
                            modifiers: KeyModifiers::NONE,
                        },
                    ],
                    description: "Go to bottom".to_string(),
                    context: ShortcutContext::Main,
                    customizable: true,
                    category: "Navigation".to_string(),
                },
                ShortcutAction::NavigateToBottom,
            ),
        ];

        for (shortcut, action) in shortcuts.iter() {
            self.shortcuts.insert(shortcut.clone(), action.clone());
        }
    }

    /// Rebuild context-specific mappings for faster lookups
    fn rebuild_context_mappings(&mut self) {
        self.context_shortcuts.clear();

        for (shortcut, action) in &self.shortcuts {
            let context_map = self.context_shortcuts
                .entry(shortcut.context.clone())
                .or_insert_with(HashMap::new);
            
            context_map.insert(shortcut.keys.clone(), action.clone());
        }
    }

    /// Find partial matches for chord sequences
    fn find_partial_matches(&self, sequence: &[KeyCombination]) -> Vec<Vec<KeyCombination>> {
        let mut matches = Vec::new();

        // Check current context
        if let Some(context_map) = self.context_shortcuts.get(&self.active_context) {
            for key_sequence in context_map.keys() {
                if key_sequence.len() > sequence.len() && 
                   key_sequence[..sequence.len()] == *sequence {
                    matches.push(key_sequence.clone());
                }
            }
        }

        // Check global context if different
        if self.active_context != ShortcutContext::Global {
            if let Some(global_map) = self.context_shortcuts.get(&ShortcutContext::Global) {
                for key_sequence in global_map.keys() {
                    if key_sequence.len() > sequence.len() && 
                       key_sequence[..sequence.len()] == *sequence {
                        matches.push(key_sequence.clone());
                    }
                }
            }
        }

        matches
    }
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ShortcutContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShortcutContext::Global => write!(f, "Global"),
            ShortcutContext::Main => write!(f, "Main"),
            ShortcutContext::ArtifactViewer => write!(f, "Artifact Viewer"),
            ShortcutContext::ConversationHistory => write!(f, "Conversation History"),
            ShortcutContext::Search => write!(f, "Search"),
            ShortcutContext::CommandPalette => write!(f, "Command Palette"),
            ShortcutContext::TextInput => write!(f, "Text Input"),
            ShortcutContext::FileBrowser => write!(f, "File Browser"),
            ShortcutContext::Settings => write!(f, "Settings"),
            ShortcutContext::Help => write!(f, "Help"),
            ShortcutContext::Custom(name) => write!(f, "Custom: {}", name),
        }
    }
}

impl fmt::Display for ShortcutAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShortcutAction::NavigateUp => write!(f, "Navigate Up"),
            ShortcutAction::NavigateDown => write!(f, "Navigate Down"),
            ShortcutAction::NavigateLeft => write!(f, "Navigate Left"),
            ShortcutAction::NavigateRight => write!(f, "Navigate Right"),
            ShortcutAction::NavigateToTop => write!(f, "Go to Top"),
            ShortcutAction::NavigateToBottom => write!(f, "Go to Bottom"),
            ShortcutAction::NavigatePageUp => write!(f, "Page Up"),
            ShortcutAction::NavigatePageDown => write!(f, "Page Down"),
            ShortcutAction::NavigateBack => write!(f, "Go Back"),
            ShortcutAction::NavigateForward => write!(f, "Go Forward"),
            ShortcutAction::OpenCommandPalette => write!(f, "Open Command Palette"),
            ShortcutAction::OpenSearch => write!(f, "Open Search"),
            ShortcutAction::OpenHelp => write!(f, "Show Help"),
            ShortcutAction::Quit => write!(f, "Quit Application"),
            ShortcutAction::Copy => write!(f, "Copy"),
            ShortcutAction::Paste => write!(f, "Paste"),
            ShortcutAction::CreateArtifact => write!(f, "Create Artifact"),
            ShortcutAction::ViewArtifact => write!(f, "View Artifact"),
            ShortcutAction::BookmarkConversation => write!(f, "Bookmark Conversation"),
            ShortcutAction::ToggleFullscreen => write!(f, "Toggle Fullscreen"),
            ShortcutAction::Custom(name) => write!(f, "Custom: {}", name),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl FromStr for KeyCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "backspace" => Ok(KeyCode::Backspace),
            "enter" => Ok(KeyCode::Enter),
            "left" => Ok(KeyCode::Left),
            "right" => Ok(KeyCode::Right),
            "up" => Ok(KeyCode::Up),
            "down" => Ok(KeyCode::Down),
            "home" => Ok(KeyCode::Home),
            "end" => Ok(KeyCode::End),
            "pageup" | "pgup" => Ok(KeyCode::PageUp),
            "pagedown" | "pgdn" => Ok(KeyCode::PageDown),
            "tab" => Ok(KeyCode::Tab),
            "delete" | "del" => Ok(KeyCode::Delete),
            "insert" | "ins" => Ok(KeyCode::Insert),
            "escape" | "esc" => Ok(KeyCode::Esc),
            "space" => Ok(KeyCode::Char(' ')),
            s if s.starts_with('f') && s.len() > 1 => {
                if let Ok(n) = s[1..].parse::<u8>() {
                    if n >= 1 && n <= 12 {
                        Ok(KeyCode::F(n))
                    } else {
                        Err(format!("Invalid function key: {}", s))
                    }
                } else {
                    Err(format!("Invalid function key: {}", s))
                }
            }
            s if s.len() == 1 => {
                Ok(KeyCode::Char(s.chars().next().unwrap().to_ascii_lowercase()))
            }
            _ => Err(format!("Unknown key: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_registration() {
        let mut manager = ShortcutManager::new();
        
        let shortcut = KeyShortcut {
            keys: vec![KeyCombination {
                key: KeyCode::Char('x'),
                modifiers: KeyModifiers::CONTROL,
            }],
            description: "Test shortcut".to_string(),
            context: ShortcutContext::Main,
            customizable: true,
            category: "Test".to_string(),
        };

        assert!(manager.register_shortcut(shortcut, ShortcutAction::Custom("test".to_string())).is_ok());
    }

    #[test]
    fn test_key_formatting() {
        let manager = ShortcutManager::new();
        
        let combo = KeyCombination {
            key: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        };

        let formatted = manager.format_key_combination(&combo);
        assert!(formatted.contains("Ctrl"));
        assert!(formatted.contains("Shift"));
        assert!(formatted.contains("C"));
    }

    #[test]
    fn test_chord_sequence() {
        let mut manager = ShortcutManager::new();
        
        // First key in chord
        let result1 = manager.process_key_event(KeyEvent {
            code: KeyCode::Char('g'),
            modifiers: KeyModifiers::NONE,
        });
        
        match result1 {
            ShortcutResult::ChordInProgress(_) => {
                // Second key in chord
                let result2 = manager.process_key_event(KeyEvent {
                    code: KeyCode::Char('g'),
                    modifiers: KeyModifiers::NONE,
                });
                
                assert!(matches!(result2, ShortcutResult::Action(ShortcutAction::NavigateToTop)));
            }
            _ => panic!("Expected chord in progress"),
        }
    }
}