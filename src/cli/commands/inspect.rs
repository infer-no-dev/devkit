use crate::cli::{CliRunner, InspectCommands};
use crate::context::RelationshipType;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;

pub async fn run(
    runner: &mut CliRunner,
    command: InspectCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        InspectCommands::Symbols {
            pattern,
            symbol_type,
            file,
        } => inspect_symbols(runner, pattern, symbol_type, file).await?,
        InspectCommands::File { path, detailed } => inspect_file(runner, path, detailed).await?,
        InspectCommands::Dependencies {
            targets,
            external_only,
            include_dev,
        } => inspect_dependencies(runner, targets, external_only, include_dev).await?,
        InspectCommands::Relationships {
            target,
            depth,
            types,
        } => inspect_relationships(runner, target, depth, types).await?,
        InspectCommands::Quality { targets, detailed } => {
            inspect_quality(runner, targets, detailed).await?
        }
    }

    Ok(())
}

async fn inspect_symbols(
    runner: &mut CliRunner,
    pattern: Option<String>,
    symbol_type: Option<String>,
    file: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info("Inspecting symbols in codebase...");

    // Ensure context manager is available
    runner.ensure_context_manager().await?;

    let current_dir = std::env::current_dir().unwrap_or_default();

    // Get or create context for the current directory
    let context = if let Some(context_manager) = runner.context_manager_mut() {
        context_manager
            .analyze_directory(&current_dir, false)
            .await?
    } else {
        return Err("Context manager not available".into());
    };

    let mut matching_symbols = Vec::new();

    // Filter by file first if specified
    let files_to_search: Vec<_> = if let Some(file_path) = &file {
        context
            .files
            .iter()
            .filter(|f| f.path == *file_path || f.relative_path == *file_path)
            .collect()
    } else {
        context.files.iter().collect()
    };

    // Search through symbols in relevant files
    for file_ctx in files_to_search {
        for symbol in &file_ctx.symbols {
            let mut matches = true;

            // Filter by pattern if specified
            if let Some(pattern) = &pattern {
                matches = matches
                    && (symbol.name.contains(pattern)
                        || symbol
                            .qualified_name
                            .as_ref()
                            .map(|qn| qn.contains(pattern))
                            .unwrap_or(false));
            }

            // Filter by symbol type if specified
            if let Some(sym_type) = &symbol_type {
                matches = matches
                    && symbol
                        .symbol_type
                        .to_string()
                        .to_lowercase()
                        .contains(&sym_type.to_lowercase());
            }

            if matches {
                matching_symbols.push((file_ctx, symbol));
            }
        }
    }

    // Output results
    match runner.format() {
        crate::cli::OutputFormat::Json => {
            let symbols_data = json!({
                "symbol_inspection": {
                    "search_criteria": {
                        "pattern": pattern,
                        "symbol_type": symbol_type,
                        "file": file.as_ref().map(|p| p.display().to_string())
                    },
                    "results_count": matching_symbols.len(),
                    "symbols": matching_symbols.iter().map(|(file_ctx, symbol)| {
                        json!({
                            "name": symbol.name,
                            "qualified_name": symbol.qualified_name,
                            "symbol_type": format!("{:?}", symbol.symbol_type),
                            "visibility": format!("{:?}", symbol.visibility),
                            "file": file_ctx.relative_path.display().to_string(),
                            "line": symbol.line,
                            "column": symbol.column,
                            "documentation": symbol.documentation
                        })
                    }).collect::<Vec<_>>()
                }
            });
            println!("{}", serde_json::to_string_pretty(&symbols_data)?);
        }
        _ => {
            if matching_symbols.is_empty() {
                runner.print_info("No symbols found matching the criteria");
                return Ok(());
            }

            runner.print_success(&format!(
                "Found {} matching symbols",
                matching_symbols.len()
            ));
            println!();

            for (file_ctx, symbol) in matching_symbols {
                println!("🔍 {}", symbol.name);
                if let Some(qualified_name) = &symbol.qualified_name {
                    println!("   Qualified: {}", qualified_name);
                }
                println!("   Type: {:?}", symbol.symbol_type);
                println!("   Visibility: {:?}", symbol.visibility);
                println!(
                    "   Location: {}:{}",
                    file_ctx.relative_path.display(),
                    symbol.line
                );
                if let Some(doc) = &symbol.documentation {
                    println!("   Documentation: {}", doc);
                }
                println!();
            }
        }
    }

    Ok(())
}

async fn inspect_file(
    runner: &mut CliRunner,
    path: PathBuf,
    detailed: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        runner.print_error(&format!("File does not exist: {}", path.display()));
        return Err(format!("File not found: {}", path.display()).into());
    }

    runner.print_info(&format!("Inspecting file: {}", path.display()));

    // Ensure context manager is available
    runner.ensure_context_manager().await?;

    let current_dir = std::env::current_dir().unwrap_or_default();

    // Get context for the current directory
    let context = if let Some(context_manager) = runner.context_manager_mut() {
        context_manager
            .analyze_directory(&current_dir, detailed)
            .await?
    } else {
        return Err("Context manager not available".into());
    };

    // Find the file in the context
    let file_context = context
        .files
        .iter()
        .find(|f| f.path == path || f.relative_path == path)
        .ok_or_else(|| format!("File not found in context: {}", path.display()))?;

    // Output file information
    match runner.format() {
        crate::cli::OutputFormat::Json => {
            let file_data = json!({
                "file_inspection": {
                    "path": file_context.path.display().to_string(),
                    "relative_path": file_context.relative_path.display().to_string(),
                    "language": file_context.language,
                    "size_bytes": file_context.size_bytes,
                    "line_count": file_context.line_count,
                    "content_hash": file_context.content_hash,
                    "last_modified": file_context.last_modified.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default().as_secs(),
                    "symbols": file_context.symbols.iter().map(|symbol| {
                        json!({
                            "name": symbol.name,
                            "qualified_name": symbol.qualified_name,
                            "symbol_type": format!("{:?}", symbol.symbol_type),
                            "visibility": format!("{:?}", symbol.visibility),
                            "line": symbol.line,
                            "column": symbol.column
                        })
                    }).collect::<Vec<_>>(),
                    "imports": file_context.imports,
                    "exports": file_context.exports,
                    "relationships": if detailed {
                        Some(file_context.relationships.iter().map(|rel| {
                            json!({
                                "target_file": rel.target_file.display().to_string(),
                                "relationship_type": format!("{:?}", rel.relationship_type),
                                "line_numbers": rel.line_numbers
                            })
                        }).collect::<Vec<_>>())
                    } else { None }
                }
            });
            println!("{}", serde_json::to_string_pretty(&file_data)?);
        }
        _ => {
            println!("📄 File: {}", file_context.relative_path.display());
            println!("🗣️  Language: {}", file_context.language);
            println!("📏 Size: {:.2} KB", file_context.size_bytes as f64 / 1024.0);
            println!("📝 Lines: {}", file_context.line_count);
            println!("🔒 Hash: {}", &file_context.content_hash[..8]);
            println!();

            if !file_context.symbols.is_empty() {
                println!("🔍 Symbols ({}):", file_context.symbols.len());
                for symbol in &file_context.symbols {
                    println!(
                        "  • {} ({:?}) at line {}",
                        symbol.name, symbol.symbol_type, symbol.line
                    );
                }
                println!();
            }

            if !file_context.imports.is_empty() {
                println!("📥 Imports ({}):", file_context.imports.len());
                for import in &file_context.imports {
                    println!("  • {}", import);
                }
                println!();
            }

            if !file_context.exports.is_empty() {
                println!("📤 Exports ({}):", file_context.exports.len());
                for export in &file_context.exports {
                    println!("  • {}", export);
                }
                println!();
            }

            if detailed && !file_context.relationships.is_empty() {
                println!("🔗 Relationships ({}):", file_context.relationships.len());
                for rel in &file_context.relationships {
                    println!(
                        "  • {:?} -> {}",
                        rel.relationship_type,
                        rel.target_file.display()
                    );
                }
            }
        }
    }

    Ok(())
}

async fn inspect_dependencies(
    runner: &mut CliRunner,
    targets: Vec<PathBuf>,
    external_only: bool,
    include_dev: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let targets = if targets.is_empty() {
        vec![std::env::current_dir().unwrap_or_default()]
    } else {
        targets
    };

    runner.print_info(&format!(
        "Analyzing dependencies for {} targets",
        targets.len()
    ));

    // Ensure context manager is available
    runner.ensure_context_manager().await?;

    let mut all_dependencies = Vec::new();

    for target in &targets {
        let context = if let Some(context_manager) = runner.context_manager_mut() {
            context_manager.analyze_directory(target, true).await?
        } else {
            return Err("Context manager not available".into());
        };

        // Filter dependencies based on criteria
        let filtered_deps: Vec<_> = context
            .dependencies
            .iter()
            .filter(|dep| {
                if external_only {
                    // Only external dependencies (from package managers)
                    matches!(
                        dep.source,
                        crate::context::DependencySource::PackageManager(_)
                    )
                } else {
                    true
                }
            })
            .filter(|dep| {
                if include_dev {
                    true
                } else {
                    // Exclude development dependencies
                    !matches!(
                        dep.dependency_type,
                        crate::context::DependencyType::Development
                    )
                }
            })
            .collect();

        all_dependencies.extend(filtered_deps.into_iter().cloned());
    }

    // Remove duplicates
    all_dependencies.sort_by(|a, b| a.name.cmp(&b.name));
    all_dependencies.dedup_by(|a, b| a.name == b.name && a.version == b.version);

    // Output results
    match runner.format() {
        crate::cli::OutputFormat::Json => {
            let deps_data = json!({
                "dependency_inspection": {
                    "targets": targets.iter().map(|t| t.display().to_string()).collect::<Vec<_>>(),
                    "filters": {
                        "external_only": external_only,
                        "include_dev": include_dev
                    },
                    "dependency_count": all_dependencies.len(),
                    "dependencies": all_dependencies.iter().map(|dep| {
                        json!({
                            "name": dep.name,
                            "version": dep.version,
                            "dependency_type": format!("{:?}", dep.dependency_type),
                            "source": format!("{:?}", dep.source)
                        })
                    }).collect::<Vec<_>>()
                }
            });
            println!("{}", serde_json::to_string_pretty(&deps_data)?);
        }
        _ => {
            if all_dependencies.is_empty() {
                runner.print_info("No dependencies found matching the criteria");
                return Ok(());
            }

            runner.print_success(&format!("Found {} dependencies", all_dependencies.len()));
            println!();

            // Group by dependency type
            let mut by_type: HashMap<String, Vec<_>> = HashMap::new();
            for dep in &all_dependencies {
                by_type
                    .entry(format!("{:?}", dep.dependency_type))
                    .or_default()
                    .push(dep);
            }

            for (dep_type, deps) in by_type {
                println!("📦 {} Dependencies ({}):", dep_type, deps.len());
                for dep in deps {
                    let version_str = dep.version.as_deref().unwrap_or("unknown");
                    println!("  • {} ({})", dep.name, version_str);
                    if let crate::context::DependencySource::PackageManager(pm) = &dep.source {
                        println!("    Source: {}", pm);
                    }
                }
                println!();
            }
        }
    }

    Ok(())
}

async fn inspect_relationships(
    runner: &mut CliRunner,
    target: String,
    depth: usize,
    types: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info(&format!("Analyzing relationships for: {}", target));

    // Ensure context manager is available
    runner.ensure_context_manager().await?;

    let current_dir = std::env::current_dir().unwrap_or_default();

    // Get context for the current directory
    let context = if let Some(context_manager) = runner.context_manager_mut() {
        context_manager
            .analyze_directory(&current_dir, true)
            .await?
    } else {
        return Err("Context manager not available".into());
    };

    // Convert string types to RelationshipType enum
    let relationship_types: Vec<RelationshipType> = types
        .iter()
        .filter_map(|t| match t.to_lowercase().as_str() {
            "imports" => Some(RelationshipType::Imports),
            "extends" => Some(RelationshipType::Extends),
            "implements" => Some(RelationshipType::Implements),
            "references" => Some(RelationshipType::References),
            "tests" => Some(RelationshipType::Tests),
            "documentation" => Some(RelationshipType::Documentation),
            _ => None,
        })
        .collect();

    let relationship_types = if relationship_types.is_empty() {
        // Use all types if none specified
        vec![
            RelationshipType::Imports,
            RelationshipType::Extends,
            RelationshipType::Implements,
            RelationshipType::References,
            RelationshipType::Tests,
            RelationshipType::Documentation,
        ]
    } else {
        relationship_types
    };

    // Find the target file or symbol
    let target_path = PathBuf::from(&target);
    let mut relationships = Vec::new();

    if let Some(context_manager) = runner.context_manager_mut() {
        let related_files =
            context_manager.find_related_files(&target_path, &context, &relationship_types);

        for file_path in related_files {
            relationships.push(format!("{} -> {}", target, file_path.display()));
        }
    }

    // Output results
    match runner.format() {
        crate::cli::OutputFormat::Json => {
            let relationships_data = json!({
                "relationship_inspection": {
                    "target": target,
                    "max_depth": depth,
                    "relationship_types": types,
                    "relationships_count": relationships.len(),
                    "relationships": relationships
                }
            });
            println!("{}", serde_json::to_string_pretty(&relationships_data)?);
        }
        _ => {
            if relationships.is_empty() {
                runner.print_info(&format!("No relationships found for: {}", target));
                return Ok(());
            }

            runner.print_success(&format!("Found {} relationships", relationships.len()));
            println!();

            println!("🔗 Relationships for: {}", target);
            for relationship in relationships {
                println!("  • {}", relationship);
            }
        }
    }

    Ok(())
}

async fn inspect_quality(
    runner: &mut CliRunner,
    targets: Vec<PathBuf>,
    detailed: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let targets = if targets.is_empty() {
        vec![std::env::current_dir().unwrap_or_default()]
    } else {
        targets
    };

    runner.print_info(&format!(
        "Analyzing code quality for {} targets",
        targets.len()
    ));

    // Ensure context manager is available
    runner.ensure_context_manager().await?;

    let mut quality_metrics = QualityMetrics::new();

    for target in &targets {
        let context = if let Some(context_manager) = runner.context_manager_mut() {
            context_manager.analyze_directory(target, detailed).await?
        } else {
            return Err("Context manager not available".into());
        };

        // Calculate quality metrics
        for file_ctx in &context.files {
            quality_metrics.total_files += 1;
            quality_metrics.total_lines += file_ctx.line_count;
            quality_metrics.total_symbols += file_ctx.symbols.len();

            // Calculate average lines per file
            if file_ctx.line_count > 0 {
                quality_metrics.avg_lines_per_file =
                    quality_metrics.total_lines as f64 / quality_metrics.total_files as f64;
            }

            // Check for potential issues
            if file_ctx.line_count > 1000 {
                quality_metrics.large_files += 1;
            }

            if file_ctx.symbols.is_empty() && file_ctx.line_count > 10 {
                quality_metrics.files_without_symbols += 1;
            }

            // Documentation coverage
            let documented_symbols = file_ctx
                .symbols
                .iter()
                .filter(|s| s.documentation.is_some())
                .count();
            quality_metrics.documented_symbols += documented_symbols;

            // Test coverage estimation (simplified)
            if file_ctx.relative_path.to_string_lossy().contains("test")
                || file_ctx.relative_path.to_string_lossy().contains("spec")
            {
                quality_metrics.test_files += 1;
            }
        }

        // Calculate percentages
        if quality_metrics.total_symbols > 0 {
            quality_metrics.documentation_coverage = (quality_metrics.documented_symbols as f64
                / quality_metrics.total_symbols as f64)
                * 100.0;
        }

        if quality_metrics.total_files > 0 {
            quality_metrics.test_coverage_estimate =
                (quality_metrics.test_files as f64 / quality_metrics.total_files as f64) * 100.0;
        }
    }

    // Output results
    match runner.format() {
        crate::cli::OutputFormat::Json => {
            let quality_data = json!({
                "quality_inspection": {
                    "targets": targets.iter().map(|t| t.display().to_string()).collect::<Vec<_>>(),
                    "detailed": detailed,
                    "metrics": {
                        "total_files": quality_metrics.total_files,
                        "total_lines": quality_metrics.total_lines,
                        "total_symbols": quality_metrics.total_symbols,
                        "avg_lines_per_file": quality_metrics.avg_lines_per_file,
                        "large_files_count": quality_metrics.large_files,
                        "files_without_symbols": quality_metrics.files_without_symbols,
                        "documented_symbols": quality_metrics.documented_symbols,
                        "documentation_coverage_percent": quality_metrics.documentation_coverage,
                        "test_files": quality_metrics.test_files,
                        "test_coverage_estimate_percent": quality_metrics.test_coverage_estimate
                    }
                }
            });
            println!("{}", serde_json::to_string_pretty(&quality_data)?);
        }
        _ => {
            println!("📊 Code Quality Analysis");
            println!("{}", "=".repeat(50));
            println!();

            println!("📁 File Metrics:");
            println!("   Total Files: {}", quality_metrics.total_files);
            println!("   Total Lines: {}", quality_metrics.total_lines);
            println!(
                "   Average Lines per File: {:.1}",
                quality_metrics.avg_lines_per_file
            );
            if quality_metrics.large_files > 0 {
                runner.print_warning(&format!(
                    "   Large Files (>1000 lines): {}",
                    quality_metrics.large_files
                ));
            }
            println!();

            println!("🔍 Symbol Metrics:");
            println!("   Total Symbols: {}", quality_metrics.total_symbols);
            if quality_metrics.total_files > 0 {
                println!(
                    "   Average Symbols per File: {:.1}",
                    quality_metrics.total_symbols as f64 / quality_metrics.total_files as f64
                );
            }
            if quality_metrics.files_without_symbols > 0 {
                runner.print_warning(&format!(
                    "   Files Without Symbols: {}",
                    quality_metrics.files_without_symbols
                ));
            }
            println!();

            println!("📖 Documentation:");
            println!(
                "   Documented Symbols: {}/{}",
                quality_metrics.documented_symbols, quality_metrics.total_symbols
            );
            println!(
                "   Documentation Coverage: {:.1}%",
                quality_metrics.documentation_coverage
            );

            if quality_metrics.documentation_coverage < 50.0 {
                runner.print_warning("   Low documentation coverage (<50%)");
            } else if quality_metrics.documentation_coverage >= 80.0 {
                runner.print_success("   Excellent documentation coverage (≥80%)");
            }
            println!();

            println!("🧪 Testing:");
            println!("   Test Files: {}", quality_metrics.test_files);
            println!(
                "   Estimated Test Coverage: {:.1}%",
                quality_metrics.test_coverage_estimate
            );

            if quality_metrics.test_coverage_estimate < 20.0 {
                runner.print_warning("   Low test coverage estimate (<20%)");
            }

            // Overall quality score
            let mut quality_score = 0.0;
            let mut factors = 0;

            // Documentation factor (0-30 points)
            quality_score += (quality_metrics.documentation_coverage / 100.0) * 30.0;
            factors += 1;

            // Test coverage factor (0-30 points)
            quality_score += (quality_metrics.test_coverage_estimate / 100.0) * 30.0;
            factors += 1;

            // File size factor (0-20 points)
            let large_file_ratio =
                quality_metrics.large_files as f64 / quality_metrics.total_files as f64;
            quality_score += (1.0 - large_file_ratio) * 20.0;
            factors += 1;

            // Symbol coverage factor (0-20 points)
            let files_with_symbols_ratio =
                (quality_metrics.total_files - quality_metrics.files_without_symbols) as f64
                    / quality_metrics.total_files as f64;
            quality_score += files_with_symbols_ratio * 20.0;
            factors += 1;

            println!();
            println!("⭐ Overall Quality Score: {:.1}/100", quality_score);

            if quality_score >= 80.0 {
                runner.print_success("   Excellent code quality! 🎉");
            } else if quality_score >= 60.0 {
                runner.print_info("   Good code quality with room for improvement");
            } else {
                runner.print_warning("   Code quality needs improvement");
            }
        }
    }

    Ok(())
}

#[derive(Debug, Default)]
struct QualityMetrics {
    total_files: usize,
    total_lines: usize,
    total_symbols: usize,
    avg_lines_per_file: f64,
    large_files: usize,
    files_without_symbols: usize,
    documented_symbols: usize,
    documentation_coverage: f64,
    test_files: usize,
    test_coverage_estimate: f64,
}

impl QualityMetrics {
    fn new() -> Self {
        Self::default()
    }
}
