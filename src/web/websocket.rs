//! WebSocket handler for real-time DevKit updates

use super::{server::AppState, WebError};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json::json;
use tokio::select;

/// WebSocket connection handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

/// Handle WebSocket connection lifecycle
async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut event_receiver = state.event_sender.subscribe();

    tracing::info!("New WebSocket connection established");

    // Send initial state to the client
    if let Err(e) = send_initial_state(&mut sender, &state).await {
        tracing::error!("Failed to send initial state: {}", e);
        return;
    }

    // Handle bidirectional communication
    loop {
        select! {
            // Handle incoming messages from client
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(e) = handle_client_message(text, &state).await {
                            tracing::error!("Failed to handle client message: {}", e);
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        tracing::info!("WebSocket connection closed by client");
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if let Err(_) = sender.send(Message::Pong(data)).await {
                            tracing::error!("Failed to send pong response");
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        tracing::info!("WebSocket stream ended");
                        break;
                    }
                    _ => {
                        // Ignore other message types (Binary, Pong)
                    }
                }
            }

            // Handle UI events and broadcast to client
            ui_event = event_receiver.recv() => {
                match ui_event {
                    Ok(event) => {
                        if let Err(e) = send_ui_event(&mut sender, &event).await {
                            tracing::error!("Failed to send UI event: {}", e);
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(count)) => {
                        tracing::warn!("WebSocket client lagged by {} events", count);
                        // Continue processing - client will catch up
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tracing::info!("Event channel closed, ending WebSocket connection");
                        break;
                    }
                }
            }
        }
    }

    tracing::info!("WebSocket connection closed");
}

/// Send initial application state to newly connected client
async fn send_initial_state(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    state: &AppState,
) -> Result<(), WebError> {
    // Send system status
    let agent_status = state.agent_status.read().await;
    let output_blocks = state.output_blocks.read().await;
    let notifications = state.notifications.read().await;

    let initial_state = json!({
        "type": "initial_state",
        "data": {
            "agents": agent_status.len(),
            "output_blocks": output_blocks.len(),
            "notifications": notifications.len(),
            "version": env!("CARGO_PKG_VERSION")
        }
    });

    sender
        .send(Message::Text(initial_state.to_string()))
        .await
        .map_err(|e| WebError::WebSocketError(format!("Failed to send initial state: {}", e)))?;

    Ok(())
}

/// Handle incoming message from WebSocket client
async fn handle_client_message(
    message: String,
    state: &AppState,
) -> Result<(), WebError> {
    let parsed: serde_json::Value = serde_json::from_str(&message)
        .map_err(|e| WebError::WebSocketError(format!("Invalid JSON: {}", e)))?;

    match parsed.get("type").and_then(|v| v.as_str()) {
        Some("command") => {
            if let Some(command) = parsed.get("command").and_then(|v| v.as_str()) {
                if let Err(_) = state.command_sender.send(command.to_string()) {
                    return Err(WebError::WebSocketError(
                        "Failed to send command to application".to_string(),
                    ));
                }
            }
        }
        Some("ping") => {
            // Client ping - we'll respond with current timestamp in the UI event handler
            tracing::debug!("Received ping from WebSocket client");
        }
        _ => {
            tracing::warn!("Unknown WebSocket message type: {:?}", parsed);
        }
    }

    Ok(())
}

/// Send UI event to WebSocket client
async fn send_ui_event(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    event: &crate::ui::UIEvent,
) -> Result<(), WebError> {
    let event_data = match event {
        crate::ui::UIEvent::AgentStatusUpdate {
            agent_name,
            status,
            task,
            priority,
            progress,
        } => {
            json!({
                "type": "agent_status_update",
                "data": {
                    "agent_name": agent_name,
                    "status": format!("{:?}", status),
                    "task": task,
                    "priority": priority.as_ref().map(|p| format!("{:?}", p)),
                    "progress": progress
                }
            })
        }
        crate::ui::UIEvent::Output { content, block_type } => {
            json!({
                "type": "output",
                "data": {
                    "content": content,
                    "block_type": block_type,
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                }
            })
        }
        crate::ui::UIEvent::ClearOutput => {
            json!({
                "type": "clear_output",
                "data": {}
            })
        }
        crate::ui::UIEvent::Notification(notification) => {
            json!({
                "type": "notification",
                "data": {
                    "id": notification.id,
                    "title": notification.title,
                    "message": notification.message,
                    "notification_type": format!("{:?}", notification.notification_type),
                    "priority": format!("{:?}", notification.priority),
                    "timestamp": notification.timestamp
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                }
            })
        }
        _ => {
            // For other events, send a generic update
            json!({
                "type": "generic_update",
                "data": {
                    "event": format!("{:?}", event),
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                }
            })
        }
    };

    sender
        .send(Message::Text(event_data.to_string()))
        .await
        .map_err(|e| WebError::WebSocketError(format!("Failed to send event: {}", e)))?;

    Ok(())
}