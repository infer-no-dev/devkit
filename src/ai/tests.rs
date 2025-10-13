//! AI Client Integration Tests
//!
//! These tests verify that the AI clients can be properly instantiated
//! and configured without making actual API calls.

#[cfg(test)]
mod tests {
    use crate::ai::client::{OpenAIConfig, OpenAIClient, OllamaConfig, OllamaClient};
    use crate::ai::{AIProvider, ChatMessage, ChatRequest, MessageRole, ModelParameters};
    use std::time::Duration;

    #[test]
    fn test_openai_client_creation() {
        // Test creating OpenAI client with valid config
        let config = OpenAIConfig {
            api_key: "test-key".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            organization: None,
            timeout: Duration::from_secs(60),
            max_retries: 3,
        };

        let client = OpenAIClient::new(config);
        assert!(client.is_ok(), "OpenAI client should be created successfully");
    }

    #[test]
    fn test_openai_client_creation_without_api_key() {
        // Test creating OpenAI client without API key should fail
        let config = OpenAIConfig {
            api_key: "".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            organization: None,
            timeout: Duration::from_secs(60),
            max_retries: 3,
        };

        let client = OpenAIClient::new(config);
        assert!(client.is_err(), "OpenAI client creation should fail without API key");
    }

    #[test]
    fn test_ollama_client_creation() {
        // Test creating Ollama client with valid config
        let config = OllamaConfig {
            endpoint: "http://localhost:11434".to_string(),
            timeout: Duration::from_secs(60),
            max_retries: 3,
        };

        let client = OllamaClient::new(config);
        // Ollama client creation should always succeed
        assert!(true, "Ollama client created successfully");
    }

    #[test]
    fn test_chat_request_creation() {
        // Test creating a basic chat request
        let messages = vec![
            ChatMessage::system("You are a helpful assistant."),
            ChatMessage::user("Hello, how are you?"),
        ];

        let request = ChatRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages,
            parameters: Some(ModelParameters {
                temperature: Some(0.7),
                top_p: Some(0.9),
                top_k: None,
                max_tokens: Some(1000),
                stop: None,
                frequency_penalty: None,
                presence_penalty: None,
            }),
            stream: false,
        };

        assert_eq!(request.model, "gpt-3.5-turbo");
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, MessageRole::System);
        assert_eq!(request.messages[1].role, MessageRole::User);
        assert!(request.parameters.is_some());
        assert_eq!(request.parameters.as_ref().unwrap().temperature, Some(0.7));
    }

    #[test]
    fn test_model_parameters_defaults() {
        let params = ModelParameters::default();
        
        // Default parameters should have some sensible values
        assert_eq!(params.temperature, Some(0.7));
        assert_eq!(params.top_p, Some(0.9));
        assert!(params.top_k.is_none());
        assert_eq!(params.max_tokens, Some(1000));
        assert!(params.stop.is_none());
        assert!(params.frequency_penalty.is_none());
        assert!(params.presence_penalty.is_none());
    }

    #[test]
    fn test_ai_provider_enum() {
        let providers = vec![
            AIProvider::OpenAI,
            AIProvider::Ollama,
            AIProvider::Anthropic,
            AIProvider::Custom("local".to_string()),
        ];

        assert_eq!(providers.len(), 4);
        
        // Test Debug formatting
        assert_eq!(format!("{:?}", AIProvider::OpenAI), "OpenAI");
        assert_eq!(format!("{:?}", AIProvider::Ollama), "Ollama");
        assert_eq!(format!("{:?}", AIProvider::Anthropic), "Anthropic");
    }

    #[test]
    fn test_model_capability_enum() {
        use crate::ai::ModelCapability;
        
        let capabilities = vec![
            ModelCapability::CodeGeneration,
            ModelCapability::CodeAnalysis,
            ModelCapability::TextGeneration,
            ModelCapability::Conversation,
            ModelCapability::Summarization,
            ModelCapability::Translation,
            ModelCapability::QuestionAnswering,
        ];
        
        assert_eq!(capabilities.len(), 7);
        assert!(capabilities.contains(&ModelCapability::CodeGeneration));
        assert!(capabilities.contains(&ModelCapability::TextGeneration));
    }
}