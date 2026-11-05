//! OF-P6 read-only refusal. A match grants no publication or acceptance authority.
use std::{collections::BTreeMap, path::Path};

use sha2::{Digest, Sha256};

use crate::{
    agent_lifecycle::{lifecycle_state_path, LifecycleState},
    depth_record::{
        onboarding::{onboarding_binding, OnboardingApproval},
        DepthRecord, Event,
    },
    runtime_advertising::source::Source,
};

/// Compare exactly the acquired paths and SHA-256 identities frozen by the shared O derivation,
/// as MA4's relay does. This neither derives a new identity nor writes/refreezes anything.
pub fn check(workspace: &Path, approval_path: &str) -> Result<(), String> {
    let source = Source::workspace(workspace)?;
    let approval = OnboardingApproval::parse(&source.read(approval_path)?, approval_path)
        .map_err(|err| format!("parse approval artifact `{approval_path}`: {err}"))?;
    let state_path = lifecycle_state_path(&approval.onboarding_pack_prefix);
    let state = source
        .optional(&state_path)?
        .map(|raw| serde_json::from_str::<LifecycleState>(&raw))
        .transpose()
        .map_err(|e| format!("{state_path}: {e}"))?;
    let object = state.as_ref().and_then(|s| s.support_depth.as_deref());
    let reports = format!("{}/reports", approval.manifest_root);
    let mut records = BTreeMap::new();
    for path in source.paths(&reports)? {
        if !path.ends_with("/depth-record.json") {
            continue;
        }
        let version = path
            .strip_prefix(&format!("{reports}/"))
            .and_then(|p| p.strip_suffix("/depth-record.json"))
            .ok_or_else(|| format!("invalid depth record path: {path}"))?;
        let record = DepthRecord::parse(&source.read(&path)?, &approval.agent_id, version)
            .map_err(|e| format!("{path}: {e}"))?;
        if matches!(record.event, Event::Onboarding { .. }) {
            records.insert(version.to_owned(), record);
        }
    }
    if object.is_none() && records.is_empty() {
        return Ok(());
    }
    onboarding_binding(object, &approval, &records).map_err(|faults| {
        format!(
            "{state_path}: {}",
            faults
                .iter()
                .map(|f| format!("{}: {}", f.code, f.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    })?;
    let mut changed = Vec::new();
    for record in records.values() {
        if let Some(o) = &record.obligations {
            if o.policy_identity != record.policy.identity {
                return Err("frozen acquisition O/P binding is stale".into());
            }
            for file in &o.acquired_inputs {
                let bytes = source.optional_bytes(&file.path)?;
                if bytes.map_or(true, |bytes| {
                    hex::encode(Sha256::digest(bytes)) != file.sha256
                }) {
                    changed.push(file.path.clone());
                }
            }
        }
    }
    changed.sort();
    changed.dedup();
    if !changed.is_empty() {
        return Err(format!(
            "changed or missing frozen acquired inputs; publication refused: {}",
            changed.join(", ")
        ));
    }
    Ok(())
}
