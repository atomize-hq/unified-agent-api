//! Current onboarding bindings reuse policy/acquisition derivation without freezing or rebasing P.
use super::*;
use crate::{
    depth_record::{onboarding::OnboardingApproval, Event, ObligationsPart},
    git_inputs::GitInputs,
    runtime_advertising::source::Source,
};

pub(crate) fn current_onboarding_bindings(
    inputs: &mut GitInputs,
    candidate: &Source,
    record: &DepthRecord,
) -> Result<(PolicyPart, ObligationsPart), String> {
    if !matches!(record.event, Event::Onboarding { .. })
        || record.policy.selection.enrollment.lifecycle_path
            != crate::agent_registry::support_depth::LifecyclePath::Onboarding
        || !record.policy.path_enabled
    {
        return Err("onboarding has no lifecycle-path authority".into());
    }
    let agent = &record.policy.selection.agent_id;
    let version = &record.policy.selection.enrollment.version;
    let registry = inputs.required(crate::agent_registry::REGISTRY_RELATIVE_PATH)?;
    let registry_text = text(&registry)?;
    let parsed = AgentRegistry::parse_unenforced(registry_text).map_err(|e| e.to_string())?;
    let entry = parsed.find(agent).ok_or("onboarding lost registry agent")?;
    let path = crate::agent_lifecycle::approval_artifact_path_for_entry(entry);
    let approval = inputs.required(&path)?;
    let approval = OnboardingApproval::parse(text(&approval)?, &path)?;
    if approval.event != record.event || approval.support_depth_version.as_ref() != Some(version) {
        return Err("onboarding approval is not the exact current Event/version".into());
    }
    if let Some(why) = approval.attribution_mismatch(
        agent,
        &entry.manifest_root,
        &entry.scaffold.onboarding_pack_prefix,
    ) {
        return Err(why);
    }
    let path = format!(
        "{}/reports/{version}/depth-record.json",
        entry.manifest_root
    );
    if DepthRecord::parse(&candidate.read(&path)?, agent, version)? != *record {
        return Err("onboarding record differs from coherent candidate bytes".into());
    }
    let rules = inputs.required(&format!("{}/RULES.json", entry.manifest_root))?;
    let inventory = inputs.required(NON_TUI_SUPPORT_DEBT_PATH)?;
    let mut rows = parse_debt_inventory(text(&inventory)?, Path::new(NON_TUI_SUPPORT_DEBT_PATH))?;
    rows.retain(|row| row.agent_id == *agent);
    if !record.policy.debt_baseline.is_empty() || !rows.is_empty() {
        return Err("onboarding requires a debt-free baseline and inventory".into());
    }
    let raw = serde_json::to_string(record).map_err(|e| e.to_string())?;
    let policy = resolve(
        registry_text,
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
    .ok_or("onboarding lost enrollment")?;
    check_current_policy(&record.policy, &policy)?;
    let carried = record
        .obligations
        .as_ref()
        .ok_or("onboarding requires frozen O")?
        .acceptance_work
        .clone();
    let required = carried.iter().map(|work| work.surface.clone()).collect();
    let obligations = acquisition::derive(
        inputs,
        &entry.manifest_root,
        &rules,
        &policy,
        carried,
        &required,
    )?
    .ok_or("onboarding current acquisition has no frozen O")?
    .obligations;
    inputs.verify()?;
    Ok((policy, obligations))
}

fn text(bytes: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(bytes).map_err(|e| e.to_string())
}
