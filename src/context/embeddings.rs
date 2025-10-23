//! Embeddings support for semantic similarity search in codebase analysis
//!
//! This module provides functionality to generate and search code embeddings
//! for enhanced context retrieval and semantic understanding.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for embedding generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model_name: String,
    pub max_chunk_size: usize,
    pub chunk_overlap: usize,
    pub similarity_threshold: f32,
    pub max_results: usize,
    pub cache_embeddings: bool,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            max_chunk_size: 512,
            chunk_overlap: 50,
            similarity_threshold: 0.7,
            max_results: 20,
            cache_embeddings: true,
        }
    }
}

/// Chunk of code with metadata for embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChunk {
    pub id: String,
    pub file_path: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
    pub language: String,
    pub chunk_type: ChunkType,
    pub symbols: Vec<String>, // Function/class names etc. in this chunk
}

/// Type of code chunk for better embedding context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkType {
    Function,
    Class,
    Method,
    Variable,
    Import,
    Comment,
    Documentation,
    Generic,
}

/// Embedding vector with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEmbedding {
    pub chunk_id: String,
    pub vector: Vec<f32>,
    pub metadata: EmbeddingMetadata,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Metadata associated with an embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
    pub file_path: PathBuf,
    pub language: String,
    pub chunk_type: ChunkType,
    pub symbols: Vec<String>,
    pub content_hash: String,
}

/// Result from similarity search
#[derive(Debug, Clone)]
pub struct SimilarityResult {
    pub chunk: CodeChunk,
    pub embedding: CodeEmbedding,
    pub similarity_score: f32,
}

/// In-memory vector store for embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStore {
    embeddings: HashMap<String, CodeEmbedding>,
    chunks: HashMap<String, CodeChunk>,
    config: EmbeddingConfig,
    #[serde(skip)]
    embedding_cache: HashMap<String, Vec<f32>>,
}

impl VectorStore {
    pub fn new(config: EmbeddingConfig) -> Self {
        Self {
            embeddings: HashMap::new(),
            chunks: HashMap::new(),
            config,
            embedding_cache: HashMap::new(),
        }
    }

    /// Add an embedding to the store
    pub fn add_embedding(&mut self, chunk: CodeChunk, embedding: CodeEmbedding) {
        self.chunks.insert(chunk.id.clone(), chunk);
        self.embeddings.insert(embedding.chunk_id.clone(), embedding);
    }

    /// Remove embeddings for a specific file
    pub fn remove_file_embeddings(&mut self, file_path: &PathBuf) {
        let chunk_ids: Vec<String> = self.chunks
            .iter()
            .filter(|(_, chunk)| &chunk.file_path == file_path)
            .map(|(id, _)| id.clone())
            .collect();

        for chunk_id in chunk_ids {
            self.chunks.remove(&chunk_id);
            self.embeddings.remove(&chunk_id);
        }
    }

    /// Find similar chunks using cosine similarity
    pub fn find_similar(&self, query_embedding: &[f32], max_results: Option<usize>) -> Vec<SimilarityResult> {
        let max_results = max_results.unwrap_or(self.config.max_results);
        let mut results = Vec::new();

        for (chunk_id, embedding) in &self.embeddings {
            if let Some(chunk) = self.chunks.get(chunk_id) {
                let similarity = cosine_similarity(query_embedding, &embedding.vector);
                
                if similarity >= self.config.similarity_threshold {
                    results.push(SimilarityResult {
                        chunk: chunk.clone(),
                        embedding: embedding.clone(),
                        similarity_score: similarity,
                    });
                }
            }
        }

        // Sort by similarity score (descending)
        results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
        results.truncate(max_results);
        results
    }

    /// Get total number of embeddings
    pub fn len(&self) -> usize {
        self.embeddings.len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.embeddings.is_empty()
    }

    /// Get embeddings for a specific file
    pub fn get_file_embeddings(&self, file_path: &PathBuf) -> Vec<&CodeEmbedding> {
        self.embeddings
            .values()
            .filter(|embedding| &embedding.metadata.file_path == file_path)
            .collect()
    }
}

/// Embedding provider trait for different backends
#[async_trait::async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embedding for a single text chunk
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;

    /// Generate embeddings for multiple text chunks (batched)
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError>;

    /// Get the dimension of embeddings produced by this provider
    fn embedding_dimension(&self) -> usize;
}

/// Local embedding provider using Candle
#[cfg(feature = "local-embeddings")]
pub struct LocalEmbeddingProvider {
    model_name: String,
    dimension: usize,
}

#[cfg(feature = "local-embeddings")]
impl LocalEmbeddingProvider {
    pub fn new(model_name: String) -> Result<Self, EmbeddingError> {
        // For now, use a simple dimension mapping
        let dimension = match model_name.as_str() {
            "sentence-transformers/all-MiniLM-L6-v2" => 384,
            "sentence-transformers/all-mpnet-base-v2" => 768,
            _ => 384, // Default
        };

        Ok(Self {
            model_name,
            dimension,
        })
    }
}

#[cfg(feature = "local-embeddings")]
#[async_trait::async_trait]
impl EmbeddingProvider for LocalEmbeddingProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        // For now, return a simple hash-based mock embedding
        // In a real implementation, this would use Candle to run the transformer model
        let hash = md5::compute(text.as_bytes());
        let mut vector = vec![0.0f32; self.dimension];
        
        // Create a simple deterministic embedding from the hash
        for (i, &byte) in hash.0.iter().enumerate().take(16) {
            let base_idx = (i * self.dimension / 16).min(self.dimension - 24);
            for j in 0..24.min(self.dimension - base_idx) {
                vector[base_idx + j] = (byte as f32 - 128.0) / 128.0;
            }
        }
        
        // Normalize the vector
        let norm = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut vector {
                *x /= norm;
            }
        }

        Ok(vector)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let mut results = Vec::new();
        for text in texts {
            results.push(self.embed_text(text).await?);
        }
        Ok(results)
    }

    fn embedding_dimension(&self) -> usize {
        self.dimension
    }
}

/// OpenAI embedding provider
#[cfg(feature = "openai-embeddings")]
pub struct OpenAIEmbeddingProvider {
    client: openai_api_rs::v1::api::Client,
    model: String,
    dimension: usize,
}

#[cfg(feature = "openai-embeddings")]
impl OpenAIEmbeddingProvider {
    pub fn new(api_key: String, model: String) -> Result<Self, EmbeddingError> {
        let client = openai_api_rs::v1::api::Client::new(api_key);
        let dimension = match model.as_str() {
            "text-embedding-ada-002" => 1536,
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            _ => 1536, // Default
        };

        Ok(Self {
            client,
            model,
            dimension,
        })
    }
}

#[cfg(feature = "openai-embeddings")]
#[async_trait::async_trait]
impl EmbeddingProvider for OpenAIEmbeddingProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        // Implementation would make actual OpenAI API call
        // For now, return mock embedding
        self.embed_batch(&[text.to_string()]).await
            .map(|mut batch| batch.pop().unwrap_or_default())
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        // Mock implementation - in real usage would call OpenAI API
        let mut results = Vec::new();
        for text in texts {
            let hash = md5::compute(text.as_bytes());
            let mut vector = vec![0.0f32; self.dimension];
            
            // Simple hash-based mock
            for (i, &byte) in hash.0.iter().enumerate().take(16) {
                let base_idx = (i * self.dimension / 16).min(self.dimension - 24);
                for j in 0..24.min(self.dimension - base_idx) {
                    vector[base_idx + j] = (byte as f32 - 128.0) / 128.0;
                }
            }
            
            let norm = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for x in &mut vector {
                    *x /= norm;
                }
            }
            results.push(vector);
        }
        Ok(results)
    }

    fn embedding_dimension(&self) -> usize {
        self.dimension
    }
}

/// Error types for embedding operations
#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("Model loading failed: {0}")]
    ModelLoadError(String),

    #[error("Embedding generation failed: {0}")]
    EmbeddingGenerationError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Text chunker for splitting code into embeddable chunks
#[derive(Debug)]
pub struct CodeChunker {
    config: EmbeddingConfig,
}

impl CodeChunker {
    pub fn new(config: EmbeddingConfig) -> Self {
        Self { config }
    }

    /// Split file content into chunks suitable for embedding
    pub fn chunk_file(&self, file_path: &PathBuf, content: &str, language: &str) -> Vec<CodeChunk> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        let mut current_chunk = String::new();
        let mut start_line = 1;
        let mut current_line = 1;

        for line in lines.iter() {
            // If adding this line would exceed max chunk size, finalize current chunk
            if current_chunk.len() + line.len() > self.config.max_chunk_size && !current_chunk.is_empty() {
                let chunk_id = format!("{}:{}:{}", file_path.to_string_lossy(), start_line, current_line - 1);
                
                chunks.push(CodeChunk {
                    id: chunk_id,
                    file_path: file_path.clone(),
                    start_line,
                    end_line: current_line - 1,
                    content: current_chunk.trim().to_string(),
                    language: language.to_string(),
                    chunk_type: self.detect_chunk_type(&current_chunk),
                    symbols: self.extract_symbols(&current_chunk, language),
                });

                // Start new chunk with overlap
                let overlap_lines = self.config.chunk_overlap.min(lines.len());
                current_chunk = lines[current_line.saturating_sub(overlap_lines)..current_line]
                    .join("\n") + "\n";
                start_line = current_line.saturating_sub(overlap_lines) + 1;
            }

            current_chunk.push_str(line);
            current_chunk.push('\n');
            current_line += 1;
        }

        // Add final chunk if not empty
        if !current_chunk.trim().is_empty() {
            let chunk_id = format!("{}:{}:{}", file_path.to_string_lossy(), start_line, current_line - 1);
            
            chunks.push(CodeChunk {
                id: chunk_id,
                file_path: file_path.clone(),
                start_line,
                end_line: current_line - 1,
                content: current_chunk.trim().to_string(),
                language: language.to_string(),
                chunk_type: self.detect_chunk_type(&current_chunk),
                symbols: self.extract_symbols(&current_chunk, language),
            });
        }

        chunks
    }

    /// Detect the type of code chunk
    fn detect_chunk_type(&self, content: &str) -> ChunkType {
        let content_lower = content.to_lowercase();
        
        if content_lower.contains("fn ") || content_lower.contains("function ") {
            ChunkType::Function
        } else if content_lower.contains("class ") || content_lower.contains("struct ") {
            ChunkType::Class
        } else if content_lower.contains("impl ") || content_lower.contains("method ") {
            ChunkType::Method
        } else if content_lower.contains("import ") || content_lower.contains("use ") {
            ChunkType::Import
        } else if content.trim_start().starts_with("//") || content.trim_start().starts_with("/*") {
            ChunkType::Comment
        } else if content_lower.contains("///") || content_lower.contains("/**") {
            ChunkType::Documentation
        } else {
            ChunkType::Generic
        }
    }

    /// Extract symbol names from chunk content
    fn extract_symbols(&self, content: &str, language: &str) -> Vec<String> {
        let mut symbols = Vec::new();
        
        match language {
            "rust" => {
                // Simple regex-based extraction for Rust
                if let Ok(re) = regex::Regex::new(r"(?:fn|struct|enum|trait|impl)\s+([a-zA-Z_][a-zA-Z0-9_]*)") {
                    for cap in re.captures_iter(content) {
                        if let Some(name) = cap.get(1) {
                            symbols.push(name.as_str().to_string());
                        }
                    }
                }
            }
            _ => {
                // Generic extraction - look for function/class-like patterns
                if let Ok(re) = regex::Regex::new(r"(?:function|class|def)\s+([a-zA-Z_][a-zA-Z0-9_]*)") {
                    for cap in re.captures_iter(content) {
                        if let Some(name) = cap.get(1) {
                            symbols.push(name.as_str().to_string());
                        }
                    }
                }
            }
        }
        
        symbols
    }
}

/// Calculate cosine similarity between two vectors
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_code_chunker() {
        let config = EmbeddingConfig::default();
        let chunker = CodeChunker::new(config);
        
        let content = "fn main() {\n    println!(\"Hello, world!\");\n}\n\nfn other_function() {\n    println!(\"Other\");\n}";
        let file_path = PathBuf::from("test.rs");
        
        let chunks = chunker.chunk_file(&file_path, content, "rust");
        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].language, "rust");
        assert!(chunks[0].symbols.contains(&"main".to_string()) || chunks[0].symbols.contains(&"other_function".to_string()));
    }

    #[tokio::test]
    async fn test_vector_store() {
        let config = EmbeddingConfig::default();
        let mut store = VectorStore::new(config);
        
        let chunk = CodeChunk {
            id: "test1".to_string(),
            file_path: PathBuf::from("test.rs"),
            start_line: 1,
            end_line: 5,
            content: "fn main() {}".to_string(),
            language: "rust".to_string(),
            chunk_type: ChunkType::Function,
            symbols: vec!["main".to_string()],
        };
        
        let embedding = CodeEmbedding {
            chunk_id: "test1".to_string(),
            vector: vec![1.0, 0.0, 0.0],
            metadata: EmbeddingMetadata {
                file_path: PathBuf::from("test.rs"),
                language: "rust".to_string(),
                chunk_type: ChunkType::Function,
                symbols: vec!["main".to_string()],
                content_hash: "hash1".to_string(),
            },
            created_at: chrono::Utc::now(),
        };
        
        store.add_embedding(chunk, embedding);
        assert_eq!(store.len(), 1);
        
        let query = vec![1.0, 0.0, 0.0];
        let results = store.find_similar(&query, Some(5));
        assert_eq!(results.len(), 1);
        assert!(results[0].similarity_score > 0.9);
    }
}