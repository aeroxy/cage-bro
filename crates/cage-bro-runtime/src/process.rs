use async_trait::async_trait;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;
use wait_timeout::ChildExt;

use crate::isolation::Isolation;
use crate::traits::*;

/// Where snapshot trees are stored on disk, relative to the runtime base dir.
const SNAPSHOT_SUBDIR: &str = ".cage-bro/snapshots";
/// Where restored/forked workspaces are materialized.
const RESTORE_SUBDIR: &str = ".cage-bro/restored";
/// Cap on captured stdout/stderr per exec, bounding server memory against
/// runaway output from untrusted code. Output past this is dropped; the child
/// receives EPIPE once the reader thread closes the pipe.
const MAX_CAPTURE_BYTES: u64 = 10 * 1024 * 1024;

pub struct ProcessRuntime {
    sandboxes: Arc<Mutex<HashMap<Uuid, Sandbox>>>,
    snapshots: Arc<Mutex<HashMap<Uuid, SnapshotRecord>>>,
    /// Base directory under which snapshot/restore trees live.
    base_dir: PathBuf,
}

/// Internal bookkeeping for a stored snapshot: the on-disk directory holding
/// the captured workspace tree.
#[derive(Clone)]
struct SnapshotRecord {
    path: PathBuf,
}

impl ProcessRuntime {
    pub fn new() -> Self {
        let base_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::with_base_dir(base_dir)
    }

    /// Construct a runtime that stores snapshots under `base_dir`.
    pub fn with_base_dir(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            sandboxes: Arc::new(Mutex::new(HashMap::new())),
            snapshots: Arc::new(Mutex::new(HashMap::new())),
            base_dir: base_dir.into(),
        }
    }
}

impl Default for ProcessRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessRuntime {
    /// All currently-tracked sandboxes (for lifecycle/listing APIs).
    pub async fn list(&self) -> Vec<Sandbox> {
        self.sandboxes.lock().await.values().cloned().collect()
    }

    /// Look up a tracked sandbox by id.
    pub async fn get(&self, id: &Uuid) -> Option<Sandbox> {
        self.sandboxes.lock().await.get(id).cloned()
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
        let isolation = Isolation::from_config(&sandbox.config, cmd.timeout_ms);
        let timeout = cmd.timeout_ms.map(Duration::from_millis);

        let working_dir = cmd
            .working_dir
            .clone()
            .or_else(|| sandbox.config.workspace_dir.clone());

        // Process spawning + waiting is blocking; run it off the async runtime.
        tokio::task::spawn_blocking(move || run_isolated(isolation, cmd, working_dir, timeout))
            .await
            .map_err(|e| RuntimeError::ExecutionFailed(format!("join error: {}", e)))?
    }

    async fn destroy(&self, sandbox: &Sandbox) -> Result<(), RuntimeError> {
        let mut sandboxes = self.sandboxes.lock().await;
        sandboxes.remove(&sandbox.id);
        tracing::info!(sandbox_id = %sandbox.id, "Sandbox destroyed");
        Ok(())
    }

    /// Capture the sandbox's workspace tree to the snapshot store. Filesystem
    /// state only — process/memory state is not preserved (see crate README).
    async fn snapshot(&self, sandbox: &Sandbox) -> Result<Snapshot, RuntimeError> {
        let workspace = sandbox.config.workspace_dir.clone().ok_or_else(|| {
            RuntimeError::SnapshotFailed("sandbox has no workspace_dir to snapshot".into())
        })?;

        let snapshot_id = Uuid::new_v4();
        let dest = self.base_dir.join(SNAPSHOT_SUBDIR).join(snapshot_id.to_string());
        let src = PathBuf::from(&workspace);

        tokio::task::spawn_blocking(move || copy_tree(&src, &dest))
            .await
            .map_err(|e| RuntimeError::SnapshotFailed(format!("join error: {}", e)))?
            .map_err(|e| RuntimeError::SnapshotFailed(e.to_string()))?;

        let snapshot = Snapshot {
            id: snapshot_id,
            sandbox_id: sandbox.id,
            created_at: chrono_now(),
            metadata: HashMap::from([("source".into(), workspace)]),
        };

        let dest = self.base_dir.join(SNAPSHOT_SUBDIR).join(snapshot_id.to_string());
        self.snapshots
            .lock()
            .await
            .insert(snapshot_id, SnapshotRecord { path: dest });

        tracing::info!(sandbox_id = %sandbox.id, snapshot_id = %snapshot_id, "Snapshot captured");
        Ok(snapshot)
    }

    /// Materialize a snapshot into a fresh workspace and return a new sandbox
    /// bound to it. Calling this repeatedly forks independent copies, enabling
    /// clone / rollback / parallel-exploration workflows.
    async fn restore(&self, snapshot: &Snapshot) -> Result<Sandbox, RuntimeError> {
        let record = self
            .snapshots
            .lock()
            .await
            .get(&snapshot.id)
            .cloned()
            .ok_or_else(|| RuntimeError::RestoreFailed(format!("unknown snapshot: {}", snapshot.id)))?;

        let new_id = Uuid::new_v4();
        let dest = self.base_dir.join(RESTORE_SUBDIR).join(new_id.to_string());
        let src = record.path.clone();

        let dest_for_copy = dest.clone();
        tokio::task::spawn_blocking(move || copy_tree(&src, &dest_for_copy))
            .await
            .map_err(|e| RuntimeError::RestoreFailed(format!("join error: {}", e)))?
            .map_err(|e| RuntimeError::RestoreFailed(e.to_string()))?;

        // Re-derive config from the snapshotted sandbox, repointing the workspace.
        let config = SandboxConfig {
            workspace_dir: Some(dest.to_string_lossy().to_string()),
            ..Default::default()
        };

        let sandbox = Sandbox {
            id: new_id,
            config,
            state: SandboxState::Running,
            created_at: chrono_now(),
        };
        self.sandboxes.lock().await.insert(new_id, sandbox.clone());

        tracing::info!(snapshot_id = %snapshot.id, sandbox_id = %new_id, "Snapshot restored into new sandbox");
        Ok(sandbox)
    }
}

/// Spawn `cmd` under `isolation`, capture stdout/stderr, and enforce `timeout`.
///
/// Runs on a blocking thread. stdout/stderr are drained by dedicated reader
/// threads so a child that fills a pipe buffer cannot deadlock the timeout.
fn run_isolated(
    isolation: Isolation,
    cmd: ExecCommand,
    working_dir: Option<String>,
    timeout: Option<Duration>,
) -> Result<ExecResult, RuntimeError> {
    let start = std::time::Instant::now();

    let mut command = std::process::Command::new(&cmd.program);
    command.args(&cmd.args);
    command.envs(&cmd.env);
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    if let Some(ref dir) = working_dir {
        command.current_dir(dir);
    }

    // Register isolation (pre_exec hook). The guard holds the parent's copy of
    // the Landlock descriptor; the child inherits its own copy in pre_exec, so
    // the parent's can be released as soon as the fork (spawn) has happened.
    let guard = isolation.apply(&mut command);

    let mut child = command
        .spawn()
        .map_err(|e| RuntimeError::ExecutionFailed(format!("spawn failed: {}", e)))?;
    drop(guard);

    // Drain stdout/stderr concurrently to avoid pipe-buffer deadlock.
    let stdout_handle = child.stdout.take().map(spawn_reader);
    let stderr_handle = child.stderr.take().map(spawn_reader);

    let (exit_code, timed_out) = match timeout {
        Some(dur) => match child.wait_timeout(dur) {
            Ok(Some(status)) => (status.code().unwrap_or(-1), false),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                (-1, true)
            }
            Err(e) => return Err(RuntimeError::ExecutionFailed(format!("wait failed: {}", e))),
        },
        None => match child.wait() {
            Ok(status) => (status.code().unwrap_or(-1), false),
            Err(e) => return Err(RuntimeError::ExecutionFailed(format!("wait failed: {}", e))),
        },
    };

    let stdout = stdout_handle.and_then(|h| h.join().ok()).unwrap_or_default();
    let mut stderr = stderr_handle.and_then(|h| h.join().ok()).unwrap_or_default();
    if timed_out {
        stderr.push_str(&format!("\n[cage-bro] process killed: exceeded timeout of {:?}", timeout));
    }

    Ok(ExecResult {
        exit_code,
        stdout,
        stderr,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

/// Spawn a thread that reads a child pipe to EOF, lossily decoding as UTF-8.
fn spawn_reader<R: Read + Send + 'static>(mut reader: R) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = reader.by_ref().take(MAX_CAPTURE_BYTES).read_to_end(&mut buf);
        String::from_utf8_lossy(&buf).into_owned()
    })
}

/// Recursively copy a directory tree from `src` to `dst`, creating `dst`.
fn copy_tree(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let target = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else if file_type.is_symlink() {
            // Preserve symlinks as-is rather than following them.
            #[cfg(unix)]
            {
                let link = std::fs::read_link(entry.path())?;
                std::os::unix::fs::symlink(link, &target)?;
            }
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn chrono_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}
