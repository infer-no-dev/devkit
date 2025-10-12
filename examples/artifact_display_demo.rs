//! Artifact Display and Visualization Demo
//!
//! This example demonstrates the comprehensive artifact display system with:
//! - Interactive artifact browser with multiple view modes
//! - Syntax highlighting for various languages
//! - Diff views for comparing artifacts
//! - Quality metrics visualization
//! - Real-time artifact management

use devkit::artifacts::{
    ArtifactManager, ArtifactDisplay, ArtifactViewerState, ArtifactDisplayConfig,
    ViewMode, DisplayTheme, EnhancedArtifact, ArtifactMetadata, VersionInfo,
    QualityMetrics, UsageStats, StorageInfo, ArtifactRelationships, VersionType,
    ArtifactSearchCriteria,
};
use devkit::agents::{AgentInfo, TaskPriority};
use devkit::codegen::Artifact;
use devkit::ui::syntax::SyntaxHighlighter;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    collections::HashMap,
    error::Error,
    fs,
    io,
    path::Path,
    time::{Duration, SystemTime},
};
use tempfile::TempDir;
use tokio::time::sleep;
use tracing::{info, warn, error};

#[derive(Debug)]
struct DemoApp {
    /// Artifact manager
    artifact_manager: ArtifactManager,
    /// Display state
    viewer_state: ArtifactViewerState,
    /// Demo artifacts
    demo_artifacts: Vec<EnhancedArtifact>,
    /// Current demo mode
    demo_mode: DemoMode,
    /// Demo step counter
    demo_step: usize,
    /// Auto-advance timer
    auto_advance: bool,
    /// Last update time
    last_update: SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
enum DemoMode {
    /// Single artifact view with tabbed interface
    SingleView,
    /// Grid layout showing multiple artifacts
    GridView,
    /// List view with previews
    ListView,
    /// Side-by-side comparison
    Comparison,
    /// Diff view between two artifacts
    DiffView,
    /// Quality metrics showcase
    QualityView,
    /// Interactive exploration mode
    Interactive,
}

impl DemoApp {
    async fn new() -> Result<Self, Box<dyn Error>> {
        // Create temporary directory for artifact storage
        let temp_dir = TempDir::new()?;
        let storage_path = temp_dir.path().to_path_buf();

        // Initialize artifact manager
        let mut artifact_manager = ArtifactManager::new(storage_path).await?;

        // Create demo configuration
        let config = ArtifactDisplayConfig {
            syntax_highlighting: true,
            show_line_numbers: true,
            show_metadata: true,
            show_quality_metrics: true,
            tab_size: 4,
            max_preview_length: 5000,
            theme: DisplayTheme::Dark,
        };

        // Generate demo artifacts
        let demo_artifacts = Self::generate_demo_artifacts().await;

        // Store artifacts in manager
        for artifact in &demo_artifacts {
            artifact_manager.store_artifact(artifact.clone()).await?;
        }

        // Initialize viewer state
        let mut viewer_state = ArtifactViewerState::new(config);
        viewer_state.set_artifacts(demo_artifacts.clone());

        Ok(Self {
            artifact_manager,
            viewer_state,
            demo_artifacts,
            demo_mode: DemoMode::SingleView,
            demo_step: 0,
            auto_advance: true,
            last_update: SystemTime::now(),
        })
    }

    async fn generate_demo_artifacts() -> Vec<EnhancedArtifact> {
        let mut artifacts = Vec::new();
        let creator_agent = AgentInfo {
            id: "demo-agent".to_string(),
            name: "Demo Code Generator".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["code-generation".to_string(), "analysis".to_string()],
            status: devkit::agents::AgentStatus::Idle,
        };

        // Rust function artifact
        let rust_code = r#"use std::collections::HashMap;

/// A high-performance cache implementation with TTL support
pub struct Cache<K, V> {
    data: HashMap<K, (V, SystemTime)>,
    ttl: Duration,
    max_size: usize,
}

impl<K, V> Cache<K, V> 
where 
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    /// Creates a new cache with the specified TTL and maximum size
    pub fn new(ttl: Duration, max_size: usize) -> Self {
        Self {
            data: HashMap::new(),
            ttl,
            max_size,
        }
    }

    /// Inserts a key-value pair into the cache
    pub fn insert(&mut self, key: K, value: V) -> Result<(), CacheError> {
        self.evict_expired();
        
        if self.data.len() >= self.max_size {
            self.evict_lru()?;
        }
        
        self.data.insert(key, (value, SystemTime::now()));
        Ok(())
    }

    /// Retrieves a value from the cache
    pub fn get(&mut self, key: &K) -> Option<V> {
        self.evict_expired();
        
        if let Some((value, _)) = self.data.get(key) {
            Some(value.clone())
        } else {
            None
        }
    }

    fn evict_expired(&mut self) {
        let now = SystemTime::now();
        self.data.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp).unwrap_or_default() < self.ttl
        });
    }

    fn evict_lru(&mut self) -> Result<(), CacheError> {
        // Simplified LRU eviction - in practice, you'd track access times
        if let Some(key) = self.data.keys().next().cloned() {
            self.data.remove(&key);
            Ok(())
        } else {
            Err(CacheError::Empty)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Cache is empty")]
    Empty,
    #[error("Cache is full and cannot evict")]
    Full,
}"#;

        artifacts.push(EnhancedArtifact {
            artifact: Artifact {
                id: "cache-impl".to_string(),
                name: "High-Performance Cache".to_string(),
                content: rust_code.to_string(),
                artifact_type: "module".to_string(),
            },
            metadata: ArtifactMetadata {
                creator_agent: creator_agent.clone(),
                created_at: SystemTime::now(),
                modified_at: SystemTime::now(),
                language: Some("rust".to_string()),
                size_bytes: rust_code.len(),
                line_count: rust_code.lines().count(),
                tags: vec!["cache".to_string(), "performance".to_string(), "data-structure".to_string()],
                description: Some("A high-performance cache implementation with TTL support and LRU eviction".to_string()),
                file_path: Some("src/cache.rs".to_string()),
            },
            version: VersionInfo {
                version: "1.0.0".to_string(),
                version_type: VersionType::Major,
                parent_version: None,
                created_at: SystemTime::now(),
                changelog: Some("Initial implementation of TTL cache".to_string()),
            },
            quality_metrics: Some(QualityMetrics {
                maintainability: Some(85.5),
                complexity: Some(12),
                security_score: Some(92.0),
                performance_score: Some(88.7),
                technical_debt: Some(15.2),
                test_coverage: Some(78.9),
            }),
            usage_stats: UsageStats {
                access_count: 15,
                modification_count: 3,
                copy_count: 2,
                export_count: 1,
                last_accessed: SystemTime::now(),
                accessing_agents: vec!["demo-agent".to_string(), "test-agent".to_string()],
            },
            storage_info: StorageInfo {
                file_path: "artifacts/cache-impl.json".to_string(),
                compressed: true,
                backup_count: 2,
                storage_size_bytes: 1024,
            },
            relationships: ArtifactRelationships {
                dependencies: vec!["std-collections".to_string()],
                dependents: vec!["web-server".to_string()],
                related_artifacts: vec!["lru-cache".to_string(), "memory-pool".to_string()],
                imports: vec!["std::collections::HashMap".to_string(), "std::time::SystemTime".to_string()],
                exports: vec!["Cache".to_string(), "CacheError".to_string()],
            },
        });

        // Python ML model artifact
        let python_code = r#"""
Machine Learning Model for Code Quality Prediction

This module implements a neural network model that predicts code quality
metrics based on various code features and patterns.
"""

import tensorflow as tf
import numpy as np
from typing import Dict, List, Optional, Tuple
import logging
from dataclasses import dataclass
from pathlib import Path

logger = logging.getLogger(__name__)

@dataclass
class CodeFeatures:
    """Features extracted from code for quality prediction."""
    line_count: int
    complexity: float
    comment_ratio: float
    function_count: int
    class_count: int
    import_count: int
    duplicate_ratio: float
    test_coverage: float

class CodeQualityPredictor:
    """Neural network model for predicting code quality metrics."""
    
    def __init__(self, model_path: Optional[Path] = None):
        """Initialize the predictor model."""
        self.model = None
        self.is_trained = False
        self.feature_scaler = None
        
        if model_path and model_path.exists():
            self.load_model(model_path)
    
    def build_model(self, input_features: int = 8) -> tf.keras.Model:
        """Build the neural network architecture."""
        model = tf.keras.Sequential([
            tf.keras.layers.Dense(64, activation='relu', input_shape=(input_features,)),
            tf.keras.layers.Dropout(0.3),
            tf.keras.layers.Dense(32, activation='relu'),
            tf.keras.layers.Dropout(0.2),
            tf.keras.layers.Dense(16, activation='relu'),
            tf.keras.layers.Dense(4, activation='linear')  # 4 quality metrics
        ])
        
        model.compile(
            optimizer='adam',
            loss='mse',
            metrics=['mae', 'mse']
        )
        
        return model
    
    def extract_features(self, code: str) -> CodeFeatures:
        """Extract features from source code."""
        lines = code.split('\n')
        non_empty_lines = [line for line in lines if line.strip()]
        
        # Calculate comment ratio
        comment_lines = [line for line in lines if line.strip().startswith('#')]
        comment_ratio = len(comment_lines) / len(non_empty_lines) if non_empty_lines else 0
        
        # Count functions and classes
        function_count = len([line for line in lines if line.strip().startswith('def ')])
        class_count = len([line for line in lines if line.strip().startswith('class ')])
        import_count = len([line for line in lines if line.strip().startswith(('import ', 'from '))])
        
        # Simplified complexity calculation
        complexity = self._calculate_complexity(code)
        
        return CodeFeatures(
            line_count=len(non_empty_lines),
            complexity=complexity,
            comment_ratio=comment_ratio,
            function_count=function_count,
            class_count=class_count,
            import_count=import_count,
            duplicate_ratio=0.0,  # Simplified
            test_coverage=0.8     # Mock value
        )
    
    def _calculate_complexity(self, code: str) -> float:
        """Calculate cyclomatic complexity approximation."""
        complexity_keywords = ['if', 'elif', 'else', 'for', 'while', 'try', 'except', 'with']
        complexity = 1  # Base complexity
        
        for line in code.split('\n'):
            line = line.strip().lower()
            for keyword in complexity_keywords:
                if keyword in line:
                    complexity += 1
        
        return float(complexity)
    
    def predict_quality(self, features: CodeFeatures) -> Dict[str, float]:
        """Predict code quality metrics."""
        if not self.is_trained:
            logger.warning("Model not trained, returning mock predictions")
            return {
                'maintainability': 75.0,
                'security': 80.0,
                'performance': 70.0,
                'reliability': 85.0
            }
        
        # Convert features to numpy array
        feature_array = np.array([[
            features.line_count,
            features.complexity,
            features.comment_ratio,
            features.function_count,
            features.class_count,
            features.import_count,
            features.duplicate_ratio,
            features.test_coverage
        ]])
        
        # Scale features
        if self.feature_scaler:
            feature_array = self.feature_scaler.transform(feature_array)
        
        # Make prediction
        predictions = self.model.predict(feature_array)[0]
        
        return {
            'maintainability': float(predictions[0]),
            'security': float(predictions[1]),
            'performance': float(predictions[2]),
            'reliability': float(predictions[3])
        }
    
    def train(self, training_data: List[Tuple[CodeFeatures, Dict[str, float]]],
              epochs: int = 100, validation_split: float = 0.2) -> Dict[str, List[float]]:
        """Train the model on provided data."""
        if not training_data:
            raise ValueError("Training data cannot be empty")
        
        # Prepare training data
        X, y = self._prepare_training_data(training_data)
        
        # Build and compile model
        self.model = self.build_model()
        
        # Train model
        history = self.model.fit(
            X, y,
            epochs=epochs,
            validation_split=validation_split,
            verbose=1,
            callbacks=[
                tf.keras.callbacks.EarlyStopping(patience=10, restore_best_weights=True),
                tf.keras.callbacks.ReduceLROnPlateau(patience=5, factor=0.5)
            ]
        )
        
        self.is_trained = True
        logger.info(f"Model trained successfully over {epochs} epochs")
        
        return history.history
    
    def _prepare_training_data(self, training_data):
        """Prepare features and labels for training."""
        features = []
        labels = []
        
        for code_features, quality_metrics in training_data:
            feature_vector = [
                code_features.line_count,
                code_features.complexity,
                code_features.comment_ratio,
                code_features.function_count,
                code_features.class_count,
                code_features.import_count,
                code_features.duplicate_ratio,
                code_features.test_coverage
            ]
            
            label_vector = [
                quality_metrics['maintainability'],
                quality_metrics['security'],
                quality_metrics['performance'],
                quality_metrics['reliability']
            ]
            
            features.append(feature_vector)
            labels.append(label_vector)
        
        return np.array(features), np.array(labels)
    
    def save_model(self, path: Path) -> None:
        """Save the trained model."""
        if not self.is_trained:
            raise ValueError("Cannot save untrained model")
        
        self.model.save(path / "quality_predictor.h5")
        logger.info(f"Model saved to {path}")
    
    def load_model(self, path: Path) -> None:
        """Load a pre-trained model."""
        model_path = path / "quality_predictor.h5"
        if model_path.exists():
            self.model = tf.keras.models.load_model(str(model_path))
            self.is_trained = True
            logger.info(f"Model loaded from {path}")
        else:
            raise FileNotFoundError(f"Model file not found: {model_path}")
"#;

        artifacts.push(EnhancedArtifact {
            artifact: Artifact {
                id: "ml-predictor".to_string(),
                name: "Code Quality ML Predictor".to_string(),
                content: python_code.to_string(),
                artifact_type: "model".to_string(),
            },
            metadata: ArtifactMetadata {
                creator_agent: creator_agent.clone(),
                created_at: SystemTime::now(),
                modified_at: SystemTime::now(),
                language: Some("python".to_string()),
                size_bytes: python_code.len(),
                line_count: python_code.lines().count(),
                tags: vec!["machine-learning".to_string(), "quality".to_string(), "tensorflow".to_string()],
                description: Some("Neural network model for predicting code quality metrics".to_string()),
                file_path: Some("ml/quality_predictor.py".to_string()),
            },
            version: VersionInfo {
                version: "2.1.0".to_string(),
                version_type: VersionType::Minor,
                parent_version: Some("2.0.0".to_string()),
                created_at: SystemTime::now(),
                changelog: Some("Added early stopping and learning rate reduction".to_string()),
            },
            quality_metrics: Some(QualityMetrics {
                maintainability: Some(92.1),
                complexity: Some(18),
                security_score: Some(87.5),
                performance_score: Some(79.3),
                technical_debt: Some(22.8),
                test_coverage: Some(85.4),
            }),
            usage_stats: UsageStats {
                access_count: 42,
                modification_count: 8,
                copy_count: 5,
                export_count: 3,
                last_accessed: SystemTime::now(),
                accessing_agents: vec!["ml-agent".to_string(), "quality-agent".to_string()],
            },
            storage_info: StorageInfo {
                file_path: "artifacts/ml-predictor.json".to_string(),
                compressed: true,
                backup_count: 3,
                storage_size_bytes: 2048,
            },
            relationships: ArtifactRelationships {
                dependencies: vec!["tensorflow".to_string(), "numpy".to_string()],
                dependents: vec!["quality-dashboard".to_string()],
                related_artifacts: vec!["feature-extractor".to_string(), "model-trainer".to_string()],
                imports: vec!["tensorflow".to_string(), "numpy".to_string(), "typing".to_string()],
                exports: vec!["CodeQualityPredictor".to_string(), "CodeFeatures".to_string()],
            },
        });

        // JavaScript React component
        let js_code = r#"/**
 * Interactive Code Artifact Viewer Component
 * 
 * A React component that provides an interactive interface for viewing
 * and managing code artifacts with syntax highlighting and diff capabilities.
 */

import React, { useState, useEffect, useCallback, useMemo } from 'react';
import PropTypes from 'prop-types';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { tomorrow } from 'react-syntax-highlighter/dist/esm/styles/prism';
import DiffViewer from 'react-diff-viewer';
import { 
  Box, 
  Tab, 
  Tabs, 
  TabList, 
  TabPanel, 
  TabPanels,
  VStack,
  HStack,
  Text,
  Badge,
  Progress,
  Stat,
  StatLabel,
  StatNumber,
  StatHelpText,
  useColorModeValue,
  IconButton,
  Tooltip
} from '@chakra-ui/react';
import { 
  ViewIcon, 
  EditIcon, 
  CopyIcon, 
  DownloadIcon,
  ChevronLeftIcon,
  ChevronRightIcon
} from '@chakra-ui/icons';

const ArtifactViewer = ({ 
  artifacts = [], 
  selectedArtifactId, 
  onArtifactSelect,
  onArtifactEdit,
  onArtifactCopy,
  onArtifactExport,
  viewMode = 'single',
  showQualityMetrics = true,
  enableSyntaxHighlighting = true 
}) => {
  const [selectedArtifact, setSelectedArtifact] = useState(null);
  const [tabIndex, setTabIndex] = useState(0);
  const [comparisonArtifact, setComparisonArtifact] = useState(null);
  
  const bgColor = useColorModeValue('white', 'gray.800');
  const borderColor = useColorModeValue('gray.200', 'gray.600');

  // Find selected artifact
  useEffect(() => {
    if (selectedArtifactId) {
      const artifact = artifacts.find(a => a.id === selectedArtifactId);
      setSelectedArtifact(artifact || null);
    }
  }, [selectedArtifactId, artifacts]);

  // Handle artifact navigation
  const navigateArtifact = useCallback((direction) => {
    if (!selectedArtifact || artifacts.length === 0) return;
    
    const currentIndex = artifacts.findIndex(a => a.id === selectedArtifact.id);
    const newIndex = direction === 'next' 
      ? (currentIndex + 1) % artifacts.length 
      : (currentIndex - 1 + artifacts.length) % artifacts.length;
    
    const newArtifact = artifacts[newIndex];
    onArtifactSelect(newArtifact.id);
  }, [selectedArtifact, artifacts, onArtifactSelect]);

  // Memoized quality metrics component
  const qualityMetricsComponent = useMemo(() => {
    if (!selectedArtifact?.qualityMetrics || !showQualityMetrics) return null;

    const metrics = selectedArtifact.qualityMetrics;
    
    return (
      <VStack spacing={4} align="stretch">
        <Text fontSize="lg" fontWeight="bold">Quality Metrics</Text>
        
        <VStack spacing={3}>
          <Box w="100%">
            <HStack justify="space-between" mb={1}>
              <Text fontSize="sm">Maintainability</Text>
              <Text fontSize="sm" fontWeight="bold">
                {metrics.maintainability?.toFixed(1)}%
              </Text>
            </HStack>
            <Progress 
              value={metrics.maintainability || 0} 
              colorScheme={metrics.maintainability >= 80 ? 'green' : metrics.maintainability >= 60 ? 'yellow' : 'red'}
              size="sm"
              borderRadius="md"
            />
          </Box>

          <Box w="100%">
            <HStack justify="space-between" mb={1}>
              <Text fontSize="sm">Security Score</Text>
              <Text fontSize="sm" fontWeight="bold">
                {metrics.securityScore?.toFixed(1)}%
              </Text>
            </HStack>
            <Progress 
              value={metrics.securityScore || 0} 
              colorScheme="blue"
              size="sm"
              borderRadius="md"
            />
          </Box>

          <Box w="100%">
            <HStack justify="space-between" mb={1}>
              <Text fontSize="sm">Performance</Text>
              <Text fontSize="sm" fontWeight="bold">
                {metrics.performanceScore?.toFixed(1)}%
              </Text>
            </HStack>
            <Progress 
              value={metrics.performanceScore || 0} 
              colorScheme="cyan"
              size="sm"
              borderRadius="md"
            />
          </Box>

          <HStack spacing={4} w="100%">
            <Stat size="sm">
              <StatLabel>Complexity</StatLabel>
              <StatNumber fontSize="lg">{metrics.complexity || 0}</StatNumber>
            </Stat>
            
            <Stat size="sm">
              <StatLabel>Tech Debt</StatLabel>
              <StatNumber fontSize="lg" color={metrics.technicalDebt > 30 ? "red.500" : "green.500"}>
                {metrics.technicalDebt?.toFixed(1)}%
              </StatNumber>
            </Stat>
          </HStack>
        </VStack>
      </VStack>
    );
  }, [selectedArtifact, showQualityMetrics]);

  // Render artifact content with syntax highlighting
  const renderContent = useCallback((artifact) => {
    if (!artifact) return <Text>No artifact selected</Text>;

    const content = artifact.content;
    const language = artifact.metadata?.language || 'text';

    if (enableSyntaxHighlighting && language !== 'text') {
      return (
        <SyntaxHighlighter
          language={language}
          style={tomorrow}
          showLineNumbers
          wrapLongLines
          customStyle={{
            margin: 0,
            borderRadius: '0.375rem',
            fontSize: '0.875rem'
          }}
        >
          {content}
        </SyntaxHighlighter>
      );
    }

    return (
      <Box
        as="pre"
        p={4}
        bg={useColorModeValue('gray.50', 'gray.900')}
        borderRadius="md"
        overflow="auto"
        fontSize="sm"
        fontFamily="mono"
        whiteSpace="pre-wrap"
      >
        {content}
      </Box>
    );
  }, [enableSyntaxHighlighting]);

  // Render metadata tab
  const renderMetadata = useCallback((artifact) => {
    if (!artifact?.metadata) return <Text>No metadata available</Text>;

    const metadata = artifact.metadata;
    
    return (
      <VStack spacing={4} align="stretch">
        <Box>
          <Text fontWeight="bold" mb={2}>Basic Information</Text>
          <VStack spacing={2} align="stretch">
            <HStack justify="space-between">
              <Text>Creator:</Text>
              <Text fontWeight="medium">{metadata.creatorAgent?.name}</Text>
            </HStack>
            <HStack justify="space-between">
              <Text>Language:</Text>
              <Badge colorScheme="blue">{metadata.language}</Badge>
            </HStack>
            <HStack justify="space-between">
              <Text>Size:</Text>
              <Text>{(metadata.sizeBytes / 1024).toFixed(1)} KB</Text>
            </HStack>
            <HStack justify="space-between">
              <Text>Lines:</Text>
              <Text>{metadata.lineCount}</Text>
            </HStack>
          </VStack>
        </Box>

        <Box>
          <Text fontWeight="bold" mb={2}>Tags</Text>
          <HStack wrap="wrap" spacing={2}>
            {metadata.tags?.map((tag, index) => (
              <Badge key={index} variant="subtle" colorScheme="purple">
                {tag}
              </Badge>
            ))}
          </HStack>
        </Box>

        {metadata.description && (
          <Box>
            <Text fontWeight="bold" mb={2}>Description</Text>
            <Text color="gray.600">{metadata.description}</Text>
          </Box>
        )}
      </VStack>
    );
  }, []);

  if (!selectedArtifact) {
    return (
      <Box
        p={8}
        textAlign="center"
        bg={bgColor}
        border="1px"
        borderColor={borderColor}
        borderRadius="md"
      >
        <Text color="gray.500">Select an artifact to view its details</Text>
      </Box>
    );
  }

  return (
    <Box
      bg={bgColor}
      border="1px"
      borderColor={borderColor}
      borderRadius="md"
      overflow="hidden"
    >
      {/* Header with navigation and actions */}
      <HStack
        p={4}
        borderBottom="1px"
        borderBottomColor={borderColor}
        justify="space-between"
        align="center"
      >
        <HStack spacing={2}>
          <IconButton
            icon={<ChevronLeftIcon />}
            size="sm"
            variant="ghost"
            onClick={() => navigateArtifact('prev')}
            isDisabled={artifacts.length <= 1}
          />
          
          <VStack spacing={0} align="start">
            <Text fontWeight="bold" fontSize="lg">
              {selectedArtifact.name}
            </Text>
            <Text fontSize="xs" color="gray.500">
              {selectedArtifact.metadata?.language} • {selectedArtifact.version?.version}
            </Text>
          </VStack>
          
          <IconButton
            icon={<ChevronRightIcon />}
            size="sm"
            variant="ghost"
            onClick={() => navigateArtifact('next')}
            isDisabled={artifacts.length <= 1}
          />
        </HStack>

        <HStack spacing={2}>
          <Tooltip label="View artifact">
            <IconButton
              icon={<ViewIcon />}
              size="sm"
              variant="ghost"
              onClick={() => onArtifactSelect(selectedArtifact.id)}
            />
          </Tooltip>
          
          <Tooltip label="Edit artifact">
            <IconButton
              icon={<EditIcon />}
              size="sm"
              variant="ghost"
              onClick={() => onArtifactEdit(selectedArtifact)}
            />
          </Tooltip>
          
          <Tooltip label="Copy artifact">
            <IconButton
              icon={<CopyIcon />}
              size="sm"
              variant="ghost"
              onClick={() => onArtifactCopy(selectedArtifact)}
            />
          </Tooltip>
          
          <Tooltip label="Export artifact">
            <IconButton
              icon={<DownloadIcon />}
              size="sm"
              variant="ghost"
              onClick={() => onArtifactExport(selectedArtifact)}
            />
          </Tooltip>
        </HStack>
      </HStack>

      {/* Tabbed content */}
      <Tabs index={tabIndex} onChange={setTabIndex}>
        <TabList px={4}>
          <Tab>Content</Tab>
          <Tab>Metadata</Tab>
          {showQualityMetrics && <Tab>Quality</Tab>}
          <Tab>History</Tab>
        </TabList>

        <TabPanels>
          <TabPanel>
            {renderContent(selectedArtifact)}
          </TabPanel>
          
          <TabPanel>
            {renderMetadata(selectedArtifact)}
          </TabPanel>
          
          {showQualityMetrics && (
            <TabPanel>
              {qualityMetricsComponent}
            </TabPanel>
          )}
          
          <TabPanel>
            <VStack spacing={3} align="stretch">
              <Text fontWeight="bold">Usage Statistics</Text>
              
              <HStack spacing={6}>
                <Stat size="sm">
                  <StatLabel>Accessed</StatLabel>
                  <StatNumber>{selectedArtifact.usageStats?.accessCount || 0}</StatNumber>
                  <StatHelpText>times</StatHelpText>
                </Stat>
                
                <Stat size="sm">
                  <StatLabel>Modified</StatLabel>
                  <StatNumber>{selectedArtifact.usageStats?.modificationCount || 0}</StatNumber>
                  <StatHelpText>times</StatHelpText>
                </Stat>
                
                <Stat size="sm">
                  <StatLabel>Copied</StatLabel>
                  <StatNumber>{selectedArtifact.usageStats?.copyCount || 0}</StatNumber>
                  <StatHelpText>times</StatHelpText>
                </Stat>
              </HStack>

              <Box>
                <Text fontWeight="medium" mb={2}>Version History</Text>
                <VStack spacing={2} align="stretch">
                  <HStack justify="space-between">
                    <Text fontSize="sm">Current Version:</Text>
                    <Badge>{selectedArtifact.version?.version}</Badge>
                  </HStack>
                  {selectedArtifact.version?.parentVersion && (
                    <HStack justify="space-between">
                      <Text fontSize="sm">Parent Version:</Text>
                      <Text fontSize="sm">{selectedArtifact.version.parentVersion}</Text>
                    </HStack>
                  )}
                  {selectedArtifact.version?.changelog && (
                    <Box>
                      <Text fontSize="sm" fontWeight="medium">Changelog:</Text>
                      <Text fontSize="xs" color="gray.600">
                        {selectedArtifact.version.changelog}
                      </Text>
                    </Box>
                  )}
                </VStack>
              </Box>
            </VStack>
          </TabPanel>
        </TabPanels>
      </Tabs>
    </Box>
  );
};

ArtifactViewer.propTypes = {
  artifacts: PropTypes.arrayOf(PropTypes.object),
  selectedArtifactId: PropTypes.string,
  onArtifactSelect: PropTypes.func.isRequired,
  onArtifactEdit: PropTypes.func,
  onArtifactCopy: PropTypes.func,
  onArtifactExport: PropTypes.func,
  viewMode: PropTypes.oneOf(['single', 'grid', 'list', 'comparison', 'diff']),
  showQualityMetrics: PropTypes.bool,
  enableSyntaxHighlighting: PropTypes.bool,
};

export default ArtifactViewer;
"#;

        artifacts.push(EnhancedArtifact {
            artifact: Artifact {
                id: "artifact-viewer".to_string(),
                name: "React Artifact Viewer".to_string(),
                content: js_code.to_string(),
                artifact_type: "component".to_string(),
            },
            metadata: ArtifactMetadata {
                creator_agent: creator_agent.clone(),
                created_at: SystemTime::now(),
                modified_at: SystemTime::now(),
                language: Some("javascript".to_string()),
                size_bytes: js_code.len(),
                line_count: js_code.lines().count(),
                tags: vec!["react".to_string(), "ui".to_string(), "component".to_string()],
                description: Some("Interactive React component for viewing code artifacts".to_string()),
                file_path: Some("components/ArtifactViewer.jsx".to_string()),
            },
            version: VersionInfo {
                version: "1.3.2".to_string(),
                version_type: VersionType::Patch,
                parent_version: Some("1.3.1".to_string()),
                created_at: SystemTime::now(),
                changelog: Some("Fixed syntax highlighting performance issue".to_string()),
            },
            quality_metrics: Some(QualityMetrics {
                maintainability: Some(89.3),
                complexity: Some(22),
                security_score: Some(94.2),
                performance_score: Some(82.1),
                technical_debt: Some(18.5),
                test_coverage: Some(91.7),
            }),
            usage_stats: UsageStats {
                access_count: 28,
                modification_count: 12,
                copy_count: 7,
                export_count: 4,
                last_accessed: SystemTime::now(),
                accessing_agents: vec!["ui-agent".to_string(), "frontend-agent".to_string()],
            },
            storage_info: StorageInfo {
                file_path: "artifacts/artifact-viewer.json".to_string(),
                compressed: false,
                backup_count: 2,
                storage_size_bytes: 1536,
            },
            relationships: ArtifactRelationships {
                dependencies: vec!["react".to_string(), "chakra-ui".to_string()],
                dependents: vec!["artifact-browser".to_string()],
                related_artifacts: vec!["syntax-highlighter".to_string(), "diff-viewer".to_string()],
                imports: vec!["react".to_string(), "react-syntax-highlighter".to_string()],
                exports: vec!["ArtifactViewer".to_string()],
            },
        });

        artifacts
    }

    fn advance_demo(&mut self) {
        self.demo_step += 1;
        
        match self.demo_step % 7 {
            0 => {
                self.demo_mode = DemoMode::SingleView;
                self.viewer_state.set_view_mode(ViewMode::Single);
                self.viewer_state.select_artifact(0);
            }
            1 => {
                self.demo_mode = DemoMode::GridView;
                self.viewer_state.set_view_mode(ViewMode::Grid);
            }
            2 => {
                self.demo_mode = DemoMode::ListView;
                self.viewer_state.set_view_mode(ViewMode::List);
            }
            3 => {
                self.demo_mode = DemoMode::QualityView;
                self.viewer_state.set_view_mode(ViewMode::Single);
                self.viewer_state.select_artifact(1); // Select Python artifact with good metrics
                self.viewer_state.active_tab = 2; // Quality tab
            }
            4 => {
                if self.demo_artifacts.len() >= 2 {
                    self.demo_mode = DemoMode::Comparison;
                    self.viewer_state.start_comparison(
                        self.demo_artifacts[0].clone(),
                        self.demo_artifacts[1].clone(),
                    );
                    self.viewer_state.set_view_mode(ViewMode::SideBySide);
                }
            }
            5 => {
                if self.demo_artifacts.len() >= 2 {
                    self.demo_mode = DemoMode::DiffView;
                    self.viewer_state.set_view_mode(ViewMode::Diff);
                }
            }
            _ => {
                self.demo_mode = DemoMode::Interactive;
                self.viewer_state.set_view_mode(ViewMode::Single);
                self.viewer_state.select_artifact(2); // Select React component
                self.auto_advance = false;
            }
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return false, // Quit
            KeyCode::Char(' ') => {
                self.auto_advance = !self.auto_advance;
                if self.auto_advance {
                    self.last_update = SystemTime::now();
                }
            }
            KeyCode::Right | KeyCode::Char('n') => {
                self.advance_demo();
                self.auto_advance = false;
            }
            KeyCode::Left | KeyCode::Char('p') => {
                if self.demo_step > 0 {
                    self.demo_step -= 1;
                } else {
                    self.demo_step = 6;
                }
                self.advance_demo();
                self.auto_advance = false;
            }
            KeyCode::Tab => {
                if self.demo_mode == DemoMode::Interactive {
                    self.viewer_state.next_tab();
                }
            }
            KeyCode::BackTab => {
                if self.demo_mode == DemoMode::Interactive {
                    self.viewer_state.previous_tab();
                }
            }
            KeyCode::Up => {
                if self.demo_mode == DemoMode::Interactive {
                    self.viewer_state.scroll_up(3);
                }
            }
            KeyCode::Down => {
                if self.demo_mode == DemoMode::Interactive {
                    self.viewer_state.scroll_down(3);
                }
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let index = c.to_digit(10).unwrap_or(0) as usize;
                if index < self.demo_artifacts.len() {
                    self.viewer_state.select_artifact(index);
                    self.auto_advance = false;
                }
            }
            _ => {}
        }
        true
    }

    fn should_advance(&self) -> bool {
        if !self.auto_advance || self.demo_mode == DemoMode::Interactive {
            return false;
        }

        self.last_update.elapsed().unwrap_or(Duration::ZERO) > Duration::from_secs(5)
    }

    fn update(&mut self) {
        if self.should_advance() {
            self.advance_demo();
            self.last_update = SystemTime::now();
        }
    }
}

async fn run_demo<B: Backend>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
    let mut app = DemoApp::new().await?;
    
    loop {
        app.update();
        
        terminal.draw(|f| {
            let size = f.size();
            
            // Create layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Min(0),    // Content
                    Constraint::Length(3), // Footer
                ].as_ref())
                .split(size);

            // Render header
            let mode_text = format!("Demo Mode: {:?} (Step {}/7)", app.demo_mode, (app.demo_step % 7) + 1);
            let header = Paragraph::new(mode_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("DevKit Artifact Display Demo")
                        .border_type(BorderType::Rounded),
                )
                .style(Style::default().fg(Color::Cyan));
            f.render_widget(header, chunks[0]);

            // Render artifact display
            let mut artifact_display = ArtifactDisplay::new(&mut app.viewer_state);
            artifact_display.render(f, chunks[1]);

            // Render footer with controls
            let auto_indicator = if app.auto_advance { "AUTO" } else { "MANUAL" };
            let controls = if app.demo_mode == DemoMode::Interactive {
                format!("Controls: [Q]uit • [Space] Toggle {} • [←/→] Navigate • [Tab] Switch tabs • [↑/↓] Scroll • [0-9] Select artifact", auto_indicator)
            } else {
                format!("Controls: [Q]uit • [Space] Toggle {} • [←/→] Navigate demos", auto_indicator)
            };
            
            let footer = Paragraph::new(controls)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                )
                .style(Style::default().fg(Color::Gray))
                .wrap(Wrap { trim: true });
            f.render_widget(footer, chunks[2]);
        })?;

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if !app.handle_key_event(key) {
                    break;
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting DevKit Artifact Display Demo");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run demo
    let result = run_demo(&mut terminal).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Print final message
    match result {
        Ok(()) => {
            println!("DevKit Artifact Display Demo completed successfully!");
            println!("Thank you for exploring the artifact management system.");
        }
        Err(e) => {
            eprintln!("Demo error: {}", e);
        }
    }

    Ok(())
}
"#;

        artifacts
    }

    fn advance_demo(&mut self) {
        // Implementation moved to main demo application
        self.demo_step += 1;
    }
}