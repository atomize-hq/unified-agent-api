use std::path::Path;

pub const RUNTIME_RUNS_ROOT: &str = "docs/agents/.uaa-temp/runtime-follow-on/runs";
pub const REQUIRED_RUNTIME_EVIDENCE_FILES: [&str; 6] = [
    "input-contract.json",
    "run-status.json",
    "run-summary.md",
    "validation-report.json",
    "written-paths.json",
    "handoff.json",
];

pub fn validate_run_id(run_id: &str) -> Result<(), String> {
    if run_id.trim().is_empty() {
        return Err("runtime evidence run id must not be empty".to_string());
    }
    if run_id.contains('/') || run_id.contains('\\') {
        return Err("runtime evidence run id must be a single path segment".to_string());
    }
    if run_id == "." || run_id == ".." || run_id.starts_with('.') {
        return Err("runtime evidence run id must not be `.` / `..` or start with `.`".to_string());
    }
    if run_id
        .chars()
        .any(|ch| !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'))
    {
        return Err(
            "runtime evidence run id may only contain ASCII letters, digits, `-`, and `_`"
                .to_string(),
        );
    }
    Ok(())
}

pub fn run_relative_root(run_id: &str) -> Result<String, String> {
    validate_run_id(run_id)?;
    Ok(format!("{RUNTIME_RUNS_ROOT}/{run_id}"))
}

pub fn run_root(workspace_root: &Path, run_id: &str) -> Result<std::path::PathBuf, String> {
    Ok(workspace_root.join(run_relative_root(run_id)?))
}

pub fn runtime_evidence_paths_for_run(run_id: &str) -> Result<Vec<String>, String> {
    let run_relative = run_relative_root(run_id)?;
    Ok(REQUIRED_RUNTIME_EVIDENCE_FILES
        .iter()
        .map(|name| format!("{run_relative}/{name}"))
        .collect())
}

pub fn validate_runtime_evidence_paths_shape(paths: &[String]) -> Result<String, String> {
    if paths.len() != REQUIRED_RUNTIME_EVIDENCE_FILES.len() {
        return Err(format!(
            "runtime_evidence_paths must contain exactly {} entries",
            REQUIRED_RUNTIME_EVIDENCE_FILES.len()
        ));
    }
    let first = paths
        .first()
        .ok_or_else(|| "runtime_evidence_paths must not be empty".to_string())?;
    let (relative_root, file_name) = first.rsplit_once('/').ok_or_else(|| {
        "runtime_evidence_paths entries must be repo-relative file paths".to_string()
    })?;
    if file_name != REQUIRED_RUNTIME_EVIDENCE_FILES[0] {
        return Err(format!(
            "runtime_evidence_paths must start with `{}`",
            REQUIRED_RUNTIME_EVIDENCE_FILES[0]
        ));
    }
    let run_id = relative_root
        .strip_prefix(&format!("{RUNTIME_RUNS_ROOT}/"))
        .ok_or_else(|| {
            format!("runtime_evidence_paths must stay under `{RUNTIME_RUNS_ROOT}/<run_id>`")
        })?;
    validate_run_id(run_id)?;
    let expected = runtime_evidence_paths_for_run(run_id)?;
    if paths == expected {
        Ok(run_id.to_string())
    } else {
        Err(
            "runtime_evidence_paths must match the canonical six-file runtime evidence set exactly"
                .to_string(),
        )
    }
}
