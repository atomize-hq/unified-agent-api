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
        "#!/bin/sh\nprintf '%s' $$ > child-pid\nexec 0<&-\nexec sleep 30\n",
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

#[cfg(unix)]
#[test]
fn wait_error_after_child_writes_terminates_and_reaps_it() {
    let root = tempfile::tempdir().unwrap();
    let mut child = Command::new("sh")
        .args([
            "-c",
            "printf output > candidate-output; printf r; exec sleep 30",
        ])
        .current_dir(root.path())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let pid = child.id().to_string();
    let mut ready = [0];
    child.stdout.take().unwrap().read_exact(&mut ready).unwrap();
    assert_eq!(
        fs::read(root.path().join("candidate-output")).unwrap(),
        b"output"
    );
    let result = wait_for_codex(&mut child, |_| {
        Err(std::io::Error::other("injected wait failure"))
    });
    assert!(
        matches!(result, Err(Error::Internal(message)) if message.contains("wait for codex exec"))
    );
    assert!(!Command::new("kill")
        .args(["-0", &pid])
        .output()
        .unwrap()
        .status
        .success());
}
