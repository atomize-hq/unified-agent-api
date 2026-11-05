//! OF-P6 read-only refusal. A match grants no publication or acceptance authority.
use std::{collections::BTreeMap, fs, path::Path};

use sha2::{Digest, Sha256};

use crate::{
    agent_lifecycle::{lifecycle_state_path, LifecycleState},
    depth_record::{
        onboarding::{onboarding_binding, OnboardingApproval},
        record_files, DepthRecord, Event,
    },
    runtime_advertising::source::Source,
};

/// Compare exactly the acquired paths and SHA-256 identities frozen by the shared O derivation,
/// as MA4's relay does. This neither derives a new identity nor writes/refreezes anything.
pub fn check(workspace: &Path, approval_path: &str) -> Result<(), String> {
    if !has_depth_bindings(workspace, approval_path)? {
        return Ok(());
    }
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

/// Detect enrollment with ordinary filesystem reads, preserving the approval loader's
/// in-workspace symlink semantics. Source's stricter rules apply only after enrollment.
fn has_depth_bindings(workspace: &Path, approval_path: &str) -> Result<bool, String> {
    let workspace = workspace.canonicalize().map_err(|e| e.to_string())?;
    let approval_file = workspace
        .join(approval_path)
        .canonicalize()
        .map_err(|e| format!("approval artifact `{approval_path}` does not resolve: {e}"))?;
    if !approval_file.starts_with(&workspace) {
        return Err(format!(
            "approval artifact `{approval_path}` resolves outside workspace root"
        ));
    }
    let raw = fs::read_to_string(approval_file).map_err(|e| e.to_string())?;
    let approval = OnboardingApproval::parse(&raw, approval_path)
        .map_err(|e| format!("parse approval artifact `{approval_path}`: {e}"))?;
    let state_path = lifecycle_state_path(&approval.onboarding_pack_prefix);
    match fs::read(workspace.join(&state_path)) {
        Ok(bytes) => {
            let state: LifecycleState =
                serde_json::from_slice(&bytes).map_err(|e| format!("{state_path}: {e}"))?;
            if state.support_depth.is_some() {
                return Ok(true);
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(format!("{state_path}: {e}")),
    }
    for (version, path) in record_files(&workspace.join(&approval.manifest_root)) {
        let raw = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let record = DepthRecord::parse(&raw, &approval.agent_id, &version)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        if matches!(record.event, Event::Onboarding { .. }) {
            return Ok(true);
        }
    }
    Ok(false)
}
