//! AI Integration Module
//!
//! This module provides integration with various AI models and providers,
//! with primary support for local Ollama instances.

pub mod client;
pub mod manager;
#[cfg(test)]
mod tests;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents different AI providers that can be used
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AIProvider {
    Ollama,
    OpenAI,
    Anthropic,
    Custom(String),
}

/// AI model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub provider: AIProvider,
    pub description: Option<String>,
    pub context_window: usize,
    pub parameters: Option<ModelParameters>,
    pub capabilities: Vec<ModelCapability>,
}

/// Model parameters for generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParameters {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<i32>,
    pub max_tokens: Option<usize>,
    pub stop: Option<Vec<String>>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
}

/// Capabilities that a model supports
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelCapability {
    CodeGeneration,
    CodeAnalysis,
    TextGeneration,
    Conversation,
    Summarization,
    Translation,
    QuestionAnswering,
}

/// Represents a chat message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Role of a message in a conversation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Request for chat completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub parameters: Option<ModelParameters>,
    pub stream: bool,
}

/// Response from chat completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: ChatMessage,
    pub model: String,
    pub usage: Option<TokenUsage>,
    pub finish_reason: Option<String>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

/// Streaming response chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStreamChunk {
    pub delta: String,
    pub finish_reason: Option<String>,
    pub model: String,
}

/// Trait for AI clients that can communicate with different providers
#[async_trait]
pub trait AIClient {
    /// Get information about available models
    async fn list_models(&self) -> Result<Vec<ModelInfo>, AIError>;

    /// Get information about a specific model
    async fn get_model_info(&self, model_name: &str) -> Result<ModelInfo, AIError>;

    /// Send a chat completion request
    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, AIError>;

    /// Send a streaming chat completion request
    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<ChatStreamChunk, AIError>>, AIError>;

    /// Check if the AI service is available
    async fn health_check(&self) -> Result<bool, AIError>;
}

/// Errors that can occur when working with AI services
#[derive(Debug, thiserror::Error)]
pub enum AIError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Parsing error: {0}")]
    ParseError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl Default for ModelParameters {
    fn default() -> Self {
        Self {
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: None,
            max_tokens: Some(1000),
            stop: None,
            frequency_penalty: None,
            presence_penalty: None,
        }
    }
}

// Re-export main AI components
// Ollama client exports available when needed
pub use manager::AIManager;

impl ChatMessage {
    /// Create a new system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
            metadata: None,
        }
    }

    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
            metadata: None,
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            metadata: None,
        }
    }

    /// Add metadata to the message
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        if self.metadata.is_none() {
            self.metadata = Some(HashMap::new());
        }
        self.metadata.as_mut().unwrap().insert(key.into(), value);
        self
    }
}
