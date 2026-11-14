use super::*;
use serde_json::json;

#[test]
fn canonical_host_facts_deduplicate_and_preserve_absence_and_unavailability() {
    let capture = Capture::start().unwrap();
    json(
        "population",
        &Ok(json!({"total_count":0,"workflow_runs":[]})),
    );
    bytes("artifact/1", &Ok(vec![1, 2, 3]));
    let first = capture.finish().unwrap();
    // Independently computed with Python json/hashlib over the literal tagged responses.
    assert_eq!(
        first,
        "98e4022e58aae5cff89ac327fad6aebb9b35b30bee4fc8901fbfd22ae1df9f42"
    );
    let capture = Capture::start().unwrap();
    bytes("artifact/1", &Ok(vec![1, 2, 3]));
    json(
        "population",
        &Ok(json!({"workflow_runs":[],"total_count":0})),
    );
    bytes("artifact/1", &Ok(vec![1, 2, 3]));
    assert_eq!(capture.finish().unwrap(), first);
    let capture = Capture::start().unwrap();
    json("population", &Err("unavailable at a local path".into()));
    let unavailable = capture.finish().unwrap();
    assert_ne!(unavailable, first);
    let capture = Capture::start().unwrap();
    json("population", &Err("another transport error".into()));
    assert_eq!(capture.finish().unwrap(), unavailable);
    let capture = Capture::start().unwrap();
    json("population", &Ok(json!([])));
    assert_ne!(capture.finish().unwrap(), unavailable);
}

#[test]
fn observation_capture_is_scoped_and_cleared_on_drop() {
    let capture = Capture::start().unwrap();
    assert!(Capture::start().is_err());
    bytes("discarded", &Ok(vec![1]));
    drop(capture);
    let capture = Capture::start().unwrap();
    // Pin the empty observation set independently of runtime-generated fixture identities.
    assert_eq!(
        capture.finish().unwrap(),
        "abf865743b3665009809797970ea64fffa1a48c25cdf1b6b58cf134d8db63845"
    );
    let capture = Capture::start().unwrap();
    bytes("artifact/1", &Err("unavailable".into()));
    let unavailable = capture.finish().unwrap();
    let capture = Capture::start().unwrap();
    bytes("artifact/1", &Ok(vec![]));
    assert_ne!(capture.finish().unwrap(), unavailable);
}
