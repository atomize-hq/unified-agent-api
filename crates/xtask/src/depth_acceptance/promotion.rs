//! Maintenance promotion authority, including retained closeouts whose working files moved on.
use super::{
    artifacts,
    input::{text, View},
    manual,
};
use crate::{
    agent_registry::{support_depth::LifecyclePath, AgentRegistry, REGISTRY_RELATIVE_PATH},
    depth_evidence::EvidenceVerdicts,
    depth_record::{AcceptanceEntry, DepthRecord, Event, ObligationsPart, PolicyPart},
    runtime_advertising::source::Source,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
pub(super) struct Production;
impl manual::Host for Production {
    fn bindings(
        &self,
        root: &Path,
        record: &DepthRecord,
    ) -> Result<(PolicyPart, ObligationsPart), String> {
        crate::support_depth::current_promotion_bindings(root, record)
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
pub(super) fn plan_with(
    root: &Path,
    before: Source,
    after: Source,
    written: &BTreeSet<String>,
    host: &impl manual::Host,
) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let raw = before.read(REGISTRY_RELATIVE_PATH)?;
    if after.read(REGISTRY_RELATIVE_PATH)? != raw {
        return Err("promotion cannot change registry authority".into());
    }
    let registry = AgentRegistry::parse_unenforced(&raw).map_err(|e| e.to_string())?;
    let mut outputs = BTreeMap::new();
    let tip = integration_tip(root)?;
    for entry in &registry.agents {
        let old = View::with_registry(before.clone(), &registry, &entry.agent_id)?;
        let new = View::with_registry(after.clone(), &registry, &entry.agent_id)?;
        if old.records != new.records {
            return Err(
                "promotion must preserve retained records and entries before admission".into(),
            );
        }
        for declaration in entry
            .support_depth
            .as_ref()
            .and_then(|d| d.enrollments.as_ref())
            .into_iter()
            .flatten()
        {
            if !new.records.contains_key(&declaration.version) {
                return Err("promotion declared destination has no depth record".into());
            }
        }
        let comparison = super::compare_views(&old, &new)?;
        let mut accepting: BTreeSet<_> = comparison
            .effects
            .iter()
            .map(|e| e.version.clone())
            .collect();
        for version in new.records.keys() {
            if written.contains(&format!("{}/versions/{version}.json", entry.manifest_root)) {
                accepting.insert(version.clone());
            }
        }
        for version in accepting {
            let record = new
                .records
                .get(&version)
                .ok_or("promotion has no destination depth record")?;
            let own = acceptance(
                root,
                &new,
                record,
                &Source::tip(root, &tip)?,
                &comparison.effects,
                host,
            )?;
            let mut accepted = record.clone();
            if !accepted.acceptance_entries.contains(&own) {
                accepted.acceptance_entries.push(own);
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
    if integration_tip(root)? != tip {
        return Err("integration tip moved during promotion admission".into());
    }
    Ok(outputs)
}
/// Same predicate and target attribution for local promotion and exact-result integration.
pub(super) fn acceptance(
    root: &Path,
    view: &View,
    record: &DepthRecord,
    tip: &Source,
    effects: &[super::Effect],
    host: &impl manual::Host,
) -> Result<AcceptanceEntry, String> {
    let version = record.policy.selection.enrollment.version.clone();
    if record.policy.selection.enrollment.lifecycle_path != LifecyclePath::Maintenance
        || !record.policy.path_enabled
    {
        return Err("promotion has no maintenance lifecycle-path authority".into());
    }
    let digest = record.stated_sha256()?;
    let closeout = record
        .acceptance_entries
        .iter()
        .find_map(|e| match e {
            AcceptanceEntry::MaintenanceCloseout {
                closeout,
                record_sha256,
            } if *record_sha256 == digest => Some(closeout),
            _ => None,
        })
        .ok_or("promotion requires a current retained maintenance closeout entry")?;
    let expected = format!(
        "docs/agents/lifecycle/{}-maintenance/governance/maintenance-closeout.json",
        view.entry.agent_id
    );
    if closeout.path != expected {
        return Err("promotion closeout entry has wrong agent/path".into());
    }
    manual::admitted(root, view, record, host)?;
    let marker = crate::agent_maintenance::stand_down::marker_relative_path(
        &format!("docs/agents/lifecycle/{}-maintenance", view.entry.agent_id),
        &version,
    );
    if let Some(raw) = tip.optional(&marker)? {
        let Event::Maintenance {
            request_recorded_at,
            ..
        } = &record.event
        else {
            return Err("promotion is not a maintenance generation".into());
        };
        // Timestamp markers survive working-file replacement. A hash-only marker still
        // requires its exact retained closeout bytes; an unrelated newer closeout cannot bind it.
        let request_hash = match view.source.optional(&closeout.path)? {
            Some(raw) if artifacts::digest(&raw) == closeout.sha256 => {
                text(&super::input::json(&raw)?, "request_sha256")?.to_owned()
            }
            _ => String::new(),
        };
        crate::agent_maintenance::stand_down::require_generation_marker(
            &view.entry.agent_id,
            &version,
            &raw,
            request_recorded_at,
            &request_hash,
        )?;
        if view.source.optional(&marker)?.is_some() {
            return Err("promotion must retire its integration marker in the same proposal".into());
        }
    }
    let mut targets = BTreeSet::new();
    for effect in effects.iter().filter(|e| e.version == version) {
        if let Some(t) = &effect.target {
            targets.insert(t.clone());
        }
    }
    let selected = crate::support_matrix::depth::covered_targets(record);
    if targets.is_empty() {
        targets.extend(selected.into_iter().map(str::to_owned));
    }
    Ok(AcceptanceEntry::Promotion {
        targets: targets.into_iter().collect(),
        record_sha256: digest,
    })
}

fn integration_tip(root: &Path) -> Result<String, String> {
    let output = crate::git_inputs::command(root)
        .args(["rev-parse", "--verify", "origin/staging^{commit}"])
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("cannot resolve integration tip for promotion".into());
    }
    String::from_utf8(output.stdout)
        .map(|s| s.trim().into())
        .map_err(|e| e.to_string())
}
