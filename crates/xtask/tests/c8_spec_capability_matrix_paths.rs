use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use xtask::agent_registry::{AgentRegistry, AgentRegistryError};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("CARGO_MANIFEST_DIR has crates/<crate> parent structure")
        .to_path_buf()
}

/// The executable resolves its workspace at build time. The ordinary leg builds from an undeclared
/// registry; the depth-declared test leg (`scripts/s10_declared_test_leg.py`) builds from a
/// declared one, where these unrouted production commands must refuse instead. The production
/// parser on the build workspace registry selects the expectation; no flag is consulted.
fn declared_build_refusal() -> Option<String> {
    match AgentRegistry::load(&repo_root()) {
        Ok(_) => None,
        Err(AgentRegistryError::Validation(message))
            if message.ends_with(" carries `support_depth`, which no command enforces yet") =>
        {
            Some(message)
        }
        Err(err) => panic!("load build workspace registry: {err}"),
    }
}

/// Ordinary leg: the command succeeds outside the repository root. Declared leg: it reaches the
/// specific production refusal (not a path-resolution failure).
fn assert_ordinary_success_or_declared_refusal(
    label: &str,
    workdir: &std::path::Path,
    output: &std::process::Output,
    declared_refusal: impl FnOnce(&str) -> String,
) -> bool {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    match declared_build_refusal() {
        None => {
            assert!(
                output.status.success(),
                "{label} from {workdir:?} failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
            );
            true
        }
        Some(message) => {
            let expected = declared_refusal(&message);
            assert!(
                !output.status.success(),
                "{label} from {workdir:?} must refuse a declared build registry\nstdout:\n{stdout}"
            );
            assert!(
                stderr.contains(&expected),
                "{label} from {workdir:?}: expected {expected:?}\nstderr:\n{stderr}"
            );
            false
        }
    }
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

#[test]
fn c8_spec_capability_matrix_succeeds_outside_repo_root() {
    let xtask_bin = PathBuf::from(env!("CARGO_BIN_EXE_xtask"));
    let workdir = repo_root().join("crates").join("xtask");
    let temp_dir = make_temp_dir("capability-matrix");
    let out = temp_dir.join("matrix.md");

    let output = Command::new(&xtask_bin)
        .arg("capability-matrix")
        .arg("--out")
        .arg(&out)
        .current_dir(&workdir)
        .output()
        .expect("spawn xtask capability-matrix");

    let ordinary =
        assert_ordinary_success_or_declared_refusal("capability-matrix", &workdir, &output, |_| {
            "capability-matrix: unsupported depth write (declared enrollment)".to_string()
        });
    if !ordinary {
        assert!(
            !out.exists(),
            "refused capability-matrix must not write {out:?}"
        );
        return;
    }

    let markdown = fs::read_to_string(&out).expect("read generated capability matrix");
    assert!(
        markdown.contains("# Capability matrix"),
        "generated markdown missing title:\n{markdown}"
    );
}

#[test]
fn c8_spec_capability_matrix_help_lists_check_flag() {
    let xtask_bin = PathBuf::from(env!("CARGO_BIN_EXE_xtask"));
    let workdir = repo_root().join("crates").join("xtask");

    let output = Command::new(&xtask_bin)
        .arg("capability-matrix")
        .arg("--help")
        .current_dir(&workdir)
        .output()
        .expect("spawn xtask capability-matrix --help");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "capability-matrix --help from {:?} failed\nstdout:\n{}\nstderr:\n{}",
        workdir,
        stdout,
        stderr
    );

    let help_text = format!("{stdout}\n{stderr}");
    assert!(help_text.contains("--check"));
    assert!(help_text.contains("Verify the checked-in capability matrix"));
}

#[test]
fn c8_spec_capability_matrix_check_succeeds_outside_repo_root() {
    let xtask_bin = PathBuf::from(env!("CARGO_BIN_EXE_xtask"));
    let workdir = repo_root().join("crates").join("xtask");

    let output = Command::new(&xtask_bin)
        .arg("capability-matrix")
        .arg("--check")
        .current_dir(&workdir)
        .output()
        .expect("spawn xtask capability-matrix --check");

    assert_ordinary_success_or_declared_refusal(
        "capability-matrix --check",
        &workdir,
        &output,
        |message| message.to_string(),
    );
}

#[test]
fn c8_spec_capability_matrix_audit_succeeds_outside_repo_root() {
    let xtask_bin = PathBuf::from(env!("CARGO_BIN_EXE_xtask"));
    let workdir = repo_root().join("crates").join("xtask");

    let output = Command::new(&xtask_bin)
        .arg("capability-matrix-audit")
        .current_dir(&workdir)
        .output()
        .expect("spawn xtask capability-matrix-audit");

    assert_ordinary_success_or_declared_refusal(
        "capability-matrix-audit",
        &workdir,
        &output,
        |message| message.to_string(),
    );
}
