//! Plugin Marketplace
//!
//! Handles discovery, installation, and management of plugins from various sources.
//! Supports both free and paid plugins with transparent pricing and licensing.

use crate::plugins::types::{PluginError, PluginMetadata};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use url::Url;
use chrono::{DateTime, Utc};
use semver::Version;

/// Plugin marketplace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    /// Default marketplace URLs (in priority order)
    pub registries: Vec<MarketplaceRegistry>,
    /// Local plugin cache directory
    pub cache_dir: PathBuf,
    /// Authentication tokens for private registries
    pub auth_tokens: HashMap<String, String>,
    /// Enable automatic updates for free plugins
    pub auto_update_free: bool,
    /// Check for updates interval (hours)
    pub update_check_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceRegistry {
    pub name: String,
    pub url: Url,
    pub priority: u32,
    /// Whether this registry requires authentication
    pub requires_auth: bool,
    /// Whether this registry has paid plugins
    pub has_paid_plugins: bool,
}

impl Default for MarketplaceConfig {
    fn default() -> Self {
        Self {
            registries: vec![
                MarketplaceRegistry {
                    name: "official".to_string(),
                    url: Url::parse("https://plugins.devkit.dev/registry").unwrap(),
                    priority: 1,
                    requires_auth: false,
                    has_paid_plugins: true,
                },
                MarketplaceRegistry {
                    name: "community".to_string(), 
                    url: Url::parse("https://community.plugins.devkit.dev/registry").unwrap(),
                    priority: 2,
                    requires_auth: false,
                    has_paid_plugins: false,
                },
            ],
            cache_dir: PathBuf::from("~/.devkit/plugin-cache"),
            auth_tokens: HashMap::new(),
            auto_update_free: true,
            update_check_interval: 24,
        }
    }
}

/// Plugin listing from marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplacePlugin {
    /// Core plugin metadata
    pub metadata: PluginMetadata,
    /// Marketplace-specific information
    pub marketplace_info: MarketplaceInfo,
    /// Download information
    pub download: DownloadInfo,
    /// Licensing and pricing
    pub licensing: LicensingInfo,
    /// Statistics and ratings
    pub stats: PluginStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceInfo {
    /// Unique plugin ID in marketplace
    pub plugin_id: String,
    /// Plugin category (e.g., "code-generation", "analysis", "workflow")
    pub category: String,
    /// Tags for search and filtering
    pub tags: Vec<String>,
    /// Plugin homepage/repository URL
    pub homepage: Option<Url>,
    /// Documentation URL
    pub documentation: Option<Url>,
    /// Issue tracker URL
    pub issues: Option<Url>,
    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Publisher information
    pub publisher: PublisherInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherInfo {
    pub name: String,
    pub email: Option<String>,
    pub website: Option<Url>,
    /// Whether publisher is verified (signed, KYC'd, etc.)
    pub verified: bool,
    /// Publisher reputation score (0-100)
    pub reputation: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    /// Download URLs for different platforms
    pub urls: HashMap<String, Url>, // platform -> url
    /// File checksums for verification
    pub checksums: HashMap<String, String>, // platform -> sha256
    /// File sizes in bytes
    pub sizes: HashMap<String, u64>,
    /// Minimum DevKit version required
    pub min_devkit_version: String,
    /// Maximum DevKit version supported
    pub max_devkit_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensingInfo {
    /// License type (MIT, Apache-2.0, Commercial, etc.)
    pub license: String,
    /// Whether this is a free plugin
    pub is_free: bool,
    /// Pricing information (if paid)
    pub pricing: Option<PricingInfo>,
    /// Trial information (if applicable)
    pub trial: Option<TrialInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingInfo {
    /// Pricing model
    pub model: PricingModel,
    /// Base price in USD cents
    pub base_price: u64,
    /// Currency code
    pub currency: String,
    /// Billing period for subscriptions
    pub billing_period: Option<BillingPeriod>,
    /// Volume discounts or tiers
    pub tiers: Vec<PricingTier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PricingModel {
    OneTime,           // Single purchase
    Subscription,      // Recurring subscription
    PayPerUse,        // Usage-based pricing
    Freemium,         // Free with paid upgrades
    Enterprise,       // Contact for pricing
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BillingPeriod {
    Monthly,
    Quarterly, 
    Yearly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingTier {
    pub name: String,
    pub min_quantity: u32,
    pub max_quantity: Option<u32>,
    pub price_per_unit: u64, // in cents
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialInfo {
    /// Trial duration in days
    pub duration_days: u32,
    /// Features available during trial
    pub features: Vec<String>,
    /// Whether trial requires credit card
    pub requires_card: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStats {
    /// Download count
    pub downloads: u64,
    /// Active installations (if telemetry enabled)
    pub active_installs: Option<u64>,
    /// Average rating (1-5)
    pub rating: Option<f32>,
    /// Number of ratings
    pub rating_count: u32,
    /// Last 30 days downloads
    pub recent_downloads: u64,
}

/// Installation status for marketplace plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstallation {
    pub plugin_id: String,
    pub version: String,
    pub installed_at: chrono::DateTime<chrono::Utc>,
    pub installation_source: InstallationSource,
    pub license_info: Option<LicenseActivation>,
    pub auto_update: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallationSource {
    Marketplace { registry: String },
    LocalFile { path: PathBuf },
    Git { url: Url, branch: Option<String> },
    Custom { source: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseActivation {
    pub license_key: String,
    pub activated_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub features: Vec<String>,
    pub max_activations: Option<u32>,
    pub current_activations: u32,
}

/// Search and filtering options
#[derive(Debug, Clone, Default)]
pub struct PluginSearchQuery {
    pub query: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub free_only: bool,
    pub verified_only: bool,
    pub min_rating: Option<f32>,
    pub sort_by: SortOption,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum SortOption {
    Relevance,
    Downloads,
    Rating, 
    Updated,
    Name,
    Price,
}

impl Default for SortOption {
    fn default() -> Self {
        SortOption::Relevance
    }
}

/// Marketplace client for plugin operations
pub struct MarketplaceClient {
    config: MarketplaceConfig,
    http_client: reqwest::Client,
    installations: HashMap<String, PluginInstallation>,
}

impl MarketplaceClient {
    pub fn new(config: MarketplaceConfig) -> Result<Self, PluginError> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent(format!("devkit/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| PluginError::LoadFailed(e.to_string()))?;

        Ok(Self {
            config,
            http_client,
            installations: HashMap::new(),
        })
    }

    /// Search for plugins in the marketplace
    pub async fn search(&self, query: PluginSearchQuery) -> Result<Vec<MarketplacePlugin>, PluginError> {
        let db = self.load_plugin_database()?;
        let mut results = db.plugins;
        
        // Apply text search filter
        if let Some(search_term) = &query.query {
            let search_term = search_term.to_lowercase();
            results.retain(|plugin| {
                plugin.metadata.name.to_lowercase().contains(&search_term)
                    || plugin.metadata.description.to_lowercase().contains(&search_term)
                    || plugin.marketplace_info.tags.iter().any(|tag| tag.to_lowercase().contains(&search_term))
            });
        }
        
        // Apply category filter
        if let Some(category) = &query.category {
            results.retain(|plugin| plugin.marketplace_info.category == *category);
        }
        
        // Apply free-only filter
        if query.free_only {
            results.retain(|plugin| plugin.licensing.is_free);
        }
        
        // Apply verified-only filter
        if query.verified_only {
            results.retain(|plugin| plugin.marketplace_info.publisher.verified);
        }
        
        // Apply minimum rating filter
        if let Some(min_rating) = query.min_rating {
            results.retain(|plugin| {
                plugin.stats.rating.map(|r| r >= min_rating).unwrap_or(false)
            });
        }
        
        // Apply tag filters
        if !query.tags.is_empty() {
            results.retain(|plugin| {
                query.tags.iter().any(|tag| {
                    plugin.marketplace_info.tags.iter().any(|plugin_tag| {
                        plugin_tag.to_lowercase() == tag.to_lowercase()
                    })
                })
            });
        }
        
        // Sort results
        results.sort_by(|a, b| {
            use crate::plugins::marketplace::SortOption;
            match query.sort_by {
                SortOption::Relevance => {
                    // Simple relevance score based on downloads and rating
                    let score_a = (a.stats.downloads as f32) * a.stats.rating.unwrap_or(0.0);
                    let score_b = (b.stats.downloads as f32) * b.stats.rating.unwrap_or(0.0);
                    score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
                }
                SortOption::Downloads => b.stats.downloads.cmp(&a.stats.downloads),
                SortOption::Rating => {
                    let rating_a = a.stats.rating.unwrap_or(0.0);
                    let rating_b = b.stats.rating.unwrap_or(0.0);
                    rating_b.partial_cmp(&rating_a).unwrap_or(std::cmp::Ordering::Equal)
                }
                SortOption::Updated => b.marketplace_info.updated_at.cmp(&a.marketplace_info.updated_at),
                SortOption::Name => a.metadata.name.cmp(&b.metadata.name),
                SortOption::Price => {
                    let price_a = a.licensing.pricing.as_ref().map(|p| p.base_price).unwrap_or(0);
                    let price_b = b.licensing.pricing.as_ref().map(|p| p.base_price).unwrap_or(0);
                    price_a.cmp(&price_b)
                }
            }
        });
        
        // Apply pagination
        let start = query.offset.unwrap_or(0);
        let end = if let Some(limit) = query.limit {
            std::cmp::min(start + limit, results.len())
        } else {
            results.len()
        };
        
        if start >= results.len() {
            Ok(vec![])
        } else {
            Ok(results[start..end].to_vec())
        }
    }

    /// Get detailed information about a specific plugin
    pub async fn get_plugin(&self, plugin_id: &str) -> Result<MarketplacePlugin, PluginError> {
        let db = self.load_plugin_database()?;
        db.plugins
            .into_iter()
            .find(|p| p.marketplace_info.plugin_id == plugin_id)
            .ok_or_else(|| PluginError::LoadFailed(format!("Plugin not found: {}", plugin_id)))
    }

    /// Install a plugin from the marketplace
    pub async fn install_plugin(
        &mut self,
        plugin_id: &str,
        version: Option<&str>,
        license_key: Option<&str>,
    ) -> Result<PluginInstallation, PluginError> {
        // Load plugin metadata
        let plugin = self.get_plugin(plugin_id).await?;
        
        // Version selection
        let desired_version = version.unwrap_or(&plugin.metadata.version);
        
        // License handling (mock): if paid and no license key, error
        if !plugin.licensing.is_free && license_key.is_none() {
            return Err(PluginError::LoadFailed("License key required for paid plugin".into()));
        }
        
        // Determine platform download URL (mock: we don't actually download)
        let platform = self.get_current_platform();
        let _url = plugin
            .download
            .urls
            .get(&platform)
            .cloned()
            .ok_or_else(|| PluginError::LoadFailed(format!("No download available for platform {}", platform)))?;
        
        // Update installation DB
        let mut db = self.load_installation_database()?;
        let installation = PluginInstallation {
            plugin_id: plugin_id.to_string(),
            version: desired_version.to_string(),
            installed_at: Utc::now(),
            installation_source: InstallationSource::Marketplace { registry: "mock".into() },
            license_info: license_key.map(|k| LicenseActivation {
                license_key: k.to_string(),
                activated_at: Utc::now(),
                expires_at: None,
                features: vec![],
                max_activations: None,
                current_activations: 1,
            }),
            auto_update: true,
        };
        db.installations.insert(plugin_id.to_string(), installation.clone());
        self.save_installation_database(&db)?;
        
        Ok(installation)
    }

    /// Uninstall a plugin
    pub async fn uninstall_plugin(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        let mut db = self.load_installation_database()?;
        db.installations.remove(plugin_id);
        self.save_installation_database(&db)?;
        Ok(())
    }

    /// Update a plugin to the latest version
    pub async fn update_plugin(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        // Mock: simply set installed version to the latest in database
        let plugin = self.get_plugin(plugin_id).await?;
        let mut db = self.load_installation_database()?;
        if let Some(inst) = db.installations.get_mut(plugin_id) {
            // Compare versions; if newer, update
            if Version::parse(&plugin.metadata.version).unwrap_or_else(|_| Version::new(0,0,0))
                > Version::parse(&inst.version).unwrap_or_else(|_| Version::new(0,0,0))
            {
                inst.version = plugin.metadata.version.clone();
                inst.installed_at = Utc::now();
                self.save_installation_database(&db)?;
            }
        } else {
            return Err(PluginError::LoadFailed("Plugin not installed".into()));
        }
        Ok(())
    }

    /// List installed plugins
    pub fn list_installed(&self) -> Result<Vec<PluginInstallation>, PluginError> {
        let db = self.load_installation_database()?;
        Ok(db.installations.into_values().collect())
    }

    /// Check for available updates
    pub async fn check_updates(&self) -> Result<Vec<PluginUpdateInfo>, PluginError> {
        let db = self.load_installation_database()?;
        let plugin_db = self.load_plugin_database()?;
        
        let mut updates = Vec::new();
        for (plugin_id, inst) in db.installations.iter() {
            if let Some(plugin) = plugin_db.plugins.iter().find(|p| &p.marketplace_info.plugin_id == plugin_id) {
                let current = Version::parse(&inst.version).unwrap_or_else(|_| Version::new(0,0,0));
                let latest = Version::parse(&plugin.metadata.version).unwrap_or_else(|_| Version::new(0,0,0));
                let update_available = latest > current;
                updates.push(PluginUpdateInfo {
                    plugin_id: plugin_id.clone(),
                    current_version: inst.version.clone(),
                    latest_version: plugin.metadata.version.clone(),
                    update_available,
                    breaking_changes: false,
                    changelog: None,
                });
            }
        }
        Ok(updates)
    }

    fn get_mock_data_path(&self) -> PathBuf {
        // Use mock_marketplace directory for development
        PathBuf::from("mock_marketplace")
    }

    fn load_plugin_database(&self) -> Result<PluginDatabase, PluginError> {
        let db_path = self.get_mock_data_path().join("plugins_database.json");
        let content = fs::read_to_string(&db_path)
            .map_err(|e| PluginError::LoadFailed(format!("Failed to read plugin database: {}", e)))?;
        
        serde_json::from_str(&content)
            .map_err(|e| PluginError::LoadFailed(format!("Failed to parse plugin database: {}", e)))
    }

    fn load_installation_database(&self) -> Result<InstallationDatabase, PluginError> {
        let db_path = self.get_mock_data_path().join("installations.json");
        match fs::read_to_string(&db_path) {
            Ok(content) => {
                serde_json::from_str(&content)
                    .map_err(|e| PluginError::LoadFailed(format!("Failed to parse installation database: {}", e)))
            }
            Err(_) => {
                // Return empty database if file doesn't exist
                Ok(InstallationDatabase {
                    installations: HashMap::new(),
                    last_update_check: Utc::now(),
                })
            }
        }
    }

    fn save_installation_database(&self, db: &InstallationDatabase) -> Result<(), PluginError> {
        let db_path = self.get_mock_data_path().join("installations.json");
        
        // Create directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| PluginError::LoadFailed(format!("Failed to create directory: {}", e)))?;
        }
        
        let content = serde_json::to_string_pretty(db)
            .map_err(|e| PluginError::LoadFailed(format!("Failed to serialize installation database: {}", e)))?;
        
        fs::write(&db_path, content)
            .map_err(|e| PluginError::LoadFailed(format!("Failed to write installation database: {}", e)))
    }

    fn get_current_platform(&self) -> String {
        // Simple platform detection
        if cfg!(target_os = "linux") {
            "linux-x86_64".to_string()
        } else if cfg!(target_os = "macos") {
            "macos-x86_64".to_string()
        } else if cfg!(target_os = "windows") {
            "windows-x86_64".to_string()
        } else {
            "unknown".to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct PluginUpdateInfo {
    pub plugin_id: String,
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub breaking_changes: bool,
    pub changelog: Option<String>,
}

/// Internal database structure for mock implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PluginDatabase {
    pub plugins: Vec<MarketplacePlugin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstallationDatabase {
    pub installations: HashMap<String, PluginInstallation>,
    pub last_update_check: DateTime<Utc>,
}

