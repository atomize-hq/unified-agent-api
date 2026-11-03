//! LC2 lifecycle, bounded buffering and diagnostic-safety evidence (Linux: uses `/proc`).
#![cfg(target_os = "linux")]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use depth_fixture::{DepthFixture, Error, Event, RunRequest, DIAGNOSTIC_BYTES};

fn fixture(scenario: &str) -> DepthFixture {
    let mut fixture = DepthFixture::new(env!("CARGO_BIN_EXE_depth_fixture"));
    fixture.timeout = Duration::from_secs(10);
    fixture
        .env
        .insert("DEPTH_FIXTURE_SCENARIO".into(), scenario.into());
    fixture
}

fn request() -> RunRequest {
    RunRequest {
        prompt: "x".into(),
        model: None,
    }
}

/// Running or stopped, but neither exited-unreaped (zombie) nor gone.
fn alive(pid: u32) -> bool {
    fs::read_to_string(format!("/proc/{pid}/stat"))
        .ok()
        .and_then(|stat| stat.rsplit(") ").next().map(|rest| !rest.starts_with('Z')))
        .unwrap_or(false)
}

/// Waited for: no process entry remains, not even a zombie.
fn reaped(pid: u32) -> bool {
    !Path::new(&format!("/proc/{pid}")).exists()
}

#[test]
fn timeout_kills_and_reaps_the_child() {
    let mut slow = fixture("slow");
    slow.timeout = Duration::from_millis(300);
    let mut run = slow.run(&request()).unwrap();
    let pid = run.id();
    assert!(matches!(run.next_event(), Some(Ok(Event::Started { .. }))));
    assert!(matches!(run.next_event(), Some(Err(Error::Timeout))));
    assert!(reaped(pid));
}

#[test]
fn cancel_kills_and_reaps_the_child() {
    let mut run = fixture("slow").run(&request()).unwrap();
    let pid = run.id();
    assert!(matches!(run.next_event(), Some(Ok(Event::Started { .. }))));
    assert!(alive(pid));
    run.cancel();
    assert!(reaped(pid));
}

#[test]
fn drop_kills_and_reaps_the_child() {
    let mut run = fixture("slow").run(&request()).unwrap();
    let pid = run.id();
    assert!(matches!(run.next_event(), Some(Ok(Event::Started { .. }))));
    drop(run);
    assert!(reaped(pid));
}

// The deadline covers queued events: polling late cannot turn a timed-out run into success.
#[test]
fn polling_after_the_deadline_times_out_even_with_queued_events() {
    let mut ok = fixture("ok");
    ok.timeout = Duration::from_millis(200);
    let mut run = ok.run(&request()).unwrap();
    let pid = run.id();
    thread::sleep(Duration::from_millis(400));
    assert!(matches!(run.next_event(), Some(Err(Error::Timeout))));
    assert!(run.next_event().is_none());
    assert!(reaped(pid));
}

// The deadline covers the exit wait: a child that closes stdout but keeps running times out.
#[test]
fn closed_stdout_with_a_live_child_times_out() {
    let dir = tempfile_dir();
    let script = dir.join("depth_fixture");
    fs::write(
        &script,
        "#!/bin/sh\nPATH=/usr/bin:/bin\nexec 1>&-\nexec sleep 30\n",
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    let mut fx = DepthFixture::new(&script);
    fx.timeout = Duration::from_millis(500);
    let begun = Instant::now();
    let mut run = spawn_retrying_busy(&fx);
    let pid = run.id();
    assert!(matches!(run.next_event(), Some(Err(Error::Timeout))));
    assert!(begun.elapsed() < Duration::from_secs(5));
    assert!(reaped(pid));
    fs::remove_dir_all(dir).unwrap();
}

// Bounded buffering: a paused consumer leaves the child blocked on its own output, then every
// event still arrives in order.
#[test]
fn paused_consumer_applies_backpressure_without_loss() {
    let mut run = fixture("burst").run(&request()).unwrap();
    let pid = run.id();
    thread::sleep(Duration::from_millis(500));
    assert!(alive(pid), "the child finished despite a paused consumer");
    let mut messages = 0;
    while let Some(event) = run.next_event() {
        match event.unwrap() {
            Event::Message { text, .. } => {
                assert!(text.starts_with(&format!("line {messages:05} ")));
                messages += 1;
            }
            Event::Started { .. } | Event::Completed => {}
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(messages, 10_000);
}

// X1 / X3: stderr is drained (the child exits with its own code, not a broken pipe), only a
// bounded tail is kept, and a parent-environment sentinel never appears.
#[test]
fn stderr_is_drained_bounded_and_free_of_parent_sentinels() {
    std::env::set_var("DEPTH_FIXTURE_SECRET", "parent-sentinel-x3");
    let result = fixture("stderr_sentinel")
        .run(&request())
        .unwrap()
        .collect();
    match result {
        Err(Error::Exit { code, stderr }) => {
            assert_eq!(code, Some(5));
            assert!(stderr.len() <= DIAGNOSTIC_BYTES);
            assert!(stderr.ends_with("secret=<unset>"), "{stderr}");
            assert!(!stderr.contains("parent-sentinel-x3"));
        }
        other => panic!("{other:?}"),
    }
}

// X3: the malformed-line projection is bounded in bytes even after lossy decoding.
#[test]
fn malformed_projection_is_bounded_after_decoding() {
    let mut run = fixture("malformed_binary").run(&request()).unwrap();
    assert!(matches!(run.next_event(), Some(Ok(Event::Started { .. }))));
    match run.next_event() {
        Some(Err(Error::Malformed { prefix })) => {
            assert!(!prefix.is_empty());
            assert!(prefix.len() <= DIAGNOSTIC_BYTES, "{}", prefix.len());
        }
        other => panic!("{other:?}"),
    }
}

// Cancel with a full buffer: the reader blocked on it must exit, not leak.
#[test]
fn cancel_with_a_full_buffer_releases_the_reader() {
    let mut run = fixture("burst").run(&request()).unwrap();
    let pid = run.id();
    thread::sleep(Duration::from_millis(300));
    run.cancel();
    assert!(reaped(pid));
    let begun = Instant::now();
    while !run.reader_finished() && begun.elapsed() < Duration::from_secs(5) {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(run.reader_finished());
    assert!(matches!(run.next_event(), Some(Err(Error::Cancelled))));
}

// A timeout too large to represent is rejected before spawn instead of panicking after it.
#[test]
fn unrepresentable_timeout_is_rejected_before_spawn() {
    let mut fx = DepthFixture::new("/nonexistent/depth_fixture");
    fx.timeout = Duration::MAX;
    assert!(matches!(fx.run(&request()), Err(Error::InvalidRequest(_))));
}

/// A script written by this test can be briefly `ETXTBSY` while another test thread forks.
fn spawn_retrying_busy(fx: &DepthFixture) -> depth_fixture::Run {
    for _ in 0..50 {
        match fx.run(&request()) {
            Err(Error::Spawn(e)) if e.raw_os_error() == Some(26) => {
                thread::sleep(Duration::from_millis(20))
            }
            other => return other.unwrap(),
        }
    }
    panic!("script stayed busy");
}

// An out-of-range line limit is rejected before spawn.
#[test]
fn out_of_range_line_limit_is_rejected_before_spawn() {
    let mut fx = DepthFixture::new("/nonexistent/depth_fixture");
    for limit in [0, usize::MAX] {
        fx.max_line_bytes = limit;
        assert!(matches!(fx.run(&request()), Err(Error::InvalidRequest(_))));
    }
}

fn tempfile_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "depth-fixture-{}-{}",
        std::process::id(),
        Instant::now().elapsed().as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}
