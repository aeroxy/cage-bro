use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::server;

#[derive(Parser)]
#[command(
    name = "cage-bro",
    about = "A sandboxed execution environment for AI agents",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the cage-bro HTTP server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "0.0.0.0")]
        host: String,

        /// Runtime mode: process or microvm
        #[arg(long, default_value = "process")]
        runtime: String,
    },

    /// Start MCP server (for Claude Desktop, Cursor, etc.)
    Mcp {
        /// Use HTTP/SSE transport instead of stdio
        #[arg(long)]
        http: bool,

        /// Port for HTTP/SSE transport
        #[arg(long, default_value = "8081")]
        port: u16,
    },

    /// Download and install dependencies (obscura browser) into .cage-bro/bin/
    Setup,
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve {
            port,
            host,
            runtime,
        } => {
            tracing::info!(
                host = %host,
                port = port,
                runtime = %runtime,
                "Starting cage-bro"
            );
            server::run(&host, port).await?;
        }
        Commands::Mcp { http, port } => {
            if http {
                tracing::info!(port = port, "Starting MCP HTTP/SSE server");
                run_mcp_http(port).await?;
            } else {
                tracing::info!("Starting MCP stdio server");
                run_mcp_stdio().await?;
            }
        }
        Commands::Setup => {
            setup().await?;
        }
    }

    Ok(())
}

async fn run_mcp_stdio() -> Result<()> {
    let workspace = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("workspace");
    tokio::fs::create_dir_all(&workspace).await?;

    let state = crate::server::AppState {
        runtime: std::sync::Arc::new(cage_bro_runtime::ProcessRuntime::new()),
        filesystem: std::sync::Arc::new(cage_bro_runtime::LocalFilesystem::new(&workspace)),
        sessions: std::sync::Arc::new(cage_bro_runtime::SessionManager::new(workspace.to_string_lossy().to_string())),
        browser: std::sync::Arc::new(crate::browser::BrowserManager::new()),
        jupyter: std::sync::Arc::new(cage_bro_code::JupyterKernelManager::new()),
    };

    let mcp = crate::mcp::McpServer::new(state);
    mcp.run_stdio().await
}

async fn run_mcp_http(port: u16) -> Result<()> {
    let workspace = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("workspace");
    tokio::fs::create_dir_all(&workspace).await?;

    let state = crate::server::AppState {
        runtime: std::sync::Arc::new(cage_bro_runtime::ProcessRuntime::new()),
        filesystem: std::sync::Arc::new(cage_bro_runtime::LocalFilesystem::new(&workspace)),
        sessions: std::sync::Arc::new(cage_bro_runtime::SessionManager::new(workspace.to_string_lossy().to_string())),
        browser: std::sync::Arc::new(crate::browser::BrowserManager::new()),
        jupyter: std::sync::Arc::new(cage_bro_code::JupyterKernelManager::new()),
    };

    let mcp = std::sync::Arc::new(crate::mcp::McpServer::new(state));

    let mcp_clone = mcp.clone();
    let app = axum::Router::new()
        .route("/mcp", axum::routing::post(move |body: axum::extract::Json<serde_json::Value>| {
            let mcp = mcp_clone.clone();
            async move {
                let request = crate::mcp::server::JsonRpcRequest {
                    jsonrpc: "2.0".to_string(),
                    id: body.get("id").cloned(),
                    method: body.get("method").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    params: body.get("params").cloned(),
                };
                let response = mcp.handle_request(request).await;
                match response {
                    Some(resp) => axum::Json(serde_json::to_value(&resp).unwrap()),
                    None => axum::Json(serde_json::json!({"jsonrpc": "2.0"})),
                }
            }
        }));

    let addr: std::net::SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    tracing::info!("MCP HTTP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn setup() -> Result<()> {
    let bin_dir = std::env::current_dir()?.join(".cage-bro").join("bin");
    tokio::fs::create_dir_all(&bin_dir).await?;

    let (os, arch) = detect_platform();
    let ext = if os == "windows" { ".zip" } else { ".tar.gz" };
    let version = "v0.1.5";
    let url = format!(
        "https://github.com/h4ckf0r0day/obscura/releases/download/{}/obscura-{}-{}{}",
        version, arch, os, ext
    );

    let dest = bin_dir.join(if os == "windows" {
        "obscura.exe"
    } else {
        "obscura"
    });

    if dest.exists() {
        println!("obscura already installed at {}", dest.display());
        return Ok(());
    }

    println!("Downloading obscura {} for {}-{}...", version, arch, os);
    println!("  URL: {}", url);

    let resp = reqwest::get(&url).await?;
    if !resp.status().is_success() {
        anyhow::bail!(
            "Download failed ({}). Check https://github.com/h4ckf0r0day/obscura/releases",
            resp.status()
        );
    }

    let bytes = resp.bytes().await?;
    let archive_path = bin_dir.join(format!("obscura-archive{}", ext));
    tokio::fs::write(&archive_path, &bytes).await?;

    // Extract
    println!("Extracting...");
    extract_archive(&archive_path, &bin_dir, os).await?;

    // Make executable on unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&dest).await?.permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&dest, perms).await?;
    }

    // Cleanup archive
    let _ = tokio::fs::remove_file(&archive_path).await;

    println!("Installed obscura to {}", dest.display());
    println!("Run `cage-bro serve` to start the sandbox.");
    Ok(())
}

fn detect_platform() -> (&'static str, &'static str) {
    let os = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "windows"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x86_64"
    };

    (os, arch)
}

async fn extract_archive(
    archive: &std::path::Path,
    dest: &std::path::Path,
    os: &str,
) -> Result<()> {
    let archive_str = archive.to_string_lossy().to_string();
    let dest_str = dest.to_string_lossy().to_string();

    if os == "windows" {
        // zip
        let status = tokio::process::Command::new("tar")
            .args(["xf", &archive_str, "-C", &dest_str])
            .status()
            .await?;
        if !status.success() {
            anyhow::bail!("Extraction failed");
        }
    } else {
        // tar.gz
        let status = tokio::process::Command::new("tar")
            .args(["xzf", &archive_str, "-C", &dest_str])
            .status()
            .await?;
        if !status.success() {
            anyhow::bail!("Extraction failed");
        }
    }

    Ok(())
}
