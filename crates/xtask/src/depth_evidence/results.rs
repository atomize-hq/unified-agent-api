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

pub(super) fn evaluated_many(
    record: &DepthRecord,
    results: BTreeMap<ObligationRef, DepthResult>,
) -> Result<EvidenceVerdicts, String> {
    Ok(EvidenceVerdicts {
        record_sha256: Some(record.stated_sha256()?),
        results,
    })
}
/// Authenticate and evaluate only the compiled isolated Codex CORE profile; no production origin.
pub fn evaluate_isolated_record(
    workspace: &std::path::Path,
    record: &DepthRecord,
) -> Result<EvidenceVerdicts, String> {
    super::verdict::isolated_record_results(workspace, record)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opaque_results_do_not_survive_a_changed_record() {
        let mut record = crate::depth_ci::profile::Profile::Core
            .record(include_str!("../../tests/fixtures/depth-core/record.json"))
            .unwrap();
        let obligation = crate::depth_ci::profile::Profile::Core
            .obligations(&record)
            .remove(0);
        let results = evaluated_many(
            &record,
            BTreeMap::from([(obligation.clone(), DepthResult::Verified)]),
        )
        .unwrap();
        assert_eq!(results.result(&record, &obligation), DepthResult::Verified);
        record.policy.path_enabled = false;
        assert_eq!(
            results.result(&record, &obligation),
            DepthResult::Unverified
        );
    }
}
