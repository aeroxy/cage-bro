use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::test]
async fn test_shell_exec_and_ws() {
    let workspace = std::env::temp_dir().join("cage-bro-test-ws");
    let _ = tokio::fs::create_dir_all(&workspace).await;

    let state = cage_bro::server::AppState {
        runtime: std::sync::Arc::new(cage_bro_runtime::ProcessRuntime::new()),
        filesystem: std::sync::Arc::new(cage_bro_runtime::LocalFilesystem::new(&workspace)),
        sessions: std::sync::Arc::new(cage_bro_runtime::SessionManager::new()),
    };

    let app = axum::Router::new()
        .route("/health", axum::routing::get(|| async { axum::Json(serde_json::json!({"status":"ok"})) }))
        .route(
            "/v1/shell/session",
            axum::routing::post(cage_bro::api::shell::create_session),
        )
        .route(
            "/v1/shell/session/list",
            axum::routing::get(cage_bro::api::shell::list_sessions),
        )
        .route(
            "/v1/shell/session/{id}/ws",
            axum::routing::get(cage_bro::api::shell::ws_terminal),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let base = format!("http://127.0.0.1:{}", port);
    let client = reqwest::Client::new();

    // Create session
    let resp = client
        .post(format!("{}/v1/shell/session", base))
        .json(&serde_json::json!({}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let ws_url = body["ws_url"].as_str().unwrap();

    // Connect via WebSocket
    let ws_uri = format!("ws://127.0.0.1:{}{}", port, ws_url);
    let (mut ws_stream, _) = connect_async(&ws_uri).await.unwrap();

    // Send a command
    ws_stream
        .send(Message::Text("echo hello-ws\n".into()))
        .await
        .unwrap();

    // Read output until we see our echo
    let mut found = false;
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);

    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(std::time::Duration::from_millis(500), ws_stream.next()).await {
            Ok(Some(Ok(Message::Binary(data)))) => {
                let text = String::from_utf8_lossy(&data);
                if text.contains("hello-ws") {
                    found = true;
                    break;
                }
            }
            _ => continue,
        }
    }

    assert!(found, "Should have received 'hello-ws' output via WebSocket");
    server_handle.abort();
    let _ = tokio::fs::remove_dir_all(&workspace).await;
}
