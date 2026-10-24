//! Independently rederive current bindings without turning today's inventory into a new P baseline.
use super::*;
use crate::{
    agent_lifecycle::maintenance_request_path,
    depth_record::{Event, ObligationsPart},
    git_inputs::GitInputs,
};

pub(crate) fn current_closeout_bindings(
    root: &Path,
    record: &DepthRecord,
) -> Result<(PolicyPart, ObligationsPart), String> {
    let mut view = GitInputs::new(root)?;
    let agent = &record.policy.selection.agent_id;
    let version = &record.policy.selection.enrollment.version;
    let raw_registry = view.required(crate::agent_registry::REGISTRY_RELATIVE_PATH)?;
    let registry =
        AgentRegistry::parse_unenforced(text(&raw_registry)?).map_err(|e| e.to_string())?;
    let entry = registry.find(agent).ok_or("missing closeout agent")?;
    let raw_record = view.required(&format!(
        "{}/reports/{version}/depth-record.json",
        entry.manifest_root
    ))?;
    if DepthRecord::parse(text(&raw_record)?, agent, version)? != *record {
        return Err("closeout record differs from coherent stage-0 input".into());
    }
    let rules = view.required(&format!("{}/RULES.json", entry.manifest_root))?;
    let inventory = view.required(NON_TUI_SUPPORT_DEBT_PATH)?;
    let policy = resolve(
        text(&raw_registry)?,
        agent,
        version,
        text(&rules)?,
        Some(text(&raw_record)?),
        text(&inventory)?,
        Some(&record.policy.debt_baseline),
    )?
    .ok_or("closeout lost depth enrollment")?;
    check_current_policy(&record.policy, &policy)?;
    let request = view.required(&maintenance_request_path(agent))?;
    let request: serde_json::Value =
        toml_edit::de::from_str(text(&request)?).map_err(|e| e.to_string())?;
    let string = |key: &str| {
        request[key]
            .as_str()
            .ok_or_else(|| format!("missing closeout request {key}"))
    };
    let event = Event::Maintenance {
        request_commit: string("request_commit")?.into(),
        request_recorded_at: string("request_recorded_at")?.into(),
        trigger_kind: string("trigger_kind")?.into(),
        opened_from: string("opened_from")?.into(),
    };
    if event != record.event
        || request["agent_id"] != *agent
        || request["detected_release"]["target_version"] != *version
        || request["support_depth"]["policy_identity"] != policy.identity
        || request["support_depth"]["obligations_identity"]
            != record
                .obligations
                .as_ref()
                .ok_or("closeout requires O")?
                .identity
    {
        return Err("closeout request is not the exact current generation/P/O".into());
    }
    let carried = record
        .obligations
        .as_ref()
        .ok_or("closeout requires O")?
        .acceptance_work
        .clone();
    let mut required: BTreeSet<_> = carried.iter().map(|w| w.surface.clone()).collect();
    if let Some(rows) = request["support_surface_audit"]["required_uplifts_this_run"].as_array() {
        for row in rows {
            let text = |k: &str| {
                row[k]
                    .as_str()
                    .ok_or_else(|| format!("invalid due work {k}"))
            };
            required.insert(
                crate::agent_maintenance::support_audit::SurfaceIdentity::new(
                    text("surface_kind")?.into(),
                    text("command_path")?.into(),
                    text("surface_id")?.into(),
                ),
            );
        }
    }
    let obligations = acquisition::derive(
        &mut view,
        &entry.manifest_root,
        &rules,
        &policy,
        carried,
        &required,
    )?
    .ok_or("current acquisition has no frozen O")?;
    view.verify()?;
    Ok((policy, obligations))
}
fn text(bytes: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(bytes).map_err(|e| e.to_string())
}
