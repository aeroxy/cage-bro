use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use uuid::Uuid;

pub struct ShellSession {
    pub id: String,
    pub pty_writer: Mutex<Box<dyn std::io::Write + Send>>,
    pub output_tx: broadcast::Sender<Vec<u8>>,
    pub child: Mutex<Box<dyn portable_pty::Child + Send>>,
}

pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, Arc<ShellSession>>>>,
    workspace: String,
}

impl SessionManager {
    pub fn new(workspace: String) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            workspace,
        }
    }

    pub async fn create_session(
        &self,
        shell: Option<&str>,
        cols: u16,
        rows: u16,
    ) -> Result<String, String> {
        let pty_system = native_pty_system();
        let pty = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("PTY open failed: {}", e))?;

        let shell_cmd = shell.unwrap_or(if cfg!(target_os = "macos") {
            "/bin/zsh"
        } else {
            "/bin/bash"
        });

        let mut builder = CommandBuilder::new(shell_cmd);
        builder.env("TERM", "xterm-256color");
        builder.cwd(&self.workspace);

        let child = pty
            .slave
            .spawn_command(builder)
            .map_err(|e| format!("Spawn failed: {}", e))?;

        let mut reader = pty
            .master
            .try_clone_reader()
            .map_err(|e| format!("Reader clone failed: {}", e))?;

        let writer = pty
            .master
            .take_writer()
            .map_err(|e| format!("Writer take failed: {}", e))?;

        let (output_tx, _) = broadcast::channel::<Vec<u8>>(4096);
        let output_tx_clone = output_tx.clone();

        let session_id = Uuid::new_v4().to_string();

        // Spawn reader task that pumps PTY output into broadcast channel
        let session_id_for_task = session_id.clone();
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let _ = output_tx_clone.send(buf[..n].to_vec());
                    }
                    Err(_) => break,
                }
            }
            tracing::info!(session_id = %session_id_for_task, "PTY reader exited");
        });

        let session = Arc::new(ShellSession {
            id: session_id.clone(),
            pty_writer: Mutex::new(writer),
            output_tx,
            child: Mutex::new(child),
        });

        let mut sessions = self.sessions.lock().await;
        sessions.insert(session_id.clone(), session);

        tracing::info!(session_id = %session_id, shell = shell_cmd, workspace = %self.workspace, "Shell session created");
        Ok(session_id)
    }

    pub async fn write_input(&self, session_id: &str, data: &[u8]) -> Result<(), String> {
        let sessions = self.sessions.lock().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        let mut writer = session.pty_writer.lock().await;
        std::io::Write::write_all(&mut *writer, data)
            .map_err(|e| format!("Write failed: {}", e))?;
        std::io::Write::flush(&mut *writer)
            .map_err(|e| format!("Flush failed: {}", e))?;

        Ok(())
    }

    pub async fn subscribe(&self, session_id: &str) -> Result<broadcast::Receiver<Vec<u8>>, String> {
        let sessions = self.sessions.lock().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        Ok(session.output_tx.subscribe())
    }

    pub async fn close(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.remove(session_id) {
            let mut child = session.child.lock().await;
            let _ = child.kill();
            tracing::info!(session_id = %session_id, "Session closed");
            Ok(())
        } else {
            Err(format!("Session not found: {}", session_id))
        }
    }

    pub async fn list(&self) -> Vec<String> {
        let sessions = self.sessions.lock().await;
        sessions.keys().cloned().collect()
    }
}
