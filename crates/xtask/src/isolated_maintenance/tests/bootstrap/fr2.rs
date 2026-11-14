//! One real ON1/acquisition/declaration fixture supplies the FR2a ON2 proof matrix.
use super::*;
use crate::depth_acceptance::admission::{
    self,
    freeze::{
        self,
        authority::{Observation, Writer},
    },
    Checkpoint, Generation, Reason,
};
use clap::Parser;

fn variables() -> Writer {
    Writer::Onboarding {
        run_id: "fr2-freeze".into(),
        contract_generated_at: "2026-11-14T00:00:00Z".into(),
        packet_generated_at: "2026-11-14T00:00:01Z".into(),
        requested_tier: crate::runtime_follow_on::RequestedTier::Default,
        minimal_justification_file: None,
        allow_rich_surface: vec![],
    }
}
fn run(f: &Fixture) {
    let args = crate::runtime_follow_on::Args::parse_from([
        "runtime-follow-on",
        "--dry-run",
        "--approval",
        APPROVAL,
        "--run-id",
        "fr2-freeze",
    ]);
    crate::runtime_follow_on::depth::dry_run_at(
        f.root.path(),
        args,
        &mut Vec::new(),
        "2026-11-14T00:00:00Z".into(),
        "2026-11-14T00:00:01Z".into(),
    )
    .unwrap();
}

#[test]
fn fr2_on2_complete_tree_canonical_outputs_authority_and_real_in1() {
    let f = public_on1::declared_chain_with_host_approval();
    let _installed = f.install();
    let parent = head(&f);
    let statement_path = crate::depth_record::acquisition::statement_path(PACK, "0.2.0");
    let statement = fs::read(f.root.path().join(&statement_path)).unwrap();
    run(&f);
    let freeze_commit = candidate(&f, &parent);
    let publication = f.resolve().unwrap().admission_publication().unwrap();
    let parent_object = admission::emitter::object(f.root.path(), &parent).unwrap();
    let candidate_object = admission::emitter::object(f.root.path(), &freeze_commit).unwrap();
    let record = crate::depth_record::DepthRecord::parse(
        &fs::read_to_string(f.root.path().join(RECORD)).unwrap(),
        "depth_fixture",
        "0.2.0",
    )
    .unwrap();
    let observation = Observation {
        repository: publication.authority.repository.clone(),
        predecessor: parent_object.clone(),
        generation: Generation::from_record(&record),
        policy: publication.authority.policy.clone(),
        writer: variables(),
    };
    f.api.3.put(
        &format!("freeze-inputs/{freeze_commit}"),
        serde_json::to_value(&observation).unwrap(),
    );
    let (plan, tree) = freeze::expected(
        f.root.path(),
        &parent_object,
        &candidate_object,
        &record,
        &observation,
    )
    .unwrap();
    plan.require_equal(&tree).unwrap();
    // Independent canonical assertions cover the actual delivered record, lifecycle and all seven files.
    assert!(record.obligations.is_some());
    assert!(record.execution.is_none());
    assert!(record.acceptance_entries.is_empty());
    assert_eq!(tree[&statement_path].bytes, statement);
    let run = "docs/agents/.uaa-temp/runtime-follow-on/runs/fr2-freeze";
    let files: Vec<_> = tree
        .keys()
        .filter(|path| path.starts_with(&format!("{run}/")))
        .collect();
    assert_eq!(files.len(), 7);
    assert_eq!(tree[&format!("{run}/written-paths.json")].bytes, b"[]");
    let report: serde_json::Value =
        serde_json::from_slice(&tree[&format!("{run}/validation-report.json")].bytes).unwrap();
    assert_eq!(
        report,
        json!({"workflow_version":"runtime_follow_on_v2","generated_at":"2026-11-14T00:00:01Z","run_id":"fr2-freeze","status":"dry_run_ready","checks":[],"errors":[]})
    );
    let status: serde_json::Value =
        serde_json::from_slice(&tree[&format!("{run}/run-status.json")].bytes).unwrap();
    assert_eq!(status["mode"], "dry_run");
    assert_eq!(status["handoff_ready"], false);
    let prompt = String::from_utf8(tree[&format!("{run}/codex-prompt.md")].bytes.clone()).unwrap();
    assert_eq!(prompt, concat!(
        "# Wrapper-only implementation\n\n",
        "Read docs/agents/.uaa-temp/runtime-follow-on/runs/fr2-freeze/input-contract.json. Implement and test the approved wrapper in `crates/depth_fixture`. Preserve the frozen support_depth identities and all acquired files, including generated coverage. No backend or agent_api test is due.\n\n",
        "- Allowed source paths: crates/depth_fixture/**\n",
        "- Update `docs/agents/.uaa-temp/runtime-follow-on/runs/fr2-freeze/handoff.json` so it passes validation with:\n",
        "  agent_id = depth_fixture; manifest_root = cli_manifests/depth_fixture; runtime_lane_complete = true; publication_refresh_required = true; blockers = [].\n",
        "- required_commands must match input-contract.json exactly.\n",
        "- Deliver substantive wrapper Rust source and wrapper tests; controller diagnostics are not implementation evidence.\n",
        "\nDepth candidate boundary: handoff.json is the only run file you may write. The input contract, prompt, lifecycle state, depth record, approval and depth-acquisition statement are immutable. The controller writes all streams, execution evidence, validation, status, written paths and summary after observation. Do not write snapshots.\n",
    ));
    let contract: serde_json::Value =
        serde_json::from_slice(&tree[&format!("{run}/input-contract.json")].bytes).unwrap();
    assert_eq!(
        contract["docs_to_read"],
        json!([
            "docs/specs/cli-agent-onboarding-charter.md",
            "docs/specs/support-depth-contract.md"
        ])
    );
    assert_eq!(contract["generated_at"], "2026-11-14T00:00:00Z");
    assert!(String::from_utf8(
        tree[crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH]
            .bytes
            .clone()
    )
    .unwrap()
    .contains("EmbeddedRuntimeSupportRecord"));
    let json: serde_json::Value =
        serde_json::from_slice(&tree[crate::support_matrix::JSON_OUTPUT_PATH].bytes).unwrap();
    assert!(json["rows"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["agent"] == "depth_fixture"
            && row["version"] == "0.2.0"
            && row["depth"].is_object()));
    let source = crate::runtime_advertising::source::Source::tip(f.root.path(), &parent).unwrap();
    assert_eq!(
        tree[crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH].bytes,
        source
            .read(crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH)
            .unwrap()
            .as_bytes()
    );
    for path in [
        "Cargo.toml",
        "Cargo.lock",
        "build.rs",
        ".github/workflows/ci.yml",
        APPROVAL,
        STATE,
        RECORD,
        "crates/xtask/src/depth_ci/profile.rs",
        "docs/unrelated.md",
        "docs/agents/.uaa-temp/runtime-follow-on/runs/fr2-freeze/codex-prompt.md",
        crate::support_matrix::JSON_OUTPUT_PATH,
        crate::support_matrix::MARKDOWN_OUTPUT_PATH,
        crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
        &statement_path,
    ] {
        let mut substituted = tree.clone();
        substituted.insert(
            path.into(),
            crate::freeze_transition::Entry::file("100644", b"substitution".to_vec()),
        );
        assert!(plan.require_equal(&substituted).is_err(), "{path}");
    }
    // Candidate expected-output files and local predecessor substitutions cannot influence derivation.
    write(f.root.path(), "expected-output/record.json", "forged");
    write(f.root.path(), RECORD, "local substitution");
    write(f.root.path(), APPROVAL, "local approval substitution");
    write(
        f.root.path(),
        &format!("{ROOT}/RULES.json"),
        "local rules substitution",
    );
    let (again, _) = freeze::expected(
        f.root.path(),
        &parent_object,
        &candidate_object,
        &record,
        &observation,
    )
    .unwrap();
    assert_eq!(again, plan);
    git(f.root.path(), &["reset", "--hard", &freeze_commit]);
    fs::remove_dir_all(f.root.path().join("expected-output")).unwrap();
    // Every identity is compared against independent authority before rendering.
    assert_eq!(
        freeze::classify(
            f.root.path(),
            &publication,
            &parent_object,
            &candidate_object,
            &record,
            None
        )
        .reason,
        Reason::FreezeAuthorityUnavailable
    );
    for mutation in 0..8 {
        let mut forged = observation.clone();
        match mutation {
            0 => forged.repository.id += 1,
            1 => forged.predecessor = candidate_object.clone(),
            2 => forged.policy.implementation = "a".repeat(64),
            3 => forged.generation.policy = "a".repeat(64),
            4 => forged.generation.obligations = None,
            5 => forged.generation.selection.enrollment.version = "9.9.9".into(),
            6 => {
                forged.generation.event = crate::depth_record::Event::Onboarding {
                    approval_commit: "abc9999".into(),
                    approval_recorded_at: "2026-11-14T00:00:00Z".into(),
                };
            }
            _ => {
                forged.generation.event = crate::depth_record::Event::Maintenance {
                    request_commit: "abc1234".into(),
                    request_recorded_at: "2026-11-14T00:00:00Z".into(),
                    trigger_kind: "upstream_release_detected".into(),
                    opened_from: ".github/workflows/ci.yml".into(),
                }
            }
        }
        assert_eq!(
            freeze::classify(
                f.root.path(),
                &publication,
                &parent_object,
                &candidate_object,
                &record,
                Some(&forged)
            )
            .reason,
            Reason::FreezeAuthorityMismatch
        );
    }
    let mut forged = observation.clone();
    if let Writer::Onboarding {
        packet_generated_at,
        ..
    } = &mut forged.writer
    {
        *packet_generated_at = "2026-11-14T00:00:02Z".into();
    }
    assert_eq!(
        freeze::classify(
            f.root.path(),
            &publication,
            &parent_object,
            &candidate_object,
            &record,
            Some(&forged)
        )
        .reason,
        Reason::FreezeTreeMismatch
    );
    check_integration(f.root.path(), &parent).unwrap();
    let retained = f
        .api
        .3
        .records()
        .into_iter()
        .find(|r| r.candidate.commit == freeze_commit)
        .unwrap();
    assert_eq!(retained.checkpoint, Checkpoint::RequestFreeze);
    assert_eq!(
        retained.transition.unwrap().writer_inputs,
        observation.identity().unwrap()
    );
    // Applicability is preserved for an additional edit, with execution required at actual IN1.
    write(f.root.path(), "docs/unrelated.md", "extra edit");
    let extra = candidate(&f, &parent);
    f.api.3.put(
        &format!("freeze-inputs/{extra}"),
        serde_json::to_value(&observation).unwrap(),
    );
    check_integration(f.root.path(), &parent).unwrap();
    let retained = f
        .api
        .3
        .records()
        .into_iter()
        .find(|r| r.candidate.commit == extra)
        .unwrap();
    assert_eq!(retained.checkpoint, Checkpoint::ExecutionRequired);
    assert!(retained.reasons.contains(&Reason::FreezeTreeMismatch));
    // Advance only after the genuine real-IN1 checkpoint publication has been asserted.
    git(f.root.path(), &["reset", "--hard", &freeze_commit]);
    f.host();
    local(&f, &freeze_commit);
    check_integration(f.root.path(), &parent).unwrap();
}
