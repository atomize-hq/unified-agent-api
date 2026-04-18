use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn make_temp_dir(prefix: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after unix epoch");
    let dir = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        now.as_nanos()
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn write_text(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write fixture file");
}

fn fixture_workspace(root: &Path, agent_api_src: &str) {
    write_text(
        &root.join("Cargo.toml"),
        r#"[workspace]
members = []
"#,
    );
    write_text(
        &root.join("crates/agent_api/Cargo.toml"),
        r#"[package]
name = "unified-agent-api"
version = "0.1.0"
edition = "2021"

[features]
default = []
codex = ["dep:codex"]
claude_code = ["dep:claude_code"]
opencode = ["dep:opencode"]

[dependencies]
codex = { path = "../codex", optional = true }
claude_code = { path = "../claude_code", optional = true }
opencode = { path = "../opencode", optional = true }
serde = "1"
"#,
    );
    write_text(&root.join("crates/agent_api/src/lib.rs"), agent_api_src);
}

#[test]
fn backend_type_leak_guard_passes_when_public_api_is_clean() {
    let xtask_bin = PathBuf::from(env!("CARGO_BIN_EXE_xtask"));
    let root = make_temp_dir("backend-type-leak-guard-pass");
    fixture_workspace(
        &root,
        r#"
pub struct PublicType;

fn internal() -> opencode::Client {
    todo!()
}
"#,
    );

    let output = Command::new(&xtask_bin)
        .arg("agent-api-backend-type-leak-guard")
        .current_dir(&root)
        .output()
        .expect("spawn xtask backend type leak guard");

    assert!(
        output.status.success(),
        "guard should pass\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn backend_type_leak_guard_fails_for_public_signature_leak() {
    let xtask_bin = PathBuf::from(env!("CARGO_BIN_EXE_xtask"));
    let root = make_temp_dir("backend-type-leak-guard-signature");
    fixture_workspace(
        &root,
        r#"
pub fn leaked() -> opencode::Client {
    todo!()
}
"#,
    );

    let output = Command::new(&xtask_bin)
        .arg("agent-api-backend-type-leak-guard")
        .current_dir(&root)
        .output()
        .expect("spawn xtask backend type leak guard");

    assert!(!output.status.success(), "signature leak should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("derived backend ids: [claude_code, codex, opencode]"));
    assert!(stderr.contains("pub signature"));
    assert!(stderr.contains("opencode::Client"));
}

#[test]
fn backend_type_leak_guard_fails_for_public_reexport_leak() {
    let xtask_bin = PathBuf::from(env!("CARGO_BIN_EXE_xtask"));
    let root = make_temp_dir("backend-type-leak-guard-reexport");
    fixture_workspace(
        &root,
        r#"
pub use opencode::Client;
"#,
    );

    let output = Command::new(&xtask_bin)
        .arg("agent-api-backend-type-leak-guard")
        .current_dir(&root)
        .output()
        .expect("spawn xtask backend type leak guard");

    assert!(!output.status.success(), "re-export leak should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("pub re-export"));
    assert!(stderr.contains("pub use opencode::Client;"));
}
