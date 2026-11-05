//! One unconditional finalization boundary after candidate execution may have begun.
use super::{
    candidate::Candidate,
    packet::{self, build_run_status, load_prepared_packet, persist_run_packet},
    tree,
    types::{Context, ValidationCheck, ValidationReport},
    validate::validate_prepared_packet,
    workflow, Error, VALIDATION_REPORT_FILE_NAME, WORKFLOW_VERSION,
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
    candidate.context.evidence_channel = staged.as_ref().map(|s| s.locator().to_owned());
    let mut output = Vec::new();
    // No `?` escapes between possible execution and observation/finalization.
    let execution =
        workflow::execute_candidate_write(&candidate.root, &candidate.context, &mut output);
    // The invocation's channel and bundle end with the executor.
    drop(staged);
    candidate.context.evidence_channel = None;
    let observation = candidate.observe();
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
    let mut report = observation
        .as_ref()
        .ok()
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
    let mut delivery = if passed {
        observation.unwrap_or_default()
    } else {
        candidate.before.clone()
    };
    delivery.extend(diagnostics);
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
    writer
        .write_all(&output)
        .map_err(|e| Error::Internal(e.to_string()))?;
    // Keep the independently validated candidate for diagnostics/recovery; it is never integrated.
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

fn persist_diagnostics(
    candidate: &Candidate,
    report: &ValidationReport,
    passed: bool,
    written: &[String],
) -> Result<tree::Tree, Error> {
    let attempt = || {
        let status = build_run_status(
            &candidate.context,
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
            &candidate.context,
            report,
            &status,
            written,
            report.codex_execution.as_ref(),
            "write",
        )?;
        packet::write_string(
            &candidate.context.run_dir.join("recovery-candidate.txt"),
            &format!("{}\n", candidate.recovery().display()),
        )?;
        candidate.diagnostic_tree()
    };
    attempt().map_err(|e| {
        Error::Validation(format!(
            "{e}; candidate retained at {}",
            candidate.recovery().display()
        ))
    })
}
