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
    response::{IntoResponse, Response},
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
    // E2B uses uppercase acronyms (templateID, not templateId), so override the
    // camelCase rule for these to match the SDK's wire format.
    #[serde(rename = "templateID")]
    template_id: Option<String>,
    metadata: Option<std::collections::HashMap<String, String>>,
    /// Seconds the sandbox should stay alive (advisory; not yet auto-reaped).
    timeout: Option<u64>,
    #[serde(rename = "memoryMB")]
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
    let workspace = e2b_workspace_dir(&id);
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
        Err(e) => {
            // Don't leak the workspace dir we just created if the runtime fails.
            let _ = tokio::fs::remove_dir_all(&workspace).await;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "code": 500, "message": format!("create failed: {}", e) })),
            )
        }
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

async fn kill(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    match parse_id(&id) {
        Some(uuid) => match state.runtime.get(&uuid).await {
            Some(sandbox) => match state.runtime.destroy(&sandbox).await {
                Ok(()) => {
                    // Delete the sandbox's actual workspace dir, but only if it
                    // lives under the e2b-managed base — so a registry sandbox
                    // carrying a foreign/shared workspace path can never trigger
                    // deletion outside `.cage-bro/e2b`. (We can't derive the dir
                    // from `uuid`: the runtime mints its own sandbox id, distinct
                    // from the throwaway id `create` used to name the dir.)
                    if let Some(ref ws) = sandbox.config.workspace_dir {
                        // Canonicalize both sides before the containment check so
                        // it can't be defeated by `..` or symlinks in the stored
                        // path — `remove_dir_all` must never escape the e2b base.
                        if let (Ok(ws_real), Ok(base_real)) = (
                            tokio::fs::canonicalize(ws).await,
                            tokio::fs::canonicalize(e2b_base_dir()).await,
                        ) {
                            // Strict subdirectory: `starts_with` is true for an
                            // equal path, which would delete the whole base.
                            if ws_real != base_real && ws_real.starts_with(&base_real) {
                                let _ = tokio::fs::remove_dir_all(&ws_real).await;
                            }
                        }
                    }
                    // 204 must not carry a body.
                    StatusCode::NO_CONTENT.into_response()
                }
                Err(e) => {
                    tracing::error!(sandbox_id = %id, "destroy failed: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "code": 500, "message": format!("destroy failed: {}", e) })),
                    )
                        .into_response()
                }
            },
            None => not_found(&id).into_response(),
        },
        None => bad_id(&id).into_response(),
    }
}

async fn set_timeout(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let uuid = match parse_id(&id) {
        Some(u) => u,
        None => return bad_id(&id).into_response(),
    };
    if state.runtime.get(&uuid).await.is_none() {
        return not_found(&id).into_response();
    }
    // Acknowledged; cage-bro does not auto-reap sandboxes yet.
    tracing::debug!(sandbox_id = %id, "E2B set_timeout (no-op ack)");
    // 204 must not carry a body.
    StatusCode::NO_CONTENT.into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
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

    if req.command.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "message": "empty command" })));
    }

    // Run via a shell so pipes / redirects / env-expansion work, matching E2B's
    // `commands.run` semantics. The whole shell tree runs under the sandbox's
    // isolation policy (Landlock + rlimits + process group).
    let cmd = ExecCommand {
        program: "sh".to_string(),
        args: vec!["-c".to_string(), req.command.clone()],
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

/// Base directory under which every E2B sandbox workspace lives. `kill` only
/// deletes paths under this prefix, bounding `remove_dir_all` to cage-bro's own
/// directories.
fn e2b_base_dir() -> std::path::PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| ".".into())
        .join(".cage-bro/e2b")
}

/// The workspace directory `create` allocates for a new E2B sandbox.
fn e2b_workspace_dir(id: &Uuid) -> std::path::PathBuf {
    e2b_base_dir().join(id.to_string())
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
