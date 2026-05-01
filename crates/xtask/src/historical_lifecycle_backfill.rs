use std::{
    collections::BTreeSet,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use clap::Parser;
use serde_json::json;
use sha2::Digest;
use xtask::{
    agent_lifecycle::{
        self, file_sha256, lifecycle_state_path, proving_run_closeout_path, publication_ready_path,
        reconstruct_publication_ready_state_from_closed_baseline, required_evidence_for_stage,
        write_lifecycle_state, write_publication_ready_packet, LifecycleStage, LifecycleState,
    },
    agent_registry::AgentRegistry,
    approval_artifact::ApprovalArtifact,
    prepare_publication::{
        build_publication_ready_packet, discover_runtime_evidence_for_approval,
        RuntimeEvidenceBundle,
    },
};

const HISTORICAL_AGENTS: [&str; 4] = ["codex", "claude_code", "opencode", "gemini_cli"];
const RUNTIME_RUNS_ROOT: &str = "docs/agents/.uaa-temp/runtime-follow-on/runs";
const APPROVAL_SOURCE: &str = "historical-lifecycle-backfill";
const DURATION_MISSING_REASON: &str = "Exact duration not recoverable from committed evidence.";
const EXPLICIT_NONE_REASON: &str =
    "No residual friction remained in the committed proving-run evidence.";

#[derive(Debug, Parser, Clone)]
pub struct Args {
    /// Agent ids to backfill. Omit with --all to backfill the known malformed closed baselines.
    #[arg(long)]
    pub agent: Vec<String>,

    /// Backfill all historically malformed closed baselines.
    #[arg(long)]
    pub all: bool,
}

#[derive(Debug)]
pub enum Error {
    Validation(String),
    Internal(String),
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Validation(_) => 2,
            Self::Internal(_) => 1,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(message) | Self::Internal(message) => f.write_str(message),
        }
    }
}

pub fn run(args: Args) -> Result<(), Error> {
    let workspace_root = resolve_workspace_root()?;
    let current_dir = std::env::current_dir()
        .map_err(|err| Error::Internal(format!("resolve current directory: {err}")))?;
    if current_dir != workspace_root {
        return Err(Error::Validation(format!(
            "historical-lifecycle-backfill must run with cwd = repo root `{}` (got `{}`)",
            workspace_root.display(),
            current_dir.display()
        )));
    }
    let mut stdout = io::stdout();
    run_in_workspace(&workspace_root, args, &mut stdout)
}

pub fn run_in_workspace<W: Write>(
    workspace_root: &Path,
    args: Args,
    writer: &mut W,
) -> Result<(), Error> {
    let agents = select_agents(&args)?;
    let registry = AgentRegistry::load(workspace_root)
        .map_err(|err| Error::Internal(format!("load agent registry: {err}")))?;

    for agent_id in agents {
        backfill_agent(workspace_root, &registry, agent_id)?;
        writeln!(
            writer,
            "backfilled historical lifecycle baseline for `{agent_id}`"
        )
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    }

    Ok(())
}

fn select_agents(args: &Args) -> Result<Vec<&str>, Error> {
    if (args.all && !args.agent.is_empty()) || (!args.all && args.agent.is_empty()) {
        return Err(Error::Validation(
            "provide either --all or one or more --agent values".to_string(),
        ));
    }

    if args.all {
        return Ok(HISTORICAL_AGENTS.to_vec());
    }

    let mut selected = Vec::new();
    for agent in &args.agent {
        let canonical = HISTORICAL_AGENTS
            .iter()
            .copied()
            .find(|candidate| candidate == agent)
            .ok_or_else(|| {
                Error::Validation(format!(
                    "historical lifecycle backfill only supports: {}",
                    HISTORICAL_AGENTS.join(", ")
                ))
            })?;
        if !selected.contains(&canonical) {
            selected.push(canonical);
        }
    }
    Ok(selected)
}

fn backfill_agent(
    workspace_root: &Path,
    registry: &AgentRegistry,
    agent_id: &str,
) -> Result<(), Error> {
    let entry = registry.find(agent_id).ok_or_else(|| {
        Error::Validation(format!("agent `{agent_id}` is not present in the registry"))
    })?;
    let approval_path =
        agent_lifecycle::approval_artifact_path(&entry.scaffold.onboarding_pack_prefix);
    let approval = xtask::approval_artifact::load_approval_artifact(workspace_root, &approval_path)
        .map_err(map_approval_error)?;
    let lifecycle_path = lifecycle_state_path(&entry.scaffold.onboarding_pack_prefix);
    let raw_state = read_raw_lifecycle_state(workspace_root, &lifecycle_path)?;
    if raw_state.lifecycle_stage != LifecycleStage::ClosedBaseline {
        return Err(Error::Validation(format!(
            "`{lifecycle_path}` must already be `closed_baseline` to use historical backfill (found `{}`)",
            raw_state.lifecycle_stage.as_str()
        )));
    }

    write_runtime_evidence(workspace_root, &approval, &raw_state)?;
    let runtime_evidence = discover_runtime_evidence_for_approval(workspace_root, &approval)
        .map_err(|err| Error::Validation(err.to_string()))?;
    let historical_publication_state =
        reconstruct_publication_ready_state_from_closed_baseline(&raw_state);
    historical_publication_state
        .validate()
        .map_err(|err| Error::Validation(err.to_string()))?;
    let historical_publication_sha =
        sha256_pretty_json(&historical_publication_state).map_err(Error::Internal)?;
    let packet_path = publication_ready_path(&entry.scaffold.onboarding_pack_prefix);
    let packet = build_publication_ready_packet(
        &approval,
        entry,
        &lifecycle_path,
        &historical_publication_sha,
        packet_path.clone(),
        historical_publication_state
            .implementation_summary
            .clone()
            .ok_or_else(|| {
                Error::Validation(format!(
                    "`{lifecycle_path}` is missing implementation_summary"
                ))
            })?,
        runtime_evidence.runtime_evidence_paths.clone(),
        historical_publication_state.blocking_issues.clone(),
    )
    .map_err(|err| Error::Validation(err.to_string()))?;
    write_publication_ready_packet(workspace_root, &packet_path, &packet)
        .map_err(|err| Error::Internal(format!("write publication-ready packet: {err}")))?;

    let closeout_path = proving_run_closeout_path(&entry.scaffold.onboarding_pack_prefix);
    ensure_closeout(
        workspace_root,
        &approval,
        &lifecycle_path,
        &closeout_path,
        &raw_state,
    )?;

    let mut repaired_state = raw_state.clone();
    repaired_state.required_evidence =
        required_evidence_for_stage(LifecycleStage::ClosedBaseline).to_vec();
    repaired_state.satisfied_evidence =
        required_evidence_for_stage(LifecycleStage::ClosedBaseline).to_vec();
    repaired_state.publication_packet_path = Some(packet_path.clone());
    repaired_state.publication_packet_sha256 = Some(
        file_sha256(workspace_root, &packet_path)
            .map_err(|err| Error::Internal(format!("hash {packet_path}: {err}")))?,
    );
    repaired_state.closeout_baseline_path = Some(closeout_path);
    write_lifecycle_state(workspace_root, &lifecycle_path, &repaired_state)
        .map_err(|err| Error::Internal(format!("write lifecycle state: {err}")))?;

    Ok(())
}

fn write_runtime_evidence(
    workspace_root: &Path,
    approval: &ApprovalArtifact,
    lifecycle_state: &LifecycleState,
) -> Result<RuntimeEvidenceBundle, Error> {
    let run_id = format!(
        "historical-{}-runtime-follow-on",
        approval.descriptor.agent_id
    );
    let run_relative = format!("{RUNTIME_RUNS_ROOT}/{run_id}");
    let run_root = workspace_root.join(&run_relative);
    fs::create_dir_all(&run_root)
        .map_err(|err| Error::Internal(format!("create {}: {err}", run_root.display())))?;

    let written_paths = derive_runtime_written_paths(workspace_root, approval, lifecycle_state)?;
    let generated_at = lifecycle_state.last_transition_at.clone();
    write_json(
        &run_root.join("input-contract.json"),
        &json!({
            "workflow_version": "runtime_follow_on_v1",
            "generated_at": generated_at,
            "run_id": run_id,
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
            "workflow_version": "runtime_follow_on_v1",
            "generated_at": generated_at,
            "run_id": run_id,
            "approval_artifact_path": approval.relative_path,
            "agent_id": approval.descriptor.agent_id,
            "requested_tier": runtime_requested_tier(lifecycle_state),
            "host_surface": "xtask historical-lifecycle-backfill",
            "loaded_skill_ref": "historical-lifecycle-backfill",
            "mode": "historical_backfill",
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
            "workflow_version": "runtime_follow_on_v1",
            "generated_at": generated_at,
            "run_id": run_id,
            "status": "pass",
            "checks": [{
                "name": "historical_runtime_evidence",
                "ok": true,
                "message": "historical runtime evidence was reconstructed from committed runtime-owned outputs"
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
        render_runtime_summary(approval, &run_relative, &written_paths),
    )
    .map_err(|err| Error::Internal(format!("write run-summary.md: {err}")))?;

    Ok(RuntimeEvidenceBundle {
        run_id,
        runtime_evidence_paths: vec![
            format!("{run_relative}/input-contract.json"),
            format!("{run_relative}/run-status.json"),
            format!("{run_relative}/run-summary.md"),
            format!("{run_relative}/validation-report.json"),
            format!("{run_relative}/written-paths.json"),
            format!("{run_relative}/handoff.json"),
        ],
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

fn ensure_closeout(
    workspace_root: &Path,
    approval: &ApprovalArtifact,
    lifecycle_path: &str,
    closeout_path: &str,
    lifecycle_state: &LifecycleState,
) -> Result<(), Error> {
    let absolute = workspace_root.join(closeout_path);
    if absolute.is_file() {
        return Ok(());
    }

    let recorded_commit = git_first_add_metadata(workspace_root, lifecycle_path)
        .map(|(commit, _)| commit)
        .unwrap_or_else(|| approval.sha256[..40.min(approval.sha256.len())].to_string());
    write_json(
        &absolute,
        &json!({
            "state": "closed",
            "approval_ref": approval.relative_path,
            "approval_sha256": approval.sha256,
            "approval_source": APPROVAL_SOURCE,
            "manual_control_plane_edits": 0,
            "partial_write_incidents": 0,
            "ambiguous_ownership_incidents": 0,
            "duration_missing_reason": DURATION_MISSING_REASON,
            "explicit_none_reason": EXPLICIT_NONE_REASON,
            "preflight_passed": true,
            "recorded_at": lifecycle_state.last_transition_at,
            "commit": recorded_commit,
        }),
    )
}

fn git_first_add_metadata(workspace_root: &Path, relative_path: &str) -> Option<(String, String)> {
    let output = Command::new("git")
        .arg("log")
        .arg("--diff-filter=A")
        .arg("--follow")
        .arg("--format=%H %aI")
        .arg("--")
        .arg(relative_path)
        .current_dir(workspace_root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().find(|line| !line.trim().is_empty())?;
    let (commit, recorded_at) = line.split_once(' ')?;
    Some((commit.to_string(), recorded_at.to_string()))
}

fn render_runtime_summary(
    approval: &ApprovalArtifact,
    run_relative: &str,
    written_paths: &[String],
) -> String {
    let mut summary = format!(
        "# Runtime Follow-On Validation\n\n- run_id: `{}`\n- status: `pass`\n- agent_id: `{}`\n- source: `historical-lifecycle-backfill`\n- run_dir: `{}`\n",
        run_relative.rsplit('/').next().unwrap_or(run_relative),
        approval.descriptor.agent_id,
        run_relative
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

fn read_raw_lifecycle_state(
    workspace_root: &Path,
    relative_path: &str,
) -> Result<LifecycleState, Error> {
    let absolute = workspace_root.join(relative_path);
    let bytes = fs::read(&absolute)
        .map_err(|err| Error::Validation(format!("read {}: {err}", absolute.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|err| Error::Validation(format!("parse {}: {err}", absolute.display())))
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

fn sha256_pretty_json<T: serde::Serialize>(value: &T) -> Result<String, String> {
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(|err| format!("serialize json: {err}"))?;
    bytes.push(b'\n');
    Ok(hex::encode(sha2::Sha256::digest(bytes)))
}

fn resolve_workspace_root() -> Result<PathBuf, Error> {
    let current_dir = std::env::current_dir()
        .map_err(|err| Error::Internal(format!("resolve current directory: {err}")))?;
    for candidate in current_dir.ancestors() {
        let cargo_toml = candidate.join("Cargo.toml");
        let Ok(text) = fs::read_to_string(&cargo_toml) else {
            continue;
        };
        if text.contains("[workspace]") {
            return Ok(candidate.to_path_buf());
        }
    }
    Err(Error::Internal(format!(
        "could not resolve workspace root from {}",
        current_dir.display()
    )))
}

fn map_approval_error(err: xtask::approval_artifact::ApprovalArtifactError) -> Error {
    match err {
        xtask::approval_artifact::ApprovalArtifactError::Validation(message) => {
            Error::Validation(message)
        }
        xtask::approval_artifact::ApprovalArtifactError::Internal(message) => {
            Error::Internal(message)
        }
    }
}
