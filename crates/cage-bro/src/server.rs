use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;

use cage_bro_code::JupyterKernelManager;
use cage_bro_runtime::{LocalFilesystem, ProcessRuntime, SessionManager};

use crate::api;
use crate::browser::BrowserManager;
use crate::dashboard;

#[derive(Clone)]
pub struct AppState {
    pub runtime: Arc<ProcessRuntime>,
    pub filesystem: Arc<LocalFilesystem>,
    pub sessions: Arc<SessionManager>,
    pub browser: Arc<BrowserManager>,
    pub jupyter: Arc<JupyterKernelManager>,
}

pub async fn run(host: &str, port: u16) -> anyhow::Result<()> {
    let workspace = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("workspace");

    tokio::fs::create_dir_all(&workspace).await?;

    let state = AppState {
        runtime: Arc::new(ProcessRuntime::new()),
        filesystem: Arc::new(LocalFilesystem::new(&workspace)),
        sessions: Arc::new(SessionManager::new(workspace.to_string_lossy().to_string())),
        browser: Arc::new(BrowserManager::new()),
        jupyter: Arc::new(JupyterKernelManager::new()),
    };

    let api_routes = Router::new()
        .route("/health", get(health))
        .route("/v1/sandbox/info", get(api::sandbox::info))
        .nest("/v1/shell", api::shell::routes())
        .nest("/v1/file", api::file::routes())
        .nest("/v1/code", api::code::routes())
        .nest("/v1/browser", api::browser::routes())
        .with_state(state);

    // Dashboard: serve embedded static assets, SPA fallback to index.html
    let dashboard_routes = Router::new().fallback(|req: axum::extract::Request| async move {
        let path = req.uri().path();
        dashboard::serve_asset(path)
    });

    let app = api_routes.merge(dashboard_routes);

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
