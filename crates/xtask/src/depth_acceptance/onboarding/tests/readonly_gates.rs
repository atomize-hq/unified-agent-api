//! Slice 10a consumes the actual ON3 checkpoint and coherent proposed Publication entry.
use super::*;
use crate::agent_maintenance::closeout_proposal::{IsolatedView, Proposal};
use crate::isolated_maintenance::CapturedRegistry;

pub(super) fn check_connected(
    caller: &Path,
    proposal: &Proposal,
    captured: &CapturedRegistry,
    view: &IsolatedView<'_>,
    scratch: &Path,
    record: &DepthRecord,
) {
    let before = crate::candidate_execution::tree::capture(caller, &Default::default()).unwrap();
    let check = || crate::readonly_gates::check_proposed(caller, proposal, captured, view, record);
    check().unwrap();

    let mut stale = record.clone();
    stale.acceptance_entries.clear();
    assert_eq!(
        crate::readonly_gates::check_proposed(caller, proposal, captured, view, &stale)
            .unwrap_err(),
        "read-only gates require the exact proposed record"
    );

    // Inject only after the exact ON3 proposal has passed its owning predicates and evidence
    // evaluation. Restore before closing observations, so the refusal belongs to the A checker.
    for (path, changed, refusal) in [
        (
            crate::support_matrix::JSON_OUTPUT_PATH,
            b"{}\n".to_vec(),
            "is stale; regenerate with `cargo run -p xtask -- support-matrix`",
        ),
        (
            crate::capability_matrix::DEFAULT_OUT_PATH,
            b"stale capability publication\n".to_vec(),
            "is stale; regenerate with `cargo run -p xtask -- capability-matrix`",
        ),
        (
            "cli_manifests/depth_fixture/versions/0.2.0.json",
            {
                let mut metadata: Value = serde_json::from_slice(
                    &std::fs::read(scratch.join("cli_manifests/depth_fixture/versions/0.2.0.json"))
                        .unwrap(),
                )
                .unwrap();
                metadata["s10_unknown_field"] = json!(true);
                encode(&metadata).into_bytes()
            },
            "VERSION_METADATA_SCHEMA_INVALID error versions/0.2.0.json",
        ),
        (
            crate::runtime_advertising::DATA,
            {
                let mut bytes =
                    std::fs::read(scratch.join(crate::runtime_advertising::DATA)).unwrap();
                bytes.extend(b"\nconst S10_CORRUPT: bool = true;\n");
                bytes
            },
            "runtime advertising data is stale or unsupported",
        ),
    ] {
        let error = crate::readonly_gates::test_hooks::with_shared_changes(
            vec![(path.into(), Some(changed))],
            check,
        )
        .unwrap_err();
        assert!(error.contains(refusal), "{path}: {error}");
    }

    let state = crate::agent_lifecycle::lifecycle_state_path_for_entry(
        captured.registry().find("codex").unwrap(),
    );
    assert!(!scratch.join(&state).exists());
    let state_bytes = include_bytes!("../../../../../../docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json").to_vec();
    std::fs::create_dir_all(scratch.join(&state).parent().unwrap()).unwrap();
    std::fs::write(scratch.join(&state), &state_bytes).unwrap();
    let matrix = crate::capability_matrix::generate_markdown_for_workspace(scratch).unwrap();
    std::fs::remove_file(scratch.join(&state)).unwrap();
    let error = crate::readonly_gates::test_hooks::with_shared_changes(
        vec![
            (state, Some(state_bytes)),
            (
                crate::capability_matrix::DEFAULT_OUT_PATH.into(),
                Some(matrix.into_bytes()),
            ),
        ],
        check,
    )
    .unwrap_err();
    assert!(
        error.starts_with("published agents: [codex]\ncapability-matrix-audit failed:"),
        "{error}"
    );
    assert!(
        error.contains("- agent_api.tools.results.v1: supported by 1 agent(s): [codex]"),
        "{error}"
    );

    // The bound scratch view also refuses deletion of every prerequisite for a required root.
    let changes = ["RULES.json", "SCHEMA.json", "VERSION_METADATA_SCHEMA.json"]
        .map(|name| (format!("cli_manifests/codex/{name}"), None))
        .to_vec();
    let error = crate::readonly_gates::test_hooks::with_shared_changes(changes, check).unwrap_err();
    assert!(
        error.contains("cli_manifests/codex: I/O error: No such file or directory"),
        "{error}"
    );
    check().unwrap();
    assert_eq!(
        crate::candidate_execution::tree::capture(caller, &Default::default()).unwrap(),
        before
    );
}

#[test]
#[cfg(target_os = "linux")]
fn readonly_gates_proposal_runs_every_a_checker() {
    crate::isolated_maintenance::connected_onboarding_publication();
}
