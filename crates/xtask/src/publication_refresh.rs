use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use clap::{ArgGroup, Parser};
use sha2::Digest;

#[cfg(test)]
use std::cell::RefCell;

use crate::{
    agent_lifecycle::{
        self, file_sha256, load_lifecycle_state, load_publication_ready_packet, now_rfc3339,
        required_evidence_for_stage, LifecycleStage, LifecycleState, PublicationReadyPacket,
    },
    agent_registry::{AgentRegistry, AgentRegistryEntry},
    approval_artifact::{load_approval_artifact, ApprovalArtifact, ApprovalArtifactError},
    capability_matrix,
    prepare_publication::{
        build_publication_ready_packet, validate_packet_pinned_runtime_evidence_for_approval,
    },
    support_matrix,
    workspace_mutation::{
        apply_mutations, plan_create_or_replace, PlannedMutation, WorkspaceMutationError,
        WorkspacePathJail,
    },
};

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

    /// Validate the committed publication packet and publication-owned outputs without rewriting them.
    #[arg(long)]
    pub check: bool,

    /// Refresh the packet-owned publication outputs, run the green gate, and advance the publication-ready next command.
    #[arg(long)]
    pub write: bool,
}

#[derive(Debug)]
pub enum Error {
    Validation(String),
    Internal(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationArtifactFile {
    pub relative_path: String,
    pub contents: Vec<u8>,
}

#[derive(Debug, Clone)]
struct RefreshContext {
    approval: ApprovalArtifact,
    entry: AgentRegistryEntry,
    lifecycle_state_path: String,
    lifecycle_state: LifecycleState,
    publication_packet_path: String,
    publication_packet: PublicationReadyPacket,
}

#[derive(Debug, Clone)]
struct FileSnapshot {
    relative_path: String,
    prior_contents: Option<Vec<u8>>,
}

#[cfg(test)]
type TestGateRunner = fn(&Path, &[String]) -> Result<(), String>;

#[cfg(test)]
thread_local! {
    static TEST_GATE_RUNNER: RefCell<Option<TestGateRunner>> = const { RefCell::new(None) };
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

impl From<WorkspaceMutationError> for Error {
    fn from(err: WorkspaceMutationError) -> Self {
        match err {
            WorkspaceMutationError::Validation(message) => Self::Validation(message),
            WorkspaceMutationError::Internal(message) => Self::Internal(message),
        }
    }
}

pub const SUPPORT_MATRIX_JSON_OUTPUT_PATH: &str = support_matrix::JSON_OUTPUT_PATH;
pub const SUPPORT_MATRIX_MARKDOWN_OUTPUT_PATH: &str = support_matrix::MARKDOWN_OUTPUT_PATH;
pub const CAPABILITY_MATRIX_OUTPUT_PATH: &str = capability_matrix::DEFAULT_OUT_PATH;

pub fn expected_publication_output_paths(
    support_enabled: bool,
    capability_enabled: bool,
) -> Vec<String> {
    let mut outputs = Vec::new();
    if support_enabled {
        outputs.push(SUPPORT_MATRIX_JSON_OUTPUT_PATH.to_string());
        outputs.push(SUPPORT_MATRIX_MARKDOWN_OUTPUT_PATH.to_string());
    }
    if capability_enabled {
        outputs.push(CAPABILITY_MATRIX_OUTPUT_PATH.to_string());
    }
    outputs
}

pub fn build_publication_artifact_plan(
    workspace_root: &Path,
    support_enabled: bool,
    capability_enabled: bool,
) -> Result<Vec<PublicationArtifactFile>, String> {
    let mut files = Vec::new();
    if support_enabled {
        let bundle = support_matrix::generate_publication_artifacts(workspace_root)?;
        files.push(PublicationArtifactFile {
            relative_path: SUPPORT_MATRIX_JSON_OUTPUT_PATH.to_string(),
            contents: bundle.json.into_bytes(),
        });
        files.push(PublicationArtifactFile {
            relative_path: SUPPORT_MATRIX_MARKDOWN_OUTPUT_PATH.to_string(),
            contents: bundle.markdown.into_bytes(),
        });
    }
    if capability_enabled {
        files.push(PublicationArtifactFile {
            relative_path: CAPABILITY_MATRIX_OUTPUT_PATH.to_string(),
            contents: capability_matrix::generate_markdown()?.into_bytes(),
        });
    }
    Ok(files)
}

pub fn run(args: Args) -> Result<(), Error> {
    let workspace_root = resolve_workspace_root()?;
    let current_dir = std::env::current_dir()
        .map_err(|err| Error::Internal(format!("resolve current directory: {err}")))?;
    if current_dir != workspace_root {
        return Err(Error::Validation(format!(
            "refresh-publication must run with cwd = repo root `{}` (got `{}`)",
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
    let planned_outputs = validate_publication_contract(workspace_root, &context)?;
    write_header(writer, &context, args.write)?;

    if args.check {
        let stale_paths = stale_output_paths(workspace_root, &planned_outputs)?;
        if !stale_paths.is_empty() {
            return Err(Error::Validation(format!(
                "publication-owned outputs are stale: {}",
                stale_paths.join(", ")
            )));
        }
        validate_packet_freshness(workspace_root, &context)?;
        writeln!(writer, "OK: refresh-publication check passed.")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        writeln!(
            writer,
            "publication_packet: {}",
            context.publication_packet_path
        )
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        return Ok(());
    }

    let packet_runtime_evidence = validate_packet_pinned_runtime_evidence_for_approval(
        workspace_root,
        &context.approval,
        &context.publication_packet,
    )
    .map_err(map_prepare_publication_error)?;
    let next_state = build_next_lifecycle_state(&context)?;
    let lifecycle_sha = sha256_json(&next_state)?;
    let next_packet = build_publication_ready_packet(
        &context.approval,
        &context.entry,
        &context.lifecycle_state_path,
        &lifecycle_sha,
        context.publication_packet_path.clone(),
        next_state.implementation_summary.clone().ok_or_else(|| {
            Error::Validation(format!(
                "`{}` must include an implementation_summary before publication refresh",
                context.lifecycle_state_path
            ))
        })?,
        packet_runtime_evidence.runtime_evidence_paths,
        next_state.blocking_issues.clone(),
    )
    .map_err(map_prepare_publication_error)?;
    next_packet
        .validate()
        .map_err(|err| Error::Validation(err.to_string()))?;

    let mut mutations = planned_output_mutations(workspace_root, &planned_outputs)?;
    mutations.extend(governance_mutations(
        workspace_root,
        &context.lifecycle_state_path,
        &next_state,
        &context.publication_packet_path,
        &next_packet,
    )?);
    let snapshots = capture_snapshots(workspace_root, &mutations)?;
    apply_mutations(workspace_root, &mutations)?;

    if let Err(err) = run_green_gate(
        workspace_root,
        &context.publication_packet.required_commands,
    ) {
        let rollback_result = restore_snapshots(workspace_root, &snapshots);
        return match rollback_result {
            Ok(()) => Err(err),
            Err(rollback_err) => Err(Error::Internal(format!(
                "refresh-publication rollback failed after gate error: {rollback_err}; original error: {err}"
            ))),
        };
    }

    writeln!(writer, "OK: refresh-publication write complete.")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "publication_packet: {}",
        context.publication_packet_path
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

#[cfg(test)]
#[doc(hidden)]
pub fn set_test_gate_runner(runner: Option<TestGateRunner>) {
    TEST_GATE_RUNNER.with(|slot| *slot.borrow_mut() = runner);
}

fn build_context(workspace_root: &Path, args: &Args) -> Result<RefreshContext, Error> {
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
    if lifecycle_state.lifecycle_stage != LifecycleStage::PublicationReady {
        return Err(Error::Validation(format!(
            "refresh-publication requires lifecycle stage `publication_ready` at `{}` (found `{}`)",
            lifecycle_state_path,
            lifecycle_state.lifecycle_stage.as_str()
        )));
    }
    validate_stage_evidence(
        &lifecycle_state,
        LifecycleStage::PublicationReady,
        &lifecycle_state_path,
    )?;
    validate_expected_next_command(&approval, &entry, &lifecycle_state_path, &lifecycle_state)?;

    let publication_packet_path = agent_lifecycle::publication_ready_path_for_entry(&entry);
    let publication_packet =
        load_publication_ready_packet(workspace_root, &publication_packet_path)
            .map_err(|err| Error::Validation(err.to_string()))?;
    validate_packet_identity(
        &publication_packet_path,
        &publication_packet,
        &approval,
        &entry,
        &lifecycle_state_path,
    )?;

    Ok(RefreshContext {
        approval,
        entry,
        lifecycle_state_path,
        lifecycle_state,
        publication_packet_path,
        publication_packet,
    })
}

fn validate_publication_contract(
    workspace_root: &Path,
    context: &RefreshContext,
) -> Result<Vec<PublicationArtifactFile>, Error> {
    let expected_paths = expected_publication_output_paths(
        context.publication_packet.support_publication_enabled,
        context.publication_packet.capability_publication_enabled,
    );
    if context.publication_packet.required_publication_outputs != expected_paths {
        return Err(Error::Validation(format!(
            "`{}` required_publication_outputs do not match the frozen publication planning seam",
            context.publication_packet_path
        )));
    }

    let planned_outputs = build_publication_artifact_plan(
        workspace_root,
        context.publication_packet.support_publication_enabled,
        context.publication_packet.capability_publication_enabled,
    )
    .map_err(Error::Validation)?;

    let planned_paths = planned_outputs
        .iter()
        .map(|file| file.relative_path.clone())
        .collect::<Vec<_>>();
    if planned_paths != context.publication_packet.required_publication_outputs {
        return Err(Error::Validation(format!(
            "planned publication outputs do not match `{}` required_publication_outputs",
            context.publication_packet_path
        )));
    }

    Ok(planned_outputs)
}

fn stale_output_paths(
    workspace_root: &Path,
    planned_outputs: &[PublicationArtifactFile],
) -> Result<Vec<String>, Error> {
    let mut stale = Vec::new();
    for file in planned_outputs {
        let path = workspace_root.join(&file.relative_path);
        match fs::read(&path) {
            Ok(current) if current == file.contents => {}
            Ok(_) | Err(_) => stale.push(file.relative_path.clone()),
        }
    }
    Ok(stale)
}

fn validate_packet_freshness(workspace_root: &Path, context: &RefreshContext) -> Result<(), Error> {
    let runtime_evidence = validate_packet_pinned_runtime_evidence_for_approval(
        workspace_root,
        &context.approval,
        &context.publication_packet,
    )
    .map_err(map_prepare_publication_error)?;
    let lifecycle_sha = file_sha256(workspace_root, &context.lifecycle_state_path)
        .map_err(|err| Error::Validation(err.to_string()))?;
    let expected_packet = build_publication_ready_packet(
        &context.approval,
        &context.entry,
        &context.lifecycle_state_path,
        &lifecycle_sha,
        context.publication_packet_path.clone(),
        context
            .lifecycle_state
            .implementation_summary
            .clone()
            .ok_or_else(|| {
                Error::Validation(format!(
                    "`{}` must include an implementation_summary before publication refresh",
                    context.lifecycle_state_path
                ))
            })?,
        runtime_evidence.runtime_evidence_paths,
        context.lifecycle_state.blocking_issues.clone(),
    )
    .map_err(map_prepare_publication_error)?;
    if context.publication_packet != expected_packet {
        return Err(Error::Validation(format!(
            "publication-ready packet `{}` is stale relative to lifecycle state and runtime evidence",
            context.publication_packet_path
        )));
    }
    Ok(())
}

fn build_next_lifecycle_state(context: &RefreshContext) -> Result<LifecycleState, Error> {
    let mut next_state = context.lifecycle_state.clone();
    next_state.current_owner_command = "refresh-publication --write".to_string();
    next_state.expected_next_command = agent_lifecycle::publication_ready_closeout_command(
        &context.approval.relative_path,
        &context.entry.scaffold.onboarding_pack_prefix,
    );
    next_state.last_transition_at =
        now_rfc3339().map_err(|err| Error::Internal(err.to_string()))?;
    next_state.last_transition_by = "xtask refresh-publication --write".to_string();
    next_state
        .validate()
        .map_err(|err| Error::Validation(err.to_string()))?;
    Ok(next_state)
}

fn planned_output_mutations(
    workspace_root: &Path,
    planned_outputs: &[PublicationArtifactFile],
) -> Result<Vec<PlannedMutation>, Error> {
    let jail = WorkspacePathJail::new(workspace_root)?;
    planned_outputs
        .iter()
        .map(|file| {
            plan_create_or_replace(
                &jail,
                PathBuf::from(&file.relative_path),
                file.contents.clone(),
            )
            .map_err(Into::into)
        })
        .collect()
}

fn governance_mutations(
    workspace_root: &Path,
    lifecycle_state_path: &str,
    next_state: &LifecycleState,
    publication_packet_path: &str,
    next_packet: &PublicationReadyPacket,
) -> Result<Vec<PlannedMutation>, Error> {
    let jail = WorkspacePathJail::new(workspace_root)?;
    let lifecycle_bytes = serialize_json_pretty(next_state)?;
    let packet_bytes = serialize_json_pretty(next_packet)?;
    Ok(vec![
        plan_create_or_replace(&jail, PathBuf::from(lifecycle_state_path), lifecycle_bytes)?,
        plan_create_or_replace(&jail, PathBuf::from(publication_packet_path), packet_bytes)?,
    ])
}

fn capture_snapshots(
    workspace_root: &Path,
    mutations: &[PlannedMutation],
) -> Result<Vec<FileSnapshot>, Error> {
    let jail = WorkspacePathJail::new(workspace_root)?;
    let mut snapshots = Vec::new();
    for mutation in mutations {
        let absolute_path = jail.resolve(mutation.relative_path())?;
        let prior_contents = match fs::read(&absolute_path) {
            Ok(bytes) => Some(bytes),
            Err(err) if err.kind() == io::ErrorKind::NotFound => None,
            Err(err) => {
                return Err(Error::Internal(format!(
                    "read {}: {err}",
                    absolute_path.display()
                )))
            }
        };
        snapshots.push(FileSnapshot {
            relative_path: mutation.relative_path().display().to_string(),
            prior_contents,
        });
    }
    Ok(snapshots)
}

fn restore_snapshots(workspace_root: &Path, snapshots: &[FileSnapshot]) -> Result<(), Error> {
    let jail = WorkspacePathJail::new(workspace_root)?;
    let mut restore_mutations = Vec::new();
    let mut remove_paths = Vec::new();

    for snapshot in snapshots {
        match &snapshot.prior_contents {
            Some(bytes) => restore_mutations.push(plan_create_or_replace(
                &jail,
                PathBuf::from(&snapshot.relative_path),
                bytes.clone(),
            )?),
            None => remove_paths.push(jail.resolve(Path::new(&snapshot.relative_path))?),
        }
    }

    if !restore_mutations.is_empty() {
        apply_mutations(workspace_root, &restore_mutations)?;
    }
    for path in remove_paths {
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|err| Error::Internal(format!("remove {}: {err}", path.display())))?;
        }
    }
    Ok(())
}

fn run_green_gate(workspace_root: &Path, commands: &[String]) -> Result<(), Error> {
    #[cfg(test)]
    {
        if let Some(runner) = TEST_GATE_RUNNER.with(|slot| *slot.borrow()) {
            return runner(workspace_root, commands).map_err(Error::Validation);
        }
    }

    for command in commands {
        let output = Command::new("sh")
            .arg("-lc")
            .arg(command)
            .current_dir(workspace_root)
            .output()
            .map_err(|err| Error::Internal(format!("run `{command}`: {err}")))?;
        if output.status.success() {
            continue;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::Validation(format!(
            "publication gate command `{command}` failed with exit code {}.\nstdout:\n{}\nstderr:\n{}",
            output.status.code().unwrap_or(1),
            stdout.trim_end(),
            stderr.trim_end()
        )));
    }
    Ok(())
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

fn validate_expected_next_command(
    approval: &ApprovalArtifact,
    entry: &AgentRegistryEntry,
    lifecycle_state_path: &str,
    lifecycle_state: &LifecycleState,
) -> Result<(), Error> {
    let expected_commands = agent_lifecycle::publication_ready_expected_next_commands(
        &approval.relative_path,
        &entry.scaffold.onboarding_pack_prefix,
    );
    if expected_commands
        .iter()
        .any(|expected| expected == &lifecycle_state.expected_next_command)
    {
        Ok(())
    } else {
        Err(Error::Validation(format!(
            "`{}` has stale expected_next_command `{}`; expected one of `{}` or `{}`",
            lifecycle_state_path,
            lifecycle_state.expected_next_command,
            expected_commands[0],
            expected_commands[1]
        )))
    }
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

fn write_header<W: Write>(
    writer: &mut W,
    context: &RefreshContext,
    write_mode: bool,
) -> Result<(), Error> {
    writeln!(
        writer,
        "== REFRESH-PUBLICATION {} ==",
        if write_mode { "WRITE" } else { "CHECK" }
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "approval: {}", context.approval.relative_path)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "agent_id: {}", context.approval.descriptor.agent_id)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
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

fn map_approval_error(err: ApprovalArtifactError) -> Error {
    match err {
        ApprovalArtifactError::Validation(message) => Error::Validation(message),
        ApprovalArtifactError::Internal(message) => Error::Internal(message),
    }
}

fn map_prepare_publication_error(err: crate::prepare_publication::Error) -> Error {
    match err {
        crate::prepare_publication::Error::Validation(message) => Error::Validation(message),
        crate::prepare_publication::Error::Internal(message) => Error::Internal(message),
    }
}

fn serialize_json_pretty<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|err| Error::Internal(format!("serialize json: {err}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn sha256_json<T: serde::Serialize>(value: &T) -> Result<String, Error> {
    let bytes = serialize_json_pretty(value)?;
    let digest = sha2::Sha256::digest(bytes);
    Ok(format!("{digest:x}"))
}
