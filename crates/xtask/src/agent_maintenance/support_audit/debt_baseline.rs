//! Shared request/audit-status debt projection checks. Full inventory semantics belong to the
//! debt planner; a rendered row reference is not a frozen row identity.
use super::SupportSurfaceAudit;

/// Names drift that uplifts cannot explain, which must fail the gate even when uplifts remain. A
/// nightly packet freezes its request before acquisition, so new uplifts always drift it; a debt
/// row that matches no live gap, or a debt baseline that changed since the freeze, never comes
/// from them.
pub(crate) fn debt_baseline_drift(
    frozen: &SupportSurfaceAudit,
    live: &SupportSurfaceAudit,
) -> Option<String> {
    let mut causes = Vec::new();
    if !live.unmatched_debt_surface.is_empty() {
        let rows = live
            .unmatched_debt_surface
            .iter()
            .map(|row| {
                format!(
                    "{} [observation={}; debt_ref={}]",
                    row.identity().describe(),
                    row.observation,
                    row.debt_ref
                )
            })
            .collect::<Vec<_>>();
        causes.push(format!(
            "debt rows match no live gap (contract field invariant 6): {}",
            rows.join(", ")
        ));
    }
    let debt_rows = |audit: &SupportSurfaceAudit| {
        let mut rows = audit
            .preexisting_unsupported_surface
            .iter()
            .map(|row| (row.identity(), row.debt_ref.clone()))
            .collect::<Vec<_>>();
        rows.sort();
        rows
    };
    let deferred_rows = |audit: &SupportSurfaceAudit| {
        let mut rows = audit
            .deferred_preexisting_gaps
            .iter()
            .map(|row| {
                let follow_on = row.blocking_follow_on.clone();
                (row.identity(), row.defer_reason.clone(), follow_on)
            })
            .collect::<Vec<_>>();
        rows.sort();
        rows
    };
    for (field, changed) in [
        (
            "preexisting_unsupported_surface",
            debt_rows(frozen) != debt_rows(live),
        ),
        (
            "deferred_preexisting_gaps",
            deferred_rows(frozen) != deferred_rows(live),
        ),
    ] {
        if changed {
            causes.push(format!(
                "frozen `support_surface_audit.{field}` differs from the live audit"
            ));
        }
    }
    if frozen.pre_run_debt_count != live.pre_run_debt_count {
        causes.push(format!(
            "frozen `support_surface_audit.pre_run_debt_count` {} differs from the live audit's {}",
            frozen.pre_run_debt_count, live.pre_run_debt_count
        ));
    }
    (!causes.is_empty()).then(|| causes.join("; "))
}
