//! Invalid committed candidates must reach each standalone entry's substantive refusal.
use super::*;

pub(super) fn invalid_entries() {
    let (f, host) = setup();
    let root = f.root.path();
    let _route = f.install();
    let _reader = host.install();
    super::super::ma5_closeout::prepare(root).unwrap();
    super::super::ma5_closeout::advance(&f, &host);
    for (path, changed, check, refusal) in [
        (
            crate::support_matrix::JSON_OUTPUT_PATH,
            b"{}\n".to_vec(),
            crate::support_matrix::check_in_workspace as fn(&Path) -> Result<(), String>,
            "is stale; regenerate with `cargo run -p xtask -- support-matrix`",
        ),
        (
            crate::capability_matrix::DEFAULT_OUT_PATH,
            b"stale capability publication\n".to_vec(),
            crate::capability_matrix::check_in_workspace,
            "is stale; regenerate with `cargo run -p xtask -- capability-matrix`",
        ),
        (
            crate::runtime_advertising::DATA,
            {
                let mut bytes = fs::read(root.join(crate::runtime_advertising::DATA)).unwrap();
                bytes.extend(b"\nconst S10_CORRUPT: bool = true;\n");
                bytes
            },
            crate::runtime_advertising::check_in_workspace,
            "runtime advertising data is stale or unsupported",
        ),
    ] {
        let original = fs::read(root.join(path)).unwrap();
        write(root, path, changed);
        super::super::ma5_closeout::advance(&f, &host);
        let before = super::super::ma5_closeout::snapshot(root);
        let error = check(root).unwrap_err();
        assert!(error.contains(refusal), "{path}: {error}");
        assert_eq!(super::super::ma5_closeout::snapshot(root), before);
        write(root, path, original);
        super::super::ma5_closeout::advance(&f, &host);
    }

    let registry = registry(root).unwrap();
    let state =
        crate::agent_lifecycle::lifecycle_state_path_for_entry(registry.find("codex").unwrap());
    assert!(!root.join(&state).exists());
    write(root, &state, include_bytes!("../../../../../../docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json"));
    super::super::ma5_closeout::advance(&f, &host);
    // Keep the capability matrix current, leaving orthogonality as the failing A check.
    let path = crate::capability_matrix::DEFAULT_OUT_PATH;
    let original = fs::read(root.join(path)).unwrap();
    write(
        root,
        path,
        crate::capability_matrix::generate_markdown_for_workspace(root).unwrap(),
    );
    super::super::ma5_closeout::advance(&f, &host);
    let before = super::super::ma5_closeout::snapshot(root);
    crate::capability_matrix::check_in_workspace(root).unwrap();
    let error = crate::capability_publication::check_in_workspace(root).unwrap_err();
    assert!(
        error.starts_with("published agents: [codex]\ncapability-matrix-audit failed:"),
        "{error}"
    );
    assert!(
        error.contains("- agent_api.tools.results.v1: supported by 1 agent(s): [codex]"),
        "{error}"
    );
    assert_eq!(super::super::ma5_closeout::snapshot(root), before);
    fs::remove_file(root.join(state)).unwrap();
    write(root, path, original);
    super::super::ma5_closeout::advance(&f, &host);

    let path = "cli_manifests/depth_fixture/versions/0.1.0.json";
    let mut metadata: Value = serde_json::from_slice(&fs::read(root.join(path)).unwrap()).unwrap();
    metadata["s10_unknown_field"] = json!(true);
    write(root, path, serde_json::to_vec(&metadata).unwrap());
    let parent = git(root, &["rev-parse", "HEAD"]);
    super::super::ma5_closeout::advance(&f, &host);
    let before = super::super::ma5_closeout::snapshot(root);
    assert_eq!(
        super::super::publication::validate_candidate(root, &parent),
        2
    );
    let codes = crate::manifest_validate::violation_codes(crate::manifest_validate::Args {
        root: Some(root.join("cli_manifests/depth_fixture")),
        rules: None,
        schema: None,
        version_schema: None,
        mode: crate::manifest_validate::Mode::Check,
        json: false,
        tip: Some(parent),
    })
    .unwrap();
    assert_eq!(codes, ["VERSION_METADATA_SCHEMA_INVALID"]);
    assert_eq!(super::super::ma5_closeout::snapshot(root), before);
}
