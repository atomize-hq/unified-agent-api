//! Binding and candidate execution tests, independent of unavailable onboarding route scope.
use super::*;
use crate::{agent_lifecycle as life, depth_record::onboarding::OnboardingApproval};
use std::os::unix::fs::PermissionsExt;

const APPROVAL: &str = "docs/agents/lifecycle/codex-cli-onboarding/governance/approved-agent.toml";
const STATE: &str = "docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json";
const RECORD: &str = "cli_manifests/codex/reports/0.156.1/depth-record.json";
const RUN_ID: &str = "create-candidate";

fn write(root: &Path, path: &str, data: impl AsRef<[u8]>) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, data).unwrap();
}

fn args() -> Args {
    Args {
        approval: APPROVAL.into(),
        dry_run: false,
        write: true,
        requested_tier: RequestedTier::Default,
        minimal_justification_file: None,
        allow_rich_surface: vec![],
        run_id: Some(RUN_ID.into()),
        codex_binary: None,
    }
}

fn setup(depth: bool) -> (tempfile::TempDir, RuntimeContext) {
    let root = tempfile::tempdir().unwrap();
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut approval = fs::read_to_string(repo.join(APPROVAL)).unwrap();
    approval.push_str("\n[descriptor.maintenance]\nmode = \"explicitly_deferred\"\n[descriptor.maintenance.deferral]\nreason = \"fixture\"\nfollow_up = \"fixture\"\napproved_scope = \"create_lane_closeout\"\n");
    write(root.path(), APPROVAL, &approval);
    write(
        root.path(),
        "docs/agents/selection/cli-agent-selection-packet.md",
        "fixture\n",
    );
    write(
        root.path(),
        crate::agent_registry::REGISTRY_RELATIVE_PATH,
        fs::read(repo.join(crate::agent_registry::REGISTRY_RELATIVE_PATH)).unwrap(),
    );
    let mut state: life::LifecycleState =
        serde_json::from_slice(&fs::read(repo.join(STATE)).unwrap()).unwrap();
    state.lifecycle_stage = life::LifecycleStage::Enrolled;
    state.support_tier = life::SupportTier::Bootstrap;
    state.approval_artifact_sha256 = hex::encode(Sha256::digest(approval.as_bytes()));
    state.active_runtime_evidence_run_id = None;
    state.required_evidence = life::required_evidence_for_stage(state.lifecycle_stage).to_vec();
    state.satisfied_evidence = state.required_evidence.clone();
    state.publication_packet_path = None;
    state.publication_packet_sha256 = None;
    state.implementation_summary = None;
    write(root.path(), STATE, bytes(&state).unwrap());
    // Build ordinary context first. This unit fixture grants no isolated onboarding route.
    let mut context = build_context(root.path(), &args()).unwrap();
    if depth {
        approval.push_str("\n[support_depth]\nversion = \"0.156.1\"\n");
        write(root.path(), APPROVAL, &approval);
        context.approval = load_approval_artifact(root.path(), APPROVAL).unwrap();
        context.input_contract.approval_artifact_sha256 = context.approval.sha256.clone();
        context
            .input_contract
            .allowed_write_paths
            .retain(|p| !p.contains("/snapshots/"));
        let mut record: crate::depth_record::DepthRecord =
            serde_json::from_str(include_str!("../../tests/fixtures/depth-core/record.json"))
                .unwrap();
        record.event = OnboardingApproval::parse(&approval, APPROVAL)
            .unwrap()
            .event;
        record.policy.selection.enrollment.lifecycle_path =
            crate::agent_registry::support_depth::LifecyclePath::Onboarding;
        record.policy.identity = crate::depth_record::part_identity(&record.policy).unwrap();
        record.obligations = None;
        record.execution = None;
        record.acceptance_entries.clear();
        write(root.path(), RECORD, bytes(&record).unwrap());
        state.approval_artifact_sha256 = context.approval.sha256.clone();
        state.support_depth = Some(
            life::Preserved::new(life::LifecycleDepth {
                policy_identity: record.policy.identity,
                obligations_identity: None,
            })
            .unwrap(),
        );
        state.validate().unwrap();
        write(root.path(), STATE, bytes(&state).unwrap());
        context.lifecycle_state = state;
        context.input_contract.support_depth =
            SupportDepthContract::copy_from(&context.approval, &context.lifecycle_state).unwrap();
    }
    persist_dry_run_artifacts(&context).unwrap();
    (root, context)
}

#[path = "create_lane_identity_tests.rs"]
mod identity;

fn prepared_prompt(context: &RuntimeContext) -> String {
    read_string(&context.run_dir.join(PROMPT_FILE_NAME)).unwrap()
}

#[test]
fn create_candidate_preparation_requires_current_onboarding_event() {
    let (root, context) = setup(true);
    check_prepared(
        root.path(),
        &context,
        &context.input_contract,
        &prepared_prompt(&context),
    )
    .unwrap();
}

#[test]
fn create_candidate_preparation_refuses_maintenance_and_earlier_events_with_matching_identities() {
    for maintenance in [true, false] {
        let (root, context) = setup(true);
        let raw = read_string(&root.path().join(RECORD)).unwrap();
        let mut record: crate::depth_record::DepthRecord = serde_json::from_str(&raw).unwrap();
        if maintenance {
            let original: crate::depth_record::DepthRecord =
                serde_json::from_str(include_str!("../../tests/fixtures/depth-core/record.json"))
                    .unwrap();
            record.event = original.event;
            record.policy.selection.enrollment.lifecycle_path =
                crate::agent_registry::support_depth::LifecyclePath::Maintenance;
            record.policy.identity = crate::depth_record::part_identity(&record.policy).unwrap();
        } else {
            record.event = crate::depth_record::Event::Onboarding {
                approval_commit: "a".repeat(40),
                approval_recorded_at: "2024-01-01T00:00:00Z".into(),
            };
        }
        write(root.path(), RECORD, bytes(&record).unwrap());
        let mut context = context;
        context.lifecycle_state.support_depth = Some(
            life::Preserved::new(life::LifecycleDepth {
                policy_identity: record.policy.identity,
                obligations_identity: None,
            })
            .unwrap(),
        );
        let error = check_prepared(
            root.path(),
            &context,
            &context.input_contract,
            &prepared_prompt(&context),
        )
        .unwrap_err();
        assert!(error.to_string().contains("Event"), "{error}");
    }
}

#[test]
fn create_candidate_preparation_refuses_caller_absolute_prompt() {
    let (root, context) = setup(true);
    let prompt = prepared_prompt(&context).replace(
        &format!("{RUNTIME_RUNS_ROOT}/{RUN_ID}"),
        &context.run_dir.to_string_lossy(),
    );
    assert!(check_prepared(root.path(), &context, &context.input_contract, &prompt).is_err());
}

#[test]
fn create_candidate_prompt_follower_writes_only_candidate_handoff_before_delivery() {
    for passed in [true, false] {
        let (root, context) = setup(true);
        let before = fs::read(context.run_dir.join(HANDOFF_FILE_NAME)).unwrap();
        let host = tempfile::tempdir().unwrap();
        let binary = host.path().join("executor.sh");
        let handoff = serde_json::json!({"agent_id":"codex", "manifest_root":"cli_manifests/codex", "runtime_lane_complete":true, "publication_refresh_required":true, "required_commands":life::REQUIRED_PUBLICATION_COMMANDS, "blockers":[]});
        fs::write(&binary, format!("#!/bin/sh\nset -eu\ntarget=$(sed -n 's/^- Update `\\([^`]*\\)` so it passes validation with:/\\1/p')\ntest -n \"$target\"\ncat > \"$target\" <<'JSON'\n{handoff}\nJSON\nexit {}\n", if passed { 0 } else { 7 })).unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
        let mut args = args();
        args.codex_binary = Some(binary.to_string_lossy().into_owned());
        let candidate = Candidate::prepare(
            root.path(),
            &format!("{RUNTIME_RUNS_ROOT}/{RUN_ID}"),
            vec![],
        )
        .unwrap();
        let (execution, observed) = candidate.execute_observed(|| {
            codex_exec::capture_depth_execution(
                root.path(),
                &candidate.root,
                &context,
                &args,
                &prepared_prompt(&context),
            )
        });
        let (evidence, _) = execution.unwrap();
        assert_eq!(evidence.exit_code, if passed { 0 } else { 7 });
        let observed = observed.unwrap();
        candidate.check(&observed).unwrap();
        validate_handoff(
            &candidate
                .root
                .join(&candidate.run_dir_rel)
                .join(HANDOFF_FILE_NAME),
            &context,
        )
        .unwrap();
        assert_eq!(
            fs::read(context.run_dir.join(HANDOFF_FILE_NAME)).unwrap(),
            before
        );
        assert_eq!(
            tree::capture(root.path(), &Default::default()).unwrap(),
            candidate.before
        );
        let delivery = candidate_execution::finalized_tree(
            &candidate.before,
            Some(&observed),
            passed,
            Tree::new(),
        );
        candidate
            .deliver(root.path(), &delivery, passed, None)
            .unwrap();
        if passed {
            validate_handoff(&context.run_dir.join(HANDOFF_FILE_NAME), &context).unwrap();
        } else {
            assert_eq!(
                tree::capture(root.path(), &Default::default()).unwrap(),
                candidate.before
            );
        }
    }
}

#[test]
fn create_candidate_non_depth_prompt_keeps_original_rendering() {
    let (_root, context) = setup(false);
    assert_eq!(prepared_prompt(&context), render_prompt(&context));
    assert!(prepared_prompt(&context).contains(&context.run_dir.to_string_lossy().to_string()));
}
