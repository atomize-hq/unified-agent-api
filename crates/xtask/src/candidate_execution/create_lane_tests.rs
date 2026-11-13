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
fn create_candidate_delivery_cleans_success_and_retains_reported_failure() {
    for (exit_code, write_recovery, refuse_delivery) in [
        (0, false, false),
        (7, false, false),
        (0, true, false),
        (0, false, true),
    ] {
        let passed = exit_code == 0 && !write_recovery;
        let (root, context) = setup(true);
        let before = fs::read(context.run_dir.join(HANDOFF_FILE_NAME)).unwrap();
        let binary = root.path().join("executor.sh");
        let handoff = serde_json::json!({"agent_id":"codex", "manifest_root":"cli_manifests/codex", "runtime_lane_complete":true, "publication_refresh_required":true, "required_commands":life::REQUIRED_PUBLICATION_COMMANDS, "blockers":[]});
        let recovery_write = if write_recovery {
            "printf '%s\\n' \"$PWD\" > \"$(dirname \"$target\")/recovery-candidate.txt\""
        } else {
            ":"
        };
        fs::write(&binary, format!("#!/bin/sh\nset -eu\ntarget=$(sed -n 's/^- Update `\\([^`]*\\)` so it passes validation with:/\\1/p')\ntest -n \"$target\"\ncat > \"$target\" <<'JSON'\n{handoff}\nJSON\nprintf '%s\\n' \"$PWD/$target\"\nprintf '%s\\n' \"$PWD/missing\" >&2\n{recovery_write}\nexit {exit_code}\n")).unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
        let mut args = args();
        args.codex_binary = Some(binary.to_string_lossy().into_owned());
        let candidate = Candidate::prepare(
            root.path(),
            &format!("{RUNTIME_RUNS_ROOT}/{RUN_ID}"),
            vec![],
        )
        .unwrap();
        let _holder = candidate_execution::TestHolder::new(&candidate.root);
        let (execution, observed) = candidate.execute_observed(|| {
            codex_exec::capture_depth_execution(
                root.path(),
                &candidate.root,
                &context,
                &args,
                &prepared_prompt(&context),
            )
        });
        let (evidence, output) = execution.unwrap();
        assert_eq!(evidence.exit_code, exit_code);
        let observed = observed.unwrap();
        let guard = candidate.check(&observed);
        let errors = if write_recovery {
            let error = guard.unwrap_err().to_string();
            assert!(error.contains("protected request"), "{error}");
            assert!(error.contains("recovery-candidate.txt"), "{error}");
            vec![error]
        } else {
            guard.unwrap();
            if passed {
                vec![]
            } else {
                vec!["executor failed".into()]
            }
        };
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
        let mut report = ValidationReport {
            workflow_version: WORKFLOW_VERSION.into(),
            generated_at: now_rfc3339().unwrap(),
            run_id: context.run_id.clone(),
            status: if passed { "pass" } else { "fail" }.into(),
            checks: vec![ValidationCheck {
                name: "codex_exec".into(),
                ok: passed,
                message: format!("codex exec completed via `{}`", evidence.binary),
            }],
            errors,
        };
        let original_evidence = bytes(&evidence).unwrap();
        let execution = Ok((evidence, output));
        let delivery = finalized_diagnostics(
            root.path(),
            &context,
            &candidate,
            Some(&observed),
            &report,
            &[],
            execution.as_ref().ok(),
        )
        .unwrap();
        let source_before: Tree = candidate
            .before
            .iter()
            .filter(|(path, _)| !path.starts_with(&format!("{RUNTIME_RUNS_ROOT}/")))
            .map(|(p, n)| (p.clone(), n.clone()))
            .collect();
        if refuse_delivery {
            fs::write(root.path().join("concurrent.txt"), "caller change").unwrap();
        }
        let outcome = candidate
            .deliver(root.path(), &delivery, passed, None)
            .map(|_| ())
            .map_err(candidate_error);
        let recovery = candidate.recovery();
        let result = finalize_delivery(
            root.path(),
            candidate,
            &context,
            &mut report,
            &execution,
            outcome,
        );
        let pointer = context.run_dir.join("recovery-candidate.txt");
        let delivered = passed && !refuse_delivery;
        assert_eq!(result.is_ok(), delivered);
        let evidence: CodexExecutionEvidence =
            load_json(&context.run_dir.join(CODEX_EXECUTION_FILE_NAME)).unwrap();
        if delivered {
            assert!(!recovery.parent().unwrap().exists());
            assert!(!pointer.exists());
            for path in [
                &evidence.prompt_path,
                &evidence.stdout_path,
                &evidence.stderr_path,
                &evidence.binary,
            ] {
                assert!(Path::new(path).is_file(), "{path}");
                assert!(Path::new(path).starts_with(root.path()), "{path}");
            }
            assert_eq!(evidence.argv.last().unwrap(), root.path().to_str().unwrap());
            for name in [
                CODEX_EXECUTION_FILE_NAME,
                VALIDATION_REPORT_FILE_NAME,
                RUN_STATUS_FILE_NAME,
                RUN_SUMMARY_FILE_NAME,
            ] {
                let text = fs::read_to_string(context.run_dir.join(name)).unwrap();
                assert!(
                    !text.contains(recovery.parent().unwrap().to_str().unwrap()),
                    "{name}: {text}"
                );
            }
            assert_eq!(
                fs::read_to_string(&evidence.stdout_path).unwrap().trim(),
                recovery
                    .join(format!("{RUNTIME_RUNS_ROOT}/{}", context.run_id))
                    .join(HANDOFF_FILE_NAME)
                    .to_str()
                    .unwrap()
            );
            assert_eq!(
                fs::read_to_string(&evidence.stderr_path).unwrap().trim(),
                recovery.join("missing").to_str().unwrap()
            );
        } else {
            assert_eq!(bytes(&evidence).unwrap(), original_evidence);
            assert!(recovery.is_dir());
            let retained: Vec<_> = fs::read_dir(recovery.parent().unwrap())
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect();
            assert_eq!(retained, vec![recovery.clone()]);
            let source_after: Tree = tree::capture(root.path(), &Default::default())
                .unwrap()
                .into_iter()
                .filter(|(path, _)| {
                    !path.starts_with(&format!("{RUNTIME_RUNS_ROOT}/")) && path != "concurrent.txt"
                })
                .collect();
            assert_eq!(source_after, source_before);
            assert_eq!(
                fs::read_to_string(pointer).unwrap().trim(),
                recovery.to_str().unwrap()
            );
            assert!(result
                .unwrap_err()
                .to_string()
                .contains(recovery.to_str().unwrap()));
        }
        if delivered {
            validate_handoff(&context.run_dir.join(HANDOFF_FILE_NAME), &context).unwrap();
        } else {
            assert_eq!(
                fs::read(context.run_dir.join(HANDOFF_FILE_NAME)).unwrap(),
                before
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

#[test]
fn create_candidate_relative_holder_name_preserves_publication_digest() {
    let (root, context) = setup(true);
    write(
        root.path(),
        &context.input_contract.required_agent_api_test,
        "// fixture\n",
    );
    let candidate = Candidate::prepare(
        root.path(),
        &format!("{RUNTIME_RUNS_ROOT}/{RUN_ID}"),
        vec![STATE.into()],
    )
    .unwrap();
    let _holder = candidate_execution::TestHolder::new(&candidate.root);
    let holder = candidate.root.parent().unwrap();
    let relative = format!("crates/codex/src{}/note.txt", holder.display());
    assert!(is_allowed_write_path(
        &relative,
        &context.approval.descriptor
    ));
    let source = format!("executor content mentions {}", holder.display());
    let candidate_context = RuntimeContext {
        approval: context.approval.clone(),
        input_contract: context.input_contract.clone(),
        lifecycle_state: context.lifecycle_state.clone(),
        lifecycle_state_path: context.lifecycle_state_path.clone(),
        run_id: context.run_id.clone(),
        run_dir: candidate.root.join(&candidate.run_dir_rel),
    };
    let (execution, observed) = candidate.execute_observed(|| {
        write(&candidate.root, &relative, &source);
        write(
            &candidate_context.run_dir,
            HANDOFF_FILE_NAME,
            bytes(&serde_json::json!({
                "agent_id": "codex", "manifest_root": "cli_manifests/codex",
                "runtime_lane_complete": true, "publication_refresh_required": true,
                "required_commands": life::REQUIRED_PUBLICATION_COMMANDS, "blockers": [],
            }))
            .unwrap(),
        );
        Ok::<_, Error>(CodexExecutionEvidence {
            workflow_version: WORKFLOW_VERSION.into(),
            generated_at: now_rfc3339().unwrap(),
            run_id: RUN_ID.into(),
            binary: "codex".into(),
            argv: vec!["--cd".into(), candidate.root.to_str().unwrap().into()],
            prompt_path: candidate_context
                .run_dir
                .join(PROMPT_FILE_NAME)
                .to_str()
                .unwrap()
                .into(),
            stdout_path: candidate_context
                .run_dir
                .join(CODEX_STDOUT_FILE_NAME)
                .to_str()
                .unwrap()
                .into(),
            stderr_path: candidate_context
                .run_dir
                .join(CODEX_STDERR_FILE_NAME)
                .to_str()
                .unwrap()
                .into(),
            exit_code: 0,
        })
    });
    let evidence = execution.unwrap();
    let observed = observed.unwrap();
    candidate.check(&observed).unwrap();
    let (mut report, written) = validate_observed_write_mode(
        &candidate.root,
        &candidate_context,
        &context.input_contract,
        &evidence,
        vec![relative.clone()],
    )
    .unwrap();
    assert_eq!(report.status, "pass", "{:?}", report.errors);
    assert_eq!(written, vec![relative.clone()]);
    let execution = Ok((
        evidence,
        std::process::Command::new("true").output().unwrap(),
    ));
    let delivery = finalized_diagnostics(
        root.path(),
        &context,
        &candidate,
        Some(&observed),
        &report,
        &written,
        execution.as_ref().ok(),
    )
    .unwrap();
    let mut state =
        lifecycle::successful_runtime_state(&context, &context.input_contract, &written).unwrap();
    state.written_paths_sha256 =
        Some(life::Preserved::new(hex::encode(Sha256::digest(bytes(&written).unwrap()))).unwrap());
    let trusted = [PlannedMutation::replace(
        STATE,
        fs::read(root.path().join(STATE)).unwrap(),
        bytes(&state).unwrap(),
    )];
    let outcome = candidate
        .deliver_with(root.path(), &delivery, true, None, &trusted)
        .map(|_| ())
        .map_err(candidate_error);
    let holder = holder.to_owned();
    finalize_delivery(
        root.path(),
        candidate,
        &context,
        &mut report,
        &execution,
        outcome,
    )
    .unwrap();
    assert!(!holder.exists());
    assert_eq!(
        fs::read_to_string(root.path().join(&relative)).unwrap(),
        source
    );
    let recorded: Vec<String> = load_json(&context.run_dir.join(WRITTEN_PATHS_FILE_NAME)).unwrap();
    assert_eq!(recorded, written);
    let status: serde_json::Value = load_json(&context.run_dir.join(RUN_STATUS_FILE_NAME)).unwrap();
    assert_eq!(status["written_paths"], serde_json::json!(written));
    assert_eq!(
        hex::encode(Sha256::digest(
            fs::read(context.run_dir.join(WRITTEN_PATHS_FILE_NAME)).unwrap()
        )),
        **state.written_paths_sha256.as_ref().unwrap()
    );
    crate::prepare_publication::validate_runtime_evidence_run_for_approval(
        root.path(),
        &context.approval,
        RUN_ID,
    )
    .unwrap();
}
