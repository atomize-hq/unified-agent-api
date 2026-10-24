//! Final integration reuses owning predicates; entries and route labels grant no authority.
use super::{input::View, manual, promotion};
use crate::{
    agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH},
    depth_record::AcceptanceEntry,
    runtime_advertising::source::Source,
};
use std::{collections::BTreeSet, path::Path};

mod generation;

pub(super) fn check(root: &Path, revision: &str) -> Result<(), String> {
    check_with(root, revision, &manual::Production, &promotion::Production)
}

pub(super) fn check_with(
    root: &Path,
    revision: &str,
    closeout_host: &impl manual::Host,
    promotion_host: &impl manual::Host,
) -> Result<(), String> {
    let tip = resolve(root, revision)?;
    let head = resolve(root, "HEAD")?;
    let before = Source::tip(root, &tip)?;
    let after = Source::workspace(root)?;
    let old_registry = AgentRegistry::parse_unenforced(&before.read(REGISTRY_RELATIVE_PATH)?)
        .map_err(|e| e.to_string())?;
    let registry = AgentRegistry::parse_unenforced(&after.read(REGISTRY_RELATIVE_PATH)?)
        .map_err(|e| e.to_string())?;
    let agents: BTreeSet<_> = old_registry
        .agents
        .iter()
        .chain(&registry.agents)
        .map(|e| e.agent_id.as_str())
        .collect();
    let mut publication = None;
    for agent in agents {
        let old = View::with_registry(
            before.clone(),
            if old_registry.find(agent).is_some() {
                &old_registry
            } else {
                &registry
            },
            agent,
        )?;
        let new = View::with_registry(
            after.clone(),
            if registry.find(agent).is_some() {
                &registry
            } else {
                &old_registry
            },
            agent,
        )?;
        generation::check(root, &old, &new).map_err(|e| format!("{agent}: {e}"))?;
        if old.records.is_empty() && new.records.is_empty() {
            continue;
        }
        let declaration = new.json(&format!("{}/RULES.json", new.entry.manifest_root))?;
        let expected = format!(
            "{}/reports/<version>/depth-record.json",
            new.entry.manifest_root
        );
        if declaration
            .pointer("/storage/committed/reports/depth_record")
            .and_then(serde_json::Value::as_str)
            != Some(expected.as_str())
        {
            return Err(format!(
                "{agent}: retained root does not adopt the canonical depth record path"
            ));
        }
        if registry.find(agent).is_none() {
            return Err("retained agent lost its registry anchor".into());
        }
        admit(root, &old, &new, &before, closeout_host, promotion_host)
            .map_err(|e| format!("{agent}: {e}"))?;
        publication = Some(new);
    }
    if let Some(view) = publication {
        super::publication::validate_current(root, &view)?;
    }
    if resolve(root, revision)? != tip || resolve(root, "HEAD")? != head {
        return Err("integration revision moved during admission".into());
    }
    Ok(())
}

fn resolve(root: &Path, revision: &str) -> Result<String, String> {
    let out = crate::git_inputs::command(root)
        .args([
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{revision}^{{commit}}"),
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err("unreadable integration revision".into());
    }
    String::from_utf8(out.stdout)
        .map(|s| s.trim().into())
        .map_err(|e| e.to_string())
}

pub(super) fn admit(
    root: &Path,
    old: &View,
    new: &View,
    tip: &Source,
    closeout_host: &impl manual::Host,
    promotion_host: &impl manual::Host,
) -> Result<(), String> {
    let comparison = super::compare_views(old, new)?;
    let qualifications = super::publication::qualifications(new)?;
    for (version, prior) in &old.records {
        let record = new
            .records
            .get(version)
            .ok_or("integration removed a retained record")?;
        if !record
            .acceptance_entries
            .starts_with(&prior.acceptance_entries)
            || !record.lacking_from(prior).is_empty()
        {
            return Err("integration changed retained entries or shrank selection".into());
        }
    }
    for (version, record) in &new.records {
        let digest = record.stated_sha256()?;
        // Dependency-only changes can invalidate a still-published qualification without
        // changing the record, its entries or any newly accepting value.
        if qualifications
            .iter()
            .any(|(agent, v, _, _)| agent == &new.entry.agent_id && v == version)
        {
            if record.policy.selection.enrollment.lifecycle_path
                != crate::agent_registry::support_depth::LifecyclePath::Maintenance
                || !record.policy.path_enabled
            {
                return Err("qualified publication lacks maintenance lifecycle authority".into());
            }
            manual::admitted(root, new, record, promotion_host)?;
            if !record
                .acceptance_entries
                .iter()
                .any(|e| e.record_sha256() == digest)
            {
                return Err("qualified publication has no current entry".into());
            }
        }
        let previous = old
            .records
            .get(version)
            .map(|r| r.acceptance_entries.as_slice())
            .unwrap_or(&[]);
        let added = &record.acceptance_entries[previous.len()..];
        let effects: Vec<_> = comparison
            .effects
            .iter()
            .filter(|e| e.version == *version)
            .collect();
        if added.is_empty() && effects.is_empty() {
            continue;
        }
        if added.iter().any(|e| e.record_sha256() != digest) {
            return Err("new acceptance entry must describe the current record".into());
        }
        let closeout_path = format!(
            "docs/agents/lifecycle/{}-maintenance/governance/maintenance-closeout.json",
            new.entry.agent_id
        );
        let mut closeout = added
            .iter()
            .any(|e| matches!(e, AcceptanceEntry::MaintenanceCloseout { .. }));
        let mut promote = added
            .iter()
            .any(|e| matches!(e, AcceptanceEntry::Promotion { .. }));
        for e in &effects {
            match e.kind.as_str() {
                "published" | "closed_baseline" | "proving_run_closeout_written" => {
                    return Err("unsupported onboarding lifecycle authority at integration".into());
                }
                "closeout semantic change"
                | "maintenance_closeout_written"
                | "absence_of_drifted" => {
                    if e.path != closeout_path {
                        return Err("wrong lifecycle closeout authority".into());
                    }
                    closeout = true;
                }
                _ => promote = true,
            }
        }
        let manual_entry = if closeout {
            let entry = manual::acceptance(root, new, record, tip, closeout_host)?;
            if !record.acceptance_entries.contains(&entry) {
                return Err("integration closeout lacks its exact current entry".into());
            }
            Some(entry)
        } else {
            None
        };
        let promotion_entry = if promote {
            // Only an already admitted tip entry or this candidate's independently admitted
            // closeout can supply retained authority. A fabricated locator is not a closeout.
            let retained = record.acceptance_entries.iter().any(|e| matches!(e,
                AcceptanceEntry::MaintenanceCloseout { record_sha256, .. }
                if *record_sha256 == digest && (previous.contains(e) || manual_entry.as_ref() == Some(e))));
            if !retained {
                return Err("promotion has no admitted retained closeout".into());
            }
            let entry =
                promotion::acceptance(root, new, record, tip, &comparison.effects, promotion_host)?;
            if !record.acceptance_entries.contains(&entry) {
                return Err("integration promotion lacks its own current target entry".into());
            }
            Some(entry)
        } else {
            None
        };
        for entry in added {
            if manual_entry.as_ref() != Some(entry) && promotion_entry.as_ref() != Some(entry) {
                return Err("new acceptance entry has no supported owning authority".into());
            }
        }
    }
    Ok(())
}
