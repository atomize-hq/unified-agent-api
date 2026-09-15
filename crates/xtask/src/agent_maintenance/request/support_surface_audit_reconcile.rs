use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use crate::agent_registry::AgentRegistryEntry;

use super::super::support_audit::{
    allowed_deferrals, coverage_report_present_for_target, derive_support_surface_audit,
    excluded_surface_kinds, surface_kinds, DebtBackedSurface, DeferredGap, EligibleSurface,
    EvidenceBackedSurface, PublicationImpact, RequiredUplift, SupportSurfaceAudit, SurfaceIdentity,
    UnmatchedDebtSurface, DEBT_OBSERVATIONS, ELIGIBILITY_REASONS, REQUIRED_WRITES,
};
use super::{
    raw::{
        RawDebtBackedSurface, RawDeferredGap, RawEligibleSurface, RawEvidenceBackedSurface,
        RawPublicationImpact, RawRequiredUplift, RawSupportSurfaceAudit, RawSurfaceIdentity,
        RawUnmatchedDebtSurface,
    },
    AuditDriftPolicy, AuditReconciliation, DetectedRelease, MaintenanceRequestError, TriggerKind,
};

pub(super) struct SupportSurfaceAuditValidation {
    pub audit: Option<SupportSurfaceAudit>,
    pub reconciliation: Option<AuditReconciliation>,
    pub reconciliation_detail: Option<String>,
}

struct ReconciledSupportSurfaceAudit {
    reconciliation: AuditReconciliation,
    detail: Option<String>,
}

pub(super) fn validate_support_surface_audit(
    workspace_root: &Path,
    request_path: &Path,
    registry_entry: &AgentRegistryEntry,
    trigger_kind: TriggerKind,
    detected_release: Option<&DetectedRelease>,
    raw: Option<RawSupportSurfaceAudit>,
    audit_drift_policy: AuditDriftPolicy,
) -> Result<SupportSurfaceAuditValidation, MaintenanceRequestError> {
    match (trigger_kind, raw) {
        (TriggerKind::UpstreamReleaseDetected, Some(raw_audit)) => {
            let detected_release = detected_release.ok_or_else(|| {
                MaintenanceRequestError::Internal(format!(
                    "maintenance request `{}` is missing detected_release while validating support_surface_audit",
                    request_path.display()
                ))
            })?;
            let actual = map_raw_support_surface_audit(raw_audit);
            let frozen_had_discovery_work = !actual.discovered_upstream_surface.is_empty()
                || !actual.required_uplifts_this_run.is_empty();
            if !actual.required {
                return Err(MaintenanceRequestError::Validation(format!(
                    "maintenance request `{}` field `support_surface_audit.required` must be `true`",
                    request_path.display()
                )));
            }
            if actual.surface_kinds != surface_kinds() {
                return Err(MaintenanceRequestError::Validation(format!(
                    "maintenance request `{}` field `support_surface_audit.surface_kinds` must match the shared maintenance contract",
                    request_path.display()
                )));
            }
            if actual.excluded_surface_kinds != excluded_surface_kinds() {
                return Err(MaintenanceRequestError::Validation(format!(
                    "maintenance request `{}` field `support_surface_audit.excluded_surface_kinds` must match the shared maintenance contract",
                    request_path.display()
                )));
            }
            if actual.allowed_deferrals != allowed_deferrals() {
                return Err(MaintenanceRequestError::Validation(format!(
                    "maintenance request `{}` field `support_surface_audit.allowed_deferrals` must match the shared maintenance contract",
                    request_path.display()
                )));
            }
            validate_support_surface_audit_row_values(request_path, &actual)?;
            if frozen_had_discovery_work && audit_drift_policy == AuditDriftPolicy::Reject {
                let report_dir = format!(
                    "{}/reports/{}",
                    registry_entry.manifest_root, detected_release.target_version
                );
                match coverage_report_present_for_target(
                    workspace_root,
                    registry_entry,
                    &detected_release.target_version,
                ) {
                    Ok(true) => {}
                    Ok(false) => {
                        return Err(MaintenanceRequestError::Validation(format!(
                            "maintenance request `{}` field `support_surface_audit` cannot confirm reconciliation because live coverage report evidence for target version `{}` is missing under `{}`",
                            request_path.display(),
                            detected_release.target_version,
                            report_dir
                        )));
                    }
                    Err(error) => {
                        return Err(MaintenanceRequestError::Validation(format!(
                            "maintenance request `{}` field `support_surface_audit` cannot confirm reconciliation because live coverage report evidence for target version `{}` under `{}` is unreadable: {}",
                            request_path.display(),
                            detected_release.target_version,
                            report_dir,
                            error
                        )));
                    }
                }
            }
            let expected =
                derive_support_surface_audit(workspace_root, registry_entry, detected_release)
                    .map_err(MaintenanceRequestError::Internal)?;
            let reconciliation = reconcile_support_surface_audit(
                request_path,
                &actual,
                &expected,
                audit_drift_policy,
            )?;
            Ok(SupportSurfaceAuditValidation {
                audit: Some(actual),
                reconciliation: Some(reconciliation.reconciliation),
                reconciliation_detail: reconciliation.detail,
            })
        }
        (TriggerKind::UpstreamReleaseDetected, None) => {
            Err(MaintenanceRequestError::Validation(format!(
                "maintenance request `{}` trigger_kind `upstream_release_detected` requires a `[support_surface_audit]` table",
                request_path.display()
            )))
        }
        (_, Some(_)) => Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` may only include `[support_surface_audit]` when `trigger_kind = \"upstream_release_detected\"`",
            request_path.display()
        ))),
        (_, None) => Ok(SupportSurfaceAuditValidation {
            audit: None,
            reconciliation: None,
            reconciliation_detail: None,
        }),
    }
}

/// Checks the row values the contract enumerates on their own, before reconciliation. Comparing
/// against the live audit cannot stand in for this: a drift-tolerant load accepts any mismatch,
/// and a satisfied reconciliation never reads the frozen uplift or eligible rows.
fn validate_support_surface_audit_row_values(
    request_path: &Path,
    audit: &SupportSurfaceAudit,
) -> Result<(), MaintenanceRequestError> {
    let check = |field: String, value: &str, allowed: &[&str]| {
        if allowed.contains(&value) {
            return Ok(());
        }
        Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `support_surface_audit.{field}` has value `{value}`, which is not one of: {}",
            request_path.display(),
            allowed.join(", ")
        )))
    };
    for (index, row) in audit.unmatched_debt_surface.iter().enumerate() {
        check(
            format!("unmatched_debt_surface[{index}].observation"),
            &row.observation,
            &DEBT_OBSERVATIONS,
        )?;
    }
    for (index, row) in audit.eligible_preexisting_surface.iter().enumerate() {
        check(
            format!("eligible_preexisting_surface[{index}].eligibility_reason"),
            &row.eligibility_reason,
            &ELIGIBILITY_REASONS,
        )?;
    }
    for (index, row) in audit.required_uplifts_this_run.iter().enumerate() {
        for value in &row.required_writes {
            check(
                format!("required_uplifts_this_run[{index}].required_writes"),
                value,
                &REQUIRED_WRITES,
            )?;
        }
    }
    // The header check above has already bound this list to the shared taxonomy.
    let allowed_deferrals = audit
        .allowed_deferrals
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    for (index, row) in audit.deferred_preexisting_gaps.iter().enumerate() {
        check(
            format!("deferred_preexisting_gaps[{index}].defer_reason"),
            &row.defer_reason,
            &allowed_deferrals,
        )?;
    }
    Ok(())
}

fn reconcile_support_surface_audit(
    request_path: &Path,
    frozen: &SupportSurfaceAudit,
    live: &SupportSurfaceAudit,
    audit_drift_policy: AuditDriftPolicy,
) -> Result<ReconciledSupportSurfaceAudit, MaintenanceRequestError> {
    if frozen == live {
        return Ok(ReconciledSupportSurfaceAudit {
            reconciliation: AuditReconciliation::Exact,
            detail: None,
        });
    }
    if support_surface_audit_satisfied(frozen, live) {
        return Ok(ReconciledSupportSurfaceAudit {
            reconciliation: AuditReconciliation::Satisfied,
            detail: None,
        });
    }

    let drift_description = describe_support_surface_audit_drift(frozen, live);
    let validation_message = format!(
        "maintenance request `{}` field `support_surface_audit` no longer matches the live derived maintenance contract: {}",
        request_path.display(),
        drift_description
    );
    match audit_drift_policy {
        AuditDriftPolicy::Reject => Err(MaintenanceRequestError::Validation(validation_message)),
        AuditDriftPolicy::Tolerate => Ok(ReconciledSupportSurfaceAudit {
            reconciliation: AuditReconciliation::Drifted,
            detail: Some(validation_message),
        }),
    }
}

fn support_surface_audit_satisfied(
    frozen: &SupportSurfaceAudit,
    live: &SupportSurfaceAudit,
) -> bool {
    let frozen_had_work = !frozen.discovered_upstream_surface.is_empty()
        || !frozen.required_uplifts_this_run.is_empty();
    if !frozen_had_work {
        return false;
    }
    if !live.discovered_upstream_surface.is_empty() || !live.required_uplifts_this_run.is_empty() {
        return false;
    }
    if live.deferred_preexisting_gaps != frozen.deferred_preexisting_gaps {
        return false;
    }
    if live.pre_run_debt_count != frozen.deferred_preexisting_gaps.len() {
        return false;
    }
    if !live.unmatched_debt_surface.is_empty() {
        return false;
    }

    let live_debt_rows = live
        .preexisting_unsupported_surface
        .iter()
        .map(DebtBackedSurface::identity)
        .collect::<BTreeSet<_>>();
    let frozen_deferred_rows = frozen
        .deferred_preexisting_gaps
        .iter()
        .map(DeferredGap::identity)
        .collect::<BTreeSet<_>>();
    live_debt_rows == frozen_deferred_rows
}

fn describe_support_surface_audit_drift(
    frozen: &SupportSurfaceAudit,
    live: &SupportSurfaceAudit,
) -> String {
    let mut diffs = Vec::new();
    push_scalar_diff(
        &mut diffs,
        "support_surface_audit.pre_run_debt_count",
        frozen.pre_run_debt_count,
        live.pre_run_debt_count,
    );
    push_scalar_diff(
        &mut diffs,
        "support_surface_audit.expected_post_run_debt_count",
        frozen.expected_post_run_debt_count,
        live.expected_post_run_debt_count,
    );
    push_row_diffs(
        &mut diffs,
        "support_surface_audit.discovered_upstream_surface",
        &frozen.discovered_upstream_surface,
        &live.discovered_upstream_surface,
        EvidenceBackedSurface::identity,
        |row| format!("evidence_ref={}", row.evidence_ref),
    );
    push_row_diffs(
        &mut diffs,
        "support_surface_audit.unmatched_debt_surface",
        &frozen.unmatched_debt_surface,
        &live.unmatched_debt_surface,
        UnmatchedDebtSurface::identity,
        |row| format!("debt_ref={}; observation={}", row.debt_ref, row.observation),
    );
    push_row_diffs(
        &mut diffs,
        "support_surface_audit.preexisting_unsupported_surface",
        &frozen.preexisting_unsupported_surface,
        &live.preexisting_unsupported_surface,
        DebtBackedSurface::identity,
        |row| format!("debt_ref={}", row.debt_ref),
    );
    push_row_diffs(
        &mut diffs,
        "support_surface_audit.eligible_preexisting_surface",
        &frozen.eligible_preexisting_surface,
        &live.eligible_preexisting_surface,
        EligibleSurface::identity,
        |row| format!("eligibility_reason={}", row.eligibility_reason),
    );
    push_row_diffs(
        &mut diffs,
        "support_surface_audit.missing_wrapper_support",
        &frozen.missing_wrapper_support,
        &live.missing_wrapper_support,
        Clone::clone,
        |_| String::new(),
    );
    push_row_diffs(
        &mut diffs,
        "support_surface_audit.missing_backend_support",
        &frozen.missing_backend_support,
        &live.missing_backend_support,
        Clone::clone,
        |_| String::new(),
    );
    push_row_diffs(
        &mut diffs,
        "support_surface_audit.required_uplifts_this_run",
        &frozen.required_uplifts_this_run,
        &live.required_uplifts_this_run,
        RequiredUplift::identity,
        |row| {
            format!(
                "reason={}; required_writes={}",
                row.reason,
                row.required_writes.join(",")
            )
        },
    );
    push_row_diffs(
        &mut diffs,
        "support_surface_audit.deferred_preexisting_gaps",
        &frozen.deferred_preexisting_gaps,
        &live.deferred_preexisting_gaps,
        DeferredGap::identity,
        |row| {
            let follow_on = row.blocking_follow_on.as_deref().unwrap_or("none");
            format!(
                "defer_reason={}; blocking_follow_on={follow_on}",
                row.defer_reason
            )
        },
    );
    push_row_diffs(
        &mut diffs,
        "support_surface_audit.publication_impacts",
        &frozen.publication_impacts,
        &live.publication_impacts,
        PublicationImpact::identity,
        |row| format!("surface_doc={}", row.surface_doc),
    );

    if diffs.is_empty() {
        "the frozen audit differs from the live derivation in an unclassified way".to_string()
    } else {
        diffs.join("; ")
    }
}

fn push_scalar_diff<T>(diffs: &mut Vec<String>, field_name: &str, frozen: T, live: T)
where
    T: PartialEq + std::fmt::Display,
{
    if frozen != live {
        diffs.push(format!(
            "{field_name} changed: frozen `{frozen}` vs live `{live}`"
        ));
    }
}

fn push_row_diffs<T, IdentityFn, DetailFn>(
    diffs: &mut Vec<String>,
    field_name: &str,
    frozen: &[T],
    live: &[T],
    identity: IdentityFn,
    detail: DetailFn,
) where
    IdentityFn: Fn(&T) -> SurfaceIdentity,
    DetailFn: Fn(&T) -> String,
{
    let frozen_map = frozen
        .iter()
        .map(|row| (identity(row), detail(row)))
        .collect::<BTreeMap<_, _>>();
    let live_map = live
        .iter()
        .map(|row| (identity(row), detail(row)))
        .collect::<BTreeMap<_, _>>();
    let identities = frozen_map
        .keys()
        .chain(live_map.keys())
        .cloned()
        .collect::<BTreeSet<_>>();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();
    for identity in identities {
        match (frozen_map.get(&identity), live_map.get(&identity)) {
            (Some(frozen_detail), Some(live_detail)) if frozen_detail != live_detail => {
                changed.push(format!(
                    "{} [frozen: {}; live: {}]",
                    identity.describe(),
                    render_detail(frozen_detail),
                    render_detail(live_detail)
                ));
            }
            (Some(frozen_detail), None) => {
                removed.push(render_row(&identity, frozen_detail));
            }
            (None, Some(live_detail)) => {
                added.push(render_row(&identity, live_detail));
            }
            _ => {}
        }
    }

    if !added.is_empty() {
        diffs.push(format!("{field_name} added: {}", added.join(", ")));
    }
    if !removed.is_empty() {
        diffs.push(format!("{field_name} removed: {}", removed.join(", ")));
    }
    if !changed.is_empty() {
        diffs.push(format!("{field_name} changed: {}", changed.join(", ")));
    }
}

fn render_row(identity: &SurfaceIdentity, detail: &str) -> String {
    if detail.is_empty() {
        identity.describe()
    } else {
        format!("{} [{}]", identity.describe(), detail)
    }
}

fn render_detail(detail: &str) -> &str {
    if detail.is_empty() {
        "no additional detail"
    } else {
        detail
    }
}

fn map_raw_support_surface_audit(raw: RawSupportSurfaceAudit) -> SupportSurfaceAudit {
    SupportSurfaceAudit {
        required: raw.required,
        surface_kinds: raw.surface_kinds,
        excluded_surface_kinds: raw.excluded_surface_kinds,
        allowed_deferrals: raw.allowed_deferrals,
        pre_run_debt_count: raw.pre_run_debt_count,
        expected_post_run_debt_count: raw.expected_post_run_debt_count,
        discovered_upstream_surface: raw
            .discovered_upstream_surface
            .into_iter()
            .map(map_raw_evidence_backed_surface)
            .collect(),
        unmatched_debt_surface: raw
            .unmatched_debt_surface
            .into_iter()
            .map(map_raw_unmatched_debt_surface)
            .collect(),
        preexisting_unsupported_surface: raw
            .preexisting_unsupported_surface
            .into_iter()
            .map(map_raw_debt_backed_surface)
            .collect(),
        eligible_preexisting_surface: raw
            .eligible_preexisting_surface
            .into_iter()
            .map(map_raw_eligible_surface)
            .collect(),
        missing_wrapper_support: raw
            .missing_wrapper_support
            .into_iter()
            .map(map_raw_surface_identity)
            .collect(),
        missing_backend_support: raw
            .missing_backend_support
            .into_iter()
            .map(map_raw_surface_identity)
            .collect(),
        required_uplifts_this_run: raw
            .required_uplifts_this_run
            .into_iter()
            .map(map_raw_required_uplift)
            .collect(),
        deferred_preexisting_gaps: raw
            .deferred_preexisting_gaps
            .into_iter()
            .map(map_raw_deferred_gap)
            .collect(),
        publication_impacts: raw
            .publication_impacts
            .into_iter()
            .map(map_raw_publication_impact)
            .collect(),
    }
}

fn map_raw_surface_identity(raw: RawSurfaceIdentity) -> SurfaceIdentity {
    SurfaceIdentity::new(raw.surface_kind, raw.command_path, raw.surface_id)
}

fn map_raw_evidence_backed_surface(raw: RawEvidenceBackedSurface) -> EvidenceBackedSurface {
    EvidenceBackedSurface {
        surface_kind: raw.surface_kind,
        command_path: raw.command_path,
        surface_id: raw.surface_id,
        evidence_ref: raw.evidence_ref,
    }
}

fn map_raw_unmatched_debt_surface(raw: RawUnmatchedDebtSurface) -> UnmatchedDebtSurface {
    UnmatchedDebtSurface {
        surface_kind: raw.surface_kind,
        command_path: raw.command_path,
        surface_id: raw.surface_id,
        debt_ref: raw.debt_ref,
        observation: raw.observation,
    }
}

fn map_raw_debt_backed_surface(raw: RawDebtBackedSurface) -> DebtBackedSurface {
    DebtBackedSurface {
        surface_kind: raw.surface_kind,
        command_path: raw.command_path,
        surface_id: raw.surface_id,
        debt_ref: raw.debt_ref,
    }
}

fn map_raw_eligible_surface(raw: RawEligibleSurface) -> EligibleSurface {
    EligibleSurface {
        surface_kind: raw.surface_kind,
        command_path: raw.command_path,
        surface_id: raw.surface_id,
        eligibility_reason: raw.eligibility_reason,
    }
}

fn map_raw_required_uplift(raw: RawRequiredUplift) -> RequiredUplift {
    RequiredUplift {
        surface_kind: raw.surface_kind,
        command_path: raw.command_path,
        surface_id: raw.surface_id,
        reason: raw.reason,
        required_writes: raw.required_writes,
    }
}

fn map_raw_deferred_gap(raw: RawDeferredGap) -> DeferredGap {
    DeferredGap {
        surface_kind: raw.surface_kind,
        command_path: raw.command_path,
        surface_id: raw.surface_id,
        defer_reason: raw.defer_reason,
        blocking_follow_on: raw.blocking_follow_on,
    }
}

fn map_raw_publication_impact(raw: RawPublicationImpact) -> PublicationImpact {
    PublicationImpact {
        surface_kind: raw.surface_kind,
        command_path: raw.command_path,
        surface_id: raw.surface_id,
        surface_doc: raw.surface_doc,
    }
}
