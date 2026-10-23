//! Opaque read-only results. Neither a digest callback nor a public Verdict constructs these.
use super::*;
use crate::depth_record::{DepthRecord, DepthResult};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct EvidenceVerdicts {
    record_sha256: Option<String>,
    results: BTreeMap<ObligationRef, DepthResult>,
}
impl EvidenceVerdicts {
    pub(crate) fn result(&self, record: &DepthRecord, obligation: &ObligationRef) -> DepthResult {
        if record.stated_sha256().ok().as_ref() != self.record_sha256.as_ref() {
            return DepthResult::Unverified;
        }
        self.results
            .get(obligation)
            .copied()
            .unwrap_or(DepthResult::Unverified)
    }
}

/// Official publication requests production-origin proof. The initial D2 profile is isolated;
/// it cannot authorize production results or turn legacy records into production enrollments.
pub(crate) fn production_results(
    _workspace: &std::path::Path,
    _record: &DepthRecord,
) -> Result<EvidenceVerdicts, String> {
    Ok(EvidenceVerdicts::default())
}

#[cfg(test)]
pub(crate) fn test_results(record: Option<&DepthRecord>) -> EvidenceVerdicts {
    let Some(record) = record else {
        return EvidenceVerdicts::default();
    };
    EvidenceVerdicts {
        record_sha256: record.stated_sha256().ok(),
        results: record
            .execution
            .iter()
            .flat_map(|e| &e.results)
            .map(|r| (r.obligation.clone(), r.result))
            .collect(),
    }
}
