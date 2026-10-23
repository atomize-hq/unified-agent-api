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
    let candidate = Candidate::prepare(caller, context)?;
    let mut output = Vec::new();
    // No `?` escapes between possible execution and observation/finalization.
    let execution =
        workflow::execute_candidate_write(&candidate.root, &candidate.context, &mut output);
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
    let status = build_run_status(
        context,
        "write",
        if passed {
            "write_validated"
        } else {
            "write_failed"
        },
        passed,
        written.clone(),
        report.errors.clone(),
    )?;
    // Diagnostics remain inside the private candidate until expected-before delivery.
    let packet_result = persist_run_packet(
        &candidate.context,
        &report,
        &status,
        &written,
        report.codex_execution.as_ref(),
        "write",
    );
    if let Err(e) = packet_result {
        return Err(Error::Validation(format!(
            "{e}; candidate retained at {}",
            candidate.recovery().display()
        )));
    }
    packet::write_string(
        &candidate.context.run_dir.join("recovery-candidate.txt"),
        &format!("{}\n", candidate.recovery().display()),
    )?;
    let diagnostics = candidate.diagnostic_tree().map_err(|e| {
        Error::Validation(format!(
            "{e}; candidate retained at {}",
            candidate.recovery().display()
        ))
    })?;
    let mut delivery = if passed {
        observation.unwrap_or_default()
    } else {
        candidate.before.clone()
    };
    delivery.extend(diagnostics);
    let copied = candidate.deliver(caller, &delivery, passed);
    if let Err(e) = copied {
        return Err(Error::Validation(format!(
            "{e}; candidate retained at {}",
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
