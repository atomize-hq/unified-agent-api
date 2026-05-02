use std::path::Path;

use crate::{
    agent_lifecycle::{self, load_lifecycle_state, LifecycleStage},
    agent_registry::AgentRegistryEntry,
    approval_artifact::load_approval_artifact,
    prepare_publication::discover_runtime_evidence_for_approval,
};

use super::{build_finding, DriftCategory, DriftFinding};

const RUNTIME_RUNS_ROOT: &str = "docs/agents/.uaa-temp/runtime-follow-on/runs";

pub(super) fn has_runtime_integrated_lifecycle(
    entry: &AgentRegistryEntry,
    workspace_root: &Path,
) -> bool {
    let lifecycle_state_path =
        agent_lifecycle::lifecycle_state_path(&entry.scaffold.onboarding_pack_prefix);
    if !workspace_root.join(&lifecycle_state_path).is_file() {
        return false;
    }

    matches!(
        load_lifecycle_state(workspace_root, &lifecycle_state_path)
            .map(|state| state.lifecycle_stage),
        Ok(LifecycleStage::RuntimeIntegrated)
    )
}

pub(super) fn inspect_runtime_evidence(
    entry: &AgentRegistryEntry,
    workspace_root: &Path,
) -> Option<DriftFinding> {
    let lifecycle_state_path =
        agent_lifecycle::lifecycle_state_path(&entry.scaffold.onboarding_pack_prefix);
    if !has_runtime_integrated_lifecycle(entry, workspace_root) {
        return None;
    }
    let lifecycle_state = load_lifecycle_state(workspace_root, &lifecycle_state_path).ok()?;

    let approval =
        load_approval_artifact(workspace_root, &lifecycle_state.approval_artifact_path).ok()?;
    let approval_path = approval.relative_path.clone();
    match discover_runtime_evidence_for_approval(workspace_root, &approval) {
        Ok(_) => None,
        Err(err) => {
            let repair_run_dir = repair_run_dir(&entry.agent_id);
            let repair_check = repair_command("--check", &approval_path);
            let repair_write = repair_command("--write", &approval_path);
            Some(build_finding(
                DriftCategory::RuntimeEvidence,
                "runtime evidence for a `runtime_integrated` agent is stale and must be repaired before publication checks can be trusted.",
                vec![
                    err.to_string(),
                    format!(
                        "Validate the repair target with `{repair_check}` or rebuild `{repair_run_dir}` with `{repair_write}`; this repair does not advance lifecycle stage."
                    ),
                ],
                vec![
                    lifecycle_state_path,
                    approval_path,
                    RUNTIME_RUNS_ROOT.to_string(),
                    repair_run_dir,
                ],
            ))
        }
    }
}

fn repair_run_dir(agent_id: &str) -> String {
    format!("{RUNTIME_RUNS_ROOT}/repair-{agent_id}-runtime-follow-on")
}

fn repair_command(mode_flag: &str, approval_path: &str) -> String {
    format!("cargo run -p xtask -- repair-runtime-evidence --approval {approval_path} {mode_flag}")
}
