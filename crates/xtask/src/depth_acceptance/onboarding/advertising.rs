//! Onboarding owns raw additions only within its selected capability and target scope.
//! Maintenance's record-scoped effect detectors intentionally retain their original output.
use super::*;
use crate::runtime_advertising::revision::Revision;

pub(super) fn check(
    before: &View,
    after: &View,
    record: &DepthRecord,
    kind: Kind,
) -> Result<(), String> {
    let owns = |agent: &str, capability: &str| {
        if agent != after.entry.agent_id
            || !record
                .policy
                .selection
                .enrollment
                .capabilities
                .iter()
                .any(|c| c.id == capability)
        {
            return Err("onboarding batch has acceptance outside its owned generation".to_owned());
        }
        if kind != Kind::Publication {
            return Err("onboarding Proving forbids capability advertising additions".to_owned());
        }
        Ok(())
    };
    let previous = super::super::publication::cells(before)?;
    for (agent, capability) in super::super::publication::cells(after)?.difference(&previous) {
        owns(agent, capability)?;
    }
    let previous = Revision::with_registry(&before.source, &before.registry)?;
    let current = Revision::with_registry(&after.source, &after.registry)?;
    for addition in current.additions(&previous)? {
        owns(&addition.agent, &addition.capability)?;
        let mut covered = false;
        for target in crate::support_matrix::depth::covered_targets(record) {
            let (os, arch) = super::super::runtime::host(target)?;
            covered |= current.host_cell(&previous, os, arch)
                == (addition.os.as_str(), addition.arch.as_str());
        }
        if !covered {
            return Err("onboarding batch has acceptance outside its owned target".into());
        }
    }
    Ok(())
}
