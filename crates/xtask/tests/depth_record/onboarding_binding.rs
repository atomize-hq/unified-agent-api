//! The lifecycle state's `support_depth` object against the onboarding record it states (OF-P2),
//! as `depth_record::onboarding::onboarding_binding` decides it (child module of `depth_record.rs`).

use std::collections::BTreeMap;

use xtask::depth_record::onboarding::{
    onboarding_binding, LifecycleDepth, OnboardingApproval, OnboardingDepth,
};
use xtask::depth_record::Event;

use super::*;

const NOW: &str = "2026-10-10T00:00:00Z";

fn onboarding_event(commit: &str, recorded_at: &str) -> Event {
    Event::Onboarding {
        approval_commit: commit.to_string(),
        approval_recorded_at: recorded_at.to_string(),
    }
}

/// The full record of `record()`, on the onboarding path, with the approval Event given.
fn onboarding_record(commit: &str, recorded_at: &str, with_o: bool) -> DepthRecord {
    let mut raw = record();
    raw["event"] = json!({
        "lifecycle_path": "onboarding", "approval_commit": commit,
        "approval_recorded_at": recorded_at
    });
    raw["policy"]["selection"]["enrollment"]["lifecycle_path"] = json!("onboarding");
    if !with_o {
        raw["obligations"] = Value::Null;
        raw["execution"] = Value::Null;
    }
    parse(&sealed(raw)).expect("parse onboarding record")
}

fn object_of(record: &DepthRecord) -> LifecycleDepth {
    LifecycleDepth {
        policy_identity: record.policy.identity.clone(),
        obligations_identity: record.obligations.as_ref().map(|o| o.identity.clone()),
    }
}

fn approval(commit: &str, recorded_at: &str, version: Option<&str>) -> OnboardingApproval {
    OnboardingApproval {
        event: onboarding_event(commit, recorded_at),
        support_depth_version: version.map(str::to_string),
        agent_id: "aider".to_string(),
        manifest_root: "cli_manifests/aider".to_string(),
        onboarding_pack_prefix: "aider-onboarding".to_string(),
    }
}

fn held(record: &DepthRecord) -> BTreeMap<String, DepthRecord> {
    BTreeMap::from([("1.2.3".to_string(), record.clone())])
}

fn codes(
    result: Result<OnboardingDepth, Vec<xtask::depth_record::onboarding::BindingFault>>,
) -> Vec<&'static str> {
    result.expect_err("faults").iter().map(|f| f.code).collect()
}

#[test]
fn no_record_and_no_object_is_not_frozen_and_not_an_error() {
    let approval = approval("abc", NOW, Some("1.2.3"));
    assert_eq!(
        onboarding_binding(None, &approval, &BTreeMap::new()),
        Ok(OnboardingDepth::NotFrozen)
    );
    // A maintenance record is not the object's.
    let maintenance = parse(&sealed(record())).unwrap();
    assert_eq!(
        onboarding_binding(None, &approval, &held(&maintenance)),
        Ok(OnboardingDepth::NotFrozen)
    );
}

#[test]
fn a_record_without_an_object_and_an_object_without_a_record_are_errors() {
    let record = onboarding_record("abc", NOW, true);
    let approval = approval("abc", NOW, Some("1.2.3"));
    // A missing object is never read as "not depth-enrolled".
    let faults = onboarding_binding(None, &approval, &held(&record)).unwrap_err();
    assert_eq!(faults.len(), 1);
    assert_eq!(faults[0].code, "DEPTH_LIFECYCLE_BINDING_MISSING");
    assert_eq!(faults[0].version.as_deref(), Some("1.2.3"));

    let object = object_of(&record);
    assert_eq!(
        codes(onboarding_binding(
            Some(&object),
            &approval,
            &BTreeMap::new()
        )),
        ["DEPTH_LIFECYCLE_BINDING_ORPHANED"]
    );
    // A maintenance record at the version is no onboarding record.
    let maintenance = parse(&sealed(super::record())).unwrap();
    assert_eq!(
        codes(onboarding_binding(
            Some(&object),
            &approval,
            &held(&maintenance)
        )),
        ["DEPTH_LIFECYCLE_BINDING_ORPHANED"]
    );
    // An approval that approves no depth enrollment has no record for the object to state.
    let none = self::approval("abc", NOW, None);
    assert_eq!(
        codes(onboarding_binding(Some(&object), &none, &held(&record))),
        [
            "DEPTH_LIFECYCLE_BINDING_MISSING",
            "DEPTH_LIFECYCLE_BINDING_ORPHANED"
        ]
    );
    // An onboarding record at another version is not the one the object states.
    let other = self::approval("abc", NOW, Some("1.2.4"));
    assert_eq!(
        codes(onboarding_binding(Some(&object), &other, &held(&record))),
        [
            "DEPTH_LIFECYCLE_BINDING_MISSING",
            "DEPTH_LIFECYCLE_BINDING_ORPHANED"
        ]
    );
}

#[test]
fn the_object_states_what_the_record_states() {
    let approval = approval("abc", NOW, Some("1.2.3"));
    for with_o in [true, false] {
        let record = onboarding_record("abc", NOW, with_o);
        assert_eq!(
            onboarding_binding(Some(&object_of(&record)), &approval, &held(&record)),
            Ok(OnboardingDepth::Frozen)
        );
    }
}

#[test]
fn a_mismatch_of_event_p_or_o_is_rejected() {
    let approval = approval("abc", NOW, Some("1.2.3"));
    let full = onboarding_record("abc", NOW, true);
    let early = onboarding_record("abc", NOW, false);
    let reject = |object: &LifecycleDepth, record: &DepthRecord, needle: &str| {
        let faults = onboarding_binding(Some(object), &approval, &held(record)).unwrap_err();
        assert_eq!(faults.len(), 1, "{faults:?}");
        assert_eq!(faults[0].code, "DEPTH_RECORD_NOT_THE_LIFECYCLES");
        assert!(faults[0].message.contains(needle), "{}", faults[0].message);
    };

    let mut p = object_of(&full);
    p.policy_identity = SHA.to_string();
    reject(&p, &full, "P");
    // O absent from the object while the record states it, and present while it does not.
    reject(&object_of(&early), &full, "O");
    let mut o = object_of(&early);
    o.obligations_identity = Some(SHA.to_string());
    reject(&o, &early, "O");
    // An Event that is neither the approval's nor an earlier approval's.
    let later = onboarding_record("abc", "2026-11-01T00:00:00Z", true);
    reject(&object_of(&later), &later, "neither");
    let same_time = onboarding_record("def", NOW, true);
    reject(&object_of(&same_time), &same_time, "neither");
}

#[test]
fn a_new_approval_keeps_the_earlier_object_until_its_first_dry_run() {
    let earlier = onboarding_record("abc", "2026-09-01T00:00:00+02:00", true);
    let approval = approval("def", NOW, Some("1.2.3"));
    let object = object_of(&earlier);
    // Insufficient depth, not an error.
    assert_eq!(
        onboarding_binding(Some(&object), &approval, &held(&earlier)),
        Ok(OnboardingDepth::EarlierGeneration)
    );
    // The earlier generation's P and O are still compared.
    let mut changed = object.clone();
    changed.obligations_identity = None;
    assert_eq!(
        codes(onboarding_binding(
            Some(&changed),
            &approval,
            &held(&earlier)
        )),
        ["DEPTH_RECORD_NOT_THE_LIFECYCLES"]
    );
    // Once this generation's dry run replaces the object and continues the record, it is frozen.
    let continued = onboarding_record("def", NOW, true);
    assert_eq!(
        onboarding_binding(Some(&object_of(&continued)), &approval, &held(&continued)),
        Ok(OnboardingDepth::Frozen)
    );
}

#[test]
fn the_object_holds_exactly_two_digests() {
    let parse_object = |raw: Value| serde_json::from_value::<LifecycleDepth>(raw);
    let object = parse_object(json!({ "policy_identity": SHA })).unwrap();
    assert!(object.validate().is_ok() && object.obligations_identity.is_none());
    // O not yet frozen is left out, never `null`.
    assert_eq!(
        serde_json::to_value(&object).unwrap(),
        json!({ "policy_identity": SHA })
    );
    assert!(parse_object(json!({ "policy_identity": SHA, "version": "1.2.3" })).is_err());
    assert!(parse_object(json!({ "obligations_identity": SHA })).is_err());
    let short = parse_object(json!({ "policy_identity": "abc" })).unwrap();
    assert!(short
        .validate()
        .unwrap_err()
        .contains("support_depth.policy_identity"));
    let upper = parse_object(json!({ "policy_identity": SHA, "obligations_identity": SHA.to_uppercase().replace('0', "A") })).unwrap();
    assert!(upper.validate().is_err());
}

#[test]
fn the_approval_states_its_event_and_depth_version() {
    let path = "docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml";
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let text = std::fs::read_to_string(repo.join(path)).expect("read committed approval");
    let committed = OnboardingApproval::parse(&text, path).expect("parse committed approval");
    assert_eq!(
        committed.event,
        onboarding_event(
            "ebbc04aec5f723e63b527351c5c1629d22489180",
            "2026-04-28T00:00:35Z"
        )
    );
    assert_eq!(committed.support_depth_version, None);
    // The approval is bound to the agent whose registry entry it is read for.
    assert_eq!(
        committed.attribution_mismatch("aider", "cli_manifests/aider", "aider-onboarding"),
        None
    );
    for (agent, root, pack) in [
        ("codex", "cli_manifests/aider", "aider-onboarding"),
        ("aider", "cli_manifests/codex", "aider-onboarding"),
        ("aider", "cli_manifests/aider", "codex-cli-onboarding"),
    ] {
        let why = committed
            .attribution_mismatch(agent, root, pack)
            .expect("mismatch");
        assert!(why.contains("the registry entry's"), "{why}");
    }

    let enrolled = format!("{text}\n[support_depth]\nversion = \"1.2.3\"\n");
    let enrolled = OnboardingApproval::parse(&enrolled, path).expect("parse depth approval");
    assert_eq!(enrolled.support_depth_version.as_deref(), Some("1.2.3"));
    // The approval loader's own rules decide the table.
    let misspelled = format!("{text}\n[support_depth]\nversoin = \"1.2.3\"\n");
    assert!(OnboardingApproval::parse(&misspelled, path).is_err());
}
