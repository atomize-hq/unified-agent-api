//! Relay-specific limits around the existing effect detector. No returned authority receipt.
use super::*;
use crate::{
    agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH},
    depth_record::DepthRecord,
};

pub(super) fn check(before: Source, after: Source, workspace: &Path) -> Result<(), String> {
    let raw = before.read(REGISTRY_RELATIVE_PATH)?;
    if after.read(REGISTRY_RELATIVE_PATH)? != raw {
        return Err("relay cannot change registry authority".into());
    }
    let registry = AgentRegistry::parse_unenforced(&raw).map_err(|e| e.to_string())?;
    for entry in &registry.agents {
        let old = View::with_registry(before.clone(), &registry, &entry.agent_id)?;
        let new = View::with_registry(after.clone(), &registry, &entry.agent_id)?;
        if old.records.keys().collect::<Vec<_>>() != new.records.keys().collect::<Vec<_>>() {
            return Err("relay cannot add/remove a depth generation".into());
        }
        for (version, record) in &new.records {
            preserved(&old.records[version], record)?;
            if old
                .source
                .optional(crate::agent_maintenance::support_audit::NON_TUI_SUPPORT_DEBT_PATH)?
                != new
                    .source
                    .optional(crate::agent_maintenance::support_audit::NON_TUI_SUPPORT_DEBT_PATH)?
            {
                return Err(
                    "relay debt edit requires independently corroborated transition".into(),
                );
            }
            if let Some(o) = &record.obligations {
                if o.policy_identity != record.policy.identity {
                    return Err("relay O/P binding is stale".into());
                }
                for file in &o.acquired_inputs {
                    use sha2::{Digest, Sha256};
                    let bytes = new
                        .source
                        .optional_bytes(&file.path)?
                        .ok_or("missing frozen acquisition input")?;
                    if hex::encode(Sha256::digest(bytes)) != file.sha256 {
                        return Err(
                            "relay changed frozen acquisition; explicit re-freeze required".into(),
                        );
                    }
                }
            }
        }
        if !new.records.is_empty() && !authorized_debt(&new)?.is_subset(&authorized_debt(&old)?) {
            return Err(
                "new authorized debt requires independently corroborated transition".into(),
            );
        }
        let comparison = compare_views(&old, &new)?;
        if !comparison.effects.is_empty() {
            return Err(format!(
                "relay acceptance effects forbidden: {:?}",
                comparison.effects
            ));
        }
        if !new.records.is_empty() {
            publication::validate_current(workspace, &new)?;
        }
    }
    Ok(())
}
fn preserved(old: &DepthRecord, new: &DepthRecord) -> Result<(), String> {
    if old.event != new.event
        || old.policy != new.policy
        || old.obligations != new.obligations
        || old.acceptance_entries != new.acceptance_entries
    {
        return Err("relay must preserve Event/P/O/acceptance entries".into());
    }
    if let Some(e) = &new.execution {
        let o = new
            .obligations
            .as_ref()
            .ok_or("relay E requires frozen O")?;
        if e.policy_identity != new.policy.identity || e.obligations_identity != o.identity {
            return Err("relay E/P/O binding is stale".into());
        }
    }
    let debts = |r: &DepthRecord| {
        r.execution
            .as_ref()
            .map(|e| (e.debt_grants.clone(), e.debt_retired.clone()))
            .unwrap_or_default()
    };
    if debts(old) != debts(new) {
        return Err("conditional debt claims do not establish relay debt authority".into());
    }
    Ok(())
}

fn authorized_debt(view: &View) -> Result<BTreeSet<(String, String, String, String)>, String> {
    let artifact: crate::support_matrix::SupportMatrixArtifact =
        serde_json::from_value(view.json(crate::support_matrix::JSON_OUTPUT_PATH)?)
            .map_err(|e| e.to_string())?;
    let mut result = BTreeSet::new();
    for row in artifact.rows {
        if let Some(depth) = row.depth {
            for promise in depth.promises {
                if promise
                    .outcomes
                    .contains(&crate::support_matrix::depth::Outcome::AuthorizedDebt)
                {
                    result.insert((
                        row.agent.clone(),
                        row.version.clone(),
                        row.target.clone(),
                        promise.operation,
                    ));
                }
            }
        }
    }
    Ok(result)
}
