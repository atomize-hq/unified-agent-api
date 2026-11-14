//! Path-neutral adequacy. Owning paths establish their own current bindings and debt policy.
use super::input::View;
use crate::{
    depth_evidence::EvidenceVerdicts,
    depth_record::{due, DepthRecord, DepthResult, ObligationsPart, PolicyPart},
};

pub(super) fn current(
    record: &DepthRecord,
    owner: &str,
    bindings: impl FnOnce() -> Result<(PolicyPart, ObligationsPart), String>,
) -> Result<(), String> {
    let o = record
        .obligations
        .as_ref()
        .ok_or(format!("{owner} requires frozen O"))?;
    let e = record
        .execution
        .as_ref()
        .ok_or(format!("{owner} requires current E"))?;
    if o.policy_identity != record.policy.identity
        || e.policy_identity != record.policy.identity
        || e.obligations_identity != o.identity
    {
        return Err(format!("{owner} P/O/E binding is stale"));
    }
    let (policy, obligations) = bindings()?;
    crate::support_depth::check_current_policy(&record.policy, &policy)?;
    if obligations != *o {
        return Err(format!("{owner} frozen O is no longer current"));
    }
    if !o.classification_required.is_empty() {
        return Err(format!("{owner} has classification_required units"));
    }
    // Current reports can describe names, not independently establish semantic satisfaction or
    // upstream removal. No corroborating producer is installed for these required work items.
    if !o.acceptance_work.is_empty() {
        return Err(format!(
            "{owner} due name-level work lacks independent semantic corroboration"
        ));
    }
    Ok(())
}

pub(super) fn evidence(
    view: &View,
    record: &DepthRecord,
    verdicts: &EvidenceVerdicts,
    owner: &str,
) -> Result<(), String> {
    let e = record
        .execution
        .as_ref()
        .ok_or_else(|| format!("{owner} requires current E"))?;
    for reference in due(record)? {
        let stated = e
            .results
            .iter()
            .find(|r| r.obligation == reference)
            .ok_or_else(|| {
                if owner == "closeout" {
                    "closeout is missing a due depth obligation".into()
                } else {
                    format!("{owner} is missing a due depth obligation: {reference:?}")
                }
            })?;
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
                "{owner} obligation is not independently satisfied: {reference:?}"
            ));
        }
    }
    Ok(())
}
