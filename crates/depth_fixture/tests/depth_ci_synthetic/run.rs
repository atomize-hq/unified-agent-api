//! Wrapper behavior against the synthetic CLI, one group per promised dimension. These tests
//! prove the wrapper against this fixture only; they qualify no real agent.

use std::time::{Duration, Instant};

use depth_fixture::{DepthFixture, Error, Event, RunRequest, DIAGNOSTIC_BYTES, MAX_PROMPT_BYTES};

fn fixture(scenario: &str) -> DepthFixture {
    let mut fixture = DepthFixture::new(env!("CARGO_BIN_EXE_depth_fixture"));
    fixture.timeout = Duration::from_secs(10);
    fixture
        .env
        .insert("DEPTH_FIXTURE_SCENARIO".into(), scenario.into());
    fixture
}

fn request(prompt: &str) -> RunRequest {
    RunRequest {
        prompt: prompt.into(),
        model: None,
    }
}

fn started() -> Event {
    Event::Started {
        session_id: "fx-1".into(),
    }
}

// R1 / G1 / OUT3: request values reach the child; known events are typed in order.
#[test]
fn run_streams_typed_events_in_order() {
    let mut req = request("hello");
    req.model = Some("fx.model-1".into());
    let events = fixture("ok").run(&req).unwrap().collect().unwrap();
    assert_eq!(
        events,
        vec![
            started(),
            Event::Message {
                text: "hello".into(),
                model: Some("fx.model-1".into())
            },
            Event::Completed
        ]
    );
}

// R1 / G1: a prompt that looks like a flag is still the prompt, because it follows `--`.
#[test]
fn flag_like_prompt_is_passed_as_the_prompt() {
    let events = fixture("ok")
        .run(&request("--json"))
        .unwrap()
        .collect()
        .unwrap();
    assert!(events.contains(&Event::Message {
        text: "--json".into(),
        model: Some("fixture-default".into())
    }));
}

// V1: invalid requests are rejected before any child is spawned.
#[test]
fn invalid_requests_are_rejected_before_spawn() {
    let mut unspawnable = DepthFixture::new("/nonexistent/depth_fixture");
    unspawnable.timeout = Duration::from_secs(1);
    let long = "x".repeat(MAX_PROMPT_BYTES + 1);
    let mut bad_model = request("ok");
    bad_model.model = Some("bad model".into());
    for req in [request(""), request(&long), request("a\0b"), bad_model] {
        // A spawn attempt would fail with `Spawn`; `InvalidRequest` proves none was made.
        assert!(matches!(
            unspawnable.run(&req),
            Err(Error::InvalidRequest(_))
        ));
    }
    unspawnable.timeout = Duration::ZERO;
    assert!(matches!(
        unspawnable.run(&request("ok")),
        Err(Error::InvalidRequest(_))
    ));
}

// E1: spawn failure is distinct from validation.
#[test]
fn spawn_failure_is_reported_as_spawn() {
    let unspawnable = DepthFixture::new("/nonexistent/depth_fixture");
    assert!(matches!(
        unspawnable.run(&request("ok")),
        Err(Error::Spawn(_))
    ));
}

// OUT3: unknown event kinds are surfaced, not dropped or fatal.
#[test]
fn unknown_events_are_surfaced() {
    let events = fixture("unknown_event")
        .run(&request("x"))
        .unwrap()
        .collect()
        .unwrap();
    assert_eq!(
        events,
        vec![
            started(),
            Event::Unknown {
                kind: "future.kind".into()
            },
            Event::Completed
        ]
    );
}

// OUT3 / E1 / X3: a malformed line is an error with a bounded projection.
#[test]
fn malformed_line_is_an_error() {
    let mut run = fixture("malformed").run(&request("x")).unwrap();
    assert_eq!(run.next_event().unwrap().unwrap(), started());
    match run.next_event().unwrap() {
        Err(Error::Malformed { prefix }) => assert_eq!(prefix, "{not json"),
        other => panic!("{other:?}"),
    }
    assert!(run.next_event().is_none());
}

// E1 / X3: a non-zero exit is an error carrying only a bounded stderr tail.
#[test]
fn non_zero_exit_is_an_error_after_streamed_events() {
    let mut run = fixture("fail").run(&request("x")).unwrap();
    assert_eq!(run.next_event().unwrap().unwrap(), started());
    match run.next_event().unwrap() {
        Err(Error::Exit { code, stderr }) => {
            assert_eq!(code, Some(3));
            assert_eq!(stderr, "fixture failure");
            assert!(stderr.len() <= DIAGNOSTIC_BYTES);
        }
        other => panic!("{other:?}"),
    }
}

// LC2: end of output is not success without a terminal event.
#[test]
fn eof_without_terminal_event_is_not_success() {
    let result = fixture("no_terminal").run(&request("x")).unwrap().collect();
    assert!(matches!(result, Err(Error::NoTerminal)));
}

// LC2: a terminal event followed by a failing exit is still a failure.
#[test]
fn completion_then_failing_exit_is_a_failure() {
    let result = fixture("completed_then_fail")
        .run(&request("x"))
        .unwrap()
        .collect();
    assert!(matches!(result, Err(Error::Exit { code: Some(4), .. })));
}

// LC2: events arrive before completion; timeout kills and reaps the child.
#[test]
fn timeout_after_streamed_event_kills_the_child() {
    let mut slow = fixture("slow");
    slow.timeout = Duration::from_millis(500);
    let begun = Instant::now();
    let mut run = slow.run(&request("x")).unwrap();
    assert_eq!(run.next_event().unwrap().unwrap(), started());
    assert!(matches!(run.next_event(), Some(Err(Error::Timeout))));
    assert!(run.next_event().is_none());
    assert!(begun.elapsed() < Duration::from_secs(10));
}

// LC2: cancel kills and reaps; the next poll reports cancellation once.
#[test]
fn cancel_stops_the_run() {
    let mut run = fixture("slow").run(&request("x")).unwrap();
    assert_eq!(run.next_event().unwrap().unwrap(), started());
    let begun = Instant::now();
    run.cancel();
    assert!(matches!(run.next_event(), Some(Err(Error::Cancelled))));
    assert!(run.next_event().is_none());
    assert!(begun.elapsed() < Duration::from_secs(5));
}

// LC2: dropping a live run does not wait for the child to finish on its own.
#[test]
fn drop_reaps_a_live_run() {
    let begun = Instant::now();
    let mut run = fixture("slow").run(&request("x")).unwrap();
    assert_eq!(run.next_event().unwrap().unwrap(), started());
    drop(run);
    assert!(begun.elapsed() < Duration::from_secs(5));
}

// X3: a line beyond the byte bound is an error, not an unbounded allocation.
#[test]
fn oversized_line_is_rejected() {
    let mut run = fixture("large").run(&request("x")).unwrap();
    assert_eq!(run.next_event().unwrap().unwrap(), started());
    assert!(matches!(
        run.next_event(),
        Some(Err(Error::OversizedLine { limit: 65_536 }))
    ));
}

// X1 / X2 / X4: cleared environment, explicit variables only, caller cwd, null stdin.
#[test]
fn child_sees_only_explicit_environment_and_null_stdin() {
    // A sentinel in the parent environment must not reach the child.
    std::env::set_var("DEPTH_FIXTURE_SECRET", "parent-sentinel");
    let dir = std::env::temp_dir();
    let mut fx = fixture("environment");
    fx.current_dir = Some(dir.clone());
    fx.env.insert("HOME".into(), "/fixture-home".into());
    let events = fx.run(&request("x")).unwrap().collect().unwrap();
    let text = events
        .iter()
        .find_map(|e| match e {
            Event::Message { text, .. } => Some(text.clone()),
            _ => None,
        })
        .unwrap();
    let cwd = dir.canonicalize().unwrap();
    assert!(text.contains("home=/fixture-home"), "{text}");
    assert!(text.contains("secret=<unset>"), "{text}");
    assert!(text.contains("stdin_bytes=0"), "{text}");
    assert!(text.contains("stdin_tty=false"), "{text}");
    assert!(
        text.contains(&format!("cwd={}", cwd.display()))
            || text.contains(&format!("cwd={}", dir.display())),
        "{text}"
    );
    assert!(!format!("{events:?}").contains("parent-sentinel"));
}

// X2: with an empty home and working directory, a run leaves both empty: no config, cache or
// other write. This shows the fixture's effects only; the wrapper is not a sandbox.
#[test]
fn run_writes_nothing_to_home_or_working_directory() {
    let base = std::env::temp_dir().join(format!("depth-fixture-x2-{}", std::process::id()));
    let (home, cwd) = (base.join("home"), base.join("cwd"));
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&cwd).unwrap();
    let mut fx = fixture("environment");
    fx.current_dir = Some(cwd.clone());
    fx.env.insert("HOME".into(), home.display().to_string());
    let events = fx.run(&request("x")).unwrap().collect().unwrap();
    assert!(format!("{events:?}").contains(&format!("home={}", home.display())));
    for dir in [&home, &cwd] {
        assert_eq!(
            std::fs::read_dir(dir).unwrap().count(),
            0,
            "{}",
            dir.display()
        );
    }
    std::fs::remove_dir_all(base).unwrap();
}

// A1: the fixture reports the upstream version it was built as.
#[test]
fn fixture_reports_its_upstream_version() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_depth_fixture"))
        .arg("--version")
        .output()
        .unwrap();
    let expected = option_env!("DEPTH_FIXTURE_UPSTREAM_VERSION").unwrap_or("0.1.0");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("depth_fixture {expected}\n")
    );
}
