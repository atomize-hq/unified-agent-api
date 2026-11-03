//! Release validation: drives the wrapper against an acquired release binary.
//!
//! The acquisition descriptor runs this with `-- --ignored` and `DEPTH_FIXTURE_BINARY` set to the
//! downloaded executable. The crate's other tests exercise the locally built CLI instead.

use std::{process::Command, time::Duration};

use depth_fixture::{DepthFixture, Error, Event, RunRequest};

fn acquired() -> String {
    std::env::var("DEPTH_FIXTURE_BINARY").expect("DEPTH_FIXTURE_BINARY names the acquired binary")
}

fn fixture(scenario: &str) -> DepthFixture {
    let mut fixture = DepthFixture::new(acquired());
    fixture.timeout = Duration::from_secs(10);
    fixture
        .env
        .insert("DEPTH_FIXTURE_SCENARIO".into(), scenario.into());
    fixture
}

#[test]
#[ignore = "release validation: needs DEPTH_FIXTURE_BINARY"]
fn acquired_binary_reports_a_semantic_version() {
    let output = Command::new(acquired()).arg("--version").output().unwrap();
    assert!(output.status.success());
    let line = String::from_utf8(output.stdout).unwrap();
    let version = line.trim().strip_prefix("depth_fixture ").expect(&line);
    let parts: Vec<_> = version.split('.').collect();
    assert_eq!(parts.len(), 3, "{version}");
    assert!(parts.iter().all(|p| p.parse::<u64>().is_ok()), "{version}");
}

#[test]
#[ignore = "release validation: needs DEPTH_FIXTURE_BINARY"]
fn acquired_binary_streams_the_wrapped_run() {
    let request = RunRequest {
        prompt: "--json".into(),
        model: Some("fx.model-1".into()),
    };
    let events = fixture("ok").run(&request).unwrap().collect().unwrap();
    assert_eq!(
        events,
        vec![
            Event::Started {
                session_id: "fx-1".into()
            },
            Event::Message {
                text: "--json".into(),
                model: Some("fx.model-1".into())
            },
            Event::Completed
        ]
    );
    match fixture("fail").run(&request).unwrap().collect() {
        Err(Error::Exit { code, .. }) => assert_eq!(code, Some(3)),
        other => panic!("{other:?}"),
    }
}
