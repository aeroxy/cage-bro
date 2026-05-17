use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("sandbox creation failed: {0}")]
    CreationFailed(String),
    #[error("execution failed: {0}")]
    ExecutionFailed(String),
    #[error("sandbox not found: {0}")]
    NotFound(Uuid),
    #[error("destroy failed: {0}")]
    DestroyFailed(String),
    #[error("snapshot failed: {0}")]
    SnapshotFailed(String),
    #[error("restore failed: {0}")]
    RestoreFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub memory_limit_mb: Option<u64>,
    pub cpu_limit_percent: Option<u8>,
    pub disk_limit_mb: Option<u64>,
    pub network_enabled: bool,
    pub workspace_dir: Option<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            memory_limit_mb: Some(512),
            cpu_limit_percent: Some(50),
            disk_limit_mb: Some(1024),
            network_enabled: true,
            workspace_dir: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sandbox {
    pub id: Uuid,
    pub config: SandboxConfig,
    pub state: SandboxState,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SandboxState {
    Creating,
    Running,
    Paused,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecCommand {
    pub program: String,
    pub args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
    pub working_dir: Option<String>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: Uuid,
    pub sandbox_id: Uuid,
    pub created_at: String,
    pub metadata: std::collections::HashMap<String, String>,
}

#[async_trait]
pub trait SandboxRuntime: Send + Sync {
    async fn create(&self, config: SandboxConfig) -> Result<Sandbox, RuntimeError>;
    async fn exec(&self, sandbox: &Sandbox, cmd: ExecCommand) -> Result<ExecResult, RuntimeError>;
    async fn destroy(&self, sandbox: &Sandbox) -> Result<(), RuntimeError>;
    async fn snapshot(&self, sandbox: &Sandbox) -> Result<Snapshot, RuntimeError>;
    async fn restore(&self, snapshot: &Snapshot) -> Result<Sandbox, RuntimeError>;
}
