//! AI Manager Module
//!
//! This module provides a high-level manager for coordinating different AI providers,
//! handling configuration, and providing a unified interface for the rest of the application.

use super::{AIClient, AIError, AIProvider, ChatRequest, ChatResponse, ChatStreamChunk, ModelInfo};
use super::client::{OllamaClient, OllamaConfig, OpenAIClient, OpenAIConfig, AnthropicClient, AnthropicConfig};
use crate::config::{AIModelConfig, Config};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// AI Manager that coordinates different AI providers
pub struct AIManager {
    config: AIModelConfig,
    clients: HashMap<AIProvider, Box<dyn AIClient + Send + Sync>>,
    model_cache: Arc<RwLock<HashMap<String, ModelInfo>>>,
    default_provider: AIProvider,
}

impl AIManager {
    /// Create a new AI manager from configuration
    pub async fn from_config(config: &Config) -> Result<Self, AIError> {
        let ai_config = config.codegen.ai_model_settings.clone();
        let mut manager = Self::new(ai_config).await?;
        manager.refresh_models().await?;
        Ok(manager)
    }

    /// Create a new AI manager with the given configuration
    pub async fn new(config: AIModelConfig) -> Result<Self, AIError> {
        let mut clients: HashMap<AIProvider, Box<dyn AIClient + Send + Sync>> = HashMap::new();
        
        // Initialize Ollama client
        let ollama_config = OllamaConfig {
            endpoint: config.ollama.endpoint.clone(),
            timeout: std::time::Duration::from_secs(config.ollama.timeout_seconds),
            max_retries: config.ollama.max_retries,
        };
        let ollama_client = OllamaClient::new(ollama_config);
        clients.insert(AIProvider::Ollama, Box::new(ollama_client));

        // Initialize OpenAI client if configuration is provided
        if let Some(openai_config) = &config.openai {
            if let Some(api_key) = &openai_config.api_key {
                let openai_client_config = OpenAIConfig {
                    api_key: api_key.clone(),
                    base_url: openai_config.base_url.clone().unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
                    organization: openai_config.organization.clone(),
                    timeout: std::time::Duration::from_secs(120),
                    max_retries: 3,
                };
                
                match OpenAIClient::new(openai_client_config) {
                    Ok(openai_client) => {
                        clients.insert(AIProvider::OpenAI, Box::new(openai_client));
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to initialize OpenAI client: {}", e);
                    }
                }
            } else {
                eprintln!("Warning: OpenAI configuration provided but no API key found");
            }
        }

        // Initialize Anthropic client if configuration is provided
        if let Some(anthropic_config) = &config.anthropic {
            if let Some(api_key) = &anthropic_config.api_key {
                let anthropic_client_config = AnthropicConfig {
                    api_key: api_key.clone(),
                    base_url: anthropic_config.base_url.clone().unwrap_or_else(|| "https://api.anthropic.com".to_string()),
                    timeout: std::time::Duration::from_secs(120),
                    max_retries: 3,
                };
                
                match AnthropicClient::new(anthropic_client_config) {
                    Ok(anthropic_client) => {
                        clients.insert(AIProvider::Anthropic, Box::new(anthropic_client));
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to initialize Anthropic client: {}", e);
                    }
                }
            } else {
                eprintln!("Warning: Anthropic configuration provided but no API key found");
            }
        }

        let default_provider = match config.default_provider.as_str() {
            "ollama" => AIProvider::Ollama,
            "openai" => AIProvider::OpenAI,
            "anthropic" => AIProvider::Anthropic,
            provider => AIProvider::Custom(provider.to_string()),
        };

        Ok(Self {
            config,
            clients,
            model_cache: Arc::new(RwLock::new(HashMap::new())),
            default_provider,
        })
    }

    /// Get the default AI provider
    pub fn default_provider(&self) -> &AIProvider {
        &self.default_provider
    }

    /// Get the default model name
    pub fn default_model(&self) -> &str {
        &self.config.default_model
    }

    /// List all available models from all providers
    pub async fn list_all_models(&self) -> Result<Vec<ModelInfo>, AIError> {
        let mut all_models = Vec::new();

        for (provider, client) in &self.clients {
            match client.list_models().await {
                Ok(models) => {
                    all_models.extend(models);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to list models from {:?}: {}", provider, e);
                }
            }
        }

        Ok(all_models)
    }

    /// Get models from a specific provider
    pub async fn list_models(&self, provider: &AIProvider) -> Result<Vec<ModelInfo>, AIError> {
        let client = self.clients.get(provider)
            .ok_or_else(|| AIError::ConfigurationError(format!("Provider {:?} not configured", provider)))?;
        
        client.list_models().await
    }

    /// Get information about a specific model
    pub async fn get_model_info(&self, model_name: &str, provider: Option<&AIProvider>) -> Result<ModelInfo, AIError> {
        // Check cache first
        {
            let cache = self.model_cache.read().await;
            if let Some(info) = cache.get(model_name) {
                return Ok(info.clone());
            }
        }

        let provider = provider.unwrap_or(&self.default_provider);
        let client = self.clients.get(provider)
            .ok_or_else(|| AIError::ConfigurationError(format!("Provider {:?} not configured", provider)))?;

        let model_info = client.get_model_info(model_name).await?;

        // Cache the result
        {
            let mut cache = self.model_cache.write().await;
            cache.insert(model_name.to_string(), model_info.clone());
        }

        Ok(model_info)
    }

    /// Send a chat completion request using the default provider and model
    pub async fn chat_completion_default(&self, request: ChatRequest) -> Result<ChatResponse, AIError> {
        self.chat_completion(request, None).await
    }

    /// Send a chat completion request to a specific provider
    pub async fn chat_completion(&self, mut request: ChatRequest, provider: Option<&AIProvider>) -> Result<ChatResponse, AIError> {
        let provider = provider.unwrap_or(&self.default_provider);
        
        // Use default model if none specified in request
        if request.model.is_empty() {
            request.model = match provider {
                AIProvider::Ollama => {
                    self.config.ollama.default_model.clone()
                        .unwrap_or_else(|| self.config.default_model.clone())
                }
                AIProvider::OpenAI => {
                    self.config.openai.as_ref()
                        .map(|c| c.default_model.clone())
                        .unwrap_or_else(|| "gpt-3.5-turbo".to_string())
                }
                AIProvider::Anthropic => {
                    self.config.anthropic.as_ref()
                        .map(|c| c.default_model.clone())
                        .unwrap_or_else(|| "claude-3-haiku-20240307".to_string())
                }
                _ => self.config.default_model.clone(),
            };
        }

        let client = self.clients.get(provider)
            .ok_or_else(|| AIError::ConfigurationError(format!("Provider {:?} not configured", provider)))?;

        client.chat_completion(request).await
    }

    /// Send a streaming chat completion request using the default provider and model
    pub async fn chat_completion_stream_default(
        &self, 
        request: ChatRequest
    ) -> Result<tokio::sync::mpsc::Receiver<Result<ChatStreamChunk, AIError>>, AIError> {
        self.chat_completion_stream(request, None).await
    }

    /// Send a streaming chat completion request to a specific provider
    pub async fn chat_completion_stream(
        &self,
        mut request: ChatRequest,
        provider: Option<&AIProvider>,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<ChatStreamChunk, AIError>>, AIError> {
        let provider = provider.unwrap_or(&self.default_provider);
        
        // Use default model if none specified in request
        if request.model.is_empty() {
            request.model = match provider {
                AIProvider::Ollama => {
                    self.config.ollama.default_model.clone()
                        .unwrap_or_else(|| self.config.default_model.clone())
                }
                AIProvider::OpenAI => {
                    self.config.openai.as_ref()
                        .map(|c| c.default_model.clone())
                        .unwrap_or_else(|| "gpt-3.5-turbo".to_string())
                }
                AIProvider::Anthropic => {
                    self.config.anthropic.as_ref()
                        .map(|c| c.default_model.clone())
                        .unwrap_or_else(|| "claude-3-haiku-20240307".to_string())
                }
                _ => self.config.default_model.clone(),
            };
        }

        let client = self.clients.get(provider)
            .ok_or_else(|| AIError::ConfigurationError(format!("Provider {:?} not configured", provider)))?;

        client.chat_completion_stream(request).await
    }

    /// Check health of all configured providers
    pub async fn health_check_all(&self) -> HashMap<AIProvider, bool> {
        let mut results = HashMap::new();

        for (provider, client) in &self.clients {
            let health = client.health_check().await.unwrap_or(false);
            results.insert(provider.clone(), health);
        }

        results
    }

    /// Check health of a specific provider
    pub async fn health_check(&self, provider: &AIProvider) -> Result<bool, AIError> {
        let client = self.clients.get(provider)
            .ok_or_else(|| AIError::ConfigurationError(format!("Provider {:?} not configured", provider)))?;

        client.health_check().await
    }

    /// Refresh the model cache
    pub async fn refresh_models(&mut self) -> Result<(), AIError> {
        let mut new_cache = HashMap::new();

        for (provider, client) in &self.clients {
            match client.list_models().await {
                Ok(models) => {
                    for model in models {
                        new_cache.insert(model.name.clone(), model);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to refresh models from {:?}: {}", provider, e);
                }
            }
        }

        {
            let mut cache = self.model_cache.write().await;
            *cache = new_cache;
        }

        Ok(())
    }

    /// Clear the model cache
    pub async fn clear_cache(&self) {
        let mut cache = self.model_cache.write().await;
        cache.clear();
    }

    /// Get the current configuration
    pub fn config(&self) -> &AIModelConfig {
        &self.config
    }

    /// Update the configuration and reinitialize clients if necessary
    pub async fn update_config(&mut self, new_config: AIModelConfig) -> Result<(), AIError> {
        // Check if we need to reinitialize clients
        let ollama_changed = self.config.ollama.endpoint != new_config.ollama.endpoint ||
                           self.config.ollama.timeout_seconds != new_config.ollama.timeout_seconds ||
                           self.config.ollama.max_retries != new_config.ollama.max_retries;

        if ollama_changed {
            let ollama_config = OllamaConfig {
                endpoint: new_config.ollama.endpoint.clone(),
                timeout: std::time::Duration::from_secs(new_config.ollama.timeout_seconds),
                max_retries: new_config.ollama.max_retries,
            };
            let ollama_client = OllamaClient::new(ollama_config);
            self.clients.insert(AIProvider::Ollama, Box::new(ollama_client));
        }

        // Update default provider if changed
        self.default_provider = match new_config.default_provider.as_str() {
            "ollama" => AIProvider::Ollama,
            "openai" => AIProvider::OpenAI,
            "anthropic" => AIProvider::Anthropic,
            provider => AIProvider::Custom(provider.to_string()),
        };

        self.config = new_config;
        
        // Clear cache as models might have changed
        self.clear_cache().await;
        
        Ok(())
    }

    /// Get available providers
    pub fn available_providers(&self) -> Vec<&AIProvider> {
        self.clients.keys().collect()
    }

    /// Check if a provider is available
    pub fn is_provider_available(&self, provider: &AIProvider) -> bool {
        self.clients.contains_key(provider)
    }

    /// Helper method for simple text generation (used by agents)
    pub async fn generate_response(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Result<String, AIError> {
        let mut parameters = super::ModelParameters::default();
        parameters.temperature = Some(temperature.unwrap_or(self.config.temperature as f32) as f64);
        parameters.max_tokens = max_tokens.map(|t| t as usize).or(Some(self.config.max_tokens));
        
        let request = ChatRequest {
            model: String::new(), // Will be filled with default
            messages: vec![
                super::ChatMessage::system(system_prompt),
                super::ChatMessage::user(user_prompt),
            ],
            parameters: Some(parameters),
            stream: false,
        };

        let response = self.chat_completion_default(request).await?;
        Ok(response.message.content)
    }
}

impl std::fmt::Debug for AIManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AIManager")
            .field("config", &self.config)
            .field("default_provider", &self.default_provider)
            .field("available_providers", &self.clients.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OllamaConfig as ConfigOllamaConfig;

    #[tokio::test]
    async fn test_ai_manager_creation() {
        let config = AIModelConfig {
            default_provider: "ollama".to_string(),
            default_model: "llama3.2".to_string(),
            ollama: ConfigOllamaConfig {
                endpoint: "http://localhost:11434".to_string(),
                timeout_seconds: 300,
                max_retries: 3,
                default_model: Some("llama3.2".to_string()),
            },
            openai: None,
            anthropic: None,
            context_window_size: 8192,
            temperature: 0.7,
            max_tokens: 1000,
        };

        let manager = AIManager::new(config).await;
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert_eq!(manager.default_provider(), &AIProvider::Ollama);
        assert_eq!(manager.default_model(), "llama3.2");
        assert!(manager.is_provider_available(&AIProvider::Ollama));
    }
}
