use super::*;
use serde_json::{json, Value};

#[test]
fn canonical_facts_deduplicate_and_preserve_absence_and_unavailability() {
    let capture = Capture::start().unwrap();
    discovery("population", false);
    discovery("population", true);
    population("population", "runs", vec![]);
    fact("artifact/1", "witness_bytes", &digest(b"", &[1, 2, 3]));
    let first = capture.finish().unwrap();
    assert!(!first.incomplete);
    // Independently computed with Python json/hashlib from these normalized facts.
    assert_eq!(
        first.identity,
        "a740cc927410b5d84653e5d424e74b60ad767838b47dc9c6659762f118ed442d"
    );
    let capture = Capture::start().unwrap();
    fact("artifact/1", "witness_bytes", &digest(b"", &[1, 2, 3]));
    population("population", "runs", vec![]);
    discovery("population", true);
    discovery("population", false);
    discovery("population", true);
    assert_eq!(capture.finish().unwrap(), first);
    let capture = Capture::start().unwrap();
    unavailable::<Value>("population", &Err("unavailable at a local path".into()));
    let failed = capture.finish().unwrap();
    assert!(failed.incomplete);
    let capture = Capture::start().unwrap();
    unavailable::<Value>("population", &Err("another transport error".into()));
    unavailable("population", &Ok(json!([])));
    assert_eq!(capture.finish().unwrap(), failed);
    assert_ne!(failed.identity, first.identity);
}

#[test]
fn observation_capture_is_scoped_and_cleared_on_drop() {
    let capture = Capture::start().unwrap();
    assert!(Capture::start().is_err());
    unavailable::<Value>("discarded", &Err("unavailable".into()));
    drop(capture);
    let capture = Capture::start().unwrap();
    let empty = capture.finish().unwrap();
    assert!(!empty.incomplete);
    assert_eq!(
        empty.identity,
        "abf865743b3665009809797970ea64fffa1a48c25cdf1b6b58cf134d8db63845"
    );
}

#[test]
fn population_order_preserves_authority_but_multiplicity_and_absence_change_it() {
    let capture = Capture::start().unwrap();
    population(
        "jobs",
        "producer_jobs",
        vec![json!([2, "success"]), json!([1, "failure"])],
    );
    let first = capture.finish().unwrap();
    let capture = Capture::start().unwrap();
    population(
        "jobs",
        "producer_jobs",
        vec![json!([1, "failure"]), json!([2, "success"])],
    );
    assert_eq!(capture.finish().unwrap(), first);
    let capture = Capture::start().unwrap();
    population(
        "jobs",
        "producer_jobs",
        vec![
            json!([1, "failure"]),
            json!([2, "success"]),
            json!([2, "success"]),
        ],
    );
    assert_ne!(capture.finish().unwrap(), first);
    let capture = Capture::start().unwrap();
    population("jobs", "producer_jobs", vec![]);
    assert_ne!(capture.finish().unwrap(), first);
}

#[test]
fn reached_decisions_pin_facts_and_survive_failed_closing_rechecks() {
    for admitted in [false, true] {
        let capture = Capture::start().unwrap();
        fact("repository", "identity", &123);
        after_decision(Ok(()), || Ok(())).unwrap();
        let complete = capture.finish().unwrap();
        let capture = Capture::start().unwrap();
        fact("repository", "identity", &123);
        let outcome = if admitted {
            Ok(())
        } else {
            Err("policy refusal".into())
        };
        let error = after_decision(outcome, || {
            unavailable::<()>("repository", &Err("late flake".into()));
            fact("repository", "identity", &999);
            Err("closing refused".into())
        })
        .unwrap_err();
        assert!(error.contains("closing refused"));
        assert!(admitted || error.contains("policy refusal"));
        // A successful later recheck cannot overwrite the reached result or completeness.
        after_decision(Ok(()), || Ok(())).unwrap();
        let observed = capture.finish().unwrap();
        assert_eq!(observed.identity, complete.identity);
        assert!(observed.incomplete);
        assert_eq!(
            observed.decision,
            Some(if admitted {
                Decision::Admitted
            } else {
                Decision::Refused
            })
        );
    }
}

#[test]
fn unavailable_owning_read_never_claims_a_substantive_decision() {
    let capture = Capture::start().unwrap();
    unavailable::<()>("repository", &Err("early flake".into()));
    assert!(after_decision::<()>(Err("read failed".into()), || Ok(())).is_err());
    let authority = capture.finish().unwrap();
    assert!(authority.incomplete);
    assert_eq!(authority.decision, None);
}
