use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use serde_json::json;

use crate::{
    agent_lifecycle::{self, LifecycleState},
    approval_artifact::ApprovalArtifact,
};

pub const RUNTIME_RUNS_ROOT: &str = "docs/agents/.uaa-temp/runtime-follow-on/runs";
const WORKFLOW_VERSION: &str = "runtime_follow_on_v1";

#[derive(Debug, Clone)]
pub struct RuntimeEvidenceBundleSpec<'a> {
    pub run_id: &'a str,
    pub host_surface: &'a str,
    pub loaded_skill_ref: &'a str,
    pub mode: &'a str,
    pub source_label: &'a str,
    pub summary_title: &'a str,
    pub validation_check_name: &'a str,
    pub validation_message: &'a str,
}

#[derive(Debug, Clone)]
pub struct GeneratedRuntimeEvidenceBundle {
    pub run_id: String,
    pub run_relative: String,
    pub runtime_evidence_paths: Vec<String>,
    pub written_paths: Vec<String>,
}

#[derive(Debug)]
pub enum Error {
    Validation(String),
    Internal(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(message) | Self::Internal(message) => f.write_str(message),
        }
    }
}

pub fn repair_run_id(agent_id: &str) -> String {
    format!("repair-{agent_id}-runtime-follow-on")
}

pub fn historical_run_id(agent_id: &str) -> String {
    format!("historical-{agent_id}-runtime-follow-on")
}

pub fn check_repairable_runtime_evidence(
    workspace_root: &Path,
    approval: &ApprovalArtifact,
    lifecycle_state: &LifecycleState,
) -> Result<Vec<String>, Error> {
    derive_runtime_written_paths(workspace_root, approval, lifecycle_state)
}

pub fn write_runtime_evidence_bundle(
    workspace_root: &Path,
    approval: &ApprovalArtifact,
    lifecycle_state: &LifecycleState,
    spec: &RuntimeEvidenceBundleSpec<'_>,
) -> Result<GeneratedRuntimeEvidenceBundle, Error> {
    let run_relative = format!("{RUNTIME_RUNS_ROOT}/{}", spec.run_id);
    let run_root = workspace_root.join(&run_relative);
    fs::create_dir_all(&run_root)
        .map_err(|err| Error::Internal(format!("create {}: {err}", run_root.display())))?;

    let written_paths = derive_runtime_written_paths(workspace_root, approval, lifecycle_state)?;
    let generated_at = lifecycle_state.last_transition_at.clone();

    write_json(
        &run_root.join("input-contract.json"),
        &json!({
            "workflow_version": WORKFLOW_VERSION,
            "generated_at": generated_at,
            "run_id": spec.run_id,
            "approval_artifact_path": approval.relative_path,
            "approval_artifact_sha256": approval.sha256,
            "agent_id": approval.descriptor.agent_id,
            "manifest_root": approval.descriptor.manifest_root,
            "required_handoff_commands": agent_lifecycle::REQUIRED_PUBLICATION_COMMANDS,
        }),
    )?;
    write_json(
        &run_root.join("run-status.json"),
        &json!({
            "workflow_version": WORKFLOW_VERSION,
            "generated_at": generated_at,
            "run_id": spec.run_id,
            "approval_artifact_path": approval.relative_path,
            "agent_id": approval.descriptor.agent_id,
            "requested_tier": runtime_requested_tier(lifecycle_state),
            "host_surface": spec.host_surface,
            "loaded_skill_ref": spec.loaded_skill_ref,
            "mode": spec.mode,
            "status": "write_validated",
            "validation_passed": true,
            "handoff_ready": true,
            "run_dir": run_root.display().to_string(),
            "written_paths": written_paths,
            "errors": [],
        }),
    )?;
    write_json(
        &run_root.join("validation-report.json"),
        &json!({
            "workflow_version": WORKFLOW_VERSION,
            "generated_at": generated_at,
            "run_id": spec.run_id,
            "status": "pass",
            "checks": [{
                "name": spec.validation_check_name,
                "ok": true,
                "message": spec.validation_message
            }],
            "errors": [],
        }),
    )?;
    write_json(
        &run_root.join("handoff.json"),
        &json!({
            "agent_id": approval.descriptor.agent_id,
            "manifest_root": approval.descriptor.manifest_root,
            "runtime_lane_complete": true,
            "publication_refresh_required": true,
            "required_commands": agent_lifecycle::REQUIRED_PUBLICATION_COMMANDS,
            "blockers": [],
        }),
    )?;
    write_json(&run_root.join("written-paths.json"), &json!(written_paths))?;
    fs::write(
        run_root.join("run-summary.md"),
        render_runtime_summary(
            approval,
            &run_relative,
            &written_paths,
            spec.summary_title,
            spec.source_label,
        ),
    )
    .map_err(|err| Error::Internal(format!("write run-summary.md: {err}")))?;

    Ok(GeneratedRuntimeEvidenceBundle {
        run_id: spec.run_id.to_string(),
        run_relative: run_relative.clone(),
        runtime_evidence_paths: vec![
            format!("{run_relative}/input-contract.json"),
            format!("{run_relative}/run-status.json"),
            format!("{run_relative}/run-summary.md"),
            format!("{run_relative}/validation-report.json"),
            format!("{run_relative}/written-paths.json"),
            format!("{run_relative}/handoff.json"),
        ],
        written_paths,
    })
}

fn derive_runtime_written_paths(
    workspace_root: &Path,
    approval: &ApprovalArtifact,
    lifecycle_state: &LifecycleState,
) -> Result<Vec<String>, Error> {
    let mut written = BTreeSet::new();
    for candidate in [
        format!("{}/src/lib.rs", approval.descriptor.crate_path),
        format!("{}/backend.rs", approval.descriptor.backend_module),
        format!("{}/mod.rs", approval.descriptor.backend_module),
        format!(
            "{}/src/wrapper_coverage_manifest.rs",
            approval.descriptor.wrapper_coverage_source_path
        ),
        format!(
            "crates/agent_api/tests/c1_{}_runtime_follow_on.rs",
            approval.descriptor.agent_id
        ),
    ] {
        if workspace_root.join(&candidate).is_file() {
            written.insert(candidate);
        }
    }

    if let Some(path) = first_file_under(
        workspace_root,
        &approval.descriptor.manifest_root,
        "supplement",
    )? {
        written.insert(path);
    }
    if let Some(path) = first_file_under(
        workspace_root,
        &approval.descriptor.manifest_root,
        "snapshots",
    )? {
        written.insert(path);
    }

    if lifecycle_state
        .implementation_summary
        .as_ref()
        .is_some_and(|summary| {
            summary
                .landed_surfaces
                .contains(&agent_lifecycle::LandedSurface::AgentApiOnboardingTest)
        })
    {
        let agent_test = format!(
            "crates/agent_api/tests/c1_{}_runtime_follow_on.rs",
            approval.descriptor.agent_id
        );
        if workspace_root.join(&agent_test).is_file() {
            written.insert(agent_test);
        }
    }

    if written.is_empty() {
        return Err(Error::Validation(format!(
            "could not derive any committed runtime-owned outputs for `{}`",
            approval.descriptor.agent_id
        )));
    }

    Ok(written.into_iter().collect())
}

fn first_file_under(
    workspace_root: &Path,
    manifest_root: &str,
    child: &str,
) -> Result<Option<String>, Error> {
    let dir = workspace_root.join(manifest_root).join(child);
    if !dir.is_dir() {
        return Ok(None);
    }
    let mut stack = vec![dir];
    let mut files = Vec::new();
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)
            .map_err(|err| Error::Internal(format!("read {}: {err}", dir.display())))?
        {
            let entry = entry.map_err(|err| {
                Error::Internal(format!("read dir entry under {}: {err}", dir.display()))
            })?;
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|err| Error::Internal(format!("stat {}: {err}", path.display())))?;
            if file_type.is_dir() {
                stack.push(path);
            } else if file_type.is_file() {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files
        .into_iter()
        .next()
        .and_then(|path| path.strip_prefix(workspace_root).ok().map(PathBuf::from))
        .map(|path| path.to_string_lossy().replace('\\', "/")))
}

fn render_runtime_summary(
    approval: &ApprovalArtifact,
    run_relative: &str,
    written_paths: &[String],
    summary_title: &str,
    source_label: &str,
) -> String {
    let mut summary = format!(
        "# {summary_title}\n\n- run_id: `{}`\n- status: `pass`\n- agent_id: `{}`\n- source: `{source_label}`\n- run_dir: `{run_relative}`\n",
        run_relative.rsplit('/').next().unwrap_or(run_relative),
        approval.descriptor.agent_id,
    );
    summary.push_str("\n## Written Paths\n");
    for path in written_paths {
        summary.push_str(&format!("- `{path}`\n"));
    }
    summary
}

fn runtime_requested_tier(lifecycle_state: &LifecycleState) -> &'static str {
    match lifecycle_state
        .implementation_summary
        .as_ref()
        .map(|summary| summary.requested_runtime_profile)
    {
        Some(agent_lifecycle::RuntimeProfile::Minimal) => "minimal",
        Some(agent_lifecycle::RuntimeProfile::FeatureRich) => "feature-rich",
        _ => "default",
    }
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| Error::Internal(format!("create {}: {err}", parent.display())))?;
    }
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|err| Error::Internal(format!("serialize {}: {err}", path.display())))?;
    bytes.push(b'\n');
    fs::write(path, bytes)
        .map_err(|err| Error::Internal(format!("write {}: {err}", path.display())))
}
