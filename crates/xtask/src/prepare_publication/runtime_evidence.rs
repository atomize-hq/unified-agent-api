use std::{fs, path::Path};

use serde::Deserialize;

use crate::approval_artifact::ApprovalArtifact;

use super::{
    read_json, validate_required_commands, Error, REQUIRED_RUNTIME_EVIDENCE_FILES,
    RUNTIME_RUNS_ROOT,
};

#[derive(Debug, Deserialize)]
struct RuntimeInputContract {
    approval_artifact_path: String,
    approval_artifact_sha256: String,
    agent_id: String,
    manifest_root: String,
    required_handoff_commands: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RuntimeRunStatus {
    approval_artifact_path: String,
    agent_id: String,
    status: String,
    validation_passed: bool,
    handoff_ready: bool,
    run_dir: String,
}

#[derive(Debug, Deserialize)]
struct RuntimeValidationReport {
    status: String,
}

#[derive(Debug, Deserialize)]
struct RuntimeHandoff {
    agent_id: String,
    manifest_root: String,
    runtime_lane_complete: bool,
    publication_refresh_required: bool,
    required_commands: Vec<String>,
    blockers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RuntimeEvidenceBundle {
    pub run_id: String,
    pub runtime_evidence_paths: Vec<String>,
}

pub(super) fn discover_runtime_evidence(
    workspace_root: &Path,
    approval: &ApprovalArtifact,
) -> Result<RuntimeEvidenceBundle, Error> {
    let runs_root = workspace_root.join(RUNTIME_RUNS_ROOT);
    let entries = fs::read_dir(&runs_root).map_err(|err| {
        Error::Validation(format!(
            "runtime evidence root `{}` is missing or unreadable: {err}",
            runs_root.display()
        ))
    })?;
    let mut candidates = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|ty| ty.is_dir()).unwrap_or(false))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.reverse();

    let mut best_error = None;
    for run_id in candidates {
        match inspect_runtime_run(workspace_root, approval, &run_id) {
            Ok(Some(bundle)) => return Ok(bundle),
            Ok(None) => {}
            Err(err) => {
                if best_error.is_none() {
                    best_error = Some(err);
                }
            }
        }
    }

    Err(best_error.unwrap_or_else(|| {
        Error::Validation(format!(
            "no successful runtime-follow-on evidence run was found for `{}` under `{RUNTIME_RUNS_ROOT}`",
            approval.descriptor.agent_id
        ))
    }))
}

fn inspect_runtime_run(
    workspace_root: &Path,
    approval: &ApprovalArtifact,
    run_id: &str,
) -> Result<Option<RuntimeEvidenceBundle>, Error> {
    let relative_run_root = format!("{RUNTIME_RUNS_ROOT}/{run_id}");
    let absolute_run_root = workspace_root.join(&relative_run_root);
    let status_path = absolute_run_root.join("run-status.json");
    if !status_path.is_file() {
        return Ok(None);
    }

    let status: RuntimeRunStatus = read_json(&status_path)?;
    if status.approval_artifact_path != approval.relative_path
        || status.agent_id != approval.descriptor.agent_id
    {
        return Ok(None);
    }
    if status.status != "write_validated" || !status.validation_passed || !status.handoff_ready {
        return Err(stale_runtime_evidence_error(
            approval,
            run_id,
            "is not a validated runtime handoff",
        ));
    }
    if status.run_dir != absolute_run_root.to_string_lossy() {
        return Err(stale_runtime_evidence_error(
            approval,
            run_id,
            &format!(
                "recorded run_dir `{}` but expected `{}`",
                status.run_dir,
                absolute_run_root.display()
            ),
        ));
    }

    let input_contract: RuntimeInputContract =
        read_json(&absolute_run_root.join("input-contract.json"))?;
    if input_contract.approval_artifact_path != approval.relative_path
        || input_contract.approval_artifact_sha256 != approval.sha256
        || input_contract.agent_id != approval.descriptor.agent_id
        || input_contract.manifest_root != approval.descriptor.manifest_root
    {
        return Err(stale_runtime_evidence_error(
            approval,
            run_id,
            "no longer matches the approval artifact continuity",
        ));
    }
    validate_required_commands(
        "runtime input-contract required_handoff_commands",
        &input_contract.required_handoff_commands,
    )
    .map_err(|err| stale_runtime_evidence_error(approval, run_id, &err.to_string()))?;

    let report: RuntimeValidationReport =
        read_json(&absolute_run_root.join("validation-report.json"))?;
    if report.status != "pass" {
        return Err(stale_runtime_evidence_error(
            approval,
            run_id,
            "validation-report.json is not green",
        ));
    }

    let handoff: RuntimeHandoff = read_json(&absolute_run_root.join("handoff.json"))?;
    if handoff.agent_id != approval.descriptor.agent_id
        || handoff.manifest_root != approval.descriptor.manifest_root
    {
        return Err(stale_runtime_evidence_error(
            approval,
            run_id,
            "handoff.json no longer matches the approval artifact continuity",
        ));
    }
    if !handoff.runtime_lane_complete || !handoff.publication_refresh_required {
        return Err(stale_runtime_evidence_error(
            approval,
            run_id,
            "handoff.json is not publication-ready",
        ));
    }
    if handoff
        .blockers
        .iter()
        .any(|blocker| !blocker.trim().is_empty())
    {
        return Err(stale_runtime_evidence_error(
            approval,
            run_id,
            "handoff.json still carries blocker text",
        ));
    }
    validate_required_commands(
        "runtime handoff required_commands",
        &handoff.required_commands,
    )
    .map_err(|err| stale_runtime_evidence_error(approval, run_id, &err.to_string()))?;

    let written_paths: Vec<String> = read_json(&absolute_run_root.join("written-paths.json"))?;
    if written_paths.is_empty() {
        return Err(stale_runtime_evidence_error(
            approval,
            run_id,
            "did not record any written paths",
        ));
    }

    let summary_path = absolute_run_root.join("run-summary.md");
    let summary = fs::read_to_string(&summary_path)
        .map_err(|err| Error::Validation(format!("read {}: {err}", summary_path.display())))?;
    if summary.trim().is_empty() {
        return Err(stale_runtime_evidence_error(
            approval,
            run_id,
            "run-summary.md is empty",
        ));
    }

    for name in REQUIRED_RUNTIME_EVIDENCE_FILES {
        if !absolute_run_root.join(name).is_file() {
            return Err(stale_runtime_evidence_error(
                approval,
                run_id,
                &format!("is missing required file `{name}`"),
            ));
        }
    }

    Ok(Some(RuntimeEvidenceBundle {
        run_id: run_id.to_string(),
        runtime_evidence_paths: REQUIRED_RUNTIME_EVIDENCE_FILES
            .iter()
            .map(|name| format!("{relative_run_root}/{name}"))
            .collect(),
    }))
}

fn stale_runtime_evidence_error(approval: &ApprovalArtifact, run_id: &str, detail: &str) -> Error {
    Error::Validation(format!(
        "runtime evidence run `{run_id}` {detail}; rerun `cargo run -p xtask -- repair-runtime-evidence --approval {} --check` to inspect or `cargo run -p xtask -- repair-runtime-evidence --approval {} --write` to repair the runtime evidence bundle",
        approval.relative_path, approval.relative_path
    ))
}
