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
