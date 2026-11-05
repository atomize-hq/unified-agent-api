//! Create-lane policy adapter. Compiled as a child of runtime_follow_on; the independent
//! candidate, observations, finalization and delivery are shared with MA4.
use super::*;
use crate::{
    agent_lifecycle::{Preserved, SideState},
    candidate_execution::{
        self,
        tree::{self, Node, Tree},
        Candidate,
    },
    isolated_maintenance::CapturedRegistry,
    workspace_mutation::{apply_admitted_mutations, PlannedMutation},
};
use sha2::{Digest, Sha256};

pub(super) fn render_depth_prompt(context: &RuntimeContext) -> String {
    // The executor runs in the independent candidate; every run-file destination must
    // resolve there, including prompts prepared in the caller before the copy exists.
    let relative_context = RuntimeContext {
        approval: context.approval.clone(),
        input_contract: context.input_contract.clone(),
        lifecycle_state: context.lifecycle_state.clone(),
        lifecycle_state_path: context.lifecycle_state_path.clone(),
        run_id: context.run_id.clone(),
        run_dir: Path::new(RUNTIME_RUNS_ROOT).join(&context.run_id),
    };
    let mut prompt = render_prompt(&relative_context);
    prompt.push_str("\nDepth candidate boundary: handoff.json is the only run file you may write. The input contract, prompt, lifecycle state, depth record, approval and depth-acquisition statement are immutable. The controller writes all streams, execution evidence, validation, status, written paths and summary after observation. Do not write snapshots.\n");
    prompt
}

pub(super) fn candidate_error(error: candidate_execution::Error) -> Error {
    match error {
        candidate_execution::Error::Validation(s) => Error::Validation(s),
        candidate_execution::Error::Internal(s) => Error::Internal(s),
    }
}
fn mutation_error(error: crate::workspace_mutation::WorkspaceMutationError) -> Error {
    Error::Validation(error.to_string())
}
fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    serde_json::to_vec_pretty(value).map_err(|e| Error::Internal(e.to_string()))
}

pub(super) fn execute<W: Write>(
    caller: &Path,
    context: &RuntimeContext,
    args: &Args,
    prepared: &InputContract,
    prompt: &str,
    writer: &mut W,
) -> Result<(), Error> {
    // Phase 1: authenticate the isolated route and frozen packet before recording an attempt.
    let captured = crate::isolated_maintenance::route(caller)
        .map_err(Error::Validation)?
        .ok_or_else(|| {
            Error::Validation("onboarding depth execution requires an isolated route".into())
        })?
        .capture(caller)
        .map_err(Error::Validation)?;
    check_prepared(caller, context, prepared, prompt)?;
    let original = fs::read(caller.join(&context.lifecycle_state_path))
        .map_err(|e| Error::Internal(e.to_string()))?;
    let mut attempt = context.lifecycle_state.clone();
    attempt.current_owner_command = "runtime-follow-on --write".into();
    attempt.last_transition_by = "xtask runtime-follow-on --write".into();
    attempt.last_transition_at = now_rfc3339()?;
    attempt.active_runtime_evidence_run_id = None;
    attempt
        .side_states
        .retain(|s| !matches!(s, SideState::Blocked | SideState::FailedRetryable));
    attempt.side_states.push(SideState::FailedRetryable);
    attempt.side_states.sort();
    attempt.blocking_issues.clear();
    attempt.retryable_failures = vec![format!(
        "write attempt {} started; consult run diagnostics",
        context.run_id
    )];
    attempt
        .validate()
        .map_err(|e| Error::Validation(e.to_string()))?;
    let attempt_bytes = bytes(&attempt)?;
    apply_admitted_mutations(
        caller,
        &[PlannedMutation::replace(
            &context.lifecycle_state_path,
            original.clone(),
            attempt_bytes.clone(),
        )],
    )
    .map_err(mutation_error)?;

    // Phase 2: the serialized baseline is reporting only. Capture after trusted preparation.
    let run_rel = format!("{RUNTIME_RUNS_ROOT}/{}", context.run_id);
    let version = context
        .approval
        .support_depth_version
        .as_deref()
        .expect("depth route");
    let protected = vec![
        context.lifecycle_state_path.clone(),
        context.approval.relative_path.clone(),
        format!(
            "{}/reports/{version}/depth-record.json",
            context.approval.descriptor.manifest_root
        ),
        format!(
            "{}/depth-acquisition/{version}.toml",
            context
                .lifecycle_state_path
                .rsplit_once('/')
                .expect("lifecycle governance")
                .0
        ),
    ];
    let candidate = Candidate::prepare(caller, &run_rel, protected).map_err(candidate_error)?;
    let _bound = captured
        .bind_candidate(&candidate.root)
        .map_err(Error::Validation)?;
    let candidate_context = RuntimeContext {
        approval: context.approval.clone(),
        input_contract: context.input_contract.clone(),
        lifecycle_state: attempt,
        lifecycle_state_path: context.lifecycle_state_path.clone(),
        run_id: context.run_id.clone(),
        run_dir: candidate.root.join(&run_rel),
    };
    // Phase 3: host streams stay private; the executor may change only handoff.json in this run.
    // There is no early exit after possible execution until observation/finalization below.
    let (execution, observation) = candidate.execute_observed(|| {
        codex_exec::capture_depth_execution(caller, &candidate.root, context, args, prompt)
    });
    // Phase 4: the shared boundary observed before any controller output was written.
    let checked = (|| {
        let after = observation
            .as_ref()
            .map_err(|e| Error::Validation(e.to_string()))?;
        candidate.check(after).map_err(candidate_error)?;
        let changes = tree::changed(&candidate.before, after);
        let mut source_paths = Vec::new();
        for path in changes {
            if matches!(after.get(&path), Some(Node::Directory(_))) {
                continue;
            }
            if path == format!("{run_rel}/{HANDOFF_FILE_NAME}") {
                continue;
            }
            if path.starts_with(&format!("{RUNTIME_RUNS_ROOT}/")) {
                return Err(Error::Validation(format!(
                    "executor changed controller run file: {path}"
                )));
            }
            if path.starts_with(&format!(
                "{}/snapshots/",
                context.approval.descriptor.manifest_root
            )) {
                return Err(Error::Validation(format!(
                    "depth write boundary forbids snapshots: {path}"
                )));
            }
            source_paths.push(path);
        }
        let (evidence, _) = execution
            .as_ref()
            .map_err(|e| Error::Validation(e.to_string()))?;
        let validated = validate_observed_write_mode(
            &candidate.root,
            &candidate_context,
            prepared,
            evidence,
            source_paths,
        )?;
        if &candidate.observe().map_err(candidate_error)? != after {
            return Err(Error::Validation(
                "candidate changed during create-lane validation".into(),
            ));
        }
        Ok(validated)
    })();
    let internal = matches!(execution, Err(Error::Internal(_)));
    let (mut report, mut written) = match checked {
        Ok(value) => value,
        Err(e) => (
            ValidationReport {
                workflow_version: WORKFLOW_VERSION.into(),
                generated_at: now_rfc3339()?,
                run_id: context.run_id.clone(),
                status: "fail".into(),
                checks: vec![],
                errors: vec![e.to_string()],
            },
            vec![],
        ),
    };
    if let Err(e) = &execution {
        if !report.errors.contains(&e.to_string()) {
            report.errors.push(e.to_string());
        }
        report.status = "fail".into();
    }
    let passed = report.status == "pass";
    if !passed {
        written.clear();
    }
    // Phase 5: host-only diagnostics and expected-before delivery. The lifecycle attempt is
    // never copied out of the candidate; success is a separate trusted mutation in the same batch.
    let outcome: Result<(), Error> = (|| {
        let diagnostics = diagnostics(
            context,
            &candidate,
            &report,
            &written,
            execution.as_ref().ok(),
        )?;
        let delivery = candidate_execution::finalized_tree(
            &candidate.before,
            observation.as_ref().ok(),
            passed,
            diagnostics,
        );
        let trusted = if passed {
            let mut state = lifecycle::successful_runtime_state(context, prepared, &written)?;
            let digest = hex::encode(Sha256::digest(bytes(&written)?));
            state.written_paths_sha256 =
                Some(Preserved::new(digest).map_err(|e| Error::Internal(e.to_string()))?);
            state
                .validate_in_workspace(caller)
                .map_err(|e| Error::Validation(e.to_string()))?;
            vec![PlannedMutation::replace(
                &context.lifecycle_state_path,
                attempt_bytes.clone(),
                bytes(&state)?,
            )]
        } else {
            vec![]
        };
        recheck(
            &captured,
            caller,
            &context.lifecycle_state_path,
            &original,
            &attempt_bytes,
        )?;
        candidate
            .deliver_with(caller, &delivery, passed, None, &trusted)
            .map_err(candidate_error)?;
        Ok(())
    })();
    if let Err(e) = outcome {
        report.status = "fail".into();
        report.errors.push(format!("delivery refused: {e}"));
        let diagnostics = diagnostics(context, &candidate, &report, &[], execution.as_ref().ok())?;
        let failed =
            candidate_execution::finalized_tree(&candidate.before, None, false, diagnostics);
        let diagnostic_error = candidate
            .deliver(caller, &failed, false, None)
            .err()
            .map(|e| format!("; diagnostics not delivered: {e}"))
            .unwrap_or_default();
        return Err(Error::Validation(format!(
            "{e}{diagnostic_error}; candidate retained at {}",
            candidate.root.display()
        )));
    }
    if !passed {
        let message = format!(
            "{}; candidate retained at {}",
            report.errors.join("\n"),
            candidate.root.display()
        );
        return Err(if internal {
            Error::Internal(message)
        } else {
            Error::Validation(message)
        });
    }
    writeln!(
        writer,
        "OK: runtime-follow-on write validation complete.\nrun_id: {}\nvalidated_paths: {}",
        context.run_id,
        written.len()
    )
    .map_err(|e| Error::Internal(e.to_string()))
}

/// The route requires its original clean tree. Temporarily undo ONLY the controller's own
/// expected-before attempt, authenticate, then restore it on both success and refusal. Concurrent
/// edits fail the byte comparison and are preserved. The candidate still holds the attempt tree.
fn recheck(
    captured: &CapturedRegistry,
    caller: &Path,
    path: &str,
    original: &[u8],
    attempt: &[u8],
) -> Result<(), Error> {
    let _index_lock = candidate_execution::lock_index(caller).map_err(candidate_error)?;
    apply_admitted_mutations(
        caller,
        &[PlannedMutation::replace(
            path,
            attempt.to_vec(),
            original.to_vec(),
        )],
    )
    .map_err(mutation_error)?;
    let checked = captured.recheck(caller).map_err(Error::Validation);
    apply_admitted_mutations(
        caller,
        &[PlannedMutation::replace(
            path,
            original.to_vec(),
            attempt.to_vec(),
        )],
    )
    .map_err(mutation_error)?;
    checked
}

fn check_prepared(
    caller: &Path,
    context: &RuntimeContext,
    prepared: &InputContract,
    prompt: &str,
) -> Result<(), Error> {
    let comparable = |contract: &InputContract| -> Result<serde_json::Value, Error> {
        let mut value =
            serde_json::to_value(contract).map_err(|e| Error::Internal(e.to_string()))?;
        let object = value.as_object_mut().expect("input contract object");
        object.remove("generated_at");
        object.remove("baseline"); // Reporting only; never authenticates the write boundary.
        Ok(value)
    };
    if comparable(prepared)? != comparable(&context.input_contract)?
        || prompt != render_depth_prompt(context)
    {
        return Err(Error::Validation(
            "prepared depth packet differs from the approved runtime scope".into(),
        ));
    }
    if context.lifecycle_state.agent_id != context.approval.descriptor.agent_id
        || context.lifecycle_state.approval_artifact_path != context.approval.relative_path
    {
        return Err(Error::Validation(
            "depth lifecycle state differs from the approved runtime scope".into(),
        ));
    }
    let version = context
        .approval
        .support_depth_version
        .as_deref()
        .expect("depth route");
    let object = context
        .lifecycle_state
        .support_depth
        .as_ref()
        .ok_or_else(|| {
            Error::Validation(
                "depth write requires the frozen lifecycle support_depth object".into(),
            )
        })?;
    let raw = read_string(&caller.join(format!(
        "{}/reports/{version}/depth-record.json",
        context.approval.descriptor.manifest_root
    )))?;
    let record = crate::depth_record::DepthRecord::parse(
        &raw,
        &context.approval.descriptor.agent_id,
        version,
    )
    .map_err(Error::Validation)?;
    let approval = crate::depth_record::onboarding::OnboardingApproval::parse(
        &read_string(&caller.join(&context.approval.relative_path))?,
        &context.approval.relative_path,
    )
    .map_err(Error::Validation)?;
    // A maintenance Event or an earlier onboarding generation cannot authenticate this
    // approval's write, even when the lifecycle object repeats the record's P/O identities.
    if let Some(why) = record.request_mismatch(&object.binding(approval.event)) {
        return Err(Error::Validation(format!("depth write: {why}")));
    }
    Ok(())
}

fn diagnostics(
    context: &RuntimeContext,
    candidate: &Candidate,
    report: &ValidationReport,
    written: &[String],
    execution: Option<&(CodexExecutionEvidence, std::process::Output)>,
) -> Result<Tree, Error> {
    let passed = report.status == "pass";
    let default_mode = candidate
        .before
        .values()
        .find_map(|n| match n {
            Node::File(_, m) => Some(*m),
            _ => None,
        })
        .unwrap_or(0o644);
    let mut out = Tree::new();
    let prefix = &candidate.run_dir_rel;
    let mut insert = |name: &str, data: Vec<u8>| {
        out.insert(format!("{prefix}/{name}"), Node::File(data, default_mode));
    };
    insert(VALIDATION_REPORT_FILE_NAME, bytes(report)?);
    insert(WRITTEN_PATHS_FILE_NAME, bytes(&written)?);
    insert(
        RUN_STATUS_FILE_NAME,
        bytes(&render_run_status(
            context,
            "write",
            if passed {
                "write_validated"
            } else {
                "write_failed"
            },
            passed,
            passed,
            written.to_vec(),
            report.errors.clone(),
        ))?,
    );
    insert(
        RUN_SUMMARY_FILE_NAME,
        render_run_summary(context, report, written, execution.map(|e| &e.0)).into_bytes(),
    );
    insert(
        "recovery-candidate.txt",
        format!("{}\n", candidate.root.display()).into_bytes(),
    );
    if let Some((evidence, output)) = execution {
        insert(CODEX_EXECUTION_FILE_NAME, bytes(evidence)?);
        insert(
            CODEX_STDOUT_FILE_NAME,
            String::from_utf8_lossy(&output.stdout)
                .into_owned()
                .into_bytes(),
        );
        insert(
            CODEX_STDERR_FILE_NAME,
            String::from_utf8_lossy(&output.stderr)
                .into_owned()
                .into_bytes(),
        );
    }
    Ok(out)
}

#[cfg(all(test, unix))]
#[path = "create_lane_tests.rs"]
mod create_candidate;
