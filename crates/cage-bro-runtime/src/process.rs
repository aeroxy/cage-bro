use async_trait::async_trait;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::traits::*;

pub struct ProcessRuntime {
    sandboxes: Arc<Mutex<HashMap<Uuid, Sandbox>>>,
}

impl ProcessRuntime {
    pub fn new() -> Self {
        Self {
            sandboxes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl SandboxRuntime for ProcessRuntime {
    async fn create(&self, config: SandboxConfig) -> Result<Sandbox, RuntimeError> {
        let sandbox = Sandbox {
            id: Uuid::new_v4(),
            config,
            state: SandboxState::Running,
            created_at: chrono_now(),
        };

        let mut sandboxes = self.sandboxes.lock().await;
        sandboxes.insert(sandbox.id, sandbox.clone());

        tracing::info!(sandbox_id = %sandbox.id, "Sandbox created");
        Ok(sandbox)
    }

    async fn exec(&self, sandbox: &Sandbox, cmd: ExecCommand) -> Result<ExecResult, RuntimeError> {
        let start = std::time::Instant::now();

        let pty_system = native_pty_system();
        let pty = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| RuntimeError::ExecutionFailed(format!("PTY open failed: {}", e)))?;

        let mut builder = CommandBuilder::new(&cmd.program);
        builder.args(&cmd.args);

        for (k, v) in &cmd.env {
            builder.env(k, v);
        }

        if let Some(ref dir) = cmd.working_dir {
            builder.cwd(dir);
        } else if let Some(ref dir) = sandbox.config.workspace_dir {
            builder.cwd(dir);
        }

        let mut child = pty
            .slave.spawn_command(builder)
            .map_err(|e| RuntimeError::ExecutionFailed(format!("Spawn failed: {}", e)))?;

        let mut reader = pty
            .master
            .try_clone_reader()
            .map_err(|e| RuntimeError::ExecutionFailed(format!("Reader clone failed: {}", e)))?;

        let mut stdout = String::new();
        reader
            .read_to_string(&mut stdout)
            .map_err(|e| RuntimeError::ExecutionFailed(format!("Read failed: {}", e)))?;

        let exit_status = child
            .wait()
            .map_err(|e| RuntimeError::ExecutionFailed(format!("Wait failed: {}", e)))?;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(ExecResult {
            exit_code: exit_status.exit_code() as i32,
            stdout,
            stderr: String::new(),
            duration_ms,
        })
    }

    async fn destroy(&self, sandbox: &Sandbox) -> Result<(), RuntimeError> {
        let mut sandboxes = self.sandboxes.lock().await;
        sandboxes.remove(&sandbox.id);
        tracing::info!(sandbox_id = %sandbox.id, "Sandbox destroyed");
        Ok(())
    }

    async fn snapshot(&self, _sandbox: &Sandbox) -> Result<Snapshot, RuntimeError> {
        Err(RuntimeError::SnapshotFailed(
            "Snapshots not supported in process mode".into(),
        ))
    }

    async fn restore(&self, _snapshot: &Snapshot) -> Result<Sandbox, RuntimeError> {
        Err(RuntimeError::RestoreFailed(
            "Restore not supported in process mode".into(),
        ))
    }
}

fn chrono_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}
