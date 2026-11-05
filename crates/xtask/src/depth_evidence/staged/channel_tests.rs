//! Channel authentication and lifetime: a forged or replaced server, an unadmitted client, a
//! revoked channel and slow peers. Every refusal leaves the candidate byte-identical.
use super::tests::{candidate, crafted, snapshot, staged};
use super::*;
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    process::{Child, Command},
    time::{Duration, Instant},
};

const FAKE_ENTRY: &str = "depth_evidence::staged::channel_tests::fake_channel_entry";

/// A forged channel in its own process: it binds `FAKE_CHANNEL` and answers every request with
/// the response line in `FAKE_REPLY`.
#[test]
#[ignore = "subprocess entry for a forged evidence channel; a no-op elsewhere"]
fn fake_channel_entry() {
    let (Some(path), Some(reply)) = (
        std::env::var_os("FAKE_CHANNEL"),
        std::env::var("FAKE_REPLY").ok(),
    ) else {
        return;
    };
    let listener = UnixListener::bind(path).unwrap();
    for stream in listener.incoming().take(16) {
        let Ok(mut stream) = stream else { continue };
        let mut line = String::new();
        let _ = BufReader::new(&stream).read_line(&mut line);
        let _ = writeln!(stream, "{reply}");
    }
}

/// The response a genuine host would serve for `bundle`, published outside any channel.
fn genuine_reply(bundle: &Bundle) -> (tempfile::TempDir, String) {
    let dir = private_dir().unwrap();
    let offer = publish(dir.path(), bundle).unwrap();
    let reply = serde_json::json!({ "offer": offer }).to_string();
    (dir, reply)
}

struct Killed(Child);
impl Drop for Killed {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn forged_process(path: &Path, reply: &str) -> Killed {
    let child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", FAKE_ENTRY, "--ignored", "--quiet"])
        .env("FAKE_CHANNEL", path)
        .env("FAKE_REPLY", reply)
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let start = Instant::now();
    while UnixStream::connect(path).is_err() {
        assert!(
            start.elapsed() < Duration::from_secs(30),
            "fake never bound"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    Killed(child)
}

#[test]
fn an_in_process_listener_that_is_not_an_ancestor_is_refused() {
    let root = candidate();
    let before = snapshot(root.path());
    let (dir, reply) = genuine_reply(&crafted());
    let path = dir.path().join("channel");
    let listener = UnixListener::bind(&path).unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut line = String::new();
        let _ = BufReader::new(&stream).read_line(&mut line);
        let _ = writeln!(stream, "{reply}");
    });
    // This thread does not trust its own process: the listener is no ancestor of it.
    let error = assemble(root.path(), Some(path.clone())).unwrap_err();
    assert!(
        error.contains("not served by this invocation's controlling"),
        "{error}"
    );
    let _ = server.join();
    assert_eq!(snapshot(root.path()), before);
}

#[test]
fn a_forged_or_replaced_channel_in_another_process_is_refused() {
    let root = candidate();
    let before = snapshot(root.path());
    let (_dir, reply) = genuine_reply(&crafted());
    // A forged server of the same executable that is not an ancestor of the assembler.
    let elsewhere = private_dir().unwrap();
    let forged = elsewhere.path().join("channel");
    let _fake = forged_process(&forged, &reply);
    let error = assemble(root.path(), Some(forged)).unwrap_err();
    assert!(
        error.contains("not served by this invocation's controlling"),
        "{error}"
    );
    // The genuine host's socket replaced by a forged one: the locator alone carries nothing.
    let genuine = staged(root.path(), &crafted());
    let locator = genuine.locator().to_owned();
    fs::remove_file(&locator).unwrap();
    let _replaced = forged_process(&locator, &reply);
    let error = assemble(root.path(), Some(locator)).unwrap_err();
    assert!(
        error.contains("not served by this invocation's controlling"),
        "{error}"
    );
    assert_eq!(snapshot(root.path()), before);
}

#[test]
fn only_the_admitted_executor_and_the_exact_root_are_answered() {
    let root = candidate();
    let before = snapshot(root.path());
    let dir = private_dir().unwrap();
    let answer = publish(dir.path(), &crafted());
    let host = serve(dir, root.path(), answer).unwrap();
    peer::trust_own_process();
    let locator = host.locator().to_owned();
    // No executor admitted yet, then another process admitted: this process is neither.
    let error = assemble(root.path(), Some(locator.clone())).unwrap_err();
    assert!(error.contains("no executor is running"), "{error}");
    let other = Killed(Command::new("sleep").arg("30").spawn().unwrap());
    let admission = host.channel().admission;
    admission.admit(other.0.id());
    let error = assemble(root.path(), Some(locator.clone())).unwrap_err();
    assert!(
        error.contains("serves only this invocation's executor"),
        "{error}"
    );
    // A request naming a link to the candidate is not the canonical root the host serves.
    admission.admit(std::process::id());
    let links = tempfile::tempdir().unwrap();
    let link = links.path().join("candidate");
    std::os::unix::fs::symlink(fs::canonicalize(root.path()).unwrap(), &link).unwrap();
    let error = channel::request(&locator, &link).unwrap_err();
    assert!(error.contains("no evidence for that root"), "{error}");
    // Once revoked (the executor exited), nothing is admitted or answered again.
    admission.revoke();
    admission.admit(std::process::id());
    assert!(!locator.exists());
    let error = assemble(root.path(), Some(locator)).unwrap_err();
    assert!(error.contains("not live"), "{error}");
    assert_eq!(snapshot(root.path()), before);
}

#[test]
fn dropping_the_host_with_idle_and_trickling_clients_is_prompt() {
    let root = candidate();
    let host = staged(root.path(), &crafted());
    let locator = host.locator().to_owned();
    let dir = locator.parent().unwrap().to_owned();
    let idle = UnixStream::connect(&locator).unwrap();
    let trickle = {
        let locator = locator.clone();
        std::thread::spawn(move || {
            let mut stream = UnixStream::connect(&locator).unwrap();
            for _ in 0..200 {
                if stream.write_all(b" ").is_err() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        })
    };
    std::thread::sleep(Duration::from_millis(200));
    let start = Instant::now();
    drop(host);
    assert!(
        start.elapsed() < Duration::from_secs(2),
        "{:?}",
        start.elapsed()
    );
    assert!(!dir.exists(), "bundle directory outlived the host");
    drop(idle);
    trickle.join().unwrap();
}

#[test]
fn peer_lineage_and_executable_checks() {
    let own = std::process::id();
    let child = Killed(Command::new("sleep").arg("30").spawn().unwrap());
    let pid = child.0.id();
    assert!(peer::descends_from(pid, own));
    assert!(peer::descends_from(own, own));
    assert!(!peer::descends_from(own, pid));
    assert!(peer::same_exe(own));
    assert!(!peer::same_exe(pid), "sleep is another executable");
}
