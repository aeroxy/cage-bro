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

// A descendant that outlives the direct child and keeps stdout open must not
// make exec hang: once the direct child exits we drain briefly, then stop.
#[tokio::test]
async fn exec_does_not_hang_on_descendant_holding_stdout() {
    let rt = ProcessRuntime::new();
    let sb = rt.create(SandboxConfig::default()).await.unwrap();
    // `sh` backgrounds a 5s sleep (inheriting stdout), prints, then exits.
    let cmd = ExecCommand {
        program: "sh".into(),
        args: vec!["-c".into(), "sleep 5 & echo done".into()],
        env: HashMap::new(),
        working_dir: None,
        timeout_ms: None,
    };
    let start = std::time::Instant::now();
    let r = rt.exec(&sb, cmd).await.unwrap();
    assert!(
        start.elapsed().as_secs() < 3,
        "exec hung waiting on a descendant-held pipe ({:?})",
        start.elapsed()
    );
    assert!(r.stdout.contains("done"), "stdout: {:?}", r.stdout);
}

// When the snapshot store lives *inside* the workspace being snapshotted (dst
// nested in src), copy must terminate (no infinite recursion) and still capture
// sibling files — not skip the whole subtree.
#[tokio::test]
async fn snapshot_when_dest_nested_in_source() {
    let tmp = std::env::temp_dir().join(format!("cagenest_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join("keep.txt"), b"data").unwrap();

    // base_dir == workspace, so dst (base/.cage-bro/snapshots/<id>) is inside src.
    let rt = ProcessRuntime::with_base_dir(&tmp);
    let cfg = SandboxConfig {
        workspace_dir: Some(tmp.to_string_lossy().to_string()),
        ..SandboxConfig::default()
    };
    let sb = rt.create(cfg).await.unwrap();

    // Must complete quickly — a regression here would recurse without bound.
    let snap = tokio::time::timeout(std::time::Duration::from_secs(10), rt.snapshot(&sb))
        .await
        .expect("snapshot did not terminate (recursion regression?)")
        .unwrap();

    let snap_dir = tmp.join(".cage-bro/snapshots").join(snap.id.to_string());
    assert!(snap_dir.join("keep.txt").exists(), "sibling file should be captured");

    std::fs::remove_dir_all(&tmp).ok();
}

// destroy() cleans up a restored workspace, but must NOT touch a regular
// (non-restored) workspace dir.
#[tokio::test]
async fn destroy_cleans_restored_dir_only() {
    let tmp = std::env::temp_dir().join(format!("cagedestroy_{}", uuid::Uuid::new_v4()));
    let ws = tmp.join("ws");
    std::fs::create_dir_all(&ws).unwrap();
    std::fs::write(ws.join("f.txt"), b"x").unwrap();

    let rt = ProcessRuntime::with_base_dir(&tmp);
    let cfg = SandboxConfig {
        workspace_dir: Some(ws.to_string_lossy().to_string()),
        ..SandboxConfig::default()
    };
    let original = rt.create(cfg).await.unwrap();
    let snap = rt.snapshot(&original).await.unwrap();
    let restored = rt.restore(&snap).await.unwrap();
    let restored_ws = restored.config.workspace_dir.clone().unwrap();
    assert!(std::path::Path::new(&restored_ws).exists());

    // Destroying the restored sandbox removes its dir under .cage-bro/restored.
    rt.destroy(&restored).await.unwrap();
    assert!(!std::path::Path::new(&restored_ws).exists(), "restored dir should be cleaned");

    // Destroying the original must NOT delete its (non-restored) workspace.
    rt.destroy(&original).await.unwrap();
    assert!(ws.join("f.txt").exists(), "regular workspace must be left intact");

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
