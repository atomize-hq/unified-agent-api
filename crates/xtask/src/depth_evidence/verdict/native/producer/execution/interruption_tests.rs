use super::*;
fn kill(signal: &str, pid: u32) {
    let _ = Command::new("/usr/bin/kill")
        .env_clear()
        .args(["--signal", signal, "--"])
        .arg(pid.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
fn running(pid: u32) -> bool {
    fs::read_to_string(format!("/proc/{pid}/stat"))
        .ok()
        .and_then(|s| s.rsplit_once(") ").map(|(_, tail)| !tail.starts_with("Z ")))
        .unwrap_or(false)
}
struct Cleanup {
    owner: std::process::Child,
    root: PathBuf,
}
impl Drop for Cleanup {
    fn drop(&mut self) {
        kill("TERM", self.owner.id());
        if let Ok(raw) = fs::read_to_string(self.root.join("collector.pid")) {
            if let Ok(pid) = raw.trim().parse::<u32>() {
                let _ = Command::new("/usr/bin/kill")
                    .env_clear()
                    .args(["--signal", "KILL", "--"])
                    .arg(format!("-{pid}"))
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
        }
        let _ = self.owner.kill();
        let _ = self.owner.wait();
    }
}
#[test]
fn interrupt_owner_fixture() {
    let Some(root) = std::env::var_os("UAA_NATIVE_INTERRUPT_FIXTURE").map(PathBuf::from) else {
        return;
    };
    let interrupts = super::super::control::Interrupts::new().unwrap();
    fs::create_dir(root.join("execution")).unwrap();
    let requested = requested(&"a".repeat(64));
    let step = form::NativeStep {
        case: requested[0].clone(),
        observation: form::NativeObservation::default(),
        outcome: CaseOutcome::Failed,
        elapsed_ms: 1,
        failure: Some(form::NativeFailure::RequestRefused),
    };
    let script = r#"import os, pathlib, subprocess, sys
root=pathlib.Path(sys.argv[1])
(root/'case-0.json').write_text(sys.argv[2])
child=subprocess.Popen(['/bin/sleep','60'])
(root/'collector.pid').write_text(str(os.getpid()))
(root/'descendant.pid').write_text(str(child.pid))
(root/'ready').write_text('ready')
child.wait()
"#;
    let mut command = Command::new("python3");
    command
        .args(["-c", script])
        .arg(root.join("execution"))
        .arg(String::from_utf8(canonical(&step).unwrap()).unwrap())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    use std::os::unix::process::CommandExt;
    command.process_group(0);
    let status = wait_owned(command, interrupts.flag(), Duration::from_secs(30)).unwrap();
    assert!(status.is_none());
    assert!(super::super::control::check(interrupts.flag()).is_err());
    // Retain the completed finite prefix even though the owner was interrupted.
    let (steps, valid) = read_steps(&root, "execution", &requested);
    assert!(valid);
    assert_eq!(steps, vec![step]);
    fs::write(
        root.join("retained.json"),
        canonical(&project(&requested, "execution", &steps).unwrap()).unwrap(),
    )
    .unwrap();
}
#[test]
fn sigint_and_sigterm_kill_owned_descendants_and_keep_prior_failure() {
    for signal in ["INT", "TERM"] {
        let tmp = tempfile::tempdir().unwrap();
        let mut c = Command::new(std::env::current_exe().unwrap());
        c.args(["--exact","depth_evidence::verdict::native::producer::execution::interruption_tests::interrupt_owner_fixture","--nocapture"]).env("UAA_NATIVE_INTERRUPT_FIXTURE",tmp.path()).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        let mut cleanup = Cleanup {
            owner: c.spawn().unwrap(),
            root: tmp.path().join("execution"),
        };
        let deadline = std::time::Instant::now() + Duration::from_secs(20);
        while !tmp.path().join("execution/ready").is_file() {
            assert!(
                cleanup.owner.try_wait().unwrap().is_none(),
                "fixture exited before explicit ready handshake"
            );
            assert!(
                std::time::Instant::now() < deadline,
                "fixture readiness timed out"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        let collector = fs::read_to_string(tmp.path().join("execution/collector.pid"))
            .unwrap()
            .parse::<u32>()
            .unwrap();
        let descendant = fs::read_to_string(tmp.path().join("execution/descendant.pid"))
            .unwrap()
            .parse::<u32>()
            .unwrap();
        assert!(running(collector) && running(descendant));
        kill(signal, cleanup.owner.id());
        assert!(
            cleanup
                .owner
                .wait_timeout(Duration::from_secs(20))
                .unwrap()
                .is_some_and(|s| s.success()),
            "interrupted owner did not finish cleanup and retention"
        );
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while running(collector) || running(descendant) {
            assert!(
                std::time::Instant::now() < deadline,
                "owned descendant survived interruption"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        let cases: Cases =
            serde_json::from_slice(&fs::read(tmp.path().join("retained.json")).unwrap()).unwrap();
        assert_eq!(cases.requested.len(), 12);
        assert_eq!(cases.executed.len(), 1);
        assert_eq!(cases.executed[0].outcome, CaseOutcome::Failed);
    }
}
