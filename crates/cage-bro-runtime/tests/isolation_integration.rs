use cage_bro_runtime::{ProcessRuntime, SandboxRuntime, SandboxConfig, ExecCommand};
use std::collections::HashMap;

#[tokio::test]
async fn exec_captures_stderr_separately() {
    let rt = ProcessRuntime::new();
    let sb = rt.create(SandboxConfig::default()).await.unwrap();
    let cmd = ExecCommand {
        program: "sh".into(),
        args: vec!["-c".into(), "echo out; echo err 1>&2; exit 3".into()],
        env: HashMap::new(), working_dir: None, timeout_ms: None,
    };
    let r = rt.exec(&sb, cmd).await.unwrap();
    assert_eq!(r.exit_code, 3, "exit code");
    assert!(r.stdout.contains("out"), "stdout: {:?}", r.stdout);
    assert!(r.stderr.contains("err"), "stderr separate: {:?}", r.stderr);
}

#[tokio::test]
async fn exec_enforces_timeout() {
    let rt = ProcessRuntime::new();
    let sb = rt.create(SandboxConfig::default()).await.unwrap();
    let cmd = ExecCommand {
        program: "sh".into(),
        args: vec!["-c".into(), "sleep 10".into()],
        env: HashMap::new(), working_dir: None, timeout_ms: Some(300),
    };
    let start = std::time::Instant::now();
    let r = rt.exec(&sb, cmd).await.unwrap();
    assert!(start.elapsed().as_secs() < 3, "should have been killed quickly");
    assert!(r.stderr.contains("timeout"), "stderr: {:?}", r.stderr);
}

#[tokio::test]
async fn snapshot_restore_roundtrip() {
    let tmp = std::env::temp_dir().join(format!("cagetest_{}", uuid::Uuid::new_v4()));
    let ws = tmp.join("ws");
    std::fs::create_dir_all(&ws).unwrap();
    std::fs::write(ws.join("hello.txt"), b"v1").unwrap();

    let rt = ProcessRuntime::with_base_dir(&tmp);
    let mut cfg = SandboxConfig::default();
    cfg.workspace_dir = Some(ws.to_string_lossy().to_string());
    let sb = rt.create(cfg).await.unwrap();

    let snap = rt.snapshot(&sb).await.unwrap();
    // mutate original after snapshot
    std::fs::write(ws.join("hello.txt"), b"v2-mutated").unwrap();

    let restored = rt.restore(&snap).await.unwrap();
    let rws = restored.config.workspace_dir.clone().unwrap();
    let content = std::fs::read_to_string(format!("{}/hello.txt", rws)).unwrap();
    assert_eq!(content, "v1", "restored snapshot should hold pre-mutation state");

    std::fs::remove_dir_all(&tmp).ok();
}

#[tokio::test]
async fn restore_preserves_non_default_config() {
    let tmp = std::env::temp_dir().join(format!("cagecfg_{}", uuid::Uuid::new_v4()));
    let ws = tmp.join("ws");
    std::fs::create_dir_all(&ws).unwrap();

    let rt = ProcessRuntime::with_base_dir(&tmp);
    let cfg = SandboxConfig {
        memory_limit_mb: Some(256),
        cpu_limit_percent: Some(25),
        network_enabled: false,
        workspace_dir: Some(ws.to_string_lossy().to_string()),
        ..SandboxConfig::default()
    };
    let sb = rt.create(cfg).await.unwrap();
    let snap = rt.snapshot(&sb).await.unwrap();
    let restored = rt.restore(&snap).await.unwrap();

    // Non-default resource/network settings survive the round-trip.
    assert_eq!(restored.config.memory_limit_mb, Some(256));
    assert_eq!(restored.config.cpu_limit_percent, Some(25));
    assert!(!restored.config.network_enabled);
    // ...but the workspace is repointed to the freshly restored dir.
    assert_ne!(restored.config.workspace_dir, sb.config.workspace_dir);

    std::fs::remove_dir_all(&tmp).ok();
}
