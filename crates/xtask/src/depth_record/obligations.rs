//! Shared complete-due selection used by admission and manual acceptance.
use super::{DepthRecord, ObligationRef};
use std::collections::BTreeSet;
pub(crate) fn due(record: &DepthRecord) -> Result<BTreeSet<ObligationRef>, String> {
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
