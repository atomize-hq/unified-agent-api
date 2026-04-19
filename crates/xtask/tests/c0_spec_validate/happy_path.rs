use super::*;

#[test]
fn c0_validate_passes_on_minimal_valid_codex_dir() {
    let temp = make_temp_dir("ccm-c0-validate-pass");
    let codex_dir = materialize_minimal_valid_workspace(&temp);

    let output = run_xtask_validate(&codex_dir);
    assert!(
        output.status.success(),
        "expected success:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn c0_validate_passes_on_standalone_codex_dir_without_workspace_support_matrix() {
    let temp = make_temp_dir("ccm-c0-validate-standalone-pass");
    let codex_dir = temp.join("cli_manifests").join("codex");
    materialize_minimal_valid_codex_dir(&codex_dir);

    let output = run_xtask_validate(&codex_dir);
    assert!(
        output.status.success(),
        "expected standalone success without workspace sibling layout:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
