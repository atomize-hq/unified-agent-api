//! Reporting E and its referenced receipt/payload files, assembled from an authenticated
//! population. Shared by the diagnostic scratch-clone proof and the MA4 staged bundle (B.4 part 2).
//! The result is an attributed claim: every consumer re-authenticates the full population.
use super::{profile, receipt_ref, Resolved};
use crate::depth_ci::profile::Profile;
use crate::depth_record::{DepthResult, ExecutionPart, FileRef, ObligationRef, ObligationResult};
use std::collections::BTreeMap;

pub(crate) struct Assembly {
    pub profile: Profile,
    /// The frozen generation (Event/P/O) the population was authenticated for.
    pub original: crate::depth_record::DepthRecord,
    pub execution: ExecutionPart,
    /// Every referenced receipt and payload, once each, in path order.
    pub files: Vec<(FileRef, Vec<u8>)>,
    pub selected: BTreeMap<ObligationRef, FileRef>,
}

pub(super) fn assemble(resolved: &Resolved) -> Result<Assembly, String> {
    let p = resolved.profile;
    let mut selected = BTreeMap::new();
    let mut files: BTreeMap<String, (FileRef, Vec<u8>)> = BTreeMap::new();
    for run in &resolved.runs {
        let r = receipt_ref(&run.receipt)?;
        selected
            .entry(run.receipt.scope.obligation.clone())
            .or_insert(r.clone());
        let mut written = profile::payloads(&run.witness)?;
        written.push((
            r,
            serde_json::to_vec(&run.receipt).map_err(|e| e.to_string())?,
        ));
        for (reference, bytes) in written {
            crate::depth_evidence::validate::file(&reference)?;
            if crate::depth_ci::reference(&reference.path, &bytes) != reference {
                return Err("hosted payload digest mismatch".into());
            }
            if !reference.path.starts_with(&format!("{}/", p.evidence())) {
                return Err("hosted payload outside the profile evidence directory".into());
            }
            match files.get(&reference.path) {
                Some((_, old)) if *old != bytes => {
                    return Err("conflicting hosted payload path".into())
                }
                Some(_) => {}
                None => {
                    files.insert(reference.path.clone(), (reference, bytes));
                }
            }
        }
    }
    let record = &resolved.original;
    let scope = profile::scope_for(record, p, p.obligations(record).remove(0));
    let mut execution = ExecutionPart {
        identity: String::new(),
        policy_identity: scope.policy_identity.clone(),
        obligations_identity: scope.obligations_identity.clone(),
        results: selected
            .iter()
            .map(|(obligation, evidence)| ObligationResult {
                obligation: obligation.clone(),
                result: DepthResult::Verified,
                rationale: None,
                evidence: vec![evidence.clone()],
            })
            .collect(),
        reuse_bindings: vec![],
        debt_grants: vec![],
        debt_retired: vec![],
    };
    // Annex B derived arrays sort by canonical serialized entries, not Rust enum order.
    execution
        .results
        .sort_by_key(|r| serde_json::to_vec(&serde_json::to_value(r).unwrap()).unwrap());
    execution.identity = crate::depth_record::part_identity(&execution)?;
    Ok(Assembly {
        profile: p,
        original: record.clone(),
        execution,
        files: files.into_values().collect(),
        selected,
    })
}
