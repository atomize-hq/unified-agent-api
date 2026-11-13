//! Executable wrapper packet, built by the same context and prompt as write preparation.
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
    let raw = freeze
        .view
        .clone()
        .required(&args.approval)
        .map_err(Error::Validation)?;
    let approval = approval::execution(workspace, &raw, &args.approval, &freeze.captured)
        .map_err(Error::Validation)?;
    let mut args = args.clone();
    args.run_id = Some(run_id.into());
    let context = context::assemble(
        workspace,
        &args,
        approval,
        freeze.captured.registry(),
        freeze.state_path.clone(),
        freeze.state.clone(),
        true,
    )?;
    let contract = &context.input_contract;
    let entry = &context.approval.descriptor;
    let object = freeze
        .state
        .support_depth
        .as_ref()
        .ok_or_else(|| Error::Validation("missing planned lifecycle object".into()))?;
    let generated_at = now_rfc3339()?;
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
    let prompt = depth_candidate::render_depth_prompt(&context);
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
