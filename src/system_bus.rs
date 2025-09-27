//! System-wide communication bus for connecting UI, Agents, Code Generation, and AI systems.
//!
//! This module provides a unified message bus that enables seamless communication
//! between all components of the agentic development environment.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use uuid::Uuid;

/// System-wide event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    // UI Events
    UIUserInput {
        input: String,
        context: serde_json::Value,
    },
    UICommandRequest {
        command: String,
        args: Vec<String>,
    },
    UIProgressUpdate {
        component: String,
        progress: f32,
        message: String,
    },
    UINotification {
        level: NotificationLevel,
        title: String,
        message: String,
    },

    // Agent Events
    AgentTaskStarted {
        agent_id: String,
        task_id: String,
        description: String,
    },
    AgentTaskCompleted {
        agent_id: String,
        task_id: String,
        result: serde_json::Value,
    },
    AgentTaskFailed {
        agent_id: String,
        task_id: String,
        error: String,
    },
    AgentStatusChanged {
        agent_id: String,
        status: String,
    },

    // Code Generation Events
    CodeGenerationRequested {
        request_id: String,
        prompt: String,
        language: Option<String>,
    },
    CodeGenerationCompleted {
        request_id: String,
        code: String,
        metadata: serde_json::Value,
    },
    CodeGenerationFailed {
        request_id: String,
        error: String,
    },

    // AI Events
    AIModelLoaded {
        provider: String,
        model: String,
    },
    AIRequestStarted {
        request_id: String,
        prompt: String,
    },
    AIResponseReceived {
        request_id: String,
        response: String,
        tokens_used: usize,
    },
    AIError {
        request_id: String,
        error: String,
    },

    // Shell Events
    ShellCommandExecuted {
        command: String,
        exit_code: i32,
        output: String,
    },
    ShellProjectSetup {
        project_type: String,
        name: String,
        success: bool,
    },

    // Context Events
    ContextAnalysisStarted {
        path: String,
    },
    ContextAnalysisCompleted {
        path: String,
        symbols_found: usize,
    },
    ContextUpdated {
        file_path: String,
        change_type: String,
    },

    // Workflow Events
    WorkflowCreated {
        workflow_id: String,
        name: String,
    },
    WorkflowStarted {
        workflow_id: String,
    },
    WorkflowCompleted {
        workflow_id: String,
    },
    WorkflowFailed {
        workflow_id: String,
        error: String,
    },

    // Configuration Events
    ConfigChanged {
        section: String,
        key: String,
        value: serde_json::Value,
    },
    ConfigValidationError {
        section: String,
        error: String,
    },
}

/// Notification levels for UI events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

/// System event message with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessage {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: String,
    pub event: SystemEvent,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// System component identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemComponent {
    UI,
    AgentSystem,
    CodeGenerator,
    AIManager,
    ShellManager,
    ContextManager,
    ConfigManager,
    CoordinationEngine,
    Custom(String),
}

/// System bus that manages communication between all components
#[derive(Debug)]
pub struct SystemBus {
    // Broadcast channel for system-wide events
    event_sender: broadcast::Sender<SystemMessage>,

    // Keep at least one receiver alive so broadcast channel doesn't close
    #[allow(dead_code)]
    _keep_alive_receiver: broadcast::Receiver<SystemMessage>,

    // Direct communication channels for each component
    component_channels: Arc<RwLock<HashMap<SystemComponent, mpsc::UnboundedSender<SystemMessage>>>>,

    // Event subscribers
    subscribers: Arc<RwLock<HashMap<String, (SystemComponent, EventFilter)>>>,
}

/// Filter for subscribing to specific event types
#[derive(Debug, Clone)]
pub struct EventFilter {
    pub event_types: Option<Vec<String>>, // None means all events
    pub sources: Option<Vec<String>>,     // None means all sources
}

/// System bus handle for components to interact with the bus
#[derive(Debug, Clone)]
pub struct SystemBusHandle {
    event_sender: broadcast::Sender<SystemMessage>,
    component: SystemComponent,
}

impl SystemBus {
    /// Create a new system bus
    pub fn new() -> Self {
        let (event_sender, keep_alive_receiver) = broadcast::channel(1000);

        Self {
            event_sender,
            _keep_alive_receiver: keep_alive_receiver,
            component_channels: Arc::new(RwLock::new(HashMap::new())),
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a component with the system bus
    pub async fn register_component(
        &self,
        component: SystemComponent,
    ) -> (SystemBusHandle, mpsc::UnboundedReceiver<SystemMessage>) {
        let (sender, receiver) = mpsc::unbounded_channel();

        {
            let mut channels = self.component_channels.write().await;
            channels.insert(component.clone(), sender);
        }

        let handle = SystemBusHandle {
            event_sender: self.event_sender.clone(),
            component,
        };

        (handle, receiver)
    }

    /// Subscribe to system events with a filter
    pub async fn subscribe(
        &self,
        subscriber_id: String,
        component: SystemComponent,
        filter: EventFilter,
    ) -> broadcast::Receiver<SystemMessage> {
        {
            let mut subscribers = self.subscribers.write().await;
            subscribers.insert(subscriber_id, (component, filter));
        }

        self.event_sender.subscribe()
    }

    /// Publish an event to the system bus
    pub async fn publish(&self, event: SystemMessage) -> Result<(), String> {
        // Send to broadcast channel
        self.event_sender
            .send(event.clone())
            .map_err(|e| format!("Failed to broadcast event: {}", e))?;

        // Send to specific component channels if applicable
        let channels = self.component_channels.read().await;

        // Determine target components based on event type
        let target_components = self.get_target_components(&event.event);

        for target in target_components {
            if let Some(sender) = channels.get(&target) {
                let _ = sender.send(event.clone()); // Don't fail if component is busy
            }
        }

        Ok(())
    }

    /// Get broadcast receiver for monitoring all events
    pub fn get_broadcast_receiver(&self) -> broadcast::Receiver<SystemMessage> {
        self.event_sender.subscribe()
    }

    /// Get component statistics
    pub async fn get_component_stats(&self) -> HashMap<SystemComponent, ComponentStats> {
        let channels = self.component_channels.read().await;
        let mut stats = HashMap::new();

        for (component, sender) in channels.iter() {
            stats.insert(
                component.clone(),
                ComponentStats {
                    is_connected: !sender.is_closed(),
                    queue_size: 0, // Would need more instrumentation for accurate queue size
                },
            );
        }

        stats
    }

    /// Determine which components should receive a specific event
    fn get_target_components(&self, event: &SystemEvent) -> Vec<SystemComponent> {
        match event {
            // UI events go to relevant components
            SystemEvent::UIUserInput { .. } | SystemEvent::UICommandRequest { .. } => {
                vec![
                    SystemComponent::AgentSystem,
                    SystemComponent::CodeGenerator,
                    SystemComponent::CoordinationEngine,
                ]
            }

            // Agent events go to UI and coordination
            SystemEvent::AgentTaskStarted { .. }
            | SystemEvent::AgentTaskCompleted { .. }
            | SystemEvent::AgentTaskFailed { .. } => {
                vec![SystemComponent::UI, SystemComponent::CoordinationEngine]
            }

            // Code generation events go to UI and agents
            SystemEvent::CodeGenerationRequested { .. } => {
                vec![SystemComponent::AIManager, SystemComponent::UI]
            }
            SystemEvent::CodeGenerationCompleted { .. }
            | SystemEvent::CodeGenerationFailed { .. } => {
                vec![SystemComponent::UI, SystemComponent::AgentSystem]
            }

            // AI events go to UI and whoever requested
            SystemEvent::AIRequestStarted { .. }
            | SystemEvent::AIResponseReceived { .. }
            | SystemEvent::AIError { .. } => {
                vec![
                    SystemComponent::UI,
                    SystemComponent::CodeGenerator,
                    SystemComponent::AgentSystem,
                ]
            }

            // Context events go to agents and UI
            SystemEvent::ContextAnalysisCompleted { .. } | SystemEvent::ContextUpdated { .. } => {
                vec![
                    SystemComponent::UI,
                    SystemComponent::AgentSystem,
                    SystemComponent::CodeGenerator,
                ]
            }

            // Configuration events go everywhere
            SystemEvent::ConfigChanged { .. } => {
                vec![
                    SystemComponent::UI,
                    SystemComponent::AgentSystem,
                    SystemComponent::CodeGenerator,
                    SystemComponent::AIManager,
                    SystemComponent::ContextManager,
                ]
            }

            // All other events are broadcast to UI for monitoring
            _ => vec![SystemComponent::UI],
        }
    }
}

impl SystemBusHandle {
    /// Publish an event from this component
    pub async fn publish(&self, event: SystemEvent) -> Result<(), String> {
        let message = SystemMessage {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            source: format!("{:?}", self.component),
            event,
            metadata: HashMap::new(),
        };

        self.event_sender
            .send(message)
            .map_err(|e| format!("Failed to publish event: {}", e))?;
        Ok(())
    }

    /// Publish an event with custom metadata
    pub async fn publish_with_metadata(
        &self,
        event: SystemEvent,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        let message = SystemMessage {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            source: format!("{:?}", self.component),
            event,
            metadata,
        };

        self.event_sender
            .send(message)
            .map_err(|e| format!("Failed to publish event: {}", e))?;
        Ok(())
    }

    /// Get the component identifier
    pub fn component(&self) -> &SystemComponent {
        &self.component
    }
}

/// Component connection statistics
#[derive(Debug, Clone)]
pub struct ComponentStats {
    pub is_connected: bool,
    pub queue_size: usize,
}

impl SystemMessage {
    /// Create a new system message
    pub fn new(source: String, event: SystemEvent) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            source,
            event,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the message
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if this message matches a filter
    pub fn matches_filter(&self, filter: &EventFilter) -> bool {
        // Check event type filter
        if let Some(event_types) = &filter.event_types {
            let event_type = format!("{:?}", self.event)
                .split('(')
                .next()
                .unwrap_or("")
                .to_string();
            if !event_types.contains(&event_type) {
                return false;
            }
        }

        // Check source filter
        if let Some(sources) = &filter.sources {
            if !sources.contains(&self.source) {
                return false;
            }
        }

        true
    }
}

impl EventFilter {
    /// Create a filter that accepts all events
    pub fn all() -> Self {
        Self {
            event_types: None,
            sources: None,
        }
    }

    /// Create a filter for specific event types
    pub fn event_types(event_types: Vec<String>) -> Self {
        Self {
            event_types: Some(event_types),
            sources: None,
        }
    }

    /// Create a filter for specific sources
    pub fn sources(sources: Vec<String>) -> Self {
        Self {
            event_types: None,
            sources: Some(sources),
        }
    }

    /// Create a filter for specific event types and sources
    pub fn event_types_and_sources(event_types: Vec<String>, sources: Vec<String>) -> Self {
        Self {
            event_types: Some(event_types),
            sources: Some(sources),
        }
    }
}

impl std::fmt::Display for SystemComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemComponent::UI => write!(f, "UI"),
            SystemComponent::AgentSystem => write!(f, "AgentSystem"),
            SystemComponent::CodeGenerator => write!(f, "CodeGenerator"),
            SystemComponent::AIManager => write!(f, "AIManager"),
            SystemComponent::ShellManager => write!(f, "ShellManager"),
            SystemComponent::ContextManager => write!(f, "ContextManager"),
            SystemComponent::ConfigManager => write!(f, "ConfigManager"),
            SystemComponent::CoordinationEngine => write!(f, "CoordinationEngine"),
            SystemComponent::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}
