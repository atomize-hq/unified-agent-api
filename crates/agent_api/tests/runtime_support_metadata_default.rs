use std::fs;
use std::path::Path;

use agent_api::{list_runtime_support, resolve_runtime_support, RuntimeSupportRecord};

fn latest_validated_version(target_triple: &str) -> String {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("agent_api crate directory parent")
        .parent()
        .expect("workspace root");

    fs::read_to_string(
        workspace_root
            .join("cli_manifests/codex/pointers/latest_validated")
            .join(format!("{target_triple}.txt")),
    )
    .expect("read committed latest_validated pointer")
    .trim()
    .to_string()
}

#[test]
fn codex_runtime_support_metadata_is_available_without_backend_features() {
    let expected = RuntimeSupportRecord {
        runtime_family: "codex".to_string(),
        target_triple: "x86_64-unknown-linux-musl".to_string(),
        version: latest_validated_version("x86_64-unknown-linux-musl"),
    };
    let resolved = resolve_runtime_support("codex", "x86_64-unknown-linux-musl")
        .expect("resolve codex tuple without backend feature");
    assert_eq!(resolved, expected);

    let listed = list_runtime_support("codex").expect("list codex tuples without backend feature");
    assert!(listed.contains(&resolved), "{listed:?}");
}
