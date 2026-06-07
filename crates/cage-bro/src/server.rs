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
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let workspace = cwd.join("workspace");

    tokio::fs::create_dir_all(&workspace).await?;

    // Hold an advisory lock for this working directory. It auto-releases when
    // the process exits, so it also tells us whether another cage-bro instance
    // is already running here. We only prune (a destructive op) when we hold the
    // lock — otherwise a second instance would clobber the first's live data.
    let _ = tokio::fs::create_dir_all(cwd.join(".cage-bro")).await;
    let instance_lock = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(cwd.join(".cage-bro/instance.lock"));
    let sole_instance = match &instance_lock {
        Ok(f) => f.try_lock().is_ok(),
        Err(_) => true, // couldn't create the lock; assume sole and proceed
    };

    if sole_instance {
        // The sandbox registry and snapshot index are in-memory, so any
        // workspace/snapshot dirs left by a prior process are unreachable after
        // a restart. Prune them to avoid an unbounded disk leak; absence
        // (NotFound) is the common case and not worth logging.
        for sub in [".cage-bro/e2b", ".cage-bro/restored", ".cage-bro/snapshots"] {
            let path = cwd.join(sub);
            if let Err(e) = tokio::fs::remove_dir_all(&path).await {
                if e.kind() != std::io::ErrorKind::NotFound {
                    tracing::error!(path = %path.display(), "failed to prune stale dir on startup: {}", e);
                }
            }
        }
    } else {
        tracing::warn!(
            dir = %cwd.display(),
            "another cage-bro instance appears active here; skipping startup prune to avoid clobbering its data"
        );
    }
    // Hold the lock for the lifetime of the server.
    let _instance_lock = instance_lock;

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
        // E2B-compatible lifecycle API, mounted at root for SDK drop-in.
        .merge(api::e2b::routes())
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
