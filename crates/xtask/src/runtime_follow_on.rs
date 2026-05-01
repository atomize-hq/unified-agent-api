use std::{
    collections::BTreeSet,
    fmt, fs,
    io::{stdout, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use clap::{ArgGroup, Parser, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::{
    agent_lifecycle::{
        self, load_lifecycle_state, write_lifecycle_state, DeferredSurface, EvidenceId,
        ImplementationSummary, LandedSurface, LifecycleStage, LifecycleState, RuntimeProfile,
        SideState, SupportTier, TemplateId,
    },
    agent_registry::{AgentRegistry, AgentRegistryEntry},
    approval_artifact::{load_approval_artifact, ApprovalArtifact, ApprovalArtifactError},
};

mod io;
mod models;
mod render;

use self::{
    io::{
        diff_snapshots, generate_run_id, load_json, now_rfc3339, read_string, snapshot_workspace,
        write_json, write_string,
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
const LEGACY_REQUIRED_PUBLICATION_COMMANDS: [&str; 4] = [
    "support-matrix --check",
    "capability-matrix --check",
    "capability-matrix-audit",
    "make preflight",
];
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

fn validate_handoff(path: &Path, context: &RuntimeContext) -> Result<(), String> {
    let payload = fs::read_to_string(path).map_err(|err| {
        format!(
            "missing or unreadable handoff.json at {}: {err}",
            path.display()
        )
    })?;
    let parsed: serde_json::Value = serde_json::from_str(&payload)
        .map_err(|err| format!("handoff.json is not valid json: {err}"))?;
    let object = parsed
        .as_object()
        .ok_or_else(|| "handoff.json root must be an object".to_string())?;

    for key in [
        "agent_id",
        "manifest_root",
        "runtime_lane_complete",
        "publication_refresh_required",
        "required_commands",
        "blockers",
    ] {
        if !object.contains_key(key) {
            return Err(format!("handoff.json is missing required field `{key}`"));
        }
    }

    let handoff: HandoffContract = serde_json::from_value(parsed)
        .map_err(|err| format!("handoff.json failed minimum schema validation: {err}"))?;
    if handoff.agent_id != context.approval.descriptor.agent_id {
        return Err(format!(
            "handoff.json agent_id `{}` does not match approval agent_id `{}`",
            handoff.agent_id, context.approval.descriptor.agent_id
        ));
    }
    if handoff.manifest_root != context.approval.descriptor.manifest_root {
        return Err(format!(
            "handoff.json manifest_root `{}` does not match approval manifest_root `{}`",
            handoff.manifest_root, context.approval.descriptor.manifest_root
        ));
    }
    if !handoff.runtime_lane_complete {
        return Err(
            "handoff.json runtime_lane_complete must be true for a successful write run"
                .to_string(),
        );
    }
    if !handoff.publication_refresh_required {
        return Err("handoff.json publication_refresh_required must be true".to_string());
    }
    let required = agent_lifecycle::REQUIRED_PUBLICATION_COMMANDS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let legacy_required = LEGACY_REQUIRED_PUBLICATION_COMMANDS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let actual = handoff
        .required_commands
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if !required.is_subset(&actual) && !legacy_required.is_subset(&actual) {
        return Err(format!(
            "handoff.json required_commands must include {}",
            agent_lifecycle::REQUIRED_PUBLICATION_COMMANDS.join(", ")
        ));
    }
    Ok(())
}

fn load_enrolled_lifecycle_state(
    workspace_root: &Path,
    approval: &ApprovalArtifact,
) -> Result<(String, LifecycleState), Error> {
    let lifecycle_state_path =
        agent_lifecycle::lifecycle_state_path(&approval.descriptor.onboarding_pack_prefix);
    let lifecycle_state = load_lifecycle_state(workspace_root, &lifecycle_state_path)
        .map_err(|err| Error::Validation(err.to_string()))?;
    if lifecycle_state.lifecycle_stage != LifecycleStage::Enrolled {
        return Err(Error::Validation(format!(
            "runtime-follow-on requires lifecycle stage `enrolled` at `{}` (found `{}`)",
            lifecycle_state_path,
            lifecycle_state.lifecycle_stage.as_str()
        )));
    }
    Ok((lifecycle_state_path, lifecycle_state))
}

fn persist_successful_runtime_integration(
    workspace_root: &Path,
    context: &RuntimeContext,
    prior_contract: &InputContract,
    written_paths: &[String],
) -> Result<(), Error> {
    let mut lifecycle_state = context.lifecycle_state.clone();
    lifecycle_state.lifecycle_stage = LifecycleStage::RuntimeIntegrated;
    lifecycle_state.support_tier = SupportTier::BaselineRuntime;
    lifecycle_state.current_owner_command = "runtime-follow-on --write".to_string();
    lifecycle_state.expected_next_command = format!(
        "prepare-publication --approval {} --write",
        context.approval.relative_path
    );
    lifecycle_state.last_transition_at = now_rfc3339()?;
    lifecycle_state.last_transition_by = "xtask runtime-follow-on --write".to_string();
    lifecycle_state.required_evidence =
        required_evidence_strings(LifecycleStage::RuntimeIntegrated);
    lifecycle_state.satisfied_evidence =
        required_evidence_strings(LifecycleStage::RuntimeIntegrated);
    lifecycle_state.side_states = lifecycle_state
        .side_states
        .into_iter()
        .filter(|state| !matches!(state, SideState::Blocked | SideState::FailedRetryable))
        .collect();
    lifecycle_state.blocking_issues.clear();
    lifecycle_state.retryable_failures.clear();
    lifecycle_state.implementation_summary = Some(build_implementation_summary(
        context,
        prior_contract,
        written_paths,
    ));
    lifecycle_state.publication_packet_path = None;
    lifecycle_state.publication_packet_sha256 = None;

    write_lifecycle_state(
        workspace_root,
        &context.lifecycle_state_path,
        &lifecycle_state,
    )
    .map_err(|err| Error::Internal(format!("write lifecycle state: {err}")))
}

fn persist_failed_runtime_integration(
    workspace_root: &Path,
    context: &RuntimeContext,
    report: &ValidationReport,
) -> Result<(), Error> {
    let mut lifecycle_state = context.lifecycle_state.clone();
    let blockers = best_effort_handoff_blockers(&context.run_dir.join(HANDOFF_FILE_NAME));
    let (side_state, issues) = if blockers.is_empty() {
        (SideState::FailedRetryable, report.errors.clone())
    } else {
        (SideState::Blocked, blockers)
    };

    lifecycle_state.current_owner_command = "runtime-follow-on --write".to_string();
    lifecycle_state.last_transition_at = now_rfc3339()?;
    lifecycle_state.last_transition_by = "xtask runtime-follow-on --write".to_string();
    lifecycle_state.side_states = lifecycle_state
        .side_states
        .into_iter()
        .filter(|state| !matches!(state, SideState::Blocked | SideState::FailedRetryable))
        .collect();
    lifecycle_state.side_states.push(side_state);
    lifecycle_state.side_states.sort();
    lifecycle_state.side_states.dedup();
    match side_state {
        SideState::Blocked => {
            lifecycle_state.blocking_issues = issues;
            lifecycle_state.retryable_failures.clear();
        }
        SideState::FailedRetryable => {
            lifecycle_state.retryable_failures = issues;
            lifecycle_state.blocking_issues.clear();
        }
        SideState::Drifted | SideState::Deprecated => {}
    }

    write_lifecycle_state(
        workspace_root,
        &context.lifecycle_state_path,
        &lifecycle_state,
    )
    .map_err(|err| Error::Internal(format!("write lifecycle state: {err}")))
}

fn required_evidence_strings(stage: LifecycleStage) -> Vec<EvidenceId> {
    agent_lifecycle::required_evidence_for_stage(stage).to_vec()
}

fn build_implementation_summary(
    context: &RuntimeContext,
    prior_contract: &InputContract,
    written_paths: &[String],
) -> ImplementationSummary {
    let requested_runtime_profile = requested_runtime_profile(prior_contract);
    let primary_template = primary_template(context, prior_contract, requested_runtime_profile);
    let mut landed_surfaces = BTreeSet::new();
    landed_surfaces.insert(LandedSurface::WrapperRuntime);
    landed_surfaces.insert(LandedSurface::BackendHarness);
    landed_surfaces.insert(LandedSurface::WrapperCoverageSource);
    landed_surfaces.insert(LandedSurface::RuntimeManifestEvidence);

    if wrote_agent_api_onboarding_test(prior_contract, written_paths) {
        landed_surfaces.insert(LandedSurface::AgentApiOnboardingTest);
    }

    for capability in all_capability_and_extension_entries(prior_contract) {
        match rich_surface_from_entry(capability) {
            Some(surface) => {
                landed_surfaces.insert(surface);
            }
            None => {}
        }
    }

    let deferred_surfaces = prior_contract
        .allow_rich_surface
        .iter()
        .filter_map(|surface| {
            let mapped = rich_surface_from_entry(surface)?;
            if landed_surfaces.contains(&mapped) {
                None
            } else {
                Some(DeferredSurface {
                    surface: mapped,
                    reason: "Allowed for this run but not landed in the runtime integration delta."
                        .to_string(),
                })
            }
        })
        .collect::<Vec<_>>();

    ImplementationSummary {
        requested_runtime_profile,
        achieved_runtime_profile: requested_runtime_profile,
        primary_template,
        template_lineage: vec![template_name(primary_template).to_string()],
        landed_surfaces: landed_surfaces.into_iter().collect(),
        deferred_surfaces,
        minimal_profile_justification: prior_contract.minimal_justification_text.clone(),
    }
}

fn requested_runtime_profile(prior_contract: &InputContract) -> RuntimeProfile {
    match prior_contract.requested_tier.as_str() {
        "minimal" => RuntimeProfile::Minimal,
        "feature-rich" => RuntimeProfile::FeatureRich,
        _ => RuntimeProfile::Default,
    }
}

fn primary_template(
    context: &RuntimeContext,
    _prior_contract: &InputContract,
    requested_runtime_profile: RuntimeProfile,
) -> TemplateId {
    match context.approval.descriptor.agent_id.as_str() {
        "opencode" => TemplateId::Opencode,
        "gemini_cli" => TemplateId::GeminiCli,
        "codex" => TemplateId::Codex,
        "claude_code" => TemplateId::ClaudeCode,
        "aider" => TemplateId::Aider,
        _ => match requested_runtime_profile {
            RuntimeProfile::Minimal => TemplateId::GeminiCli,
            RuntimeProfile::FeatureRich => TemplateId::Codex,
            RuntimeProfile::Default => TemplateId::Opencode,
        },
    }
}

fn template_name(template: TemplateId) -> &'static str {
    match template {
        TemplateId::Opencode => "opencode",
        TemplateId::GeminiCli => "gemini_cli",
        TemplateId::Codex => "codex",
        TemplateId::ClaudeCode => "claude_code",
        TemplateId::Aider => "aider",
    }
}

fn wrote_agent_api_onboarding_test(
    prior_contract: &InputContract,
    written_paths: &[String],
) -> bool {
    written_paths.iter().any(|path| {
        path == &prior_contract.required_agent_api_test
            || path.starts_with(&format!(
                "crates/agent_api/tests/c1_{}_runtime_follow_on/",
                prior_contract.agent_id
            ))
    })
}

fn all_capability_and_extension_entries(prior_contract: &InputContract) -> Vec<&str> {
    prior_contract
        .always_on_capabilities
        .iter()
        .chain(prior_contract.target_gated_capabilities.iter())
        .chain(prior_contract.config_gated_capabilities.iter())
        .chain(prior_contract.backend_extensions.iter())
        .map(String::as_str)
        .collect()
}

fn rich_surface_from_entry(entry: &str) -> Option<LandedSurface> {
    let normalized = entry.replace('_', "-");
    if normalized.contains("agent-api-exec-add-dirs-v1") || normalized == "add-dirs" {
        Some(LandedSurface::AddDirs)
    } else if normalized.contains("agent-api-exec-external-sandbox-v1")
        || normalized == "external-sandbox-policy"
    {
        Some(LandedSurface::ExternalSandboxPolicy)
    } else if normalized.contains("agent-api-tools-mcp-") || normalized == "mcp-management" {
        Some(LandedSurface::McpManagement)
    } else if normalized.contains("agent-api-session-resume-v1") || normalized == "session-resume" {
        Some(LandedSurface::SessionResume)
    } else if normalized.contains("agent-api-session-fork-v1") || normalized == "session-fork" {
        Some(LandedSurface::SessionFork)
    } else if normalized.contains("agent-api-tools-structured-v1")
        || normalized == "structured-tools"
    {
        Some(LandedSurface::StructuredTools)
    } else {
        None
    }
}

fn best_effort_handoff_blockers(path: &Path) -> Vec<String> {
    let Ok(payload) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&payload) else {
        return Vec::new();
    };
    let Some(blockers) = parsed.get("blockers").and_then(|value| value.as_array()) else {
        return Vec::new();
    };
    blockers
        .iter()
        .filter_map(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter(|value| *value != "Pending runtime follow-on implementation.")
        .map(ToOwned::to_owned)
        .collect()
}

fn required_publication_commands() -> Vec<String> {
    agent_lifecycle::REQUIRED_PUBLICATION_COMMANDS
        .iter()
        .map(|value| (*value).to_string())
        .collect()
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

fn required_agent_api_test(agent_id: &str) -> String {
    format!("crates/agent_api/tests/c1_{agent_id}_runtime_follow_on.rs")
}

fn docs_to_read(agent_id: &str) -> Vec<String> {
    vec![
        "docs/cli-agent-onboarding-factory-operator-guide.md".to_string(),
        "docs/specs/cli-agent-onboarding-charter.md".to_string(),
        "docs/adr/0013-agent-api-backend-harness.md".to_string(),
        format!(
            "docs/agents/lifecycle/{}-onboarding/HANDOFF.md",
            agent_id.replace('_', "-")
        ),
        "crates/agent_api/src/backends/opencode".to_string(),
        "crates/opencode".to_string(),
    ]
}

fn allowed_write_paths(descriptor: &crate::approval_artifact::ApprovalDescriptor) -> Vec<String> {
    vec![
        "Cargo.lock".to_string(),
        format!("{}/**", descriptor.crate_path),
        format!("{}/**", descriptor.backend_module),
        "crates/agent_api/Cargo.toml".to_string(),
        "crates/agent_api/src/backends/mod.rs".to_string(),
        "crates/agent_api/src/lib.rs".to_string(),
        required_agent_api_test(&descriptor.agent_id),
        format!(
            "crates/agent_api/tests/c1_{}_runtime_follow_on/**",
            descriptor.agent_id
        ),
        format!("crates/agent_api/src/bin/fake_{}*", descriptor.agent_id),
        format!("crates/agent_api/src/bin/fake_{}*/**", descriptor.agent_id),
        format!("{}/src/**", descriptor.wrapper_coverage_source_path),
        format!("{}/snapshots/**", descriptor.manifest_root),
        format!("{}/supplement/**", descriptor.manifest_root),
        format!("{}/**", RUNTIME_RUNS_ROOT),
    ]
}

fn execute_codex_write(
    workspace_root: &Path,
    context: &RuntimeContext,
    args: &Args,
    prompt: &str,
) -> Result<CodexExecutionEvidence, Error> {
    let binary = resolve_codex_binary(args);
    let argv = vec![
        "exec".to_string(),
        "--skip-git-repo-check".to_string(),
        "--dangerously-bypass-approvals-and-sandbox".to_string(),
        "--cd".to_string(),
        workspace_root.to_string_lossy().into_owned(),
    ];
    let mut child = Command::new(&binary)
        .current_dir(workspace_root)
        .args(&argv)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| Error::Internal(format!("spawn codex binary `{binary}`: {err}")))?;
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| Error::Internal("codex exec stdin was not captured".to_string()))?;
        stdin
            .write_all(prompt.as_bytes())
            .map_err(|err| Error::Internal(format!("write codex prompt to stdin: {err}")))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|err| Error::Internal(format!("wait for codex exec: {err}")))?;
    let stdout_path = context.run_dir.join(CODEX_STDOUT_FILE_NAME);
    let stderr_path = context.run_dir.join(CODEX_STDERR_FILE_NAME);
    write_string(&stdout_path, &String::from_utf8_lossy(&output.stdout))?;
    write_string(&stderr_path, &String::from_utf8_lossy(&output.stderr))?;

    Ok(CodexExecutionEvidence {
        workflow_version: WORKFLOW_VERSION.to_string(),
        generated_at: now_rfc3339()?,
        run_id: context.run_id.clone(),
        binary,
        argv,
        prompt_path: context
            .run_dir
            .join(PROMPT_FILE_NAME)
            .to_string_lossy()
            .into_owned(),
        stdout_path: stdout_path.to_string_lossy().into_owned(),
        stderr_path: stderr_path.to_string_lossy().into_owned(),
        exit_code: output.status.code().unwrap_or(1),
    })
}

fn resolve_codex_binary(args: &Args) -> String {
    args.codex_binary
        .clone()
        .or_else(|| std::env::var(CODEX_BINARY_ENV).ok())
        .unwrap_or_else(|| "codex".to_string())
}

fn approval_descriptor_view(
    input_contract: &InputContract,
) -> crate::approval_artifact::ApprovalDescriptor {
    crate::approval_artifact::ApprovalDescriptor {
        agent_id: input_contract.agent_id.clone(),
        display_name: input_contract.display_name.clone(),
        crate_path: input_contract.crate_path.clone(),
        backend_module: input_contract.backend_module.clone(),
        manifest_root: input_contract.manifest_root.clone(),
        package_name: String::new(),
        canonical_targets: input_contract.canonical_targets.clone(),
        wrapper_coverage_binding_kind: String::new(),
        wrapper_coverage_source_path: input_contract.wrapper_coverage_source_path.clone(),
        always_on_capabilities: input_contract.always_on_capabilities.clone(),
        target_gated_capabilities: input_contract.target_gated_capabilities.clone(),
        config_gated_capabilities: input_contract.config_gated_capabilities.clone(),
        backend_extensions: input_contract.backend_extensions.clone(),
        support_matrix_enabled: input_contract.support_matrix_enabled,
        capability_matrix_enabled: input_contract.capability_matrix_enabled,
        capability_matrix_target: input_contract.capability_matrix_target.clone(),
        docs_release_track: String::new(),
        onboarding_pack_prefix: String::new(),
    }
}

fn is_allowed_write_path(
    path: &str,
    descriptor: &crate::approval_artifact::ApprovalDescriptor,
) -> bool {
    let required_test = required_agent_api_test(&descriptor.agent_id);
    let runtime_test_dir = format!(
        "crates/agent_api/tests/c1_{}_runtime_follow_on/",
        descriptor.agent_id
    );
    let fake_bin_prefix = format!("crates/agent_api/src/bin/fake_{}", descriptor.agent_id);

    path == "crates/agent_api/Cargo.toml"
        || path == "Cargo.lock"
        || path == "crates/agent_api/src/backends/mod.rs"
        || path == "crates/agent_api/src/lib.rs"
        || path == required_test
        || path.starts_with(&(descriptor.crate_path.clone() + "/"))
        || path.starts_with(&(descriptor.backend_module.clone() + "/"))
        || path.starts_with(&runtime_test_dir)
        || path == fake_bin_prefix
        || path.starts_with(&fake_bin_prefix)
        || path.starts_with(&(descriptor.wrapper_coverage_source_path.clone() + "/src/"))
        || path.starts_with(&(descriptor.manifest_root.clone() + "/snapshots/"))
        || path.starts_with(&(descriptor.manifest_root.clone() + "/supplement/"))
}

fn is_publication_owned_manifest_path(path: &str, manifest_root: &str) -> bool {
    path.starts_with(&(manifest_root.to_string() + "/"))
        && !path.starts_with(&(manifest_root.to_string() + "/snapshots/"))
        && !path.starts_with(&(manifest_root.to_string() + "/supplement/"))
}

fn map_approval_error(err: ApprovalArtifactError) -> Error {
    match err {
        ApprovalArtifactError::Validation(message) => Error::Validation(message),
        ApprovalArtifactError::Internal(message) => Error::Internal(message),
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
