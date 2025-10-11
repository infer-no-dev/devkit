//! Web server implementation using Axum framework

use super::{handlers, websocket, DashboardConfig, WebError};
use crate::ui::UIEvent;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};

/// Web server configuration for Axum
pub type WebConfig = DashboardConfig;

/// Shared application state for web handlers
#[derive(Clone)]
pub struct AppState {
    pub event_sender: broadcast::Sender<UIEvent>,
    pub command_sender: mpsc::UnboundedSender<String>,
    pub agent_status: Arc<tokio::sync::RwLock<std::collections::HashMap<String, crate::agents::AgentStatus>>>,
    pub output_blocks: Arc<tokio::sync::RwLock<Vec<crate::ui::blocks::OutputBlock>>>,
    pub notifications: Arc<tokio::sync::RwLock<Vec<crate::ui::notifications::Notification>>>,
}

impl AppState {
    pub fn new(
        event_sender: broadcast::Sender<UIEvent>,
        command_sender: mpsc::UnboundedSender<String>,
    ) -> Self {
        Self {
            event_sender,
            command_sender,
            agent_status: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            output_blocks: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            notifications: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

/// Web server for DevKit dashboard
pub struct WebServer {
    config: WebConfig,
    app_state: AppState,
}

impl WebServer {
    /// Create a new web server instance
    pub fn new(
        config: WebConfig,
        event_sender: broadcast::Sender<UIEvent>,
        command_sender: mpsc::UnboundedSender<String>,
    ) -> Self {
        let app_state = AppState::new(event_sender, command_sender);
        Self { config, app_state }
    }

    /// Start the web server
    pub async fn start(self) -> Result<(), WebError> {
        let addr = format!("{}:{}", self.config.host, self.config.port)
            .parse::<SocketAddr>()
            .map_err(|e| WebError::StartupFailed(format!("Invalid address: {}", e)))?;

        let app = self.create_router();

        tracing::info!("Starting DevKit web dashboard on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| WebError::StartupFailed(format!("Failed to bind to {}: {}", addr, e)))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| WebError::StartupFailed(format!("Server error: {}", e)))?;

        Ok(())
    }

    /// Create the Axum router with all routes and middleware
    fn create_router(self) -> Router {
        let mut app = Router::new()
            // Dashboard routes
            .route("/", get(dashboard_handler))
            .route("/dashboard", get(dashboard_handler))
            
            // API routes
            .route("/api/status", get(handlers::get_system_status))
            .route("/api/agents", get(handlers::get_agents))
            .route("/api/agents/:name/status", get(handlers::get_agent_status))
            .route("/api/output", get(handlers::get_output_blocks))
            .route("/api/output/clear", post(handlers::clear_output))
            .route("/api/notifications", get(handlers::get_notifications))
            .route("/api/command", post(handlers::execute_command))
            
            // WebSocket route for real-time updates
            .route("/ws", get(websocket::websocket_handler))
            
            // Health check
            .route("/health", get(health_check))
            
            .with_state(self.app_state);

        // Add CORS middleware if enabled
        if self.config.cors_enabled {
            app = app.layer(
                ServiceBuilder::new().layer(
                    CorsLayer::permissive()
                )
            );
        }

        // Add static file serving if path is configured
        if let Some(static_path) = &self.config.static_files_path {
            app = app.nest_service("/static", ServeDir::new(static_path));
        }

        app
    }
}

/// Dashboard home page handler
async fn dashboard_handler() -> impl IntoResponse {
    Html(include_str!("../../web/dashboard.html"))
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "DevKit Web Dashboard OK")
}