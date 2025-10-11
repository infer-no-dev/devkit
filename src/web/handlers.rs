//! HTTP API handlers for the DevKit web dashboard

use super::{server::AppState, WebError};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Json as JsonRequest,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// System status response
#[derive(Debug, Serialize)]
pub struct SystemStatusResponse {
    pub status: String,
    pub version: String,
    pub uptime: u64,
    pub agents_active: usize,
    pub total_output_blocks: usize,
    pub pending_notifications: usize,
}

/// Agent status response
#[derive(Debug, Serialize)]
pub struct AgentResponse {
    pub name: String,
    pub status: String,
    pub current_task: Option<String>,
    pub progress: Option<f64>,
    pub last_update: u64,
}

/// Output block response
#[derive(Debug, Serialize, Clone)]
pub struct OutputBlockResponse {
    pub id: String,
    pub content: String,
    pub block_type: String,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

/// Notification response
#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub id: String,
    pub title: String,
    pub message: String,
    pub level: String,
    pub timestamp: u64,
}

/// Command execution request
#[derive(Debug, Deserialize)]
pub struct CommandRequest {
    pub command: String,
}

/// Query parameters for output blocks
#[derive(Debug, Deserialize)]
pub struct OutputQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub filter: Option<String>,
}

/// Get system status
pub async fn get_system_status(
    State(state): State<AppState>,
) -> Result<Json<SystemStatusResponse>, StatusCode> {
    let agent_status = state.agent_status.read().await;
    let output_blocks = state.output_blocks.read().await;
    let notifications = state.notifications.read().await;

    let response = SystemStatusResponse {
        status: "running".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: 0, // TODO: Track actual uptime
        agents_active: agent_status.len(),
        total_output_blocks: output_blocks.len(),
        pending_notifications: notifications.len(),
    };

    Ok(Json(response))
}

/// Get all agents
pub async fn get_agents(
    State(state): State<AppState>,
) -> Result<Json<Vec<AgentResponse>>, StatusCode> {
    let agent_status = state.agent_status.read().await;
    
    let agents: Vec<AgentResponse> = agent_status
        .iter()
        .map(|(name, status)| AgentResponse {
            name: name.clone(),
            status: format!("{:?}", status),
            current_task: None, // TODO: Track current tasks
            progress: None,     // TODO: Track progress
            last_update: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
        .collect();

    Ok(Json(agents))
}

/// Get specific agent status
pub async fn get_agent_status(
    State(state): State<AppState>,
    Path(agent_name): Path<String>,
) -> Result<Json<AgentResponse>, StatusCode> {
    let agent_status = state.agent_status.read().await;
    
    if let Some(status) = agent_status.get(&agent_name) {
        let response = AgentResponse {
            name: agent_name,
            status: format!("{:?}", status),
            current_task: None,
            progress: None,
            last_update: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        Ok(Json(response))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Get output blocks with optional filtering
pub async fn get_output_blocks(
    State(state): State<AppState>,
    Query(query): Query<OutputQuery>,
) -> Result<Json<Vec<OutputBlockResponse>>, StatusCode> {
    let output_blocks = state.output_blocks.read().await;
    
    let mut blocks: Vec<OutputBlockResponse> = output_blocks
        .iter()
        .map(|block| {
            OutputBlockResponse {
                id: block.id.clone(),
                content: block.content.clone(),
                block_type: format!("{:?}", block.block_type),
                timestamp: block.timestamp
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                metadata: block.metadata.clone(),
            }
        })
        .collect();

    // Apply filtering if specified
    if let Some(filter) = &query.filter {
        blocks.retain(|block| {
            block.content.contains(filter) || block.block_type.contains(filter)
        });
    }

    // Apply pagination
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(100).min(1000); // Cap at 1000 items

    let total_blocks = blocks.len();
    if offset >= total_blocks {
        blocks.clear();
    } else {
        let end = (offset + limit).min(total_blocks);
        blocks = blocks[offset..end].to_vec();
    }

    Ok(Json(blocks))
}

/// Clear all output blocks
pub async fn clear_output(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Send clear output event
    if let Err(_) = state.event_sender.send(crate::ui::UIEvent::ClearOutput) {
        tracing::warn!("Failed to send ClearOutput event - no subscribers");
    }

    // Clear the cached output blocks
    state.output_blocks.write().await.clear();

    Ok(Json(serde_json::json!({ "status": "cleared" })))
}

/// Get notifications
pub async fn get_notifications(
    State(state): State<AppState>,
) -> Result<Json<Vec<NotificationResponse>>, StatusCode> {
    let notifications = state.notifications.read().await;
    
    let response: Vec<NotificationResponse> = notifications
        .iter()
        .map(|notification| NotificationResponse {
            id: notification.id.clone(),
            title: notification.title.clone(),
            message: notification.message.clone(),
            level: format!("{:?}", notification.notification_type),
            timestamp: notification.timestamp
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
        .collect();

    Ok(Json(response))
}

/// Execute a command
pub async fn execute_command(
    State(state): State<AppState>,
    JsonRequest(request): JsonRequest<CommandRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Send the command to the main application
    if let Err(_) = state.command_sender.send(request.command.clone()) {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(serde_json::json!({
        "status": "executed",
        "command": request.command
    })))
}