//! Manual maintenance authority. Public claims, conditional debt plans and existing entries do
//! not replace independently current bindings or evaluated evidence.
use super::{
    artifacts,
    input::{self, View},
};
use crate::{
    agent_maintenance::{stand_down, support_audit::NON_TUI_SUPPORT_DEBT_PATH},
    agent_registry::{support_depth::LifecyclePath, AgentRegistry, REGISTRY_RELATIVE_PATH},
    depth_evidence::EvidenceVerdicts,
    depth_record::{
        AcceptanceEntry, DepthRecord, DepthResult, Event, ObligationRef, ObligationsPart,
        PolicyPart,
    },
    runtime_advertising::source::Source,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(super) trait Host {
    fn bindings(
        &self,
        root: &Path,
        record: &DepthRecord,
    ) -> Result<(PolicyPart, ObligationsPart), String>;
    fn evidence(&self, root: &Path, record: &DepthRecord) -> Result<EvidenceVerdicts, String>;
}
struct Production;
impl Host for Production {
    fn bindings(
        &self,
        root: &Path,
        record: &DepthRecord,
    ) -> Result<(PolicyPart, ObligationsPart), String> {
        crate::support_depth::current_closeout_bindings(root, record)
    }
    fn evidence(&self, root: &Path, record: &DepthRecord) -> Result<EvidenceVerdicts, String> {
        crate::depth_evidence::production_results(root, record)
    }
}
pub(super) fn plan(
    root: &Path,
    before: Source,
    after: Source,
    written: &BTreeSet<String>,
) -> Result<BTreeMap<String, Vec<u8>>, String> {
    plan_with(root, before, after, written, &Production)
}
fn plan_with(
    root: &Path,
    before: Source,
    after: Source,
    written: &BTreeSet<String>,
    host: &impl Host,
) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let raw = before.read(REGISTRY_RELATIVE_PATH)?;
    if after.read(REGISTRY_RELATIVE_PATH)? != raw {
        return Err("closeout cannot change registry authority".into());
    }
    let registry = AgentRegistry::parse_unenforced(&raw).map_err(|e| e.to_string())?;
    let mut allowed = BTreeSet::new();
    for entry in &registry.agents {
        allowed.extend(output_paths(entry));
    }
    if !written.is_subset(&allowed) {
        return Err("manual closeout proposed an unowned output".into());
    }
    let mut outputs = BTreeMap::new();
    let mut integration = None;
    for entry in &registry.agents {
        if written.is_disjoint(&output_paths(entry)) {
            continue;
        }
        let old = View::with_registry(before.clone(), &registry, &entry.agent_id)?;
        let new = View::with_registry(after.clone(), &registry, &entry.agent_id)?;
        if old.records != new.records {
            return Err(
                "closeout must preserve every retained record and entry before admission".into(),
            );
        }
        if old.records.is_empty() && entry.support_depth.is_none() {
            continue;
        }
        let closeout_path = format!(
            "docs/agents/lifecycle/{}-maintenance/governance/maintenance-closeout.json",
            entry.agent_id
        );
        // Resolve old ownership even when a newer ordinary request replaces the working file.
        artifacts::maintenance_owner(&old)?;
        let destination = artifacts::maintenance_owner(&new)?;
        let mut effects = BTreeSet::new();
        artifacts::detect(&old, &new, &mut effects)?;
        let selected = |version: &str| {
            new.records.contains_key(version)
                || entry
                    .support_depth
                    .as_ref()
                    .and_then(|d| d.enrollments.as_ref())
                    .is_some_and(|es| es.iter().any(|e| e.version == version))
        };
        let mut accepting: BTreeSet<String> = effects
            .into_iter()
            .filter(|e| selected(&e.version))
            .map(|e| e.version)
            .collect();
        // Even an idempotent write must establish authority and have the exact matching entry.
        if let Some(version) =
            destination.filter(|v| written.contains(&closeout_path) && selected(v))
        {
            accepting.insert(version);
        }
        for version in accepting {
            let record = new
                .records
                .get(&version)
                .ok_or("selected closeout destination has no depth record")?;
            if record.policy.selection.enrollment.lifecycle_path != LifecyclePath::Maintenance
                || !record.policy.path_enabled
            {
                return Err("manual maintenance closeout has no lifecycle-path authority".into());
            }
            let closeout = new.json(&closeout_path)?;
            let request_digest = input::text(&closeout, "request_sha256")?;
            if closeout["preflight_passed"] != true {
                return Err("depth closeout requires passed preflight".into());
            }
            crate::agent_maintenance::closeout::require_commit_binding(
                root,
                input::text(&closeout, "commit")?,
            )
            .map_err(|e| e.to_string())?;
            let Event::Maintenance {
                request_recorded_at,
                ..
            } = &record.event
            else {
                return Err("not a maintenance generation".into());
            };
            let tip = integration.get_or_insert(integration_tip(root)?).clone();
            let marker_path = stand_down::marker_relative_path(
                &format!("docs/agents/lifecycle/{}-maintenance", entry.agent_id),
                &version,
            );
            let marker = Source::tip(root, &tip)?.read(&marker_path)?;
            stand_down::require_generation_marker(
                &entry.agent_id,
                &version,
                &marker,
                request_recorded_at,
                request_digest,
            )?;
            admitted(root, &new, record, host)?;
            let mut accepted = record.clone();
            let acceptance = AcceptanceEntry::MaintenanceCloseout {
                closeout: crate::depth_record::FileRef {
                    path: closeout_path.clone(),
                    sha256: artifacts::digest(&new.source.read(&closeout_path)?),
                },
                record_sha256: record.stated_sha256()?,
            };
            if !accepted.acceptance_entries.contains(&acceptance) {
                accepted.acceptance_entries.push(acceptance);
            }
            let mut bytes = serde_json::to_vec_pretty(&accepted).map_err(|e| e.to_string())?;
            bytes.push(b'\n');
            outputs.insert(
                format!(
                    "{}/reports/{version}/depth-record.json",
                    entry.manifest_root
                ),
                bytes,
            );
        }
    }
    if let Some(tip) = integration {
        if integration_tip(root)? != tip {
            return Err("integration tip moved during closeout admission".into());
        }
    }
    Ok(outputs)
}
fn integration_tip(root: &Path) -> Result<String, String> {
    let out = crate::git_inputs::command(root)
        .args(["rev-parse", "--verify", "origin/staging^{commit}"])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err("cannot resolve integration tip for closeout".into());
    }
    String::from_utf8(out.stdout)
        .map(|s| s.trim().to_owned())
        .map_err(|e| e.to_string())
}
pub(super) fn admitted(
    root: &Path,
    view: &View,
    record: &DepthRecord,
    host: &impl Host,
) -> Result<(), String> {
    let o = record
        .obligations
        .as_ref()
        .ok_or("closeout requires frozen O")?;
    let e = record
        .execution
        .as_ref()
        .ok_or("closeout requires current E")?;
    if o.policy_identity != record.policy.identity
        || e.policy_identity != record.policy.identity
        || e.obligations_identity != o.identity
    {
        return Err("closeout P/O/E binding is stale".into());
    }
    let (policy, obligations) = host.bindings(root, record)?;
    crate::support_depth::check_current_policy(&record.policy, &policy)?;
    if obligations != *o {
        return Err("closeout frozen O is no longer current".into());
    }
    if !o.classification_required.is_empty() {
        return Err("closeout has classification_required units".into());
    }
    // Current reports can describe names, not independently establish semantic satisfaction or
    // upstream removal. No corroborating producer is installed for these required work items.
    if !o.acceptance_work.is_empty() {
        return Err("closeout due name-level work lacks independent semantic corroboration".into());
    }
    let mut reports = BTreeMap::new();
    for path in view
        .source
        .paths(&format!("{}/reports", view.entry.manifest_root))?
    {
        if path.contains("/coverage.") && path.ends_with(".json") {
            reports.insert(path.clone(), view.source.read(&path)?);
        }
    }
    let debt = crate::support_depth::debt::plan_debt_transition(
        record,
        &view.source.read(NON_TUI_SUPPORT_DEBT_PATH)?,
        &reports,
    )?;
    if debt.requires_corroboration() {
        return Err("closeout debt transitions lack independent semantic corroboration".into());
    }
    let verdicts = host.evidence(root, record)?;
    for reference in due(record)? {
        let stated = e
            .results
            .iter()
            .find(|r| r.obligation == reference)
            .ok_or("closeout is missing a due depth obligation")?;
        let operation = record
            .policy
            .selection
            .enrollment
            .operations
            .iter()
            .find(|op| op.id == reference.operation)
            .ok_or("unknown due operation")?;
        let valid = match stated.result {
            DepthResult::Verified => {
                if stated.evidence.is_empty()
                    || verdicts.result(record, &reference) != DepthResult::Verified
                {
                    false
                } else {
                    let mut bound = true;
                    for file in &stated.evidence {
                        bound &= view
                            .source
                            .optional_bytes(&file.path)?
                            .is_some_and(|bytes| {
                                crate::depth_ci::reference(&file.path, &bytes) == *file
                            });
                    }
                    bound
                }
            }
            DepthResult::NotApplicable => reference
                .id
                .is_some_and(|id| operation.not_applicable_permitted.contains(&id)),
            _ => false,
        };
        if !valid {
            return Err(format!(
                "closeout obligation is not independently satisfied: {reference:?}"
            ));
        }
    }
    Ok(())
}
fn due(record: &DepthRecord) -> Result<BTreeSet<ObligationRef>, String> {
    let enrollment = &record.policy.selection.enrollment;
    let o = record.obligations.as_ref().ok_or("missing O")?;
    let mut result = BTreeSet::new();
    for op in &enrollment.operations {
        let stated = o
            .operations
            .iter()
            .find(|s| s.id == op.id)
            .ok_or("O is missing an operation")?;
        if stated.obligations.iter().copied().collect::<BTreeSet<_>>() != op.obligations() {
            return Err("O has an incomplete due obligation set".into());
        }
        for target in enrollment
            .modes
            .iter()
            .filter(|m| op.serves.contains(&m.selector))
            .flat_map(|m| m.targets.as_ref().unwrap_or(&enrollment.targets))
            .collect::<BTreeSet<_>>()
        {
            if !o.targets.contains(target) {
                return Err("O omits a selected target".into());
            }
            for id in op.obligations() {
                result.insert(ObligationRef {
                    operation: op.id.clone(),
                    id: Some(id),
                    capability: None,
                    target: target.clone(),
                });
            }
            for capability in enrollment
                .capabilities
                .iter()
                .filter(|c| c.operations.contains(&op.id))
            {
                result.insert(ObligationRef {
                    operation: op.id.clone(),
                    id: None,
                    capability: Some(capability.id.clone()),
                    target: target.clone(),
                });
            }
        }
    }
    Ok(result)
}

fn output_paths(entry: &crate::agent_registry::AgentRegistryEntry) -> BTreeSet<String> {
    let root = format!("docs/agents/lifecycle/{}-maintenance", entry.agent_id);
    BTreeSet::from([
        format!("{root}/governance/maintenance-closeout.json"),
        format!("{root}/HANDOFF.md"),
        format!("{root}/governance/remediation-log.md"),
        crate::agent_lifecycle::lifecycle_state_path_for_entry(entry),
    ])
}

#[cfg(test)]
mod tests;
