use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::cdp::CdpClient;

fn obscura_bin() -> String {
    let local = dirs()
        .join("bin")
        .join(if cfg!(target_os = "windows") {
            "obscura.exe"
        } else {
            "obscura"
        });
    if local.exists() {
        tracing::info!(path = %local.display(), "Using local obscura binary");
        return local.to_string_lossy().to_string();
    }
    tracing::warn!("Local obscura not found at {:?}, falling back to PATH", local);
    "obscura".to_string()
}

fn dirs() -> std::path::PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(".cage-bro")
}

pub struct BrowserManager {
    inner: Arc<Mutex<BrowserInner>>,
}

struct BrowserInner {
    process: Option<tokio::process::Child>,
    cdp: Option<CdpClient>,
    session_id: Option<String>,
    port: u16,
    stealth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageContent {
    pub url: String,
    pub title: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotData {
    pub data: String,
    pub format: String,
    pub width: u32,
    pub height: u32,
}

impl BrowserManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(BrowserInner {
                process: None,
                cdp: None,
                session_id: None,
                port: 9333,
                stealth: false,
            })),
        }
    }

    pub async fn launch(&self, port: Option<u16>, stealth: bool) -> Result<String, String> {
        let mut inner = self.inner.lock().await;

        if inner.process.is_some() {
            return Err("Browser already running".to_string());
        }

        let port = port.unwrap_or(9333);
        inner.port = port;
        inner.stealth = stealth;

        let mut cmd = tokio::process::Command::new(obscura_bin());
        cmd.arg("serve")
            .arg("--port")
            .arg(port.to_string())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        if stealth {
            cmd.arg("--stealth");
        }

        let child = cmd
            .spawn()
            .map_err(|e| format!("Failed to launch obscura: {}. Run `cage-bro setup`.", e))?;

        inner.process = Some(child);

        // Wait for CDP server
        let browser_ws_url = format!("ws://127.0.0.1:{}/devtools/browser", port);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let mut retries = 15;
        let cdp = loop {
            match CdpClient::connect(&browser_ws_url).await {
                Ok(client) => break client,
                Err(_) if retries > 0 => {
                    retries -= 1;
                    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                }
                Err(e) => {
                    inner.process = None;
                    return Err(format!("CDP connect failed: {}", e));
                }
            }
        };

        // Create a new page target
        let create_result = cdp
            .call("Target.createTarget", Some(json!({"url": "about:blank"})))
            .await?;
        let target_id = create_result["targetId"]
            .as_str()
            .ok_or_else(|| format!("No targetId: {}", create_result))?
            .to_string();

        // Attach to the target to get a session
        let attach_result = cdp
            .call(
                "Target.attachToTarget",
                Some(json!({"targetId": target_id, "flatten": true})),
            )
            .await?;
        let session_id = attach_result["sessionId"]
            .as_str()
            .ok_or_else(|| format!("No sessionId: {}", attach_result))?
            .to_string();

        inner.cdp = Some(cdp);
        inner.session_id = Some(session_id);

        tracing::info!(port = port, stealth = stealth, target_id = %target_id, "Obscura browser launched");
        Ok(format!("Browser running on port {}", port))
    }

    pub async fn navigate(&self, url: &str) -> Result<PageContent, String> {
        let inner = self.inner.lock().await;
        let cdp = inner.cdp.as_ref().ok_or("Browser not launched")?;
        let sid = inner.session_id.as_deref().ok_or("No session")?;

        cdp.call_with_session("Page.navigate", Some(json!({"url": url})), Some(sid))
            .await?;

        // Wait for navigation
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

        self.eval_page_content(cdp, sid).await
    }

    pub async fn screenshot(&self, quality: Option<u32>) -> Result<ScreenshotData, String> {
        let inner = self.inner.lock().await;
        let cdp = inner.cdp.as_ref().ok_or("Browser not launched")?;
        let sid = inner.session_id.as_deref().ok_or("No session")?;

        let mut params = json!({"format": "png"});
        if let Some(q) = quality {
            params["quality"] = json!(q);
        }

        let result = cdp
            .call_with_session("Page.captureScreenshot", Some(params), Some(sid))
            .await?;

        let data = result["data"].as_str().ok_or("No screenshot data")?.to_string();

        Ok(ScreenshotData {
            data,
            format: "png".to_string(),
            width: 1280,
            height: 720,
        })
    }

    pub async fn execute_js(&self, expression: &str) -> Result<Value, String> {
        let inner = self.inner.lock().await;
        let cdp = inner.cdp.as_ref().ok_or("Browser not launched")?;
        let sid = inner.session_id.as_deref().ok_or("No session")?;

        let result = cdp
            .call_with_session(
                "Runtime.evaluate",
                Some(json!({
                    "expression": expression,
                    "returnByValue": true,
                    "awaitPromise": true
                })),
                Some(sid),
            )
            .await?;

        if let Some(err) = result["exceptionDetails"].as_object() {
            let msg = err["text"].as_str().unwrap_or("JS error");
            return Err(msg.to_string());
        }

        Ok(result["result"]["value"].clone())
    }

    pub async fn click(&self, selector: &str) -> Result<(), String> {
        let expression = format!(
            "(() => {{ const el = document.querySelector('{}'); if (!el) throw new Error('Not found'); el.click(); return true; }})()",
            selector.replace('\'', "\\'")
        );
        self.execute_js(&expression).await?;
        Ok(())
    }

    pub async fn type_text(&self, selector: &str, text: &str) -> Result<(), String> {
        let expression = format!(
            "(() => {{ const el = document.querySelector('{}'); if (!el) throw new Error('Not found'); el.focus(); el.value = '{}'; el.dispatchEvent(new Event('input',{{bubbles:true}})); el.dispatchEvent(new Event('change',{{bubbles:true}})); return true; }})()",
            selector.replace('\'', "\\'"),
            text.replace('\'', "\\'")
        );
        self.execute_js(&expression).await?;
        Ok(())
    }

    pub async fn get_content(&self) -> Result<PageContent, String> {
        let inner = self.inner.lock().await;
        let cdp = inner.cdp.as_ref().ok_or("Browser not launched")?;
        let sid = inner.session_id.as_deref().ok_or("No session")?;
        self.eval_page_content(cdp, sid).await
    }

    async fn eval_page_content(&self, cdp: &CdpClient, sid: &str) -> Result<PageContent, String> {
        let result = cdp
            .call_with_session(
                "Runtime.evaluate",
                Some(json!({
                    "expression": "JSON.stringify({url: location.href, title: document.title, text: document.body?.innerText || ''})",
                    "returnByValue": true
                })),
                Some(sid),
            )
            .await?;

        let value_str = result["result"]["value"].as_str().unwrap_or("{}");
        let content: Value = serde_json::from_str(value_str).unwrap_or_else(|_| json!({}));

        Ok(PageContent {
            url: content["url"].as_str().unwrap_or("").to_string(),
            title: content["title"].as_str().unwrap_or("").to_string(),
            text: content["text"].as_str().unwrap_or("").to_string(),
        })
    }

    pub async fn get_markdown(&self) -> Result<String, String> {
        let inner = self.inner.lock().await;
        let cdp = inner.cdp.as_ref().ok_or("Browser not launched")?;
        let sid = inner.session_id.as_deref().ok_or("No session")?;

        let result = cdp
            .call_with_session("LP.getMarkdown", None, Some(sid))
            .await?;

        Ok(result["markdown"].as_str().unwrap_or("").to_string())
    }

    pub async fn status(&self) -> Value {
        let inner = self.inner.lock().await;
        json!({
            "running": inner.process.is_some(),
            "port": inner.port,
            "stealth": inner.stealth,
        })
    }

    pub async fn close(&self) -> Result<(), String> {
        let mut inner = self.inner.lock().await;

        if let Some(cdp) = inner.cdp.take() {
            let _ = cdp.close().await;
        }

        if let Some(mut process) = inner.process.take() {
            let _ = process.kill().await;
            tracing::info!("Obscura browser closed");
        }

        inner.session_id = None;
        Ok(())
    }
}
