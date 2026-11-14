//! Execution context; the ordinary loader remains the production default.
use super::*;

pub(super) fn build_context(workspace_root: &Path, args: &Args) -> Result<RuntimeContext, Error> {
    let approval =
        load_approval_artifact(workspace_root, &args.approval).map_err(map_approval_error)?;
    if approval.support_depth_version.is_some() {
        if let Some(run_id) = &args.run_id {
            crate::runtime_evidence_run::validate_run_id(run_id).map_err(Error::Validation)?;
        }
    }
    let registry = if approval.support_depth_version.is_some() {
        crate::isolated_maintenance::route(workspace_root)
            .map_err(Error::Validation)?
            .ok_or_else(|| {
                Error::Validation("onboarding depth execution requires an isolated route".into())
            })?
            .registry(workspace_root)
            .map_err(Error::Validation)?
            .clone()
    } else {
        AgentRegistry::load(workspace_root).map_err(|err| Error::Validation(err.to_string()))?
    };
    validate_alignment(&approval, &registry)?;
    let (lifecycle_state_path, lifecycle_state) =
        load_enrolled_lifecycle_state(workspace_root, &approval)?;
    assemble(
        workspace_root,
        args,
        approval,
        &registry,
        lifecycle_state_path,
        lifecycle_state,
        false,
    )
}

pub(super) fn assemble(
    workspace_root: &Path,
    args: &Args,
    approval: crate::approval_artifact::ApprovalArtifact,
    registry: &AgentRegistry,
    lifecycle_state_path: String,
    lifecycle_state: crate::agent_lifecycle::LifecycleState,
    wrapper_only: bool,
) -> Result<RuntimeContext, Error> {
    let generated_at = now_rfc3339()?;
    assemble_at(
        workspace_root,
        args,
        approval,
        registry,
        lifecycle_state_path,
        lifecycle_state,
        wrapper_only,
        generated_at,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn assemble_at(
    workspace_root: &Path,
    args: &Args,
    approval: crate::approval_artifact::ApprovalArtifact,
    registry: &AgentRegistry,
    lifecycle_state_path: String,
    lifecycle_state: crate::agent_lifecycle::LifecycleState,
    wrapper_only: bool,
    generated_at: String,
) -> Result<RuntimeContext, Error> {
    validate_alignment(&approval, registry)?;
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
    let required_agent_api_test = if wrapper_only {
        String::new()
    } else {
        required_agent_api_test(&approval.descriptor.agent_id)
    };
    let mut input_contract = InputContract {
        workflow_version: WORKFLOW_VERSION.to_string(),
        support_depth: SupportDepthContract::copy_from(&approval, &lifecycle_state)
            .map_err(Error::Validation)?,
        generated_at,
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

    if approval.support_depth_version.is_some() {
        input_contract
            .allowed_write_paths
            .retain(|p| p != &format!("{}/snapshots/**", approval.descriptor.manifest_root));
    }
    if wrapper_only {
        input_contract.allowed_write_paths = vec![format!("{}/**", approval.descriptor.crate_path)];
        input_contract.docs_to_read = vec![
            "docs/specs/cli-agent-onboarding-charter.md".into(),
            "docs/specs/support-depth-contract.md".into(),
        ];
    }
    Ok(RuntimeContext {
        wrapper_only,
        approval,
        input_contract,
        lifecycle_state,
        lifecycle_state_path,
        run_id,
        run_dir,
    })
}

fn validate_alignment(
    approval: &crate::approval_artifact::ApprovalArtifact,
    registry: &AgentRegistry,
) -> Result<(), Error> {
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
    validate_registry_alignment(approval, &registry_entry)
}
