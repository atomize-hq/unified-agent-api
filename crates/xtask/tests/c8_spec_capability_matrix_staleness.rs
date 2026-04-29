use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("CARGO_MANIFEST_DIR has crates/<crate> parent structure")
        .to_path_buf()
}

fn make_temp_dir(prefix: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after unix epoch");
    let unique = format!("{}-{}-{}", prefix, std::process::id(), now.as_nanos());
    let dir = std::env::temp_dir().join(unique);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn write_text(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write fixture file");
}

fn copy_repo_file(fixture_root: &Path, relative_path: &str) {
    let source = repo_root().join(relative_path);
    let destination = fixture_root.join(relative_path);
    let contents = fs::read(&source).unwrap_or_else(|err| panic!("read {source:?}: {err}"));
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).expect("create destination parent dirs");
    }
    fs::write(&destination, contents)
        .unwrap_or_else(|err| panic!("write {destination:?} from {source:?}: {err}"));
}

fn seed_fixture_workspace(fixture_root: &Path) {
    write_text(
        &fixture_root.join("Cargo.toml"),
        "[workspace]\nmembers = []\n",
    );
    copy_repo_file(fixture_root, "crates/xtask/data/agent_registry.toml");
    copy_repo_file(fixture_root, "cli_manifests/codex/current.json");
    copy_repo_file(fixture_root, "cli_manifests/claude_code/current.json");
    copy_repo_file(fixture_root, "cli_manifests/opencode/current.json");
    copy_repo_file(fixture_root, "cli_manifests/gemini_cli/current.json");
    copy_repo_file(fixture_root, "cli_manifests/aider/current.json");
}

#[test]
fn c8_spec_capability_matrix_check_passes_when_fresh_and_fails_without_rewriting_when_stale() {
    let xtask_bin = PathBuf::from(env!("CARGO_BIN_EXE_xtask"));
    let fixture_root = make_temp_dir("capability-matrix-staleness");
    seed_fixture_workspace(&fixture_root);

    let checked_in = fixture_root
        .join("docs")
        .join("specs")
        .join("unified-agent-api")
        .join("capability-matrix.md");

    let generate = Command::new(&xtask_bin)
        .arg("capability-matrix")
        .current_dir(&fixture_root)
        .output()
        .expect("spawn xtask capability-matrix");

    let stdout = String::from_utf8_lossy(&generate.stdout);
    let stderr = String::from_utf8_lossy(&generate.stderr);
    assert!(
        generate.status.success(),
        "capability-matrix generation failed\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );

    let baseline = fs::read_to_string(&checked_in).expect("read generated capability matrix");
    assert!(
        baseline.contains("# Capability matrix"),
        "generated capability matrix missing title:\n{baseline}"
    );

    let fresh_check = Command::new(&xtask_bin)
        .arg("capability-matrix")
        .arg("--check")
        .current_dir(&fixture_root)
        .output()
        .expect("spawn xtask capability-matrix --check");
    assert!(
        fresh_check.status.success(),
        "capability-matrix --check should pass on fresh generated file\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&fresh_check.stdout),
        String::from_utf8_lossy(&fresh_check.stderr)
    );

    let mutated = baseline.replacen(
        "| `agent_api.run` |",
        "| `agent_api.run.local-mutation` |",
        1,
    );
    fs::write(&checked_in, &mutated).expect("write deliberate local mutation");

    let stale_check = Command::new(&xtask_bin)
        .arg("capability-matrix")
        .arg("--check")
        .current_dir(&fixture_root)
        .output()
        .expect("spawn stale xtask capability-matrix --check");
    assert!(
        !stale_check.status.success(),
        "capability-matrix --check should fail on a deliberate local mutation\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&stale_check.stdout),
        String::from_utf8_lossy(&stale_check.stderr)
    );

    let stale_stderr = String::from_utf8_lossy(&stale_check.stderr);
    assert!(stale_stderr.contains("capability-matrix.md is stale"));
    assert!(stale_stderr.contains("cargo run -p xtask -- capability-matrix"));

    let post_check = fs::read_to_string(&checked_in).expect("read capability matrix after check");
    assert_eq!(
        post_check, mutated,
        "--check must not rewrite the checked-in capability matrix"
    );
}
