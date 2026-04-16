use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use super::derive::{
    build_evidence_notes, classify_pointer_promotion, derive_rows_for_loaded_roots,
    load_agent_root, load_support_report, AgentRoot, LoadedAgentRoot,
};
use super::{
    BackendSupportState, ManifestSupportState, PointerPromotionState, SupportRow, UaaSupportState,
    CURRENT_AGENT_ROOTS,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SupportRowIdentity {
    agent: String,
    version: String,
    target: String,
}

impl SupportRowIdentity {
    fn from_row(row: &SupportRow) -> Self {
        Self {
            agent: row.agent.clone(),
            version: row.version.clone(),
            target: row.target.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SupportMatrixConsistencyIssue {
    pub code: &'static str,
    pub agent: String,
    pub version: String,
    pub target: String,
    pub message: String,
}

impl SupportMatrixConsistencyIssue {
    fn missing_row(identity: &SupportRowIdentity) -> Self {
        Self {
            code: "SUPPORT_MATRIX_ROW_MISSING",
            agent: identity.agent.clone(),
            version: identity.version.clone(),
            target: identity.target.clone(),
            message: "publication is missing a committed (agent, version, target) row".to_string(),
        }
    }

    fn unexpected_row(identity: &SupportRowIdentity) -> Self {
        Self {
            code: "SUPPORT_MATRIX_ROW_UNEXPECTED",
            agent: identity.agent.clone(),
            version: identity.version.clone(),
            target: identity.target.clone(),
            message: "publication contains a row not implied by committed manifest metadata"
                .to_string(),
        }
    }

    fn duplicate_row(identity: &SupportRowIdentity, count: usize) -> Self {
        Self {
            code: "SUPPORT_MATRIX_ROW_DUPLICATE",
            agent: identity.agent.clone(),
            version: identity.version.clone(),
            target: identity.target.clone(),
            message: format!(
                "publication contains {count} rows for the same committed (agent, version, target) tuple"
            ),
        }
    }

    fn pointer_promotion_mismatch(row: &SupportRow, observed: PointerPromotionState) -> Self {
        Self {
            code: "SUPPORT_MATRIX_POINTER_PROMOTION_MISMATCH",
            agent: row.agent.clone(),
            version: row.version.clone(),
            target: row.target.clone(),
            message: format!(
                "row pointer_promotion={} does not match committed pointer state {}",
                row.pointer_promotion.as_str(),
                observed.as_str()
            ),
        }
    }

    fn omission_contradiction(row: &SupportRow, expected_notes: &[String]) -> Self {
        Self {
            code: "SUPPORT_MATRIX_CURRENT_SNAPSHOT_OMISSION_MISMATCH",
            agent: row.agent.clone(),
            version: row.version.clone(),
            target: row.target.clone(),
            message: format!(
                "row claims support truth incompatible with current snapshot omission evidence; expected notes {:?}",
                expected_notes
            ),
        }
    }

    fn evidence_notes_mismatch(row: &SupportRow, expected_notes: &[String]) -> Self {
        Self {
            code: "SUPPORT_MATRIX_EVIDENCE_NOTES_MISMATCH",
            agent: row.agent.clone(),
            version: row.version.clone(),
            target: row.target.clone(),
            message: format!(
                "row evidence_notes {:?} do not match committed evidence {:?}",
                row.evidence_notes, expected_notes
            ),
        }
    }

    fn manifest_support_mismatch(row: &SupportRow, expected: ManifestSupportState) -> Self {
        Self {
            code: "SUPPORT_MATRIX_MANIFEST_SUPPORT_MISMATCH",
            agent: row.agent.clone(),
            version: row.version.clone(),
            target: row.target.clone(),
            message: format!(
                "row manifest_support={} does not match committed support state {}",
                row.manifest_support.as_str(),
                expected.as_str()
            ),
        }
    }

    fn backend_support_mismatch(row: &SupportRow, expected: BackendSupportState) -> Self {
        Self {
            code: "SUPPORT_MATRIX_BACKEND_SUPPORT_MISMATCH",
            agent: row.agent.clone(),
            version: row.version.clone(),
            target: row.target.clone(),
            message: format!(
                "row backend_support={} does not match committed backend state {}",
                row.backend_support.as_str(),
                expected.as_str()
            ),
        }
    }

    fn uaa_support_mismatch(row: &SupportRow, expected: UaaSupportState) -> Self {
        Self {
            code: "SUPPORT_MATRIX_UAA_SUPPORT_MISMATCH",
            agent: row.agent.clone(),
            version: row.version.clone(),
            target: row.target.clone(),
            message: format!(
                "row uaa_support={} does not match committed unified support state {}",
                row.uaa_support.as_str(),
                expected.as_str()
            ),
        }
    }

    fn row_order_mismatch(row: &SupportRow, expected: &SupportRow, index: usize) -> Self {
        Self {
            code: "SUPPORT_MATRIX_ROW_ORDER_MISMATCH",
            agent: row.agent.clone(),
            version: row.version.clone(),
            target: row.target.clone(),
            message: format!(
                "row order is not canonical at index {index}; expected {} {} {} at this position",
                expected.agent, expected.version, expected.target
            ),
        }
    }
}

#[derive(Debug, Clone)]
struct RootConsistencyContext {
    loaded_root: LoadedAgentRoot,
    version_statuses: BTreeMap<String, Option<String>>,
}

pub(crate) fn validate_publication_consistency(
    workspace_root: &Path,
    rows: &[SupportRow],
) -> Result<(), Vec<SupportMatrixConsistencyIssue>> {
    let mut issues = Vec::new();
    let mut roots = BTreeMap::new();
    let mut loaded_roots = Vec::new();
    let known_agents = CURRENT_AGENT_ROOTS
        .iter()
        .map(|(agent, _)| *agent)
        .collect::<BTreeSet<_>>();

    for (agent, rel_root) in CURRENT_AGENT_ROOTS {
        let root = AgentRoot {
            agent: agent.to_string(),
            root: workspace_root.join(rel_root),
        };
        if !root.root.exists() {
            // The committed root set is authoritative even when publication rows for that agent
            // have already been dropped; otherwise missing roots evade the exact-row-set check.
            issues.push(SupportMatrixConsistencyIssue {
                code: "SUPPORT_MATRIX_ROOT_READ_ERROR",
                agent: root.agent.clone(),
                version: String::new(),
                target: String::new(),
                message: format!(
                    "committed manifest root is missing from workspace: {}",
                    root.root.display()
                ),
            });
            continue;
        }
        let loaded_root = match load_agent_root(&root) {
            Ok(loaded_root) => loaded_root,
            Err(err) => {
                issues.push(SupportMatrixConsistencyIssue {
                    code: "SUPPORT_MATRIX_ROOT_READ_ERROR",
                    agent: root.agent.clone(),
                    version: String::new(),
                    target: String::new(),
                    message: err,
                });
                continue;
            }
        };
        let version_statuses = loaded_root
            .versions
            .iter()
            .map(|metadata| (metadata.semantic_version.clone(), metadata.status.clone()))
            .collect::<BTreeMap<_, _>>();
        loaded_roots.push(loaded_root.clone());
        roots.insert(
            root.agent.clone(),
            RootConsistencyContext {
                loaded_root,
                version_statuses,
            },
        );
    }

    for agent in rows
        .iter()
        .map(|row| row.agent.as_str())
        .filter(|agent| !known_agents.contains(agent))
        .collect::<BTreeSet<_>>()
    {
        issues.push(SupportMatrixConsistencyIssue {
            code: "SUPPORT_MATRIX_UNKNOWN_AGENT",
            agent: agent.to_string(),
            version: String::new(),
            target: String::new(),
            message: "row agent does not match a committed manifest root".to_string(),
        });
    }

    let expected_rows = match derive_rows_for_loaded_roots(&loaded_roots) {
        Ok(expected_rows) => expected_rows,
        Err(err) => {
            issues.push(SupportMatrixConsistencyIssue {
                code: "SUPPORT_MATRIX_ROOT_READ_ERROR",
                agent: String::new(),
                version: String::new(),
                target: String::new(),
                message: err,
            });
            return Err(issues);
        }
    };
    let expected_identities = expected_rows
        .iter()
        .map(SupportRowIdentity::from_row)
        .collect::<BTreeSet<_>>();
    let expected_rows_by_identity = expected_rows
        .iter()
        .map(|row| (SupportRowIdentity::from_row(row), row))
        .collect::<BTreeMap<_, _>>();
    let mut observed_counts = BTreeMap::<SupportRowIdentity, usize>::new();
    for row in rows {
        let identity = SupportRowIdentity::from_row(row);
        *observed_counts.entry(identity).or_default() += 1;
    }

    for identity in &expected_identities {
        if !observed_counts.contains_key(identity) {
            issues.push(SupportMatrixConsistencyIssue::missing_row(identity));
        }
    }
    for (identity, count) in &observed_counts {
        if *count > 1 {
            issues.push(SupportMatrixConsistencyIssue::duplicate_row(
                identity, *count,
            ));
        }
        if !expected_identities.contains(identity) {
            issues.push(SupportMatrixConsistencyIssue::unexpected_row(identity));
        }
    }

    let exact_identity_match = expected_identities.len() == observed_counts.len()
        && expected_identities
            .iter()
            .all(|identity| observed_counts.get(identity) == Some(&1))
        && observed_counts
            .iter()
            .all(|(identity, count)| *count == 1 && expected_identities.contains(identity));

    for row in rows {
        let Some(ctx) = roots.get(&row.agent) else {
            continue;
        };
        let identity = SupportRowIdentity::from_row(row);
        if !expected_identities.contains(&identity) {
            continue;
        }

        let expected_row = expected_rows_by_identity
            .get(&identity)
            .expect("expected row for known committed identity");
        validate_row_consistency(row, expected_row, ctx, &mut issues);
    }

    if exact_identity_match {
        for (index, (row, expected_row)) in rows.iter().zip(expected_rows.iter()).enumerate() {
            if SupportRowIdentity::from_row(row) != SupportRowIdentity::from_row(expected_row) {
                issues.push(SupportMatrixConsistencyIssue::row_order_mismatch(
                    row,
                    expected_row,
                    index,
                ));
                break;
            }
        }
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(issues)
    }
}

pub(crate) fn format_publication_issues(issues: &[SupportMatrixConsistencyIssue]) -> String {
    let mut out = String::from("support-matrix publication contradictions detected");
    for issue in issues {
        out.push_str(&format!(
            "\n- [{}] {} {} {}: {}",
            issue.code, issue.agent, issue.version, issue.target, issue.message
        ));
    }
    out
}

fn validate_row_consistency(
    row: &SupportRow,
    expected_row: &SupportRow,
    ctx: &RootConsistencyContext,
    issues: &mut Vec<SupportMatrixConsistencyIssue>,
) {
    if row.manifest_support != expected_row.manifest_support {
        issues.push(SupportMatrixConsistencyIssue::manifest_support_mismatch(
            row,
            expected_row.manifest_support,
        ));
    }
    if row.backend_support != expected_row.backend_support {
        issues.push(SupportMatrixConsistencyIssue::backend_support_mismatch(
            row,
            expected_row.backend_support,
        ));
    }
    if row.uaa_support != expected_row.uaa_support {
        issues.push(SupportMatrixConsistencyIssue::uaa_support_mismatch(
            row,
            expected_row.uaa_support,
        ));
    }

    let observed_promotion =
        classify_pointer_promotion(&ctx.loaded_root.pointers, &row.target, &row.version);
    if row.pointer_promotion != observed_promotion {
        issues.push(SupportMatrixConsistencyIssue::pointer_promotion_mismatch(
            row,
            observed_promotion,
        ));
    }

    let report = match load_support_report(&ctx.loaded_root.layout, &row.version, &row.target) {
        Ok(report) => report,
        Err(err) => {
            issues.push(SupportMatrixConsistencyIssue {
                code: "SUPPORT_MATRIX_REPORT_READ_ERROR",
                agent: row.agent.clone(),
                version: row.version.clone(),
                target: row.target.clone(),
                message: err,
            });
            return;
        }
    };
    let expected_notes = build_evidence_notes(
        report.as_ref(),
        &ctx.loaded_root.posture,
        &row.target,
        &row.version,
    );

    if expected_notes != row.evidence_notes {
        issues.push(SupportMatrixConsistencyIssue::evidence_notes_mismatch(
            row,
            &expected_notes,
        ));
    }

    if let Some(status) = ctx
        .version_statuses
        .get(&row.version)
        .and_then(|value| value.as_deref())
    {
        let requires_validation_status = matches!(
            row.pointer_promotion,
            PointerPromotionState::LatestValidated
                | PointerPromotionState::LatestSupportedAndValidated
        );
        if requires_validation_status && !matches!(status, "validated" | "supported") {
            issues.push(SupportMatrixConsistencyIssue {
                code: "SUPPORT_MATRIX_VERSION_STATUS_MISMATCH",
                agent: row.agent.clone(),
                version: row.version.clone(),
                target: row.target.clone(),
                message: format!(
                    "row pointer_promotion={} requires version status validated or supported, got {}",
                    row.pointer_promotion.as_str(),
                    status
                ),
            });
        }
    }

    let omitted_current_target = ctx.loaded_root.posture.current_version.as_deref()
        == Some(row.version.as_str())
        && !ctx
            .loaded_root
            .posture
            .current_targets
            .contains(&row.target);
    if omitted_current_target {
        let support_is_unsupported = row.manifest_support == ManifestSupportState::Unsupported
            && row.uaa_support == UaaSupportState::Unsupported
            && row.pointer_promotion == PointerPromotionState::None;
        if !support_is_unsupported {
            issues.push(SupportMatrixConsistencyIssue::omission_contradiction(
                row,
                &expected_notes,
            ));
        }
    }
}
