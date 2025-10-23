//! Advanced Multi-Agent State Management System
//!
//! This module provides state machines for agent coordination, convergence guarantees,
//! deadlock detection, and cross-agent memory sharing with conflict resolution.

use crate::agents::{AgentError, AgentResult, AgentStatus, AgentTask, TaskPriority};
// use crate::telemetry::TelemetryManager; // TODO: Re-enable when telemetry is properly implemented
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore, watch};
use uuid::Uuid;

/// Advanced state machine for multi-agent coordination
pub struct AgentStateMachine {
    /// Current orchestration state
    state: Arc<RwLock<OrchestrationState>>,
    /// Agent states and transitions
    agent_states: Arc<RwLock<HashMap<Uuid, AgentState>>>,
    /// Shared memory across agents
    shared_memory: Arc<RwLock<SharedMemory>>,
    /// Convergence detector
    convergence_detector: Arc<ConvergenceDetector>,
    /// Deadlock detector
    deadlock_detector: Arc<DeadlockDetector>,
    /// Conflict resolver
    conflict_resolver: Arc<ConflictResolver>,
    /// State transition listeners
    state_listeners: Arc<RwLock<Vec<StateListener>>>,
    /// Concurrency limiter
    concurrency_limiter: Arc<Semaphore>,
    /// Telemetry integration
    telemetry: Option<Arc<dyn std::fmt::Debug + Send + Sync>>, // TODO: Replace with actual TelemetryManager when available
}

/// Overall orchestration state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationState {
    pub phase: OrchestrationPhase,
    pub active_agents: HashSet<Uuid>,
    pub completed_agents: HashSet<Uuid>,
    pub failed_agents: HashMap<Uuid, String>,
    pub shared_goals: Vec<Goal>,
    pub convergence_metrics: ConvergenceMetrics,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub last_state_change: chrono::DateTime<chrono::Utc>,
    pub timeout: Option<chrono::DateTime<chrono::Utc>>,
}

/// Individual agent state within the state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub agent_id: Uuid,
    pub phase: AgentPhase,
    pub status: AgentStatus,
    pub dependencies: Vec<Uuid>, // Agents this agent depends on
    pub dependents: Vec<Uuid>,   // Agents that depend on this agent
    pub local_memory: LocalMemory,
    pub last_checkpoint: Option<StateCheckpoint>,
    pub transition_history: VecDeque<StateTransition>,
    pub convergence_contribution: f64,
    pub lock_requests: Vec<ResourceLockRequest>,
    pub current_locks: HashSet<String>,
}

/// Phases of the overall orchestration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrchestrationPhase {
    Initializing,
    Planning,
    Executing,
    Coordinating,
    Converging,
    Finalizing,
    Completed,
    Failed(String),
    DeadlockDetected,
    RecoveryInProgress,
}

/// Individual agent phases
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentPhase {
    Idle,
    Planning,
    WaitingForDependencies,
    Executing,
    WaitingForResources,
    Coordinating,
    Finalizing,
    Completed,
    Failed(String),
    Suspended,
}

/// Shared memory accessible by all agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedMemory {
    pub facts: HashMap<String, Fact>,
    pub goals: HashMap<String, Goal>,
    pub artifacts: HashMap<String, SharedArtifact>,
    pub constraints: Vec<Constraint>,
    pub metrics: HashMap<String, f64>,
    pub version: u64, // For conflict detection
}

/// Local memory for individual agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalMemory {
    pub working_set: HashMap<String, serde_json::Value>,
    pub intermediate_results: Vec<IntermediateResult>,
    pub private_goals: Vec<Goal>,
    pub resource_reservations: Vec<ResourceReservation>,
    pub version: u64,
}

/// A fact in the shared knowledge base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub key: String,
    pub value: serde_json::Value,
    pub confidence: f64,
    pub source_agent: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub ttl: Option<chrono::DateTime<chrono::Utc>>,
    pub dependencies: Vec<String>, // Other facts this depends on
}

/// A goal in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub description: String,
    pub priority: TaskPriority,
    pub assigned_agents: Vec<Uuid>,
    pub success_criteria: Vec<SuccessCriterion>,
    pub current_progress: f64,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub parent_goal: Option<String>,
    pub sub_goals: Vec<String>,
}

/// Shared artifacts between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedArtifact {
    pub id: String,
    pub content: serde_json::Value,
    pub artifact_type: ArtifactType,
    pub owner_agent: Uuid,
    pub access_permissions: Vec<AccessPermission>,
    pub version: u64,
    pub checksum: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
}

/// Types of artifacts that can be shared
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    CodeFile,
    Documentation,
    TestResults,
    Analysis,
    Configuration,
    Data,
    Model,
    Custom(String),
}

/// Access permissions for shared artifacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPermission {
    pub agent_id: Uuid,
    pub permission: Permission,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Permission levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    Read,
    Write,
    Execute,
    Delete,
    Admin,
}

/// Constraints in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    pub constraint_type: ConstraintType,
    pub description: String,
    pub applies_to: Vec<Uuid>, // Agents this constraint applies to
    pub is_hard: bool, // Hard constraint (must be satisfied) vs soft (should be satisfied)
}

/// Types of constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    ResourceLimit,
    TemporalOrdering,
    MutualExclusion,
    Dependency,
    QualityThreshold,
    Custom(String),
}

/// Success criteria for goals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriterion {
    pub id: String,
    pub description: String,
    pub metric: String,
    pub threshold: f64,
    pub operator: ComparisonOperator,
    pub weight: f64, // Relative importance
}

/// Comparison operators for success criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    Equal,
    GreaterThanOrEqual,
    LessThanOrEqual,
    NotEqual,
}

/// Intermediate results from agent processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntermediateResult {
    pub id: String,
    pub content: serde_json::Value,
    pub confidence: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub dependencies: Vec<String>,
}

/// Resource reservation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReservation {
    pub resource_id: String,
    pub amount: u64,
    pub reserved_until: chrono::DateTime<chrono::Utc>,
    pub reservation_type: ReservationType,
}

/// Types of resource reservations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReservationType {
    Exclusive,
    Shared,
    Priority,
}

/// Resource lock request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLockRequest {
    pub resource_id: String,
    pub lock_type: LockType,
    pub priority: TaskPriority,
    pub requested_at: chrono::DateTime<chrono::Utc>,
    pub timeout: Option<Duration>,
}

/// Types of locks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LockType {
    Read,
    Write,
    Exclusive,
}

/// State checkpoint for rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateCheckpoint {
    pub id: String,
    pub agent_state: Box<AgentState>,
    pub shared_memory_version: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub description: String,
}

/// State transition record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from_phase: AgentPhase,
    pub to_phase: AgentPhase,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub trigger: TransitionTrigger,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// What triggered a state transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionTrigger {
    TaskCompleted,
    DependencyMet,
    ResourceAvailable,
    Error(String),
    Timeout,
    ExternalSignal,
    ConvergenceReached,
    DeadlockResolution,
}

/// Metrics for measuring convergence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceMetrics {
    pub consensus_score: f64,
    pub goal_completion_rate: f64,
    pub agent_agreement_rate: f64,
    pub fact_stability_score: f64,
    pub convergence_velocity: f64,
    pub last_major_change: chrono::DateTime<chrono::Utc>,
    pub stability_window: Duration,
}

/// Convergence detection system
pub struct ConvergenceDetector {
    /// Minimum consensus threshold
    consensus_threshold: f64,
    /// Stability window duration
    stability_window: Duration,
    /// Historical metrics
    metrics_history: Arc<RwLock<VecDeque<ConvergenceMetrics>>>,
    /// Convergence listeners
    listeners: Arc<RwLock<Vec<Box<dyn ConvergenceListener + Send + Sync>>>>,
}

/// Deadlock detection system
pub struct DeadlockDetector {
    /// Detection interval
    detection_interval: Duration,
    /// Last detection run
    last_detection: Arc<RwLock<Instant>>,
    /// Detected cycles
    detected_cycles: Arc<RwLock<Vec<DependencyCycle>>>,
    /// Detection listeners
    listeners: Arc<RwLock<Vec<Box<dyn DeadlockListener + Send + Sync>>>>,
}

/// A detected dependency cycle (potential deadlock)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyCycle {
    pub agents: Vec<Uuid>,
    pub resources: Vec<String>,
    pub cycle_type: CycleType,
    pub severity: DeadlockSeverity,
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

/// Types of dependency cycles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CycleType {
    AgentDependency,
    ResourceLock,
    GoalDependency,
    Mixed,
}

/// Severity levels for deadlocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeadlockSeverity {
    Low,    // May resolve naturally
    Medium, // Likely needs intervention
    High,   // Requires immediate resolution
    Critical, // System-threatening
}

/// Conflict resolution system
pub struct ConflictResolver {
    /// Resolution strategies
    strategies: Vec<Box<dyn ResolutionStrategy + Send + Sync>>,
    /// Active conflicts
    active_conflicts: Arc<RwLock<HashMap<String, Conflict>>>,
    /// Resolution history
    resolution_history: Arc<RwLock<VecDeque<ConflictResolution>>>,
}

/// A detected conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub id: String,
    pub conflict_type: ConflictType,
    pub involved_agents: Vec<Uuid>,
    pub conflicting_resources: Vec<String>,
    pub severity: ConflictSeverity,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub description: String,
}

/// Types of conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    ResourceContention,
    GoalContradiction,
    FactDisagreement,
    VersionConflict,
    PriorityConflict,
    ConstraintViolation,
}

/// Conflict severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Record of a conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub conflict_id: String,
    pub strategy_used: String,
    pub resolution_time: Duration,
    pub success: bool,
    pub outcome: String,
    pub resolved_at: chrono::DateTime<chrono::Utc>,
}

/// Trait for convergence listeners
pub trait ConvergenceListener: Send + Sync {
    fn on_convergence_progress(&self, metrics: &ConvergenceMetrics);
    fn on_convergence_achieved(&self, final_metrics: &ConvergenceMetrics);
    fn on_divergence_detected(&self, metrics: &ConvergenceMetrics);
}

/// Trait for deadlock listeners
pub trait DeadlockListener: Send + Sync {
    fn on_deadlock_detected(&self, cycle: &DependencyCycle);
    fn on_deadlock_resolved(&self, cycle: &DependencyCycle);
}

/// Trait for resolution strategies
pub trait ResolutionStrategy: Send + Sync {
    fn can_resolve(&self, conflict: &Conflict) -> bool;
    fn resolve(&self, conflict: &Conflict, state: &OrchestrationState) -> Result<ConflictResolution, AgentError>;
    fn strategy_name(&self) -> &str;
    fn priority(&self) -> u32;
}

/// State change listener
pub type StateListener = Box<dyn Fn(&OrchestrationState, &AgentState) + Send + Sync>;

/// Errors specific to state management
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Deadlock detected: {0}")]
    DeadlockDetected(String),
    
    #[error("Convergence timeout: {0}")]
    ConvergenceTimeout(String),
    
    #[error("Conflict resolution failed: {0}")]
    ConflictResolutionFailed(String),
    
    #[error("Invalid state transition: {from:?} -> {to:?}")]
    InvalidTransition { from: AgentPhase, to: AgentPhase },
    
    #[error("Resource lock timeout: {0}")]
    ResourceLockTimeout(String),
    
    #[error("Dependency cycle detected: {0}")]
    DependencyCycle(String),
    
    #[error("State machine error: {0}")]
    StateMachineError(String),
}

impl AgentStateMachine {
    /// Create a new agent state machine
    pub async fn new(
        max_concurrent_agents: usize,
        telemetry: Option<Arc<dyn std::fmt::Debug + Send + Sync>>,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(OrchestrationState::new())),
            agent_states: Arc::new(RwLock::new(HashMap::new())),
            shared_memory: Arc::new(RwLock::new(SharedMemory::new())),
            convergence_detector: Arc::new(ConvergenceDetector::new(0.8, Duration::from_secs(10))),
            deadlock_detector: Arc::new(DeadlockDetector::new(Duration::from_secs(5))),
            conflict_resolver: Arc::new(ConflictResolver::new()),
            state_listeners: Arc::new(RwLock::new(Vec::new())),
            concurrency_limiter: Arc::new(Semaphore::new(max_concurrent_agents)),
            telemetry,
        }
    }
    
    /// Register an agent in the state machine
    pub async fn register_agent(&self, agent_id: Uuid) -> Result<(), StateError> {
        let mut states = self.agent_states.write().await;
        
        if states.contains_key(&agent_id) {
            return Err(StateError::StateMachineError(format!(
                "Agent {} already registered", agent_id
            )));
        }
        
        let agent_state = AgentState::new(agent_id);
        states.insert(agent_id, agent_state.clone());
        
        // Update orchestration state
        let mut orch_state = self.state.write().await;
        orch_state.active_agents.insert(agent_id);
        orch_state.last_state_change = chrono::Utc::now();
        
        // Notify listeners
        self.notify_state_change(&orch_state, &agent_state).await;
        
        Ok(())
    }
    
    /// Transition an agent to a new phase
    pub async fn transition_agent(
        &self,
        agent_id: Uuid,
        new_phase: AgentPhase,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<(), StateError> {
        let mut states = self.agent_states.write().await;
        
        let agent_state = states.get_mut(&agent_id)
            .ok_or_else(|| StateError::StateMachineError(format!(
                "Agent {} not registered", agent_id
            )))?;
        
        let old_phase = agent_state.phase.clone();
        
        // Validate transition
        if !self.is_valid_transition(&old_phase, &new_phase) {
            return Err(StateError::InvalidTransition {
                from: old_phase,
                to: new_phase,
            });
        }
        
        // Record transition
        let transition = StateTransition {
            from_phase: old_phase,
            to_phase: new_phase.clone(),
            timestamp: chrono::Utc::now(),
            trigger: TransitionTrigger::ExternalSignal, // Would be determined by context
            metadata,
        };
        
        agent_state.phase = new_phase;
        agent_state.transition_history.push_back(transition);
        
        // Limit transition history size
        while agent_state.transition_history.len() > 100 {
            agent_state.transition_history.pop_front();
        }
        
        // Update orchestration state
        let mut orch_state = self.state.write().await;
        orch_state.last_state_change = chrono::Utc::now();
        
        // Check if agent completed
        if matches!(agent_state.phase, AgentPhase::Completed) {
            orch_state.active_agents.remove(&agent_id);
            orch_state.completed_agents.insert(agent_id);
        }
        
        // Check if agent failed
        if matches!(agent_state.phase, AgentPhase::Failed(_)) {
            orch_state.active_agents.remove(&agent_id);
            if let AgentPhase::Failed(error) = &agent_state.phase {
                orch_state.failed_agents.insert(agent_id, error.clone());
            }
        }
        
        // Notify listeners
        self.notify_state_change(&orch_state, agent_state).await;
        
        // Check for convergence
        tokio::spawn({
            let detector = self.convergence_detector.clone();
            let state = self.state.clone();
            let agent_states = self.agent_states.clone();
            async move {
                detector.check_convergence(&state, &agent_states).await;
            }
        });
        
        Ok(())
    }
    
    /// Add a fact to shared memory
    pub async fn add_fact(&self, fact: Fact) -> Result<(), StateError> {
        let mut memory = self.shared_memory.write().await;
        
        // Check for conflicts
        if let Some(existing_fact) = memory.facts.get(&fact.key) {
            if existing_fact.value != fact.value && existing_fact.confidence > 0.5 {
                // Potential conflict - delegate to conflict resolver
                let conflict = Conflict {
                    id: Uuid::new_v4().to_string(),
                    conflict_type: ConflictType::FactDisagreement,
                    involved_agents: vec![existing_fact.source_agent, fact.source_agent],
                    conflicting_resources: vec![fact.key.clone()],
                    severity: ConflictSeverity::Warning,
                    detected_at: chrono::Utc::now(),
                    description: format!("Conflicting facts for key: {}", fact.key),
                };
                
                // For now, prioritize higher confidence
                if fact.confidence > existing_fact.confidence {
                    memory.facts.insert(fact.key.clone(), fact);
                }
                
                self.conflict_resolver.handle_conflict(conflict).await;
            } else {
                memory.facts.insert(fact.key.clone(), fact);
            }
        } else {
            memory.facts.insert(fact.key.clone(), fact);
        }
        
        memory.version += 1;
        Ok(())
    }
    
    /// Get current orchestration state
    pub async fn get_orchestration_state(&self) -> OrchestrationState {
        self.state.read().await.clone()
    }
    
    /// Get agent state
    pub async fn get_agent_state(&self, agent_id: Uuid) -> Option<AgentState> {
        self.agent_states.read().await.get(&agent_id).cloned()
    }
    
    /// Get shared memory snapshot
    pub async fn get_shared_memory(&self) -> SharedMemory {
        self.shared_memory.read().await.clone()
    }
    
    /// Check for deadlocks
    pub async fn check_deadlocks(&self) -> Result<Vec<DependencyCycle>, StateError> {
        self.deadlock_detector.detect_cycles(&self.agent_states).await
    }
    
    /// Start convergence monitoring
    pub async fn start_convergence_monitoring(&self) {
        let detector = self.convergence_detector.clone();
        let state = self.state.clone();
        let agent_states = self.agent_states.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                let _ = detector.check_convergence(&state, &agent_states).await;
            }
        });
    }
    
    /// Validate if a state transition is allowed
    fn is_valid_transition(&self, from: &AgentPhase, to: &AgentPhase) -> bool {
        use AgentPhase::*;
        
        matches!((from, to), 
            (Idle, Planning) |
            (Planning, WaitingForDependencies) |
            (Planning, Executing) |
            (WaitingForDependencies, Executing) |
            (Executing, WaitingForResources) |
            (Executing, Coordinating) |
            (Executing, Finalizing) |
            (Executing, Failed(_)) |
            (WaitingForResources, Executing) |
            (Coordinating, Finalizing) |
            (Coordinating, Failed(_)) |
            (Finalizing, Completed) |
            (Finalizing, Failed(_)) |
            (_, Suspended) |
            (Suspended, _)
        )
    }
    
    /// Notify state change listeners
    async fn notify_state_change(&self, orch_state: &OrchestrationState, agent_state: &AgentState) {
        let listeners = self.state_listeners.read().await;
        for listener in listeners.iter() {
            listener(orch_state, agent_state);
        }
    }
}

impl OrchestrationState {
    fn new() -> Self {
        Self {
            phase: OrchestrationPhase::Initializing,
            active_agents: HashSet::new(),
            completed_agents: HashSet::new(),
            failed_agents: HashMap::new(),
            shared_goals: Vec::new(),
            convergence_metrics: ConvergenceMetrics::default(),
            started_at: chrono::Utc::now(),
            last_state_change: chrono::Utc::now(),
            timeout: None,
        }
    }
}

impl AgentState {
    fn new(agent_id: Uuid) -> Self {
        Self {
            agent_id,
            phase: AgentPhase::Idle,
            status: AgentStatus::Idle,
            dependencies: Vec::new(),
            dependents: Vec::new(),
            local_memory: LocalMemory::new(),
            last_checkpoint: None,
            transition_history: VecDeque::new(),
            convergence_contribution: 0.0,
            lock_requests: Vec::new(),
            current_locks: HashSet::new(),
        }
    }
}

impl SharedMemory {
    fn new() -> Self {
        Self {
            facts: HashMap::new(),
            goals: HashMap::new(),
            artifacts: HashMap::new(),
            constraints: Vec::new(),
            metrics: HashMap::new(),
            version: 0,
        }
    }
}

impl LocalMemory {
    fn new() -> Self {
        Self {
            working_set: HashMap::new(),
            intermediate_results: Vec::new(),
            private_goals: Vec::new(),
            resource_reservations: Vec::new(),
            version: 0,
        }
    }
}

impl ConvergenceDetector {
    fn new(consensus_threshold: f64, stability_window: Duration) -> Self {
        Self {
            consensus_threshold,
            stability_window,
            metrics_history: Arc::new(RwLock::new(VecDeque::new())),
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    async fn check_convergence(
        &self,
        state: &Arc<RwLock<OrchestrationState>>,
        agent_states: &Arc<RwLock<HashMap<Uuid, AgentState>>>,
    ) -> Result<ConvergenceMetrics, StateError> {
        // Implementation would calculate convergence metrics
        // based on agent agreement, goal completion, etc.
        let metrics = ConvergenceMetrics::default();
        
        let mut history = self.metrics_history.write().await;
        history.push_back(metrics.clone());
        
        // Keep only recent history
        while history.len() > 100 {
            history.pop_front();
        }
        
        Ok(metrics)
    }
}

impl DeadlockDetector {
    fn new(detection_interval: Duration) -> Self {
        Self {
            detection_interval,
            last_detection: Arc::new(RwLock::new(Instant::now())),
            detected_cycles: Arc::new(RwLock::new(Vec::new())),
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    async fn detect_cycles(
        &self,
        agent_states: &Arc<RwLock<HashMap<Uuid, AgentState>>>,
    ) -> Result<Vec<DependencyCycle>, StateError> {
        let states = agent_states.read().await;
        
        // Build dependency graph
        let mut graph: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for (agent_id, state) in states.iter() {
            graph.insert(*agent_id, state.dependencies.clone());
        }
        
        // Detect cycles using DFS
        let cycles = self.find_cycles_dfs(&graph);
        
        // Convert to DependencyCycle structs
        let dependency_cycles: Vec<DependencyCycle> = cycles.into_iter().map(|cycle| DependencyCycle {
            agents: cycle,
            resources: Vec::new(), // Would be populated based on actual resource dependencies
            cycle_type: CycleType::AgentDependency,
            severity: DeadlockSeverity::Medium,
            detected_at: chrono::Utc::now(),
        }).collect();
        
        if !dependency_cycles.is_empty() {
            let mut detected = self.detected_cycles.write().await;
            detected.extend(dependency_cycles.clone());
        }
        
        Ok(dependency_cycles)
    }
    
    fn find_cycles_dfs(&self, graph: &HashMap<Uuid, Vec<Uuid>>) -> Vec<Vec<Uuid>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        let mut on_path = HashSet::new();
        
        for &node in graph.keys() {
            if !visited.contains(&node) {
                self.dfs_visit(node, graph, &mut visited, &mut path, &mut on_path, &mut cycles);
            }
        }
        
        cycles
    }
    
    fn dfs_visit(
        &self,
        node: Uuid,
        graph: &HashMap<Uuid, Vec<Uuid>>,
        visited: &mut HashSet<Uuid>,
        path: &mut Vec<Uuid>,
        on_path: &mut HashSet<Uuid>,
        cycles: &mut Vec<Vec<Uuid>>,
    ) {
        visited.insert(node);
        path.push(node);
        on_path.insert(node);
        
        if let Some(neighbors) = graph.get(&node) {
            for &neighbor in neighbors {
                if on_path.contains(&neighbor) {
                    // Found a cycle
                    let cycle_start = path.iter().position(|&x| x == neighbor).unwrap();
                    let cycle = path[cycle_start..].to_vec();
                    cycles.push(cycle);
                } else if !visited.contains(&neighbor) {
                    self.dfs_visit(neighbor, graph, visited, path, on_path, cycles);
                }
            }
        }
        
        path.pop();
        on_path.remove(&node);
    }
}

impl ConflictResolver {
    fn new() -> Self {
        Self {
            strategies: Vec::new(),
            active_conflicts: Arc::new(RwLock::new(HashMap::new())),
            resolution_history: Arc::new(RwLock::new(VecDeque::new())),
        }
    }
    
    async fn handle_conflict(&self, conflict: Conflict) -> Result<ConflictResolution, StateError> {
        let mut conflicts = self.active_conflicts.write().await;
        conflicts.insert(conflict.id.clone(), conflict.clone());
        
        // Try resolution strategies in order of priority
        for strategy in &self.strategies {
            if strategy.can_resolve(&conflict) {
                // Placeholder for actual resolution logic
                let resolution = ConflictResolution {
                    conflict_id: conflict.id.clone(),
                    strategy_used: strategy.strategy_name().to_string(),
                    resolution_time: Duration::from_millis(100),
                    success: true,
                    outcome: "Conflict resolved".to_string(),
                    resolved_at: chrono::Utc::now(),
                };
                
                conflicts.remove(&conflict.id);
                
                let mut history = self.resolution_history.write().await;
                history.push_back(resolution.clone());
                
                return Ok(resolution);
            }
        }
        
        Err(StateError::ConflictResolutionFailed(format!(
            "No strategy could resolve conflict: {}", conflict.id
        )))
    }
}

impl Default for ConvergenceMetrics {
    fn default() -> Self {
        Self {
            consensus_score: 0.0,
            goal_completion_rate: 0.0,
            agent_agreement_rate: 0.0,
            fact_stability_score: 0.0,
            convergence_velocity: 0.0,
            last_major_change: chrono::Utc::now(),
            stability_window: Duration::from_secs(10),
        }
    }
}

// Custom Debug implementations for structs with trait objects
impl std::fmt::Debug for AgentStateMachine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentStateMachine")
            .field("state", &"<OrchestrationState>")
            .field("agent_states", &"<AgentStates>")
            .field("shared_memory", &"<SharedMemory>")
            .field("convergence_detector", &"<ConvergenceDetector>")
            .field("deadlock_detector", &"<DeadlockDetector>")
            .field("conflict_resolver", &"<ConflictResolver>")
            .field("state_listeners", &format!("<{} listeners>", "?"))
            .field("concurrency_limiter", &"<Semaphore>")
            .field("telemetry", &self.telemetry.is_some())
            .finish()
    }
}

impl std::fmt::Debug for ConvergenceDetector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConvergenceDetector")
            .field("consensus_threshold", &self.consensus_threshold)
            .field("stability_window", &self.stability_window)
            .field("metrics_history", &"<VecDeque<ConvergenceMetrics>>")
            .field("listeners", &"<Vec<Box<dyn ConvergenceListener>>>")
            .finish()
    }
}

impl std::fmt::Debug for DeadlockDetector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeadlockDetector")
            .field("detection_interval", &self.detection_interval)
            .field("last_detection", &"<Instant>")
            .field("detected_cycles", &"<Vec<DependencyCycle>>")
            .field("listeners", &"<Vec<Box<dyn DeadlockListener>>>")
            .finish()
    }
}

impl std::fmt::Debug for ConflictResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConflictResolver")
            .field("strategies", &format!("<{} strategies>", self.strategies.len()))
            .field("active_conflicts", &"<HashMap<String, Conflict>>")
            .field("resolution_history", &"<VecDeque<ConflictResolution>>")
            .finish()
    }
}
