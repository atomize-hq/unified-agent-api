use super::*;
use serde_json::json;

#[test]
fn reporting_context_literal_canonical_bytes_digest_version_and_bounds() {
    let bytes = include_bytes!("context-v1.json");
    let record = ContextRecord::parse(bytes).unwrap();
    assert_eq!(record.canonical().unwrap(), bytes);
    assert_eq!(
        record.identity().unwrap(),
        "63e791e20e3d0828b64e1e258fe565868afa6d39fe673379da12207c5e4e4c6a"
    );
    assert_ne!(record.identity().unwrap(), digest(admission::DOMAIN, bytes));
    let mut spaced = bytes.to_vec();
    spaced.push(b'\n');
    assert_eq!(
        ContextRecord::parse(&spaced).unwrap_err(),
        "noncanonical reporting context"
    );
    for field in ["schema_version", "unknown", "selected_mode"] {
        let mut value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
        value[field] = json!(2);
        assert!(ContextRecord::parse(&canonical(&value).unwrap()).is_err());
    }
    assert_eq!(
        ContextRecord::parse(&vec![b' '; RECORD_LIMIT + 1]).unwrap_err(),
        "reporting context exceeds 64 KiB"
    );
    let mut large = record.clone();
    large.repository.name = "x".repeat(RECORD_LIMIT);
    assert_eq!(
        large.canonical().unwrap_err(),
        "reporting context exceeds 64 KiB"
    );
    for mutation in 0..5 {
        let mut context = record.clone();
        match mutation {
            0 => context.invocation.substantive_invoked = true,
            1 => context.enrolled_substantive_path_not_invoked = false,
            2 => context.reporting_outcome = "failure".into(),
            3 => context.reporting_outcome = "aborted".into(),
            _ => context.collection_complete = false,
        }
        assert!(!context.successful());
    }
}
