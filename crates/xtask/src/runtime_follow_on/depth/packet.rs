//! Reporting artifacts for the private non-runtime proof. These never authorize execution.
use super::*;
use crate::workspace_mutation::{PlannedMutation, WorkspacePathJail};
use serde_json::json;

pub(super) fn json_bytes(value: &impl Serialize) -> Result<Vec<u8>, Error> {
    serde_json::to_vec_pretty(value).map_err(|e| Error::Internal(e.to_string()))
}

pub(super) fn prepare(
    workspace: &Path,
    args: &Args,
    run_id: &str,
    freeze: &Freeze,
) -> Result<Vec<PlannedMutation>, Error> {
    let entry = freeze
        .captured
        .registry()
        .find(&freeze.approval.agent_id)
        .ok_or_else(|| Error::Validation("missing isolated subject".into()))?;
    let object = freeze
        .state
        .support_depth
        .as_ref()
        .ok_or_else(|| Error::Validation("missing planned lifecycle object".into()))?;
    let generated_at = now_rfc3339()?;
    let contract = InputContract {
        workflow_version: WORKFLOW_VERSION.into(),
        support_depth: Some(SupportDepthContract {
            version: freeze
                .plan
                .record
                .policy
                .selection
                .enrollment
                .version
                .clone(),
            policy_identity: object.policy_identity.clone(),
            obligations_identity: object.obligations_identity.clone(),
        }),
        generated_at: generated_at.clone(),
        run_id: run_id.into(),
        approval_artifact_path: args.approval.clone(),
        approval_artifact_sha256: freeze.approval_sha256.clone(),
        agent_id: entry.agent_id.clone(),
        display_name: entry.display_name.clone(),
        crate_path: entry.crate_path.clone(),
        // The compiled synthetic subject has no runtime backend. This reporting-only packet
        // cannot be passed to the public execution loader, which continues to require one.
        backend_module: String::new(),
        manifest_root: entry.manifest_root.clone(),
        wrapper_coverage_source_path: entry.wrapper_coverage.source_path.clone(),
        canonical_targets: entry.canonical_targets.clone(),
        always_on_capabilities: entry.capability_declaration.always_on.clone(),
        target_gated_capabilities: vec![],
        config_gated_capabilities: vec![],
        backend_extensions: entry.capability_declaration.backend_extensions.clone(),
        support_matrix_enabled: entry.publication.support_matrix_enabled,
        capability_matrix_enabled: entry.publication.capability_matrix_enabled,
        capability_matrix_target: entry.publication.capability_matrix_target.clone(),
        requested_tier: args.requested_tier.as_str().into(),
        minimal_justification_file: args.minimal_justification_file.clone(),
        minimal_justification_text: args
            .minimal_justification_file
            .as_ref()
            .map(|p| read_string(&workspace.join(p)))
            .transpose()?,
        allow_rich_surface: args.allow_rich_surface.clone(),
        required_agent_api_test: required_agent_api_test(&entry.agent_id),
        required_handoff_commands: required_publication_commands(),
        docs_to_read: docs_to_read(&entry.agent_id),
        allowed_write_paths: vec![
            format!("{}/**", entry.crate_path),
            format!("{}/versions/**", entry.manifest_root),
        ],
        ignored_diff_roots: vec![RUNTIME_RUNS_ROOT.into()],
        baseline: snapshot_workspace(workspace, &[Path::new(RUNTIME_RUNS_ROOT)])?,
    };
    let run_dir = format!("{RUNTIME_RUNS_ROOT}/{run_id}");
    let status = json!({
        "workflow_version":WORKFLOW_VERSION, "generated_at":generated_at,
        "run_id":run_id, "approval_artifact_path":args.approval,
        "agent_id":entry.agent_id, "requested_tier":args.requested_tier.as_str(),
        "host_surface":"isolated synthetic dry run", "loaded_skill_ref":SKILL_PATH,
        "mode":"dry_run", "status":"dry_run_ready", "validation_passed":true,
        "handoff_ready":false, "run_dir":run_dir, "written_paths":[], "errors":[],
    });
    let report = ValidationReport {
        workflow_version: WORKFLOW_VERSION.into(),
        generated_at,
        run_id: run_id.into(),
        status: "dry_run_ready".into(),
        checks: vec![],
        errors: vec![],
    };
    let prompt = format!("# Isolated synthetic runtime-follow-on proof\n\nRead input-contract.json. Preserve its frozen support_depth identities. The executor's only run file is handoff.json. Snapshots, governance and acquired inputs are protected. This reporting packet grants no production authority.\n\nAgent: {}\n", entry.agent_id);
    let summary = format!("# Runtime-follow-on dry run\n\nrun_id: {run_id}\n\nP: {}\nO: {}\n", object.policy_identity,
        object.obligations_identity.as_deref().unwrap_or("not yet produced; a passing write closes the window; depth acceptance requires a new approval"));
    let files = [
        (INPUT_CONTRACT_FILE_NAME, json_bytes(&contract)?),
        (PROMPT_FILE_NAME, prompt.into_bytes()),
        (RUN_STATUS_FILE_NAME, json_bytes(&status)?),
        (VALIDATION_REPORT_FILE_NAME, json_bytes(&report)?),
        (RUN_SUMMARY_FILE_NAME, summary.into_bytes()),
    ];
    let jail = WorkspacePathJail::new(workspace).map_err(|e| Error::Validation(e.to_string()))?;
    files
        .into_iter()
        .map(|(name, after)| {
            let path = format!("{run_dir}/{name}");
            let full = jail
                .resolve(Path::new(&path))
                .map_err(|e| Error::Validation(e.to_string()))?;
            match fs::read(full) {
                Ok(before) => Ok(PlannedMutation::replace(path, before, after)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    Ok(PlannedMutation::create(path, after))
                }
                Err(e) => Err(Error::Validation(e.to_string())),
            }
        })
        .collect()
}
