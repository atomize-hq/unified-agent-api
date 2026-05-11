use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use clap::Parser;
use serde::Deserialize;
use sha2::Digest;
use xtask::{
    agent_lifecycle::{
        self, file_sha256, lifecycle_state_path, proving_run_closeout_path, publication_ready_path,
        reconstruct_publication_ready_state_from_closed_baseline, required_evidence_for_stage,
        write_lifecycle_state, write_publication_ready_packet, LifecycleStage, LifecycleState,
    },
    agent_registry::{AgentRegistry, AgentRegistryEntry},
    approval_artifact::{ApprovalArtifact, ApprovalMaintenanceMode},
    prepare_publication::{
        build_publication_ready_packet, validate_runtime_evidence_run_for_approval,
        RuntimeEvidenceBundle,
    },
    proving_run_closeout::{
        build_closeout, render_closeout_json, DurationTruth, MaintenanceSettlement,
        MaintenanceSettlementMode, ProvingRunCloseoutHumanFields, ProvingRunCloseoutMachineFields,
        ProvingRunCloseoutState, ResidualFrictionTruth,
    },
    runtime_evidence_bundle::{
        self, historical_run_id, write_runtime_evidence_bundle, RuntimeEvidenceBundleSpec,
    },
};

const HISTORICAL_AGENTS: [&str; 4] = ["codex", "claude_code", "opencode", "gemini_cli"];
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
    let mut normalized_state = raw_state.clone();
    normalized_state.approval_artifact_sha256 = approval.sha256.clone();

    let runtime_evidence = write_runtime_evidence(workspace_root, &approval, &normalized_state)?;
    validate_runtime_evidence_run_for_approval(workspace_root, &approval, &runtime_evidence.run_id)
        .map_err(|err| Error::Validation(err.to_string()))?;
    let historical_publication_state =
        reconstruct_publication_ready_state_from_closed_baseline(&normalized_state);
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
        entry,
        &approval,
        &closeout_path,
        &normalized_state,
    )?;

    let mut repaired_state = normalized_state.clone();
    repaired_state.required_evidence =
        required_evidence_for_stage(LifecycleStage::ClosedBaseline).to_vec();
    repaired_state.satisfied_evidence =
        required_evidence_for_stage(LifecycleStage::ClosedBaseline).to_vec();
    repaired_state.active_runtime_evidence_run_id = None;
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
    let bundle = write_runtime_evidence_bundle(
        workspace_root,
        approval,
        lifecycle_state,
        &RuntimeEvidenceBundleSpec {
            run_id: &historical_run_id(&approval.descriptor.agent_id),
            host_surface: "xtask historical-lifecycle-backfill",
            loaded_skill_ref: "historical-lifecycle-backfill",
            mode: "historical_backfill",
            source_label: "historical-lifecycle-backfill",
            summary_title: "Runtime Follow-On Validation",
            validation_check_name: "historical_runtime_evidence",
            validation_message:
                "historical runtime evidence was reconstructed from committed runtime-owned outputs",
        },
    )
    .map_err(map_runtime_evidence_bundle_error)?;

    Ok(RuntimeEvidenceBundle {
        run_id: bundle.run_id,
        runtime_evidence_paths: bundle.runtime_evidence_paths,
    })
}

fn ensure_closeout(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    approval: &ApprovalArtifact,
    closeout_path: &str,
    lifecycle_state: &LifecycleState,
) -> Result<(), Error> {
    let absolute = workspace_root.join(closeout_path);
    let maintenance_settlement = settle_maintenance_readiness(entry, approval, closeout_path)?;
    let existing = load_existing_closeout_seed(&absolute)
        .map_err(|err| Error::Validation(format!("parse {}: {err}", absolute.display())))?;
    let recorded_commit = existing
        .as_ref()
        .and_then(|seed| seed.commit.clone())
        .or_else(|| git_first_add_metadata(workspace_root, closeout_path).map(|(commit, _)| commit))
        .unwrap_or_else(|| approval.sha256[..40.min(approval.sha256.len())].to_string());
    let recorded_at = existing
        .as_ref()
        .and_then(|seed| seed.recorded_at.clone())
        .unwrap_or_else(|| lifecycle_state.last_transition_at.clone());
    let closeout = build_closeout(
        ProvingRunCloseoutState::Closed,
        ProvingRunCloseoutMachineFields {
            approval_ref: approval.relative_path.clone(),
            approval_sha256: approval.sha256.clone(),
            approval_source: APPROVAL_SOURCE.to_string(),
            maintenance_settlement: Some(maintenance_settlement),
            preflight_passed: true,
            recorded_at,
            commit: recorded_commit,
        },
        ProvingRunCloseoutHumanFields {
            manual_control_plane_edits: existing
                .as_ref()
                .map_or(0, |seed| seed.manual_control_plane_edits),
            partial_write_incidents: existing
                .as_ref()
                .map_or(0, |seed| seed.partial_write_incidents),
            ambiguous_ownership_incidents: existing
                .as_ref()
                .map_or(0, |seed| seed.ambiguous_ownership_incidents),
            duration: existing.as_ref().map_or_else(
                || DurationTruth::MissingReason(DURATION_MISSING_REASON.to_string()),
                ExistingCloseoutSeed::duration_truth,
            ),
            residual_friction: existing.as_ref().map_or_else(
                || ResidualFrictionTruth::ExplicitNone(EXPLICIT_NONE_REASON.to_string()),
                ExistingCloseoutSeed::residual_friction_truth,
            ),
        },
    )
    .map_err(map_closeout_error)?;
    fs::write(
        &absolute,
        render_closeout_json(&closeout)
            .map_err(map_closeout_error)?
            .into_bytes(),
    )
    .map_err(|err| Error::Internal(format!("write {}: {err}", absolute.display())))
}

fn settle_maintenance_readiness(
    entry: &AgentRegistryEntry,
    approval: &ApprovalArtifact,
    closeout_path: &str,
) -> Result<MaintenanceSettlement, Error> {
    match &approval.maintenance.mode {
        ApprovalMaintenanceMode::ReleaseWatchEnrolled {
            release_watch_sha256,
            ..
        } => {
            let registry_release_watch = entry.maintenance.release_watch.as_ref().ok_or_else(|| {
                Error::Validation(format!(
                    "{closeout_path}: approval maintenance mode `release_watch_enrolled` requires registry `maintenance.release_watch` for `{}`",
                    entry.agent_id
                ))
            })?;
            let registry_release_watch_sha256 =
                xtask::agent_registry::normalized_release_watch_sha256(registry_release_watch)
                    .map_err(|err| {
                        Error::Validation(format!(
                            "{closeout_path}: registry maintenance.release_watch is invalid for `{}`: {err}",
                            entry.agent_id
                        ))
                    })?;
            if registry_release_watch_sha256 != *release_watch_sha256 {
                return Err(Error::Validation(format!(
                    "{closeout_path}: approval and registry maintenance.release_watch truth diverge for `{}`",
                    entry.agent_id
                )));
            }
            Ok(MaintenanceSettlement {
                mode: MaintenanceSettlementMode::ReleaseWatchEnrolled,
                approval_section_sha256: approval.maintenance.section_sha256.clone(),
                release_watch_sha256: Some(registry_release_watch_sha256),
                deferral_sha256: None,
            })
        }
        ApprovalMaintenanceMode::ExplicitlyDeferred {
            deferral_sha256, ..
        } => {
            if entry.maintenance.release_watch.is_some() {
                return Err(Error::Validation(format!(
                    "{closeout_path}: approval maintenance mode `explicitly_deferred` forbids registry `maintenance.release_watch` for `{}`",
                    entry.agent_id
                )));
            }
            Ok(MaintenanceSettlement {
                mode: MaintenanceSettlementMode::ExplicitlyDeferred,
                approval_section_sha256: approval.maintenance.section_sha256.clone(),
                release_watch_sha256: None,
                deferral_sha256: Some(deferral_sha256.clone()),
            })
        }
    }
}

fn load_existing_closeout_seed(path: &Path) -> Result<Option<ExistingCloseoutSeed>, String> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(format!("read {}: {err}", path.display())),
    };
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|err| err.to_string())
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

fn map_runtime_evidence_bundle_error(err: runtime_evidence_bundle::Error) -> Error {
    match err {
        runtime_evidence_bundle::Error::Validation(message) => Error::Validation(message),
        runtime_evidence_bundle::Error::Internal(message) => Error::Internal(message),
    }
}

fn map_closeout_error(err: xtask::proving_run_closeout::ProvingRunCloseoutError) -> Error {
    match err {
        xtask::proving_run_closeout::ProvingRunCloseoutError::Validation(message) => {
            Error::Validation(message)
        }
        xtask::proving_run_closeout::ProvingRunCloseoutError::Internal(message) => {
            Error::Internal(message)
        }
    }
}

#[derive(Debug, Deserialize)]
struct ExistingCloseoutSeed {
    #[serde(default)]
    manual_control_plane_edits: u64,
    #[serde(default)]
    partial_write_incidents: u64,
    #[serde(default)]
    ambiguous_ownership_incidents: u64,
    recorded_at: Option<String>,
    commit: Option<String>,
    duration_seconds: Option<u64>,
    duration_missing_reason: Option<String>,
    residual_friction: Option<Vec<String>>,
    explicit_none_reason: Option<String>,
}

impl ExistingCloseoutSeed {
    fn duration_truth(&self) -> DurationTruth {
        match (self.duration_seconds, self.duration_missing_reason.as_ref()) {
            (Some(seconds), None) => DurationTruth::Seconds(seconds),
            _ => DurationTruth::MissingReason(
                self.duration_missing_reason
                    .clone()
                    .unwrap_or_else(|| DURATION_MISSING_REASON.to_string()),
            ),
        }
    }

    fn residual_friction_truth(&self) -> ResidualFrictionTruth {
        match (
            self.residual_friction.as_ref(),
            self.explicit_none_reason.as_ref(),
        ) {
            (Some(items), None) => ResidualFrictionTruth::Items(items.clone()),
            _ => ResidualFrictionTruth::ExplicitNone(
                self.explicit_none_reason
                    .clone()
                    .unwrap_or_else(|| EXPLICIT_NONE_REASON.to_string()),
            ),
        }
    }
}
