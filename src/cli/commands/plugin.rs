//! Plugin CLI commands
//!
//! Handles plugin marketplace operations: search, install, uninstall, list, update

use crate::plugins::{
    marketplace::{MarketplaceClient, PluginSearchQuery, SortOption},
    PluginManager, PluginSystemConfig,
};
use clap::{Args, Subcommand};
use serde_json;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct PluginArgs {
    #[command(subcommand)]
    pub command: PluginCommand,
}

#[derive(Debug, Subcommand)]
pub enum PluginCommand {
    /// Search for plugins in the marketplace
    Search {
        /// Search query
        query: Option<String>,
        
        /// Plugin category to filter by
        #[arg(long)]
        category: Option<String>,
        
        /// Tags to filter by (comma-separated)
        #[arg(long)]
        tags: Option<String>,
        
        /// Show only free plugins
        #[arg(long)]
        free_only: bool,
        
        /// Show only verified publisher plugins
        #[arg(long)]
        verified_only: bool,
        
        /// Minimum rating (1-5)
        #[arg(long)]
        min_rating: Option<f32>,
        
        /// Sort results by
        #[arg(long, value_enum)]
        sort: Option<PluginSortOption>,
        
        /// Limit number of results
        #[arg(long)]
        limit: Option<usize>,
        
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Show detailed information about a plugin
    Info {
        /// Plugin ID to show info for
        plugin_id: String,
        
        /// Output format
        #[arg(long, value_enum, default_value = "detailed")]
        format: OutputFormat,
    },

    /// Install a plugin from the marketplace
    Install {
        /// Plugin ID to install
        plugin_id: String,
        
        /// Specific version to install
        #[arg(long)]
        version: Option<String>,
        
        /// License key for paid plugins
        #[arg(long)]
        license_key: Option<String>,
        
        /// Enable automatic updates
        #[arg(long)]
        auto_update: bool,
        
        /// Force reinstall if already installed
        #[arg(long)]
        force: bool,
        
        /// Install from local file instead of marketplace
        #[arg(long)]
        local: Option<PathBuf>,
        
        /// Install from git repository
        #[arg(long)]
        git: Option<String>,
    },

    /// Uninstall a plugin
    Uninstall {
        /// Plugin ID to uninstall
        plugin_id: String,
        
        /// Remove all plugin data
        #[arg(long)]
        purge: bool,
        
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },

    /// List installed plugins
    List {
        /// Show only enabled plugins
        #[arg(long)]
        enabled_only: bool,
        
        /// Show only plugins with updates available
        #[arg(long)]
        updates_only: bool,
        
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Update plugins
    Update {
        /// Specific plugin to update (updates all if not specified)
        plugin_id: Option<String>,
        
        /// Update to specific version
        #[arg(long)]
        version: Option<String>,
        
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
        
        /// Check for updates only, don't install
        #[arg(long)]
        check_only: bool,
    },

    /// Enable or disable plugins
    Toggle {
        /// Plugin ID to toggle
        plugin_id: String,
        
        /// Enable the plugin
        #[arg(long, conflicts_with = "disable")]
        enable: bool,
        
        /// Disable the plugin
        #[arg(long, conflicts_with = "enable")]
        disable: bool,
    },

    /// Configure plugin settings
    Configure {
        /// Plugin ID to configure
        plugin_id: String,
        
        /// Configuration key to set
        #[arg(long)]
        set: Option<String>,
        
        /// Configuration value
        #[arg(long, requires = "set")]
        value: Option<String>,
        
        /// Show current configuration
        #[arg(long)]
        show: bool,
        
        /// Reset to default configuration
        #[arg(long)]
        reset: bool,
    },

    /// Manage plugin marketplace registries
    Registry {
        #[command(subcommand)]
        command: RegistryCommand,
    },

    /// Show plugin system status and diagnostics
    Status {
        /// Show detailed diagnostics
        #[arg(long)]
        detailed: bool,
        
        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
    },
}

#[derive(Debug, Subcommand)]
pub enum RegistryCommand {
    /// List configured registries
    List,
    
    /// Add a new registry
    Add {
        /// Registry name
        name: String,
        /// Registry URL
        url: String,
        /// Registry priority (lower = higher priority)
        #[arg(long, default_value = "10")]
        priority: u32,
    },
    
    /// Remove a registry
    Remove {
        /// Registry name to remove
        name: String,
    },
    
    /// Update registry information
    Update {
        /// Registry name to update
        name: String,
        /// New URL
        #[arg(long)]
        url: Option<String>,
        /// New priority
        #[arg(long)]
        priority: Option<u32>,
    },
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum PluginSortOption {
    Relevance,
    Downloads,
    Rating,
    Updated,
    Name,
    Price,
}

impl From<PluginSortOption> for SortOption {
    fn from(option: PluginSortOption) -> Self {
        match option {
            PluginSortOption::Relevance => SortOption::Relevance,
            PluginSortOption::Downloads => SortOption::Downloads,
            PluginSortOption::Rating => SortOption::Rating,
            PluginSortOption::Updated => SortOption::Updated,
            PluginSortOption::Name => SortOption::Name,
            PluginSortOption::Price => SortOption::Price,
        }
    }
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
    Detailed,
    Compact,
}

/// Execute plugin commands
pub async fn run(args: PluginArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        PluginCommand::Search {
            query,
            category,
            tags,
            free_only,
            verified_only,
            min_rating,
            sort,
            limit,
            format,
        } => {
            handle_search(SearchArgs {
                query,
                category,
                tags,
                free_only,
                verified_only,
                min_rating,
                sort,
                limit,
                format,
            }).await
        }
        
        PluginCommand::Info { plugin_id, format } => {
            handle_info(&plugin_id, format).await
        }
        
        PluginCommand::Install {
            plugin_id,
            version,
            license_key,
            auto_update,
            force,
            local,
            git,
        } => {
            handle_install(InstallArgs {
                plugin_id,
                version,
                license_key,
                auto_update,
                force,
                local,
                git,
            }).await
        }
        
        PluginCommand::Uninstall { plugin_id, purge, yes } => {
            handle_uninstall(&plugin_id, purge, yes).await
        }
        
        PluginCommand::List { enabled_only, updates_only, format } => {
            handle_list(enabled_only, updates_only, format).await
        }
        
        PluginCommand::Update { plugin_id, version, yes, check_only } => {
            handle_update(plugin_id.as_deref(), version.as_deref(), yes, check_only).await
        }
        
        PluginCommand::Toggle { plugin_id, enable, disable } => {
            handle_toggle(&plugin_id, enable, disable).await
        }
        
        PluginCommand::Configure { plugin_id, set, value, show, reset } => {
            handle_configure(&plugin_id, set.as_deref(), value.as_deref(), show, reset).await
        }
        
        PluginCommand::Registry { command } => {
            handle_registry(command).await
        }
        
        PluginCommand::Status { detailed, format } => {
            handle_status(detailed, format).await
        }
    }
}

struct SearchArgs {
    query: Option<String>,
    category: Option<String>,
    tags: Option<String>,
    free_only: bool,
    verified_only: bool,
    min_rating: Option<f32>,
    sort: Option<PluginSortOption>,
    limit: Option<usize>,
    format: OutputFormat,
}

struct InstallArgs {
    plugin_id: String,
    version: Option<String>,
    license_key: Option<String>,
    auto_update: bool,
    force: bool,
    local: Option<PathBuf>,
    git: Option<String>,
}

async fn handle_search(args: SearchArgs) -> Result<(), Box<dyn std::error::Error>> {
    let config = PluginSystemConfig::default();
    let client = MarketplaceClient::new(config.into())?;
    
    let search_query = PluginSearchQuery {
        query: args.query,
        category: args.category,
        tags: args.tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect()).unwrap_or_default(),
        free_only: args.free_only,
        verified_only: args.verified_only,
        min_rating: args.min_rating,
        sort_by: args.sort.map(Into::into).unwrap_or_default(),
        limit: args.limit,
        offset: None,
    };
    
    let results = client.search(search_query).await?;
    
    match args.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        OutputFormat::Table => {
            print_plugins_table(&results);
        }
        _ => {
            print_plugins_table(&results);
        }
    }
    
    Ok(())
}

async fn handle_info(plugin_id: &str, format: OutputFormat) -> Result<(), Box<dyn std::error::Error>> {
    let config = PluginSystemConfig::default();
    let client = MarketplaceClient::new(config.into())?;
    
    let plugin = client.get_plugin(plugin_id).await?;
    
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&plugin)?);
        }
        OutputFormat::Detailed => {
            print_plugin_details(&plugin);
        }
        _ => {
            print_plugin_details(&plugin);
        }
    }
    
    Ok(())
}

async fn handle_install(args: InstallArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("Installing plugin: {}", args.plugin_id);
    if let Some(version) = &args.version {
        println!("Version: {}", version);
    }
    if let Some(license_key) = &args.license_key {
        println!("Using license key: {}...", &license_key[..8.min(license_key.len())]);
    }
    
    // Implementation would use MarketplaceClient::install_plugin
    todo!("Implement plugin installation")
}

async fn handle_uninstall(plugin_id: &str, purge: bool, yes: bool) -> Result<(), Box<dyn std::error::Error>> {
    if !yes {
        print!("Are you sure you want to uninstall '{}'? [y/N] ", plugin_id);
        // Implementation would prompt for confirmation
    }
    
    println!("Uninstalling plugin: {}", plugin_id);
    if purge {
        println!("Purging all plugin data...");
    }
    
    todo!("Implement plugin uninstallation")
}

async fn handle_list(enabled_only: bool, updates_only: bool, format: OutputFormat) -> Result<(), Box<dyn std::error::Error>> {
    let config = PluginSystemConfig::default();
    let client = MarketplaceClient::new(config.into())?;
    
    let installed = client.list_installed();
    
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&installed)?);
        }
        OutputFormat::Table => {
            print_installations_table(&installed);
        }
        _ => {
            print_installations_table(&installed);
        }
    }
    
    Ok(())
}

async fn handle_update(plugin_id: Option<&str>, _version: Option<&str>, _yes: bool, check_only: bool) -> Result<(), Box<dyn std::error::Error>> {
    if check_only {
        println!("Checking for updates...");
        // Implementation would check for updates without installing
    } else {
        match plugin_id {
            Some(id) => println!("Updating plugin: {}", id),
            None => println!("Updating all plugins..."),
        }
    }
    
    todo!("Implement plugin updates")
}

async fn handle_toggle(plugin_id: &str, enable: bool, disable: bool) -> Result<(), Box<dyn std::error::Error>> {
    if enable {
        println!("Enabling plugin: {}", plugin_id);
    } else if disable {
        println!("Disabling plugin: {}", plugin_id);
    } else {
        println!("Please specify --enable or --disable");
        return Ok(());
    }
    
    todo!("Implement plugin enable/disable")
}

async fn handle_configure(plugin_id: &str, set: Option<&str>, value: Option<&str>, show: bool, reset: bool) -> Result<(), Box<dyn std::error::Error>> {
    if show {
        println!("Configuration for plugin: {}", plugin_id);
        // Show current configuration
    } else if reset {
        println!("Resetting configuration for plugin: {}", plugin_id);
        // Reset to defaults
    } else if let Some(key) = set {
        if let Some(val) = value {
            println!("Setting {}={} for plugin: {}", key, val, plugin_id);
            // Set configuration value
        }
    }
    
    todo!("Implement plugin configuration")
}

async fn handle_registry(command: RegistryCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        RegistryCommand::List => {
            println!("Configured plugin registries:");
            // List registries
        }
        RegistryCommand::Add { name, url, priority } => {
            println!("Adding registry '{}' at {} (priority: {})", name, url, priority);
            // Add registry
        }
        RegistryCommand::Remove { name } => {
            println!("Removing registry '{}'", name);
            // Remove registry
        }
        RegistryCommand::Update { name, url, priority } => {
            println!("Updating registry '{}'", name);
            if let Some(url) = url {
                println!("New URL: {}", url);
            }
            if let Some(priority) = priority {
                println!("New priority: {}", priority);
            }
        }
    }
    
    todo!("Implement registry management")
}

async fn handle_status(detailed: bool, format: OutputFormat) -> Result<(), Box<dyn std::error::Error>> {
    println!("Plugin System Status:");
    
    if detailed {
        println!("Detailed diagnostics...");
        // Show detailed system information
    }
    
    match format {
        OutputFormat::Json => {
            // Output as JSON
        }
        _ => {
            // Output as text
        }
    }
    
    todo!("Implement plugin system status")
}

// Helper functions for output formatting
fn print_plugins_table(plugins: &[crate::plugins::marketplace::MarketplacePlugin]) {
    println!("{:<20} {:<10} {:<15} {:<8} {:<30}", "NAME", "VERSION", "CATEGORY", "PRICE", "DESCRIPTION");
    println!("{}", "-".repeat(90));
    
    for plugin in plugins {
        let price = if plugin.licensing.is_free {
            "Free".to_string()
        } else {
            plugin.licensing.pricing.as_ref()
                .map(|p| format!("${:.2}", p.base_price as f64 / 100.0))
                .unwrap_or_else(|| "N/A".to_string())
        };
        
        println!(
            "{:<20} {:<10} {:<15} {:<8} {:<30}",
            plugin.metadata.name,
            plugin.metadata.version,
            plugin.marketplace_info.category,
            price,
            plugin.metadata.description.chars().take(30).collect::<String>()
        );
    }
}

fn print_plugin_details(plugin: &crate::plugins::marketplace::MarketplacePlugin) {
    println!("Plugin: {}", plugin.metadata.name);
    println!("Version: {}", plugin.metadata.version);
    println!("Author: {}", plugin.metadata.author);
    println!("Description: {}", plugin.metadata.description);
    println!("Category: {}", plugin.marketplace_info.category);
    println!("License: {}", plugin.licensing.license);
    
    if !plugin.licensing.is_free {
        if let Some(pricing) = &plugin.licensing.pricing {
            println!("Price: ${:.2} ({:?})", pricing.base_price as f64 / 100.0, pricing.model);
        }
    }
    
    if let Some(rating) = plugin.stats.rating {
        println!("Rating: {:.1}/5 ({} reviews)", rating, plugin.stats.rating_count);
    }
    
    println!("Downloads: {}", plugin.stats.downloads);
    
    if let Some(homepage) = &plugin.marketplace_info.homepage {
        println!("Homepage: {}", homepage);
    }
}

fn print_installations_table(installations: &[&crate::plugins::marketplace::PluginInstallation]) {
    println!("{:<20} {:<10} {:<12} {:<15} {:<10}", "NAME", "VERSION", "STATUS", "INSTALLED", "AUTO-UPDATE");
    println!("{}", "-".repeat(75));
    
    for installation in installations {
        println!(
            "{:<20} {:<10} {:<12} {:<15} {:<10}",
            installation.plugin_id,
            installation.version,
            "Installed", // Would check actual status
            installation.installed_at.format("%Y-%m-%d"),
            if installation.auto_update { "Yes" } else { "No" }
        );
    }
}