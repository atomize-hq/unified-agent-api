//! Executable wrapper packet, built by the same context and prompt as write preparation.
use super::*;
use serde_json::json;

pub(super) fn json_bytes(value: &impl Serialize) -> Result<Vec<u8>, Error> {
    serde_json::to_vec_pretty(value).map_err(|e| Error::Internal(e.to_string()))
}

pub(super) fn context(
    workspace: &Path,
    args: &Args,
    run_id: &str,
    freeze: &Freeze,
    contract_generated_at: String,
) -> Result<RuntimeContext, Error> {
    let raw = freeze
        .view
        .clone()
        .required(&args.approval)
        .map_err(Error::Validation)?;
    let approval = approval::execution(workspace, &raw, &args.approval, &freeze.captured)
        .map_err(Error::Validation)?;
    let mut args = args.clone();
    args.run_id = Some(run_id.into());
    let context = context::assemble_at(
        workspace,
        &args,
        approval,
        freeze.captured.registry(),
        freeze.state_path.clone(),
        freeze.state.clone(),
        true,
        contract_generated_at,
    )?;
    Ok(context)
}

/// All seven executable-packet files, with explicit clock and invocation inputs.
pub(super) fn render(
    args: &Args,
    context: &RuntimeContext,
    generated_at: String,
) -> Result<Vec<(String, Vec<u8>)>, Error> {
    let run_id = &context.run_id;
    let contract = &context.input_contract;
    let entry = &context.approval.descriptor;
    let object = context
        .lifecycle_state
        .support_depth
        .as_ref()
        .ok_or_else(|| Error::Validation("missing planned lifecycle object".into()))?;
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
        run_id: run_id.clone(),
        status: "dry_run_ready".into(),
        checks: vec![],
        errors: vec![],
    };
    let prompt = depth_candidate::render_depth_prompt(context);
    let summary = format!("# Runtime-follow-on dry run\n\nrun_id: {run_id}\n\nP: {}\nO: {}\n", object.policy_identity,
        object.obligations_identity.as_deref().unwrap_or("not yet produced; a passing write closes the window; depth acceptance requires a new approval"));
    let files = [
        (INPUT_CONTRACT_FILE_NAME, json_bytes(contract)?),
        (PROMPT_FILE_NAME, prompt.into_bytes()),
        (RUN_STATUS_FILE_NAME, json_bytes(&status)?),
        (VALIDATION_REPORT_FILE_NAME, json_bytes(&report)?),
        (RUN_SUMMARY_FILE_NAME, summary.into_bytes()),
        (WRITTEN_PATHS_FILE_NAME, json_bytes(&Vec::<String>::new())?),
        (
            HANDOFF_FILE_NAME,
            json_bytes(&HandoffContract {
                agent_id: entry.agent_id.clone(),
                manifest_root: entry.manifest_root.clone(),
                runtime_lane_complete: false,
                publication_refresh_required: true,
                required_commands: required_publication_commands(),
                blockers: vec!["Pending runtime follow-on implementation.".into()],
            })?,
        ),
    ];
    Ok(files
        .into_iter()
        .map(|(name, bytes)| (format!("{run_dir}/{name}"), bytes))
        .collect())
}

pub(super) fn context_from_predecessor(
    workspace: &Path,
    args: &Args,
    registry: &AgentRegistry,
    state_path: String,
    state: LifecycleState,
    contract_generated_at: String,
) -> Result<RuntimeContext, String> {
    let raw = fs::read(workspace.join(&args.approval)).map_err(|e| e.to_string())?;
    let parsed = OnboardingApproval::parse(text(&raw)?, &args.approval)?;
    let entry = registry
        .find(&parsed.agent_id)
        .ok_or("unknown approval agent")?;
    let approval = approval::execution_for_entry(workspace, &raw, &args.approval, entry)?;
    let context = context::assemble_at(
        workspace,
        args,
        approval,
        registry,
        state_path,
        state,
        true,
        contract_generated_at,
    )
    .map_err(|e| e.to_string())?;
    Ok(context)
}
