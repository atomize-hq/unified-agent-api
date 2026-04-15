use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use crate::wrapper_coverage_shared::RootIntakeLayout;

use super::{
    build_evidence_notes, classify_pointer_promotion, load_support_report,
    read_current_root_posture, read_json, read_pointers, AgentRoot, BackendSupportState,
    CurrentRootPosture, ManifestSupportState, PointerPromotionState, PointerSet, SupportRow,
    UaaSupportState, VersionMetadata, CURRENT_AGENT_ROOTS,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SupportMatrixConsistencyIssue {
    pub code: &'static str,
    pub agent: String,
    pub version: String,
    pub target: String,
    pub message: String,
}

impl SupportMatrixConsistencyIssue {
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
}

#[derive(Debug, Clone)]
struct RootConsistencyContext {
    posture: CurrentRootPosture,
    pointers: PointerSet,
    layout: RootIntakeLayout,
    version_statuses: BTreeMap<String, Option<String>>,
}

pub(crate) fn validate_publication_consistency(
    workspace_root: &Path,
    rows: &[SupportRow],
) -> Result<(), Vec<SupportMatrixConsistencyIssue>> {
    let mut issues = Vec::new();
    let mut roots = BTreeMap::new();

    let required_agents = rows
        .iter()
        .map(|row| row.agent.as_str())
        .collect::<BTreeSet<_>>();
    if required_agents.is_empty() {
        return Ok(());
    }

    for agent in required_agents {
        let Some((_, rel_root)) = CURRENT_AGENT_ROOTS
            .iter()
            .find(|(candidate, _)| *candidate == agent)
        else {
            issues.push(SupportMatrixConsistencyIssue {
                code: "SUPPORT_MATRIX_UNKNOWN_AGENT",
                agent: agent.to_string(),
                version: String::new(),
                target: String::new(),
                message: "row agent does not match a committed manifest root".to_string(),
            });
            continue;
        };
        let root = AgentRoot {
            agent: agent.to_string(),
            root: workspace_root.join(rel_root),
        };
        let posture = match read_current_root_posture(&root.root) {
            Ok(posture) => posture,
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
        let pointers = match read_pointers(&root.root, &posture.expected_targets) {
            Ok(pointers) => pointers,
            Err(err) => {
                issues.push(SupportMatrixConsistencyIssue {
                    code: "SUPPORT_MATRIX_POINTER_READ_ERROR",
                    agent: root.agent.clone(),
                    version: String::new(),
                    target: String::new(),
                    message: err,
                });
                continue;
            }
        };
        roots.insert(
            root.agent.clone(),
            RootConsistencyContext {
                posture,
                pointers,
                layout: RootIntakeLayout::new(root.root.clone()),
                version_statuses: match read_version_statuses(&root.root) {
                    Ok(version_statuses) => version_statuses,
                    Err(err) => {
                        issues.push(SupportMatrixConsistencyIssue {
                            code: "SUPPORT_MATRIX_VERSION_STATUS_READ_ERROR",
                            agent: root.agent.clone(),
                            version: String::new(),
                            target: String::new(),
                            message: err,
                        });
                        continue;
                    }
                },
            },
        );
    }

    for row in rows {
        let Some(ctx) = roots.get(&row.agent) else {
            continue;
        };

        validate_row_consistency(row, ctx, &mut issues);
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

fn read_version_statuses(root: &Path) -> Result<BTreeMap<String, Option<String>>, String> {
    let layout = RootIntakeLayout::new(root.to_path_buf());
    let versions_dir = layout.versions_dir();
    let mut version_statuses = BTreeMap::new();

    for entry in std::fs::read_dir(&versions_dir)
        .map_err(|err| format!("read_dir({}): {err}", versions_dir.display()))?
    {
        let entry = entry.map_err(|err| format!("read_dir entry error: {err}"))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let metadata: VersionMetadata = read_json(&path)?;
        version_statuses.insert(metadata.semantic_version, metadata.status);
    }

    Ok(version_statuses)
}

fn validate_row_consistency(
    row: &SupportRow,
    ctx: &RootConsistencyContext,
    issues: &mut Vec<SupportMatrixConsistencyIssue>,
) {
    let observed_promotion = classify_pointer_promotion(&ctx.pointers, &row.target, &row.version);
    if row.pointer_promotion != observed_promotion {
        issues.push(SupportMatrixConsistencyIssue::pointer_promotion_mismatch(
            row,
            observed_promotion,
        ));
    }

    let report = match load_support_report(&ctx.layout, &row.version, &row.target) {
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
    let expected_notes =
        build_evidence_notes(report.as_ref(), &ctx.posture, &row.target, &row.version);

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

    let omitted_current_target = ctx.posture.current_version.as_deref()
        == Some(row.version.as_str())
        && !ctx.posture.current_targets.contains(&row.target);
    if omitted_current_target {
        let support_is_unsupported = row.manifest_support == ManifestSupportState::Unsupported
            && row.backend_support == BackendSupportState::Unsupported
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
