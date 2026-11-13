//! One unconditional finalization boundary after candidate execution may have begun.
use super::{
    candidate::Candidate,
    packet::{self, build_run_status, load_prepared_packet, persist_run_packet},
    tree,
    types::{Context, ValidationCheck, ValidationReport},
    validate::validate_prepared_packet,
    workflow, Error, CODEX_STDERR_FILE_NAME, CODEX_STDOUT_FILE_NAME, INPUT_CONTRACT_FILE_NAME,
    VALIDATION_REPORT_FILE_NAME, WORKFLOW_VERSION,
};
use std::{io::Write, path::Path};

pub(super) fn execute_write_mode<W: Write>(
    caller: &Path,
    context: &Context,
    writer: &mut W,
) -> Result<(), Error> {
    let (prepared, prompt) = load_prepared_packet(context)?;
    validate_prepared_packet(context, &prepared, &prompt)?;
    let mut candidate = Candidate::prepare(caller, context)?;
    // The candidate cannot resolve the isolated route; bind the registry this execution captured
    // to its exact root for every in-process check below. Nothing has executed yet.
    let _bound = context
        .registry
        .as_ref()
        .map(|captured| captured.bind_candidate(&candidate.root))
        .transpose()
        .map_err(Error::Validation)?;
    // Under the captured route only, stage authenticated evidence outside the candidate and serve
    // it to this candidate alone while the executor runs. Production stages nothing.
    let staged = context
        .registry
        .as_ref()
        .map(|captured| stage_evidence(caller, captured, &candidate.root, context))
        .transpose()?;
    candidate.context.evidence_channel = staged.as_ref().map(|s| s.channel());
    candidate.staged_evidence = staged.as_ref().map(|s| s.files().clone());
    let mut output = Vec::new();
    // No `?` escapes between possible execution and observation/finalization.
    let (execution, observation) = candidate.execute_observed(|| {
        let result =
            workflow::execute_candidate_write(&candidate.root, &candidate.context, &mut output);
        // The executor spawn revoked the channel; end its bundle before observation.
        drop(staged);
        result
    });
    candidate.context.evidence_channel = None;
    let guard = observation
        .as_ref()
        .map_err(|e| Error::Validation(e.to_string()))
        .and_then(|after| candidate.check(after));
    let internal = matches!(execution, Err(Error::Internal(_)));
    let mut errors = Vec::new();
    if let Err(e) = execution {
        errors.push(e.to_string());
    }
    if let Err(e) = guard {
        errors.push(e.to_string());
    }
    let passed = errors.is_empty();
    let report_path = format!("{}/{VALIDATION_REPORT_FILE_NAME}", context.run_dir_rel);
    // A failed candidate is never read back: its report, like every failure diagnostic, comes
    // only from what the host holds in memory.
    let mut report = observation
        .as_ref()
        .ok()
        .filter(|_| passed)
        .and_then(|after| match after.get(&report_path) {
            Some(tree::Node::File(bytes, _)) if bytes.len() <= 1_048_576 => {
                serde_json::from_slice::<ValidationReport>(bytes).ok()
            }
            _ => None,
        })
        .unwrap_or_else(|| ValidationReport {
            workflow_version: WORKFLOW_VERSION.into(),
            run_id: context.run_id.clone(),
            status: "fail".into(),
            checks: vec![],
            errors: vec![],
            preflight: None,
            codex_execution: None,
            gate_results: vec![],
        });
    report.workflow_version = WORKFLOW_VERSION.into();
    report.run_id = context.run_id.clone();
    report.codex_execution = candidate
        .context
        .host_capture
        .lock()
        .ok()
        .and_then(|capture| capture.as_ref().map(|c| c.execution.clone()));
    report.status = if passed { "pass" } else { "fail" }.into();
    report.errors.extend(errors.clone());
    report.checks.push(ValidationCheck {
        name: "relay_reporting_guard".into(),
        ok: passed,
        message: if passed {
            "candidate passed relay checks".into()
        } else {
            format!(
                "failed candidate retained at {}",
                candidate.recovery().display()
            )
        },
    });
    let written = if passed {
        observation
            .as_ref()
            .map(|after| {
                tree::changed(&candidate.before, after)
                    .into_iter()
                    .filter(|p| {
                        matches!(after.get(p), Some(tree::Node::File(..)))
                            && !p.starts_with(&format!("{}/", context.run_dir_rel))
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        vec![]
    };
    let diagnostics = persist_diagnostics(&candidate, &report, passed, &written)?;
    let delivery = crate::candidate_execution::finalized_tree(
        &candidate.before,
        observation.as_ref().ok(),
        passed,
        diagnostics,
    );
    // Re-authenticate the captured isolated route before anything reaches the caller.
    let copied = match context
        .registry
        .as_ref()
        .map(|captured| captured.recheck(caller))
    {
        Some(Err(e)) if passed => Err(Error::Validation(format!(
            "isolated maintenance route changed during execution: {e}"
        ))),
        _ => candidate.deliver(caller, &delivery, passed),
    };
    if let Err(e) = copied {
        report.status = "fail".into();
        report.errors.push(format!("delivery refused: {e}"));
        report.checks.push(ValidationCheck {
            name: "relay_delivery".into(),
            ok: false,
            message: e.to_string(),
        });
        let diagnostics = persist_diagnostics(&candidate, &report, false, &[])?;
        let mut failed = candidate.before.clone();
        failed.extend(diagnostics);
        let diagnostic_error = candidate
            .deliver(caller, &failed, false)
            .err()
            .map(|e| format!("; failure diagnostics not delivered: {e}"))
            .unwrap_or_default();
        return Err(Error::Validation(format!(
            "{e}{diagnostic_error}; candidate retained at {}",
            candidate.recovery().display()
        )));
    }
    if !passed {
        let message = format!(
            "{}; rejected candidate retained at {}",
            errors.join("\n"),
            candidate.recovery().display()
        );
        return Err(if internal {
            Error::Internal(message)
        } else {
            Error::Validation(message)
        });
    }
    candidate.discard();
    writer
        .write_all(&output)
        .map_err(|e| Error::Internal(e.to_string()))?;
    Ok(())
}

/// The bundle is assembled by the host from the captured route's evidence host. An assembly
/// failure is served to the executor as a refusal; only a staging failure stops the run, before
/// anything executed.
fn stage_evidence(
    caller: &Path,
    captured: &crate::isolated_maintenance::CapturedRegistry,
    candidate: &Path,
    context: &Context,
) -> Result<crate::depth_evidence::Staged, Error> {
    let request = &context.envelope.request;
    let target = crate::depth_evidence::StagedTarget {
        agent_id: request.agent_id.clone(),
        version: request
            .detected_release
            .as_ref()
            .map(|release| release.target_version.clone())
            .unwrap_or_default(),
        manifest_root: captured
            .registry()
            .find(&request.agent_id)
            .map(|entry| entry.manifest_root.clone())
            .unwrap_or_default(),
    };
    crate::depth_evidence::stage_isolated_evidence(
        caller,
        candidate,
        captured.evidence_host(caller),
        &target,
    )
    .map_err(|e| Error::Validation(format!("isolated evidence staging: {e}")))
}

/// The run's diagnostics, serialized by the host into its own private directory from what it
/// holds in memory (report, status, the executor's output as the host received it, the recovery
/// pointer). Nothing is read from the candidate, so a failed run copies back no file the executor
/// wrote.
fn persist_diagnostics(
    candidate: &Candidate,
    report: &ValidationReport,
    passed: bool,
    written: &[String],
) -> Result<tree::Tree, Error> {
    let attempt = || -> Result<tree::Tree, Error> {
        let dir = tempfile::Builder::new()
            .prefix("uaa-diagnostics-")
            .tempdir()
            .map_err(tree::io)?;
        let mut context = candidate.context.clone();
        context.run_dir = dir.path().join(&context.run_dir_rel);
        std::fs::create_dir_all(&context.run_dir).map_err(tree::io)?;
        // Persistence requires the prepared contract: the caller's own copy, never delivered.
        let contract = format!("{}/{INPUT_CONTRACT_FILE_NAME}", context.run_dir_rel);
        let contract = match candidate.before.get(&contract) {
            Some(tree::Node::File(bytes, _)) => bytes.clone(),
            _ => b"{}".to_vec(),
        };
        std::fs::write(context.run_dir.join(INPUT_CONTRACT_FILE_NAME), contract)
            .map_err(tree::io)?;
        let status = build_run_status(
            &context,
            "write",
            if passed {
                "write_validated"
            } else {
                "write_failed"
            },
            passed,
            written.to_vec(),
            report.errors.clone(),
        )?;
        persist_run_packet(
            &context,
            report,
            &status,
            written,
            report.codex_execution.as_ref(),
            "write",
        )?;
        if !passed {
            packet::write_string(
                &context.run_dir.join("recovery-candidate.txt"),
                &format!("{}\n", candidate.recovery().display()),
            )?;
        }
        let capture = context.host_capture.lock().ok().and_then(|c| c.clone());
        if let Some(capture) = capture {
            packet::write_string(
                &context.run_dir.join(CODEX_STDOUT_FILE_NAME),
                &String::from_utf8_lossy(&capture.stdout),
            )?;
            packet::write_string(
                &context.run_dir.join(CODEX_STDERR_FILE_NAME),
                &String::from_utf8_lossy(&capture.stderr),
            )?;
        }
        let files = tree::capture(&context.run_dir, &Default::default())?;
        // The retained candidate records the same outcome for recovery, best effort: it is
        // written, never read back, and an executor-broken path there never blocks the caller's
        // diagnostics.
        for (relative, node) in &files {
            if let tree::Node::File(bytes, _) = node {
                if relative != INPUT_CONTRACT_FILE_NAME {
                    let _ = packet::write_string(
                        &candidate.context.run_dir.join(relative),
                        &String::from_utf8_lossy(bytes),
                    );
                }
            }
        }
        Ok(files
            .into_iter()
            .map(|(p, n)| (format!("{}/{p}", context.run_dir_rel), n))
            .collect())
    };
    attempt().map_err(|e| {
        Error::Validation(format!(
            "{e}; candidate retained at {}",
            candidate.recovery().display()
        ))
    })
}
