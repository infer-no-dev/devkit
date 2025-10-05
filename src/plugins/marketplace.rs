//! Plugin Marketplace
//!
//! Handles discovery, installation, and management of plugins from various sources.
//! Supports both free and paid plugins with transparent pricing and licensing.

use crate::plugins::types::{PluginError, PluginMetadata};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use url::Url;

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
        // Implementation would query each registry in priority order
        // and aggregate results, handling auth where needed
        todo!("Implement marketplace search")
    }

    /// Get detailed information about a specific plugin
    pub async fn get_plugin(&self, plugin_id: &str) -> Result<MarketplacePlugin, PluginError> {
        todo!("Implement plugin lookup")
    }

    /// Install a plugin from the marketplace
    pub async fn install_plugin(
        &mut self,
        plugin_id: &str,
        version: Option<&str>,
        license_key: Option<&str>,
    ) -> Result<PluginInstallation, PluginError> {
        todo!("Implement plugin installation")
    }

    /// Uninstall a plugin
    pub async fn uninstall_plugin(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        todo!("Implement plugin uninstallation")
    }

    /// Update a plugin to the latest version
    pub async fn update_plugin(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        todo!("Implement plugin updates")
    }

    /// List installed plugins
    pub fn list_installed(&self) -> Vec<&PluginInstallation> {
        self.installations.values().collect()
    }

    /// Check for available updates
    pub async fn check_updates(&self) -> Result<Vec<PluginUpdateInfo>, PluginError> {
        todo!("Implement update checking")
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