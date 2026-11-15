//! Each A checker must retain its substantive refusal inside the borrowed scope.
use super::*;

pub(super) fn refusals(root: &Path, registry: &AgentRegistry) {
    let path = crate::capability_matrix::DEFAULT_OUT_PATH;
    let original = fs::read_to_string(root.join(path)).unwrap();
    write(root, path, format!("{original}\n"));
    let error = gates::check_shared(root).unwrap_err();
    assert!(
        error.ends_with("is stale; regenerate with `cargo run -p xtask -- capability-matrix`"),
        "{error}"
    );
    write(root, path, original);

    let path = "cli_manifests/depth_fixture/versions/0.1.0.json";
    let original = fs::read(root.join(path)).unwrap();
    let mut invalid: Value = serde_json::from_slice(&original).unwrap();
    invalid["s10_unknown_field"] = json!(true);
    write(root, path, serde_json::to_vec(&invalid).unwrap());
    let error = gates::check_shared(root).unwrap_err();
    assert!(
        error.starts_with("VERSION_METADATA_SCHEMA_INVALID error versions/0.1.0.json "),
        "{error}"
    );
    write(root, path, original);

    let schemas: Vec<_> = ["RULES.json", "SCHEMA.json", "VERSION_METADATA_SCHEMA.json"]
        .map(|name| {
            let path = format!("cli_manifests/codex/{name}");
            let bytes = fs::read(root.join(&path)).unwrap();
            fs::remove_file(root.join(&path)).unwrap();
            (path, bytes)
        })
        .into_iter()
        .collect();
    let error = gates::check_shared(root).unwrap_err();
    assert!(
        error.contains("cli_manifests/codex: I/O error: No such file or directory"),
        "{error}"
    );
    for (path, bytes) in schemas {
        write(root, &path, bytes);
    }

    let path = crate::runtime_advertising::DATA;
    let original = fs::read_to_string(root.join(path)).unwrap();
    write(
        root,
        path,
        format!("{original}\nconst S10_CORRUPT: bool = true;\n"),
    );
    assert_eq!(
        gates::check_shared(root).unwrap_err(),
        "runtime advertising data is stale or unsupported"
    );
    write(root, path, original);

    // A current matrix can still violate orthogonality. Enroll only the existing codex
    // inventory in this scratch fixture, so its non-allowlisted capabilities have one owner.
    let state =
        crate::agent_lifecycle::lifecycle_state_path_for_entry(registry.find("codex").unwrap());
    assert!(!root.join(&state).exists());
    write(root, &state, include_bytes!("../../../../../../docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json"));
    let path = crate::capability_matrix::DEFAULT_OUT_PATH;
    let original = fs::read(root.join(path)).unwrap();
    write(
        root,
        path,
        crate::capability_matrix::generate_markdown_for_workspace(root).unwrap(),
    );
    crate::capability_matrix::check_publication(root).unwrap();
    let error = gates::check_shared(root).unwrap_err();
    assert!(
        error.starts_with("published agents: [codex]\ncapability-matrix-audit failed:"),
        "{error}"
    );
    assert!(
        error.contains("- agent_api.tools.results.v1: supported by 1 agent(s): [codex]"),
        "{error}"
    );
    fs::remove_file(root.join(state)).unwrap();
    write(root, path, original);
    gates::check_shared(root).unwrap();
}
