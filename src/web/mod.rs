//! Web dashboard module for DevKit
//! 
//! Provides a web-based interface for monitoring and controlling DevKit agents,
//! viewing output, and managing the development workflow through a browser.

pub mod server;
pub mod handlers;
pub mod websocket;

pub use server::WebServer;
pub use crate::config::WebConfig;

use crate::ui::UIEvent;
use tokio::sync::{broadcast, mpsc};

/// Type alias for web event broadcasting
pub type WebEventSender = broadcast::Sender<UIEvent>;
pub type WebEventReceiver = broadcast::Receiver<UIEvent>;

/// Type alias for command sending from web to core
pub type WebCommandSender = mpsc::UnboundedSender<String>;

/// Web server configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DashboardConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub cors_enabled: bool,
    pub static_files_path: Option<String>,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: "127.0.0.1".to_string(),
            port: 8080,
            cors_enabled: true,
            static_files_path: None,
        }
    }
}

/// Error types for web operations
#[derive(Debug, thiserror::Error)]
pub enum WebError {
    #[error("Server startup failed: {0}")]
    StartupFailed(String),
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
    #[error("API error: {0}")]
    ApiError(String),
}