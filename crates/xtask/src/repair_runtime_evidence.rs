use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use clap::{ArgGroup, Parser};

use crate::{
    agent_lifecycle::{self, load_lifecycle_state, LifecycleStage, LifecycleState},
    agent_registry::{AgentRegistry, AgentRegistryEntry},
    approval_artifact::{load_approval_artifact, ApprovalArtifact, ApprovalArtifactError},
    runtime_evidence_bundle::{
        self, check_repairable_runtime_evidence, repair_run_id, write_runtime_evidence_bundle,
        RuntimeEvidenceBundleSpec,
    },
};

const HOST_SURFACE: &str = "xtask repair-runtime-evidence";
const LOADED_SKILL_REF: &str = "repair-runtime-evidence";

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

    /// Check whether truthful runtime evidence can be reconstructed from committed runtime-owned outputs.
    #[arg(long)]
    pub check: bool,

    /// Write a repaired runtime evidence bundle without advancing lifecycle stage.
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
struct RepairContext {
    approval: ApprovalArtifact,
    lifecycle_state_path: String,
    lifecycle_state: LifecycleState,
}

pub fn run(args: Args) -> Result<(), Error> {
    let workspace_root = resolve_workspace_root()?;
    let current_dir = std::env::current_dir()
        .map_err(|err| Error::Internal(format!("resolve current directory: {err}")))?;
    if current_dir != workspace_root {
        return Err(Error::Validation(format!(
            "repair-runtime-evidence must run with cwd = repo root `{}` (got `{}`)",
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
    let run_id = repair_run_id(&context.approval.descriptor.agent_id);
    write_header(writer, &context, &run_id, args.write)?;

    if args.check {
        let written_paths = check_repairable_runtime_evidence(
            workspace_root,
            &context.approval,
            &context.lifecycle_state,
        )
        .map_err(map_bundle_error)?;
        writeln!(writer, "OK: repair-runtime-evidence check passed.")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        writeln!(writer, "run_id: {run_id}")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        writeln!(writer, "written_paths: {}", written_paths.len())
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        return Ok(());
    }

    let bundle = write_runtime_evidence_bundle(
        workspace_root,
        &context.approval,
        &context.lifecycle_state,
        &RuntimeEvidenceBundleSpec {
            run_id: &run_id,
            host_surface: HOST_SURFACE,
            loaded_skill_ref: LOADED_SKILL_REF,
            mode: "repair",
            source_label: "repair-runtime-evidence",
            summary_title: "Runtime Evidence Repair",
            validation_check_name: "runtime_evidence_repair",
            validation_message:
                "runtime evidence was reconstructed from committed runtime-owned outputs",
        },
    )
    .map_err(map_bundle_error)?;
    let reloaded_lifecycle_state =
        load_lifecycle_state(workspace_root, &context.lifecycle_state_path)
            .map_err(|err| Error::Internal(format!("reload lifecycle state: {err}")))?;
    if reloaded_lifecycle_state.lifecycle_stage != context.lifecycle_state.lifecycle_stage {
        return Err(Error::Internal(format!(
            "`{}` changed lifecycle stage during repair",
            context.lifecycle_state_path
        )));
    }

    writeln!(writer, "OK: repair-runtime-evidence write complete.")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "run_id: {}", bundle.run_id)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "run_dir: {}", bundle.run_relative)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "written_paths: {}", bundle.written_paths.len())
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

fn build_context(workspace_root: &Path, args: &Args) -> Result<RepairContext, Error> {
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

    let lifecycle_state_path =
        agent_lifecycle::lifecycle_state_path(&approval.descriptor.onboarding_pack_prefix);
    let lifecycle_state = load_lifecycle_state(workspace_root, &lifecycle_state_path)
        .map_err(|err| Error::Validation(err.to_string()))?;
    validate_approval_continuity(&lifecycle_state, &lifecycle_state_path, &approval)?;
    if lifecycle_state.lifecycle_stage != LifecycleStage::RuntimeIntegrated {
        return Err(Error::Validation(format!(
            "repair-runtime-evidence requires lifecycle stage `runtime_integrated` at `{}` (found `{}`)",
            lifecycle_state_path,
            lifecycle_state.lifecycle_stage.as_str()
        )));
    }

    Ok(RepairContext {
        approval,
        lifecycle_state_path,
        lifecycle_state,
    })
}

fn validate_registry_alignment(
    approval: &ApprovalArtifact,
    registry_entry: &AgentRegistryEntry,
) -> Result<(), Error> {
    let descriptor = &approval.descriptor;
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

fn write_header<W: Write>(
    writer: &mut W,
    context: &RepairContext,
    run_id: &str,
    write_mode: bool,
) -> Result<(), Error> {
    writeln!(
        writer,
        "== REPAIR-RUNTIME-EVIDENCE {} ==",
        if write_mode { "WRITE" } else { "CHECK" }
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "approval: {}", context.approval.relative_path)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "agent_id: {}", context.approval.descriptor.agent_id)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "run_id: {run_id}")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

fn map_approval_error(err: ApprovalArtifactError) -> Error {
    match err {
        ApprovalArtifactError::Validation(message) => Error::Validation(message),
        ApprovalArtifactError::Internal(message) => Error::Internal(message),
    }
}

fn map_bundle_error(err: runtime_evidence_bundle::Error) -> Error {
    match err {
        runtime_evidence_bundle::Error::Validation(message) => Error::Validation(message),
        runtime_evidence_bundle::Error::Internal(message) => Error::Internal(message),
    }
}
