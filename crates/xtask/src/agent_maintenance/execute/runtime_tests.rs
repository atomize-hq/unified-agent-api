use super::*;
#[cfg(unix)]
use std::fs;
#[cfg(unix)]
#[test]
fn prompt_pipe_error_terminates_and_reaps_started_child() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    let binary = root.path().join("close-stdin.sh");
    fs::write(
        &binary,
        "#!/bin/sh\nexec 0<&-\nprintf '%s' $$ > child-pid\nexec sleep 30\n",
    )
    .unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
    let result = spawn_codex_exec(
        binary.to_str().unwrap(),
        root.path(),
        &"x".repeat(1_048_576),
        &[],
        false,
    );
    assert!(result.is_err());
    let pid = fs::read_to_string(root.path().join("child-pid")).unwrap();
    let alive = Command::new("kill").args(["-0", &pid]).output().unwrap();
    assert!(
        !alive.status.success(),
        "child still running after failed prompt delivery"
    );
}
