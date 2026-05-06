use std::{
    fmt, fs,
    io::{stdout, Write},
    path::Path,
};

use clap::{ArgGroup, Parser, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::{agent_registry::AgentRegistry, approval_artifact::load_approval_artifact};

mod codex_exec;
mod io;
mod lifecycle;
mod models;
mod render;

use self::{
    codex_exec::{
        allowed_write_paths, approval_descriptor_view, docs_to_read, execute_codex_write,
        is_allowed_write_path, is_publication_owned_manifest_path, map_approval_error,
        required_agent_api_test, resolve_workspace_root,
    },
    io::{
        diff_snapshots, generate_run_id, load_json, now_rfc3339, read_string, snapshot_workspace,
        write_json, write_string,
    },
    lifecycle::{
        load_enrolled_lifecycle_state, persist_failed_runtime_integration,
        persist_successful_runtime_integration, required_publication_commands, validate_handoff,
        validate_registry_alignment,
    },
    models::{
        CodexExecutionEvidence, HandoffContract, InputContract, RuntimeContext, ValidationCheck,
        ValidationReport,
    },
    render::{
        render_dry_run_summary, render_prompt, render_run_status, render_run_summary, write_header,
    },
};

const RUNTIME_RUNS_ROOT: &str = "docs/agents/.uaa-temp/runtime-follow-on/runs";
const SKILL_PATH: &str = ".codex/skills/runtime-follow-on/SKILL.md";
const HANDOFF_FILE_NAME: &str = "handoff.json";
const INPUT_CONTRACT_FILE_NAME: &str = "input-contract.json";
const CODEX_EXECUTION_FILE_NAME: &str = "codex-execution.json";
const CODEX_STDOUT_FILE_NAME: &str = "codex-stdout.log";
const CODEX_STDERR_FILE_NAME: &str = "codex-stderr.log";
const PROMPT_FILE_NAME: &str = "codex-prompt.md";
const RUN_STATUS_FILE_NAME: &str = "run-status.json";
const RUN_SUMMARY_FILE_NAME: &str = "run-summary.md";
const VALIDATION_REPORT_FILE_NAME: &str = "validation-report.json";
const WRITTEN_PATHS_FILE_NAME: &str = "written-paths.json";
const WORKFLOW_VERSION: &str = "runtime_follow_on_v1";
const CODEX_BINARY_ENV: &str = "XTASK_RUNTIME_FOLLOW_ON_CODEX_BINARY";
const WRAPPER_COVERAGE_MANIFEST_PATH: &str = "src/wrapper_coverage_manifest.rs";
const PROMPT_TEMPLATE: &str = include_str!("../templates/runtime_follow_on_codex_prompt.md");

#[derive(Debug, Parser, Clone)]
#[command(group(
    ArgGroup::new("mode")
        .required(true)
        .args(["dry_run", "write"])
        .multiple(false)
))]
pub struct Args {
    /// Repo-relative approved onboarding artifact under docs/agents/lifecycle/**/governance/approved-agent.toml.
    #[arg(long)]
    pub approval: String,

    /// Materialize the packet and prompt without validating runtime outputs.
    #[arg(long)]
    pub dry_run: bool,

    /// Validate the runtime lane outputs and handoff contract using a prepared run id.
    #[arg(long)]
    pub write: bool,

    /// Requested implementation tier.
    #[arg(long, value_enum, default_value_t = RequestedTier::Default)]
    pub requested_tier: RequestedTier,

    /// Required when requested tier is `minimal`.
    #[arg(long)]
    pub minimal_justification_file: Option<String>,

    /// Explicit richer surfaces approved for this run.
    #[arg(long)]
    pub allow_rich_surface: Vec<String>,

    /// Stable run identifier. Required for `--write`; optional for `--dry-run`.
    #[arg(long)]
    pub run_id: Option<String>,

    /// Explicit `codex` binary path. Falls back to XTASK_RUNTIME_FOLLOW_ON_CODEX_BINARY, then `codex`.
    #[arg(long)]
    pub codex_binary: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum RequestedTier {
    Default,
    Minimal,
    FeatureRich,
}

impl RequestedTier {
    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Minimal => "minimal",
            Self::FeatureRich => "feature-rich",
        }
    }
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
            "runtime-follow-on must run with cwd = repo root `{}` (got `{}`)",
            workspace_root.display(),
            current_dir.display()
        )));
    }
    let mut stdout = stdout();
    run_in_workspace(&workspace_root, args, &mut stdout)
}

pub fn run_in_workspace<W: Write>(
    workspace_root: &Path,
    args: Args,
    writer: &mut W,
) -> Result<(), Error> {
    validate_args(&args, workspace_root)?;
    let context = build_context(workspace_root, &args)?;
    write_header(writer, &context, args.write)?;

    if args.dry_run {
        persist_dry_run_artifacts(&context)?;
        writeln!(writer, "OK: runtime-follow-on dry-run packet prepared.")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        writeln!(writer, "run_id: {}", context.run_id)
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        writeln!(writer, "run_dir: {}", context.run_dir.display())
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        return Ok(());
    }

    let prior_contract =
        load_json::<InputContract>(&context.run_dir.join(INPUT_CONTRACT_FILE_NAME))?;
    let prompt = read_string(&context.run_dir.join(PROMPT_FILE_NAME))?;
    let codex_execution = execute_codex_write(workspace_root, &context, &args, &prompt)?;
    write_json(
        &context.run_dir.join(CODEX_EXECUTION_FILE_NAME),
        &codex_execution,
    )?;
    let (report, written_paths) =
        validate_write_mode(workspace_root, &context, &prior_contract, &codex_execution)?;
    let passed = report.status == "pass";
    if passed {
        persist_successful_runtime_integration(
            workspace_root,
            &context,
            &prior_contract,
            &written_paths,
        )?;
    } else {
        persist_failed_runtime_integration(workspace_root, &context, &report)?;
    }
    write_json(&context.run_dir.join(VALIDATION_REPORT_FILE_NAME), &report)?;
    let status = render_run_status(
        &context,
        "write",
        if passed {
            "write_validated"
        } else {
            "write_failed"
        },
        passed,
        passed,
        written_paths.clone(),
        report.errors.clone(),
    );
    write_json(&context.run_dir.join(RUN_STATUS_FILE_NAME), &status)?;
    write_json(
        &context.run_dir.join(WRITTEN_PATHS_FILE_NAME),
        &written_paths,
    )?;
    write_string(
        &context.run_dir.join(RUN_SUMMARY_FILE_NAME),
        &render_run_summary(&context, &report, &written_paths, Some(&codex_execution)),
    )?;

    if !passed {
        return Err(Error::Validation(report.errors.join("\n")));
    }

    writeln!(writer, "OK: runtime-follow-on write validation complete.")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "run_id: {}", context.run_id)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "validated_paths: {}", written_paths.len())
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

fn validate_args(args: &Args, workspace_root: &Path) -> Result<(), Error> {
    if args.write && args.run_id.is_none() {
        return Err(Error::Validation(
            "--run-id is required with --write so the command can validate against a prepared dry-run baseline".to_string(),
        ));
    }
    if matches!(args.requested_tier, RequestedTier::Minimal)
        && args.minimal_justification_file.is_none()
    {
        return Err(Error::Validation(
            "--minimal-justification-file is required when --requested-tier minimal".to_string(),
        ));
    }
    if let Some(path) = &args.minimal_justification_file {
        let candidate = workspace_root.join(path);
        if !candidate.is_file() {
            return Err(Error::Validation(format!(
                "minimal justification file `{path}` does not exist"
            )));
        }
    }
    Ok(())
}

fn build_context(workspace_root: &Path, args: &Args) -> Result<RuntimeContext, Error> {
    let approval =
        load_approval_artifact(workspace_root, &args.approval).map_err(map_approval_error)?;
    let registry =
        AgentRegistry::load(workspace_root).map_err(|err| Error::Validation(err.to_string()))?;
    let registry_entry = registry
        .find(&approval.descriptor.agent_id)
        .cloned()
        .ok_or_else(|| {
            Error::Validation(format!(
                "approval/registry mismatch: `{}` is not present in {}",
                approval.descriptor.agent_id,
                crate::agent_registry::REGISTRY_RELATIVE_PATH
            ))
        })?;
    validate_registry_alignment(&approval, &registry_entry)?;
    let (lifecycle_state_path, lifecycle_state) =
        load_enrolled_lifecycle_state(workspace_root, &approval)?;

    let run_id = args.run_id.clone().unwrap_or_else(generate_run_id);
    let run_dir = workspace_root.join(RUNTIME_RUNS_ROOT).join(&run_id);
    let minimal_justification_text = args
        .minimal_justification_file
        .as_ref()
        .map(|path| {
            fs::read_to_string(workspace_root.join(path))
                .map_err(|err| Error::Internal(format!("read {path}: {err}")))
        })
        .transpose()?;
    let required_agent_api_test = required_agent_api_test(&approval.descriptor.agent_id);
    let input_contract = InputContract {
        workflow_version: WORKFLOW_VERSION.to_string(),
        generated_at: now_rfc3339()?,
        run_id: run_id.clone(),
        approval_artifact_path: approval.relative_path.clone(),
        approval_artifact_sha256: approval.sha256.clone(),
        agent_id: approval.descriptor.agent_id.clone(),
        display_name: approval.descriptor.display_name.clone(),
        crate_path: approval.descriptor.crate_path.clone(),
        backend_module: approval.descriptor.backend_module.clone(),
        manifest_root: approval.descriptor.manifest_root.clone(),
        wrapper_coverage_source_path: approval.descriptor.wrapper_coverage_source_path.clone(),
        canonical_targets: approval.descriptor.canonical_targets.clone(),
        always_on_capabilities: approval.descriptor.always_on_capabilities.clone(),
        target_gated_capabilities: approval.descriptor.target_gated_capabilities.clone(),
        config_gated_capabilities: approval.descriptor.config_gated_capabilities.clone(),
        backend_extensions: approval.descriptor.backend_extensions.clone(),
        support_matrix_enabled: approval.descriptor.support_matrix_enabled,
        capability_matrix_enabled: approval.descriptor.capability_matrix_enabled,
        capability_matrix_target: approval.descriptor.capability_matrix_target.clone(),
        requested_tier: args.requested_tier.as_str().to_string(),
        minimal_justification_file: args.minimal_justification_file.clone(),
        minimal_justification_text,
        allow_rich_surface: args.allow_rich_surface.clone(),
        required_agent_api_test,
        required_handoff_commands: required_publication_commands(),
        docs_to_read: docs_to_read(&approval.descriptor.agent_id),
        allowed_write_paths: allowed_write_paths(&approval.descriptor),
        ignored_diff_roots: vec![RUNTIME_RUNS_ROOT.to_string()],
        baseline: snapshot_workspace(workspace_root, &[Path::new(RUNTIME_RUNS_ROOT)])?,
    };

    Ok(RuntimeContext {
        approval,
        input_contract,
        lifecycle_state,
        lifecycle_state_path,
        run_id,
        run_dir,
    })
}

fn persist_dry_run_artifacts(context: &RuntimeContext) -> Result<(), Error> {
    fs::create_dir_all(&context.run_dir)
        .map_err(|err| Error::Internal(format!("create {}: {err}", context.run_dir.display())))?;
    write_json(
        &context.run_dir.join(INPUT_CONTRACT_FILE_NAME),
        &context.input_contract,
    )?;
    write_string(
        &context.run_dir.join(PROMPT_FILE_NAME),
        &render_prompt(context),
    )?;
    write_json(
        &context.run_dir.join(RUN_STATUS_FILE_NAME),
        &render_run_status(
            context,
            "dry_run",
            "dry_run_ready",
            true,
            false,
            Vec::new(),
            Vec::new(),
        ),
    )?;
    write_string(
        &context.run_dir.join(RUN_SUMMARY_FILE_NAME),
        &render_dry_run_summary(context),
    )?;
    write_json(
        &context.run_dir.join(VALIDATION_REPORT_FILE_NAME),
        &ValidationReport {
            workflow_version: WORKFLOW_VERSION.to_string(),
            generated_at: now_rfc3339()?,
            run_id: context.run_id.clone(),
            status: "prepared".to_string(),
            checks: vec![
                ValidationCheck {
                    name: "approval_registry_alignment".to_string(),
                    ok: true,
                    message: "approval artifact and registry entry are aligned".to_string(),
                },
                ValidationCheck {
                    name: "prompt_packet_prepared".to_string(),
                    ok: true,
                    message: "dry-run wrote the frozen prompt and input contract".to_string(),
                },
            ],
            errors: Vec::new(),
        },
    )?;
    write_json(
        &context.run_dir.join(WRITTEN_PATHS_FILE_NAME),
        &Vec::<String>::new(),
    )?;
    write_json(
        &context.run_dir.join(HANDOFF_FILE_NAME),
        &HandoffContract {
            agent_id: context.approval.descriptor.agent_id.clone(),
            manifest_root: context.approval.descriptor.manifest_root.clone(),
            runtime_lane_complete: false,
            publication_refresh_required: true,
            required_commands: required_publication_commands(),
            blockers: vec!["Pending runtime follow-on implementation.".to_string()],
        },
    )?;
    Ok(())
}

fn validate_write_mode(
    workspace_root: &Path,
    context: &RuntimeContext,
    prior_contract: &InputContract,
    codex_execution: &CodexExecutionEvidence,
) -> Result<(ValidationReport, Vec<String>), Error> {
    let current_snapshot = snapshot_workspace(workspace_root, &[Path::new(RUNTIME_RUNS_ROOT)])?;
    let changed_paths = diff_snapshots(&prior_contract.baseline, &current_snapshot);
    let mut checks = Vec::new();
    let mut errors = Vec::new();

    if codex_execution.exit_code == 0 {
        checks.push(ValidationCheck {
            name: "codex_exec".to_string(),
            ok: true,
            message: format!("codex exec completed via `{}`", codex_execution.binary),
        });
    } else {
        errors.push(format!(
            "codex exec failed with exit code {} (see {})",
            codex_execution.exit_code, codex_execution.stderr_path
        ));
        checks.push(ValidationCheck {
            name: "codex_exec".to_string(),
            ok: false,
            message: format!(
                "codex exec exited non-zero; stderr captured at {}",
                codex_execution.stderr_path
            ),
        });
    }

    let boundary_violations = changed_paths
        .iter()
        .filter(|path| !is_allowed_write_path(path, &context.approval.descriptor))
        .cloned()
        .collect::<Vec<_>>();
    if boundary_violations.is_empty() {
        checks.push(ValidationCheck {
            name: "write_boundary".to_string(),
            ok: true,
            message: "all changed paths stay inside the runtime-owned boundary".to_string(),
        });
    } else {
        errors.push(format!(
            "write boundary violation: {}",
            boundary_violations.join(", ")
        ));
        checks.push(ValidationCheck {
            name: "write_boundary".to_string(),
            ok: false,
            message: format!(
                "out-of-bounds paths detected: {}",
                boundary_violations.join(", ")
            ),
        });
    }

    let manifest_violations = changed_paths
        .iter()
        .filter(|path| {
            is_publication_owned_manifest_path(path, &context.approval.descriptor.manifest_root)
        })
        .cloned()
        .collect::<Vec<_>>();
    if manifest_violations.is_empty() {
        checks.push(ValidationCheck {
            name: "manifest_split".to_string(),
            ok: true,
            message: "manifest writes stay inside runtime-owned evidence roots".to_string(),
        });
    } else {
        errors.push(format!(
            "publication-owned manifest writes are forbidden: {}",
            manifest_violations.join(", ")
        ));
        checks.push(ValidationCheck {
            name: "manifest_split".to_string(),
            ok: false,
            message: format!(
                "publication-owned manifest writes detected: {}",
                manifest_violations.join(", ")
            ),
        });
    }

    let written_paths = changed_paths
        .iter()
        .filter(|path| is_allowed_write_path(path, &approval_descriptor_view(prior_contract)))
        .cloned()
        .collect::<Vec<_>>();
    if written_paths.is_empty() {
        errors.push(
            "runtime-follow-on write produced no runtime-owned output changes from the prepared baseline"
                .to_string(),
        );
        checks.push(ValidationCheck {
            name: "runtime_owned_writes".to_string(),
            ok: false,
            message: "no runtime-owned output changes were detected after codex exec".to_string(),
        });
    } else {
        checks.push(ValidationCheck {
            name: "runtime_owned_writes".to_string(),
            ok: true,
            message: format!(
                "detected {} runtime-owned output change(s)",
                written_paths.len()
            ),
        });
    }

    let generated_wrapper_coverage_path = format!(
        "{}/wrapper_coverage.json",
        context.approval.descriptor.manifest_root
    );
    if changed_paths
        .iter()
        .any(|path| path == &generated_wrapper_coverage_path)
    {
        errors.push(format!(
            "generated wrapper coverage edits are forbidden: {generated_wrapper_coverage_path}"
        ));
        checks.push(ValidationCheck {
            name: "wrapper_coverage_generated_json".to_string(),
            ok: false,
            message: format!(
                "generated wrapper coverage output `{generated_wrapper_coverage_path}` was edited"
            ),
        });
    } else {
        checks.push(ValidationCheck {
            name: "wrapper_coverage_generated_json".to_string(),
            ok: true,
            message: "no generated wrapper_coverage.json edit was used".to_string(),
        });
    }

    let required_test = workspace_root.join(&prior_contract.required_agent_api_test);
    let requires_default_test = matches!(prior_contract.requested_tier.as_str(), "default");
    if !requires_default_test || required_test.is_file() {
        checks.push(ValidationCheck {
            name: "required_agent_api_test".to_string(),
            ok: true,
            message: "required default-tier agent_api onboarding test is present".to_string(),
        });
    } else {
        errors.push(format!(
            "default-tier run requires `{}`",
            prior_contract.required_agent_api_test
        ));
        checks.push(ValidationCheck {
            name: "required_agent_api_test".to_string(),
            ok: false,
            message: format!(
                "missing required test `{}`",
                prior_contract.required_agent_api_test
            ),
        });
    }

    let handoff_path = context.run_dir.join(HANDOFF_FILE_NAME);
    match validate_handoff(&handoff_path, context) {
        Ok(()) => checks.push(ValidationCheck {
            name: "handoff_contract".to_string(),
            ok: true,
            message: "handoff.json passed schema and semantic validation".to_string(),
        }),
        Err(message) => {
            errors.push(message.clone());
            checks.push(ValidationCheck {
                name: "handoff_contract".to_string(),
                ok: false,
                message,
            });
        }
    }

    Ok((
        ValidationReport {
            workflow_version: WORKFLOW_VERSION.to_string(),
            generated_at: now_rfc3339()?,
            run_id: context.run_id.clone(),
            status: if errors.is_empty() {
                "pass".to_string()
            } else {
                "fail".to_string()
            },
            checks,
            errors,
        },
        written_paths,
    ))
}
