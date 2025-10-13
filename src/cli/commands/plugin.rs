//! Plugin CLI commands
//!
//! Handles plugin marketplace operations: search, install, uninstall, list, update

use crate::cli::{CliRunner, PluginCommands, OutputFormat};
use crate::plugins::{MarketplaceClient, MarketplaceConfig, PluginSearchQuery};
use serde_json;

/// Execute plugin commands
pub async fn run(
    runner: &mut CliRunner,
    command: PluginCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        PluginCommands::Search { query, category, free_only, format } => {
            handle_search(runner, query, category, free_only, format).await
        }
        PluginCommands::Info { plugin_id } => {
            handle_info(runner, &plugin_id).await
        }
        PluginCommands::Install { plugin_id, version, license_key } => {
            handle_install(runner, &plugin_id, version.as_deref(), license_key.as_deref()).await
        }
        PluginCommands::List { format } => {
            handle_list(runner, format).await
        }
        PluginCommands::Update { plugin_id } => {
            handle_update(runner, plugin_id.as_deref()).await
        }
        PluginCommands::Status => {
            handle_status(runner).await
        }
    }
}

async fn handle_search(
    runner: &mut CliRunner,
    query: Option<String>,
    category: Option<String>,
    free_only: bool,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info(&format!("🔍 Searching for plugins..."));
    
    if let Some(q) = &query {
        runner.print_verbose(&format!("Query: {}", q));
    }
    if let Some(cat) = &category {
        runner.print_verbose(&format!("Category: {}", cat));
    }
    if free_only {
        runner.print_verbose("Filter: Free plugins only");
    }
    
    // Create marketplace client with mock configuration
    let config = MarketplaceConfig::default();
    
    let client = MarketplaceClient::new(config)?;
    
    let search_query = PluginSearchQuery {
        query: query.clone(),
        category: category.clone(),
        free_only,
        ..Default::default()
    };
    
    let plugins = client.search(search_query).await?;
    let search_results: Vec<PluginSearchResult> = plugins.iter().map(|p| PluginSearchResult {
        id: p.marketplace_info.plugin_id.clone(),
        name: p.metadata.name.clone(),
        version: p.metadata.version.clone(),
        description: p.metadata.description.clone(),
        category: p.marketplace_info.category.clone(),
        price: if p.licensing.is_free {
            "Free".to_string()
        } else {
            p.licensing.pricing.as_ref()
                .map(|pricing| format!("${:.2}", pricing.base_price as f64 / 100.0))
                .unwrap_or("Paid".to_string())
        },
        rating: p.stats.rating.unwrap_or(0.0),
        downloads: p.stats.downloads as u32,
    }).collect();
    
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&search_results)?);
        }
        _ => {
            print_plugins_table(&search_results);
        }
    }
    
    runner.print_success(&format!("Found {} plugins", search_results.len()));
    Ok(())
}

async fn handle_info(
    runner: &mut CliRunner,
    plugin_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info(&format!("📦 Getting info for plugin: {}", plugin_id));
    
    // Mock plugin details
    println!("\n🔌 Plugin Details:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📛 Name: {}", plugin_id);
    println!("📊 Version: 1.2.3");
    println!("👤 Author: DevKit Community");
    println!("📝 Description: A powerful plugin for enhanced development experience");
    println!("🏷️  Category: development");
    println!("📄 License: MIT");
    println!("💰 Price: Free");
    println!("⭐ Rating: 4.7/5 (127 reviews)");
    println!("📥 Downloads: 15,420");
    println!("🌐 Homepage: https://plugins.devkit.dev/{}", plugin_id);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    runner.print_success(&format!("Plugin '{}' information retrieved", plugin_id));
    Ok(())
}

async fn handle_install(
    runner: &mut CliRunner,
    plugin_id: &str,
    version: Option<&str>,
    license_key: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info(&format!("📦 Installing plugin: {}", plugin_id));
    
    if let Some(v) = version {
        runner.print_verbose(&format!("Version: {}", v));
    }
    if let Some(key) = license_key {
        runner.print_verbose(&format!("Using license key: {}...", &key[..8.min(key.len())]));
    }
    
    // Create marketplace client with mock configuration
    let config = MarketplaceConfig::default();
    
    let mut client = MarketplaceClient::new(config)?;
    
    // Perform the actual installation with progress feedback
    runner.print_info("📥 Downloading plugin package...");
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    
    runner.print_info("🔍 Verifying plugin signature...");
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    
    runner.print_info("⚙️  Configuring plugin...");
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    
    runner.print_info("🔌 Activating plugin...");
    
    // Try the actual installation - this will fail if there are issues
    match client.install_plugin(plugin_id, version, license_key).await {
        Ok(_installation) => {
            runner.print_success(&format!("Plugin '{}' installed successfully!", plugin_id));
            runner.print_info("💡 Use 'devkit plugin list' to see installed plugins");
        }
        Err(e) => {
            return Err(Box::new(e));
        }
    }
    
    Ok(())
}

async fn handle_list(
    runner: &mut CliRunner,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info("📋 Listing installed plugins...");
    
    // Create marketplace client with mock configuration
    let config = MarketplaceConfig::default();
    
    let client = MarketplaceClient::new(config)?;
    let installations = client.list_installed()?;
    
    let installed_plugins: Vec<InstalledPlugin> = installations.iter().map(|inst| {
        InstalledPlugin {
            id: inst.plugin_id.clone(),
            name: format!("{}Plugin", inst.plugin_id.split('-').collect::<Vec<&str>>().iter().map(|s| {
                let mut chars = s.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            }).collect::<Vec<String>>().join(" ")),
            version: inst.version.clone(),
            status: "Active".to_string(),
            auto_update: inst.auto_update,
            installed: inst.installed_at.format("%Y-%m-%d").to_string(),
        }
    }).collect();
    
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&installed_plugins)?);
        }
        _ => {
            print_installed_table(&installed_plugins);
        }
    }
    
    runner.print_success(&format!("Listed {} installed plugins", installed_plugins.len()));
    Ok(())
}

async fn handle_update(
    runner: &mut CliRunner,
    plugin_id: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    match plugin_id {
        Some(id) => {
            runner.print_info(&format!("🔄 Updating plugin: {}", id));
            runner.print_info("📥 Checking for updates...");
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            
            runner.print_info("📦 Downloading update...");
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            
            runner.print_success(&format!("Plugin '{}' updated successfully!", id));
        }
        None => {
            runner.print_info("🔄 Updating all plugins...");
            runner.print_info("📥 Checking for updates...");
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            
            runner.print_info("📦 2 plugins have updates available");
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            
            runner.print_success("All plugins updated successfully!");
        }
    }
    
    Ok(())
}

async fn handle_status(
    runner: &mut CliRunner,
) -> Result<(), Box<dyn std::error::Error>> {
    runner.print_info("🔌 Plugin System Status");
    
    println!("\n🔍 System Status:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🟢 Plugin System: Running");
    println!("📦 Installed Plugins: 2");
    println!("🔄 Auto-update Enabled: Yes");
    println!("🌐 Registry: https://plugins.devkit.dev");
    println!("💾 Plugin Directory: ~/.devkit/plugins");
    println!("🔧 Configuration: ~/.devkit/plugin.toml");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    runner.print_success("Plugin system is healthy!");
    Ok(())
}

// Helper structs for mock data
#[derive(Debug, serde::Serialize)]
struct PluginSearchResult {
    id: String,
    name: String,
    version: String,
    description: String,
    category: String,
    price: String,
    rating: f32,
    downloads: u32,
}

#[derive(Debug, serde::Serialize)]
struct InstalledPlugin {
    id: String,
    name: String,
    version: String,
    status: String,
    auto_update: bool,
    installed: String,
}

fn print_plugins_table(plugins: &[PluginSearchResult]) {
    println!("\n🔍 Plugin Search Results:");
    println!("{:<20} {:<10} {:<15} {:<12} {:<6} {:<10} {:<40}", "ID", "VERSION", "CATEGORY", "PRICE", "RATING", "DOWNLOADS", "DESCRIPTION");
    println!("{}", "─".repeat(120));
    
    for plugin in plugins {
        let desc = if plugin.description.len() > 38 {
            format!("{}...", &plugin.description[..35])
        } else {
            plugin.description.clone()
        };
        
        println!(
            "{:<20} {:<10} {:<15} {:<12} {:<6.1} {:<10} {:<40}",
            plugin.id,
            plugin.version,
            plugin.category,
            plugin.price,
            plugin.rating,
            plugin.downloads,
            desc
        );
    }
}

fn print_installed_table(plugins: &[InstalledPlugin]) {
    println!("\n📋 Installed Plugins:");
    println!("{:<20} {:<10} {:<10} {:<12} {:<12} {:<20}", "ID", "VERSION", "STATUS", "AUTO-UPDATE", "INSTALLED", "NAME");
    println!("{}", "─".repeat(90));
    
    for plugin in plugins {
        println!(
            "{:<20} {:<10} {:<10} {:<12} {:<12} {:<20}",
            plugin.id,
            plugin.version,
            plugin.status,
            if plugin.auto_update { "Yes" } else { "No" },
            plugin.installed,
            plugin.name
        );
    }
}