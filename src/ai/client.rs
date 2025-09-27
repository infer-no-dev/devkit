//! AI Client Implementations
//!
//! This module provides clients for interacting with various AI providers:
//! - Ollama: Local LLM instances
//! - OpenAI: GPT models via API
//! - Anthropic: Claude models via API

use super::{
    AIClient, AIError, AIProvider, ChatMessage, ChatRequest, ChatResponse, ChatStreamChunk,
    MessageRole, ModelCapability, ModelInfo, ModelParameters, TokenUsage,
};
use async_trait::async_trait;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

/// Ollama client configuration
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub endpoint: String,
    pub timeout: Duration,
    pub max_retries: usize,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_string(),
            timeout: Duration::from_secs(300), // 5 minutes for long generations
            max_retries: 3,
        }
    }
}

/// Ollama HTTP client for local LLM interactions
pub struct OllamaClient {
    client: Client,
    config: OllamaConfig,
}

/// Ollama API request/response structures
#[derive(Debug, Serialize, Deserialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw: Option<bool>,
    #[serde(flatten)]
    options: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(flatten)]
    options: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<OllamaChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    done: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    load_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_eval_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_eval_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eval_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eval_duration: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaModel {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<OllamaModelDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaModelDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    families: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameter_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quantization_level: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaShowRequest {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaShowResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modelfile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<OllamaModelDetails>,
}

impl OllamaClient {
    /// Create a new Ollama client
    pub fn new(config: OllamaConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    /// Create a new Ollama client with default configuration
    pub fn default() -> Self {
        Self::new(OllamaConfig::default())
    }

    /// Convert Ollama model to our ModelInfo structure
    fn convert_model_info(
        &self,
        model: OllamaModel,
        show_response: Option<OllamaShowResponse>,
    ) -> ModelInfo {
        let capabilities = self.infer_capabilities(&model.name);
        let context_window = self.infer_context_window(&model.name);

        ModelInfo {
            name: model.name.clone(),
            provider: AIProvider::Ollama,
            description: show_response
                .as_ref()
                .and_then(|r| r.modelfile.as_ref())
                .map(|_m| format!("Ollama model: {}", model.name)),
            context_window,
            parameters: Some(ModelParameters::default()),
            capabilities,
        }
    }

    /// Infer model capabilities based on model name
    fn infer_capabilities(&self, model_name: &str) -> Vec<ModelCapability> {
        let name_lower = model_name.to_lowercase();
        let mut capabilities = vec![
            ModelCapability::TextGeneration,
            ModelCapability::Conversation,
        ];

        // Add code capabilities for code-focused models
        if name_lower.contains("code")
            || name_lower.contains("llama")
            || name_lower.contains("mistral")
            || name_lower.contains("deepseek")
            || name_lower.contains("qwen")
            || name_lower.contains("phi")
        {
            capabilities.push(ModelCapability::CodeGeneration);
            capabilities.push(ModelCapability::CodeAnalysis);
        }

        // Add additional capabilities for larger models
        if name_lower.contains("70b") || name_lower.contains("34b") || name_lower.contains("33b") {
            capabilities.push(ModelCapability::Summarization);
            capabilities.push(ModelCapability::QuestionAnswering);
        }

        capabilities
    }

    /// Infer context window size based on model name
    fn infer_context_window(&self, model_name: &str) -> usize {
        let name_lower = model_name.to_lowercase();

        // Known context windows for popular models
        if name_lower.contains("llama2") {
            4096
        } else if name_lower.contains("llama3") || name_lower.contains("llama-3") {
            8192
        } else if name_lower.contains("mistral") {
            32768
        } else if name_lower.contains("mixtral") {
            32768
        } else if name_lower.contains("codellama") {
            16384
        } else if name_lower.contains("phi") {
            2048
        } else if name_lower.contains("gemma") {
            8192
        } else if name_lower.contains("qwen") {
            32768
        } else {
            4096 // Default
        }
    }

    /// Convert our ChatMessage to Ollama format
    fn convert_message(message: &ChatMessage) -> OllamaChatMessage {
        let role = match message.role {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::Tool => "user", // Ollama doesn't have tool role, map to user
        };

        OllamaChatMessage {
            role: role.to_string(),
            content: message.content.clone(),
        }
    }

    /// Convert model parameters to Ollama options
    fn convert_parameters(params: &ModelParameters) -> HashMap<String, Value> {
        let mut options = HashMap::new();

        if let Some(temp) = params.temperature {
            options.insert("temperature".to_string(), Value::from(temp));
        }

        if let Some(top_p) = params.top_p {
            options.insert("top_p".to_string(), Value::from(top_p));
        }

        if let Some(top_k) = params.top_k {
            options.insert("top_k".to_string(), Value::from(top_k));
        }

        if let Some(max_tokens) = params.max_tokens {
            options.insert("num_predict".to_string(), Value::from(max_tokens));
        }

        if let Some(stop) = &params.stop {
            options.insert("stop".to_string(), Value::from(stop.clone()));
        }

        options
    }

    /// Handle HTTP response errors
    async fn handle_response_error(response: Response) -> AIError {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        match status.as_u16() {
            404 => AIError::ModelNotFound(text),
            401 | 403 => AIError::AuthenticationError(text),
            429 => AIError::RateLimitExceeded,
            500..=599 => AIError::ServiceUnavailable(format!("Server error: {}", text)),
            _ => AIError::NetworkError(format!("HTTP {}: {}", status, text)),
        }
    }
}

#[async_trait]
impl AIClient for OllamaClient {
    async fn list_models(&self) -> Result<Vec<ModelInfo>, AIError> {
        let url = format!("{}/api/tags", self.config.endpoint);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::handle_response_error(response).await);
        }

        let models_response: OllamaModelsResponse = response
            .json()
            .await
            .map_err(|e| AIError::ParseError(e.to_string()))?;

        let mut model_infos = Vec::new();
        for model in models_response.models {
            let model_info = self.convert_model_info(model, None);
            model_infos.push(model_info);
        }

        Ok(model_infos)
    }

    async fn get_model_info(&self, model_name: &str) -> Result<ModelInfo, AIError> {
        let url = format!("{}/api/show", self.config.endpoint);
        let request = OllamaShowRequest {
            name: model_name.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::handle_response_error(response).await);
        }

        let show_response: OllamaShowResponse = response
            .json()
            .await
            .map_err(|e| AIError::ParseError(e.to_string()))?;

        // Create a mock model for conversion
        let model = OllamaModel {
            name: model_name.to_string(),
            modified_at: None,
            size: None,
            digest: None,
            details: show_response.details.clone(),
        };

        Ok(self.convert_model_info(model, Some(show_response)))
    }

    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, AIError> {
        let url = format!("{}/api/chat", self.config.endpoint);

        let messages: Vec<OllamaChatMessage> =
            request.messages.iter().map(Self::convert_message).collect();

        let options = request
            .parameters
            .as_ref()
            .map(Self::convert_parameters)
            .unwrap_or_default();

        let ollama_request = OllamaChatRequest {
            model: request.model.clone(),
            messages,
            stream: Some(false),
            options,
        };

        let response = self
            .client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::handle_response_error(response).await);
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .map_err(|e| AIError::ParseError(e.to_string()))?;

        let message = if let Some(msg) = ollama_response.message {
            ChatMessage::assistant(msg.content)
        } else if let Some(content) = ollama_response.response {
            ChatMessage::assistant(content)
        } else {
            return Err(AIError::ParseError(
                "No message content in response".to_string(),
            ));
        };

        let usage = if let (Some(prompt_tokens), Some(completion_tokens)) = (
            ollama_response.prompt_eval_count.map(|c| c as usize),
            ollama_response.eval_count.map(|c| c as usize),
        ) {
            Some(TokenUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            })
        } else {
            None
        };

        Ok(ChatResponse {
            message,
            model: request.model,
            usage,
            finish_reason: Some("stop".to_string()),
        })
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
    ) -> Result<mpsc::Receiver<Result<ChatStreamChunk, AIError>>, AIError> {
        let url = format!("{}/api/chat", self.config.endpoint);

        let messages: Vec<OllamaChatMessage> =
            request.messages.iter().map(Self::convert_message).collect();

        let options = request
            .parameters
            .as_ref()
            .map(Self::convert_parameters)
            .unwrap_or_default();

        let ollama_request = OllamaChatRequest {
            model: request.model.clone(),
            messages,
            stream: Some(true),
            options,
        };

        let response = self
            .client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::handle_response_error(response).await);
        }

        let (tx, rx) = mpsc::channel(100);
        let model_name = request.model.clone();

        // Spawn task to handle streaming response
        tokio::spawn(async move {
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        let chunk_str = String::from_utf8_lossy(chunk.as_ref());
                        buffer.push_str(&chunk_str);

                        // Process complete lines
                        while let Some(line_end) = buffer.find('\n') {
                            let line = buffer[..line_end].trim().to_string();
                            buffer.drain(..=line_end);

                            if line.is_empty() {
                                continue;
                            }

                            match serde_json::from_str::<OllamaResponse>(&line) {
                                Ok(ollama_response) => {
                                    let delta = if let Some(msg) = &ollama_response.message {
                                        msg.content.clone()
                                    } else if let Some(response) = &ollama_response.response {
                                        response.clone()
                                    } else {
                                        String::new()
                                    };

                                    let finish_reason = if ollama_response.done == Some(true) {
                                        Some("stop".to_string())
                                    } else {
                                        None
                                    };

                                    let chunk = ChatStreamChunk {
                                        delta,
                                        finish_reason,
                                        model: model_name.clone(),
                                    };

                                    if tx.send(Ok(chunk)).await.is_err() {
                                        break; // Receiver dropped
                                    }
                                }
                                Err(e) => {
                                    let error = AIError::ParseError(format!(
                                        "Failed to parse streaming response: {}",
                                        e
                                    ));
                                    let _ = tx.send(Err(error)).await;
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let error = AIError::NetworkError(format!("Stream error: {}", e));
                        let _ = tx.send(Err(error)).await;
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    async fn health_check(&self) -> Result<bool, AIError> {
        let url = format!("{}/api/tags", self.config.endpoint);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

// ================================================================================================
// OpenAI Client Implementation
// ================================================================================================

/// OpenAI client configuration
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub base_url: String,
    pub organization: Option<String>,
    pub timeout: Duration,
    pub max_retries: usize,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            base_url: "https://api.openai.com/v1".to_string(),
            organization: std::env::var("OPENAI_ORG_ID").ok(),
            timeout: Duration::from_secs(120),
            max_retries: 3,
        }
    }
}

/// OpenAI HTTP client for GPT models
pub struct OpenAIClient {
    client: Client,
    config: OpenAIConfig,
}

/// OpenAI API request/response structures
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIChatResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIChoice {
    index: usize,
    message: OpenAIChatMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIModelsList {
    object: String,
    data: Vec<OpenAIModel>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIModel {
    id: String,
    object: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    owned_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIStreamChunk {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIStreamChoice {
    index: usize,
    delta: OpenAIStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIStreamDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}

impl OpenAIClient {
    /// Create a new OpenAI client
    pub fn new(config: OpenAIConfig) -> Result<Self, AIError> {
        if config.api_key.is_empty() {
            return Err(AIError::ConfigurationError(
                "OpenAI API key is required. Set OPENAI_API_KEY environment variable.".to_string(),
            ));
        }

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", config.api_key)
                .parse()
                .map_err(|e| AIError::ConfigurationError(format!("Invalid API key: {}", e)))?,
        );

        if let Some(org) = &config.organization {
            headers.insert(
                "OpenAI-Organization",
                org.parse().map_err(|e| {
                    AIError::ConfigurationError(format!("Invalid organization ID: {}", e))
                })?,
            );
        }

        let client = Client::builder()
            .timeout(config.timeout)
            .default_headers(headers)
            .build()
            .map_err(|e| {
                AIError::ConfigurationError(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(Self { client, config })
    }

    /// Create a new OpenAI client with default configuration
    pub fn default() -> Result<Self, AIError> {
        Self::new(OpenAIConfig::default())
    }

    /// Convert our ChatMessage to OpenAI format
    fn convert_message(message: &ChatMessage) -> OpenAIChatMessage {
        let role = match message.role {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::Tool => "user", // Map tool to user for OpenAI
        };

        OpenAIChatMessage {
            role: role.to_string(),
            content: message.content.clone(),
        }
    }

    /// Convert model parameters to OpenAI request
    fn apply_parameters(request: &mut OpenAIChatRequest, params: &ModelParameters) {
        if let Some(temp) = params.temperature {
            request.temperature = Some(temp);
        }
        if let Some(top_p) = params.top_p {
            request.top_p = Some(top_p);
        }
        if let Some(max_tokens) = params.max_tokens {
            request.max_tokens = Some(max_tokens);
        }
        if let Some(stop) = &params.stop {
            request.stop = Some(stop.clone());
        }
        if let Some(freq_penalty) = params.frequency_penalty {
            request.frequency_penalty = Some(freq_penalty);
        }
        if let Some(pres_penalty) = params.presence_penalty {
            request.presence_penalty = Some(pres_penalty);
        }
    }

    /// Get model capabilities based on model name
    fn get_model_capabilities(model_name: &str) -> Vec<ModelCapability> {
        let name_lower = model_name.to_lowercase();
        let mut capabilities = vec![
            ModelCapability::TextGeneration,
            ModelCapability::Conversation,
            ModelCapability::QuestionAnswering,
            ModelCapability::Summarization,
        ];

        // Add code capabilities for code-focused models
        if name_lower.contains("code") || name_lower.starts_with("gpt-4") {
            capabilities.push(ModelCapability::CodeGeneration);
            capabilities.push(ModelCapability::CodeAnalysis);
        }

        capabilities
    }

    /// Get context window size based on model name
    fn get_context_window(model_name: &str) -> usize {
        let name_lower = model_name.to_lowercase();

        if name_lower.contains("gpt-4-turbo") || name_lower.contains("gpt-4-1106") {
            128000
        } else if name_lower.contains("gpt-4-32k") {
            32768
        } else if name_lower.contains("gpt-4") {
            8192
        } else if name_lower.contains("gpt-3.5-turbo-16k") {
            16384
        } else if name_lower.contains("gpt-3.5-turbo") {
            4096
        } else {
            4096 // Default
        }
    }

    /// Handle OpenAI HTTP response errors
    async fn handle_response_error(response: Response) -> AIError {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        match status.as_u16() {
            401 => AIError::AuthenticationError("Invalid API key".to_string()),
            403 => AIError::AuthenticationError("Access denied".to_string()),
            404 => AIError::ModelNotFound(text),
            429 => AIError::RateLimitExceeded,
            500..=599 => AIError::ServiceUnavailable(format!("OpenAI server error: {}", text)),
            _ => AIError::NetworkError(format!("HTTP {}: {}", status, text)),
        }
    }
}

#[async_trait]
impl AIClient for OpenAIClient {
    async fn list_models(&self) -> Result<Vec<ModelInfo>, AIError> {
        let url = format!("{}/models", self.config.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::handle_response_error(response).await);
        }

        let models_response: OpenAIModelsList = response
            .json()
            .await
            .map_err(|e| AIError::ParseError(e.to_string()))?;

        let model_infos = models_response
            .data
            .into_iter()
            .map(|model| ModelInfo {
                name: model.id.clone(),
                provider: AIProvider::OpenAI,
                description: Some(format!("OpenAI model: {}", model.id)),
                context_window: Self::get_context_window(&model.id),
                parameters: Some(ModelParameters::default()),
                capabilities: Self::get_model_capabilities(&model.id),
            })
            .collect();

        Ok(model_infos)
    }

    async fn get_model_info(&self, model_name: &str) -> Result<ModelInfo, AIError> {
        // OpenAI doesn't have a specific model info endpoint, so we construct it
        Ok(ModelInfo {
            name: model_name.to_string(),
            provider: AIProvider::OpenAI,
            description: Some(format!("OpenAI model: {}", model_name)),
            context_window: Self::get_context_window(model_name),
            parameters: Some(ModelParameters::default()),
            capabilities: Self::get_model_capabilities(model_name),
        })
    }

    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, AIError> {
        let url = format!("{}/chat/completions", self.config.base_url);

        let messages: Vec<OpenAIChatMessage> =
            request.messages.iter().map(Self::convert_message).collect();

        let mut openai_request = OpenAIChatRequest {
            model: request.model.clone(),
            messages,
            temperature: None,
            top_p: None,
            max_tokens: None,
            stop: None,
            frequency_penalty: None,
            presence_penalty: None,
            stream: Some(false),
        };

        if let Some(params) = &request.parameters {
            Self::apply_parameters(&mut openai_request, params);
        }

        let response = self
            .client
            .post(&url)
            .json(&openai_request)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::handle_response_error(response).await);
        }

        let openai_response: OpenAIChatResponse = response
            .json()
            .await
            .map_err(|e| AIError::ParseError(e.to_string()))?;

        let choice = openai_response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| AIError::ParseError("No choices in response".to_string()))?;

        let message = ChatMessage::assistant(choice.message.content);
        let usage = Some(TokenUsage {
            prompt_tokens: openai_response.usage.prompt_tokens,
            completion_tokens: openai_response.usage.completion_tokens,
            total_tokens: openai_response.usage.total_tokens,
        });

        Ok(ChatResponse {
            message,
            model: request.model,
            usage,
            finish_reason: choice.finish_reason,
        })
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
    ) -> Result<mpsc::Receiver<Result<ChatStreamChunk, AIError>>, AIError> {
        let url = format!("{}/chat/completions", self.config.base_url);

        let messages: Vec<OpenAIChatMessage> =
            request.messages.iter().map(Self::convert_message).collect();

        let mut openai_request = OpenAIChatRequest {
            model: request.model.clone(),
            messages,
            temperature: None,
            top_p: None,
            max_tokens: None,
            stop: None,
            frequency_penalty: None,
            presence_penalty: None,
            stream: Some(true),
        };

        if let Some(params) = &request.parameters {
            Self::apply_parameters(&mut openai_request, params);
        }

        let response = self
            .client
            .post(&url)
            .json(&openai_request)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::handle_response_error(response).await);
        }

        let (tx, rx) = mpsc::channel(100);
        let model_name = request.model.clone();

        // Spawn task to handle streaming response
        tokio::spawn(async move {
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        let chunk_str = String::from_utf8_lossy(chunk.as_ref());
                        buffer.push_str(&chunk_str);

                        // Process complete lines (SSE format: "data: {json}\n\n")
                        while let Some(line_end) = buffer.find("\n\n") {
                            let line = buffer[..line_end].trim().to_string();
                            buffer.drain(..=line_end + 1);

                            if line.is_empty() || !line.starts_with("data: ") {
                                continue;
                            }

                            let json_str = &line[6..]; // Remove "data: " prefix

                            if json_str == "[DONE]" {
                                break;
                            }

                            match serde_json::from_str::<OpenAIStreamChunk>(json_str) {
                                Ok(openai_chunk) => {
                                    if let Some(choice) = openai_chunk.choices.into_iter().next() {
                                        let delta = choice.delta.content.unwrap_or_default();

                                        let chunk = ChatStreamChunk {
                                            delta,
                                            finish_reason: choice.finish_reason,
                                            model: model_name.clone(),
                                        };

                                        if tx.send(Ok(chunk)).await.is_err() {
                                            break; // Receiver dropped
                                        }
                                    }
                                }
                                Err(e) => {
                                    let error = AIError::ParseError(format!(
                                        "Failed to parse streaming response: {}",
                                        e
                                    ));
                                    let _ = tx.send(Err(error)).await;
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let error = AIError::NetworkError(format!("Stream error: {}", e));
                        let _ = tx.send(Err(error)).await;
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    async fn health_check(&self) -> Result<bool, AIError> {
        let url = format!("{}/models", self.config.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

// ================================================================================================
// Anthropic Client Implementation
// ================================================================================================

/// Anthropic client configuration
#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout: Duration,
    pub max_retries: usize,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
            base_url: "https://api.anthropic.com".to_string(),
            timeout: Duration::from_secs(120),
            max_retries: 3,
        }
    }
}

/// Anthropic HTTP client for Claude models
pub struct AnthropicClient {
    client: Client,
    config: AnthropicConfig,
}

/// Anthropic API request/response structures
#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessageRequest {
    model: String,
    max_tokens: usize,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessageResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<AnthropicContent>,
    model: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicUsage {
    input_tokens: usize,
    output_tokens: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicStreamChunk {
    #[serde(rename = "type")]
    chunk_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<AnthropicStreamMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delta: Option<AnthropicStreamDelta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicStreamMessage {
    id: String,
    #[serde(rename = "type")]
    message_type: String,
    role: String,
    content: Vec<String>,
    model: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicStreamDelta {
    #[serde(rename = "type")]
    delta_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequence: Option<String>,
}

impl AnthropicClient {
    /// Create a new Anthropic client
    pub fn new(config: AnthropicConfig) -> Result<Self, AIError> {
        if config.api_key.is_empty() {
            return Err(AIError::ConfigurationError(
                "Anthropic API key is required. Set ANTHROPIC_API_KEY environment variable."
                    .to_string(),
            ));
        }

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            config
                .api_key
                .parse()
                .map_err(|e| AIError::ConfigurationError(format!("Invalid API key: {}", e)))?,
        );
        headers.insert(
            "anthropic-version",
            "2023-06-01".parse().map_err(|e| {
                AIError::ConfigurationError(format!("Invalid version header: {}", e))
            })?,
        );
        headers.insert(
            "content-type",
            "application/json".parse().map_err(|e| {
                AIError::ConfigurationError(format!("Invalid content-type header: {}", e))
            })?,
        );

        let client = Client::builder()
            .timeout(config.timeout)
            .default_headers(headers)
            .build()
            .map_err(|e| {
                AIError::ConfigurationError(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(Self { client, config })
    }

    /// Create a new Anthropic client with default configuration
    pub fn default() -> Result<Self, AIError> {
        Self::new(AnthropicConfig::default())
    }

    /// Convert our ChatMessage to Anthropic format, separating system messages
    fn convert_messages(messages: &[ChatMessage]) -> (Option<String>, Vec<AnthropicMessage>) {
        let mut system_content: Option<String> = None;
        let mut anthropic_messages = Vec::new();

        for message in messages {
            match message.role {
                MessageRole::System => {
                    // Anthropic uses a separate system field
                    if let Some(ref mut existing) = system_content {
                        existing.push('\n');
                        existing.push_str(&message.content);
                    } else {
                        system_content = Some(message.content.clone());
                    }
                }
                MessageRole::User | MessageRole::Tool => {
                    anthropic_messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: message.content.clone(),
                    });
                }
                MessageRole::Assistant => {
                    anthropic_messages.push(AnthropicMessage {
                        role: "assistant".to_string(),
                        content: message.content.clone(),
                    });
                }
            }
        }

        (system_content, anthropic_messages)
    }

    /// Convert model parameters to Anthropic request
    fn apply_parameters(request: &mut AnthropicMessageRequest, params: &ModelParameters) {
        if let Some(temp) = params.temperature {
            request.temperature = Some(temp);
        }
        if let Some(top_p) = params.top_p {
            request.top_p = Some(top_p);
        }
        if let Some(top_k) = params.top_k {
            request.top_k = Some(top_k);
        }
        if let Some(max_tokens) = params.max_tokens {
            request.max_tokens = max_tokens;
        }
        if let Some(stop) = &params.stop {
            request.stop_sequences = Some(stop.clone());
        }
    }

    /// Get model capabilities for Claude models
    fn get_model_capabilities(model_name: &str) -> Vec<ModelCapability> {
        let name_lower = model_name.to_lowercase();
        let mut capabilities = vec![
            ModelCapability::TextGeneration,
            ModelCapability::Conversation,
            ModelCapability::QuestionAnswering,
            ModelCapability::Summarization,
        ];

        // All Claude models are good with code
        if name_lower.contains("claude") {
            capabilities.push(ModelCapability::CodeGeneration);
            capabilities.push(ModelCapability::CodeAnalysis);
        }

        capabilities
    }

    /// Get context window size based on model name
    fn get_context_window(model_name: &str) -> usize {
        let name_lower = model_name.to_lowercase();

        if name_lower.contains("claude-3-opus") {
            200000
        } else if name_lower.contains("claude-3-sonnet") {
            200000
        } else if name_lower.contains("claude-3-haiku") {
            200000
        } else if name_lower.contains("claude-2.1") {
            200000
        } else if name_lower.contains("claude-2") {
            100000
        } else if name_lower.contains("claude-instant") {
            100000
        } else {
            100000 // Default
        }
    }

    /// Handle Anthropic HTTP response errors
    async fn handle_response_error(response: Response) -> AIError {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        match status.as_u16() {
            401 => AIError::AuthenticationError("Invalid API key".to_string()),
            403 => AIError::AuthenticationError("Access denied".to_string()),
            404 => AIError::ModelNotFound(text),
            429 => AIError::RateLimitExceeded,
            500..=599 => AIError::ServiceUnavailable(format!("Anthropic server error: {}", text)),
            _ => AIError::NetworkError(format!("HTTP {}: {}", status, text)),
        }
    }
}

#[async_trait]
impl AIClient for AnthropicClient {
    async fn list_models(&self) -> Result<Vec<ModelInfo>, AIError> {
        // Anthropic doesn't have a public models endpoint, so we return known models
        let models = vec![
            ("claude-3-opus-20240229", "Claude 3 Opus"),
            ("claude-3-sonnet-20240229", "Claude 3 Sonnet"),
            ("claude-3-haiku-20240307", "Claude 3 Haiku"),
            ("claude-2.1", "Claude 2.1"),
            ("claude-2.0", "Claude 2.0"),
            ("claude-instant-1.2", "Claude Instant 1.2"),
        ];

        let model_infos = models
            .into_iter()
            .map(|(id, name)| ModelInfo {
                name: id.to_string(),
                provider: AIProvider::Anthropic,
                description: Some(format!("Anthropic model: {}", name)),
                context_window: Self::get_context_window(id),
                parameters: Some(ModelParameters::default()),
                capabilities: Self::get_model_capabilities(id),
            })
            .collect();

        Ok(model_infos)
    }

    async fn get_model_info(&self, model_name: &str) -> Result<ModelInfo, AIError> {
        // Anthropic doesn't have a specific model info endpoint, so we construct it
        Ok(ModelInfo {
            name: model_name.to_string(),
            provider: AIProvider::Anthropic,
            description: Some(format!("Anthropic model: {}", model_name)),
            context_window: Self::get_context_window(model_name),
            parameters: Some(ModelParameters::default()),
            capabilities: Self::get_model_capabilities(model_name),
        })
    }

    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, AIError> {
        let url = format!("{}/v1/messages", self.config.base_url);

        let (system, messages) = Self::convert_messages(&request.messages);

        let mut anthropic_request = AnthropicMessageRequest {
            model: request.model.clone(),
            max_tokens: 1000, // Default
            messages,
            system,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: Some(false),
        };

        if let Some(params) = &request.parameters {
            Self::apply_parameters(&mut anthropic_request, params);
        }

        let response = self
            .client
            .post(&url)
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::handle_response_error(response).await);
        }

        let anthropic_response: AnthropicMessageResponse = response
            .json()
            .await
            .map_err(|e| AIError::ParseError(e.to_string()))?;

        let content = anthropic_response
            .content
            .into_iter()
            .map(|c| c.text)
            .collect::<Vec<_>>()
            .join("");

        let message = ChatMessage::assistant(content);
        let usage = Some(TokenUsage {
            prompt_tokens: anthropic_response.usage.input_tokens,
            completion_tokens: anthropic_response.usage.output_tokens,
            total_tokens: anthropic_response.usage.input_tokens
                + anthropic_response.usage.output_tokens,
        });

        Ok(ChatResponse {
            message,
            model: request.model,
            usage,
            finish_reason: anthropic_response.stop_reason,
        })
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
    ) -> Result<mpsc::Receiver<Result<ChatStreamChunk, AIError>>, AIError> {
        let url = format!("{}/v1/messages", self.config.base_url);

        let (system, messages) = Self::convert_messages(&request.messages);

        let mut anthropic_request = AnthropicMessageRequest {
            model: request.model.clone(),
            max_tokens: 1000, // Default
            messages,
            system,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: Some(true),
        };

        if let Some(params) = &request.parameters {
            Self::apply_parameters(&mut anthropic_request, params);
        }

        let response = self
            .client
            .post(&url)
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::handle_response_error(response).await);
        }

        let (tx, rx) = mpsc::channel(100);
        let model_name = request.model.clone();

        // Spawn task to handle streaming response
        tokio::spawn(async move {
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        let chunk_str = String::from_utf8_lossy(chunk.as_ref());
                        buffer.push_str(&chunk_str);

                        // Process complete lines (SSE format: "data: {json}\n\n")
                        while let Some(line_end) = buffer.find("\n\n") {
                            let line = buffer[..line_end].trim().to_string();
                            buffer.drain(..=line_end + 1);

                            if line.is_empty() || !line.starts_with("data: ") {
                                continue;
                            }

                            let json_str = &line[6..]; // Remove "data: " prefix

                            if json_str == "[DONE]" {
                                break;
                            }

                            match serde_json::from_str::<AnthropicStreamChunk>(json_str) {
                                Ok(anthropic_chunk) => {
                                    let (delta, finish_reason) = match anthropic_chunk
                                        .chunk_type
                                        .as_str()
                                    {
                                        "content_block_delta" => {
                                            if let Some(delta) = anthropic_chunk.delta {
                                                (delta.text.unwrap_or_default(), delta.stop_reason)
                                            } else {
                                                (String::new(), None)
                                            }
                                        }
                                        "message_delta" => {
                                            if let Some(delta) = anthropic_chunk.delta {
                                                (String::new(), delta.stop_reason)
                                            } else {
                                                (String::new(), None)
                                            }
                                        }
                                        _ => (String::new(), None),
                                    };

                                    let chunk = ChatStreamChunk {
                                        delta,
                                        finish_reason,
                                        model: model_name.clone(),
                                    };

                                    if tx.send(Ok(chunk)).await.is_err() {
                                        break; // Receiver dropped
                                    }
                                }
                                Err(e) => {
                                    let error = AIError::ParseError(format!(
                                        "Failed to parse streaming response: {}",
                                        e
                                    ));
                                    let _ = tx.send(Err(error)).await;
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let error = AIError::NetworkError(format!("Stream error: {}", e));
                        let _ = tx.send(Err(error)).await;
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    async fn health_check(&self) -> Result<bool, AIError> {
        // Anthropic doesn't have a dedicated health check endpoint
        // We'll use a simple message request with minimal tokens as a health check
        let test_request = AnthropicMessageRequest {
            model: "claude-3-haiku-20240307".to_string(),
            max_tokens: 1,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: "Hi".to_string(),
            }],
            system: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: Some(false),
        };

        let url = format!("{}/v1/messages", self.config.base_url);
        match self.client.post(&url).json(&test_request).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

// Export client implementations publicly
// These are already defined in this module, so no need to re-export
