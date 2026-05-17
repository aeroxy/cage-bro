use axum::{extract::State, routing::post, Json, Router};
use cage_bro_runtime::SandboxRuntime;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::server::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/python", post(python))
        .route("/node", post(node))
        .route("/jupyter/start", post(jupyter_start))
        .route("/jupyter/execute", post(jupyter_execute))
        .route("/jupyter/interrupt", post(jupyter_interrupt))
        .route("/jupyter/shutdown", post(jupyter_shutdown))
        .route("/jupyter/list", post(jupyter_list))
}

#[derive(Deserialize)]
struct CodeRequest {
    code: String,
    timeout_ms: Option<u64>,
}

#[derive(Deserialize)]
struct JupyterStartRequest {
    language: Option<String>,
}

#[derive(Deserialize)]
struct JupyterExecRequest {
    kernel_id: String,
    code: String,
}

#[derive(Deserialize)]
struct JupyterKernelRequest {
    kernel_id: String,
}

async fn python(
    State(state): State<AppState>,
    Json(req): Json<CodeRequest>,
) -> Json<Value> {
    tracing::info!("python exec ({} chars)", req.code.len());

    let temp_file = format!("/tmp/cage_bro_{}.py", uuid::Uuid::new_v4());
    if let Err(e) = tokio::fs::write(&temp_file, &req.code).await {
        return Json(json!({ "error": format!("Failed to write temp file: {}", e) }));
    }

    let config = cage_bro_runtime::SandboxConfig::default();
    let sandbox = match state.runtime.create(config).await {
        Ok(s) => s,
        Err(e) => {
            let _ = tokio::fs::remove_file(&temp_file).await;
            return Json(json!({ "error": format!("Failed to create sandbox: {}", e) }));
        }
    };

    let cmd = cage_bro_runtime::ExecCommand {
        program: "python3".to_string(),
        args: vec![temp_file.clone()],
        env: std::collections::HashMap::new(),
        working_dir: None,
        timeout_ms: req.timeout_ms,
    };

    let result = state.runtime.exec(&sandbox, cmd).await;
    let _ = state.runtime.destroy(&sandbox).await;
    let _ = tokio::fs::remove_file(&temp_file).await;

    match result {
        Ok(exec_result) => Json(json!({
            "stdout": exec_result.stdout,
            "stderr": exec_result.stderr,
            "exit_code": exec_result.exit_code,
            "duration_ms": exec_result.duration_ms,
        })),
        Err(e) => Json(json!({ "error": format!("Execution failed: {}", e) })),
    }
}

async fn node(
    State(state): State<AppState>,
    Json(req): Json<CodeRequest>,
) -> Json<Value> {
    tracing::info!("node exec ({} chars)", req.code.len());

    let temp_file = format!("/tmp/cage_bro_{}.js", uuid::Uuid::new_v4());
    if let Err(e) = tokio::fs::write(&temp_file, &req.code).await {
        return Json(json!({ "error": format!("Failed to write temp file: {}", e) }));
    }

    let config = cage_bro_runtime::SandboxConfig::default();
    let sandbox = match state.runtime.create(config).await {
        Ok(s) => s,
        Err(e) => {
            let _ = tokio::fs::remove_file(&temp_file).await;
            return Json(json!({ "error": format!("Failed to create sandbox: {}", e) }));
        }
    };

    let cmd = cage_bro_runtime::ExecCommand {
        program: "node".to_string(),
        args: vec![temp_file.clone()],
        env: std::collections::HashMap::new(),
        working_dir: None,
        timeout_ms: req.timeout_ms,
    };

    let result = state.runtime.exec(&sandbox, cmd).await;
    let _ = state.runtime.destroy(&sandbox).await;
    let _ = tokio::fs::remove_file(&temp_file).await;

    match result {
        Ok(exec_result) => Json(json!({
            "stdout": exec_result.stdout,
            "stderr": exec_result.stderr,
            "exit_code": exec_result.exit_code,
            "duration_ms": exec_result.duration_ms,
        })),
        Err(e) => Json(json!({ "error": format!("Execution failed: {}", e) })),
    }
}

async fn jupyter_start(
    State(state): State<AppState>,
    Json(req): Json<JupyterStartRequest>,
) -> Json<Value> {
    let language = req.language.unwrap_or_else(|| "python".to_string());
    tracing::info!(language = %language, "jupyter kernel start");

    match state.jupyter.start_kernel(&language).await {
        Ok(kernel_id) => Json(json!({
            "kernel_id": kernel_id,
            "language": language,
            "status": "ready",
        })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

async fn jupyter_execute(
    State(state): State<AppState>,
    Json(req): Json<JupyterExecRequest>,
) -> Json<Value> {
    tracing::info!(kernel_id = %req.kernel_id, code_len = req.code.len(), "jupyter execute");

    match state.jupyter.execute(&req.kernel_id, &req.code).await {
        Ok(result) => Json(json!({
            "stdout": result.stdout,
            "stderr": result.stderr,
            "exit_code": result.exit_code,
            "duration_ms": result.duration_ms,
        })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

async fn jupyter_interrupt(
    State(state): State<AppState>,
    Json(req): Json<JupyterKernelRequest>,
) -> Json<Value> {
    match state.jupyter.interrupt(&req.kernel_id).await {
        Ok(()) => Json(json!({ "status": "interrupted" })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

async fn jupyter_shutdown(
    State(state): State<AppState>,
    Json(req): Json<JupyterKernelRequest>,
) -> Json<Value> {
    match state.jupyter.shutdown(&req.kernel_id).await {
        Ok(()) => Json(json!({ "status": "shutdown" })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

async fn jupyter_list(State(state): State<AppState>) -> Json<Value> {
    let kernels = state.jupyter.list().await;
    let items: Vec<Value> = kernels
        .iter()
        .map(|k| {
            json!({
                "kernel_id": k.id,
                "language": format!("{:?}", k.language),
                "status": format!("{:?}", k.status),
            })
        })
        .collect();
    Json(json!({ "kernels": items }))
}
