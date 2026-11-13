//! Shared observed-write checks.
use super::*;

pub(super) fn validate_write_mode(
    workspace_root: &Path,
    context: &RuntimeContext,
    prior_contract: &InputContract,
    codex_execution: &CodexExecutionEvidence,
) -> Result<(ValidationReport, Vec<String>), Error> {
    let current_snapshot = snapshot_workspace(workspace_root, &[Path::new(RUNTIME_RUNS_ROOT)])?;
    let changed_paths = diff_snapshots(&prior_contract.baseline, &current_snapshot);
    validate_observed_write_mode(
        workspace_root,
        context,
        prior_contract,
        codex_execution,
        changed_paths,
    )
}

pub(super) fn validate_observed_write_mode(
    workspace_root: &Path,
    context: &RuntimeContext,
    prior_contract: &InputContract,
    codex_execution: &CodexExecutionEvidence,
    changed_paths: Vec<String>,
) -> Result<(ValidationReport, Vec<String>), Error> {
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

    if context.wrapper_only {
        depth::check_implementation(
            workspace_root,
            context,
            &written_paths,
            &mut checks,
            &mut errors,
        )?;
    } else {
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
