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
        None,
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

#[cfg(unix)]
const SCRUB_ENTRY: &str = "agent_maintenance::execute::runtime::tests::credential_scrub_entry";

/// Runs one executor spawn and one real green gate, each dumping its environment.
#[cfg(unix)]
#[test]
#[ignore = "subprocess entry for the credential scrub test; a no-op elsewhere"]
fn credential_scrub_entry() {
    use std::os::unix::fs::PermissionsExt;
    let Some(out) = std::env::var_os("UAA_SCRUB_OUT").map(std::path::PathBuf::from) else {
        return;
    };
    let executor = out.join("executor.sh");
    fs::write(&executor, "#!/bin/sh\ncat >/dev/null\nenv > executor.env\n").unwrap();
    fs::set_permissions(&executor, fs::Permissions::from_mode(0o755)).unwrap();
    let (_, output) = spawn_codex_exec(executor.to_str().unwrap(), &out, "p", &[], None).unwrap();
    assert!(output.status.success());
    let (mut checks, mut errors) = (vec![], vec![]);
    run_green_gates(&out, &["env > gate.env".into()], &mut checks, &mut errors).unwrap();
    assert!(errors.is_empty(), "{errors:?}");
}

#[cfg(unix)]
#[test]
fn host_credentials_never_reach_the_executor_or_green_gates() {
    let out = tempfile::tempdir().unwrap();
    let mut entry = Command::new(std::env::current_exe().unwrap());
    entry
        .args(["--exact", SCRUB_ENTRY, "--ignored", "--quiet"])
        .env("UAA_SCRUB_OUT", out.path())
        .env("UAA_SCRUB_CONTROL", "inherited");
    for key in HOST_CREDENTIAL_ENV {
        entry.env(key, format!("uaa-sentinel-{key}"));
    }
    let status = entry.stdout(Stdio::null()).status().unwrap();
    assert!(status.success());
    for phase in ["executor.env", "gate.env"] {
        let env = fs::read_to_string(out.path().join(phase)).unwrap();
        // Ordinary inheritance (the executor's own home and login resolution) is untouched.
        assert!(env.contains("UAA_SCRUB_CONTROL=inherited"), "{phase}");
        assert!(env.lines().any(|l| l.starts_with("HOME=")), "{phase}");
        for key in HOST_CREDENTIAL_ENV {
            assert!(
                !env.lines().any(|l| l.starts_with(&format!("{key}="))),
                "{key} reached {phase}"
            );
        }
        assert!(!env.contains("uaa-sentinel"), "{phase}");
    }
}
