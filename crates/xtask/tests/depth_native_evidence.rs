use serde_json::{json, Value};
use xtask::depth_evidence::{native::NativeReceipt, EvidenceReceipt};
fn native() -> Value {
    let mut v: Value =
        serde_json::from_str(include_str!("fixtures/depth-evidence-v1.json")).unwrap();
    v["schema_version"] = json!(2);
    v["scope"]["enrollment_origin"] = json!("isolated");
    v["producer"] = json!({"kind":"maintainer_native","repository":{"name":"owner/proof","id":1,"owner_id":2},
        "pull_request":3,"registration":{"id":4,"body_sha256":"a".repeat(64)},
        "completion":{"id":5,"body_sha256":"b".repeat(64)},
        "collector":{"path":"scripts/collector.rs","sha256":"c".repeat(64)}});
    v
}
#[test]
fn native_revision_and_host_references_are_not_an_authority_bypass() {
    assert!(NativeReceipt::parse(&native().to_string()).is_ok());
    assert!(EvidenceReceipt::parse(&native().to_string()).is_err());
    for (field, bad) in [("schema_version", json!(1)), ("scope", Value::Null)] {
        let mut v = native();
        v[field] = bad;
        assert!(NativeReceipt::parse(&v.to_string()).is_err(), "{field}");
    }
    for key in ["id", "owner_id"] {
        let mut v = native();
        v["producer"]["repository"][key] = json!(0);
        assert!(NativeReceipt::parse(&v.to_string()).is_err(), "{key}");
    }
    let mut v = native();
    v["producer"]["completion"]["body_sha256"] = json!("not-a-digest");
    assert!(NativeReceipt::parse(&v.to_string()).is_err());
}
#[test]
fn native_common_claims_keep_exact_mapping_and_shape_guards() {
    let mut v = native();
    v["adequacy"]["endorsement"]["mapping_sha256"] = json!("0".repeat(64));
    assert!(NativeReceipt::parse(&v.to_string()).is_err());
    let raw = native().to_string();
    assert!(NativeReceipt::parse(&raw.replacen(
        "\"schema_version\":2",
        "\"schema_version\":2,\"schema_version\":2",
        1
    ))
    .is_err());
    let mut v = native();
    v["producer"]["trusted"] = json!(true);
    assert!(NativeReceipt::parse(&v.to_string()).is_err());
    let mut v = native();
    v["scope"]["policy_identity"] = json!("bad");
    assert!(NativeReceipt::parse(&v.to_string()).is_err());
}
#[test]
fn native_attempt_payload_forms_reject_ambiguous_or_incomplete_objects() {
    use xtask::depth_evidence::native::{Bundle, Finish, Plan, Start};
    let r = NativeReceipt::parse(&native().to_string()).unwrap();
    let file = json!({"path":"reports/file.json","sha256":"a".repeat(64)});
    let observed = json!({"path":"reports/context.json","sha256":"b".repeat(64)});
    let start = json!({"schema_version":1,"profile":"isolated-codex-native-v1","repository":r.producer.repository,"pull_request":3,"packet_head":"a".repeat(40),"scope":r.scope,"plan":file});
    let finish = json!({"schema_version":1,"registration":r.producer.registration,"evidence_commit":"b".repeat(40),"bundle":file,"executor_identity":"actual-executor","population_through_id":4,"registrations":[4],"execution_attested":true,"outcomes_attested":true,"registration_order_attested":true,"completeness_attested":true});
    let plan = json!({"schema_version":1,"scope":r.scope,"packet_head":"a".repeat(40),"build":r.build,"native":{"version":"0.156.1","target":"x86_64-unknown-linux-musl","executable_sha256":"c".repeat(64),"acquisition":file},"context":file,"collector":file,"requested":r.cases.requested});
    let bundle = json!({"schema_version":1,"plan":file,"observed_context":observed,"claims":[file],"files":[file,observed]});
    type Accepts = fn(&str) -> bool;
    let forms: [(Value, Accepts); 4] = [
        (start, |s| Start::parse(s).is_ok()),
        (finish, |s| Finish::parse(s).is_ok()),
        (plan, |s| Plan::parse(s).is_ok()),
        (bundle, |s| Bundle::parse(s).is_ok()),
    ];
    for (v, accepts) in forms {
        assert!(accepts(&v.to_string()));
        for key in v.as_object().unwrap().keys() {
            let mut bad = v.clone();
            bad.as_object_mut().unwrap().remove(key);
            assert!(!accepts(&bad.to_string()), "missing {key}");
            let mut bad = v.clone();
            bad[key] = Value::Null;
            assert!(!accepts(&bad.to_string()), "null {key}");
        }
        let mut bad = v.clone();
        bad["trusted"] = json!(true);
        assert!(!accepts(&bad.to_string()));
        let mut bad = v.clone();
        bad["schema_version"] = json!(2);
        assert!(!accepts(&bad.to_string()));
        let raw = v.to_string().replacen(
            "\"schema_version\":1",
            "\"schema_version\":1,\"schema_version\":1",
            1,
        );
        assert!(!accepts(&raw));
        assert!(!accepts(
            &serde_json::to_string(&v.as_object().unwrap().values().collect::<Vec<_>>()).unwrap()
        ));
    }
}
