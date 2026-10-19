//! What a maintenance request froze, as `DepthRecord::request_mismatch` compares it with a record
//! (child module of `depth_record.rs`).

use xtask::depth_record::{Event, RequestBinding};

use super::*;

const EVENT: &str = "the record's Event is not the request's";
const P: &str = "the record's P is not the one the request froze";
const O: &str = "the record's O is not the one the request froze";

/// What a request states when it froze exactly what `record` states.
fn froze(record: &DepthRecord) -> RequestBinding {
    RequestBinding {
        event: record.event.clone(),
        policy_identity: Some(record.policy.identity.clone()),
        obligations_identity: record.obligations.as_ref().map(|o| o.identity.clone()),
    }
}

#[test]
fn a_record_states_what_a_maintenance_request_froze() {
    let full = parse(&sealed(record())).unwrap();
    let mut early = record();
    early["obligations"] = Value::Null;
    early["execution"] = Value::Null;
    let early = parse(&sealed(early)).unwrap();

    // P and O frozen, and P frozen with O not yet produced.
    assert_eq!(full.request_mismatch(&froze(&full)), None);
    assert_eq!(early.request_mismatch(&froze(&early)), None);

    // Every member of the Event is compared, and so is the lifecycle path.
    for member in 0..4 {
        let mut request = froze(&full);
        let Event::Maintenance {
            request_commit,
            request_recorded_at,
            trigger_kind,
            opened_from,
        } = &mut request.event
        else {
            unreachable!("the fixture's Event is a maintenance Event")
        };
        let members = [
            request_commit,
            request_recorded_at,
            trigger_kind,
            opened_from,
        ];
        members[member].push('x');
        assert_eq!(
            full.request_mismatch(&request),
            Some(EVENT),
            "member {member}"
        );
    }
    let mut request = froze(&full);
    request.event = Event::Onboarding {
        approval_commit: "abc".to_string(),
        approval_recorded_at: "2026-10-10T00:00:00Z".to_string(),
    };
    assert_eq!(full.request_mismatch(&request), Some(EVENT));

    // P: another identity, and a request that froze none.
    let mut request = froze(&full);
    request.policy_identity = Some(SHA.to_string());
    assert_eq!(full.request_mismatch(&request), Some(P));
    request.policy_identity = None;
    assert_eq!(full.request_mismatch(&request), Some(P));

    // O: another identity, none frozen though the record states O, and one frozen that the record
    // does not yet state.
    let mut request = froze(&full);
    request.obligations_identity = Some(SHA.to_string());
    assert_eq!(full.request_mismatch(&request), Some(O));
    request.obligations_identity = None;
    assert_eq!(full.request_mismatch(&request), Some(O));
    let mut request = froze(&early);
    request.obligations_identity = full.obligations.as_ref().map(|o| o.identity.clone());
    assert_eq!(early.request_mismatch(&request), Some(O));

    // The Event is decided first, then P, then O.
    let mut request = froze(&full);
    request.event = Event::Onboarding {
        approval_commit: "abc".to_string(),
        approval_recorded_at: "2026-10-10T00:00:00Z".to_string(),
    };
    request.policy_identity = None;
    request.obligations_identity = None;
    assert_eq!(full.request_mismatch(&request), Some(EVENT));
    request.event = full.event.clone();
    assert_eq!(full.request_mismatch(&request), Some(P));
}
