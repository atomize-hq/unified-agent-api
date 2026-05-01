mod runtime_evidence;

use std::{
    fs,
    io::{self, Write},
    path::{Component, Path, PathBuf},
};

use clap::{ArgGroup, Parser};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use self::runtime_evidence::discover_runtime_evidence;
pub use self::runtime_evidence::RuntimeEvidenceBundle;
use crate::{
    agent_lifecycle::{
        self, file_sha256, load_lifecycle_state, load_publication_ready_packet, now_rfc3339,
        publication_ready_path_for_entry, required_evidence_for_stage, write_lifecycle_state,
        LifecycleStage, LifecycleState, PublicationReadyPacket, SideState, SupportTier,
        LIFECYCLE_SCHEMA_VERSION,
    },
    agent_registry::{AgentRegistry, AgentRegistryEntry},
    approval_artifact::{load_approval_artifact, ApprovalArtifact, ApprovalArtifactError},
    capability_matrix, support_matrix,
};

const RUNTIME_RUNS_ROOT: &str = "docs/agents/.uaa-temp/runtime-follow-on/runs";
const REQUIRED_RUNTIME_EVIDENCE_FILES: [&str; 6] = [
    "input-contract.json",
    "run-status.json",
    "run-summary.md",
    "validation-report.json",
    "written-paths.json",
    "handoff.json",
];

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

    /// Revalidate the publication seam without rewriting committed governance output.
    #[arg(long)]
    pub check: bool,

    /// Write the committed publication handoff packet and advance lifecycle state.
    #[arg(long)]
    pub write: bool,
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

#[derive(Debug, Clone)]
struct PublicationContext {
    approval: ApprovalArtifact,
    entry: AgentRegistryEntry,
    lifecycle_state_path: String,
    lifecycle_state: LifecycleState,
    publication_packet_path: String,
    runtime_evidence: RuntimeEvidenceBundle,
}

pub fn run(args: Args) -> Result<(), Error> {
    let workspace_root = resolve_workspace_root()?;
    let current_dir = std::env::current_dir()
        .map_err(|err| Error::Internal(format!("resolve current directory: {err}")))?;
    if current_dir != workspace_root {
        return Err(Error::Validation(format!(
            "prepare-publication must run with cwd = repo root `{}` (got `{}`)",
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
    let context = build_context(workspace_root, &args)?;
    write_header(writer, &context, args.write)?;

    if args.check {
        validate_check_mode(workspace_root, &context)?;
        writeln!(writer, "OK: prepare-publication check passed.")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        writeln!(
            writer,
            "publication_packet: {}",
            context.publication_packet_path
        )
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        return Ok(());
    }

    if context.lifecycle_state.lifecycle_stage != LifecycleStage::RuntimeIntegrated {
        return Err(Error::Validation(format!(
            "prepare-publication --write requires lifecycle stage `runtime_integrated` at `{}` (found `{}`)",
            context.lifecycle_state_path,
            context.lifecycle_state.lifecycle_stage.as_str()
        )));
    }
    validate_stage_evidence(
        &context.lifecycle_state,
        LifecycleStage::RuntimeIntegrated,
        &context.lifecycle_state_path,
    )?;
    let implementation_summary = require_explicit_implementation_summary(
        &context.lifecycle_state,
        &context.lifecycle_state_path,
    )?
    .clone();

    let mut next_state = context.lifecycle_state.clone();
    next_state.lifecycle_stage = LifecycleStage::PublicationReady;
    next_state.support_tier = SupportTier::BaselineRuntime;
    next_state.current_owner_command = "prepare-publication --write".to_string();
    next_state.expected_next_command = agent_lifecycle::PUBLICATION_READY_NEXT_COMMAND.to_string();
    next_state.last_transition_at =
        now_rfc3339().map_err(|err| Error::Internal(err.to_string()))?;
    next_state.last_transition_by = "xtask prepare-publication --write".to_string();
    next_state.required_evidence =
        required_evidence_for_stage(LifecycleStage::PublicationReady).to_vec();
    next_state.satisfied_evidence =
        required_evidence_for_stage(LifecycleStage::PublicationReady).to_vec();
    next_state
        .side_states
        .retain(|state| !matches!(state, SideState::Blocked | SideState::FailedRetryable));
    next_state.blocking_issues.clear();
    next_state.retryable_failures.clear();
    next_state.implementation_summary = Some(implementation_summary.clone());
    next_state.publication_packet_path = None;
    next_state.publication_packet_sha256 = None;
    next_state
        .validate()
        .map_err(|err| Error::Validation(err.to_string()))?;

    let lifecycle_state_sha256 = sha256_json(&next_state)?;
    let packet = build_publication_ready_packet(
        &context.approval,
        &context.entry,
        &context.lifecycle_state_path,
        &lifecycle_state_sha256,
        context.publication_packet_path.clone(),
        implementation_summary,
        context.runtime_evidence.runtime_evidence_paths.clone(),
        next_state.blocking_issues.clone(),
    )?;
    packet
        .validate()
        .map_err(|err| Error::Validation(err.to_string()))?;
    write_publication_transition(workspace_root, &context, &next_state, &packet)?;

    writeln!(writer, "OK: prepare-publication write complete.")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "publication_packet: {}",
        context.publication_packet_path
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "runtime_evidence_run: {}",
        context.runtime_evidence.run_id
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

pub fn discover_runtime_evidence_for_approval(
    workspace_root: &Path,
    approval: &ApprovalArtifact,
) -> Result<RuntimeEvidenceBundle, Error> {
    discover_runtime_evidence(workspace_root, approval)
}

fn build_context(workspace_root: &Path, args: &Args) -> Result<PublicationContext, Error> {
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
    validate_registry_alignment(&approval, &entry)?;
    capability_matrix::validate_agent_publication_continuity(workspace_root, &entry)
        .map_err(Error::Validation)?;

    let lifecycle_state_path =
        agent_lifecycle::lifecycle_state_path(&approval.descriptor.onboarding_pack_prefix);
    let lifecycle_state = load_lifecycle_state(workspace_root, &lifecycle_state_path)
        .map_err(|err| Error::Validation(err.to_string()))?;
    validate_approval_continuity(&lifecycle_state, &lifecycle_state_path, &approval)?;

    let runtime_evidence = discover_runtime_evidence_for_approval(workspace_root, &approval)?;
    let publication_packet_path = publication_ready_path_for_entry(&entry);
    Ok(PublicationContext {
        approval,
        entry,
        lifecycle_state_path,
        lifecycle_state,
        publication_packet_path,
        runtime_evidence,
    })
}

fn validate_check_mode(workspace_root: &Path, context: &PublicationContext) -> Result<(), Error> {
    match context.lifecycle_state.lifecycle_stage {
        LifecycleStage::RuntimeIntegrated => {
            validate_stage_evidence(
                &context.lifecycle_state,
                LifecycleStage::RuntimeIntegrated,
                &context.lifecycle_state_path,
            )?;
            require_explicit_implementation_summary(
                &context.lifecycle_state,
                &context.lifecycle_state_path,
            )?;
            if workspace_root.join(&context.publication_packet_path).exists() {
                return Err(Error::Validation(format!(
                    "publication-ready packet `{}` exists while lifecycle stage is still `runtime_integrated`",
                    context.publication_packet_path
                )));
            }
            Ok(())
        }
        LifecycleStage::PublicationReady => {
            validate_stage_evidence(
                &context.lifecycle_state,
                LifecycleStage::PublicationReady,
                &context.lifecycle_state_path,
            )?;
            if context.lifecycle_state.expected_next_command
                != agent_lifecycle::PUBLICATION_READY_NEXT_COMMAND
            {
                return Err(Error::Validation(format!(
                    "`{}` has stale expected_next_command `{}`",
                    context.lifecycle_state_path,
                    context.lifecycle_state.expected_next_command
                )));
            }
            let packet =
                load_publication_ready_packet(workspace_root, &context.publication_packet_path)
                    .map_err(|err| Error::Validation(err.to_string()))?;
            let lifecycle_sha = file_sha256(workspace_root, &context.lifecycle_state_path)
                .map_err(|err| Error::Validation(err.to_string()))?;
            let expected_packet = build_publication_ready_packet(
                &context.approval,
                &context.entry,
                &context.lifecycle_state_path,
                &lifecycle_sha,
                context.publication_packet_path.clone(),
                require_explicit_implementation_summary(
                    &context.lifecycle_state,
                    &context.lifecycle_state_path,
                )?
                .clone(),
                context.runtime_evidence.runtime_evidence_paths.clone(),
                context.lifecycle_state.blocking_issues.clone(),
            )?;
            if packet != expected_packet {
                return Err(Error::Validation(format!(
                    "publication-ready packet `{}` is stale relative to lifecycle state and runtime evidence",
                    context.publication_packet_path
                )));
            }
            Ok(())
        }
        other => Err(Error::Validation(format!(
            "prepare-publication requires lifecycle stage `runtime_integrated` or `publication_ready` at `{}` (found `{}`)",
            context.lifecycle_state_path,
            other.as_str()
        ))),
    }
}

pub fn build_publication_ready_packet(
    approval: &ApprovalArtifact,
    entry: &AgentRegistryEntry,
    lifecycle_state_path: &str,
    lifecycle_state_sha256: &str,
    publication_packet_path: String,
    implementation_summary: crate::agent_lifecycle::ImplementationSummary,
    runtime_evidence_paths: Vec<String>,
    blocking_issues: Vec<String>,
) -> Result<PublicationReadyPacket, Error> {
    Ok(PublicationReadyPacket {
        schema_version: LIFECYCLE_SCHEMA_VERSION.to_string(),
        agent_id: approval.descriptor.agent_id.clone(),
        approval_artifact_path: approval.relative_path.clone(),
        approval_artifact_sha256: approval.sha256.clone(),
        lifecycle_state_path: lifecycle_state_path.to_string(),
        lifecycle_state_sha256: lifecycle_state_sha256.to_string(),
        lifecycle_stage: LifecycleStage::PublicationReady,
        support_tier_at_emit: SupportTier::BaselineRuntime,
        manifest_root: approval.descriptor.manifest_root.clone(),
        expected_targets: approval.descriptor.canonical_targets.clone(),
        capability_publication_enabled: approval.descriptor.capability_matrix_enabled,
        support_publication_enabled: approval.descriptor.support_matrix_enabled,
        capability_matrix_target: approval.descriptor.capability_matrix_target.clone(),
        required_commands: required_publication_commands(),
        required_publication_outputs: required_publication_outputs(entry),
        runtime_evidence_paths,
        publication_owned_paths: vec![lifecycle_state_path.to_string(), publication_packet_path],
        blocking_issues,
        implementation_summary,
    })
}

fn write_publication_transition(
    workspace_root: &Path,
    context: &PublicationContext,
    next_state: &LifecycleState,
    packet: &PublicationReadyPacket,
) -> Result<(), Error> {
    let packet_path = resolve_repo_relative_path(workspace_root, &context.publication_packet_path)?;
    if packet_path.exists() {
        return Err(Error::Validation(format!(
            "publication-ready packet `{}` already exists; rerun with --check or clear the stale packet explicitly",
            context.publication_packet_path
        )));
    }

    let lifecycle_path = resolve_repo_relative_path(workspace_root, &context.lifecycle_state_path)?;
    let prior_lifecycle_bytes = fs::read(&lifecycle_path)
        .map_err(|err| Error::Internal(format!("read {}: {err}", lifecycle_path.display())))?;
    let packet_bytes = serialize_json_pretty(packet)?;
    let temp_packet_path = packet_path.with_extension("json.tmp");
    write_bytes(&temp_packet_path, &packet_bytes)?;

    if let Err(err) =
        write_lifecycle_state(workspace_root, &context.lifecycle_state_path, next_state)
    {
        let _ = fs::remove_file(&temp_packet_path);
        return Err(Error::Internal(format!("write lifecycle state: {err}")));
    }

    if let Err(err) = fs::rename(&temp_packet_path, &packet_path) {
        let _ = fs::write(&lifecycle_path, prior_lifecycle_bytes);
        let _ = fs::remove_file(&temp_packet_path);
        return Err(Error::Internal(format!(
            "finalize {}: {err}",
            packet_path.display()
        )));
    }

    Ok(())
}

fn validate_registry_alignment(
    approval: &ApprovalArtifact,
    registry_entry: &AgentRegistryEntry,
) -> Result<(), Error> {
    let descriptor = &approval.descriptor;
    let publication_packet_path = publication_ready_path_for_entry(registry_entry);
    let approval_packet_path =
        agent_lifecycle::approval_artifact_path(&registry_entry.scaffold.onboarding_pack_prefix);
    let mismatches = [
        (
            "crate_path",
            descriptor.crate_path.as_str(),
            registry_entry.crate_path.as_str(),
        ),
        (
            "backend_module",
            descriptor.backend_module.as_str(),
            registry_entry.backend_module.as_str(),
        ),
        (
            "manifest_root",
            descriptor.manifest_root.as_str(),
            registry_entry.manifest_root.as_str(),
        ),
        (
            "package_name",
            descriptor.package_name.as_str(),
            registry_entry.package_name.as_str(),
        ),
        (
            "wrapper_coverage_source_path",
            descriptor.wrapper_coverage_source_path.as_str(),
            registry_entry.wrapper_coverage.source_path.as_str(),
        ),
        (
            "approved_agent_path",
            approval.relative_path.as_str(),
            approval_packet_path.as_str(),
        ),
        (
            "publication_ready_path",
            publication_ready_path_for_entry(registry_entry).as_str(),
            publication_packet_path.as_str(),
        ),
    ]
    .into_iter()
    .filter(|(_, expected, actual)| expected != actual)
    .map(|(field, expected, actual)| format!("{field}: approval=`{expected}` registry=`{actual}`"))
    .collect::<Vec<_>>();

    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(Error::Validation(format!(
            "approval/registry mismatch: {}",
            mismatches.join("; ")
        )))
    }
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
        Ok(())
    } else {
        Err(Error::Validation(format!(
            "`{lifecycle_state_path}` is missing required evidence for `{}`: {}",
            expected_stage.as_str(),
            missing.join(", ")
        )))
    }
}

fn require_explicit_implementation_summary<'a>(
    lifecycle_state: &'a LifecycleState,
    lifecycle_state_path: &str,
) -> Result<&'a crate::agent_lifecycle::ImplementationSummary, Error> {
    let summary = lifecycle_state
        .implementation_summary
        .as_ref()
        .ok_or_else(|| {
            Error::Validation(format!(
            "`{lifecycle_state_path}` must include an implementation_summary before publication"
        ))
        })?;
    if summary.landed_surfaces.is_empty() {
        return Err(Error::Validation(format!(
            "`{lifecycle_state_path}` implementation_summary.landed_surfaces must not be empty"
        )));
    }
    Ok(summary)
}

fn required_publication_commands() -> Vec<String> {
    agent_lifecycle::REQUIRED_PUBLICATION_COMMANDS
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

fn required_publication_outputs(entry: &AgentRegistryEntry) -> Vec<String> {
    let mut outputs = Vec::new();
    if entry.publication.support_matrix_enabled {
        outputs.push(support_matrix::JSON_OUTPUT_PATH.to_string());
        outputs.push(support_matrix::MARKDOWN_OUTPUT_PATH.to_string());
    }
    if entry.publication.capability_matrix_enabled {
        outputs.push(capability_matrix::DEFAULT_OUT_PATH.to_string());
    }
    outputs
}

fn validate_required_commands(field: &str, commands: &[String]) -> Result<(), Error> {
    let expected = required_publication_commands();
    if commands == expected {
        Ok(())
    } else {
        Err(Error::Validation(format!(
            "{field} must match the frozen publication command set exactly"
        )))
    }
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

fn resolve_repo_relative_path(
    workspace_root: &Path,
    relative_path: &str,
) -> Result<PathBuf, Error> {
    if relative_path.trim().is_empty() {
        return Err(Error::Validation("path must not be empty".to_string()));
    }
    let path = Path::new(relative_path);
    if path.is_absolute() {
        return Err(Error::Validation(format!(
            "path `{relative_path}` must be repo-relative"
        )));
    }
    if relative_path.contains('\\') {
        return Err(Error::Validation(format!(
            "path `{relative_path}` must use `/` separators"
        )));
    }
    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(Error::Validation(format!(
                    "path `{relative_path}` must stay inside the workspace root"
                )))
            }
        }
    }
    Ok(workspace_root.join(path))
}

fn write_header<W: Write>(
    writer: &mut W,
    context: &PublicationContext,
    write_mode: bool,
) -> Result<(), Error> {
    writeln!(
        writer,
        "== PREPARE-PUBLICATION {} ==",
        if write_mode { "WRITE" } else { "CHECK" }
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "approval: {}", context.approval.relative_path)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "agent_id: {}", context.approval.descriptor.agent_id)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "runtime_evidence_run: {}",
        context.runtime_evidence.run_id
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

fn map_approval_error(err: ApprovalArtifactError) -> Error {
    match err {
        ApprovalArtifactError::Validation(message) => Error::Validation(message),
        ApprovalArtifactError::Internal(message) => Error::Internal(message),
    }
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, Error> {
    let bytes = fs::read(path)
        .map_err(|err| Error::Validation(format!("read {}: {err}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|err| Error::Validation(format!("parse {}: {err}", path.display())))
}

fn serialize_json_pretty<T: Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|err| Error::Internal(format!("serialize json: {err}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn sha256_json<T: Serialize>(value: &T) -> Result<String, Error> {
    let bytes = serialize_json_pretty(value)?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| Error::Internal(format!("create {}: {err}", parent.display())))?;
    }
    fs::write(path, bytes)
        .map_err(|err| Error::Internal(format!("write {}: {err}", path.display())))
}
