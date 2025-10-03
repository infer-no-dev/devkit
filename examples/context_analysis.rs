//! Context Analysis System Example
//!
//! This example demonstrates the full context analysis workflow:
//! 1. Initialize context manager
//! 2. Analyze codebase structure and extract metadata
//! 3. Build symbol index with cross-references
//! 4. Analyze repository information (Git integration)
//! 5. Query the context for insights and relationships

use devkit_env::context::symbols::SymbolType;
use devkit_env::context::{AnalysisConfig, ContextManager};
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🔍 Context Analysis System Example");
    println!("==================================\n");

    // Get the current directory or use command line argument
    let target_path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("Failed to get current directory"));

    println!("📂 Target directory: {}", target_path.display());

    // Step 1: Initialize Context Manager
    println!("\n🔧 Initializing Context Manager...");
    let mut context_manager = match ContextManager::new() {
        Ok(manager) => {
            println!("✅ Context Manager initialized successfully");
            manager
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize Context Manager: {}", e);
            return Ok(());
        }
    };

    // Step 2: Configure Analysis Settings
    println!("\n⚙️ Configuring Analysis Settings...");
    let config = AnalysisConfig {
        include_patterns: vec![
            "**/*.rs".to_string(),
            "**/*.py".to_string(),
            "**/*.js".to_string(),
            "**/*.ts".to_string(),
            "**/*.go".to_string(),
            "**/*.java".to_string(),
        ],
        exclude_patterns: vec![
            "**/target/**".to_string(),
            "**/node_modules/**".to_string(),
            "**/.git/**".to_string(),
            "**/build/**".to_string(),
            "**/dist/**".to_string(),
        ],
        max_file_size_mb: 5,
        follow_symlinks: false,
        analyze_dependencies: true,
        deep_analysis: true,
        cache_results: true,
    };

    println!("✅ Analysis configured:");
    println!("   • Include patterns: {:?}", config.include_patterns);
    println!("   • Max file size: {}MB", config.max_file_size_mb);
    println!("   • Deep analysis: {}", config.deep_analysis);
    println!("   • Dependency analysis: {}", config.analyze_dependencies);

    // Step 3: Perform Comprehensive Codebase Analysis
    println!("\n🚀 Analyzing codebase...");
    let analysis_start = std::time::Instant::now();

    let context = match context_manager
        .analyze_codebase(target_path.clone(), config)
        .await
    {
        Ok(context) => {
            let analysis_duration = analysis_start.elapsed();
            println!(
                "✅ Analysis completed in {:.2}s",
                analysis_duration.as_secs_f64()
            );
            context
        }
        Err(e) => {
            eprintln!("❌ Analysis failed: {}", e);
            return Ok(());
        }
    };

    // Step 4: Display Analysis Results
    println!("\n📊 Analysis Results");
    println!("==================");
    println!("📁 Root path: {}", context.root_path.display());
    println!("📄 Total files: {}", context.metadata.total_files);
    println!("📝 Total lines: {}", context.metadata.total_lines);
    println!("🔍 Indexed symbols: {}", context.metadata.indexed_symbols);
    println!(
        "⏱️  Analysis time: {}ms",
        context.metadata.analysis_duration_ms
    );

    // Display language distribution
    println!("\n🌐 Language Distribution:");
    let mut langs: Vec<_> = context.metadata.languages.iter().collect();
    langs.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending
    for (language, count) in langs {
        let percentage = (*count as f64 / context.metadata.total_files as f64) * 100.0;
        println!("   • {}: {} files ({:.1}%)", language, count, percentage);
    }

    // Display dependencies if available
    if !context.dependencies.is_empty() {
        println!("\n📦 Dependencies ({}):", context.dependencies.len());
        for (i, dep) in context.dependencies.iter().enumerate().take(10) {
            println!("   {}. {} ({:?})", i + 1, dep.name, dep.dependency_type);
        }
        if context.dependencies.len() > 10 {
            println!("   ... and {} more", context.dependencies.len() - 10);
        }
    }

    // Step 5: Repository Information
    if let Some(ref repo_info) = context.repository_info {
        println!("\n🗃️  Repository Information:");
        if let Some(ref branch) = repo_info.current_branch {
            println!("   • Current branch: {}", branch);
        }
        if let Some(ref remote) = repo_info.remote_url {
            println!("   • Remote URL: {}", remote);
        }
        println!(
            "   • Status: {}",
            if repo_info.status.is_clean {
                "Clean"
            } else {
                "Modified"
            }
        );

        if !repo_info.status.is_clean {
            if !repo_info.status.modified_files.is_empty() {
                println!(
                    "   • Modified files: {}",
                    repo_info.status.modified_files.len()
                );
            }
            if !repo_info.status.untracked_files.is_empty() {
                println!(
                    "   • Untracked files: {}",
                    repo_info.status.untracked_files.len()
                );
            }
            if !repo_info.status.staged_files.is_empty() {
                println!("   • Staged files: {}", repo_info.status.staged_files.len());
            }
        }

        if !repo_info.recent_commits.is_empty() {
            println!("   • Recent commits: {}", repo_info.recent_commits.len());
            for (i, commit) in repo_info.recent_commits.iter().enumerate().take(3) {
                let short_hash = if commit.hash.len() > 7 {
                    &commit.hash[..7]
                } else {
                    &commit.hash
                };
                println!(
                    "     {}. {} - {} ({})",
                    i + 1,
                    short_hash,
                    commit.message.chars().take(50).collect::<String>(),
                    commit.author
                );
            }
        }
    }

    // Step 6: Symbol Analysis
    println!("\n🎯 Symbol Analysis");
    println!("==================");

    let symbol_stats = analyze_symbols(&context);
    println!("📈 Symbol Statistics:");
    for (symbol_type, count) in symbol_stats {
        println!("   • {:?}: {}", symbol_type, count);
    }

    // Step 7: Demonstrate Context Queries
    println!("\n🔎 Context Query Examples");
    println!("========================");

    // Query 1: Search for main functions
    println!("\n🔍 Searching for 'main' functions...");
    let main_symbols =
        context_manager.search_symbols("main", &context, Some(&[SymbolType::Function]));

    if !main_symbols.is_empty() {
        println!("Found {} main function(s):", main_symbols.len());
        for (i, symbol) in main_symbols.iter().enumerate().take(5) {
            println!(
                "   {}. {} in {} (line {})",
                i + 1,
                symbol.name,
                symbol
                    .file_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
                symbol.line_number
            );
        }
    } else {
        println!("No main functions found.");
    }

    // Query 2: Find the largest files
    println!("\n📊 Largest files by line count:");
    let mut files_by_size = context.files.clone();
    files_by_size.sort_by(|a, b| b.line_count.cmp(&a.line_count));

    for (i, file) in files_by_size.iter().enumerate().take(5) {
        println!(
            "   {}. {} ({} lines, {})",
            i + 1,
            file.relative_path.display(),
            file.line_count,
            file.language
        );
    }

    // Query 3: Search for test files
    println!("\n🧪 Test files:");
    let test_files: Vec<_> = context
        .files
        .iter()
        .filter(|f| {
            let path_str = f.relative_path.to_string_lossy().to_lowercase();
            path_str.contains("test")
                || path_str.contains("spec")
                || f.relative_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.ends_with("_test") || s.ends_with(".test"))
                    .unwrap_or(false)
        })
        .collect();

    if !test_files.is_empty() {
        println!("Found {} test file(s):", test_files.len());
        for (i, file) in test_files.iter().enumerate().take(5) {
            println!(
                "   {}. {} ({} lines)",
                i + 1,
                file.relative_path.display(),
                file.line_count
            );
        }
    } else {
        println!("No test files found.");
    }

    // Step 8: Demonstrate File Relationship Analysis
    println!("\n🔗 File Relationships");
    println!("====================");

    // Find files with imports/dependencies
    let files_with_imports: Vec<_> = context
        .files
        .iter()
        .filter(|f| !f.imports.is_empty())
        .collect();

    println!("📥 Files with imports: {}", files_with_imports.len());

    if !files_with_imports.is_empty() {
        for (i, file) in files_with_imports.iter().enumerate().take(3) {
            println!(
                "   {}. {} ({} imports)",
                i + 1,
                file.relative_path.display(),
                file.imports.len()
            );

            // Show first few imports
            for (_j, import) in file.imports.iter().enumerate().take(3) {
                println!("      - {}", import);
            }
            if file.imports.len() > 3 {
                println!("      ... and {} more", file.imports.len() - 3);
            }
        }
    }

    // Files with exports
    let files_with_exports: Vec<_> = context
        .files
        .iter()
        .filter(|f| !f.exports.is_empty())
        .collect();

    println!("\n📤 Files with exports: {}", files_with_exports.len());

    // Step 9: Advanced Queries
    println!("\n🎯 Advanced Analysis");
    println!("===================");

    // Find files that might be configuration
    let config_files: Vec<_> = context
        .files
        .iter()
        .filter(|f| {
            let name = f
                .relative_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();
            name.contains("config")
                || name.contains("setting")
                || matches!(f.language.as_str(), "toml" | "yaml" | "json")
        })
        .collect();

    println!("⚙️ Configuration files: {}", config_files.len());
    for (i, file) in config_files.iter().enumerate().take(5) {
        println!(
            "   {}. {} ({})",
            i + 1,
            file.relative_path.display(),
            file.language
        );
    }

    // Language complexity analysis
    println!("\n📈 Code Complexity Analysis:");
    let total_lines = context.metadata.total_lines as f64;
    for (language, file_count) in &context.metadata.languages {
        let lang_lines: usize = context
            .files
            .iter()
            .filter(|f| f.language == *language)
            .map(|f| f.line_count)
            .sum();

        let avg_lines_per_file = if *file_count > 0 {
            lang_lines as f64 / *file_count as f64
        } else {
            0.0
        };

        println!(
            "   • {}: {:.1} avg lines/file ({:.1}% of total code)",
            language,
            avg_lines_per_file,
            (lang_lines as f64 / total_lines) * 100.0
        );
    }

    println!("\n🎉 Context analysis completed!");
    println!("\n💡 Context Usage Tips:");
    println!("   • Use the context to provide better code suggestions to AI agents");
    println!("   • Query symbols to understand code structure and relationships");
    println!("   • Monitor repository changes to trigger incremental analysis");
    println!("   • Use file relationships for dependency-aware code generation");
    println!("   • Cache analysis results for faster subsequent queries");

    Ok(())
}

/// Analyze symbols and return statistics by type
fn analyze_symbols(
    context: &devkit_env::context::CodebaseContext,
) -> Vec<(SymbolType, usize)> {
    use std::collections::HashMap;

    let mut symbol_counts: HashMap<SymbolType, usize> = HashMap::new();

    // Count symbols by type
    for file in &context.files {
        for symbol in &file.symbols {
            *symbol_counts.entry(symbol.symbol_type.clone()).or_insert(0) += 1;
        }
    }

    // Also count from the symbol index
    for symbol_type in [
        SymbolType::Function,
        SymbolType::Struct,
        SymbolType::Class,
        SymbolType::Interface,
        SymbolType::Enum,
        SymbolType::Trait,
        SymbolType::Variable,
        SymbolType::Constant,
    ] {
        let count = context.symbols.find_symbols_by_type(&symbol_type).len();
        if count > 0 {
            *symbol_counts.entry(symbol_type).or_insert(0) += count;
        }
    }

    let mut stats: Vec<_> = symbol_counts.into_iter().collect();
    stats.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending
    stats
}
