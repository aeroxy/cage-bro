use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodeError {
    #[error("execution failed: {0}")]
    ExecutionFailed(String),
    #[error("timeout after {0}ms")]
    Timeout(u64),
    #[error("runtime not found: {0}")]
    RuntimeNotFound(String),
    #[error("kernel error: {0}")]
    KernelError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRequest {
    pub language: Language,
    pub code: String,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Language {
    Python,
    Node,
    Ruby,
    Shell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub mime_type: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelInfo {
    pub id: String,
    pub language: Language,
    pub status: KernelStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KernelStatus {
    Starting,
    Ready,
    Busy,
    Dead,
}

#[async_trait]
pub trait CodeEngine: Send + Sync {
    async fn execute_stateless(&self, request: CodeRequest) -> Result<CodeResult, CodeError>;
    async fn kernel_start(&self, language: Language) -> Result<String, CodeError>;
    async fn kernel_execute(&self, kernel_id: &str, code: &str) -> Result<CodeResult, CodeError>;
    async fn kernel_interrupt(&self, kernel_id: &str) -> Result<(), CodeError>;
    async fn kernel_restart(&self, kernel_id: &str) -> Result<(), CodeError>;
    async fn kernel_shutdown(&self, kernel_id: &str) -> Result<(), CodeError>;
    async fn kernel_list(&self) -> Result<Vec<KernelInfo>, CodeError>;
}
