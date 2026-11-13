//! The channel under the host-side guarantee. A forged server can make the assembler write E in
//! a standalone checkout (the assembler does not authenticate its server; delivery's byte binding
//! refuses such E, see `isolated_maintenance::tests::ma4_evidence`). The host answers only its
//! executor's process tree presenting the invocation nonce, for its exact root, until revoked;
//! slow peers never hold it.
use super::tests::{candidate, crafted, crafted_for, in_process, snapshot, staged};
use super::*;
use std::{
    io::{BufRead, BufReader, Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    process::{Child, Command},
    time::{Duration, Instant},
};

const FAKE_ENTRY: &str = "depth_evidence::staged::channel_tests::fake_channel_entry";

/// A forged channel in its own process: it binds `FAKE_CHANNEL` and answers every request with
/// the response line in `FAKE_REPLY`, whatever the request carries.
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
    for stream in listener.incoming().take(64) {
        let Ok(mut stream) = stream else { continue };
        let mut line = String::new();
        let _ = BufReader::new(&stream).read_line(&mut line);
        let _ = writeln!(stream, "{reply}");
    }
}

/// A forged server in another process, serving a self-consistent bundle for `record` whose
/// receipt is `receipt`. Dropping it stops the server and removes its bundle.
pub(crate) struct Forged(Child, #[allow(dead_code)] tempfile::TempDir);
impl Drop for Forged {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
pub(crate) fn forged_channel(path: &Path, record: &DepthRecord, receipt: &[u8]) -> Forged {
    let dir = private_dir().unwrap();
    let offer = publish(dir.path(), &crafted_for(record, receipt)).unwrap();
    let reply = serde_json::json!({ "offer": offer }).to_string();
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
    Forged(child, dir)
}
struct Killed(Child);
impl Drop for Killed {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn a_forged_channel_can_write_only_a_local_e() {
    // The assembler's channel use is a fail-closed convenience, not the authority: a forged or
    // replaced server, even with a made-up nonce, gets a standalone checkout's E written.
    let root = candidate();
    let elsewhere = private_dir().unwrap();
    let path = elsewhere.path().join("channel");
    let _forged = forged_channel(&path, &super::tests::frozen(), b"{\"forged\":true}");
    let summary = assemble(root.path(), Some(path), "made-up").unwrap();
    assert!(summary.contains("stays untrusted"), "{summary}");
    let record = fs::read_to_string(root.path().join(super::tests::RECORD)).unwrap();
    assert!(record.contains("\"execution\""), "{record}");
    // Without a live answering channel, it still writes nothing.
    let other = candidate();
    let before = snapshot(other.path());
    let gone = elsewhere.path().join("absent");
    assert!(assemble(other.path(), Some(gone), "made-up")
        .unwrap_err()
        .contains("not live"));
    assert_eq!(snapshot(other.path()), before);
}

#[test]
fn only_the_admitted_executor_with_the_nonce_and_exact_root_is_answered() {
    let root = candidate();
    let before = snapshot(root.path());
    let dir = private_dir().unwrap();
    let answer = publish(dir.path(), &crafted());
    let host = serve(dir, root.path(), answer).unwrap();
    let (locator, nonce) = (host.locator().to_owned(), host.nonce());
    // No executor admitted yet, then another process admitted: this process is neither.
    let error = assemble(root.path(), Some(locator.clone()), &nonce).unwrap_err();
    assert!(error.contains("no executor is running"), "{error}");
    let other = Killed(Command::new("sleep").arg("30").spawn().unwrap());
    let admission = host.channel().admission;
    admission.admit(other.0.id());
    let error = assemble(root.path(), Some(locator.clone()), &nonce).unwrap_err();
    assert!(
        error.contains("serves only this invocation's executor"),
        "{error}"
    );
    // The admitted executor without the invocation's nonce is refused.
    admission.admit(std::process::id());
    for wrong in ["", "0".repeat(64).as_str()] {
        let error = assemble(root.path(), Some(locator.clone()), wrong).unwrap_err();
        assert!(error.contains("nonce"), "{error}");
    }
    assert_ne!(
        nonce,
        staged(root.path(), &crafted()).nonce(),
        "nonces are per invocation"
    );
    // A request naming a link to the candidate is not the canonical root the host serves.
    let links = tempfile::tempdir().unwrap();
    let link = links.path().join("candidate");
    std::os::unix::fs::symlink(fs::canonicalize(root.path()).unwrap(), &link).unwrap();
    let error = channel::request(&locator, &link, &nonce).unwrap_err();
    assert!(error.contains("no evidence for that root"), "{error}");
    // Once revoked (the executor exited), nothing is admitted or answered again.
    admission.revoke();
    admission.admit(std::process::id());
    assert!(!locator.exists());
    let error = assemble(root.path(), Some(locator), &nonce).unwrap_err();
    assert!(error.contains("not live"), "{error}");
    assert_eq!(snapshot(root.path()), before);
}

#[test]
fn a_refusal_sent_before_the_request_is_written_still_reaches_the_client() {
    // The host refuses an unadmitted peer without reading its request and closes. A client
    // descheduled between connecting and writing (a loaded runner) must still read that refusal,
    // not report the live host as gone.
    let root = candidate();
    let dir = private_dir().unwrap();
    let answer = publish(dir.path(), &crafted());
    let host = serve(dir, root.path(), answer).unwrap();
    let (locator, nonce) = (host.locator().to_owned(), host.nonce());
    let canonical = fs::canonicalize(root.path()).unwrap();
    let error =
        channel::request_with_pause(&locator, &canonical, &nonce, Duration::from_millis(500))
            .unwrap_err();
    assert!(error.contains("no executor is running"), "{error}");
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
fn an_actively_served_trickling_request_hits_the_absolute_deadline() {
    let root = candidate();
    let dir = private_dir().unwrap();
    let answer = publish(dir.path(), &crafted());
    let channel = channel::Server::start_with_deadline(
        &dir.path().join("channel"),
        fs::canonicalize(root.path()).unwrap(),
        answer,
        Duration::from_millis(300),
    )
    .unwrap();
    let host = in_process(Staged {
        channel,
        _dir: dir,
        files: BTreeMap::new(),
    });
    let locator = host.locator().to_owned();
    // The only client, so the host is servicing it: it admits it and reads every byte as it
    // trickles in, but the request never completes. Per-read progress never extends the deadline.
    let mut stream = UnixStream::connect(&locator).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_millis(20)))
        .unwrap();
    let (start, mut reply) = (Instant::now(), Vec::new());
    let closed = loop {
        if start.elapsed() > Duration::from_secs(5) {
            break None;
        }
        if stream.write_all(b" ").is_err() {
            break Some(start.elapsed());
        }
        let mut buf = [0u8; 256];
        match stream.read(&mut buf) {
            Ok(0) => break Some(start.elapsed()),
            Ok(n) => reply.extend_from_slice(&buf[..n]),
            Err(_) => {}
        }
        std::thread::sleep(Duration::from_millis(30));
    };
    let closed = closed.expect("the host serviced a trickling request past its deadline");
    assert!(
        closed >= Duration::from_millis(250) && closed < Duration::from_secs(2),
        "{closed:?}"
    );
    assert!(reply.is_empty(), "a request past its deadline got a reply");
    // The host stays live for the next request.
    assemble(root.path(), Some(locator), &host.nonce()).unwrap();
}

#[test]
fn executor_lineage_checks() {
    let own = std::process::id();
    let child = Killed(Command::new("sleep").arg("30").spawn().unwrap());
    let pid = child.0.id();
    assert!(peer::descends_from(pid, own));
    assert!(peer::descends_from(own, own));
    assert!(!peer::descends_from(own, pid));
}
