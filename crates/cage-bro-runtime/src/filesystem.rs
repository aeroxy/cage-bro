use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("file not found: {0}")]
    NotFound(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("io error: {0}")]
    IoError(String),
    #[error("path outside sandbox: {0}")]
    PathEscape(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadResult {
    pub path: String,
    pub content: String,
    pub encoding: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWriteRequest {
    pub path: String,
    pub content: String,
    pub encoding: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEditRequest {
    pub path: String,
    pub old_text: String,
    pub new_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchRequest {
    pub query: String,
    pub path: Option<String>,
    pub file_pattern: Option<String>,
    pub max_results: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchResult {
    pub path: String,
    pub line_number: usize,
    pub line_content: String,
}

#[async_trait]
pub trait Filesystem: Send + Sync {
    async fn read(&self, path: &str) -> Result<FileReadResult, FileError>;
    async fn write(&self, request: FileWriteRequest) -> Result<(), FileError>;
    async fn edit(&self, request: FileEditRequest) -> Result<(), FileError>;
    async fn list(&self, path: &str) -> Result<Vec<FileInfo>, FileError>;
    async fn delete(&self, path: &str) -> Result<(), FileError>;
    async fn search(&self, request: FileSearchRequest) -> Result<Vec<FileSearchResult>, FileError>;
    async fn exists(&self, path: &str) -> Result<bool, FileError>;
}
