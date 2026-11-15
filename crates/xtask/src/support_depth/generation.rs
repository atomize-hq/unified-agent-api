//! Re-derive a maintenance generation through the approved policy and acquisition logic.
use super::{acquisition, acquisition_inputs::Inputs, *};
use crate::agent_maintenance::support_audit::SurfaceIdentity;

pub(crate) fn check(
    inputs: &mut impl Inputs,
    record: &DepthRecord,
    required: &BTreeSet<SurfaceIdentity>,
) -> Result<(), String> {
    let agent = &record.policy.selection.agent_id;
    let version = &record.policy.selection.enrollment.version;
    let registry = inputs.required(crate::agent_registry::REGISTRY_RELATIVE_PATH)?;
    let parsed = AgentRegistry::parse_unenforced(text(&registry)?).map_err(|e| e.to_string())?;
    let entry = parsed
        .find(agent)
        .ok_or("maintenance generation lost registry agent")?;
    let rules = inputs.required(&format!("{}/RULES.json", entry.manifest_root))?;
    let inventory = inputs.required(NON_TUI_SUPPORT_DEBT_PATH)?;
    let raw = serde_json::to_string(record).map_err(|e| e.to_string())?;
    let policy = resolve(
        text(&registry)?,
        agent,
        version,
        text(&rules)?,
        Some(&raw),
        text(&inventory)?,
        ResolutionContext {
            baseline: Some(&record.policy.debt_baseline),
            intake: true,
            pilot: false,
        },
    )?
    .ok_or("maintenance generation lost enrollment")?;
    check_current_policy(&record.policy, &policy)?;
    let carried = record
        .obligations
        .as_ref()
        .map(|o| o.acceptance_work.clone())
        .unwrap_or_default();
    let obligations = acquisition::derive(
        inputs,
        &entry.manifest_root,
        &rules,
        &policy,
        carried,
        required,
    )?
    .map(|a| a.obligations);
    if obligations != record.obligations {
        return Err("maintenance generation has stale frozen O".into());
    }
    Ok(())
}
fn text(bytes: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(bytes).map_err(|e| e.to_string())
}
