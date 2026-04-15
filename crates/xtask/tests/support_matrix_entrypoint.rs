use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("CARGO_MANIFEST_DIR has crates/<crate> parent structure")
        .to_path_buf()
}

#[test]
fn support_matrix_entrypoint_exposes_reserved_command_contract() {
    let xtask_bin = PathBuf::from(env!("CARGO_BIN_EXE_xtask"));
    let workdir = repo_root().join("crates").join("xtask");

    let help = Command::new(&xtask_bin)
        .arg("--help")
        .current_dir(&workdir)
        .output()
        .expect("spawn xtask --help");
    assert!(
        help.status.success(),
        "xtask --help failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&help.stdout),
        String::from_utf8_lossy(&help.stderr)
    );
    let help_text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&help.stdout),
        String::from_utf8_lossy(&help.stderr)
    );
    assert!(
        help_text.contains("support-matrix"),
        "root help must list support-matrix:\n{help_text}"
    );
    assert!(
        help_text.contains("capability-matrix"),
        "root help must still list capability-matrix:\n{help_text}"
    );
    assert!(
        help_text.contains("capability-matrix-audit"),
        "root help must still list capability-matrix-audit:\n{help_text}"
    );

    let sub_help = Command::new(&xtask_bin)
        .arg("support-matrix")
        .arg("--help")
        .current_dir(&workdir)
        .output()
        .expect("spawn xtask support-matrix --help");
    assert!(
        sub_help.status.success(),
        "xtask support-matrix --help failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&sub_help.stdout),
        String::from_utf8_lossy(&sub_help.stderr)
    );
    let sub_help_text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&sub_help.stdout),
        String::from_utf8_lossy(&sub_help.stderr)
    );
    assert!(
        sub_help_text.contains("Reserve the neutral support-matrix publication entrypoint"),
        "support-matrix help text must reflect the reserved publication contract:\n{sub_help_text}"
    );

    let run = Command::new(&xtask_bin)
        .arg("support-matrix")
        .current_dir(&workdir)
        .output()
        .expect("spawn xtask support-matrix");
    assert!(
        !run.status.success(),
        "support-matrix should be reserved-only for now"
    );
    let run_stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        run_stderr.contains("reserved for later implementation"),
        "support-matrix dispatch should return the reserved placeholder message:\n{run_stderr}"
    );
}
