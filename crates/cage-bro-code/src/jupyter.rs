use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::traits::*;

pub struct JupyterKernelManager {
    kernels: Arc<Mutex<HashMap<String, KernelHandle>>>,
}

struct KernelHandle {
    info: KernelInfo,
    process: tokio::process::Child,
    connection_file: String,
}

#[derive(Serialize, Deserialize)]
struct ConnectionFile {
    ip: String,
    transport: String,
    signature_scheme: String,
    key: String,
    shell_port: u16,
    iopub_port: u16,
    stdin_port: u16,
    control_port: u16,
    hb_port: u16,
}

const EXECUTE_SCRIPT: &str = r#"
import json, sys
from jupyter_client.manager import KernelManager

conn_file = sys.argv[1]
code = sys.argv[2]

km = KernelManager(connection_file=conn_file)
km.load_connection_file()
kc = km.client()
kc.start_channels()
try:
    kc.wait_for_ready(timeout=10)
    kc.execute(code, silent=False, store_history=True)
    stdout = ''
    stderr = ''
    for _ in range(200):
        try:
            msg = kc.get_iopub_msg(timeout=5)
            mt = msg['header']['msg_type']
            if mt == 'stream':
                if msg['content'].get('name') == 'stderr':
                    stderr += msg['content']['text']
                else:
                    stdout += msg['content']['text']
            elif mt == 'execute_result':
                stdout += msg['content']['data'].get('text/plain', '')
            elif mt == 'error':
                stderr += msg['content']['ename'] + ': ' + msg['content']['evalue']
            elif mt == 'status' and msg['content']['execution_state'] == 'idle':
                break
        except Exception:
            break
    print(json.dumps({'stdout': stdout, 'stderr': stderr, 'exit_code': 0}))
finally:
    kc.stop_channels()
"#;

impl JupyterKernelManager {
    pub fn new() -> Self {
        Self {
            kernels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start_kernel(&self, language: &str) -> Result<String, CodeError> {
        let kernel_id = Uuid::new_v4().to_string();
        let key = Uuid::new_v4().to_string().replace('-', "");

        let ports = find_free_ports(5)
            .map_err(|e| CodeError::KernelError(format!("Failed to find ports: {}", e)))?;

        let conn = ConnectionFile {
            ip: "127.0.0.1".to_string(),
            transport: "tcp".to_string(),
            signature_scheme: "hmac-sha256".to_string(),
            key: key.clone(),
            shell_port: ports[0],
            iopub_port: ports[1],
            stdin_port: ports[2],
            control_port: ports[3],
            hb_port: ports[4],
        };

        let conn_json = serde_json::to_string_pretty(&conn)
            .map_err(|e| CodeError::KernelError(format!("Serialize failed: {}", e)))?;

        let conn_dir = std::env::temp_dir().join("cage-bro-kernels");
        tokio::fs::create_dir_all(&conn_dir)
            .await
            .map_err(|e| CodeError::KernelError(format!("Mkdir failed: {}", e)))?;

        let conn_file = conn_dir.join(format!("kernel-{}.json", kernel_id));
        tokio::fs::write(&conn_file, &conn_json)
            .await
            .map_err(|e| CodeError::KernelError(format!("Write conn file failed: {}", e)))?;

        let kernel_cmd = match language {
            "python" => "python3",
            _ => return Err(CodeError::RuntimeNotFound(language.to_string())),
        };

        let child = tokio::process::Command::new(kernel_cmd)
            .args(["-m", "ipykernel", "-f", &conn_file.to_string_lossy()])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                CodeError::KernelError(format!(
                    "Failed to launch kernel: {}. Is ipykernel installed?",
                    e
                ))
            })?;

        // Wait for kernel to start
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        let info = KernelInfo {
            id: kernel_id.clone(),
            language: crate::traits::Language::Python,
            status: KernelStatus::Ready,
        };

        let handle = KernelHandle {
            info: info.clone(),
            process: child,
            connection_file: conn_file.to_string_lossy().to_string(),
        };

        let mut kernels = self.kernels.lock().await;
        kernels.insert(kernel_id.clone(), handle);

        tracing::info!(kernel_id = %kernel_id, language = language, "Jupyter kernel started");
        Ok(kernel_id)
    }

    pub async fn execute(&self, kernel_id: &str, code: &str) -> Result<CodeResult, CodeError> {
        let kernels = self.kernels.lock().await;
        let kernel = kernels
            .get(kernel_id)
            .ok_or_else(|| CodeError::KernelError(format!("Kernel not found: {}", kernel_id)))?;

        let conn_file = kernel.connection_file.clone();
        let start = std::time::Instant::now();

        // Write execute script to temp file
        let script_file = std::env::temp_dir().join(format!("cage_bro_exec_{}.py", Uuid::new_v4()));
        std::fs::write(&script_file, EXECUTE_SCRIPT)
            .map_err(|e| CodeError::KernelError(format!("Write script failed: {}", e)))?;

        let output = tokio::process::Command::new("python3")
            .args([
                script_file.to_string_lossy().as_ref(),
                &conn_file,
                code,
            ])
            .output()
            .await
            .map_err(|e| CodeError::KernelError(format!("Execute failed: {}", e)))?;

        let _ = tokio::fs::remove_file(&script_file).await;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr_str = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Err(CodeError::KernelError(format!(
                "Execute failed: {}",
                stderr_str
            )));
        }

        let result: serde_json::Value = serde_json::from_str(stdout.trim())
            .map_err(|e| CodeError::KernelError(format!("Parse failed: {}. Output: {}", e, stdout)))?;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(CodeResult {
            stdout: result["stdout"].as_str().unwrap_or("").to_string(),
            stderr: result["stderr"].as_str().unwrap_or("").to_string(),
            exit_code: result["exit_code"].as_i64().unwrap_or(1) as i32,
            duration_ms,
            artifacts: vec![],
        })
    }

    pub async fn interrupt(&self, kernel_id: &str) -> Result<(), CodeError> {
        let kernels = self.kernels.lock().await;
        let _kernel = kernels
            .get(kernel_id)
            .ok_or_else(|| CodeError::KernelError(format!("Kernel not found: {}", kernel_id)))?;
        Ok(())
    }

    pub async fn shutdown(&self, kernel_id: &str) -> Result<(), CodeError> {
        let mut kernels = self.kernels.lock().await;
        if let Some(mut kernel) = kernels.remove(kernel_id) {
            let _ = kernel.process.kill().await;
            let _ = tokio::fs::remove_file(&kernel.connection_file).await;
            tracing::info!(kernel_id = %kernel_id, "Kernel shut down");
        }
        Ok(())
    }

    pub async fn list(&self) -> Vec<KernelInfo> {
        let kernels = self.kernels.lock().await;
        kernels.values().map(|k| k.info.clone()).collect()
    }
}

fn find_free_ports(n: usize) -> Result<Vec<u16>, std::io::Error> {
    let mut ports = Vec::new();
    for _ in 0..n {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        ports.push(port);
    }
    Ok(ports)
}
