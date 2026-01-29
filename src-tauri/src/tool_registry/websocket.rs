use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State as AxumState, WebSocketUpgrade};
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

#[derive(Clone)]
pub struct ControlPanelState {
    pub event_tx: broadcast::Sender<ControlPanelEvent>,
    pub connected_clients: Arc<Mutex<Vec<tokio::sync::mpsc::Sender<ControlPanelEvent>>>>,
}

impl ControlPanelState {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            event_tx,
            connected_clients: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn broadcast_event(&self,
        event: ControlPanelEvent,
    ) {
        let _ = self.event_tx.send(event);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlPanelEvent {
    ToolStatusChanged {
        tool_id: String,
        status: String,
        version: Option<String>,
    },
    ToolInstallationUpdated {
        tool_id: String,
        is_installed: bool,
        version: Option<String>,
    },
    WarningAdded {
        warning_id: String,
        severity: String,
        message: String,
    },
    WarningResolved {
        warning_id: String,
    },
    ValidationCompleted {
        tool_id: String,
        results: Vec<ValidationResultEvent>,
    },
    JobProgress {
        job_id: String,
        progress: i32,
        stage: String,
        message: String,
    },
    JobCompleted {
        job_id: String,
        status: String,
    },
    MCPServerUpdated {
        server_id: String,
        name: String,
        is_enabled: bool,
    },
    MCPSyncCompleted {
        server_id: String,
        tool_id: String,
        success: bool,
    },
    UsageUpdated {
        tool_id: String,
        stats: UsageStats,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationResultEvent {
    pub validation_type: String,
    pub status: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UsageStats {
    pub total_invocations: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum ClientMessage {
    #[serde(rename = "subscribe")]
    Subscribe { channels: Vec<String> },
    #[serde(rename = "unsubscribe")]
    Unsubscribe { channels: Vec<String> },
    #[serde(rename = "ping")]
    Ping,
}

#[derive(Debug, Serialize)]
pub struct ServerMessage {
    pub timestamp: String,
    pub event: ControlPanelEvent,
}

pub async fn control_panel_websocket(
    ws: WebSocketUpgrade,
    AxumState(state): AxumState<Arc<ControlPanelState>>,
) -> Response {
    ws.on_upgrade(move |socket| control_panel_websocket_handler(socket, state))
}

async fn control_panel_websocket_handler(socket: WebSocket, state: Arc<ControlPanelState>) {
    let (mut sender, mut receiver) = socket.split();

    let (client_tx, mut client_rx) = tokio::sync::mpsc::channel::<ControlPanelEvent>(100);

    {
        let mut clients = state.connected_clients.lock().await;
        clients.push(client_tx);
    }

    let mut event_rx = state.event_tx.subscribe();

    let forward_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                Ok(event) = event_rx.recv() => {
                    let msg = ServerMessage {
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        event,
                    };
                    let json = serde_json::to_string(&msg).unwrap_or_default();
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Some(event) = client_rx.recv() => {
                    let msg = ServerMessage {
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        event,
                    };
                    let json = serde_json::to_string(&msg).unwrap_or_default();
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                else => break,
            }
        }
    });

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            match serde_json::from_str::<ClientMessage>(&text) {
                Ok(ClientMessage::Ping) => {
                    let pong = serde_json::json!({
                        "type": "pong",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });
                    let _ = state.event_tx.send(ControlPanelEvent::ToolStatusChanged {
                        tool_id: "ping".to_string(),
                        status: "pong".to_string(),
                        version: None,
                    });
                }
                Ok(ClientMessage::Subscribe { channels }) => {
                    println!("Client subscribed to channels: {:?}", channels);
                }
                Ok(ClientMessage::Unsubscribe { channels }) => {
                    println!("Client unsubscribed from channels: {:?}", channels);
                }
                Err(e) => {
                    println!("Failed to parse client message: {}", e);
                }
            }
        } else if let Message::Close(_) = msg {
            break;
        }
    }

    forward_task.abort();

    {
        let mut clients = state.connected_clients.lock().await;
        clients.retain(|tx| !tx.is_closed());
    }
}
