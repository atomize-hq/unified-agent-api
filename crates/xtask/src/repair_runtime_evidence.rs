use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use clap::{ArgGroup, Parser};

use crate::{
    agent_lifecycle::{self, load_lifecycle_state, LifecycleStage, LifecycleState},
    agent_registry::{AgentRegistry, AgentRegistryEntry},
    approval_artifact::{load_approval_artifact, ApprovalArtifact, ApprovalArtifactError},
    prepare_publication::validate_runtime_evidence_run_for_approval,
    runtime_evidence_bundle::{
        self, check_repairable_runtime_evidence, repair_run_id, write_runtime_evidence_bundle_at,
        RuntimeEvidenceBundleSpec, RUNTIME_RUNS_ROOT,
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

    let canonical_run_relative = format!("{RUNTIME_RUNS_ROOT}/{run_id}");
    let canonical_run_root = workspace_root.join(&canonical_run_relative);
    let suffix = unique_suffix()?;
    let temp_run_root = workspace_root.join(format!("{RUNTIME_RUNS_ROOT}/.tmp-{run_id}-{suffix}"));
    let backup_run_root =
        workspace_root.join(format!("{RUNTIME_RUNS_ROOT}/.bak-{run_id}-{suffix}"));
    remove_dir_if_exists(&temp_run_root)?;
    remove_dir_if_exists(&backup_run_root)?;

    let bundle = write_runtime_evidence_bundle_at(
        workspace_root,
        &temp_run_root,
        &canonical_run_relative,
        &canonical_run_root,
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
    promote_repair_bundle(
        workspace_root,
        &context.approval,
        &run_id,
        &canonical_run_root,
        &temp_run_root,
        &backup_run_root,
    )?;
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

fn promote_repair_bundle(
    workspace_root: &Path,
    approval: &ApprovalArtifact,
    run_id: &str,
    canonical_run_root: &Path,
    temp_run_root: &Path,
    backup_run_root: &Path,
) -> Result<(), Error> {
    let had_existing_canonical = canonical_run_root.exists();
    if had_existing_canonical {
        fs::rename(canonical_run_root, backup_run_root).map_err(|err| {
            Error::Internal(format!(
                "stage existing repair bundle {} -> {}: {err}",
                canonical_run_root.display(),
                backup_run_root.display()
            ))
        })?;
    }

    if let Err(err) = fs::rename(temp_run_root, canonical_run_root) {
        remove_dir_if_exists(temp_run_root)?;
        restore_backup(canonical_run_root, backup_run_root, had_existing_canonical)?;
        return Err(Error::Internal(format!(
            "promote staged repair bundle {} -> {}: {err}",
            temp_run_root.display(),
            canonical_run_root.display()
        )));
    }

    if let Err(err) = validate_runtime_evidence_run_for_approval(workspace_root, approval, run_id) {
        remove_dir_if_exists(canonical_run_root)?;
        restore_backup(canonical_run_root, backup_run_root, had_existing_canonical)?;
        return Err(Error::Validation(err.to_string()));
    }

    remove_dir_if_exists(backup_run_root)?;
    Ok(())
}

fn restore_backup(
    canonical_run_root: &Path,
    backup_run_root: &Path,
    had_existing_canonical: bool,
) -> Result<(), Error> {
    if !had_existing_canonical {
        return Ok(());
    }
    if canonical_run_root.exists() {
        remove_dir_if_exists(canonical_run_root)?;
    }
    if backup_run_root.exists() {
        fs::rename(backup_run_root, canonical_run_root).map_err(|err| {
            Error::Internal(format!(
                "restore repair bundle backup {} -> {}: {err}",
                backup_run_root.display(),
                canonical_run_root.display()
            ))
        })?;
    }
    Ok(())
}

fn remove_dir_if_exists(path: &Path) -> Result<(), Error> {
    if path.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|err| Error::Internal(format!("remove {}: {err}", path.display())))?;
    }
    Ok(())
}

fn unique_suffix() -> Result<String, Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| Error::Internal(format!("system clock before unix epoch: {err}")))?;
    Ok(format!("{}-{}", std::process::id(), now.as_nanos()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use serde_json::json;

    use crate::approval_artifact::{ApprovalArtifact, ApprovalDescriptor};

    #[test]
    fn promote_repair_bundle_restores_backup_when_canonical_validation_fails() {
        let workspace_root = std::env::temp_dir().join(format!(
            "repair-runtime-evidence-rollback-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time after unix epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&workspace_root).expect("create workspace root");

        let approval = ApprovalArtifact {
            relative_path:
                "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml"
                    .to_string(),
            canonical_path: workspace_root
                .join("docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml"),
            sha256: "approval-sha".to_string(),
            descriptor: ApprovalDescriptor {
                agent_id: "gemini_cli".to_string(),
                display_name: "Gemini CLI".to_string(),
                crate_path: "crates/gemini_cli".to_string(),
                backend_module: "crates/agent_api/src/backends/gemini_cli".to_string(),
                manifest_root: "cli_manifests/gemini_cli".to_string(),
                package_name: "unified-agent-api-gemini-cli".to_string(),
                canonical_targets: vec!["darwin-arm64".to_string()],
                wrapper_coverage_binding_kind: "generated_from_wrapper_crate".to_string(),
                wrapper_coverage_source_path: "crates/gemini_cli".to_string(),
                always_on_capabilities: vec![],
                target_gated_capabilities: vec![],
                config_gated_capabilities: vec![],
                backend_extensions: vec![],
                support_matrix_enabled: true,
                capability_matrix_enabled: true,
                capability_matrix_target: None,
                docs_release_track: "crates-io".to_string(),
                onboarding_pack_prefix: "gemini-cli-onboarding".to_string(),
            },
        };

        let run_id = repair_run_id(&approval.descriptor.agent_id);
        let canonical_run_root = workspace_root.join(format!("{RUNTIME_RUNS_ROOT}/{run_id}"));
        let temp_run_root =
            workspace_root.join(format!("{RUNTIME_RUNS_ROOT}/.tmp-{run_id}-rollback"));
        let backup_run_root =
            workspace_root.join(format!("{RUNTIME_RUNS_ROOT}/.bak-{run_id}-rollback"));

        fs::create_dir_all(&canonical_run_root).expect("create canonical root");
        fs::write(
            canonical_run_root.join("run-summary.md"),
            "# Previous Repair Bundle\n",
        )
        .expect("write prior summary");

        fs::create_dir_all(&temp_run_root).expect("create temp root");
        let canonical_run_dir = canonical_run_root.to_string_lossy().into_owned();
        write_test_json(
            &temp_run_root.join("input-contract.json"),
            &json!({
                "approval_artifact_path": approval.relative_path,
                "approval_artifact_sha256": approval.sha256,
                "agent_id": approval.descriptor.agent_id,
                "manifest_root": approval.descriptor.manifest_root,
                "required_handoff_commands": crate::agent_lifecycle::REQUIRED_PUBLICATION_COMMANDS,
            }),
        );
        write_test_json(
            &temp_run_root.join("run-status.json"),
            &json!({
                "approval_artifact_path": approval.relative_path,
                "agent_id": approval.descriptor.agent_id,
                "status": "write_validated",
                "validation_passed": true,
                "handoff_ready": true,
                "run_dir": canonical_run_dir,
            }),
        );
        write_test_json(
            &temp_run_root.join("validation-report.json"),
            &json!({
                "status": "pass"
            }),
        );
        write_test_json(
            &temp_run_root.join("handoff.json"),
            &json!({
                "agent_id": approval.descriptor.agent_id,
                "manifest_root": approval.descriptor.manifest_root,
                "runtime_lane_complete": true,
                "publication_refresh_required": true,
                "required_commands": crate::agent_lifecycle::REQUIRED_PUBLICATION_COMMANDS,
                "blockers": []
            }),
        );
        write_test_json(&temp_run_root.join("written-paths.json"), &json!([]));
        fs::write(
            temp_run_root.join("run-summary.md"),
            "# Invalid Replacement Bundle\n",
        )
        .expect("write invalid summary");

        let err = promote_repair_bundle(
            &workspace_root,
            &approval,
            &run_id,
            &canonical_run_root,
            &temp_run_root,
            &backup_run_root,
        )
        .expect_err("invalid canonical bundle should fail validation");
        assert!(
            matches!(err, Error::Validation(message) if message.contains("did not record any written paths"))
        );

        let restored_summary = fs::read_to_string(canonical_run_root.join("run-summary.md"))
            .expect("read restored summary");
        assert!(restored_summary.contains("Previous Repair Bundle"));
        assert!(
            !temp_run_root.exists(),
            "temp staging dir should be cleaned up"
        );
        assert!(
            !backup_run_root.exists(),
            "backup dir should be restored back into canonical location"
        );

        fs::remove_dir_all(&workspace_root).expect("remove workspace root");
    }

    fn write_test_json(path: &Path, value: &serde_json::Value) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent dirs");
        }
        let mut bytes = serde_json::to_vec_pretty(value).expect("serialize json");
        bytes.push(b'\n');
        fs::write(path, bytes).expect("write json");
    }
}
