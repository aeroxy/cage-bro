//! E2B-compatible sandbox **lifecycle** REST surface.
//!
//! Mounted at the server root so the E2B SDK's API client (`E2B_API_URL=...`)
//! can drive sandbox create/list/get/kill/timeout against cage-bro. These map
//! onto [`ProcessRuntime`] sandboxes, each backed by its own workspace dir.
//!
//! Scope note: this is the *orchestration* half of the E2B protocol. The
//! in-sandbox `envd` RPC layer (per-sandbox filesystem/process/pty over
//! Connect-RPC, reached via a per-sandbox hostname) is **not** implemented, so
//! the official SDK's in-sandbox calls won't round-trip yet. For executing code
//! today, use the cage-bro extension endpoint `POST /sandboxes/{id}/exec` or the
//! native `/v1/*` API. See README for the honest compatibility matrix.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use cage_bro_runtime::{ExecCommand, Sandbox, SandboxConfig, SandboxRuntime};

use crate::server::AppState;

/// Default per-sandbox memory ceiling (MB) when the request omits one.
const DEFAULT_MEMORY_MB: u64 = 512;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/sandboxes", post(create).get(list))
        .route("/sandboxes/{id}", get(get_one).delete(kill))
        .route("/sandboxes/{id}/timeout", post(set_timeout))
        // cage-bro extension: run a command inside a tracked sandbox.
        .route("/sandboxes/{id}/exec", post(exec))
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct CreateSandboxRequest {
    template_id: Option<String>,
    metadata: Option<std::collections::HashMap<String, String>>,
    /// Seconds the sandbox should stay alive (advisory; not yet auto-reaped).
    timeout: Option<u64>,
    memory_mb: Option<u64>,
}

/// Render an internal [`Sandbox`] as an E2B-shaped JSON object.
fn to_e2b(sandbox: &Sandbox, template_id: &str) -> Value {
    json!({
        "sandboxID": sandbox.id.to_string(),
        "templateID": template_id,
        "clientID": "cage-bro",
        "state": match sandbox.state {
            cage_bro_runtime::SandboxState::Running => "running",
            cage_bro_runtime::SandboxState::Paused => "paused",
            _ => "stopped",
        },
        "startedAt": sandbox.created_at,
        "memoryMB": sandbox.config.memory_limit_mb,
        "cpuCount": 1,
    })
}

async fn create(
    State(state): State<AppState>,
    body: Option<Json<CreateSandboxRequest>>,
) -> (StatusCode, Json<Value>) {
    let req = body.map(|Json(b)| b).unwrap_or_default();
    let template_id = req.template_id.clone().unwrap_or_else(|| "base".to_string());

    // Each E2B sandbox gets an isolated workspace dir.
    let id = Uuid::new_v4();
    let workspace = std::env::current_dir()
        .unwrap_or_else(|_| ".".into())
        .join(".cage-bro/e2b")
        .join(id.to_string());
    if let Err(e) = tokio::fs::create_dir_all(&workspace).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "code": 500, "message": format!("workspace create failed: {}", e) })),
        );
    }

    let config = SandboxConfig {
        memory_limit_mb: Some(req.memory_mb.unwrap_or(DEFAULT_MEMORY_MB)),
        workspace_dir: Some(workspace.to_string_lossy().to_string()),
        ..Default::default()
    };

    match state.runtime.create(config).await {
        Ok(sandbox) => {
            tracing::info!(sandbox_id = %sandbox.id, %template_id, "E2B sandbox created");
            (StatusCode::CREATED, Json(to_e2b(&sandbox, &template_id)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "code": 500, "message": format!("create failed: {}", e) })),
        ),
    }
}

async fn list(State(state): State<AppState>) -> Json<Value> {
    let items: Vec<Value> = state
        .runtime
        .list()
        .await
        .iter()
        .map(|s| to_e2b(s, "base"))
        .collect();
    Json(json!(items))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Value>) {
    match parse_id(&id) {
        Some(uuid) => match state.runtime.get(&uuid).await {
            Some(sandbox) => (StatusCode::OK, Json(to_e2b(&sandbox, "base"))),
            None => not_found(&id),
        },
        None => bad_id(&id),
    }
}

async fn kill(State(state): State<AppState>, Path(id): Path<String>) -> (StatusCode, Json<Value>) {
    match parse_id(&id) {
        Some(uuid) => match state.runtime.get(&uuid).await {
            Some(sandbox) => {
                let _ = state.runtime.destroy(&sandbox).await;
                (StatusCode::NO_CONTENT, Json(json!({})))
            }
            None => not_found(&id),
        },
        None => bad_id(&id),
    }
}

async fn set_timeout(Path(id): Path<String>) -> (StatusCode, Json<Value>) {
    // Acknowledged; cage-bro does not auto-reap sandboxes yet.
    tracing::debug!(sandbox_id = %id, "E2B set_timeout (no-op ack)");
    (StatusCode::NO_CONTENT, Json(json!({})))
}

#[derive(Deserialize)]
struct ExecRequest {
    command: String,
    timeout_ms: Option<u64>,
}

/// cage-bro extension: execute a shell command inside a tracked sandbox,
/// under the sandbox's isolation policy and workspace.
async fn exec(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ExecRequest>,
) -> (StatusCode, Json<Value>) {
    let uuid = match parse_id(&id) {
        Some(u) => u,
        None => return bad_id(&id),
    };
    let sandbox = match state.runtime.get(&uuid).await {
        Some(s) => s,
        None => return not_found(&id),
    };

    let parts = match shell_words::split(&req.command) {
        Ok(p) if !p.is_empty() => p,
        Ok(_) => return (StatusCode::BAD_REQUEST, Json(json!({ "message": "empty command" }))),
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "message": format!("parse error: {}", e) })),
            )
        }
    };

    let cmd = ExecCommand {
        program: parts[0].clone(),
        args: parts[1..].to_vec(),
        env: std::collections::HashMap::new(),
        working_dir: None,
        timeout_ms: req.timeout_ms,
    };

    match state.runtime.exec(&sandbox, cmd).await {
        Ok(r) => (
            StatusCode::OK,
            Json(json!({
                "exitCode": r.exit_code,
                "stdout": r.stdout,
                "stderr": r.stderr,
                "durationMs": r.duration_ms,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": format!("exec failed: {}", e) })),
        ),
    }
}

fn parse_id(id: &str) -> Option<Uuid> {
    Uuid::parse_str(id).ok()
}

fn not_found(id: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "code": 404, "message": format!("sandbox not found: {}", id) })),
    )
}

fn bad_id(id: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "code": 400, "message": format!("invalid sandbox id: {}", id) })),
    )
}
