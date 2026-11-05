//! Command-boundary refusals: the installed route still admits maintenance scope only.
use super::*;
use crate::{agent_lifecycle as life, runtime_follow_on as runtime};
use sha2::{Digest, Sha256};
use std::os::unix::fs::PermissionsExt;

const APPROVAL: &str = "docs/agents/lifecycle/codex-cli-onboarding/governance/approved-agent.toml";
const STATE: &str = "docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json";
const RECORD: &str = "cli_manifests/codex/reports/0.156.1/depth-record.json";
const STATEMENT: &str =
    "docs/agents/lifecycle/codex-cli-onboarding/governance/depth-acquisition/0.156.1.toml";
const RUN: &str = "docs/agents/.uaa-temp/runtime-follow-on/runs/create-candidate";

fn repo() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn setup() -> Fixture {
    let f = Fixture::new();
    let root = f.root.path();
    let mut approval = fs::read_to_string(repo().join(APPROVAL)).unwrap();
    approval.push_str("\n[descriptor.maintenance]\nmode = \"explicitly_deferred\"\n[descriptor.maintenance.deferral]\nreason = \"fixture\"\nfollow_up = \"fixture\"\napproved_scope = \"create_lane_closeout\"\n[support_depth]\nversion = \"0.156.1\"\n");
    write(root, APPROVAL, &approval);
    write(
        root,
        "docs/agents/selection/cli-agent-selection-packet.md",
        "fixture\n",
    );
    write(root, ".gitignore", "docs/agents/.uaa-temp/\n");
    write(root, STATEMENT, "# protected acquisition statement\n");
    let mut record: crate::depth_record::DepthRecord = serde_json::from_str(include_str!(
        "../../../tests/fixtures/depth-core/record.json"
    ))
    .unwrap();
    // Deliberately invalid for onboarding preparation, despite matching lifecycle P/O.
    let table: crate::agent_registry::support_depth::SupportDepth =
        serde_json::from_value(declaration()).unwrap();
    record.policy.selection.enrollment = table.enrollments.unwrap().remove(0);
    record.policy.identity = crate::depth_record::part_identity(&record.policy).unwrap();
    record.obligations = None;
    record.execution = None;
    record.acceptance_entries.clear();
    write(root, RECORD, serde_json::to_vec_pretty(&record).unwrap());
    crate::depth_record::DepthRecord::parse(
        &fs::read_to_string(root.join(RECORD)).unwrap(),
        "codex",
        "0.156.1",
    )
    .unwrap();
    let mut state: life::LifecycleState =
        serde_json::from_slice(&fs::read(repo().join(STATE)).unwrap()).unwrap();
    state.lifecycle_stage = life::LifecycleStage::Enrolled;
    state.support_tier = life::SupportTier::Bootstrap;
    state.approval_artifact_sha256 = hex::encode(Sha256::digest(approval.as_bytes()));
    state.active_runtime_evidence_run_id = None;
    state.required_evidence =
        life::required_evidence_for_stage(life::LifecycleStage::Enrolled).to_vec();
    state.satisfied_evidence = state.required_evidence.clone();
    state.publication_packet_path = None;
    state.publication_packet_sha256 = None;
    state.implementation_summary = None;
    state.support_depth = Some(
        life::Preserved::new(life::LifecycleDepth {
            policy_identity: record.policy.identity,
            obligations_identity: None,
        })
        .unwrap(),
    );
    state.validate().unwrap();
    write(root, STATE, serde_json::to_vec_pretty(&state).unwrap());
    f.commit();
    f
}
fn args(write: bool, binary: &str) -> runtime::Args {
    runtime::Args {
        approval: APPROVAL.into(),
        dry_run: !write,
        write,
        requested_tier: runtime::RequestedTier::Default,
        minimal_justification_file: None,
        allow_rich_surface: vec![],
        run_id: Some("create-candidate".into()),
        codex_binary: Some(binary.into()),
    }
}
fn executor(dir: &Path, script: &str) -> String {
    let binary = dir.join("executor.sh");
    fs::write(&binary, format!("#!/bin/sh\ncat >/dev/null\n{script}\n")).unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
    binary.to_string_lossy().into_owned()
}
fn run(f: &Fixture, binary: &str) -> Result<(), runtime::Error> {
    runtime::run_in_workspace(f.root.path(), args(true, binary), &mut vec![])
}
fn prepare(f: &Fixture, binary: &str) {
    runtime::run_in_workspace(f.root.path(), args(false, binary), &mut vec![]).unwrap();
}
fn json_file(root: &Path, path: &str) -> Value {
    serde_json::from_slice(&fs::read(root.join(path)).unwrap()).unwrap()
}

fn tree(root: &Path) -> crate::candidate_execution::tree::Tree {
    crate::candidate_execution::tree::capture(root, &Default::default()).unwrap()
}

#[test]
fn depth_create_refuses_maintenance_binding_before_attempt_or_executor() {
    let f = setup();
    let _installed = f.install();
    let host = tempfile::tempdir().unwrap();
    let binary = executor(host.path(), "touch executor-started");
    prepare(&f, &binary);
    let before = tree(f.root.path());
    let error = run(&f, &binary).unwrap_err().to_string();
    assert!(error.contains("Event"), "{error}");
    assert_eq!(tree(f.root.path()), before);
    assert_eq!(
        json_file(f.root.path(), STATE)["lifecycle_stage"],
        "enrolled"
    );
    assert!(!f.root.path().join("executor-started").exists());
}

#[test]
fn depth_create_refuses_input_identity_mismatch_before_attempt_or_executor() {
    for field in [
        "missing",
        "version",
        "policy_identity",
        "obligations_identity",
        "workflow_version",
    ] {
        let f = setup();
        let _installed = f.install();
        let host = tempfile::tempdir().unwrap();
        let binary = executor(host.path(), "touch executor-started");
        prepare(&f, &binary);
        let path = format!("{RUN}/input-contract.json");
        let mut contract = json_file(f.root.path(), &path);
        match field {
            "missing" => {
                contract.as_object_mut().unwrap().remove("support_depth");
            }
            "workflow_version" => contract[field] = json!("runtime_follow_on_v1"),
            "version" => contract["support_depth"][field] = json!("9.9.9"),
            _ => contract["support_depth"][field] = json!("a".repeat(64)),
        }
        write(
            f.root.path(),
            &path,
            serde_json::to_vec_pretty(&contract).unwrap(),
        );
        let before = tree(f.root.path());
        let error = run(&f, &binary).unwrap_err().to_string();
        assert!(error.contains("support_depth"), "{field}: {error}");
        // This fails before the deliberately stale record's Event check, and before an attempt.
        assert!(!error.contains("Event"), "{field}: {error}");
        assert_eq!(tree(f.root.path()), before);
        assert!(!f.root.path().join("executor-started").exists());
    }
}

#[test]
fn depth_create_refuses_invalid_binding_even_with_caller_absolute_prompt_before_attempt_or_executor(
) {
    let f = setup();
    let _installed = f.install();
    let host = tempfile::tempdir().unwrap();
    let binary = executor(host.path(), "touch executor-started");
    prepare(&f, &binary);
    let path = f.root.path().join(format!("{RUN}/codex-prompt.md"));
    let prompt = fs::read_to_string(&path)
        .unwrap()
        .replace(RUN, &f.root.path().join(RUN).to_string_lossy());
    fs::write(path, prompt).unwrap();
    let before = tree(f.root.path());
    let error = run(&f, &binary).unwrap_err().to_string();
    // OF-P8 now rejects this fixture's maintenance Event before candidate preparation
    // inspects the prompt. The adapter unit test separately proves the prompt refusal.
    assert!(
        error.contains("input contract support_depth") && error.contains("Event"),
        "{error}"
    );
    assert_eq!(tree(f.root.path()), before);
}

#[test]
fn depth_create_refuses_current_and_earlier_onboarding_scope_without_prerequisite() {
    for earlier in [false, true] {
        let f = setup();
        let host = tempfile::tempdir().unwrap();
        let binary = executor(host.path(), "touch executor-started");
        {
            let _installed = f.install();
            prepare(&f, &binary);
        }
        let root = f.root.path();
        let approval = crate::depth_record::onboarding::OnboardingApproval::parse(
            &fs::read_to_string(root.join(APPROVAL)).unwrap(),
            APPROVAL,
        )
        .unwrap();
        let mut record: crate::depth_record::DepthRecord =
            serde_json::from_value(json_file(root, RECORD)).unwrap();
        record.event = if earlier {
            crate::depth_record::Event::Onboarding {
                approval_commit: "a".repeat(40),
                approval_recorded_at: "2024-01-01T00:00:00Z".into(),
            }
        } else {
            approval.event
        };
        record.policy.selection.enrollment.lifecycle_path =
            crate::agent_registry::support_depth::LifecyclePath::Onboarding;
        record.policy.identity = crate::depth_record::part_identity(&record.policy).unwrap();
        write(root, RECORD, serde_json::to_vec_pretty(&record).unwrap());
        let mut state = json_file(root, STATE);
        state["support_depth"]["policy_identity"] = json!(record.policy.identity);
        write(root, STATE, serde_json::to_vec_pretty(&state).unwrap());
        f.commit();
        let _installed = f.install();
        let before = tree(root);
        for write in [false, true] {
            let error = runtime::run_in_workspace(root, args(write, &binary), &mut vec![])
                .unwrap_err()
                .to_string();
            assert!(error.contains("retained isolated selection"), "{error}");
            assert_eq!(tree(root), before);
        }
        assert!(!root.join("executor-started").exists());
    }
}

#[test]
fn depth_create_refuses_production_before_attempt_or_executor() {
    let f = setup();
    let host = tempfile::tempdir().unwrap();
    let binary = executor(host.path(), "touch executor-started");
    {
        let _installed = f.install();
        prepare(&f, &binary);
    }
    let before = tree(f.root.path());
    let error = run(&f, &binary).unwrap_err().to_string();
    assert!(error.contains("unsupported depth write"), "{error}");
    assert_eq!(tree(f.root.path()), before);
}
