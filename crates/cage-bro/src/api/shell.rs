use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};

use cage_bro_runtime::{ExecCommand, SandboxConfig, SandboxRuntime};

use crate::server::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/exec", post(exec))
        .route("/session", post(create_session))
        .route("/session/list", get(list_sessions))
        .route("/session/{id}/ws", get(ws_terminal))
        .route("/session/{id}/close", post(close_session))
}

#[derive(Deserialize)]
struct ExecRequest {
    command: String,
    timeout_ms: Option<u64>,
}

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    shell: Option<String>,
    cols: Option<u16>,
    rows: Option<u16>,
}

async fn exec(
    State(state): State<AppState>,
    Json(req): Json<ExecRequest>,
) -> Json<Value> {
    tracing::info!(command = %req.command, "shell exec");

    let parts = match shell_words::split(&req.command) {
        Ok(p) => p,
        Err(e) => {
            return Json(json!({
                "error": format!("Failed to parse command: {}", e),
            }));
        }
    };

    if parts.is_empty() {
        return Json(json!({ "error": "Empty command" }));
    }

    let config = SandboxConfig {
        workspace_dir: Some(
            std::env::current_dir()
                .unwrap_or_else(|_| ".".into())
                .join("workspace")
                .to_string_lossy()
                .to_string(),
        ),
        ..Default::default()
    };

    let sandbox = match state.runtime.create(config).await {
        Ok(s) => s,
        Err(e) => {
            return Json(json!({ "error": format!("Failed to create sandbox: {}", e) }));
        }
    };

    let cmd = ExecCommand {
        program: parts[0].clone(),
        args: parts[1..].to_vec(),
        env: std::collections::HashMap::new(),
        working_dir: None,
        timeout_ms: req.timeout_ms,
    };

    let result = state.runtime.exec(&sandbox, cmd).await;
    let _ = state.runtime.destroy(&sandbox).await;

    match result {
        Ok(exec_result) => Json(json!({
            "exit_code": exec_result.exit_code,
            "stdout": exec_result.stdout,
            "stderr": exec_result.stderr,
            "duration_ms": exec_result.duration_ms,
        })),
        Err(e) => Json(json!({ "error": format!("Execution failed: {}", e) })),
    }
}

pub async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> Json<Value> {
    let cols = req.cols.unwrap_or(80);
    let rows = req.rows.unwrap_or(24);

    match state
        .sessions
        .create_session(req.shell.as_deref(), cols, rows)
        .await
    {
        Ok(session_id) => Json(json!({
            "session_id": session_id,
            "status": "created",
            "ws_url": format!("/v1/shell/session/{}/ws", session_id),
        })),
        Err(e) => Json(json!({ "error": e })),
    }
}

pub async fn list_sessions(State(state): State<AppState>) -> Json<Value> {
    let sessions = state.sessions.list().await;
    Json(json!({ "sessions": sessions }))
}

async fn close_session(
    State(state): State<AppState>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> Json<Value> {
    match state.sessions.close(&session_id).await {
        Ok(()) => Json(json!({ "status": "closed" })),
        Err(e) => Json(json!({ "error": e })),
    }
}

pub async fn ws_terminal(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_terminal(socket, state, session_id))
}

async fn handle_ws_terminal(mut socket: WebSocket, state: AppState, session_id: String) {
    tracing::info!(session_id = %session_id, "WebSocket terminal connected");

    let mut rx = match state.sessions.subscribe(&session_id).await {
        Ok(rx) => rx,
        Err(e) => {
            let _ = socket
                .send(Message::Text(format!("ERROR: {}", e).into()))
                .await;
            return;
        }
    };

    let (mut sender, mut receiver) = socket.split();

    // Task: PTY output → WebSocket
    let send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(data) => {
                    if sender.send(Message::Binary(data.into())).await.is_err() {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("WebSocket lagged, dropped {} messages", n);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    // Task: WebSocket input → PTY
    let sessions = state.sessions.clone();
    let sid = session_id.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Binary(data)) => {
                    if let Err(e) = sessions.write_input(&sid, &data).await {
                        tracing::error!("PTY write error: {}", e);
                        break;
                    }
                }
                Ok(Message::Text(text)) => {
                    // Also accept text messages as input
                    if let Err(e) = sessions.write_input(&sid, text.as_bytes()).await {
                        tracing::error!("PTY write error: {}", e);
                        break;
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    tracing::info!(session_id = %session_id, "WebSocket terminal disconnected");
}
