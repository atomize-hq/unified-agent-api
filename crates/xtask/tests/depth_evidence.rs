use serde_json::{json, Value};
use xtask::depth_evidence::EvidenceReceipt;

const GOLDEN: &str = include_str!("fixtures/depth-evidence-v1.json");

fn check(value: &Value) -> Result<EvidenceReceipt, String> {
    EvidenceReceipt::parse(&value.to_string())
}

#[test]
fn golden_is_a_typed_claim_without_an_admission_verdict() {
    let receipt = EvidenceReceipt::parse(GOLDEN).unwrap();
    assert_eq!(receipt.scope.agent_id, "codex");
    assert_eq!(receipt.cases.executed.len(), 1);
    // Golden mapping identity was computed independently using Python's canonical ASCII JSON.
    assert_eq!(
        receipt.adequacy.endorsement.mapping_sha256,
        "0dcd00bde9fb09151b6524b244fb55bd1e3fe3db4263e976cda78a0d43684358"
    );
    assert_eq!(
        check(&serde_json::to_value(&receipt).unwrap()).unwrap(),
        receipt
    );
}

#[test]
fn every_object_rejects_missing_null_and_unknown_members() {
    fn objects(value: &Value, path: String, out: &mut Vec<(String, String)>) {
        match value {
            Value::Object(map) => {
                out.push((path.clone(), map.keys().next().unwrap().clone()));
                for (k, v) in map {
                    objects(v, format!("{path}/{k}"), out);
                }
            }
            Value::Array(items) => {
                for (i, v) in items.iter().enumerate() {
                    objects(v, format!("{path}/{i}"), out);
                }
            }
            _ => {}
        }
    }
    let golden: Value = serde_json::from_str(GOLDEN).unwrap();
    let mut paths = vec![];
    objects(&golden, String::new(), &mut paths);
    for (path, key) in paths {
        for mutation in ["missing", "null", "unknown"] {
            let mut value = golden.clone();
            let object = value.pointer_mut(&path).unwrap().as_object_mut().unwrap();
            match mutation {
                "missing" => {
                    object.remove(&key);
                }
                "null" => {
                    object.insert(key.clone(), Value::Null);
                }
                _ => {
                    object.insert("unrecognized".into(), json!(true));
                }
            }
            assert!(check(&value).is_err(), "{mutation} {path}/{key}");
        }
    }
}

#[test]
fn malformed_binding_and_attribution_fields_refuse() {
    for (path, bad) in [
        ("/schema_version", json!(2)),
        ("/producer/kind", json!("local")),
        ("/producer/run_attempt", json!(0)),
        ("/producer/run_id", json!(-1)),
        ("/producer/repository", json!("../repo")),
        ("/producer/workflow_path", json!("scripts/unrelated.yml")),
        ("/producer/workflow_commit", json!("A".repeat(40))),
        ("/scope/version", json!("v1.2.3")),
        ("/scope/version", json!("1.2.3+build")),
        ("/scope/policy_identity", json!("not a digest")),
        ("/scope/obligation/operation", json!(" ")),
        ("/scope/enrollment_origin", json!("legacy")),
        ("/build/dependency_graph/path", json!("../escape")),
        ("/build/toolchain/path", json!(".git/config")),
        ("/cases/executed/0/outcome", json!("success")),
        ("/adequacy/endorsement/kind", json!("local_file")),
        (
            "/adequacy/endorsement/mapping_sha256",
            json!("f".repeat(64)),
        ),
        (
            "/adequacy/mapping/assertions/0/counterfactual",
            json!("looks fine"),
        ),
    ] {
        let mut value: Value = serde_json::from_str(GOLDEN).unwrap();
        *value.pointer_mut(path).unwrap() = bad;
        assert!(check(&value).is_err(), "{path}");
    }
    let mut value: Value = serde_json::from_str(GOLDEN).unwrap();
    value["scope"]["obligation"]["capability"] = json!("agent_api.run");
    assert!(check(&value).is_err());
}

#[test]
fn limits_duplicate_identities_and_present_absent_conflicts_refuse() {
    let base: Value = serde_json::from_str(GOLDEN).unwrap();
    for path in [
        "/cases/requested",
        "/cases/executed",
        "/build/covered_inputs",
        "/build/features",
        "/provenance",
        "/adequacy/mapping/assertions",
    ] {
        let mut value = base.clone();
        let list = value.pointer_mut(path).unwrap().as_array_mut().unwrap();
        list.push(list[0].clone());
        assert!(check(&value).is_err(), "{path}");
    }
    let mut value = base.clone();
    value["build"]["absent_inputs"] = json!(["Cargo.lock"]);
    assert!(check(&value).is_err());
    for path in [
        "reports/toolchain.txt",
        "reports/environment.json",
        "reports/dependencies.json",
    ] {
        value = base.clone();
        value["build"]["absent_inputs"] = json!([path]);
        assert!(check(&value).is_err(), "present build reference {path}");
    }
    value = base.clone();
    value["build"]["features"] = json!((0..4097)
        .map(|i| format!("feature-{i}"))
        .collect::<Vec<_>>());
    assert!(check(&value).is_err());
    value = base;
    value["scope"]["agent_id"] = json!("a".repeat(4097));
    assert!(check(&value).is_err());
    assert!(EvidenceReceipt::parse(&format!("{}{}", GOLDEN, " ".repeat(1_048_576))).is_err());
    assert!(EvidenceReceipt::parse(&GOLDEN.replacen(
        "\"schema_version\": 1",
        "\"schema_version\": 1, \"schema_version\": 1",
        1
    ))
    .is_err());
}

#[test]
fn null_optional_obligation_members_do_not_become_omitted_members() {
    let mut value: Value = serde_json::from_str(GOLDEN).unwrap();
    value["scope"]["obligation"]["capability"] = Value::Null;
    assert!(check(&value).is_err());
}

#[test]
fn positional_arrays_cannot_replace_named_objects() {
    let golden: Value = serde_json::from_str(GOLDEN).unwrap();
    let mut accepted = vec![];
    for (path, keys) in [
        (
            "",
            vec![
                "schema_version",
                "scope",
                "producer",
                "build",
                "provenance",
                "cases",
                "adequacy",
            ],
        ),
        (
            "/scope",
            vec![
                "agent_id",
                "version",
                "event",
                "enrollment_origin",
                "obligation",
                "policy_identity",
                "obligations_identity",
                "rules_revision",
                "trust_revision",
                "review_revision",
            ],
        ),
        (
            "/producer",
            vec![
                "kind",
                "repository",
                "workflow_path",
                "workflow_commit",
                "collector",
                "run_id",
                "run_attempt",
                "job",
                "artifact_name",
            ],
        ),
        (
            "/build",
            vec![
                "source_commit",
                "source_tree",
                "host_target",
                "toolchain",
                "features",
                "environment",
                "dependency_graph",
                "covered_inputs",
                "absent_inputs",
            ],
        ),
        ("/build/toolchain", vec!["path", "sha256"]),
        ("/cases", vec!["requested", "executed", "raw_output"]),
        ("/cases/requested/0", vec!["name", "executable_sha256"]),
        ("/cases/executed/0", vec!["case", "outcome"]),
        ("/adequacy", vec!["mapping", "endorsement"]),
        (
            "/adequacy/mapping",
            vec![
                "obligation",
                "reviewer_identity",
                "executor_identity",
                "assertions",
            ],
        ),
        (
            "/adequacy/mapping/assertions/0",
            vec!["case", "assertion", "production_inputs", "counterfactual"],
        ),
        (
            "/adequacy/endorsement",
            vec![
                "kind",
                "repository",
                "pull_request",
                "id",
                "author",
                "source_commit",
                "body_sha256",
                "mapping_sha256",
            ],
        ),
    ] {
        let mut value = golden.clone();
        let object = value.pointer_mut(path).unwrap();
        *object = Value::Array(keys.iter().map(|key| object[*key].clone()).collect());
        if check(&value).is_ok() {
            accepted.push(path);
        }
    }
    assert!(
        accepted.is_empty(),
        "accepted positional object encodings at {accepted:?}"
    );
}

#[test]
fn alternate_event_mapping_and_endorsement_forms_parse_without_granting_trust() {
    let mut value: Value = serde_json::from_str(GOLDEN).unwrap();
    value["scope"]["event"] = json!({
        "lifecycle_path":"onboarding", "approval_commit":"b".repeat(40),
        "approval_recorded_at":"2026-10-23T00:00:00Z"
    });
    value["scope"]["enrollment_origin"] = json!("production");
    let mapping_obligation =
        json!({"operation":"run", "capability":"agent_api.run", "target":"linux"});
    value["scope"]["obligation"] = mapping_obligation.clone();
    value["adequacy"]["mapping"]["obligation"] = mapping_obligation;
    let mapping: xtask::depth_evidence::AdequacyMapping =
        serde_json::from_value(value["adequacy"]["mapping"].clone()).unwrap();
    value["adequacy"]["endorsement"]["mapping_sha256"] = json!(mapping.sha256().unwrap());
    value["adequacy"]["endorsement"]["kind"] = json!("pr_review");
    for kind in ["source", "binary", "fixture"] {
        value["provenance"][0]["kind"] = json!(kind);
        assert!(check(&value).is_ok());
    }
    // These host names/origin labels are syntactic claims. Parsing does not recognize a producer
    // or authenticate an independent review, even with an internally consistent mapping digest.
}

#[test]
fn truthfully_recorded_nonqualifying_runs_are_not_erased_by_the_reader() {
    let mut value: Value = serde_json::from_str(GOLDEN).unwrap();
    for outcome in ["failed", "ignored", "skipped", "aborted"] {
        value["cases"]["executed"][0]["outcome"] = json!(outcome);
        assert!(check(&value).is_ok(), "{outcome}");
    }
    value["cases"]["executed"] = json!([]);
    assert!(check(&value).is_ok()); // Missing bound cases need a later unverified verdict.
    value["cases"]["requested"] = json!([]);
    value["build"]["covered_inputs"] = json!([]);
    value["provenance"] = json!([]);
    assert!(check(&value).is_ok()); // Zero tests/missing provenance confer no verification.
}
