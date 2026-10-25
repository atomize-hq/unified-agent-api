//! Opaque read-only results. Neither a digest callback nor a public Verdict constructs these.
use super::*;
use crate::depth_record::{DepthRecord, DepthResult};
use std::collections::BTreeMap;

// This private in-memory seal includes schema version and acceptance entries. It is separate
// from the acceptance digest, whose deliberate exclusion of entries prevents self-reference.
fn seal(record: &DepthRecord) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let bytes = serde_json::to_vec(record).map_err(|e| e.to_string())?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

#[derive(Debug, Clone, Default)]
pub struct EvidenceVerdicts {
    record_seal: Option<String>,
    results: BTreeMap<ObligationRef, DepthResult>,
}
impl EvidenceVerdicts {
    pub(crate) fn result(&self, record: &DepthRecord, obligation: &ObligationRef) -> DepthResult {
        if seal(record).ok().as_ref() != self.record_seal.as_ref() {
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
        record_seal: seal(record).ok(),
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
        record_seal: Some(seal(record)?),
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
    #[test]
    fn opaque_results_do_not_survive_entry_append_remove_or_reorder() {
        use crate::depth_record::AcceptanceEntry;
        let mut record = crate::depth_ci::profile::Profile::Core
            .record(include_str!("../../tests/fixtures/depth-core/record.json"))
            .unwrap();
        let obligation = crate::depth_ci::profile::Profile::Core
            .obligations(&record)
            .remove(0);
        let first = AcceptanceEntry::Promotion {
            targets: vec!["first".into()],
            record_sha256: record.stated_sha256().unwrap(),
        };
        let second = AcceptanceEntry::Promotion {
            targets: vec!["second".into()],
            record_sha256: record.stated_sha256().unwrap(),
        };
        record.acceptance_entries = vec![first, second];
        for test_constructor in [false, true] {
            if test_constructor {
                // The private acceptance-test seam seals the same complete record.
                let raw = serde_json::to_value(&record).unwrap();
                record.execution = Some(
                    serde_json::from_value(serde_json::json!({
                        "identity": "a".repeat(64), "policy_identity": raw["policy"]["identity"],
                        "obligations_identity": raw["obligations"]["identity"],
                        "results": [{"obligation": obligation, "result":"verified", "evidence":[]}],
                        "reuse_bindings":[], "debt_grants":[], "debt_retired":[]
                    }))
                    .unwrap(),
                );
            }
            let results = if test_constructor {
                test_results(Some(&record))
            } else {
                evaluated_many(
                    &record,
                    BTreeMap::from([(obligation.clone(), DepthResult::Verified)]),
                )
                .unwrap()
            };
            assert_eq!(results.result(&record, &obligation), DepthResult::Verified);
            for mutation in 0..3 {
                let mut changed = record.clone();
                match mutation {
                    0 => changed
                        .acceptance_entries
                        .push(changed.acceptance_entries[0].clone()),
                    1 => {
                        changed.acceptance_entries.pop();
                    }
                    _ => changed.acceptance_entries.swap(0, 1),
                }
                assert_eq!(
                    record.stated_sha256().unwrap(),
                    changed.stated_sha256().unwrap()
                );
                assert_eq!(
                    results.result(&changed, &obligation),
                    DepthResult::Unverified,
                    "constructor {test_constructor}, mutation {mutation}"
                );
            }
        }
    }
}
