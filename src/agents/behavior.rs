//! Agent Behavior Customization System
//!
//! This module provides a comprehensive system for customizing agent behaviors,
//! personalities, decision-making patterns, and interaction styles. It allows
//! users to create custom agent profiles, define behavior traits, and configure
//! how agents operate in different contexts.

use crate::agents::{Agent, AgentError, AgentResult, AgentStatus, AgentTask, TaskPriority};
use crate::config::ConfigManager;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tracing::{debug, info, trace, warn};
use uuid::Uuid;

/// Maximum number of custom behavior profiles
const MAX_BEHAVIOR_PROFILES: usize = 50;
/// Maximum number of behavior traits per profile
const MAX_TRAITS_PER_PROFILE: usize = 20;
/// Default behavior evaluation interval
const DEFAULT_EVALUATION_INTERVAL: Duration = Duration::from_secs(30);

/// A complete behavior profile for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBehaviorProfile {
    /// Unique profile identifier
    pub id: String,
    /// Display name for the profile
    pub name: String,
    /// Description of the behavior profile
    pub description: String,
    /// Version of the profile (for updates)
    pub version: String,
    /// Author/creator of the profile
    pub author: Option<String>,
    /// Tags for categorization
    pub tags: HashSet<String>,
    /// Core personality traits
    pub personality: PersonalityTraits,
    /// Decision-making patterns
    pub decision_making: DecisionMakingPattern,
    /// Communication style
    pub communication: CommunicationStyle,
    /// Task handling preferences
    pub task_handling: TaskHandlingBehavior,
    /// Learning and adaptation settings
    pub learning: LearningBehavior,
    /// Error handling strategies
    pub error_handling: ErrorHandlingBehavior,
    /// Collaboration preferences
    pub collaboration: CollaborationBehavior,
    /// Resource usage policies
    pub resource_usage: ResourceUsageBehavior,
    /// Custom parameters for extensibility
    pub custom_parameters: HashMap<String, BehaviorValue>,
    /// When this profile was created
    pub created_at: SystemTime,
    /// Last modification time
    pub updated_at: SystemTime,
    /// Whether this profile is active
    pub active: bool,
}

/// Core personality traits that define agent character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityTraits {
    /// How proactive vs reactive the agent is (0.0 = reactive, 1.0 = proactive)
    pub proactiveness: f64,
    /// Risk tolerance (0.0 = risk-averse, 1.0 = risk-seeking)
    pub risk_tolerance: f64,
    /// Creativity level (0.0 = conservative, 1.0 = highly creative)
    pub creativity: f64,
    /// Social interaction preference (0.0 = solitary, 1.0 = highly social)
    pub sociability: f64,
    /// Attention to detail (0.0 = big picture, 1.0 = detail-oriented)
    pub detail_orientation: f64,
    /// Speed vs accuracy preference (0.0 = accuracy-focused, 1.0 = speed-focused)
    pub speed_vs_accuracy: f64,
    /// Formality level (0.0 = casual, 1.0 = formal)
    pub formality: f64,
    /// Helpfulness (0.0 = minimal help, 1.0 = maximum help)
    pub helpfulness: f64,
    /// Persistence (0.0 = give up easily, 1.0 = very persistent)
    pub persistence: f64,
    /// Confidence level (0.0 = uncertain, 1.0 = very confident)
    pub confidence: f64,
}

/// Decision-making patterns and strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionMakingPattern {
    /// Primary decision-making strategy
    pub strategy: DecisionStrategy,
    /// Time to spend on analysis before making decisions
    pub analysis_time: Duration,
    /// Whether to seek confirmation for important decisions
    pub seek_confirmation: bool,
    /// Confidence threshold for autonomous decisions (0.0 to 1.0)
    pub autonomy_threshold: f64,
    /// Whether to explain reasoning for decisions
    pub explain_reasoning: bool,
    /// Factors to consider when making decisions
    pub decision_factors: Vec<DecisionFactor>,
    /// Whether to use historical data in decisions
    pub use_historical_data: bool,
    /// Whether to consider user preferences
    pub consider_user_preferences: bool,
}

/// Different decision-making strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionStrategy {
    /// Quick decisions based on heuristics
    Heuristic,
    /// Analytical approach with detailed evaluation
    Analytical,
    /// Data-driven decisions based on metrics
    DataDriven,
    /// Conservative approach minimizing risks
    Conservative,
    /// Aggressive approach maximizing opportunities
    Aggressive,
    /// Balanced approach considering multiple factors
    Balanced,
    /// User-guided decisions with confirmation
    UserGuided,
    /// Collaborative decisions with other agents
    Collaborative,
}

/// Factors to consider in decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionFactor {
    /// Time constraints and deadlines
    TimeConstraints,
    /// Available resources (CPU, memory, etc.)
    ResourceAvailability,
    /// User preferences and history
    UserPreferences,
    /// Risk assessment and mitigation
    RiskAssessment,
    /// Impact on other tasks and agents
    ImpactAnalysis,
    /// Cost-benefit analysis
    CostBenefit,
    /// Quality requirements
    QualityRequirements,
    /// Compliance and constraints
    Compliance,
}

/// Communication style and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStyle {
    /// Verbosity level (0.0 = terse, 1.0 = verbose)
    pub verbosity: f64,
    /// Use of technical language (0.0 = plain language, 1.0 = technical)
    pub technical_level: f64,
    /// Emoji and emoticon usage (0.0 = none, 1.0 = frequent)
    pub emoji_usage: f64,
    /// Frequency of progress updates
    pub update_frequency: UpdateFrequency,
    /// Whether to provide explanations for actions
    pub provide_explanations: bool,
    /// Whether to ask for clarification when uncertain
    pub ask_for_clarification: bool,
    /// Preferred communication channels
    pub preferred_channels: Vec<CommunicationChannel>,
    /// Whether to summarize completed work
    pub summarize_work: bool,
    /// Language and localization preferences
    pub language_preferences: LanguagePreferences,
}

/// How frequently to provide updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateFrequency {
    /// No automatic updates
    Never,
    /// Only when requested
    OnRequest,
    /// At major milestones
    Milestones,
    /// Regular intervals
    Regular(Duration),
    /// For every significant action
    Frequent,
    /// Constant real-time updates
    Realtime,
}

/// Communication channels and methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationChannel {
    /// Direct terminal output
    Terminal,
    /// Log files
    Logs,
    /// Interactive notifications
    Notifications,
    /// Status bar updates
    StatusBar,
    /// Pop-up messages
    Popups,
    /// File-based communication
    Files,
    /// Network communication
    Network,
}

/// Language and localization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguagePreferences {
    /// Primary language (ISO 639-1 code)
    pub primary_language: String,
    /// Fallback languages in order of preference
    pub fallback_languages: Vec<String>,
    /// Timezone for time-related communications
    pub timezone: String,
    /// Date/time format preferences
    pub datetime_format: String,
    /// Number format preferences
    pub number_format: String,
}

/// Task handling behavior and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskHandlingBehavior {
    /// Task prioritization strategy
    pub prioritization_strategy: PrioritizationStrategy,
    /// Maximum number of concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Whether to batch similar tasks
    pub batch_similar_tasks: bool,
    /// Task timeout settings
    pub task_timeouts: TaskTimeoutSettings,
    /// Whether to break down complex tasks
    pub decompose_complex_tasks: bool,
    /// Minimum task complexity threshold for decomposition
    pub decomposition_threshold: f64,
    /// Whether to validate task requirements before starting
    pub validate_requirements: bool,
    /// Whether to estimate task duration
    pub estimate_duration: bool,
    /// Progress reporting interval
    pub progress_reporting: Duration,
    /// Whether to save task state for recovery
    pub save_task_state: bool,
}

/// Different task prioritization strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrioritizationStrategy {
    /// First In, First Out
    FIFO,
    /// Last In, First Out
    LIFO,
    /// Priority-based (high to low)
    Priority,
    /// Shortest job first
    ShortestFirst,
    /// Deadline-based (earliest deadline first)
    EarliestDeadline,
    /// User-defined priority
    UserDefined,
    /// Balanced approach considering multiple factors
    Balanced,
    /// AI-determined optimal order
    AIOptimized,
}

/// Task timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTimeoutSettings {
    /// Default timeout for tasks
    pub default_timeout: Duration,
    /// Timeout per task priority level
    pub priority_timeouts: HashMap<TaskPriority, Duration>,
    /// Whether to allow timeout extensions
    pub allow_extensions: bool,
    /// Maximum number of extensions allowed
    pub max_extensions: u32,
    /// Extension duration
    pub extension_duration: Duration,
}

/// Learning and adaptation behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningBehavior {
    /// Whether to learn from user interactions
    pub learn_from_interactions: bool,
    /// Whether to adapt behavior based on success/failure
    pub adapt_from_outcomes: bool,
    /// Learning rate (0.0 = no learning, 1.0 = rapid learning)
    pub learning_rate: f64,
    /// Whether to remember user preferences
    pub remember_preferences: bool,
    /// How long to retain learned information
    pub retention_period: Duration,
    /// Whether to share learning with other agents
    pub share_learning: bool,
    /// Types of patterns to learn
    pub learning_patterns: Vec<LearningPattern>,
    /// Whether to provide feedback on learning
    pub provide_learning_feedback: bool,
    /// Minimum confidence threshold for applying learned behaviors
    pub application_threshold: f64,
}

/// Types of patterns that can be learned
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningPattern {
    /// User command patterns and preferences
    UserCommandPatterns,
    /// Task success/failure patterns
    TaskOutcomePatterns,
    /// Resource usage optimization
    ResourceOptimization,
    /// Error patterns and solutions
    ErrorSolutions,
    /// Timing and scheduling patterns
    TimingPatterns,
    /// Context-specific behaviors
    ContextualBehaviors,
    /// Collaboration patterns
    CollaborationPatterns,
}

/// Error handling strategies and behaviors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingBehavior {
    /// Primary error handling strategy
    pub strategy: ErrorHandlingStrategy,
    /// Number of retry attempts for recoverable errors
    pub max_retries: u32,
    /// Delay between retry attempts
    pub retry_delay: Duration,
    /// Whether to escalate unresolved errors
    pub escalate_errors: bool,
    /// Types of errors to handle automatically
    pub auto_handle_errors: Vec<ErrorType>,
    /// Whether to log detailed error information
    pub detailed_logging: bool,
    /// Whether to notify users of errors
    pub notify_on_errors: bool,
    /// Whether to attempt graceful degradation
    pub graceful_degradation: bool,
    /// Whether to save error context for analysis
    pub save_error_context: bool,
}

/// Error handling strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorHandlingStrategy {
    /// Fail fast and report immediately
    FailFast,
    /// Retry with exponential backoff
    RetryWithBackoff,
    /// Attempt workarounds and alternatives
    WorkAround,
    /// Graceful degradation maintaining partial functionality
    GracefulDegrade,
    /// Escalate to human intervention
    EscalateToHuman,
    /// Collaborate with other agents for solutions
    CollaborativeResolve,
    /// Learn from errors and adapt
    LearnAndAdapt,
}

/// Types of errors for automatic handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    /// Network connectivity issues
    NetworkErrors,
    /// File system access errors
    FileSystemErrors,
    /// Resource exhaustion (memory, CPU)
    ResourceErrors,
    /// Temporary service unavailability
    ServiceErrors,
    /// Parsing and format errors
    ParsingErrors,
    /// Configuration errors
    ConfigurationErrors,
    /// Timeout errors
    TimeoutErrors,
    /// Permission and access errors
    PermissionErrors,
}

/// Collaboration behavior with other agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationBehavior {
    /// Willingness to collaborate (0.0 = individualistic, 1.0 = highly collaborative)
    pub collaboration_willingness: f64,
    /// Whether to actively seek collaboration opportunities
    pub seek_collaboration: bool,
    /// Whether to share resources with other agents
    pub share_resources: bool,
    /// Whether to delegate tasks to other agents
    pub delegate_tasks: bool,
    /// Types of tasks to delegate
    pub delegation_criteria: Vec<DelegationCriterion>,
    /// Whether to mentor or help other agents
    pub mentor_others: bool,
    /// Whether to accept help from other agents
    pub accept_help: bool,
    /// Communication protocols with other agents
    pub agent_communication: AgentCommunicationProtocol,
    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolutionStrategy,
}

/// Criteria for task delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelegationCriterion {
    /// Tasks outside agent's expertise
    OutsideExpertise,
    /// Tasks requiring specialized resources
    SpecializedResources,
    /// High-volume repetitive tasks
    HighVolumeRepetitive,
    /// Time-critical tasks when overloaded
    TimeCriticalOverload,
    /// Tasks that benefit from parallel execution
    ParallelExecution,
    /// Tasks requiring diverse perspectives
    DiversePerspectives,
}

/// Communication protocols between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCommunicationProtocol {
    /// Preferred communication method
    pub preferred_method: AgentCommunicationMethod,
    /// Message format and structure
    pub message_format: MessageFormat,
    /// Whether to use encryption for sensitive communications
    pub use_encryption: bool,
    /// Message priority levels
    pub priority_levels: Vec<MessagePriority>,
    /// Acknowledgment requirements
    pub acknowledgment_required: bool,
    /// Timeout for responses
    pub response_timeout: Duration,
}

/// Methods for agent-to-agent communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentCommunicationMethod {
    /// Direct in-memory messaging
    DirectMessaging,
    /// File-based message exchange
    FileBased,
    /// Network-based communication
    Network,
    /// Shared memory or database
    SharedStorage,
    /// Event-driven messaging
    EventDriven,
}

/// Message format for agent communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageFormat {
    /// Simple text messages
    PlainText,
    /// Structured JSON messages
    JSON,
    /// Binary format for efficiency
    Binary,
    /// Custom protocol-specific format
    Custom(String),
}

/// Message priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePriority {
    /// Critical system messages
    Critical,
    /// High priority operational messages
    High,
    /// Normal priority messages
    Normal,
    /// Low priority informational messages
    Low,
    /// Background/maintenance messages
    Background,
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolutionStrategy {
    /// Avoid conflicts by stepping aside
    Avoidance,
    /// Compete aggressively for resources
    Competition,
    /// Accommodate other agents' needs
    Accommodation,
    /// Seek compromise solutions
    Compromise,
    /// Collaborate to find win-win solutions
    Collaboration,
    /// Escalate to human arbitration
    Escalation,
    /// Use democratic voting mechanisms
    Voting,
}

/// Resource usage behavior and policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageBehavior {
    /// CPU usage limits (0.0 to 1.0 of available)
    pub cpu_limit: f64,
    /// Memory usage limits in bytes
    pub memory_limit: u64,
    /// Network bandwidth limits in bytes/second
    pub network_limit: u64,
    /// Disk space limits in bytes
    pub disk_limit: u64,
    /// Whether to yield resources when not actively working
    pub yield_when_idle: bool,
    /// Resource sharing policies
    pub sharing_policies: ResourceSharingPolicies,
    /// Whether to monitor and optimize resource usage
    pub optimize_usage: bool,
    /// Whether to respect system-wide resource limits
    pub respect_system_limits: bool,
    /// Cache behavior and limits
    pub cache_behavior: CacheBehavior,
}

/// Resource sharing policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSharingPolicies {
    /// Whether to share CPU with other agents
    pub share_cpu: bool,
    /// Whether to share memory pools
    pub share_memory: bool,
    /// Whether to share network connections
    pub share_network: bool,
    /// Whether to share disk caches
    pub share_disk_cache: bool,
    /// Fair sharing algorithm
    pub sharing_algorithm: SharingAlgorithm,
}

/// Algorithms for resource sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SharingAlgorithm {
    /// First-come, first-served
    FCFS,
    /// Round-robin allocation
    RoundRobin,
    /// Priority-based allocation
    Priority,
    /// Fair share based on agent importance
    FairShare,
    /// Dynamic allocation based on need
    Dynamic,
    /// Auction-based resource allocation
    Auction,
}

/// Cache behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheBehavior {
    /// Whether to use caching
    pub enabled: bool,
    /// Maximum cache size in bytes
    pub max_size: u64,
    /// Cache eviction policy
    pub eviction_policy: CacheEvictionPolicy,
    /// Time-to-live for cached items
    pub ttl: Duration,
    /// Whether to share cache with other agents
    pub shared: bool,
    /// Types of data to cache
    pub cache_types: Vec<CacheType>,
}

/// Cache eviction policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheEvictionPolicy {
    /// Least Recently Used
    LRU,
    /// Least Frequently Used
    LFU,
    /// First In, First Out
    FIFO,
    /// Random eviction
    Random,
    /// Time-based eviction
    TTL,
}

/// Types of data that can be cached
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheType {
    /// Task results and outputs
    TaskResults,
    /// File contents and metadata
    FileData,
    /// Network responses
    NetworkData,
    /// Computation results
    ComputationResults,
    /// User preferences
    UserPreferences,
    /// Agent learning data
    LearningData,
}

/// Generic behavior parameter value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BehaviorValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Floating point value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Duration value
    Duration(Duration),
    /// List of values
    List(Vec<BehaviorValue>),
    /// Map of key-value pairs
    Map(HashMap<String, BehaviorValue>),
}

/// Behavior profile manager for storing and managing profiles
pub struct BehaviorProfileManager {
    /// All loaded behavior profiles
    profiles: HashMap<String, AgentBehaviorProfile>,
    /// Default profile to use when none specified
    default_profile: Option<String>,
    /// Path to behavior profiles directory
    profiles_path: PathBuf,
    /// Configuration manager for persistence
    config_manager: ConfigManager,
    /// Currently active evaluation parameters
    evaluation_params: BehaviorEvaluationParams,
    /// Statistics and metrics
    stats: BehaviorStats,
}

/// Parameters for evaluating and applying behavior profiles
#[derive(Debug, Clone)]
pub struct BehaviorEvaluationParams {
    /// Current context (e.g., "code_generation", "analysis", "debugging")
    pub context: String,
    /// User preferences and history
    pub user_preferences: HashMap<String, BehaviorValue>,
    /// Available system resources
    pub available_resources: SystemResources,
    /// Current workload and task queue
    pub workload: WorkloadInfo,
    /// Time constraints
    pub time_constraints: Option<Duration>,
    /// Quality requirements
    pub quality_requirements: QualityRequirements,
}

/// System resource information
#[derive(Debug, Clone)]
pub struct SystemResources {
    /// Available CPU percentage (0.0 to 1.0)
    pub cpu_available: f64,
    /// Available memory in bytes
    pub memory_available: u64,
    /// Available disk space in bytes
    pub disk_available: u64,
    /// Network bandwidth in bytes/second
    pub network_bandwidth: u64,
}

/// Current workload information
#[derive(Debug, Clone)]
pub struct WorkloadInfo {
    /// Number of active tasks
    pub active_tasks: usize,
    /// Number of queued tasks
    pub queued_tasks: usize,
    /// Average task duration
    pub average_task_duration: Duration,
    /// Current system load (0.0 to 1.0)
    pub system_load: f64,
}

/// Quality requirements for tasks
#[derive(Debug, Clone)]
pub struct QualityRequirements {
    /// Required accuracy level (0.0 to 1.0)
    pub accuracy_requirement: f64,
    /// Speed vs quality preference (0.0 = quality, 1.0 = speed)
    pub speed_preference: f64,
    /// Whether formal validation is required
    pub formal_validation: bool,
    /// Whether human review is needed
    pub human_review_required: bool,
}

/// Statistics about behavior profile usage
#[derive(Debug, Default)]
pub struct BehaviorStats {
    /// Number of profiles loaded
    pub profiles_loaded: usize,
    /// Number of behavior evaluations performed
    pub evaluations_performed: u64,
    /// Profile usage counts
    pub profile_usage: HashMap<String, u64>,
    /// Average evaluation time
    pub average_evaluation_time: Duration,
    /// Most frequently used profiles
    pub popular_profiles: Vec<(String, u64)>,
}

impl Default for PersonalityTraits {
    fn default() -> Self {
        Self {
            proactiveness: 0.7,
            risk_tolerance: 0.5,
            creativity: 0.6,
            sociability: 0.6,
            detail_orientation: 0.7,
            speed_vs_accuracy: 0.4, // Slightly favor accuracy
            formality: 0.5,
            helpfulness: 0.8,
            persistence: 0.7,
            confidence: 0.6,
        }
    }
}

impl Default for DecisionMakingPattern {
    fn default() -> Self {
        Self {
            strategy: DecisionStrategy::Balanced,
            analysis_time: Duration::from_secs(5),
            seek_confirmation: true,
            autonomy_threshold: 0.8,
            explain_reasoning: true,
            decision_factors: vec![
                DecisionFactor::TimeConstraints,
                DecisionFactor::UserPreferences,
                DecisionFactor::QualityRequirements,
                DecisionFactor::RiskAssessment,
            ],
            use_historical_data: true,
            consider_user_preferences: true,
        }
    }
}

impl Default for CommunicationStyle {
    fn default() -> Self {
        Self {
            verbosity: 0.6,
            technical_level: 0.5,
            emoji_usage: 0.2,
            update_frequency: UpdateFrequency::Milestones,
            provide_explanations: true,
            ask_for_clarification: true,
            preferred_channels: vec![
                CommunicationChannel::Terminal,
                CommunicationChannel::StatusBar,
            ],
            summarize_work: true,
            language_preferences: LanguagePreferences::default(),
        }
    }
}

impl Default for LanguagePreferences {
    fn default() -> Self {
        Self {
            primary_language: "en".to_string(),
            fallback_languages: vec!["en".to_string()],
            timezone: "UTC".to_string(),
            datetime_format: "%Y-%m-%d %H:%M:%S %Z".to_string(),
            number_format: "en_US".to_string(),
        }
    }
}

impl Default for TaskHandlingBehavior {
    fn default() -> Self {
        Self {
            prioritization_strategy: PrioritizationStrategy::Priority,
            max_concurrent_tasks: 3,
            batch_similar_tasks: true,
            task_timeouts: TaskTimeoutSettings::default(),
            decompose_complex_tasks: true,
            decomposition_threshold: 0.7,
            validate_requirements: true,
            estimate_duration: true,
            progress_reporting: Duration::from_secs(30),
            save_task_state: true,
        }
    }
}

impl Default for TaskTimeoutSettings {
    fn default() -> Self {
        let mut priority_timeouts = HashMap::new();
        priority_timeouts.insert(TaskPriority::Low, Duration::from_secs(300));
        priority_timeouts.insert(TaskPriority::Normal, Duration::from_secs(600));
        priority_timeouts.insert(TaskPriority::High, Duration::from_secs(1200));
        priority_timeouts.insert(TaskPriority::Critical, Duration::from_secs(1800));

        Self {
            default_timeout: Duration::from_secs(600),
            priority_timeouts,
            allow_extensions: true,
            max_extensions: 3,
            extension_duration: Duration::from_secs(300),
        }
    }
}

impl Default for LearningBehavior {
    fn default() -> Self {
        Self {
            learn_from_interactions: true,
            adapt_from_outcomes: true,
            learning_rate: 0.3,
            remember_preferences: true,
            retention_period: Duration::from_secs(30 * 24 * 60 * 60), // 30 days
            share_learning: false,
            learning_patterns: vec![
                LearningPattern::UserCommandPatterns,
                LearningPattern::TaskOutcomePatterns,
                LearningPattern::ErrorSolutions,
            ],
            provide_learning_feedback: false,
            application_threshold: 0.7,
        }
    }
}

impl Default for ErrorHandlingBehavior {
    fn default() -> Self {
        Self {
            strategy: ErrorHandlingStrategy::RetryWithBackoff,
            max_retries: 3,
            retry_delay: Duration::from_secs(2),
            escalate_errors: true,
            auto_handle_errors: vec![
                ErrorType::NetworkErrors,
                ErrorType::ServiceErrors,
                ErrorType::TimeoutErrors,
            ],
            detailed_logging: true,
            notify_on_errors: false,
            graceful_degradation: true,
            save_error_context: true,
        }
    }
}

impl Default for CollaborationBehavior {
    fn default() -> Self {
        Self {
            collaboration_willingness: 0.7,
            seek_collaboration: false,
            share_resources: true,
            delegate_tasks: false,
            delegation_criteria: vec![
                DelegationCriterion::OutsideExpertise,
                DelegationCriterion::SpecializedResources,
            ],
            mentor_others: false,
            accept_help: true,
            agent_communication: AgentCommunicationProtocol::default(),
            conflict_resolution: ConflictResolutionStrategy::Compromise,
        }
    }
}

impl Default for AgentCommunicationProtocol {
    fn default() -> Self {
        Self {
            preferred_method: AgentCommunicationMethod::DirectMessaging,
            message_format: MessageFormat::JSON,
            use_encryption: false,
            priority_levels: vec![
                MessagePriority::Critical,
                MessagePriority::High,
                MessagePriority::Normal,
                MessagePriority::Low,
            ],
            acknowledgment_required: false,
            response_timeout: Duration::from_secs(30),
        }
    }
}

impl Default for ResourceUsageBehavior {
    fn default() -> Self {
        Self {
            cpu_limit: 0.8,
            memory_limit: 1024 * 1024 * 1024, // 1GB
            network_limit: 10 * 1024 * 1024,  // 10MB/s
            disk_limit: 10 * 1024 * 1024 * 1024, // 10GB
            yield_when_idle: true,
            sharing_policies: ResourceSharingPolicies::default(),
            optimize_usage: true,
            respect_system_limits: true,
            cache_behavior: CacheBehavior::default(),
        }
    }
}

impl Default for ResourceSharingPolicies {
    fn default() -> Self {
        Self {
            share_cpu: true,
            share_memory: true,
            share_network: true,
            share_disk_cache: true,
            sharing_algorithm: SharingAlgorithm::FairShare,
        }
    }
}

impl Default for CacheBehavior {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size: 256 * 1024 * 1024, // 256MB
            eviction_policy: CacheEvictionPolicy::LRU,
            ttl: Duration::from_secs(3600), // 1 hour
            shared: false,
            cache_types: vec![
                CacheType::TaskResults,
                CacheType::FileData,
                CacheType::ComputationResults,
            ],
        }
    }
}

impl BehaviorProfileManager {
    /// Create a new behavior profile manager
    pub fn new(profiles_path: PathBuf, config_manager: ConfigManager) -> Self {
        Self {
            profiles: HashMap::new(),
            default_profile: None,
            profiles_path,
            config_manager,
            evaluation_params: BehaviorEvaluationParams {
                context: "general".to_string(),
                user_preferences: HashMap::new(),
                available_resources: SystemResources {
                    cpu_available: 1.0,
                    memory_available: 1024 * 1024 * 1024,
                    disk_available: 10 * 1024 * 1024 * 1024,
                    network_bandwidth: 10 * 1024 * 1024,
                },
                workload: WorkloadInfo {
                    active_tasks: 0,
                    queued_tasks: 0,
                    average_task_duration: Duration::from_secs(60),
                    system_load: 0.5,
                },
                time_constraints: None,
                quality_requirements: QualityRequirements {
                    accuracy_requirement: 0.8,
                    speed_preference: 0.4,
                    formal_validation: false,
                    human_review_required: false,
                },
            },
            stats: BehaviorStats::default(),
        }
    }

    /// Load behavior profiles from disk
    pub async fn load_profiles(&mut self) -> Result<(), AgentError> {
        info!("Loading behavior profiles from {:?}", self.profiles_path);

        if !self.profiles_path.exists() {
            std::fs::create_dir_all(&self.profiles_path)
                .map_err(|e| AgentError::ConfigError(format!("Failed to create profiles directory: {}", e)))?;
        }

        let mut loaded_count = 0;
        let entries = std::fs::read_dir(&self.profiles_path)
            .map_err(|e| AgentError::ConfigError(format!("Failed to read profiles directory: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| AgentError::ConfigError(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                match self.load_profile_from_file(&path).await {
                    Ok(profile) => {
                        self.profiles.insert(profile.id.clone(), profile);
                        loaded_count += 1;
                    }
                    Err(e) => {
                        warn!("Failed to load profile from {:?}: {}", path, e);
                    }
                }
            }
        }

        // Create default profile if none exist
        if self.profiles.is_empty() {
            self.create_default_profiles().await?;
        }

        // Set default profile if not set
        if self.default_profile.is_none() && !self.profiles.is_empty() {
            self.default_profile = self.profiles.keys().next().cloned();
        }

        self.stats.profiles_loaded = self.profiles.len();
        info!("Loaded {} behavior profiles", loaded_count);
        Ok(())
    }

    /// Save all profiles to disk
    pub async fn save_profiles(&self) -> Result<(), AgentError> {
        for profile in self.profiles.values() {
            self.save_profile(profile).await?;
        }
        Ok(())
    }

    /// Create a new behavior profile
    pub fn create_profile(&mut self, mut profile: AgentBehaviorProfile) -> Result<String, AgentError> {
        if self.profiles.len() >= MAX_BEHAVIOR_PROFILES {
            return Err(AgentError::ConfigError(
                format!("Maximum number of behavior profiles ({}) exceeded", MAX_BEHAVIOR_PROFILES)
            ));
        }

        // Generate ID if not provided
        if profile.id.is_empty() {
            profile.id = Uuid::new_v4().to_string();
        }

        // Check for duplicate IDs
        if self.profiles.contains_key(&profile.id) {
            return Err(AgentError::ConfigError(
                format!("Profile with ID '{}' already exists", profile.id)
            ));
        }

        // Set timestamps
        let now = SystemTime::now();
        profile.created_at = now;
        profile.updated_at = now;

        let profile_id = profile.id.clone();
        self.profiles.insert(profile_id.clone(), profile);

        debug!("Created behavior profile: {}", profile_id);
        Ok(profile_id)
    }

    /// Update an existing behavior profile
    pub fn update_profile(&mut self, profile: AgentBehaviorProfile) -> Result<(), AgentError> {
        if !self.profiles.contains_key(&profile.id) {
            return Err(AgentError::ConfigError(
                format!("Profile with ID '{}' does not exist", profile.id)
            ));
        }

        let mut updated_profile = profile;
        updated_profile.updated_at = SystemTime::now();

        self.profiles.insert(updated_profile.id.clone(), updated_profile);
        debug!("Updated behavior profile: {}", updated_profile.id);
        Ok(())
    }

    /// Delete a behavior profile
    pub fn delete_profile(&mut self, profile_id: &str) -> Result<(), AgentError> {
        if !self.profiles.contains_key(profile_id) {
            return Err(AgentError::ConfigError(
                format!("Profile with ID '{}' does not exist", profile_id)
            ));
        }

        // Don't allow deleting the default profile
        if self.default_profile.as_ref() == Some(&profile_id.to_string()) {
            return Err(AgentError::ConfigError(
                "Cannot delete the default behavior profile".to_string()
            ));
        }

        self.profiles.remove(profile_id);
        debug!("Deleted behavior profile: {}", profile_id);
        Ok(())
    }

    /// Get a behavior profile by ID
    pub fn get_profile(&self, profile_id: &str) -> Option<&AgentBehaviorProfile> {
        self.profiles.get(profile_id)
    }

    /// Get all available profiles
    pub fn get_all_profiles(&self) -> Vec<&AgentBehaviorProfile> {
        self.profiles.values().collect()
    }

    /// Get profiles by tags
    pub fn get_profiles_by_tags(&self, tags: &[String]) -> Vec<&AgentBehaviorProfile> {
        self.profiles
            .values()
            .filter(|profile| {
                tags.iter().any(|tag| profile.tags.contains(tag))
            })
            .collect()
    }

    /// Set the default behavior profile
    pub fn set_default_profile(&mut self, profile_id: &str) -> Result<(), AgentError> {
        if !self.profiles.contains_key(profile_id) {
            return Err(AgentError::ConfigError(
                format!("Profile with ID '{}' does not exist", profile_id)
            ));
        }

        self.default_profile = Some(profile_id.to_string());
        debug!("Set default behavior profile: {}", profile_id);
        Ok(())
    }

    /// Get the default behavior profile
    pub fn get_default_profile(&self) -> Option<&AgentBehaviorProfile> {
        self.default_profile
            .as_ref()
            .and_then(|id| self.profiles.get(id))
    }

    /// Evaluate which behavior profile to use for a given context
    pub fn evaluate_best_profile(&mut self, context: &BehaviorEvaluationParams) -> Option<&AgentBehaviorProfile> {
        let start_time = SystemTime::now();

        // Simple evaluation strategy - can be made more sophisticated
        let best_profile = self.profiles
            .values()
            .filter(|profile| profile.active)
            .max_by(|a, b| {
                let score_a = self.calculate_profile_score(a, context);
                let score_b = self.calculate_profile_score(b, context);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Ok(elapsed) = start_time.elapsed() {
            self.stats.average_evaluation_time = elapsed;
        }
        self.stats.evaluations_performed += 1;

        if let Some(profile) = best_profile {
            *self.stats.profile_usage.entry(profile.id.clone()).or_insert(0) += 1;
        }

        best_profile
    }

    /// Calculate a score for how well a profile matches the current context
    fn calculate_profile_score(&self, profile: &AgentBehaviorProfile, context: &BehaviorEvaluationParams) -> f64 {
        let mut score = 0.0;
        
        // Base score for active profiles
        if profile.active {
            score += 1.0;
        }

        // Context-specific scoring
        match context.context.as_str() {
            "code_generation" => {
                score += profile.personality.creativity * 0.3;
                score += profile.personality.detail_orientation * 0.2;
                score += if matches!(profile.decision_making.strategy, DecisionStrategy::Analytical) { 0.2 } else { 0.0 };
            }
            "analysis" => {
                score += profile.personality.detail_orientation * 0.4;
                score += (1.0 - profile.personality.speed_vs_accuracy) * 0.3;
                score += if matches!(profile.decision_making.strategy, DecisionStrategy::DataDriven) { 0.2 } else { 0.0 };
            }
            "debugging" => {
                score += profile.personality.persistence * 0.3;
                score += profile.personality.detail_orientation * 0.3;
                score += if matches!(profile.error_handling.strategy, ErrorHandlingStrategy::LearnAndAdapt) { 0.2 } else { 0.0 };
            }
            _ => {
                // General scoring for unknown contexts
                score += profile.personality.helpfulness * 0.2;
                score += profile.personality.proactiveness * 0.1;
            }
        }

        // Resource constraints scoring
        if context.available_resources.cpu_available < 0.5 {
            score += if profile.resource_usage.cpu_limit < 0.6 { 0.2 } else { -0.2 };
        }

        if context.available_resources.memory_available < 512 * 1024 * 1024 {
            score += if profile.resource_usage.memory_limit < 512 * 1024 * 1024 { 0.2 } else { -0.2 };
        }

        // Time constraints scoring
        if let Some(time_limit) = context.time_constraints {
            if time_limit < Duration::from_secs(60) {
                score += profile.personality.speed_vs_accuracy * 0.3;
            } else {
                score += (1.0 - profile.personality.speed_vs_accuracy) * 0.2;
            }
        }

        // Quality requirements scoring
        if context.quality_requirements.accuracy_requirement > 0.8 {
            score += (1.0 - profile.personality.speed_vs_accuracy) * 0.3;
            score += profile.personality.detail_orientation * 0.2;
        }

        score.max(0.0)
    }

    /// Get statistics about profile usage
    pub fn get_stats(&self) -> &BehaviorStats {
        &self.stats
    }

    /// Update evaluation parameters
    pub fn update_evaluation_params(&mut self, params: BehaviorEvaluationParams) {
        self.evaluation_params = params;
    }

    // Private methods

    async fn load_profile_from_file(&self, path: &PathBuf) -> Result<AgentBehaviorProfile, AgentError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AgentError::ConfigError(format!("Failed to read profile file: {}", e)))?;

        let profile: AgentBehaviorProfile = toml::from_str(&content)
            .map_err(|e| AgentError::ConfigError(format!("Failed to parse profile TOML: {}", e)))?;

        trace!("Loaded profile '{}' from {:?}", profile.name, path);
        Ok(profile)
    }

    async fn save_profile(&self, profile: &AgentBehaviorProfile) -> Result<(), AgentError> {
        let filename = format!("{}.toml", profile.id);
        let path = self.profiles_path.join(filename);

        let content = toml::to_string_pretty(profile)
            .map_err(|e| AgentError::ConfigError(format!("Failed to serialize profile: {}", e)))?;

        std::fs::write(&path, content)
            .map_err(|e| AgentError::ConfigError(format!("Failed to write profile file: {}", e)))?;

        trace!("Saved profile '{}' to {:?}", profile.name, path);
        Ok(())
    }

    async fn create_default_profiles(&mut self) -> Result<(), AgentError> {
        let default_profiles = vec![
            AgentBehaviorProfile {
                id: "balanced".to_string(),
                name: "Balanced Assistant".to_string(),
                description: "A well-balanced agent suitable for general tasks".to_string(),
                version: "1.0.0".to_string(),
                author: Some("DevKit".to_string()),
                tags: ["default", "general", "balanced"].iter().map(|s| s.to_string()).collect(),
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
            },
            AgentBehaviorProfile {
                id: "creative".to_string(),
                name: "Creative Assistant".to_string(),
                description: "A creative agent focused on innovative solutions".to_string(),
                version: "1.0.0".to_string(),
                author: Some("DevKit".to_string()),
                tags: ["creative", "innovative", "artistic"].iter().map(|s| s.to_string()).collect(),
                personality: PersonalityTraits {
                    creativity: 0.9,
                    risk_tolerance: 0.7,
                    proactiveness: 0.8,
                    ..PersonalityTraits::default()
                },
                decision_making: DecisionMakingPattern {
                    strategy: DecisionStrategy::Heuristic,
                    analysis_time: Duration::from_secs(3),
                    autonomy_threshold: 0.6,
                    ..DecisionMakingPattern::default()
                },
                communication: CommunicationStyle {
                    verbosity: 0.8,
                    emoji_usage: 0.6,
                    ..CommunicationStyle::default()
                },
                task_handling: TaskHandlingBehavior {
                    decompose_complex_tasks: true,
                    ..TaskHandlingBehavior::default()
                },
                learning: LearningBehavior {
                    learning_rate: 0.5,
                    ..LearningBehavior::default()
                },
                error_handling: ErrorHandlingBehavior {
                    strategy: ErrorHandlingStrategy::WorkAround,
                    ..ErrorHandlingBehavior::default()
                },
                collaboration: CollaborationBehavior {
                    collaboration_willingness: 0.8,
                    seek_collaboration: true,
                    ..CollaborationBehavior::default()
                },
                resource_usage: ResourceUsageBehavior::default(),
                custom_parameters: HashMap::new(),
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                active: true,
            },
            AgentBehaviorProfile {
                id: "analytical".to_string(),
                name: "Analytical Assistant".to_string(),
                description: "A detail-oriented agent focused on accuracy and thoroughness".to_string(),
                version: "1.0.0".to_string(),
                author: Some("DevKit".to_string()),
                tags: ["analytical", "detail-oriented", "accurate"].iter().map(|s| s.to_string()).collect(),
                personality: PersonalityTraits {
                    detail_orientation: 0.9,
                    speed_vs_accuracy: 0.2,
                    persistence: 0.8,
                    confidence: 0.7,
                    ..PersonalityTraits::default()
                },
                decision_making: DecisionMakingPattern {
                    strategy: DecisionStrategy::Analytical,
                    analysis_time: Duration::from_secs(10),
                    seek_confirmation: true,
                    autonomy_threshold: 0.9,
                    explain_reasoning: true,
                    ..DecisionMakingPattern::default()
                },
                communication: CommunicationStyle {
                    verbosity: 0.8,
                    technical_level: 0.7,
                    provide_explanations: true,
                    ..CommunicationStyle::default()
                },
                task_handling: TaskHandlingBehavior {
                    validate_requirements: true,
                    estimate_duration: true,
                    decompose_complex_tasks: true,
                    ..TaskHandlingBehavior::default()
                },
                learning: LearningBehavior {
                    learning_rate: 0.4,
                    application_threshold: 0.8,
                    ..LearningBehavior::default()
                },
                error_handling: ErrorHandlingBehavior {
                    strategy: ErrorHandlingStrategy::LearnAndAdapt,
                    detailed_logging: true,
                    save_error_context: true,
                    ..ErrorHandlingBehavior::default()
                },
                collaboration: CollaborationBehavior {
                    mentor_others: true,
                    ..CollaborationBehavior::default()
                },
                resource_usage: ResourceUsageBehavior::default(),
                custom_parameters: HashMap::new(),
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                active: true,
            },
        ];

        for profile in default_profiles {
            self.create_profile(profile)?;
        }

        // Set balanced as the default
        self.set_default_profile("balanced")?;

        info!("Created {} default behavior profiles", self.profiles.len());
        Ok(())
    }
}

impl BehaviorValue {
    /// Convert to string if possible
    pub fn as_string(&self) -> Option<&String> {
        if let BehaviorValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Convert to integer if possible
    pub fn as_integer(&self) -> Option<i64> {
        if let BehaviorValue::Integer(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    /// Convert to float if possible
    pub fn as_float(&self) -> Option<f64> {
        if let BehaviorValue::Float(f) = self {
            Some(*f)
        } else {
            None
        }
    }

    /// Convert to boolean if possible
    pub fn as_boolean(&self) -> Option<bool> {
        if let BehaviorValue::Boolean(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    /// Convert to duration if possible
    pub fn as_duration(&self) -> Option<Duration> {
        if let BehaviorValue::Duration(d) = self {
            Some(*d)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_profile_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_manager = ConfigManager::new(temp_dir.path().to_path_buf()).unwrap();
        let mut manager = BehaviorProfileManager::new(
            temp_dir.path().join("profiles"),
            config_manager,
        );

        let profile = AgentBehaviorProfile {
            id: "test".to_string(),
            name: "Test Profile".to_string(),
            description: "A test profile".to_string(),
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
        };

        let profile_id = manager.create_profile(profile).unwrap();
        assert_eq!(profile_id, "test");
        assert!(manager.get_profile("test").is_some());
    }

    #[tokio::test]
    async fn test_default_profiles() {
        let temp_dir = TempDir::new().unwrap();
        let config_manager = ConfigManager::new(temp_dir.path().to_path_buf()).unwrap();
        let mut manager = BehaviorProfileManager::new(
            temp_dir.path().join("profiles"),
            config_manager,
        );

        manager.load_profiles().await.unwrap();
        
        assert!(!manager.profiles.is_empty());
        assert!(manager.get_default_profile().is_some());
        assert!(manager.get_profile("balanced").is_some());
        assert!(manager.get_profile("creative").is_some());
        assert!(manager.get_profile("analytical").is_some());
    }

    #[test]
    fn test_profile_scoring() {
        let temp_dir = TempDir::new().unwrap();
        let config_manager = ConfigManager::new(temp_dir.path().to_path_buf()).unwrap();
        let manager = BehaviorProfileManager::new(
            temp_dir.path().join("profiles"),
            config_manager,
        );

        let profile = AgentBehaviorProfile {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            tags: HashSet::new(),
            personality: PersonalityTraits {
                creativity: 0.9,
                detail_orientation: 0.3,
                ..PersonalityTraits::default()
            },
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
        };

        let context = BehaviorEvaluationParams {
            context: "code_generation".to_string(),
            user_preferences: HashMap::new(),
            available_resources: SystemResources {
                cpu_available: 1.0,
                memory_available: 1024 * 1024 * 1024,
                disk_available: 10 * 1024 * 1024 * 1024,
                network_bandwidth: 10 * 1024 * 1024,
            },
            workload: WorkloadInfo {
                active_tasks: 0,
                queued_tasks: 0,
                average_task_duration: Duration::from_secs(60),
                system_load: 0.5,
            },
            time_constraints: None,
            quality_requirements: QualityRequirements {
                accuracy_requirement: 0.6,
                speed_preference: 0.5,
                formal_validation: false,
                human_review_required: false,
            },
        };

        let score = manager.calculate_profile_score(&profile, &context);
        assert!(score > 0.0);
    }
}