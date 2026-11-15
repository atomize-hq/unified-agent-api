//! Structurally sealed ON2 vector for refusal-only fixtures. Placeholder input digests grant
//! no acquisition or evidence authority. The connected E.8 leg separately uses actual records.
pub fn record() -> Vec<u8> {
    use xtask::depth_record::{part_identity, DepthRecord};
    fn expand(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::String(text) if text.starts_with("{{") => {
                *text = if text == "{{APPROVAL_COMMIT}}" {
                    "a".repeat(40)
                } else {
                    "b".repeat(64)
                };
            }
            serde_json::Value::Array(values) => values.iter_mut().for_each(expand),
            serde_json::Value::Object(values) => values.values_mut().for_each(expand),
            _ => {}
        }
    }
    let mut value = serde_json::from_str(include_str!(
        "../fixtures/fr2-canonical/on2/cli_manifests/depth_fixture/reports/0.2.0/depth-record.json"
    ))
    .unwrap();
    expand(&mut value);
    let mut record: DepthRecord = serde_json::from_value(value).unwrap();
    record.policy.identity = part_identity(&record.policy).unwrap();
    let obligations = record.obligations.as_mut().unwrap();
    obligations.policy_identity = record.policy.identity.clone();
    obligations.identity = part_identity(obligations).unwrap();
    let raw = serde_json::to_vec_pretty(&record).unwrap();
    DepthRecord::parse(std::str::from_utf8(&raw).unwrap(), "depth_fixture", "0.2.0").unwrap();
    raw
}
