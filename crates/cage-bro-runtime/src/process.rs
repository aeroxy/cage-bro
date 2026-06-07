use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::isolation::Isolation;
use crate::traits::*;

/// Where snapshot trees are stored on disk, relative to the runtime base dir.
const SNAPSHOT_SUBDIR: &str = ".cage-bro/snapshots";
/// Where restored/forked workspaces are materialized.
const RESTORE_SUBDIR: &str = ".cage-bro/restored";
/// Cap on captured stdout/stderr per exec, bounding server memory against
/// runaway output from untrusted code. Output past this is dropped; the child
/// receives EPIPE once the reader closes the pipe.
const MAX_CAPTURE_BYTES: u64 = 10 * 1024 * 1024;
/// Grace period to drain buffered output after the child exits before aborting
/// the reader tasks (so a pipe held open by an escaped descendant can't hang us).
const DRAIN_GRACE: Duration = Duration::from_millis(500);

pub struct ProcessRuntime {
    sandboxes: Arc<Mutex<HashMap<Uuid, Sandbox>>>,
    snapshots: Arc<Mutex<HashMap<Uuid, SnapshotRecord>>>,
    /// Base directory under which snapshot/restore trees live.
    base_dir: PathBuf,
}

/// Internal bookkeeping for a stored snapshot: the on-disk directory holding
/// the captured workspace tree, plus the originating sandbox's config so a
/// restore preserves its resource limits / network setting instead of
/// reverting to defaults.
#[derive(Clone)]
struct SnapshotRecord {
    path: PathBuf,
    config: SandboxConfig,
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

        run_isolated(isolation, cmd, working_dir, timeout).await
    }

    async fn destroy(&self, sandbox: &Sandbox) -> Result<(), RuntimeError> {
        // Drop the registry entry and release the lock before any filesystem I/O.
        let removed = self.sandboxes.lock().await.remove(&sandbox.id).is_some();

        // If this sandbox owns a restored/forked workspace (created by `restore`
        // under .cage-bro/restored), remove it so repeated restore→destroy cycles
        // don't leak dirs. The cheap lexical pre-check keeps the common exec path
        // (workspace = cwd/workspace, or e2b dirs) free of any syscall; the
        // canonicalized strict-subdir check then guards the actual delete so it
        // can't escape the restored base.
        if removed {
            if let Some(ref ws) = sandbox.config.workspace_dir {
                let restore_base = self.base_dir.join(RESTORE_SUBDIR);
                if Path::new(ws).starts_with(&restore_base) {
                    if let (Ok(ws_real), Ok(base_real)) = (
                        tokio::fs::canonicalize(ws).await,
                        tokio::fs::canonicalize(&restore_base).await,
                    ) {
                        if ws_real != base_real && ws_real.starts_with(&base_real) {
                            let _ = tokio::fs::remove_dir_all(&ws_real).await;
                        }
                    }
                }
            }
        }

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

        copy_tree_async(src, dest.clone())
            .await
            .map_err(RuntimeError::SnapshotFailed)?;

        let snapshot = Snapshot {
            id: snapshot_id,
            sandbox_id: sandbox.id,
            created_at: chrono_now(),
            metadata: HashMap::from([("source".into(), workspace)]),
        };

        self.snapshots.lock().await.insert(
            snapshot_id,
            SnapshotRecord { path: dest, config: sandbox.config.clone() },
        );

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

        copy_tree_async(src, dest.clone())
            .await
            .map_err(RuntimeError::RestoreFailed)?;

        // Reuse the snapshotted sandbox's config, repointing only the workspace.
        let mut config = record.config.clone();
        config.workspace_dir = Some(dest.to_string_lossy().to_string());

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
async fn run_isolated(
    isolation: Isolation,
    cmd: ExecCommand,
    working_dir: Option<String>,
    timeout: Option<Duration>,
) -> Result<ExecResult, RuntimeError> {
    let start = std::time::Instant::now();

    // Build with std::process::Command so isolation can install its pre_exec
    // hook, then hand it to tokio for async (thread-free, cancellable) I/O.
    let mut std_command = std::process::Command::new(&cmd.program);
    std_command.args(&cmd.args);
    std_command.envs(&cmd.env);
    std_command.stdin(Stdio::null());
    std_command.stdout(Stdio::piped());
    std_command.stderr(Stdio::piped());
    if let Some(ref dir) = working_dir {
        std_command.current_dir(dir);
    }
    // Put the child in its own process group so a timeout can kill the whole
    // tree (same-group descendants), not just the direct child.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        std_command.process_group(0);
    }

    // Register isolation (pre_exec hook). The guard holds the parent's copy of
    // the Landlock descriptor; the child inherits its own copy in pre_exec, so
    // the parent's can be released as soon as the fork (spawn) has happened.
    let guard = isolation.apply(&mut std_command);

    let mut command = tokio::process::Command::from(std_command);
    command.kill_on_drop(true);

    let mut child = command
        .spawn()
        .map_err(|e| RuntimeError::ExecutionFailed(format!("spawn failed: {}", e)))?;
    drop(guard);

    // If this future is cancelled (e.g. the client disconnects) before the child
    // is reaped, SIGKILL the whole process group so descendants don't orphan —
    // kill_on_drop only kills the direct child. Disarmed after the reap below so
    // it never targets a recycled PID.
    #[cfg(unix)]
    let mut group_guard = GroupKillGuard { pgid: child.id().map(|p| p as libc::pid_t) };

    // Drain stdout/stderr concurrently into shared buffers via async tasks (no
    // OS threads). Reads are cancellable, so a descendant that escapes the
    // process group and holds the pipe open can't make us hang or leak.
    let cap = MAX_CAPTURE_BYTES as usize;
    let stdout_buf = Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
    let stderr_buf = Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
    let out_task = child.stdout.take().map(|r| tokio::spawn(pump(r, stdout_buf.clone(), cap)));
    let err_task = child.stderr.take().map(|r| tokio::spawn(pump(r, stderr_buf.clone(), cap)));

    // Helper: a wait() result is terminal — once it resolves (Ok or Err) the
    // child is no longer ours to signal, so disarm the guard *before* acting on
    // it. Otherwise an error return would drop the armed guard and could
    // `killpg` a now-recycled PID.
    let (exit_code, timed_out) = match timeout {
        Some(dur) => match tokio::time::timeout(dur, child.wait()).await {
            Ok(res) => {
                #[cfg(unix)]
                group_guard.disarm();
                match res {
                    Ok(status) => (status.code().unwrap_or(-1), false),
                    Err(e) => return Err(RuntimeError::ExecutionFailed(format!("wait failed: {}", e))),
                }
            }
            Err(_elapsed) => {
                // Timed out: kill the whole group so same-group descendants die,
                // reap, then disarm (PID may be recycled after this).
                #[cfg(unix)]
                if let Some(pid) = child.id() {
                    unsafe { libc::killpg(pid as libc::pid_t, libc::SIGKILL); }
                }
                #[cfg(not(unix))]
                let _ = child.start_kill();
                let _ = child.wait().await;
                #[cfg(unix)]
                group_guard.disarm();
                (-1, true)
            }
        },
        None => {
            let res = child.wait().await;
            #[cfg(unix)]
            group_guard.disarm();
            match res {
                Ok(status) => (status.code().unwrap_or(-1), false),
                Err(e) => return Err(RuntimeError::ExecutionFailed(format!("wait failed: {}", e))),
            }
        }
    };

    // The direct child has exited (or been killed). Give the readers a brief
    // grace to flush buffered output, then abort them — a pipe held open by an
    // escaped descendant must not block us. Aborting captures whatever made it
    // into the shared buffers; it never blocks (tasks own no OS threads).
    let out_abort = out_task.as_ref().map(|h| h.abort_handle());
    let err_abort = err_task.as_ref().map(|h| h.abort_handle());
    let join_readers = async move {
        if let Some(h) = out_task { let _ = h.await; }
        if let Some(h) = err_task { let _ = h.await; }
    };
    if tokio::time::timeout(DRAIN_GRACE, join_readers).await.is_err() {
        if let Some(a) = out_abort { a.abort(); }
        if let Some(a) = err_abort { a.abort(); }
    }

    let stdout = String::from_utf8_lossy(&stdout_buf.lock().unwrap()).into_owned();
    let mut stderr = String::from_utf8_lossy(&stderr_buf.lock().unwrap()).into_owned();
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

/// RAII guard that SIGKILLs the child's whole process group if dropped while
/// still armed — e.g. when the exec future is cancelled mid-flight. Disarmed
/// once the child is reaped, so it never targets a recycled PID. (The child is
/// put in its own group via `process_group(0)`, so its PID is the group id.)
#[cfg(unix)]
struct GroupKillGuard {
    pgid: Option<libc::pid_t>,
}

#[cfg(unix)]
impl GroupKillGuard {
    fn disarm(&mut self) {
        self.pgid = None;
    }
}

#[cfg(unix)]
impl Drop for GroupKillGuard {
    fn drop(&mut self) {
        if let Some(pgid) = self.pgid {
            unsafe {
                libc::killpg(pgid, libc::SIGKILL);
            }
        }
    }
}

/// Read a child pipe into `sink` (capped at `cap` bytes) until EOF, error, or
/// the cap is reached. Runs as a tokio task; aborting it is clean (no thread).
async fn pump<R>(mut reader: R, sink: Arc<std::sync::Mutex<Vec<u8>>>, cap: usize)
where
    R: tokio::io::AsyncRead + Unpin,
{
    use tokio::io::AsyncReadExt;
    let mut chunk = [0u8; 8192];
    loop {
        match reader.read(&mut chunk).await {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                let mut buf = sink.lock().unwrap();
                if buf.len() >= cap {
                    break;
                }
                let room = cap - buf.len();
                buf.extend_from_slice(&chunk[..n.min(room)]);
                if buf.len() >= cap {
                    break;
                }
            }
        }
    }
}

/// Copy `src` → `dst` on a blocking thread, removing any partially-copied
/// `dst` if the copy fails so a failed snapshot/restore leaves no orphan dir.
/// Returns a stringified error suitable for wrapping in a `RuntimeError`.
async fn copy_tree_async(src: PathBuf, dst: PathBuf) -> Result<(), String> {
    let dst_for_copy = dst.clone();
    let result = tokio::task::spawn_blocking(move || copy_tree(&src, &dst_for_copy)).await;
    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => {
            let _ = tokio::fs::remove_dir_all(&dst).await;
            Err(e.to_string())
        }
        Err(e) => {
            let _ = tokio::fs::remove_dir_all(&dst).await;
            Err(format!("join error: {}", e))
        }
    }
}

/// Recursively copy a directory tree from `src` to `dst`, creating `dst`.
fn copy_tree(src: &Path, dst: &Path) -> std::io::Result<()> {
    // Canonicalize both ends (resolving `.`/`..`/symlinks) so the
    // destination-skip check in copy_tree_inner is reliable: if `dst` lives
    // inside `src`, we must recognize it by exact path to skip *only* it —
    // preventing unbounded recursion into our own output without over-skipping
    // unrelated siblings. `dst` is created first so it can be canonicalized.
    let src = src.canonicalize()?;
    std::fs::create_dir_all(dst)?;
    let dst = dst.canonicalize()?;
    copy_tree_inner(&src, &dst, &dst)
}

fn copy_tree_inner(src: &Path, dst: &Path, dst_root: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        // Skip exactly the destination dir if we encounter it nested in `src`.
        if entry_path == dst_root {
            continue;
        }
        let file_type = entry.file_type()?;
        let target = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_tree_inner(&entry_path, &target, dst_root)?;
        } else if file_type.is_symlink() {
            // Preserve symlinks as-is rather than following them.
            #[cfg(unix)]
            {
                let link = std::fs::read_link(&entry_path)?;
                std::os::unix::fs::symlink(link, &target)?;
            }
        } else {
            std::fs::copy(&entry_path, &target)?;
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
