use async_trait::async_trait;
use std::path::{Component, Path, PathBuf};

use crate::filesystem::*;

/// Normalize a path by resolving `.` and `..` without requiring the path to exist.
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                components.pop();
            }
            other => components.push(other),
        }
    }
    components.iter().collect()
}

pub struct LocalFilesystem {
    root: PathBuf,
}

impl LocalFilesystem {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn resolve(&self, path: &str) -> Result<PathBuf, FileError> {
        let p = Path::new(path);
        let resolved = if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.root.join(p)
        };

        // Normalize the path (resolve .. and . components) without requiring existence
        let normalized = normalize_path(&resolved);

        // Normalize root for comparison
        let root_normalized = normalize_path(&self.root);

        // Ensure the root exists
        if !self.root.exists() {
            return Err(FileError::IoError(format!(
                "Workspace root does not exist: {:?}",
                self.root
            )));
        }

        // Check that the normalized path is under the root
        if !normalized.starts_with(&root_normalized) {
            return Err(FileError::PathEscape(path.to_string()));
        }

        Ok(normalized)
    }
}

#[async_trait]
impl Filesystem for LocalFilesystem {
    async fn read(&self, path: &str) -> Result<FileReadResult, FileError> {
        let resolved = self.resolve(path)?;
        let content = tokio::fs::read_to_string(&resolved)
            .await
            .map_err(|e| FileError::IoError(e.to_string()))?;

        Ok(FileReadResult {
            path: resolved.to_string_lossy().to_string(),
            content,
            encoding: "utf-8".to_string(),
        })
    }

    async fn write(&self, request: FileWriteRequest) -> Result<(), FileError> {
        let resolved = self.resolve(&request.path)?;

        if let Some(parent) = resolved.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| FileError::IoError(e.to_string()))?;
        }

        tokio::fs::write(&resolved, &request.content)
            .await
            .map_err(|e| FileError::IoError(e.to_string()))?;

        Ok(())
    }

    async fn edit(&self, request: FileEditRequest) -> Result<(), FileError> {
        let resolved = self.resolve(&request.path)?;
        let content = tokio::fs::read_to_string(&resolved)
            .await
            .map_err(|e| FileError::IoError(e.to_string()))?;

        if !content.contains(&request.old_text) {
            return Err(FileError::IoError(
                "old_text not found in file".to_string(),
            ));
        }

        let new_content = content.replacen(&request.old_text, &request.new_text, 1);

        tokio::fs::write(&resolved, &new_content)
            .await
            .map_err(|e| FileError::IoError(e.to_string()))?;

        Ok(())
    }

    async fn list(&self, path: &str) -> Result<Vec<FileInfo>, FileError> {
        let resolved = self.resolve(path)?;
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(&resolved)
            .await
            .map_err(|e| FileError::IoError(e.to_string()))?;

        while let Some(entry) = dir
            .next_entry()
            .await
            .map_err(|e| FileError::IoError(e.to_string()))?
        {
            let metadata = entry
                .metadata()
                .await
                .map_err(|e| FileError::IoError(e.to_string()))?;

            let name = entry.file_name().to_string_lossy().to_string();

            entries.push(FileInfo {
                path: entry.path().to_string_lossy().to_string(),
                name,
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                modified: format!(
                    "{:?}",
                    metadata
                        .modified()
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                ),
            });
        }

        Ok(entries)
    }

    async fn delete(&self, path: &str) -> Result<(), FileError> {
        let resolved = self.resolve(path)?;
        let metadata = tokio::fs::metadata(&resolved)
            .await
            .map_err(|e| FileError::IoError(e.to_string()))?;

        if metadata.is_dir() {
            tokio::fs::remove_dir_all(&resolved)
                .await
                .map_err(|e| FileError::IoError(e.to_string()))?;
        } else {
            tokio::fs::remove_file(&resolved)
                .await
                .map_err(|e| FileError::IoError(e.to_string()))?;
        }

        Ok(())
    }

    async fn search(&self, request: FileSearchRequest) -> Result<Vec<FileSearchResult>, FileError> {
        let search_root = match &request.path {
            Some(p) => self.resolve(p)?,
            None => self.root.clone(),
        };

        let pattern = request.file_pattern.as_deref().unwrap_or("*");
        let max_results = request.max_results.unwrap_or(100);
        let query = request.query.to_lowercase();

        let mut results = Vec::new();
        let mut stack = vec![search_root];

        while let Some(dir) = stack.pop() {
            let mut entries = match tokio::fs::read_dir(&dir).await {
                Ok(e) => e,
                Err(_) => continue,
            };

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| FileError::IoError(e.to_string()))?
            {
                let path = entry.path();
                let metadata = entry
                    .metadata()
                    .await
                    .map_err(|e| FileError::IoError(e.to_string()))?;

                if metadata.is_dir() {
                    stack.push(path.clone());
                    continue;
                }

                // Simple glob matching for file pattern
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let matches_pattern = if pattern == "*" {
                    true
                } else if let Some(ext) = pattern.strip_prefix("*.") {
                    name.ends_with(&format!(".{}", ext))
                } else {
                    name.contains(pattern)
                };

                if !matches_pattern {
                    continue;
                }

                // Search file content
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    for (line_num, line) in content.lines().enumerate() {
                        if line.to_lowercase().contains(&query) {
                            results.push(FileSearchResult {
                                path: path.to_string_lossy().to_string(),
                                line_number: line_num + 1,
                                line_content: line.to_string(),
                            });

                            if results.len() >= max_results {
                                return Ok(results);
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    async fn exists(&self, path: &str) -> Result<bool, FileError> {
        let resolved = match self.resolve(path) {
            Ok(p) => p,
            Err(_) => return Ok(false),
        };
        Ok(resolved.exists())
    }
}
