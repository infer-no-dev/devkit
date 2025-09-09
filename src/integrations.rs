//! Integration adapters for connecting components to the system bus.
//!
//! This module provides adapters that wrap existing components and connect them
//! to the system-wide communication bus.

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use crate::system_bus::{SystemBus, SystemBusHandle, SystemEvent, SystemComponent, SystemMessage};
use crate::agents::{AgentSystem, AgentTask, TaskPriority};
use crate::ai::AIManager;
use crate::shell::{ShellManager, CommandOperation};
use crate::codegen::CodeGenerator;
use crate::context::ContextManager;

/// Integrated system that connects all components via the system bus
#[derive(Debug)]
pub struct IntegratedSystem {
    pub system_bus: Arc<SystemBus>,
    pub agent_adapter: Arc<AgentSystemAdapter>,
    pub ai_adapter: Arc<AIManagerAdapter>,
    pub shell_adapter: Arc<ShellManagerAdapter>,
    pub codegen_adapter: Arc<CodeGeneratorAdapter>,
    pub context_adapter: Option<Arc<ContextManagerAdapter>>,
}

/// Agent system adapter that integrates AgentSystem with the system bus
#[derive(Debug)]
pub struct AgentSystemAdapter {
    agent_system: Arc<AgentSystem>,
    bus_handle: SystemBusHandle,
    message_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<SystemMessage>>>>,
}

/// AI manager adapter that integrates AIManager with the system bus
#[derive(Debug)]
pub struct AIManagerAdapter {
    ai_manager: Arc<AIManager>,
    bus_handle: SystemBusHandle,
    message_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<SystemMessage>>>>,
}

/// Shell manager adapter that integrates ShellManager with the system bus
#[derive(Debug)]
pub struct ShellManagerAdapter {
    shell_manager: Arc<ShellManager>,
    bus_handle: SystemBusHandle,
    message_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<SystemMessage>>>>,
}

/// Code generator adapter that integrates CodeGenerator with the system bus
#[derive(Debug)]
pub struct CodeGeneratorAdapter {
    code_generator: Arc<CodeGenerator>,
    bus_handle: SystemBusHandle,
    message_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<SystemMessage>>>>,
}

/// Context manager adapter that integrates ContextManager with the system bus
#[derive(Debug)]
pub struct ContextManagerAdapter {
    context_manager: Arc<ContextManager>,
    bus_handle: SystemBusHandle,
    message_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<SystemMessage>>>>,
}

impl IntegratedSystem {
    /// Create a new integrated system
    pub async fn new(
        ai_manager: Arc<AIManager>,
        shell_manager: Arc<ShellManager>,
        code_generator: Arc<CodeGenerator>,
        context_manager: Option<Arc<ContextManager>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let system_bus = Arc::new(SystemBus::new());
        
        // Create agent system with AI manager
        let agent_system = Arc::new(AgentSystem::with_ai_manager(ai_manager.clone()));
        agent_system.initialize().await;
        
        // Create adapters
        let agent_adapter = Arc::new(AgentSystemAdapter::new(system_bus.clone(), agent_system).await?);
        let ai_adapter = Arc::new(AIManagerAdapter::new(system_bus.clone(), ai_manager).await?);
        let shell_adapter = Arc::new(ShellManagerAdapter::new(system_bus.clone(), shell_manager).await?);
        let codegen_adapter = Arc::new(CodeGeneratorAdapter::new(system_bus.clone(), code_generator).await?);
        
        let context_adapter = if let Some(ctx_mgr) = context_manager {
            Some(Arc::new(ContextManagerAdapter::new(system_bus.clone(), ctx_mgr).await?))
        } else {
            None
        };
        
        let integrated_system = Self {
            system_bus,
            agent_adapter,
            ai_adapter,
            shell_adapter,
            codegen_adapter,
            context_adapter,
        };
        
        // Start message processing for all adapters
        integrated_system.start_message_processing().await;
        
        Ok(integrated_system)
    }
    
    /// Start message processing for all adapters
    async fn start_message_processing(&self) {
        // Start agent system message processing
        let agent_adapter = self.agent_adapter.clone();
        tokio::spawn(async move {
            agent_adapter.start_message_processing().await;
        });
        
        // Start AI manager message processing
        let ai_adapter = self.ai_adapter.clone();
        tokio::spawn(async move {
            ai_adapter.start_message_processing().await;
        });
        
        // Start shell manager message processing
        let shell_adapter = self.shell_adapter.clone();
        tokio::spawn(async move {
            shell_adapter.start_message_processing().await;
        });
        
        // Start code generator message processing
        let codegen_adapter = self.codegen_adapter.clone();
        tokio::spawn(async move {
            codegen_adapter.start_message_processing().await;
        });
        
        // Start context manager message processing if available
        if let Some(context_adapter) = &self.context_adapter {
            let context_adapter = context_adapter.clone();
            tokio::spawn(async move {
                context_adapter.start_message_processing().await;
            });
        }
    }
    
    /// Process a user command through the integrated system
    pub async fn process_user_command(&self, command: String, args: Vec<String>) -> Result<String, Box<dyn std::error::Error>> {
        // Publish user command event
        let event = SystemEvent::UICommandRequest { command: command.clone(), args: args.clone() };
        self.system_bus.publish(SystemMessage::new("User".to_string(), event)).await?;
        
        // Route command to appropriate component
        match command.as_str() {
            "generate" => {
                if let Some(prompt) = args.first() {
                    self.agent_adapter.request_code_generation(prompt.clone()).await
                } else {
                    Err("Generate command requires a prompt".into())
                }
            }
            "analyze" => {
                if let Some(path) = args.first() {
                    self.agent_adapter.request_code_analysis(path.clone()).await
                } else {
                    Err("Analyze command requires a path".into())
                }
            }
            "shell" => {
                if let Some(cmd) = args.first() {
                    self.shell_adapter.execute_shell_command(cmd.clone()).await
                } else {
                    Err("Shell command requires a command to execute".into())
                }
            }
            _ => Err(format!("Unknown command: {}", command).into()),
        }
    }
    
    /// Get system status
    pub async fn get_system_status(&self) -> SystemStatus {
        let component_stats = self.system_bus.get_component_stats().await;
        
        SystemStatus {
            components_connected: component_stats.len(),
            agent_system_ready: component_stats.get(&SystemComponent::AgentSystem).map(|s| s.is_connected).unwrap_or(false),
            ai_manager_ready: component_stats.get(&SystemComponent::AIManager).map(|s| s.is_connected).unwrap_or(false),
            shell_manager_ready: component_stats.get(&SystemComponent::ShellManager).map(|s| s.is_connected).unwrap_or(false),
            code_generator_ready: component_stats.get(&SystemComponent::CodeGenerator).map(|s| s.is_connected).unwrap_or(false),
        }
    }
}

/// System status information
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub components_connected: usize,
    pub agent_system_ready: bool,
    pub ai_manager_ready: bool,
    pub shell_manager_ready: bool,
    pub code_generator_ready: bool,
}

impl AgentSystemAdapter {
    async fn new(system_bus: Arc<SystemBus>, agent_system: Arc<AgentSystem>) -> Result<Self, Box<dyn std::error::Error>> {
        let (bus_handle, message_receiver) = system_bus.register_component(SystemComponent::AgentSystem).await;
        
        Ok(Self {
            agent_system,
            bus_handle,
            message_receiver: Arc::new(RwLock::new(Some(message_receiver))),
        })
    }
    
    async fn start_message_processing(&self) {
        if let Some(mut receiver) = self.message_receiver.write().await.take() {
            let agent_system = self.agent_system.clone();
            let bus_handle = self.bus_handle.clone();
            
            tokio::spawn(async move {
                while let Some(message) = receiver.recv().await {
                    if let Err(e) = Self::handle_message(&agent_system, &bus_handle, message).await {
                        eprintln!("Agent system message handling error: {}", e);
                    }
                }
            });
        }
    }
    
    async fn handle_message(
        agent_system: &Arc<AgentSystem>,
        bus_handle: &SystemBusHandle,
        message: SystemMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match message.event {
            SystemEvent::UICommandRequest { command, args } => {
                match command.as_str() {
                    "generate" | "analyze" | "debug" | "test" => {
                        let prompt = args.join(" ");
                        let task_type = match command.as_str() {
                            "generate" => "generate_code",
                            "analyze" => "analyze_code",
                            "debug" => "debug_code",
                            "test" => "generate_tests",
                            _ => "general",
                        };
                        
                        let task = AgentTask {
                            id: uuid::Uuid::new_v4().to_string(),
                            task_type: task_type.to_string(),
                            description: prompt,
                            context: serde_json::json!({"source": "ui_command", "args": args}),
                            priority: TaskPriority::High,
                            deadline: None,
                            metadata: std::collections::HashMap::new(),
                        };
                        
                        // Notify task started
                        bus_handle.publish(SystemEvent::AgentTaskStarted {
                            agent_id: "system".to_string(),
                            task_id: task.id.clone(),
                            description: task.description.clone(),
                        }).await?;
                        
                        // Submit task
                        match agent_system.submit_task(task.clone()).await {
                            Ok(result) => {
                                bus_handle.publish(SystemEvent::AgentTaskCompleted {
                                    agent_id: "system".to_string(),
                                    task_id: task.id,
                                    result: serde_json::json!({
                                        "output": result.output,
                                        "artifacts": result.artifacts.len()
                                    }),
                                }).await?;
                            }
                            Err(e) => {
                                bus_handle.publish(SystemEvent::AgentTaskFailed {
                                    agent_id: "system".to_string(),
                                    task_id: task.id,
                                    error: e.to_string(),
                                }).await?;
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Request code generation through the agent system
    pub async fn request_code_generation(&self, prompt: String) -> Result<String, Box<dyn std::error::Error>> {
        let task = AgentTask {
            id: uuid::Uuid::new_v4().to_string(),
            task_type: "generate_code".to_string(),
            description: prompt,
            context: serde_json::json!({"source": "api_request"}),
            priority: TaskPriority::High,
            deadline: None,
            metadata: std::collections::HashMap::new(),
        };
        
        match self.agent_system.submit_task(task).await {
            Ok(result) => Ok(result.output),
            Err(e) => Err(e.into()),
        }
    }
    
    /// Request code analysis through the agent system
    pub async fn request_code_analysis(&self, path: String) -> Result<String, Box<dyn std::error::Error>> {
        let task = AgentTask {
            id: uuid::Uuid::new_v4().to_string(),
            task_type: "analyze_code".to_string(),
            description: format!("Analyze code at path: {}", path),
            context: serde_json::json!({"path": path, "source": "api_request"}),
            priority: TaskPriority::High,
            deadline: None,
            metadata: std::collections::HashMap::new(),
        };
        
        match self.agent_system.submit_task(task).await {
            Ok(result) => Ok(result.output),
            Err(e) => Err(e.into()),
        }
    }
}

impl ShellManagerAdapter {
    async fn new(system_bus: Arc<SystemBus>, shell_manager: Arc<ShellManager>) -> Result<Self, Box<dyn std::error::Error>> {
        let (bus_handle, message_receiver) = system_bus.register_component(SystemComponent::ShellManager).await;
        
        Ok(Self {
            shell_manager,
            bus_handle,
            message_receiver: Arc::new(RwLock::new(Some(message_receiver))),
        })
    }
    
    async fn start_message_processing(&self) {
        if let Some(mut receiver) = self.message_receiver.write().await.take() {
            let shell_manager = self.shell_manager.clone();
            let bus_handle = self.bus_handle.clone();
            
            tokio::spawn(async move {
                while let Some(_message) = receiver.recv().await {
                    // Handle shell-specific messages here
                }
            });
        }
    }
    
    /// Execute a shell command and publish the result
    pub async fn execute_shell_command(&self, command: String) -> Result<String, Box<dyn std::error::Error>> {
        let result = self.shell_manager.execute_command(&command, None).await?;
        
        // Publish shell command execution event
        self.bus_handle.publish(SystemEvent::ShellCommandExecuted {
            command: command.clone(),
            exit_code: result.exit_code,
            output: result.stdout.clone(),
        }).await?;
        
        if result.exit_code == 0 {
            Ok(result.stdout)
        } else {
            Err(format!("Command failed with exit code {}: {}", result.exit_code, result.stderr).into())
        }
    }
}

// Implement similar patterns for other adapters
impl AIManagerAdapter {
    async fn new(system_bus: Arc<SystemBus>, ai_manager: Arc<AIManager>) -> Result<Self, Box<dyn std::error::Error>> {
        let (bus_handle, message_receiver) = system_bus.register_component(SystemComponent::AIManager).await;
        
        Ok(Self {
            ai_manager,
            bus_handle,
            message_receiver: Arc::new(RwLock::new(Some(message_receiver))),
        })
    }
    
    async fn start_message_processing(&self) {
        if let Some(mut receiver) = self.message_receiver.write().await.take() {
            while let Some(_message) = receiver.recv().await {
                // Handle AI-specific messages
            }
        }
    }
}

impl CodeGeneratorAdapter {
    async fn new(system_bus: Arc<SystemBus>, code_generator: Arc<CodeGenerator>) -> Result<Self, Box<dyn std::error::Error>> {
        let (bus_handle, message_receiver) = system_bus.register_component(SystemComponent::CodeGenerator).await;
        
        Ok(Self {
            code_generator,
            bus_handle,
            message_receiver: Arc::new(RwLock::new(Some(message_receiver))),
        })
    }
    
    async fn start_message_processing(&self) {
        if let Some(mut receiver) = self.message_receiver.write().await.take() {
            while let Some(_message) = receiver.recv().await {
                // Handle code generator specific messages
            }
        }
    }
}

impl ContextManagerAdapter {
    async fn new(system_bus: Arc<SystemBus>, context_manager: Arc<ContextManager>) -> Result<Self, Box<dyn std::error::Error>> {
        let (bus_handle, message_receiver) = system_bus.register_component(SystemComponent::ContextManager).await;
        
        Ok(Self {
            context_manager,
            bus_handle,
            message_receiver: Arc::new(RwLock::new(Some(message_receiver))),
        })
    }
    
    async fn start_message_processing(&self) {
        if let Some(mut receiver) = self.message_receiver.write().await.take() {
            while let Some(_message) = receiver.recv().await {
                // Handle context manager specific messages
            }
        }
    }
}
