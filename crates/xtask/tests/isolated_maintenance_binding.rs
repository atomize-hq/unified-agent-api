use std::{fs, process::Command};

#[test]
fn uninstalled_probe_refuses_before_host_reads_and_preserves_workspace() {
    let root = tempfile::tempdir().unwrap();
    let tools = tempfile::tempdir().unwrap();
    let marker = root.path().join("host-called");
    let sentinel = tools.path().join("gh");
    fs::write(
        &sentinel,
        "#!/bin/sh\nprintf called > \"$HOST_SENTINEL\"\nexit 99\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&sentinel, fs::Permissions::from_mode(0o755)).unwrap();
    }
    fs::write(root.path().join("caller-input"), b"preserve me").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("isolated-maintenance-binding-check")
        .current_dir(root.path())
        .env("PATH", tools.path())
        .env("HOST_SENTINEL", &marker)
        .env("GH_REPO", "forged/isolated")
        .env("XTASK_ISOLATED_PROFILE", "approved")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("isolated maintenance profile is not installed"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!marker.exists());
    assert_eq!(
        fs::read(root.path().join("caller-input")).unwrap(),
        b"preserve me"
    );
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
}

#[test]
fn probe_accepts_no_host_trust_root_or_write_selector() {
    for argument in ["--repo", "--root", "--trust", "--write", "--profile"] {
        let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(["isolated-maintenance-binding-check", argument, "forged"])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("unexpected argument"));
    }
}
