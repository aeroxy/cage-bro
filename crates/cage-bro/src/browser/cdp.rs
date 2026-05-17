use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};

type PendingCalls = HashMap<u64, oneshot::Sender<Result<Value, String>>>;

pub struct CdpClient {
    ws_tx: Arc<Mutex<futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        tokio_tungstenite::tungstenite::Message,
    >>>,
    pending: Arc<Mutex<PendingCalls>>,
    next_id: AtomicU64,
}

#[derive(Serialize)]
struct CdpRequest {
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
}

#[derive(Deserialize)]
struct CdpResponse {
    id: Option<u64>,
    result: Option<Value>,
    error: Option<CdpError>,
}

#[derive(Deserialize)]
struct CdpError {
    message: String,
}

impl CdpClient {
    pub async fn connect(ws_url: &str) -> Result<Self, String> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(ws_url)
            .await
            .map_err(|e| format!("CDP WebSocket connect failed: {}", e))?;

        let (tx, mut rx) = ws_stream.split();
        let pending: Arc<Mutex<PendingCalls>> = Arc::new(Mutex::new(HashMap::new()));
        let pending_clone = pending.clone();

        // Reader task: match responses by id, ignore events
        tokio::spawn(async move {
            while let Some(msg) = rx.next().await {
                match msg {
                    Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                        if let Ok(resp) = serde_json::from_str::<CdpResponse>(&text) {
                            if let Some(id) = resp.id {
                                let mut pending = pending_clone.lock().await;
                                if let Some(tx) = pending.remove(&id) {
                                    let result = if let Some(err) = resp.error {
                                        Err(err.message)
                                    } else {
                                        Ok(resp.result.unwrap_or(Value::Null))
                                    };
                                    let _ = tx.send(result);
                                }
                            }
                            // Events (no id) are silently consumed
                        }
                    }
                    Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => break,
                    Err(_) => break,
                    _ => {}
                }
            }
        });

        Ok(Self {
            ws_tx: Arc::new(Mutex::new(tx)),
            pending,
            next_id: AtomicU64::new(1),
        })
    }

    pub async fn call(&self, method: &str, params: Option<Value>) -> Result<Value, String> {
        self.call_with_session(method, params, None).await
    }

    pub async fn call_with_session(
        &self,
        method: &str,
        params: Option<Value>,
        session_id: Option<&str>,
    ) -> Result<Value, String> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (resp_tx, resp_rx) = oneshot::channel();

        {
            let mut pending = self.pending.lock().await;
            pending.insert(id, resp_tx);
        }

        let req = CdpRequest {
            id,
            method: method.to_string(),
            params,
            session_id: session_id.map(|s| s.to_string()),
        };

        let msg = serde_json::to_string(&req).map_err(|e| format!("Serialize failed: {}", e))?;

        let mut tx = self.ws_tx.lock().await;
        tx.send(tokio_tungstenite::tungstenite::Message::Text(msg.into()))
            .await
            .map_err(|e| format!("WebSocket send failed: {}", e))?;

        // Timeout after 30s
        match tokio::time::timeout(std::time::Duration::from_secs(30), resp_rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err("Response channel closed".to_string()),
            Err(_) => {
                // Clean up pending entry
                self.pending.lock().await.remove(&id);
                Err("CDP call timed out".to_string())
            }
        }
    }

    pub async fn close(&self) -> Result<(), String> {
        let mut tx = self.ws_tx.lock().await;
        tx.send(tokio_tungstenite::tungstenite::Message::Close(None))
            .await
            .map_err(|e| format!("Close failed: {}", e))?;
        Ok(())
    }
}
