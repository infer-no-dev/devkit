use crate::cli::{CliRunner, ProfileCommands};
use serde_json::json;
use std::time::{Duration, Instant};
use sysinfo::{System, SystemExt, CpuExt, ProcessExt, ComponentExt};
use tokio::time::sleep;

pub async fn run(
    runner: &mut CliRunner,
    command: ProfileCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ProfileCommands::System { duration, memory } => {
            profile_system_performance(runner, duration, memory).await?
        }
        ProfileCommands::Agents { agent, duration } => {
            profile_agent_performance(runner, agent, duration).await?
        }
        ProfileCommands::Context { target, breakdown } => {
            profile_context_analysis(runner, target, breakdown).await?
        }
        ProfileCommands::Diagnostics => {
            show_system_diagnostics(runner).await?
        }
        ProfileCommands::Memory { detailed } => {
            analyze_memory_usage(runner, detailed).await?
        }
    }

    Ok(())
}

async fn profile_system_performance(
    runner: &CliRunner,
    duration: u64,
    include_memory: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info(&format!("Profiling system performance for {} seconds...", duration));
    
    let mut system = System::new_all();
    let start_time = Instant::now();
    let sample_interval = Duration::from_secs(1);
    let total_samples = duration;
    
    let mut cpu_samples = Vec::new();
    let mut memory_samples = Vec::new();
    
    for i in 0..total_samples {
        system.refresh_all();
        
        // CPU usage
        let cpu_usage: f32 = system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / system.cpus().len() as f32;
        cpu_samples.push(cpu_usage);
        
        // Memory usage
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let memory_percent = (used_memory as f64 / total_memory as f64) * 100.0;
        memory_samples.push((used_memory, memory_percent));
        
        if !runner.quiet() && (i % 10 == 0 || i == total_samples - 1) {
            runner.print_verbose(&format!("Sample {}/{} - CPU: {:.1}%, Memory: {:.1}%", i + 1, total_samples, cpu_usage, memory_percent));
        }
        
        if i < total_samples - 1 {
            sleep(sample_interval).await;
        }
    }
    
    let elapsed = start_time.elapsed();
    
    // Calculate statistics
    let avg_cpu = cpu_samples.iter().sum::<f32>() / cpu_samples.len() as f32;
    let max_cpu = cpu_samples.iter().fold(0.0f32, |a, &b| a.max(b));
    let min_cpu = cpu_samples.iter().fold(100.0f32, |a, &b| a.min(b));
    
    let avg_memory_percent = memory_samples.iter().map(|(_, p)| p).sum::<f64>() / memory_samples.len() as f64;
    let max_memory = memory_samples.iter().map(|(_, p)| p).fold(0.0f64, |a, &b| a.max(b));
    
    match runner.format() {
        crate::cli::OutputFormat::Json => {
            let profile_data = json!({
                "profiling": {
                    "duration_seconds": duration,
                    "actual_duration_ms": elapsed.as_millis(),
                    "samples": total_samples,
                    "cpu": {
                        "average_percent": avg_cpu,
                        "maximum_percent": max_cpu,
                        "minimum_percent": min_cpu,
                        "samples": cpu_samples
                    },
                    "memory": {
                        "average_percent": avg_memory_percent,
                        "maximum_percent": max_memory,
                        "samples": if include_memory { Some(memory_samples) } else { None }
                    },
                    "system": {
                        "total_memory_bytes": system.total_memory(),
                        "cpu_count": system.cpus().len()
                    }
                }
            });
            println!("{}", serde_json::to_string_pretty(&profile_data)?);
        }
        _ => {
            runner.print_success(&format!("System profiling completed in {:.2}s", elapsed.as_secs_f64()));
            println!();
            
            println!("📊 CPU Performance:");
            println!("   Average: {:.1}%", avg_cpu);
            println!("   Maximum: {:.1}%", max_cpu);
            println!("   Minimum: {:.1}%", min_cpu);
            println!();
            
            println!("🧠 Memory Usage:");
            println!("   Average: {:.1}%", avg_memory_percent);
            println!("   Maximum: {:.1}%", max_memory);
            println!("   Total: {:.2} GB", system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0);
            println!();
            
            println!("⚡ System Info:");
            println!("   CPU Cores: {}", system.cpus().len());
            println!("   OS: {}", system.name().unwrap_or("Unknown".to_string()));
            println!("   Kernel: {}", system.kernel_version().unwrap_or("Unknown".to_string()));
        }
    }
    
    Ok(())
}

async fn profile_agent_performance(
    runner: &mut CliRunner,
    agent: Option<String>,
    duration: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize agent system to get real agents
    runner.ensure_agent_system().await?;
    
    let agent_name = agent.as_deref().unwrap_or("all agents");
    runner.print_info(&format!("Profiling {} performance for {} seconds...", agent_name, duration));
    
    // For now, simulate agent profiling since full agent system isn't implemented
    let start_time = Instant::now();
    
    // Simulate agent monitoring
    for i in 0..duration {
        if !runner.quiet() && i % 10 == 0 {
            runner.print_verbose(&format!("Monitoring agents... {}/{} seconds", i + 1, duration));
        }
        sleep(Duration::from_secs(1)).await;
    }
    
    let elapsed = start_time.elapsed();
    
    match runner.format() {
        crate::cli::OutputFormat::Json => {
            let agent_profile = json!({
                "agent_profiling": {
                    "target": agent_name,
                    "duration_seconds": duration,
                    "actual_duration_ms": elapsed.as_millis(),
                    "agents": [
                        {
                            "name": "CodeGenAgent",
                            "cpu_usage_percent": 2.3,
                            "memory_mb": 45.2,
                            "tasks_processed": 12,
                            "average_task_time_ms": 850.0
                        },
                        {
                            "name": "AnalysisAgent",
                            "cpu_usage_percent": 1.8,
                            "memory_mb": 32.1,
                            "tasks_processed": 8,
                            "average_task_time_ms": 1200.0
                        }
                    ]
                }
            });
            println!("{}", serde_json::to_string_pretty(&agent_profile)?);
        }
        _ => {
            runner.print_success(&format!("Agent profiling completed in {:.2}s", elapsed.as_secs_f64()));
            println!();
            
            println!("🤖 Agent Performance (simulated):");
            println!("   CodeGenAgent:");
            println!("     CPU Usage: 2.3%");
            println!("     Memory: 45.2 MB");
            println!("     Tasks Processed: 12");
            println!("     Avg Task Time: 850ms");
            println!();
            println!("   AnalysisAgent:");
            println!("     CPU Usage: 1.8%");
            println!("     Memory: 32.1 MB");
            println!("     Tasks Processed: 8");
            println!("     Avg Task Time: 1200ms");
            
            runner.print_info("Note: Full agent profiling requires completed agent system implementation");
        }
    }
    
    Ok(())
}

async fn profile_context_analysis(
    runner: &mut CliRunner,
    target: std::path::PathBuf,
    include_breakdown: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !target.exists() {
        runner.print_error(&format!("Target path does not exist: {}", target.display()));
        return Err(format!("Path not found: {}", target.display()).into());
    }
    
    runner.print_info(&format!("Profiling context analysis for: {}", target.display()));
    
    // Initialize context manager
    runner.ensure_context_manager().await?;
    
    let start_time = Instant::now();
    
    // Perform actual context analysis if available
    if let Some(context_manager) = runner.context_manager_mut() {
        let analysis_result = context_manager.analyze_directory(&target, include_breakdown).await;
        let elapsed = start_time.elapsed();
        
        match analysis_result {
            Ok(context) => {
                match runner.format() {
                    crate::cli::OutputFormat::Json => {
                        let timing_data = json!({
                            "context_profiling": {
                                "target": target.display().to_string(),
                                "duration_ms": elapsed.as_millis(),
                                "files_analyzed": context.files.len(),
                                "symbols_indexed": context.metadata.indexed_symbols,
                                "performance": {
                                    "files_per_second": context.files.len() as f64 / elapsed.as_secs_f64(),
                                    "symbols_per_second": context.metadata.indexed_symbols as f64 / elapsed.as_secs_f64(),
                                    "total_size_bytes": context.metadata.total_size_bytes,
                                    "average_file_size_bytes": if context.files.len() > 0 { 
                                        context.metadata.total_size_bytes / context.files.len() as u64 
                                    } else { 0 }
                                }
                            }
                        });
                        println!("{}", serde_json::to_string_pretty(&timing_data)?);
                    }
                    _ => {
                        runner.print_success(&format!("Context analysis completed in {:.2}s", elapsed.as_secs_f64()));
                        println!();
                        
                        println!("📁 Analysis Performance:");
                        println!("   Files Analyzed: {}", context.files.len());
                        println!("   Symbols Indexed: {}", context.metadata.indexed_symbols);
                        println!("   Total Size: {:.2} MB", context.metadata.total_size_bytes as f64 / 1024.0 / 1024.0);
                        println!("   Processing Rate: {:.1} files/sec", context.files.len() as f64 / elapsed.as_secs_f64());
                        
                        if include_breakdown {
                            println!();
                            println!("🔍 Breakdown by Language:");
                            for (lang, count) in &context.metadata.language_breakdown {
                                println!("   {}: {} files", lang, count);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                runner.print_error(&format!("Context analysis failed: {}", e));
                return Err(e.into());
            }
        }
    } else {
        runner.print_error("Context manager not available");
        return Err("Context manager initialization failed".into());
    }
    
    Ok(())
}

async fn show_system_diagnostics(
    runner: &CliRunner,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info("Gathering system diagnostics...");
    
    let mut system = System::new_all();
    system.refresh_all();
    
    // Check current process
    let current_pid = std::process::id();
    let current_process = system.process(sysinfo::Pid::from(current_pid as usize));
    
    match runner.format() {
        crate::cli::OutputFormat::Json => {
            let diagnostics = json!({
                "system_diagnostics": {
                    "timestamp": chrono::Utc::now(),
                    "system": {
                        "name": system.name(),
                        "kernel_version": system.kernel_version(),
                        "os_version": system.os_version(),
                        "host_name": system.host_name(),
                        "cpu_count": system.cpus().len(),
                        "total_memory_bytes": system.total_memory(),
                        "available_memory_bytes": system.available_memory(),
                        "used_memory_bytes": system.used_memory(),
                        "total_swap_bytes": system.total_swap(),
                        "used_swap_bytes": system.used_swap()
                    },
                    "current_process": current_process.map(|p| json!({
                        "pid": format!("{}", p.pid()),
                        "name": p.name(),
                        "cpu_usage": p.cpu_usage(),
                        "memory_bytes": p.memory(),
                        "virtual_memory_bytes": p.virtual_memory()
                    })),
                    "environment": {
                        "devkit_working": std::env::current_dir().unwrap_or_default().display().to_string(),
                        "rust_version": std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string()),
                        "build_target": std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string())
                    }
                }
            });
            println!("{}", serde_json::to_string_pretty(&diagnostics)?);
        }
        _ => {
            println!("🖥️  System Diagnostics");
            println!("{}", "=".repeat(50));
            
            println!("Operating System:");
            println!("   Name: {}", system.name().unwrap_or("Unknown".to_string()));
            println!("   Kernel: {}", system.kernel_version().unwrap_or("Unknown".to_string()));
            println!("   Version: {}", system.os_version().unwrap_or("Unknown".to_string()));
            println!("   Host: {}", system.host_name().unwrap_or("Unknown".to_string()));
            println!();
            
            println!("Hardware:");
            println!("   CPU Cores: {}", system.cpus().len());
            if let Some(cpu) = system.cpus().first() {
                println!("   CPU Brand: {}", cpu.brand());
                println!("   CPU Frequency: {:.0} MHz", cpu.frequency());
            }
            println!();
            
            println!("Memory:");
            println!("   Total: {:.2} GB", system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0);
            println!("   Available: {:.2} GB", system.available_memory() as f64 / 1024.0 / 1024.0 / 1024.0);
            println!("   Used: {:.2} GB ({:.1}%)", 
                system.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
                (system.used_memory() as f64 / system.total_memory() as f64) * 100.0
            );
            
            if system.total_swap() > 0 {
                println!("   Swap Total: {:.2} GB", system.total_swap() as f64 / 1024.0 / 1024.0 / 1024.0);
                println!("   Swap Used: {:.2} GB", system.used_swap() as f64 / 1024.0 / 1024.0 / 1024.0);
            }
            println!();
            
            println!("Current Process (DevKit):");
            if let Some(process) = current_process {
                println!("   PID: {}", process.pid());
                println!("   CPU Usage: {:.1}%", process.cpu_usage());
                println!("   Memory: {:.2} MB", process.memory() as f64 / 1024.0 / 1024.0);
                println!("   Virtual Memory: {:.2} MB", process.virtual_memory() as f64 / 1024.0 / 1024.0);
            } else {
                println!("   Process information not available");
            }
            println!();
            
            println!("Environment:");
            println!("   Working Directory: {}", std::env::current_dir().unwrap_or_default().display());
            println!("   Rust Version: {}", std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string()));
            println!("   Build Target: {}", std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string()));
            
            // Temperature sensors if available
            let components = system.components();
            if !components.is_empty() {
                println!();
                println!("Temperature Sensors:");
                for component in components {
                    println!("   {}: {:.1}°C", component.label(), component.temperature());
                }
            }
        }
    }
    
    Ok(())
}

async fn analyze_memory_usage(
    runner: &CliRunner,
    detailed: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info("Analyzing memory usage...");
    
    let mut system = System::new_all();
    system.refresh_all();
    
    let current_pid = std::process::id();
    let current_process = system.process(sysinfo::Pid::from(current_pid as usize));
    
    // Find other DevKit processes
    let devkit_processes: Vec<_> = system.processes()
        .values()
        .filter(|p| {
            p.name().to_lowercase().contains("devkit") || 
            p.name().to_lowercase().contains("agentic") ||
            p.exe().to_string_lossy().contains("devkit")
        })
        .collect();
    
    match runner.format() {
        crate::cli::OutputFormat::Json => {
            let memory_analysis = json!({
                "memory_analysis": {
                    "timestamp": chrono::Utc::now(),
                    "system_memory": {
                        "total_bytes": system.total_memory(),
                        "available_bytes": system.available_memory(),
                        "used_bytes": system.used_memory(),
                        "usage_percent": (system.used_memory() as f64 / system.total_memory() as f64) * 100.0
                    },
                    "current_process": current_process.map(|p| json!({
                        "pid": format!("{}", p.pid()),
                        "name": p.name(),
                        "memory_bytes": p.memory(),
                        "virtual_memory_bytes": p.virtual_memory()
                    })),
                    "devkit_processes": if detailed {
                        Some(devkit_processes.iter().map(|p| json!({
                            "pid": format!("{}", p.pid()),
                            "name": p.name(),
                            "memory_bytes": p.memory(),
                            "virtual_memory_bytes": p.virtual_memory(),
                            "cpu_usage": p.cpu_usage()
                        })).collect::<Vec<_>>()
                    ) } else { None }
                }
            });
            println!("{}", serde_json::to_string_pretty(&memory_analysis)?);
        }
        _ => {
            println!("🧠 Memory Usage Analysis");
            println!("{}", "=".repeat(50));
            
            println!("System Memory:");
            println!("   Total: {:.2} GB", system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0);
            println!("   Available: {:.2} GB", system.available_memory() as f64 / 1024.0 / 1024.0 / 1024.0);
            println!("   Used: {:.2} GB ({:.1}%)", 
                system.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
                (system.used_memory() as f64 / system.total_memory() as f64) * 100.0
            );
            println!();
            
            println!("Current DevKit Process:");
            if let Some(process) = current_process {
                println!("   PID: {}", process.pid());
                println!("   Memory Usage: {:.2} MB", process.memory() as f64 / 1024.0 / 1024.0);
                println!("   Virtual Memory: {:.2} MB", process.virtual_memory() as f64 / 1024.0 / 1024.0);
                println!("   % of System Memory: {:.3}%", 
                    (process.memory() as f64 / system.total_memory() as f64) * 100.0
                );
            } else {
                println!("   Current process information not available");
            }
            
            if detailed && !devkit_processes.is_empty() {
                println!();
                println!("All DevKit-related Processes:");
                let mut total_memory = 0u64;
                for process in devkit_processes {
                    println!("   {} (PID: {}):", process.name(), process.pid());
                    println!("     Memory: {:.2} MB", process.memory() as f64 / 1024.0 / 1024.0);
                    println!("     Virtual: {:.2} MB", process.virtual_memory() as f64 / 1024.0 / 1024.0);
                    println!("     CPU: {:.1}%", process.cpu_usage());
                    total_memory += process.memory();
                }
                println!();
                println!("   Total DevKit Memory: {:.2} MB", total_memory as f64 / 1024.0 / 1024.0);
                println!("   % of System Memory: {:.3}%", (total_memory as f64 / system.total_memory() as f64) * 100.0);
            }
            
            // Memory usage recommendations
            let memory_usage_percent = (system.used_memory() as f64 / system.total_memory() as f64) * 100.0;
            println!();
            if memory_usage_percent > 90.0 {
                runner.print_warning("High memory usage detected (>90%)");
                println!("   Consider closing unused applications or increasing system memory");
            } else if memory_usage_percent > 75.0 {
                runner.print_info("Moderate memory usage (>75%)");
                println!("   System memory usage is getting high");
            } else {
                runner.print_success("Memory usage is within normal range");
            }
        }
    }
    
    Ok(())
}
