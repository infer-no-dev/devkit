//! Context analysis and management tests

use crate::context::{CodebaseContext, FileContext, ContextManager};
use crate::context::symbols::{Symbol, SymbolType, SymbolIndex, Visibility};
use crate::context::analyzer::CodebaseAnalyzer;
use crate::testing::test_utils::{create_sample_rust_project, create_sample_python_project};

#[tokio::test]
async fn test_codebase_analyzer_creation() {
    let analyzer = CodebaseAnalyzer::new();
    
    // Test that analyzer can be created
    assert!(std::mem::size_of_val(&analyzer) > 0);
}

#[tokio::test]
async fn test_context_manager_creation() {
    let context_manager = ContextManager::new();
    
    // Test basic functionality
    assert!(context_manager.is_ok());
    let context_manager = context_manager.unwrap();
    assert!(std::mem::size_of_val(&context_manager) > 0);
}

#[tokio::test] 
async fn test_analyze_rust_project() {
    let (_temp_dir, project_path) = create_sample_rust_project();
    
    let analyzer = CodebaseAnalyzer::new();
    
    // Test analyzing the sample project
    let result = analyzer.analyze_project(&project_path).await;
    assert!(result.is_ok(), "Should be able to analyze Rust project");
    
    if let Ok(contexts) = result {
        assert!(!contexts.is_empty(), "Should find files to analyze");
        
        // Check that main.rs was analyzed
        let main_context = contexts.iter().find(|ctx| ctx.file_path.ends_with("main.rs"));
        assert!(main_context.is_some(), "Should have analyzed main.rs");
        
        if let Some(ctx) = main_context {
            assert!(!ctx.symbols.is_empty(), "Should have found symbols in main.rs");
            assert!(ctx.language == "rust", "Should detect Rust language");
        }
    }
}

#[tokio::test]
async fn test_analyze_python_project() {
    let (_temp_dir, project_path) = create_sample_python_project();
    
    let analyzer = CodebaseAnalyzer::new();
    
    // Test analyzing the sample Python project
    let result = analyzer.analyze_project(&project_path).await;
    assert!(result.is_ok(), "Should be able to analyze Python project");
    
    if let Ok(contexts) = result {
        assert!(!contexts.is_empty(), "Should find files to analyze");
        
        // Check that main.py was analyzed
        let main_context = contexts.iter().find(|ctx| ctx.file_path.ends_with("main.py"));
        assert!(main_context.is_some(), "Should have analyzed main.py");
        
        if let Some(ctx) = main_context {
            assert!(ctx.language == "python", "Should detect Python language");
        }
    }
}

#[test]
fn test_symbol_creation() {
    let symbol = Symbol {
        name: "test_function".to_string(),
        symbol_type: SymbolType::Function,
        file_path: std::path::PathBuf::from("test.rs"),
        line_number: 10,
        column: 5,
        signature: Some("fn test_function()".to_string()),
        documentation: Some("A test function".to_string()),
        visibility: Visibility::Public,
        references: Vec::new(),
    };
    
    assert_eq!(symbol.name, "test_function");
    assert!(matches!(symbol.symbol_type, SymbolType::Function));
    assert_eq!(symbol.line_number, 10);
    assert!(matches!(symbol.visibility, Visibility::Public));
}

#[test]
fn test_symbol_index() {
    let mut index = SymbolIndex::new();
    
    let symbol = Symbol {
        name: "test_symbol".to_string(),
        symbol_type: SymbolType::Variable,
        file_path: std::path::PathBuf::from("test.rs"),
        line_number: 5,
        column: 0,
        signature: None,
        documentation: None,
        visibility: Visibility::Private,
        references: Vec::new(),
    };
    
    index.add_symbol(symbol);
    
    // Test that symbol was added
    let symbols = index.find_symbols("test_symbol");
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "test_symbol");
}

#[test]
fn test_file_context() {
    let context = FileContext {
        path: std::path::PathBuf::from("src/main.rs"),
        relative_path: std::path::PathBuf::from("src/main.rs"),
        language: "rust".to_string(),
        size_bytes: 100,
        line_count: 3,
        last_modified: std::time::SystemTime::now(),
        content_hash: "abc123".to_string(),
        symbols: Vec::new(),
        imports: Vec::new(),
        exports: Vec::new(),
        relationships: Vec::new(),
    };
    
    assert_eq!(context.language, "rust");
    assert!(context.path.ends_with("main.rs"));
    assert_eq!(context.line_count, 3);
    assert_eq!(context.size_bytes, 100);
}

#[test]
fn test_codebase_context() {
    let context = CodebaseContext {
        root_path: std::path::PathBuf::from("/test/project"),
        files: Vec::new(),
        symbols: SymbolIndex::new(),
        dependencies: Vec::new(),
        repository_info: None,
        semantic_analysis: None,
        metadata: crate::context::ContextMetadata::default(),
    };
    
    assert!(context.root_path.ends_with("project"));
    assert!(context.files.is_empty());
    assert!(context.dependencies.is_empty());
}

#[test]
fn test_symbol_types() {
    let types = vec![
        SymbolType::Function,
        SymbolType::Struct,
        SymbolType::Enum,
        SymbolType::Variable,
        SymbolType::Constant,
        SymbolType::Module,
        SymbolType::Class,
        SymbolType::Interface,
    ];
    
    assert_eq!(types.len(), 8);
    
    // Test that types can be matched
    match types[0] {
        SymbolType::Function => assert!(true),
        _ => panic!("Expected Function type"),
    }
}

#[test]
fn test_symbol_visibility() {
    let visibilities = vec![
        Visibility::Public,
        Visibility::Private,
        Visibility::Protected,
        Visibility::Internal,
    ];
    
    assert_eq!(visibilities.len(), 4);
    
    // Test visibility matching
    match visibilities[0] {
        Visibility::Public => assert!(true),
        _ => panic!("Expected Public visibility"),
    }
}

#[tokio::test]
async fn test_context_manager_operations() {
    let context_manager = ContextManager::new();
    assert!(context_manager.is_ok());
    let mut context_manager = context_manager.unwrap();
    let (_temp_dir, project_path) = create_sample_rust_project();
    
    // Create a simple analysis config
    let config = crate::context::AnalysisConfig {
        include_patterns: vec!["*.rs".to_string()],
        exclude_patterns: Vec::new(),
        max_file_size_mb: 10,
        follow_symlinks: false,
        analyze_dependencies: true,
        deep_analysis: true,
        cache_results: false,
    };
    
    // Test basic context manager operations
    let result = context_manager.analyze_codebase(project_path, config).await;
    assert!(result.is_ok(), "Context manager should be able to analyze codebase");
    
    if let Ok(context) = result {
        assert!(!context.files.is_empty(), "Should generate context from sample project");
    }
}

#[test]
fn test_relationship_types() {
    use crate::context::RelationshipType;
    
    let relationships = vec![
        RelationshipType::Import,
        RelationshipType::Inheritance,
        RelationshipType::Composition,
        RelationshipType::Usage,
        RelationshipType::Reference,
    ];
    
    assert_eq!(relationships.len(), 5);
    
    // Test that relationships can be created and matched
    match relationships[0] {
        RelationshipType::Import => assert!(true),
        _ => panic!("Expected Import relationship"),
    }
}

#[tokio::test]
async fn test_concurrent_analysis() {
    let (_temp_dir, project_path) = create_sample_rust_project();
    
    // Test that multiple analyses can run concurrently
    let handles = (0..3).map(|_| {
        let path = project_path.clone();
        tokio::spawn(async move {
            let analyzer = CodebaseAnalyzer::new();
            analyzer.analyze_project(&path).await
        })
    }).collect::<Vec<_>>();
    
    let results = futures::future::join_all(handles).await;
    
    // All analyses should succeed
    for result in results {
        assert!(result.is_ok());
        if let Ok(analysis_result) = result {
            assert!(analysis_result.is_ok());
        }
    }
}
