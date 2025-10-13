//! Multi-Agent Coordination Visualization System
//!
//! This module provides comprehensive real-time visualization of multi-agent
//! coordination, including agent interactions, task flows, system state,
//! resource usage, and collaboration patterns. It offers multiple visualization
//! modes including network graphs, timeline views, and dashboard layouts.

use crate::agents::{AgentStatus, TaskPriority, AgentMetrics};
// use crate::session::{Session, AgentSessionInfo, TaskInfo}; // Temporarily disabled - unused imports
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{
        Block, BorderType, Borders, Clear, Gauge, 
        List, ListItem, ListState, Paragraph, ScrollbarState, Tabs,
        canvas::Canvas,
    },
    Frame,
};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime};
use tracing::{debug, trace};

/// Maximum number of data points to keep in memory for charts
const MAX_DATA_POINTS: usize = 1000;
/// Update interval for real-time data
const UPDATE_INTERVAL: Duration = Duration::from_millis(100);
/// Maximum number of interaction events to track
const MAX_INTERACTION_HISTORY: usize = 500;

/// Multi-agent coordination visualization system
#[derive(Debug)]
pub struct CoordinationVisualizer {
    /// Current visualization mode
    mode: VisualizationMode,
    /// Available visualization modes
    modes: Vec<VisualizationMode>,
    /// Current mode index
    mode_index: usize,
    /// Agent network graph
    network_graph: AgentNetworkGraph,
    /// Task flow visualization
    task_flow: TaskFlowVisualizer,
    /// Resource monitoring
    resource_monitor: ResourceMonitor,
    /// Timeline view
    timeline: TimelineView,
    /// Analytics dashboard
    dashboard: AnalyticsDashboard,
    /// System state tracker
    state_tracker: SystemStateTracker,
    /// Interaction history
    interaction_history: VecDeque<AgentInteraction>,
    /// Selected agent for detailed view
    selected_agent: Option<String>,
    /// View settings and preferences
    settings: VisualizationSettings,
    /// Last update timestamp
    last_update: Instant,
    /// Whether the visualizer is visible
    visible: bool,
    /// Current data snapshot
    current_snapshot: SystemSnapshot,
    /// Historical data for trends
    historical_data: VecDeque<SystemSnapshot>,
    /// UI state management
    ui_state: UIState,
}

/// Different visualization modes
#[derive(Debug, Clone, PartialEq)]
pub enum VisualizationMode {
    /// Network graph showing agent connections and interactions
    NetworkGraph,
    /// Task flow and dependency visualization
    TaskFlow,
    /// Resource usage monitoring
    ResourceMonitor,
    /// Timeline of events and activities
    Timeline,
    /// Analytics dashboard with metrics
    Dashboard,
    /// Combined overview with multiple panels
    Overview,
}

impl std::fmt::Display for VisualizationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VisualizationMode::NetworkGraph => write!(f, "Network"),
            VisualizationMode::TaskFlow => write!(f, "Task Flow"),
            VisualizationMode::ResourceMonitor => write!(f, "Resources"),
            VisualizationMode::Timeline => write!(f, "Timeline"),
            VisualizationMode::Dashboard => write!(f, "Dashboard"),
            VisualizationMode::Overview => write!(f, "Overview"),
        }
    }
}

/// Visualization settings and preferences
#[derive(Debug, Clone)]
pub struct VisualizationSettings {
    /// Update frequency for real-time data
    pub update_frequency: Duration,
    /// Show agent labels
    pub show_labels: bool,
    /// Show connection strengths
    pub show_connection_strength: bool,
    /// Animation enabled
    pub animations_enabled: bool,
    /// Color scheme
    pub color_scheme: ColorScheme,
    /// Show performance metrics
    pub show_metrics: bool,
    /// Maximum history to display
    pub max_history: usize,
    /// Auto-scale charts
    pub auto_scale: bool,
    /// Show grid lines
    pub show_grid: bool,
    /// Transparency level (0.0 to 1.0)
    pub transparency: f64,
}

/// Color schemes for visualization
#[derive(Debug, Clone, PartialEq)]
pub enum ColorScheme {
    Default,
    HighContrast,
    Monochrome,
    Vibrant,
    Dark,
    Light,
}

/// UI state management
#[derive(Debug, Clone)]
pub struct UIState {
    /// Current focus/selection
    pub focused_element: Option<String>,
    /// Scroll positions
    pub scroll_positions: HashMap<String, usize>,
    /// List states
    pub list_states: HashMap<String, ListState>,
    /// Scrollbar states
    pub scrollbar_states: HashMap<String, ScrollbarState>,
    /// Panel sizes
    pub panel_sizes: HashMap<String, Rect>,
    /// Animation states
    pub animation_states: HashMap<String, f64>,
}

/// System snapshot for visualization
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemSnapshot {
    /// Timestamp of the snapshot (skipped in serialization)
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
    /// System time (skipped in serialization) 
    #[serde(skip, default = "SystemTime::now")]
    pub system_time: SystemTime,
    /// Agent states
    pub agents: HashMap<String, AgentState>,
    /// Task states
    pub tasks: HashMap<String, TaskState>,
    /// Resource usage
    pub resources: ResourceUsage,
    /// System metrics
    pub metrics: SystemMetrics,
    /// Active interactions
    pub interactions: Vec<AgentInteraction>,
}

/// Agent state for visualization
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentState {
    /// Agent identifier
    pub id: String,
    /// Agent name
    pub name: String,
    /// Agent type
    pub agent_type: String,
    /// Current status
    pub status: AgentStatus,
    /// Position in visualization (x, y)
    pub position: (f64, f64),
    /// Current task (if any)
    pub current_task: Option<String>,
    /// Performance metrics
    pub metrics: AgentMetrics,
    /// Connection strength to other agents
    pub connections: HashMap<String, f64>,
    /// Recent activity level (0.0 to 1.0)
    pub activity_level: f64,
    /// Health status (0.0 to 1.0)
    pub health: f64,
    /// Load level (0.0 to 1.0)
    pub load: f64,
}

/// Task state for visualization
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskState {
    /// Task identifier
    pub id: String,
    /// Task description
    pub description: String,
    /// Task priority
    pub priority: TaskPriority,
    /// Current progress (0.0 to 1.0)
    pub progress: f64,
    /// Assigned agent
    pub assigned_agent: Option<String>,
    /// Task dependencies
    pub dependencies: Vec<String>,
    /// Start time (skipped in serialization)
    #[serde(skip, default = "Instant::now")]
    pub started_at: Instant,
    /// Estimated completion time (skipped in serialization)
    #[serde(skip, default)]
    pub estimated_completion: Option<Instant>,
    /// Task status
    pub status: TaskStatus,
}

/// Task status for visualization
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Blocked,
}

/// Resource usage information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceUsage {
    /// CPU usage percentage (0.0 to 1.0)
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Total available memory
    pub memory_total: u64,
    /// Network I/O bytes per second
    pub network_io: u64,
    /// Disk I/O bytes per second
    pub disk_io: u64,
    /// Number of active threads
    pub active_threads: usize,
    /// Number of database connections
    pub db_connections: usize,
    /// Queue sizes
    pub queue_sizes: HashMap<String, usize>,
}

/// System-wide metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemMetrics {
    /// Total tasks processed
    pub total_tasks: usize,
    /// Tasks per second
    pub tasks_per_second: f64,
    /// Average response time in milliseconds
    pub avg_response_time: f64,
    /// Error rate percentage
    pub error_rate: f64,
    /// System uptime
    pub uptime: Duration,
    /// Active sessions
    pub active_sessions: usize,
    /// Total agents
    pub total_agents: usize,
    /// Active agents
    pub active_agents: usize,
}

/// Agent interaction event
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentInteraction {
    /// Unique interaction identifier
    pub id: String,
    /// Source agent
    pub from_agent: String,
    /// Target agent
    pub to_agent: String,
    /// Interaction type
    pub interaction_type: InteractionType,
    /// Timestamp (skipped in serialization)
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
    /// Duration (if applicable)
    pub duration: Option<Duration>,
    /// Data transferred (bytes)
    pub data_size: Option<u64>,
    /// Success/failure status
    pub success: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Types of agent interactions
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum InteractionType {
    /// Task delegation
    TaskDelegation,
    /// Data sharing
    DataSharing,
    /// Status update
    StatusUpdate,
    /// Resource request
    ResourceRequest,
    /// Collaboration request
    CollaborationRequest,
    /// Error notification
    ErrorNotification,
    /// Completion notification
    CompletionNotification,
    /// Health check
    HealthCheck,
}

/// Agent network graph visualization
#[derive(Debug)]
pub struct AgentNetworkGraph {
    /// Graph layout algorithm
    layout_algorithm: LayoutAlgorithm,
    /// Node positions
    node_positions: HashMap<String, (f64, f64)>,
    /// Edge weights (connection strengths)
    edge_weights: HashMap<(String, String), f64>,
    /// Graph bounds
    bounds: GraphBounds,
    /// Animation state
    animation_state: f64,
    /// Selected node
    selected_node: Option<String>,
    /// Hover state
    hover_node: Option<String>,
}

/// Graph layout algorithms
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutAlgorithm {
    /// Force-directed layout
    ForceDirected,
    /// Circular layout
    Circular,
    /// Hierarchical layout
    Hierarchical,
    /// Grid layout
    Grid,
    /// Custom manual layout
    Manual,
}

/// Graph bounds for rendering
#[derive(Debug, Clone)]
pub struct GraphBounds {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

/// Task flow visualizer
#[derive(Debug)]
pub struct TaskFlowVisualizer {
    /// Current tasks being tracked
    tasks: HashMap<String, TaskFlowNode>,
    /// Task dependencies
    dependencies: HashMap<String, Vec<String>>,
    /// Flow layout
    layout: FlowLayout,
    /// Animation states
    animations: HashMap<String, f64>,
}

/// Task flow node
#[derive(Debug, Clone)]
pub struct TaskFlowNode {
    /// Task information
    pub task: TaskState,
    /// Visual position
    pub position: (f64, f64),
    /// Visual size
    pub size: (f64, f64),
    /// Animation phase
    pub animation_phase: f64,
}

/// Flow layout settings
#[derive(Debug, Clone)]
pub struct FlowLayout {
    /// Layout direction
    pub direction: FlowDirection,
    /// Node spacing
    pub node_spacing: f64,
    /// Layer spacing
    pub layer_spacing: f64,
    /// Auto-arrange enabled
    pub auto_arrange: bool,
}

/// Flow direction options
#[derive(Debug, Clone, PartialEq)]
pub enum FlowDirection {
    LeftToRight,
    TopToBottom,
    Circular,
    Radial,
}

/// Resource monitoring component
#[derive(Debug)]
pub struct ResourceMonitor {
    /// CPU usage history
    cpu_history: VecDeque<(Instant, f64)>,
    /// Memory usage history
    memory_history: VecDeque<(Instant, f64)>,
    /// Network I/O history
    network_history: VecDeque<(Instant, f64)>,
    /// Disk I/O history
    disk_history: VecDeque<(Instant, f64)>,
    /// Chart settings
    chart_settings: ChartSettings,
}

/// Chart configuration settings
#[derive(Debug, Clone)]
pub struct ChartSettings {
    /// Time window for charts
    pub time_window: Duration,
    /// Chart colors
    pub colors: Vec<Color>,
    /// Show data points
    pub show_points: bool,
    /// Smooth lines
    pub smooth_lines: bool,
    /// Auto-scale Y axis
    pub auto_scale_y: bool,
    /// Show legend
    pub show_legend: bool,
}

/// Timeline view component
#[derive(Debug)]
pub struct TimelineView {
    /// Timeline events
    events: VecDeque<TimelineEvent>,
    /// Current time window
    time_window: Duration,
    /// Zoom level
    zoom_level: f64,
    /// Scroll position
    scroll_position: f64,
    /// Selected event
    selected_event: Option<String>,
    /// Event filters
    filters: TimelineFilters,
}

/// Timeline event representation
#[derive(Debug, Clone)]
pub struct TimelineEvent {
    /// Event identifier
    pub id: String,
    /// Event timestamp
    pub timestamp: Instant,
    /// Event type
    pub event_type: EventType,
    /// Event description
    pub description: String,
    /// Associated agent
    pub agent: Option<String>,
    /// Associated task
    pub task: Option<String>,
    /// Event duration
    pub duration: Option<Duration>,
    /// Event priority/importance
    pub importance: EventImportance,
    /// Event metadata
    pub metadata: HashMap<String, String>,
}

/// Timeline event types
#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    AgentStarted,
    AgentStopped,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    AgentInteraction,
    ResourceAlert,
    SystemEvent,
    UserAction,
    Error,
}

/// Event importance levels
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum EventImportance {
    Low,
    Normal,
    High,
    Critical,
}

/// Timeline filtering options
#[derive(Debug, Clone)]
pub struct TimelineFilters {
    /// Filter by event type
    pub event_types: Option<Vec<EventType>>,
    /// Filter by agent
    pub agents: Option<Vec<String>>,
    /// Filter by importance
    pub min_importance: Option<EventImportance>,
    /// Time range filter
    pub time_range: Option<(Instant, Instant)>,
    /// Text filter
    pub text_filter: Option<String>,
}

/// Analytics dashboard component
#[derive(Debug)]
pub struct AnalyticsDashboard {
    /// Key performance indicators
    kpis: HashMap<String, KPI>,
    /// Chart configurations
    charts: HashMap<String, DashboardChart>,
    /// Dashboard layout
    layout: DashboardLayout,
    /// Alert conditions
    alerts: Vec<AlertCondition>,
}

/// Key Performance Indicator
#[derive(Debug, Clone)]
pub struct KPI {
    /// KPI name
    pub name: String,
    /// Current value
    pub current_value: f64,
    /// Previous value for comparison
    pub previous_value: Option<f64>,
    /// Target value
    pub target_value: Option<f64>,
    /// Unit of measurement
    pub unit: String,
    /// Trend direction
    pub trend: TrendDirection,
    /// KPI status
    pub status: KPIStatus,
}

/// Trend direction
#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    Up,
    Down,
    Stable,
    Unknown,
}

/// KPI status
#[derive(Debug, Clone, PartialEq)]
pub enum KPIStatus {
    Good,
    Warning,
    Critical,
    Unknown,
}

/// Dashboard chart configuration
#[derive(Debug, Clone)]
pub struct DashboardChart {
    /// Chart type
    pub chart_type: ChartType,
    /// Chart title
    pub title: String,
    /// Data series
    pub data_series: Vec<DataSeries>,
    /// Chart settings
    pub settings: ChartSettings,
}

/// Chart types
#[derive(Debug, Clone, PartialEq)]
pub enum ChartType {
    Line,
    Bar,
    Histogram,
    Pie,
    Scatter,
    Gauge,
}

/// Data series for charts
#[derive(Debug, Clone)]
pub struct DataSeries {
    /// Series name
    pub name: String,
    /// Data points
    pub data: VecDeque<(f64, f64)>,
    /// Series color
    pub color: Color,
    /// Series style
    pub style: SeriesStyle,
}

/// Series display style
#[derive(Debug, Clone, PartialEq)]
pub enum SeriesStyle {
    Solid,
    Dashed,
    Dotted,
    Points,
}

/// Dashboard layout configuration
#[derive(Debug, Clone)]
pub struct DashboardLayout {
    /// Grid dimensions
    pub grid_size: (usize, usize),
    /// Widget positions
    pub widget_positions: HashMap<String, (usize, usize, usize, usize)>, // x, y, width, height
    /// Responsive layout
    pub responsive: bool,
}

/// Alert condition
#[derive(Debug, Clone)]
pub struct AlertCondition {
    /// Alert name
    pub name: String,
    /// Metric to monitor
    pub metric: String,
    /// Threshold value
    pub threshold: f64,
    /// Comparison operator
    pub operator: ComparisonOperator,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Currently active
    pub active: bool,
}

/// Comparison operators for alerts
#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    Equal,
    GreaterOrEqual,
    LessOrEqual,
    NotEqual,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// System state tracker
#[derive(Debug)]
pub struct SystemStateTracker {
    /// Current system state
    current_state: SystemState,
    /// State history
    state_history: VecDeque<SystemState>,
    /// State transition tracking
    transitions: VecDeque<StateTransition>,
    /// Performance counters
    counters: HashMap<String, u64>,
}

/// System state representation
#[derive(Debug, Clone)]
pub struct SystemState {
    /// State timestamp
    pub timestamp: Instant,
    /// Overall system health
    pub health: f64,
    /// System load
    pub load: f64,
    /// Active components
    pub active_components: HashMap<String, ComponentState>,
    /// Error count
    pub error_count: u64,
    /// Warning count
    pub warning_count: u64,
}

/// Component state
#[derive(Debug, Clone)]
pub struct ComponentState {
    /// Component name
    pub name: String,
    /// Component health (0.0 to 1.0)
    pub health: f64,
    /// Component status
    pub status: ComponentStatus,
    /// Last update time
    pub last_update: Instant,
}

/// Component status
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentStatus {
    Healthy,
    Degraded,
    Failed,
    Unknown,
}

/// State transition event
#[derive(Debug, Clone)]
pub struct StateTransition {
    /// Transition timestamp
    pub timestamp: Instant,
    /// Component affected
    pub component: String,
    /// Previous state
    pub from_state: ComponentStatus,
    /// New state
    pub to_state: ComponentStatus,
    /// Transition reason
    pub reason: String,
}

impl Default for VisualizationSettings {
    fn default() -> Self {
        Self {
            update_frequency: UPDATE_INTERVAL,
            show_labels: true,
            show_connection_strength: true,
            animations_enabled: true,
            color_scheme: ColorScheme::Default,
            show_metrics: true,
            max_history: MAX_DATA_POINTS,
            auto_scale: true,
            show_grid: true,
            transparency: 0.8,
        }
    }
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            focused_element: None,
            scroll_positions: HashMap::new(),
            list_states: HashMap::new(),
            scrollbar_states: HashMap::new(),
            panel_sizes: HashMap::new(),
            animation_states: HashMap::new(),
        }
    }
}

impl Default for GraphBounds {
    fn default() -> Self {
        Self {
            min_x: -100.0,
            max_x: 100.0,
            min_y: -100.0,
            max_y: 100.0,
        }
    }
}

impl Default for ChartSettings {
    fn default() -> Self {
        Self {
            time_window: Duration::from_secs(300), // 5 minutes
            colors: vec![
                Color::Blue,
                Color::Green,
                Color::Red,
                Color::Yellow,
                Color::Magenta,
                Color::Cyan,
            ],
            show_points: false,
            smooth_lines: true,
            auto_scale_y: true,
            show_legend: true,
        }
    }
}

impl Default for TimelineFilters {
    fn default() -> Self {
        Self {
            event_types: None,
            agents: None,
            min_importance: None,
            time_range: None,
            text_filter: None,
        }
    }
}

impl CoordinationVisualizer {
    /// Create a new coordination visualizer
    pub fn new() -> Self {
        let modes = vec![
            VisualizationMode::Overview,
            VisualizationMode::NetworkGraph,
            VisualizationMode::TaskFlow,
            VisualizationMode::ResourceMonitor,
            VisualizationMode::Timeline,
            VisualizationMode::Dashboard,
        ];

        Self {
            mode: VisualizationMode::Overview,
            modes,
            mode_index: 0,
            network_graph: AgentNetworkGraph {
                layout_algorithm: LayoutAlgorithm::ForceDirected,
                node_positions: HashMap::new(),
                edge_weights: HashMap::new(),
                bounds: GraphBounds::default(),
                animation_state: 0.0,
                selected_node: None,
                hover_node: None,
            },
            task_flow: TaskFlowVisualizer {
                tasks: HashMap::new(),
                dependencies: HashMap::new(),
                layout: FlowLayout {
                    direction: FlowDirection::LeftToRight,
                    node_spacing: 20.0,
                    layer_spacing: 50.0,
                    auto_arrange: true,
                },
                animations: HashMap::new(),
            },
            resource_monitor: ResourceMonitor {
                cpu_history: VecDeque::new(),
                memory_history: VecDeque::new(),
                network_history: VecDeque::new(),
                disk_history: VecDeque::new(),
                chart_settings: ChartSettings::default(),
            },
            timeline: TimelineView {
                events: VecDeque::new(),
                time_window: Duration::from_secs(600), // 10 minutes
                zoom_level: 1.0,
                scroll_position: 0.0,
                selected_event: None,
                filters: TimelineFilters::default(),
            },
            dashboard: AnalyticsDashboard {
                kpis: HashMap::new(),
                charts: HashMap::new(),
                layout: DashboardLayout {
                    grid_size: (3, 3),
                    widget_positions: HashMap::new(),
                    responsive: true,
                },
                alerts: Vec::new(),
            },
            state_tracker: SystemStateTracker {
                current_state: SystemState {
                    timestamp: Instant::now(),
                    health: 1.0,
                    load: 0.0,
                    active_components: HashMap::new(),
                    error_count: 0,
                    warning_count: 0,
                },
                state_history: VecDeque::new(),
                transitions: VecDeque::new(),
                counters: HashMap::new(),
            },
            interaction_history: VecDeque::new(),
            selected_agent: None,
            settings: VisualizationSettings::default(),
            last_update: Instant::now(),
            visible: false,
            current_snapshot: SystemSnapshot {
                timestamp: Instant::now(),
                system_time: SystemTime::now(),
                agents: HashMap::new(),
                tasks: HashMap::new(),
                resources: ResourceUsage {
                    cpu_usage: 0.0,
                    memory_usage: 0,
                    memory_total: 1024 * 1024 * 1024,
                    network_io: 0,
                    disk_io: 0,
                    active_threads: 0,
                    db_connections: 0,
                    queue_sizes: HashMap::new(),
                },
                metrics: SystemMetrics {
                    total_tasks: 0,
                    tasks_per_second: 0.0,
                    avg_response_time: 0.0,
                    error_rate: 0.0,
                    uptime: Duration::default(),
                    active_sessions: 0,
                    total_agents: 0,
                    active_agents: 0,
                },
                interactions: Vec::new(),
            },
            historical_data: VecDeque::new(),
            ui_state: UIState::default(),
        }
    }

    /// Show the coordination visualizer
    pub fn show(&mut self) {
        self.visible = true;
        trace!("Coordination visualizer opened");
    }

    /// Hide the coordination visualizer
    pub fn hide(&mut self) {
        self.visible = false;
        trace!("Coordination visualizer closed");
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Check if visualizer is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Update with new system snapshot
    pub fn update_snapshot(&mut self, snapshot: SystemSnapshot) {
        self.current_snapshot = snapshot.clone();
        
        // Add to historical data
        self.historical_data.push_back(snapshot);
        if self.historical_data.len() > self.settings.max_history {
            self.historical_data.pop_front();
        }

        // Update resource monitor
        self.update_resource_monitor();
        
        // Update network graph
        self.update_network_graph();
        
        // Update task flow
        self.update_task_flow();
        
        // Update timeline
        self.update_timeline();
        
        // Update analytics
        self.update_analytics();
        
        self.last_update = Instant::now();
    }

    /// Add agent interaction
    pub fn add_interaction(&mut self, interaction: AgentInteraction) {
        self.interaction_history.push_back(interaction.clone());
        
        // Maintain size limit
        if self.interaction_history.len() > MAX_INTERACTION_HISTORY {
            self.interaction_history.pop_front();
        }

        // Update network graph edge weights
        self.update_interaction_weights(&interaction);
        
        // Add to timeline
        self.add_timeline_event(TimelineEvent {
            id: interaction.id.clone(),
            timestamp: interaction.timestamp,
            event_type: EventType::AgentInteraction,
            description: format!(
                "{} → {} ({})", 
                interaction.from_agent, 
                interaction.to_agent, 
                format!("{:?}", interaction.interaction_type)
            ),
            agent: Some(interaction.from_agent),
            task: None,
            duration: interaction.duration,
            importance: EventImportance::Normal,
            metadata: interaction.metadata,
        });
    }

    /// Handle key events
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> CoordinationAction {
        if !self.visible {
            return CoordinationAction::None;
        }

        match key_event.code {
            KeyCode::Esc => {
                self.hide();
                CoordinationAction::Close
            }
            KeyCode::Tab => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.previous_mode();
                } else {
                    self.next_mode();
                }
                CoordinationAction::ModeChanged
            }
            KeyCode::Char('r') => {
                self.refresh_data();
                CoordinationAction::Refresh
            }
            KeyCode::Char('s') => {
                self.toggle_settings();
                CoordinationAction::ToggleSettings
            }
            KeyCode::Char('f') => {
                self.toggle_fullscreen();
                CoordinationAction::ToggleFullscreen
            }
            KeyCode::F(5) => {
                self.refresh_data();
                CoordinationAction::Refresh
            }
            KeyCode::Up => {
                self.navigate_up();
                CoordinationAction::Navigate
            }
            KeyCode::Down => {
                self.navigate_down();
                CoordinationAction::Navigate
            }
            KeyCode::Left => {
                self.navigate_left();
                CoordinationAction::Navigate
            }
            KeyCode::Right => {
                self.navigate_right();
                CoordinationAction::Navigate
            }
            KeyCode::Enter => {
                self.select_current();
                CoordinationAction::Select
            }
            KeyCode::Char(' ') => {
                self.toggle_pause();
                CoordinationAction::TogglePause
            }
            KeyCode::Char('+') => {
                self.zoom_in();
                CoordinationAction::Zoom
            }
            KeyCode::Char('-') => {
                self.zoom_out();
                CoordinationAction::Zoom
            }
            _ => CoordinationAction::None,
        }
    }

    /// Render the coordination visualizer
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
                Constraint::Min(1),    // Content
                Constraint::Length(1), // Status
            ])
            .split(area);

        // Render header
        self.render_header(f, chunks[0]);

        // Render content based on current mode
        match self.mode {
            VisualizationMode::Overview => self.render_overview(f, chunks[1]),
            VisualizationMode::NetworkGraph => self.render_network_graph(f, chunks[1]),
            VisualizationMode::TaskFlow => self.render_task_flow(f, chunks[1]),
            VisualizationMode::ResourceMonitor => self.render_resource_monitor(f, chunks[1]),
            VisualizationMode::Timeline => self.render_timeline(f, chunks[1]),
            VisualizationMode::Dashboard => self.render_dashboard(f, chunks[1]),
        }

        // Render status bar
        self.render_status_bar(f, chunks[2]);
    }

    /// Get current visualization mode
    pub fn get_mode(&self) -> &VisualizationMode {
        &self.mode
    }

    /// Set visualization mode
    pub fn set_mode(&mut self, mode: VisualizationMode) {
        if let Some(index) = self.modes.iter().position(|m| m == &mode) {
            self.mode_index = index;
        }
        self.mode = mode;
    }

    /// Get current system snapshot
    pub fn get_current_snapshot(&self) -> &SystemSnapshot {
        &self.current_snapshot
    }

    /// Get interaction history
    pub fn get_interaction_history(&self) -> &VecDeque<AgentInteraction> {
        &self.interaction_history
    }

    // Private methods

    fn next_mode(&mut self) {
        self.mode_index = (self.mode_index + 1) % self.modes.len();
        self.mode = self.modes[self.mode_index].clone();
    }

    fn previous_mode(&mut self) {
        self.mode_index = if self.mode_index == 0 {
            self.modes.len() - 1
        } else {
            self.mode_index - 1
        };
        self.mode = self.modes[self.mode_index].clone();
    }

    fn refresh_data(&mut self) {
        // Refresh logic would be implemented here
        debug!("Refreshing visualization data");
    }

    fn toggle_settings(&mut self) {
        // Settings toggle logic
        debug!("Toggling visualization settings");
    }

    fn toggle_fullscreen(&mut self) {
        // Fullscreen toggle logic
        debug!("Toggling fullscreen mode");
    }

    fn navigate_up(&mut self) {
        // Navigation logic for current mode
        debug!("Navigate up");
    }

    fn navigate_down(&mut self) {
        // Navigation logic for current mode
        debug!("Navigate down");
    }

    fn navigate_left(&mut self) {
        // Navigation logic for current mode
        debug!("Navigate left");
    }

    fn navigate_right(&mut self) {
        // Navigation logic for current mode
        debug!("Navigate right");
    }

    fn select_current(&mut self) {
        // Selection logic for current mode
        debug!("Select current item");
    }

    fn toggle_pause(&mut self) {
        // Pause/resume logic
        debug!("Toggle pause");
    }

    fn zoom_in(&mut self) {
        // Zoom in logic
        debug!("Zoom in");
    }

    fn zoom_out(&mut self) {
        // Zoom out logic
        debug!("Zoom out");
    }

    fn update_resource_monitor(&mut self) {
        let now = Instant::now();
        let resources = &self.current_snapshot.resources;

        // Add to CPU history
        self.resource_monitor.cpu_history.push_back((now, resources.cpu_usage));
        if self.resource_monitor.cpu_history.len() > MAX_DATA_POINTS {
            self.resource_monitor.cpu_history.pop_front();
        }

        // Add to memory history
        let memory_percent = resources.memory_usage as f64 / resources.memory_total as f64;
        self.resource_monitor.memory_history.push_back((now, memory_percent));
        if self.resource_monitor.memory_history.len() > MAX_DATA_POINTS {
            self.resource_monitor.memory_history.pop_front();
        }

        // Add to network history
        self.resource_monitor.network_history.push_back((now, resources.network_io as f64));
        if self.resource_monitor.network_history.len() > MAX_DATA_POINTS {
            self.resource_monitor.network_history.pop_front();
        }

        // Add to disk history
        self.resource_monitor.disk_history.push_back((now, resources.disk_io as f64));
        if self.resource_monitor.disk_history.len() > MAX_DATA_POINTS {
            self.resource_monitor.disk_history.pop_front();
        }
    }

    fn update_network_graph(&mut self) {
        // Update node positions and edge weights
        for (agent_id, agent_state) in &self.current_snapshot.agents {
            self.network_graph.node_positions.insert(
                agent_id.clone(),
                agent_state.position,
            );

            // Update edge weights based on connections
            for (target_id, strength) in &agent_state.connections {
                self.network_graph.edge_weights.insert(
                    (agent_id.clone(), target_id.clone()),
                    *strength,
                );
            }
        }
    }

    fn update_task_flow(&mut self) {
        // Update task flow visualization
        for (task_id, task_state) in &self.current_snapshot.tasks {
            let flow_node = TaskFlowNode {
                task: task_state.clone(),
                position: (0.0, 0.0), // Would be calculated based on layout
                size: (100.0, 50.0),
                animation_phase: 0.0,
            };
            self.task_flow.tasks.insert(task_id.clone(), flow_node);
        }
    }

    fn update_timeline(&mut self) {
        // Timeline updates are handled in add_timeline_event
    }

    fn update_analytics(&mut self) {
        // Update KPIs
        let metrics = &self.current_snapshot.metrics;
        
        self.dashboard.kpis.insert("total_tasks".to_string(), KPI {
            name: "Total Tasks".to_string(),
            current_value: metrics.total_tasks as f64,
            previous_value: None,
            target_value: None,
            unit: "tasks".to_string(),
            trend: TrendDirection::Up,
            status: KPIStatus::Good,
        });

        self.dashboard.kpis.insert("tasks_per_second".to_string(), KPI {
            name: "Tasks/Second".to_string(),
            current_value: metrics.tasks_per_second,
            previous_value: None,
            target_value: Some(10.0),
            unit: "tasks/s".to_string(),
            trend: TrendDirection::Stable,
            status: if metrics.tasks_per_second < 10.0 { KPIStatus::Warning } else { KPIStatus::Good },
        });

        self.dashboard.kpis.insert("error_rate".to_string(), KPI {
            name: "Error Rate".to_string(),
            current_value: metrics.error_rate,
            previous_value: None,
            target_value: Some(1.0),
            unit: "%".to_string(),
            trend: TrendDirection::Down,
            status: if metrics.error_rate > 5.0 { KPIStatus::Critical } 
                   else if metrics.error_rate > 1.0 { KPIStatus::Warning } 
                   else { KPIStatus::Good },
        });
    }

    fn update_interaction_weights(&mut self, interaction: &AgentInteraction) {
        let key = (interaction.from_agent.clone(), interaction.to_agent.clone());
        let current_weight = self.network_graph.edge_weights.get(&key).unwrap_or(&0.0);
        let new_weight = (current_weight + 0.1).min(1.0);
        self.network_graph.edge_weights.insert(key, new_weight);
    }

    fn add_timeline_event(&mut self, event: TimelineEvent) {
        self.timeline.events.push_back(event);
        
        // Maintain size limit
        if self.timeline.events.len() > MAX_INTERACTION_HISTORY {
            self.timeline.events.pop_front();
        }
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let title = format!("Multi-Agent Coordination - {}", self.mode);
        
        // Create tabs for different modes
        let tab_titles: Vec<Line> = self.modes
            .iter()
            .map(|mode| Line::from(mode.to_string()))
            .collect();

        let tabs = Tabs::new(tab_titles)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_type(BorderType::Rounded))
            .style(Style::default().fg(Color::Gray))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .select(self.mode_index);

        f.render_widget(tabs, area);
    }

    fn render_overview(&mut self, f: &mut Frame, area: Rect) {
        // Split area into multiple panels for overview
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(area);

        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[0]);

        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[1]);

        // Render mini versions of different views
        self.render_mini_network_graph(f, top_chunks[0]);
        self.render_mini_resource_monitor(f, top_chunks[1]);
        self.render_mini_task_flow(f, bottom_chunks[0]);
        self.render_mini_timeline(f, bottom_chunks[1]);
    }

    fn render_network_graph(&mut self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Agent Network")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        // Render network graph using canvas
        let canvas = Canvas::default()
            .block(block)
            .x_bounds([self.network_graph.bounds.min_x, self.network_graph.bounds.max_x])
            .y_bounds([self.network_graph.bounds.min_y, self.network_graph.bounds.max_y])
            .paint(|ctx| {
                // Draw edges
                for ((from, to), weight) in &self.network_graph.edge_weights {
                    if let (Some(from_pos), Some(to_pos)) = (
                        self.network_graph.node_positions.get(from),
                        self.network_graph.node_positions.get(to),
                    ) {
                        let color = if *weight > 0.7 {
                            Color::Green
                        } else if *weight > 0.4 {
                            Color::Yellow
                        } else {
                            Color::Gray
                        };
                        
                        ctx.draw(&ratatui::widgets::canvas::Line {
                            x1: from_pos.0,
                            y1: from_pos.1,
                            x2: to_pos.0,
                            y2: to_pos.1,
                            color,
                        });
                    }
                }

                // Draw nodes
                for (agent_id, position) in &self.network_graph.node_positions {
                    if let Some(agent_state) = self.current_snapshot.agents.get(agent_id) {
                        let color = match agent_state.status {
                            AgentStatus::Idle => Color::Blue,
                            AgentStatus::Processing { .. } => Color::Green,
                            AgentStatus::Busy => Color::Yellow,
                            AgentStatus::Error { .. } => Color::Red,
                            AgentStatus::ShuttingDown => Color::Gray,
                            AgentStatus::Offline => Color::DarkGray,
                        };

                        ctx.draw(&ratatui::widgets::canvas::Points {
                            coords: &[(position.0, position.1)],
                            color,
                        });
                    }
                }
            });

        f.render_widget(canvas, area);
    }

    fn render_task_flow(&mut self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Task Flow")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        // Create a simple list view of tasks
        let items: Vec<ListItem> = self.current_snapshot.tasks
            .values()
            .map(|task| {
                let status_color = match task.status {
                    TaskStatus::Running => Color::Green,
                    TaskStatus::Completed => Color::Blue,
                    TaskStatus::Failed => Color::Red,
                    TaskStatus::Blocked => Color::Yellow,
                    TaskStatus::Pending => Color::Gray,
                    TaskStatus::Cancelled => Color::DarkGray,
                };

                let progress_bar = "█".repeat((task.progress * 20.0) as usize);
                let content = format!(
                    "{} [{:>20}] {:.1}% - {}",
                    task.id,
                    progress_bar,
                    task.progress * 100.0,
                    task.description
                );

                ListItem::new(content).style(Style::default().fg(status_color))
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        f.render_widget(list, area);
    }

    fn render_resource_monitor(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(area);

        // CPU Usage
        self.render_resource_gauge(f, chunks[0], "CPU Usage", 
                                  self.current_snapshot.resources.cpu_usage, Color::Blue);

        // Memory Usage
        let memory_percent = self.current_snapshot.resources.memory_usage as f64 / 
                           self.current_snapshot.resources.memory_total as f64;
        self.render_resource_gauge(f, chunks[1], "Memory Usage", memory_percent, Color::Green);

        // Network I/O (simplified representation)
        let network_percent = (self.current_snapshot.resources.network_io as f64 / 1_000_000.0).min(1.0);
        self.render_resource_gauge(f, chunks[2], "Network I/O", network_percent, Color::Yellow);

        // Disk I/O (simplified representation)
        let disk_percent = (self.current_snapshot.resources.disk_io as f64 / 1_000_000.0).min(1.0);
        self.render_resource_gauge(f, chunks[3], "Disk I/O", disk_percent, Color::Magenta);
    }

    fn render_resource_gauge(&self, f: &mut Frame, area: Rect, title: &str, ratio: f64, color: Color) {
        let gauge = Gauge::default()
            .block(Block::default().title(title).borders(Borders::ALL))
            .gauge_style(Style::default().fg(color))
            .percent((ratio * 100.0) as u16)
            .label(format!("{:.1}%", ratio * 100.0));

        f.render_widget(gauge, area);
    }

    fn render_timeline(&mut self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Timeline")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let items: Vec<ListItem> = self.timeline.events
            .iter()
            .rev() // Show most recent first
            .take(50) // Limit display
            .map(|event| {
                let importance_symbol = match event.importance {
                    EventImportance::Critical => "🔴",
                    EventImportance::High => "🟠",
                    EventImportance::Normal => "🟡",
                    EventImportance::Low => "⚪",
                };

                let content = format!(
                    "{} {:?} {}",
                    importance_symbol,
                    event.event_type,
                    event.description
                );

                ListItem::new(content)
            })
            .collect();

        let list = List::new(items).block(block);
        f.render_widget(list, area);
    }

    fn render_dashboard(&mut self, f: &mut Frame, area: Rect) {
        // Simple KPI display
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(area);

        // KPIs header
        let kpi_text = format!(
            "Total Tasks: {} | Tasks/s: {:.2} | Error Rate: {:.2}% | Active Agents: {}",
            self.current_snapshot.metrics.total_tasks,
            self.current_snapshot.metrics.tasks_per_second,
            self.current_snapshot.metrics.error_rate,
            self.current_snapshot.metrics.active_agents
        );

        let kpi_paragraph = Paragraph::new(kpi_text)
            .block(Block::default()
                .title("Key Performance Indicators")
                .borders(Borders::ALL))
            .alignment(Alignment::Center);

        f.render_widget(kpi_paragraph, chunks[0]);

        // Additional dashboard content would go in chunks[1]
        let placeholder = Paragraph::new("Dashboard charts and detailed metrics would be displayed here")
            .block(Block::default()
                .title("Analytics")
                .borders(Borders::ALL))
            .alignment(Alignment::Center);

        f.render_widget(placeholder, chunks[1]);
    }

    fn render_mini_network_graph(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Network")
            .borders(Borders::ALL);

        let content = format!("Agents: {} | Connections: {}", 
                            self.current_snapshot.agents.len(),
                            self.network_graph.edge_weights.len());
        
        let paragraph = Paragraph::new(content)
            .block(block)
            .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
    }

    fn render_mini_resource_monitor(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Resources")
            .borders(Borders::ALL);

        let content = format!("CPU: {:.1}% | Mem: {:.1}%", 
                            self.current_snapshot.resources.cpu_usage * 100.0,
                            (self.current_snapshot.resources.memory_usage as f64 / 
                             self.current_snapshot.resources.memory_total as f64) * 100.0);
        
        let paragraph = Paragraph::new(content)
            .block(block)
            .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
    }

    fn render_mini_task_flow(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Tasks")
            .borders(Borders::ALL);

        let running_tasks = self.current_snapshot.tasks.values()
            .filter(|task| task.status == TaskStatus::Running)
            .count();
        
        let content = format!("Total: {} | Running: {}", 
                            self.current_snapshot.tasks.len(),
                            running_tasks);
        
        let paragraph = Paragraph::new(content)
            .block(block)
            .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
    }

    fn render_mini_timeline(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Recent Events")
            .borders(Borders::ALL);

        let recent_events = self.timeline.events.len();
        let critical_events = self.timeline.events.iter()
            .filter(|event| event.importance == EventImportance::Critical)
            .count();
        
        let content = format!("Events: {} | Critical: {}", recent_events, critical_events);
        
        let paragraph = Paragraph::new(content)
            .block(block)
            .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
    }

    fn render_status_bar(&self, f: &mut Frame, area: Rect) {
        let status_text = format!(
            "Mode: {} | Agents: {} | Tasks: {} | Last Update: {:?} ago | Tab: Switch Mode | R: Refresh | S: Settings",
            self.mode,
            self.current_snapshot.metrics.active_agents,
            self.current_snapshot.tasks.len(),
            self.last_update.elapsed()
        );

        let status_paragraph = Paragraph::new(status_text)
            .style(Style::default().fg(Color::Gray));

        f.render_widget(status_paragraph, area);
    }
}

/// Actions that can be triggered by the coordination visualizer
#[derive(Debug, Clone, PartialEq)]
pub enum CoordinationAction {
    None,
    Close,
    ModeChanged,
    Refresh,
    ToggleSettings,
    ToggleFullscreen,
    Navigate,
    Select,
    TogglePause,
    Zoom,
}

impl Default for CoordinationVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualizer_creation() {
        let visualizer = CoordinationVisualizer::new();
        assert_eq!(visualizer.mode, VisualizationMode::Overview);
        assert!(!visualizer.is_visible());
        assert_eq!(visualizer.modes.len(), 6);
    }

    #[test]
    fn test_mode_switching() {
        let mut visualizer = CoordinationVisualizer::new();
        
        visualizer.next_mode();
        assert_eq!(visualizer.mode, VisualizationMode::NetworkGraph);
        
        visualizer.previous_mode();
        assert_eq!(visualizer.mode, VisualizationMode::Overview);
    }

    #[test]
    fn test_interaction_tracking() {
        let mut visualizer = CoordinationVisualizer::new();
        
        let interaction = AgentInteraction {
            id: "test-interaction".to_string(),
            from_agent: "agent1".to_string(),
            to_agent: "agent2".to_string(),
            interaction_type: InteractionType::TaskDelegation,
            timestamp: Instant::now(),
            duration: Some(Duration::from_millis(100)),
            data_size: Some(1024),
            success: true,
            metadata: HashMap::new(),
        };

        visualizer.add_interaction(interaction);
        assert_eq!(visualizer.interaction_history.len(), 1);
        assert_eq!(visualizer.timeline.events.len(), 1);
    }
}