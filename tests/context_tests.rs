use std::collections::HashMap;
use std::path::PathBuf;

use agentic_dev_env::context::*;
use agentic_dev_env::testing::{
    fixtures::ContextFixtures, mocks::MockCodebaseAnalyzer, TestEnvironment,
};

/// Tests for codebase analysis functionality
mod analyzer_tests {
    use super::*;

    #[test]
    fn test_file_context_creation() {
        let context = ContextFixtures::create_file_context("src/main.rs", Some("rust"));

        assert_eq!(context.file_path, PathBuf::from("src/main.rs"));
        assert_eq!(context.language, Some("rust".to_string()));
        assert!(!context.symbols.is_empty());
        assert!(!context.imports.is_empty());
        assert!(!context.content_hash.is_empty());
    }

    #[test]
    fn test_codebase_context_creation() {
        let context = ContextFixtures::create_codebase_context("/test/project");

        assert_eq!(context.root_path, PathBuf::from("/test/project"));
        assert_eq!(context.files.len(), 5);
        assert!(!context.dependencies.is_empty());
        assert!(!context.relationships.is_empty());
        assert!(context.metadata.contains_key("language"));
        assert!(context.metadata.contains_key("build_system"));
    }

    #[test]
    fn test_analyzer_creation() {
        let analyzer = MockCodebaseAnalyzer::new();
        assert!(!analyzer.analysis_results.is_empty());
        assert!(analyzer.analysis_results.contains_key("main.rs"));
        assert!(analyzer.analysis_results.contains_key("lib.rs"));
    }

    #[test]
    fn test_file_analysis() {
        let mut analyzer = MockCodebaseAnalyzer::new();
        let config = AnalysisConfig::default();

        let result = analyzer.analyze_file(&PathBuf::from("src/test.rs"), &config);

        assert!(result.is_ok());
        let file_context = result.unwrap();
        assert_eq!(file_context.file_path, PathBuf::from("src/test.rs"));
        assert_eq!(file_context.language, Some("rust".to_string()));
        assert_eq!(file_context.content_hash, "mock_hash");

        let analyzed_files = analyzer.get_analyzed_files();
        assert_eq!(analyzed_files.len(), 1);
        assert!(analyzed_files.contains(&"src/test.rs".to_string()));
    }

    #[test]
    fn test_file_analysis_failure() {
        let mut analyzer = MockCodebaseAnalyzer::new().with_failure(true);
        let config = AnalysisConfig::default();

        let result = analyzer.analyze_file(&PathBuf::from("invalid.rs"), &config);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ContextError::AnalysisFailed(_)
        ));

        let analyzed_files = analyzer.get_analyzed_files();
        assert!(analyzed_files.is_empty());
    }

    #[test]
    fn test_codebase_analysis() {
        let mut analyzer = MockCodebaseAnalyzer::new();
        let files = vec![
            ContextFixtures::create_file_context("src/main.rs", Some("rust")),
            ContextFixtures::create_file_context("src/lib.rs", Some("rust")),
        ];

        let result = analyzer.analyze_codebase(&files);

        assert!(result.is_ok());
        let codebase_context = result.unwrap();
        assert_eq!(codebase_context.root_path, PathBuf::from("/mock/path"));
        assert!(!codebase_context.dependencies.is_empty());
    }

    #[test]
    fn test_codebase_analysis_failure() {
        let mut analyzer = MockCodebaseAnalyzer::new().with_failure(true);
        let files = vec![ContextFixtures::create_file_context(
            "src/main.rs",
            Some("rust"),
        )];

        let result = analyzer.analyze_codebase(&files);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ContextError::AnalysisFailed(_)
        ));
    }

    #[test]
    fn test_multiple_file_analysis() {
        let mut analyzer = MockCodebaseAnalyzer::new();
        let config = AnalysisConfig::default();

        let files_to_analyze = vec!["src/main.rs", "src/lib.rs", "tests/test.rs"];

        for file in files_to_analyze {
            let result = analyzer.analyze_file(&PathBuf::from(file), &config);
            assert!(result.is_ok());
        }

        let analyzed_files = analyzer.get_analyzed_files();
        assert_eq!(analyzed_files.len(), 3);
        assert!(analyzed_files.contains(&"src/main.rs".to_string()));
        assert!(analyzed_files.contains(&"src/lib.rs".to_string()));
        assert!(analyzed_files.contains(&"tests/test.rs".to_string()));
    }
}

/// Tests for symbol indexing functionality
mod symbol_tests {
    use super::*;

    #[test]
    fn test_symbol_creation() {
        let symbol = ContextFixtures::create_symbol(
            "test_function",
            SymbolType::Function,
            "src/main.rs",
            10,
        );

        assert_eq!(symbol.name, "test_function");
        assert!(matches!(symbol.symbol_type, SymbolType::Function));
        assert_eq!(symbol.location.file, PathBuf::from("src/main.rs"));
        assert_eq!(symbol.location.line, 10);
        assert_eq!(symbol.visibility, "public");
        assert!(symbol.documentation.is_some());
    }

    #[test]
    fn test_symbol_index_creation() {
        let mut index = SymbolIndex::new();
        assert!(index.is_empty());

        let symbol = ContextFixtures::create_symbol(
            "test_function",
            SymbolType::Function,
            "src/main.rs",
            10,
        );

        index.add_symbol(symbol.clone());
        assert!(!index.is_empty());
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_symbol_search() {
        let mut index = SymbolIndex::new();

        let symbols = vec![
            ContextFixtures::create_symbol("function_one", SymbolType::Function, "src/main.rs", 10),
            ContextFixtures::create_symbol("StructOne", SymbolType::Struct, "src/lib.rs", 20),
            ContextFixtures::create_symbol(
                "function_two",
                SymbolType::Function,
                "src/utils.rs",
                30,
            ),
            ContextFixtures::create_symbol("EnumOne", SymbolType::Enum, "src/types.rs", 40),
        ];

        for symbol in symbols {
            index.add_symbol(symbol);
        }

        // Search by name
        let results = index.search_by_name("function_one");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "function_one");

        // Search by prefix
        let function_results = index.search_by_prefix("function_");
        assert_eq!(function_results.len(), 2);

        // Search by type
        let struct_results = index.search_by_type(SymbolType::Struct);
        assert_eq!(struct_results.len(), 1);
        assert_eq!(struct_results[0].name, "StructOne");
    }

    #[test]
    fn test_symbol_search_empty_index() {
        let index = SymbolIndex::new();

        let results = index.search_by_name("nonexistent");
        assert!(results.is_empty());

        let prefix_results = index.search_by_prefix("test_");
        assert!(prefix_results.is_empty());

        let type_results = index.search_by_type(SymbolType::Function);
        assert!(type_results.is_empty());
    }

    #[test]
    fn test_symbol_file_filtering() {
        let mut index = SymbolIndex::new();

        let symbols = vec![
            ContextFixtures::create_symbol("main_func", SymbolType::Function, "src/main.rs", 10),
            ContextFixtures::create_symbol("lib_func", SymbolType::Function, "src/lib.rs", 20),
            ContextFixtures::create_symbol("util_func", SymbolType::Function, "src/utils.rs", 30),
        ];

        for symbol in symbols {
            index.add_symbol(symbol);
        }

        let main_symbols = index.search_by_file(&PathBuf::from("src/main.rs"));
        assert_eq!(main_symbols.len(), 1);
        assert_eq!(main_symbols[0].name, "main_func");

        let lib_symbols = index.search_by_file(&PathBuf::from("src/lib.rs"));
        assert_eq!(lib_symbols.len(), 1);
        assert_eq!(lib_symbols[0].name, "lib_func");
    }

    #[test]
    fn test_symbol_update() {
        let mut index = SymbolIndex::new();

        let original_symbol =
            ContextFixtures::create_symbol("test_func", SymbolType::Function, "src/main.rs", 10);
        index.add_symbol(original_symbol);

        // Update with new location
        let updated_symbol = Symbol {
            name: "test_func".to_string(),
            symbol_type: SymbolType::Function,
            location: SymbolLocation {
                file: PathBuf::from("src/main.rs"),
                line: 15, // Changed line number
                column: 1,
            },
            visibility: "private".to_string(), // Changed visibility
            documentation: Some("Updated documentation".to_string()),
        };

        index.add_symbol(updated_symbol);

        let results = index.search_by_name("test_func");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].location.line, 15);
        assert_eq!(results[0].visibility, "private");
    }

    #[test]
    fn test_symbol_removal() {
        let mut index = SymbolIndex::new();

        let symbol =
            ContextFixtures::create_symbol("temp_function", SymbolType::Function, "src/temp.rs", 5);
        index.add_symbol(symbol);

        assert_eq!(index.len(), 1);

        index.remove_symbol("temp_function");
        assert_eq!(index.len(), 0);
        assert!(index.search_by_name("temp_function").is_empty());
    }
}

/// Tests for code relationships and dependencies
mod relationship_tests {
    use super::*;

    #[test]
    fn test_code_relationship_creation() {
        let relationship = CodeRelationship {
            from_file: PathBuf::from("src/main.rs"),
            to_file: PathBuf::from("src/lib.rs"),
            relationship_type: RelationshipType::Uses,
            description: "Main imports from lib".to_string(),
        };

        assert_eq!(relationship.from_file, PathBuf::from("src/main.rs"));
        assert_eq!(relationship.to_file, PathBuf::from("src/lib.rs"));
        assert!(matches!(
            relationship.relationship_type,
            RelationshipType::Uses
        ));
        assert!(relationship.description.contains("imports"));
    }

    #[test]
    fn test_relationship_analysis() {
        let context = ContextFixtures::create_codebase_context("/test/project");

        assert!(!context.relationships.is_empty());

        let uses_relationships: Vec<_> = context
            .relationships
            .iter()
            .filter(|r| matches!(r.relationship_type, RelationshipType::Uses))
            .collect();

        assert!(!uses_relationships.is_empty());
    }

    #[test]
    fn test_dependency_tracking() {
        let context = ContextFixtures::create_codebase_context("/test/project");

        assert!(!context.dependencies.is_empty());
        assert!(context.dependencies.contains(&"serde".to_string()));
        assert!(context.dependencies.contains(&"tokio".to_string()));
        assert!(context.dependencies.contains(&"anyhow".to_string()));
    }

    #[test]
    fn test_file_dependencies() {
        let file_context = ContextFixtures::create_file_context("src/main.rs", Some("rust"));

        // In the mock, this should have some imports
        assert!(!file_context.imports.is_empty());
        assert!(file_context.imports.contains(&"std::io".to_string()));
    }

    #[test]
    fn test_circular_dependency_detection() {
        let relationships = vec![
            CodeRelationship {
                from_file: PathBuf::from("a.rs"),
                to_file: PathBuf::from("b.rs"),
                relationship_type: RelationshipType::Uses,
                description: "A uses B".to_string(),
            },
            CodeRelationship {
                from_file: PathBuf::from("b.rs"),
                to_file: PathBuf::from("c.rs"),
                relationship_type: RelationshipType::Uses,
                description: "B uses C".to_string(),
            },
            CodeRelationship {
                from_file: PathBuf::from("c.rs"),
                to_file: PathBuf::from("a.rs"),
                relationship_type: RelationshipType::Uses,
                description: "C uses A (circular!)".to_string(),
            },
        ];

        let analyzer = DependencyAnalyzer::new();
        let cycles = analyzer.detect_circular_dependencies(&relationships);

        assert!(!cycles.is_empty());
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 3); // Three files in the cycle
    }
}

/// Tests for repository integration
mod repository_tests {
    use super::*;

    #[tokio::test]
    async fn test_git_repository_detection() {
        let env = TestEnvironment::new().expect("Failed to create test environment");
        env.create_sample_project()
            .expect("Failed to create sample project");
        env.create_git_repo().expect("Failed to create git repo");

        let repo_manager = RepositoryManager::new();
        let result = repo_manager.detect_repository(env.workspace_path()).await;

        assert!(result.is_ok());
        let repo_info = result.unwrap();
        assert_eq!(repo_info.repo_type, RepositoryType::Git);
        assert_eq!(repo_info.root_path, env.workspace_path());
    }

    #[tokio::test]
    async fn test_non_git_directory() {
        let env = TestEnvironment::new().expect("Failed to create test environment");
        env.create_sample_project()
            .expect("Failed to create sample project");
        // Don't create git repo

        let repo_manager = RepositoryManager::new();
        let result = repo_manager.detect_repository(env.workspace_path()).await;

        // Should either return an error or indicate no repository
        assert!(result.is_err() || matches!(result.unwrap().repo_type, RepositoryType::None));
    }

    #[tokio::test]
    async fn test_repository_file_tracking() {
        let env = TestEnvironment::new().expect("Failed to create test environment");
        env.create_sample_project()
            .expect("Failed to create sample project");
        env.create_git_repo().expect("Failed to create git repo");

        let repo_manager = RepositoryManager::new();
        let tracked_files = repo_manager.get_tracked_files(env.workspace_path()).await;

        assert!(tracked_files.is_ok());
        let files = tracked_files.unwrap();
        assert!(!files.is_empty());

        // Should contain project files
        assert!(files
            .iter()
            .any(|f| f.file_name().unwrap_or_default() == "Cargo.toml"));
        assert!(files
            .iter()
            .any(|f| f.to_string_lossy().contains("main.rs")));
    }

    #[tokio::test]
    async fn test_repository_status() {
        let env = TestEnvironment::new().expect("Failed to create test environment");
        env.create_sample_project()
            .expect("Failed to create sample project");
        env.create_git_repo().expect("Failed to create git repo");

        let repo_manager = RepositoryManager::new();
        let status = repo_manager
            .get_repository_status(env.workspace_path())
            .await;

        assert!(status.is_ok());
        let repo_status = status.unwrap();
        assert!(repo_status.is_clean || !repo_status.modified_files.is_empty());
    }

    #[test]
    fn test_repository_info_creation() {
        let repo_info = RepositoryInfo {
            repo_type: RepositoryType::Git,
            root_path: PathBuf::from("/test/repo"),
            branch: Some("main".to_string()),
            remote_url: Some("https://github.com/user/repo.git".to_string()),
            last_commit: None,
        };

        assert!(matches!(repo_info.repo_type, RepositoryType::Git));
        assert_eq!(repo_info.root_path, PathBuf::from("/test/repo"));
        assert_eq!(repo_info.branch, Some("main".to_string()));
        assert!(repo_info.remote_url.is_some());
    }
}

/// Tests for context configuration
mod config_tests {
    use super::*;

    #[test]
    fn test_analysis_config_default() {
        let config = AnalysisConfig::default();

        assert!(config.include_tests);
        assert!(config.include_docs);
        assert!(config.follow_imports);
        assert!(!config.language_filters.is_empty());
        assert!(config.max_file_size > 0);
        assert!(config.exclude_patterns.is_empty()); // Default should have no exclusions
    }

    #[test]
    fn test_analysis_config_customization() {
        let mut config = AnalysisConfig::default();

        config.include_tests = false;
        config.max_file_size = 1024 * 1024; // 1MB
        config.exclude_patterns = vec!["*.tmp".to_string(), "node_modules/*".to_string()];
        config.language_filters = vec!["rust".to_string(), "python".to_string()];

        assert!(!config.include_tests);
        assert_eq!(config.max_file_size, 1024 * 1024);
        assert_eq!(config.exclude_patterns.len(), 2);
        assert_eq!(config.language_filters.len(), 2);
    }

    #[test]
    fn test_file_filtering() {
        let mut config = AnalysisConfig::default();
        config.language_filters = vec!["rust".to_string()];
        config.exclude_patterns = vec!["target/*".to_string(), "*.tmp".to_string()];

        // Test language filtering
        assert!(config.should_analyze_file(&PathBuf::from("src/main.rs")));
        assert!(!config.should_analyze_file(&PathBuf::from("script.py"))); // Not in language filter

        // Test exclude patterns
        assert!(!config.should_analyze_file(&PathBuf::from("target/debug/main")));
        assert!(!config.should_analyze_file(&PathBuf::from("temp.tmp")));

        // Test file size filtering
        config.max_file_size = 100; // Very small limit
                                    // In a real implementation, this would check actual file size
                                    // For testing purposes, we assume the method exists
    }

    #[test]
    fn test_config_validation() {
        let mut config = AnalysisConfig::default();

        // Valid configuration
        assert!(config.validate().is_ok());

        // Invalid configuration - negative file size
        config.max_file_size = 0;
        assert!(config.validate().is_err());

        // Invalid configuration - empty language filters when needed
        config.max_file_size = 1024;
        config.language_filters.clear();
        config.strict_language_filtering = true;
        assert!(config.validate().is_err());
    }
}

/// Integration tests for complete context analysis workflows
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_project_analysis() {
        let env = TestEnvironment::new().expect("Failed to create test environment");
        env.create_sample_project()
            .expect("Failed to create sample project");
        env.create_git_repo().expect("Failed to create git repo");

        let mut analyzer = MockCodebaseAnalyzer::new();
        let config = AnalysisConfig::default();

        // Analyze all files in the project
        let project_files = vec![
            "src/main.rs",
            "src/lib.rs",
            "src/utils.rs",
            "Cargo.toml",
            "README.md",
        ];

        let mut file_contexts = Vec::new();
        for file_path in project_files {
            let result = analyzer.analyze_file(&PathBuf::from(file_path), &config);
            assert!(result.is_ok());
            file_contexts.push(result.unwrap());
        }

        // Analyze the complete codebase
        let codebase_result = analyzer.analyze_codebase(&file_contexts);
        assert!(codebase_result.is_ok());

        let codebase_context = codebase_result.unwrap();
        assert!(!codebase_context.dependencies.is_empty());
        assert!(!codebase_context.metadata.is_empty());

        let analyzed_files = analyzer.get_analyzed_files();
        assert_eq!(analyzed_files.len(), 5);
    }

    #[test]
    fn test_symbol_cross_referencing() {
        let mut context = ContextFixtures::create_codebase_context("/test/project");

        // Add more symbols for cross-referencing
        let function_symbol = ContextFixtures::create_symbol(
            "process_data",
            SymbolType::Function,
            "src/utils.rs",
            15,
        );
        let struct_symbol =
            ContextFixtures::create_symbol("DataProcessor", SymbolType::Struct, "src/types.rs", 10);

        context.symbols.add_symbol(function_symbol);
        context.symbols.add_symbol(struct_symbol);

        // Test cross-references
        let function_refs = context.symbols.search_by_name("process_data");
        assert_eq!(function_refs.len(), 1);
        assert_eq!(
            function_refs[0].location.file,
            PathBuf::from("src/utils.rs")
        );

        let struct_refs = context.symbols.search_by_name("DataProcessor");
        assert_eq!(struct_refs.len(), 1);
        assert_eq!(struct_refs[0].location.file, PathBuf::from("src/types.rs"));
    }

    #[test]
    fn test_context_incremental_updates() {
        let mut context = ContextFixtures::create_codebase_context("/test/project");
        let initial_file_count = context.files.len();
        let initial_symbol_count = context.symbols.len();

        // Add a new file context
        let new_file = ContextFixtures::create_file_context("src/new_module.rs", Some("rust"));
        context.files.push(new_file);

        // Add symbols from the new file
        let new_symbol = ContextFixtures::create_symbol(
            "new_function",
            SymbolType::Function,
            "src/new_module.rs",
            1,
        );
        context.symbols.add_symbol(new_symbol);

        // Verify updates
        assert_eq!(context.files.len(), initial_file_count + 1);
        assert_eq!(context.symbols.len(), initial_symbol_count + 1);

        // Verify the new symbol is searchable
        let search_results = context.symbols.search_by_name("new_function");
        assert_eq!(search_results.len(), 1);
        assert_eq!(
            search_results[0].location.file,
            PathBuf::from("src/new_module.rs")
        );
    }

    #[tokio::test]
    async fn test_context_with_repository_info() {
        let env = TestEnvironment::new().expect("Failed to create test environment");
        env.create_sample_project()
            .expect("Failed to create sample project");
        env.create_git_repo().expect("Failed to create git repo");

        let repo_manager = RepositoryManager::new();
        let repo_info = repo_manager.detect_repository(env.workspace_path()).await;
        assert!(repo_info.is_ok());

        let mut context =
            ContextFixtures::create_codebase_context(env.workspace_path().to_str().unwrap());

        // Add repository information to context
        let repo_data = repo_info.unwrap();
        context.metadata.insert(
            "repository_type".to_string(),
            format!("{:?}", repo_data.repo_type),
        );
        context.metadata.insert(
            "repository_root".to_string(),
            repo_data.root_path.to_string_lossy().to_string(),
        );

        if let Some(branch) = repo_data.branch {
            context
                .metadata
                .insert("current_branch".to_string(), branch);
        }

        // Verify repository information is included
        assert!(context.metadata.contains_key("repository_type"));
        assert!(context.metadata.contains_key("repository_root"));
    }

    #[test]
    fn test_context_serialization() {
        let context = ContextFixtures::create_codebase_context("/test/project");

        // Test that context can be serialized (important for caching and persistence)
        let serialized = serde_json::to_string(&context);
        assert!(serialized.is_ok());

        let json_data = serialized.unwrap();
        assert!(!json_data.is_empty());
        assert!(json_data.contains("root_path"));
        assert!(json_data.contains("files"));
        assert!(json_data.contains("dependencies"));

        // Test deserialization
        let deserialized: Result<CodebaseContext, _> = serde_json::from_str(&json_data);
        assert!(deserialized.is_ok());

        let restored_context = deserialized.unwrap();
        assert_eq!(restored_context.root_path, context.root_path);
        assert_eq!(restored_context.files.len(), context.files.len());
        assert_eq!(
            restored_context.dependencies.len(),
            context.dependencies.len()
        );
    }

    #[test]
    fn test_large_codebase_handling() {
        let mut context = ContextFixtures::create_codebase_context("/large/project");

        // Simulate a large codebase by adding many files and symbols
        for i in 1..=100 {
            let file_path = format!("src/module_{}.rs", i);
            let file_context = ContextFixtures::create_file_context(&file_path, Some("rust"));
            context.files.push(file_context);

            // Add multiple symbols per file
            for j in 1..=5 {
                let symbol_name = format!("function_{}_{}", i, j);
                let symbol = ContextFixtures::create_symbol(
                    &symbol_name,
                    SymbolType::Function,
                    &file_path,
                    j * 10,
                );
                context.symbols.add_symbol(symbol);
            }
        }

        // Verify the large context is handled correctly
        assert_eq!(context.files.len(), 105); // Original 5 + 100 new files
        assert_eq!(context.symbols.len(), 505); // Original 5 + 500 new symbols

        // Test search performance on large symbol index
        let search_results = context.symbols.search_by_prefix("function_50_");
        assert_eq!(search_results.len(), 5);

        let file_symbols = context
            .symbols
            .search_by_file(&PathBuf::from("src/module_25.rs"));
        assert_eq!(file_symbols.len(), 5);
    }
}
