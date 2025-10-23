//! Mixed-model routing and caching system
//!
//! This module provides intelligent routing between different AI models based on
//! task type, performance characteristics, and cost considerations. It includes
//! result caching and lightweight evaluation tracking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{AIManager, ChatRequest, ChatResponse, ModelParameters};

/// Model router that selects the best model for each task
#[derive(Debug)]
pub struct ModelRouter {
    models: HashMap<String, ModelConfig>,
    cache: Arc<ResponseCache>,
    evaluator: Arc<ModelEvaluator>,
    routing_rules: Vec<RoutingRule>,
    default_model: String,
}

/// Configuration for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub endpoint: Option<String>,
    pub capabilities: ModelCapabilities,
    pub performance_metrics: ModelMetrics,
    pub cost_per_token: f64,
    pub max_tokens: usize,
    pub enabled: bool,
}

/// Capabilities of a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub code_generation: bool,
    pub text_analysis: bool,
    pub reasoning: bool,
    pub multilingual: bool,
    pub context_length: usize,
    pub supported_languages: Vec<String>,
    pub strengths: Vec<TaskType>,
}

/// Performance metrics for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    pub avg_response_time_ms: f64,
    pub success_rate: f64,
    pub quality_score: f64,
    pub tokens_per_second: f64,
    pub total_requests: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Types of tasks for routing decisions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    CodeGeneration,
    CodeReview,
    Documentation,
    Testing,
    Debugging,
    Refactoring,
    Analysis,
    Planning,
    Chat,
}

/// Routing rule for model selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub name: String,
    pub conditions: Vec<RoutingCondition>,
    pub target_model: String,
    pub priority: u32,
    pub enabled: bool,
}

/// Condition for routing rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingCondition {
    TaskType(TaskType),
    TokenCountRange { min: usize, max: usize },
    RequiredCapability(String),
    MaxLatency(Duration),
    MaxCost(f64),
    Language(String),
}

/// Response cache for AI model results
#[derive(Debug)]
pub struct ResponseCache {
    cache: RwLock<HashMap<String, CacheEntry>>,
    max_entries: usize,
    ttl: Duration,
}

/// Cache entry with TTL and metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub response: ChatResponse,
    pub created_at: Instant,
    pub hit_count: u32,
    pub model_used: String,
    pub task_type: Option<TaskType>,
}

/// Model evaluator for tracking performance
#[derive(Debug)]
pub struct ModelEvaluator {
    evaluations: RwLock<HashMap<String, Vec<ModelEvaluation>>>,
    max_evaluations_per_model: usize,
}

/// Evaluation result for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEvaluation {
    pub model_name: String,
    pub task_type: TaskType,
    pub request_tokens: usize,
    pub response_tokens: usize,
    pub latency: Duration,
    pub quality_score: Option<f64>,
    pub cost: f64,
    pub success: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub error: Option<String>,
}

/// Request context for routing decisions
#[derive(Debug, Clone)]
pub struct RoutingContext {
    pub task_type: TaskType,
    pub estimated_tokens: usize,
    pub language: Option<String>,
    pub priority: RequestPriority,
    pub max_latency: Option<Duration>,
    pub max_cost: Option<f64>,
    pub user_preferences: HashMap<String, String>,
}

/// Priority levels for requests
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl ModelRouter {
    /// Create a new model router
    pub fn new(default_model: String) -> Self {
        Self {
            models: HashMap::new(),
            cache: Arc::new(ResponseCache::new(1000, Duration::from_secs(24 * 60 * 60))),
            evaluator: Arc::new(ModelEvaluator::new(100)),
            routing_rules: Vec::new(),
            default_model,
        }
    }
    
    /// Register a model with the router
    pub fn register_model(&mut self, config: ModelConfig) {
        self.models.insert(config.name.clone(), config);
    }
    
    /// Add a routing rule
    pub fn add_routing_rule(&mut self, rule: RoutingRule) {
        self.routing_rules.push(rule);
        // Sort by priority (highest first)
        self.routing_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
    
    /// Route a request to the best model
    pub async fn route_request(
        &self,
        request: &ChatRequest,
        context: &RoutingContext,
        ai_manager: &AIManager,
    ) -> Result<ChatResponse, ModelRoutingError> {
        // Check cache first
        let cache_key = self.generate_cache_key(request, context);
        if let Some(cached_response) = self.cache.get(&cache_key).await {
            return Ok(cached_response.response);
        }
        
        // Select the best model
        let selected_model = self.select_model(request, context)?;
        
        // Execute the request with performance tracking
        let start_time = Instant::now();
        let result = self.execute_with_model(request, &selected_model, ai_manager).await;
        let latency = start_time.elapsed();
        
        match result {
            Ok(response) => {
                // Cache the response
                let cache_entry = CacheEntry {
                    response: response.clone(),
                    created_at: Instant::now(),
                    hit_count: 0,
                    model_used: selected_model.clone(),
                    task_type: Some(context.task_type.clone()),
                };
                self.cache.put(cache_key, cache_entry).await;
                
                // Record evaluation
                let evaluation = ModelEvaluation {
                    model_name: selected_model,
                    task_type: context.task_type.clone(),
                    request_tokens: self.estimate_tokens(&request.messages),
                    response_tokens: self.estimate_tokens(&[response.message.clone()]),
                    latency,
                    quality_score: None, // Would be calculated by quality assessment
                    cost: 0.0, // Would be calculated based on model pricing
                    success: true,
                    timestamp: chrono::Utc::now(),
                    error: None,
                };
                self.evaluator.record_evaluation(evaluation).await;
                
                Ok(response)
            }
            Err(e) => {
                // Record failed evaluation
                let evaluation = ModelEvaluation {
                    model_name: selected_model,
                    task_type: context.task_type.clone(),
                    request_tokens: self.estimate_tokens(&request.messages),
                    response_tokens: 0,
                    latency,
                    quality_score: None,
                    cost: 0.0,
                    success: false,
                    timestamp: chrono::Utc::now(),
                    error: Some(e.to_string()),
                };
                self.evaluator.record_evaluation(evaluation).await;
                
                Err(ModelRoutingError::ModelExecutionFailed(e.to_string()))
            }
        }
    }
    
    /// Select the best model for a request
    fn select_model(&self, request: &ChatRequest, context: &RoutingContext) -> Result<String, ModelRoutingError> {
        // Apply routing rules in priority order
        for rule in &self.routing_rules {
            if !rule.enabled {
                continue;
            }
            
            if self.matches_conditions(&rule.conditions, request, context) {
                if self.models.get(&rule.target_model).map(|m| m.enabled).unwrap_or(false) {
                    return Ok(rule.target_model.clone());
                }
            }
        }
        
        // Fallback to best available model based on context
        if let Some(best_model) = self.find_best_model_for_task(context) {
            Ok(best_model)
        } else {
            // Use default model as final fallback
            Ok(self.default_model.clone())
        }
    }
    
    /// Check if routing conditions are met
    fn matches_conditions(&self, conditions: &[RoutingCondition], request: &ChatRequest, context: &RoutingContext) -> bool {
        for condition in conditions {
            match condition {
                RoutingCondition::TaskType(task_type) => {
                    if context.task_type != *task_type {
                        return false;
                    }
                }
                RoutingCondition::TokenCountRange { min, max } => {
                    if context.estimated_tokens < *min || context.estimated_tokens > *max {
                        return false;
                    }
                }
                RoutingCondition::RequiredCapability(capability) => {
                    // Check if any enabled model has this capability
                    if !self.models.values().any(|m| m.enabled && self.model_has_capability(m, capability)) {
                        return false;
                    }
                }
                RoutingCondition::MaxLatency(max_latency) => {
                    if let Some(required_latency) = context.max_latency {
                        if required_latency > *max_latency {
                            return false;
                        }
                    }
                }
                RoutingCondition::MaxCost(max_cost) => {
                    if let Some(required_cost) = context.max_cost {
                        if required_cost > *max_cost {
                            return false;
                        }
                    }
                }
                RoutingCondition::Language(language) => {
                    if let Some(req_lang) = &context.language {
                        if req_lang != language {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }
    
    /// Find the best model for a specific task type
    fn find_best_model_for_task(&self, context: &RoutingContext) -> Option<String> {
        let mut candidates: Vec<_> = self.models.values()
            .filter(|m| m.enabled && self.model_supports_task(m, &context.task_type))
            .collect();
        
        // Sort by performance score (quality * speed / cost)
        candidates.sort_by(|a, b| {
            let score_a = self.calculate_model_score(a, context);
            let score_b = self.calculate_model_score(b, context);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        candidates.first().map(|m| m.name.clone())
    }
    
    /// Calculate a composite score for model selection
    fn calculate_model_score(&self, model: &ModelConfig, context: &RoutingContext) -> f64 {
        let quality_weight = 0.4;
        let speed_weight = 0.3;
        let cost_weight = 0.3;
        
        let quality_score = model.performance_metrics.quality_score;
        let speed_score = if model.performance_metrics.avg_response_time_ms > 0.0 {
            1000.0 / model.performance_metrics.avg_response_time_ms
        } else {
            1.0
        };
        let cost_score = if model.cost_per_token > 0.0 {
            1.0 / model.cost_per_token
        } else {
            1.0
        };
        
        // Apply priority weighting
        let priority_multiplier = match context.priority {
            RequestPriority::Critical => 1.5,
            RequestPriority::High => 1.2,
            RequestPriority::Normal => 1.0,
            RequestPriority::Low => 0.8,
        };
        
        (quality_score * quality_weight + speed_score * speed_weight + cost_score * cost_weight) * priority_multiplier
    }
    
    /// Check if a model supports a specific task type
    fn model_supports_task(&self, model: &ModelConfig, task_type: &TaskType) -> bool {
        match task_type {
            TaskType::CodeGeneration => model.capabilities.code_generation,
            TaskType::CodeReview => model.capabilities.code_generation && model.capabilities.reasoning,
            TaskType::Documentation => model.capabilities.text_analysis,
            TaskType::Testing => model.capabilities.code_generation && model.capabilities.reasoning,
            TaskType::Debugging => model.capabilities.code_generation && model.capabilities.reasoning,
            TaskType::Refactoring => model.capabilities.code_generation && model.capabilities.reasoning,
            TaskType::Analysis => model.capabilities.text_analysis && model.capabilities.reasoning,
            TaskType::Planning => model.capabilities.reasoning,
            TaskType::Chat => true, // All models should support basic chat
        }
    }
    
    /// Check if a model has a specific capability
    fn model_has_capability(&self, model: &ModelConfig, capability: &str) -> bool {
        match capability {
            "code_generation" => model.capabilities.code_generation,
            "text_analysis" => model.capabilities.text_analysis,
            "reasoning" => model.capabilities.reasoning,
            "multilingual" => model.capabilities.multilingual,
            _ => false,
        }
    }
    
    /// Execute request with a specific model
    async fn execute_with_model(
        &self,
        request: &ChatRequest,
        model_name: &str,
        ai_manager: &AIManager,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Create a modified request with the selected model
        let mut routed_request = request.clone();
        routed_request.model = model_name.to_string();
        
        // Execute the request
        ai_manager.chat_completion_default(routed_request).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
    
    /// Generate cache key for a request
    fn generate_cache_key(&self, request: &ChatRequest, context: &RoutingContext) -> String {
        // Create a hash from serializable content
        let messages_str = serde_json::to_string(&request.messages).unwrap_or_default();
        let model = &request.model;
        let task_type = format!("{:?}", context.task_type);
        
        // Simple hash based on content
        let combined = format!("{}:{}:{}", messages_str, model, task_type);
        format!("{:x}", md5::compute(combined.as_bytes()))
    }
    
    /// Estimate token count for messages
    fn estimate_tokens(&self, messages: &[super::ChatMessage]) -> usize {
        // Simple estimation: ~4 characters per token
        messages.iter()
            .map(|m| m.content.len() / 4)
            .sum()
    }
    
    /// Get routing statistics
    pub async fn get_routing_stats(&self) -> RoutingStats {
        let evaluations = self.evaluator.get_all_evaluations().await;
        
        let total_requests = evaluations.len();
        let successful_requests = evaluations.iter().filter(|e| e.success).count();
        let avg_latency = if !evaluations.is_empty() {
            evaluations.iter()
                .map(|e| e.latency.as_millis() as f64)
                .sum::<f64>() / evaluations.len() as f64
        } else {
            0.0
        };
        
        let model_usage: HashMap<String, usize> = evaluations.iter()
            .fold(HashMap::new(), |mut acc, eval| {
                *acc.entry(eval.model_name.clone()).or_insert(0) += 1;
                acc
            });
        
        let task_distribution: HashMap<TaskType, usize> = evaluations.iter()
            .fold(HashMap::new(), |mut acc, eval| {
                *acc.entry(eval.task_type.clone()).or_insert(0) += 1;
                acc
            });
        
        let cache_stats = self.cache.get_stats().await;
        
        RoutingStats {
            total_requests,
            successful_requests,
            success_rate: if total_requests > 0 {
                successful_requests as f64 / total_requests as f64
            } else {
                0.0
            },
            avg_latency_ms: avg_latency,
            model_usage,
            task_distribution,
            cache_hit_rate: cache_stats.hit_rate,
            cache_size: cache_stats.entry_count,
        }
    }
}

/// Statistics about model routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingStats {
    pub total_requests: usize,
    pub successful_requests: usize,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub model_usage: HashMap<String, usize>,
    pub task_distribution: HashMap<TaskType, usize>,
    pub cache_hit_rate: f64,
    pub cache_size: usize,
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entry_count: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate: f64,
}

impl ResponseCache {
    pub fn new(max_entries: usize, ttl: Duration) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            max_entries,
            ttl,
        }
    }
    
    pub async fn get(&self, key: &str) -> Option<CacheEntry> {
        let cache = self.cache.read().await;
        if let Some(entry) = cache.get(key) {
            if entry.created_at.elapsed() < self.ttl {
                return Some(entry.clone());
            }
        }
        None
    }
    
    pub async fn put(&self, key: String, entry: CacheEntry) {
        let mut cache = self.cache.write().await;
        
        // Clean up expired entries
        cache.retain(|_, v| v.created_at.elapsed() < self.ttl);
        
        // Remove oldest entries if at capacity
        if cache.len() >= self.max_entries {
            if let Some((oldest_key, _)) = cache.iter()
                .min_by_key(|(_, v)| v.created_at)
                .map(|(k, v)| (k.clone(), v.clone()))
            {
                cache.remove(&oldest_key);
            }
        }
        
        cache.insert(key, entry);
    }
    
    pub async fn get_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let total_hits: u32 = cache.values().map(|e| e.hit_count).sum();
        
        CacheStats {
            entry_count: cache.len(),
            hit_count: total_hits as u64,
            miss_count: 0, // Would need additional tracking
            hit_rate: 0.0, // Would need hit/miss ratio tracking
        }
    }
}

impl ModelEvaluator {
    pub fn new(max_evaluations_per_model: usize) -> Self {
        Self {
            evaluations: RwLock::new(HashMap::new()),
            max_evaluations_per_model,
        }
    }
    
    pub async fn record_evaluation(&self, evaluation: ModelEvaluation) {
        let mut evaluations = self.evaluations.write().await;
        let model_evals = evaluations.entry(evaluation.model_name.clone())
            .or_insert_with(Vec::new);
        
        model_evals.push(evaluation);
        
        // Keep only the most recent evaluations
        if model_evals.len() > self.max_evaluations_per_model {
            model_evals.remove(0);
        }
    }
    
    pub async fn get_all_evaluations(&self) -> Vec<ModelEvaluation> {
        let evaluations = self.evaluations.read().await;
        evaluations.values().flatten().cloned().collect()
    }
}

impl TaskType {
    fn hash_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
}

/// Errors in model routing
#[derive(Debug, thiserror::Error)]
pub enum ModelRoutingError {
    #[error("No suitable model found for task")]
    NoSuitableModel,
    
    #[error("Model execution failed: {0}")]
    ModelExecutionFailed(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Cache error: {0}")]
    CacheError(String),
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self {
            code_generation: true,
            text_analysis: true,
            reasoning: true,
            multilingual: false,
            context_length: 4096,
            supported_languages: vec!["en".to_string()],
            strengths: vec![TaskType::Chat],
        }
    }
}

impl Default for ModelMetrics {
    fn default() -> Self {
        Self {
            avg_response_time_ms: 1000.0,
            success_rate: 0.95,
            quality_score: 0.8,
            tokens_per_second: 50.0,
            total_requests: 0,
            last_updated: chrono::Utc::now(),
        }
    }
}