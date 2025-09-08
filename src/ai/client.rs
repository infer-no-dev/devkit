//! Ollama Client Implementation
//!
//! This module provides a client for interacting with Ollama local LLM instances.

use super::{AIClient, AIError, AIProvider, ChatMessage, ChatRequest, ChatResponse, ChatStreamChunk, MessageRole, ModelCapability, ModelInfo, ModelParameters, TokenUsage};
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
    fn convert_model_info(&self, model: OllamaModel, show_response: Option<OllamaShowResponse>) -> ModelInfo {
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
        if name_lower.contains("code") || name_lower.contains("llama") || 
           name_lower.contains("mistral") || name_lower.contains("deepseek") ||
           name_lower.contains("qwen") || name_lower.contains("phi") {
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
        
        let response = self.client
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

        let response = self.client
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
        
        let messages: Vec<OllamaChatMessage> = request.messages
            .iter()
            .map(Self::convert_message)
            .collect();

        let options = request.parameters
            .as_ref()
            .map(Self::convert_parameters)
            .unwrap_or_default();

        let ollama_request = OllamaChatRequest {
            model: request.model.clone(),
            messages,
            stream: Some(false),
            options,
        };

        let response = self.client
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
            return Err(AIError::ParseError("No message content in response".to_string()));
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
        
        let messages: Vec<OllamaChatMessage> = request.messages
            .iter()
            .map(Self::convert_message)
            .collect();

        let options = request.parameters
            .as_ref()
            .map(Self::convert_parameters)
            .unwrap_or_default();

        let ollama_request = OllamaChatRequest {
            model: request.model.clone(),
            messages,
            stream: Some(true),
            options,
        };

        let response = self.client
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
                                    let error = AIError::ParseError(format!("Failed to parse streaming response: {}", e));
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
