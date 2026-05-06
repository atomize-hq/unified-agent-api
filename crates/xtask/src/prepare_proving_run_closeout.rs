use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use clap::{ArgGroup, Parser};

use crate::{
    agent_lifecycle::{
        self, file_sha256, load_lifecycle_state, now_rfc3339, required_evidence_for_stage,
        LifecycleStage, LifecycleState, PublicationReadyPacket,
    },
    agent_registry::{AgentRegistry, AgentRegistryEntry},
    approval_artifact::{load_approval_artifact, ApprovalArtifact, ApprovalArtifactError},
    onboard_agent::preview::build_closeout_docs_preview_for_entry,
    proving_run_closeout::{
        build_closeout, build_prepared_closeout, load_validated_closeout_if_present_with_states,
        render_closeout_json, DurationTruth, ProvingRunCloseout, ProvingRunCloseoutError,
        ProvingRunCloseoutExpected, ProvingRunCloseoutHumanFields, ProvingRunCloseoutMachineFields,
        ProvingRunCloseoutState, ResidualFrictionTruth,
    },
    workspace_mutation::{
        apply_mutations, plan_create_or_replace, PlannedMutation, WorkspaceMutationError,
        WorkspacePathJail,
    },
};

const APPROVAL_SOURCE: &str = "governance-review";

#[derive(Debug, Parser, Clone)]
#[command(group(
    ArgGroup::new("mode")
        .required(true)
        .args(["check", "write"])
        .multiple(false)
))]
pub struct Args {
    /// Repo-relative approved onboarding artifact under docs/agents/lifecycle/**/governance/approved-agent.toml.
    #[arg(long)]
    pub approval: String,

    /// Validate that the closeout-preparation contract can be satisfied without rewriting files.
    #[arg(long)]
    pub check: bool,

    /// Materialize the canonical proving-run closeout draft from published lifecycle truth.
    #[arg(long)]
    pub write: bool,
}

#[derive(Debug)]
pub enum Error {
    Validation(String),
    Internal(String),
}

#[derive(Debug, Clone)]
struct PrepareContext {
    approval: ApprovalArtifact,
    entry: AgentRegistryEntry,
    lifecycle_state_path: String,
    lifecycle_state: LifecycleState,
    closeout_path: String,
    existing_closeout: Option<ProvingRunCloseout>,
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Validation(_) => 2,
            Self::Internal(_) => 1,
        }
    }
}

impl From<WorkspaceMutationError> for Error {
    fn from(err: WorkspaceMutationError) -> Self {
        match err {
            WorkspaceMutationError::Validation(message) => Self::Validation(message),
            WorkspaceMutationError::Internal(message) => Self::Internal(message),
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
    let mut stdout = io::stdout();
    run_in_workspace(&workspace_root, args, &mut stdout)
}

pub fn run_in_workspace<W: Write>(
    workspace_root: &Path,
    args: Args,
    writer: &mut W,
) -> Result<(), Error> {
    let context = build_context(workspace_root, &args)?;

    if args.check {
        writeln!(writer, "OK: prepare-proving-run-closeout check passed.")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        writeln!(writer, "closeout_path: {}", context.closeout_path)
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        return Ok(());
    }

    let recorded_at = now_rfc3339().map_err(|err| Error::Internal(err.to_string()))?;
    let closeout = build_closeout_draft(
        &context,
        ProvingRunCloseoutMachineFields {
            approval_ref: context.approval.relative_path.clone(),
            approval_sha256: context.approval.sha256.clone(),
            approval_source: APPROVAL_SOURCE.to_string(),
            preflight_passed: true,
            recorded_at: recorded_at.clone(),
            commit: current_head_commit(workspace_root)?,
        },
    )?;
    let lifecycle_state = build_next_lifecycle_state(&context, recorded_at)?;
    let jail = WorkspacePathJail::new(workspace_root)?;
    let docs_preview = build_closeout_docs_preview_for_entry(&context.entry, &closeout);
    let mut mutations = vec![
        plan_create_or_replace(
            &jail,
            PathBuf::from(&context.closeout_path),
            render_closeout_json(&closeout)
                .map_err(map_closeout_error)?
                .into_bytes(),
        )?,
        plan_create_or_replace(
            &jail,
            PathBuf::from(&context.lifecycle_state_path),
            serialize_json_pretty(&lifecycle_state)?,
        )?,
    ];
    mutations.extend(plan_docs_mutations(&jail, &docs_preview)?);
    apply_mutations(workspace_root, &mutations)?;

    writeln!(writer, "OK: prepare-proving-run-closeout write complete.")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "closeout_path: {}", context.closeout_path)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

fn build_context(workspace_root: &Path, args: &Args) -> Result<PrepareContext, Error> {
    let approval =
        load_approval_artifact(workspace_root, &args.approval).map_err(map_approval_error)?;
    let registry =
        AgentRegistry::load(workspace_root).map_err(|err| Error::Validation(err.to_string()))?;
    let entry = registry
        .find(&approval.descriptor.agent_id)
        .cloned()
        .ok_or_else(|| {
            Error::Validation(format!(
                "approval/registry mismatch: `{}` is not present in {}",
                approval.descriptor.agent_id,
                crate::agent_registry::REGISTRY_RELATIVE_PATH
            ))
        })?;
    let lifecycle_state_path =
        agent_lifecycle::lifecycle_state_path(&approval.descriptor.onboarding_pack_prefix);
    let lifecycle_state = load_lifecycle_state(workspace_root, &lifecycle_state_path)
        .map_err(|err| Error::Validation(err.to_string()))?;
    validate_approval_continuity(&lifecycle_state, &lifecycle_state_path, &approval)?;
    if lifecycle_state.lifecycle_stage != LifecycleStage::Published {
        return Err(Error::Validation(format!(
            "prepare-proving-run-closeout requires lifecycle stage `published` at `{}` (found `{}`)",
            lifecycle_state_path,
            lifecycle_state.lifecycle_stage.as_str()
        )));
    }
    validate_stage_evidence(
        &lifecycle_state,
        LifecycleStage::Published,
        &lifecycle_state_path,
    )?;
    validate_expected_next_command(&approval, &entry, &lifecycle_state_path, &lifecycle_state)?;
    validate_published_continuity(
        workspace_root,
        &approval,
        &entry,
        &lifecycle_state_path,
        &lifecycle_state,
    )?;

    let closeout_path =
        agent_lifecycle::proving_run_closeout_path(&approval.descriptor.onboarding_pack_prefix);
    let existing_closeout = load_validated_closeout_if_present_with_states(
        workspace_root,
        Path::new(&closeout_path),
        &workspace_root.join(&closeout_path),
        ProvingRunCloseoutExpected {
            approval_path: Some(Path::new(&approval.relative_path)),
            onboarding_pack_prefix: &approval.descriptor.onboarding_pack_prefix,
        },
        &[ProvingRunCloseoutState::Prepared],
    )
    .map_err(map_closeout_error)?;

    Ok(PrepareContext {
        approval,
        entry,
        lifecycle_state_path,
        lifecycle_state,
        closeout_path,
        existing_closeout,
    })
}

fn build_closeout_draft(
    context: &PrepareContext,
    machine: ProvingRunCloseoutMachineFields,
) -> Result<ProvingRunCloseout, Error> {
    match &context.existing_closeout {
        Some(closeout) => build_closeout(
            ProvingRunCloseoutState::Prepared,
            machine,
            ProvingRunCloseoutHumanFields {
                manual_control_plane_edits: closeout.manual_control_plane_edits,
                partial_write_incidents: closeout.partial_write_incidents,
                ambiguous_ownership_incidents: closeout.ambiguous_ownership_incidents,
                duration: match &closeout.duration {
                    DurationTruth::Seconds(seconds) => DurationTruth::Seconds(*seconds),
                    DurationTruth::MissingReason(reason) => {
                        DurationTruth::MissingReason(reason.clone())
                    }
                },
                residual_friction: match &closeout.residual_friction {
                    ResidualFrictionTruth::Items(items) => {
                        ResidualFrictionTruth::Items(items.clone())
                    }
                    ResidualFrictionTruth::ExplicitNone(reason) => {
                        ResidualFrictionTruth::ExplicitNone(reason.clone())
                    }
                },
            },
        )
        .map_err(map_closeout_error),
        None => build_prepared_closeout(machine).map_err(map_closeout_error),
    }
}

fn build_next_lifecycle_state(
    context: &PrepareContext,
    recorded_at: String,
) -> Result<LifecycleState, Error> {
    let mut next_state = context.lifecycle_state.clone();
    next_state.current_owner_command = "prepare-proving-run-closeout --write".to_string();
    next_state.expected_next_command = agent_lifecycle::publication_ready_closeout_command(
        &context.approval.relative_path,
        &context.entry.scaffold.onboarding_pack_prefix,
    );
    next_state.last_transition_at = recorded_at;
    next_state.last_transition_by = "xtask prepare-proving-run-closeout --write".to_string();
    next_state
        .validate()
        .map_err(|err| Error::Validation(err.to_string()))?;
    Ok(next_state)
}

fn current_head_commit(workspace_root: &Path) -> Result<String, Error> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .current_dir(workspace_root)
        .output()
        .map_err(|err| Error::Internal(format!("run `git rev-parse HEAD`: {err}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::Internal(format!(
            "`git rev-parse HEAD` failed with exit code {}: {}",
            output.status.code().unwrap_or(1),
            stderr.trim()
        )));
    }
    let commit = String::from_utf8(output.stdout)
        .map_err(|err| Error::Internal(format!("parse `git rev-parse HEAD` stdout: {err}")))?;
    Ok(commit.trim().to_string())
}

fn validate_approval_continuity(
    lifecycle_state: &LifecycleState,
    lifecycle_state_path: &str,
    approval: &ApprovalArtifact,
) -> Result<(), Error> {
    if lifecycle_state.approval_artifact_path != approval.relative_path {
        return Err(Error::Validation(format!(
            "`{lifecycle_state_path}` approval_artifact_path `{}` does not match `{}`",
            lifecycle_state.approval_artifact_path, approval.relative_path
        )));
    }
    if lifecycle_state.approval_artifact_sha256 != approval.sha256 {
        return Err(Error::Validation(format!(
            "`{lifecycle_state_path}` approval_artifact_sha256 does not match `{}`",
            approval.relative_path
        )));
    }
    Ok(())
}

fn validate_stage_evidence(
    lifecycle_state: &LifecycleState,
    expected_stage: LifecycleStage,
    lifecycle_state_path: &str,
) -> Result<(), Error> {
    let missing = required_evidence_for_stage(expected_stage)
        .iter()
        .filter(|evidence| !lifecycle_state.satisfied_evidence.contains(evidence))
        .map(|evidence| evidence.as_str().to_string())
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return Ok(());
    }

    Err(Error::Validation(format!(
        "`{lifecycle_state_path}` is missing required evidence for `{}`: {}",
        expected_stage.as_str(),
        missing.join(", ")
    )))
}

fn validate_expected_next_command(
    approval: &ApprovalArtifact,
    entry: &AgentRegistryEntry,
    lifecycle_state_path: &str,
    lifecycle_state: &LifecycleState,
) -> Result<(), Error> {
    let prepare_command =
        agent_lifecycle::published_prepare_closeout_command(&approval.relative_path);
    let closeout_command = agent_lifecycle::publication_ready_closeout_command(
        &approval.relative_path,
        &entry.scaffold.onboarding_pack_prefix,
    );
    if lifecycle_state.expected_next_command == prepare_command
        || lifecycle_state.expected_next_command == closeout_command
    {
        return Ok(());
    }

    Err(Error::Validation(format!(
        "`{}` has stale expected_next_command `{}`; expected `{}` or `{}`",
        lifecycle_state_path,
        lifecycle_state.expected_next_command,
        prepare_command,
        closeout_command
    )))
}

fn validate_published_continuity(
    workspace_root: &Path,
    approval: &ApprovalArtifact,
    entry: &AgentRegistryEntry,
    lifecycle_state_path: &str,
    lifecycle_state: &LifecycleState,
) -> Result<(), Error> {
    let publication_packet_path = agent_lifecycle::publication_ready_path_for_entry(entry);
    if lifecycle_state.publication_packet_path.as_deref() != Some(publication_packet_path.as_str())
    {
        return Err(Error::Validation(format!(
            "`{}` published continuity must record publication_packet_path `{}` before closeout preparation",
            lifecycle_state_path, publication_packet_path
        )));
    }

    let publication_packet_sha256 = lifecycle_state
        .publication_packet_sha256
        .as_deref()
        .ok_or_else(|| {
            Error::Validation(format!(
                "`{}` published continuity must record publication_packet_sha256 for `{}` before closeout preparation",
                lifecycle_state_path, publication_packet_path
            ))
        })?;
    let actual_sha256 = file_sha256(workspace_root, &publication_packet_path)
        .map_err(|err| Error::Validation(err.to_string()))?;
    if publication_packet_sha256 != actual_sha256 {
        return Err(Error::Validation(format!(
            "`{}` published continuity must record the current publication_packet_sha256 for `{}` before closeout preparation",
            lifecycle_state_path, publication_packet_path
        )));
    }

    let packet = load_published_publication_packet(workspace_root, &publication_packet_path)
        .map_err(|err| Error::Validation(err.to_string()))?;
    validate_packet_identity(
        &publication_packet_path,
        &packet,
        approval,
        entry,
        lifecycle_state_path,
    )
}

fn load_published_publication_packet(
    workspace_root: &Path,
    packet_path: &str,
) -> Result<PublicationReadyPacket, crate::agent_lifecycle::LifecycleError> {
    let packet_bytes = fs::read(workspace_root.join(packet_path)).map_err(|err| {
        crate::agent_lifecycle::LifecycleError::Validation(format!("read {packet_path}: {err}"))
    })?;
    let packet: PublicationReadyPacket = serde_json::from_slice(&packet_bytes).map_err(|err| {
        crate::agent_lifecycle::LifecycleError::Validation(format!("parse {packet_path}: {err}"))
    })?;
    packet
        .validate()
        .map_err(|err| crate::agent_lifecycle::LifecycleError::Validation(err.to_string()))?;
    Ok(packet)
}

fn validate_packet_identity(
    packet_path: &str,
    packet: &PublicationReadyPacket,
    approval: &ApprovalArtifact,
    entry: &AgentRegistryEntry,
    lifecycle_state_path: &str,
) -> Result<(), Error> {
    if packet.approval_artifact_path != approval.relative_path {
        return Err(Error::Validation(format!(
            "`{packet_path}` approval_artifact_path `{}` does not match `{}`",
            packet.approval_artifact_path, approval.relative_path
        )));
    }
    if packet.approval_artifact_sha256 != approval.sha256 {
        return Err(Error::Validation(format!(
            "`{packet_path}` approval_artifact_sha256 does not match `{}`",
            approval.relative_path
        )));
    }
    if packet.agent_id != approval.descriptor.agent_id {
        return Err(Error::Validation(format!(
            "`{packet_path}` agent_id `{}` does not match `{}`",
            packet.agent_id, approval.descriptor.agent_id
        )));
    }
    if packet.lifecycle_state_path != lifecycle_state_path {
        return Err(Error::Validation(format!(
            "`{packet_path}` lifecycle_state_path `{}` does not match `{}`",
            packet.lifecycle_state_path, lifecycle_state_path
        )));
    }
    if packet.manifest_root != entry.manifest_root {
        return Err(Error::Validation(format!(
            "`{packet_path}` manifest_root `{}` does not match `{}`",
            packet.manifest_root, entry.manifest_root
        )));
    }
    Ok(())
}

fn serialize_json_pretty<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|err| Error::Internal(format!("serialize json: {err}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn plan_docs_mutations(
    jail: &WorkspacePathJail,
    docs_preview: &[(String, Option<String>)],
) -> Result<Vec<PlannedMutation>, Error> {
    docs_preview
        .iter()
        .map(|(relative_path, contents)| {
            plan_create_or_replace(
                jail,
                PathBuf::from(relative_path),
                contents.clone().unwrap_or_default().into_bytes(),
            )
            .map_err(Error::from)
        })
        .collect()
}

fn resolve_workspace_root() -> Result<PathBuf, Error> {
    let current_dir = std::env::current_dir()
        .map_err(|err| Error::Internal(format!("resolve current directory: {err}")))?;
    for candidate in current_dir.ancestors() {
        let cargo_toml = candidate.join("Cargo.toml");
        let Ok(text) = std::fs::read_to_string(&cargo_toml) else {
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

fn map_approval_error(err: ApprovalArtifactError) -> Error {
    match err {
        ApprovalArtifactError::Validation(message) => Error::Validation(message),
        ApprovalArtifactError::Internal(message) => Error::Internal(message),
    }
}

fn map_closeout_error(err: ProvingRunCloseoutError) -> Error {
    match err {
        ProvingRunCloseoutError::Validation(message) => Error::Validation(message),
        ProvingRunCloseoutError::Internal(message) => Error::Internal(message),
    }
}
