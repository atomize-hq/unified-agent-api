use std::{collections::BTreeSet, fs, path::Path};

use crate::{
    agent_registry::AgentRegistryEntry,
    support_matrix::{BackendSupportState, SupportRow, UaaSupportState},
};
use serde::Deserialize;

use super::{
    build_finding, shared, DriftCategory, DriftFinding, CAPABILITY_MATRIX_PATH,
    SUPPORT_MATRIX_JSON_PATH, SUPPORT_MATRIX_MARKDOWN_PATH,
};

pub(super) fn inspect_governance_docs(
    entry: &AgentRegistryEntry,
    workspace_root: &Path,
    capability_truth: Result<&BTreeSet<String>, &String>,
    expected_support_rows: &[SupportRow],
) -> Option<DriftFinding> {
    let docs_root = workspace_root.join(shared::historical_pack_root(entry));
    if !docs_root.exists() {
        return None;
    }

    let closed_surfaces = closed_governance_surfaces(workspace_root, entry);
    let mut issues = Vec::new();
    let mut surfaces = BTreeSet::new();

    let seam2_path = docs_root.join("governance/seam-2-closeout.md");
    let seam2_surface = shared::path_to_repo_relative(workspace_root, &seam2_path);
    if seam2_path.exists() {
        match capability_truth {
            Ok(truth) => match inspect_governance_capability_claim(&seam2_path, truth) {
                Ok(Some(issue)) => {
                    if !closed_surfaces.contains(&seam2_surface) {
                        issues.push(issue);
                        surfaces.insert(seam2_surface.clone());
                        surfaces.insert(CAPABILITY_MATRIX_PATH.to_string());
                    }
                }
                Ok(None) => {}
                Err(err) => {
                    if !closed_surfaces.contains(&seam2_surface) {
                        issues.push(err);
                        surfaces.insert(seam2_surface.clone());
                    }
                }
            },
            Err(err) => {
                if !closed_surfaces.contains(&seam2_surface) {
                    issues.push(err.clone());
                    surfaces.insert(seam2_surface.clone());
                }
            }
        }
    }

    let seam3_path = docs_root.join("governance/seam-3-closeout.md");
    let seam3_surface = shared::path_to_repo_relative(workspace_root, &seam3_path);
    if seam3_path.exists() {
        match inspect_governance_support_claim(&seam3_path, expected_support_rows) {
            Ok(Some(issue)) => {
                if !closed_surfaces.contains(&seam3_surface) {
                    issues.push(issue);
                    surfaces.insert(seam3_surface.clone());
                    surfaces.insert(SUPPORT_MATRIX_MARKDOWN_PATH.to_string());
                    surfaces.insert(SUPPORT_MATRIX_JSON_PATH.to_string());
                }
            }
            Ok(None) => {}
            Err(err) => {
                if !closed_surfaces.contains(&seam3_surface) {
                    issues.push(err);
                    surfaces.insert(seam3_surface.clone());
                }
            }
        }
    }

    if issues.is_empty() {
        None
    } else {
        Some(build_finding(
            DriftCategory::GovernanceDoc,
            "historical implementation/governance docs no longer match landed capability or support truth.",
            issues,
            surfaces.into_iter().collect(),
        ))
    }
}

fn closed_governance_surfaces(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
) -> BTreeSet<String> {
    let closeout_path = workspace_root.join(format!(
        "docs/project_management/next/{}-maintenance/governance/maintenance-closeout.json",
        entry.agent_id
    ));
    let Ok(text) = fs::read_to_string(closeout_path) else {
        return BTreeSet::new();
    };
    let Ok(closeout) = serde_json::from_str::<MaintenanceCloseoutRecord>(&text) else {
        return BTreeSet::new();
    };

    closeout
        .resolved_findings
        .into_iter()
        .filter(|finding| finding.category_id == DriftCategory::GovernanceDoc.category_id())
        .flat_map(|finding| finding.surfaces.into_iter())
        .collect()
}

fn inspect_governance_capability_claim(
    path: &Path,
    capability_truth: &BTreeSet<String>,
) -> Result<Option<String>, String> {
    let text =
        fs::read_to_string(path).map_err(|err| format!("read({}): {err}", path.display()))?;
    let Some(claim_lines) = shared::extract_bullet_block(
        &text,
        "are the claimed OpenCode v1 capability ids under the current",
    ) else {
        return Ok(None);
    };

    let claimed = shared::inline_code_ids(&claim_lines);
    if claimed == *capability_truth {
        return Ok(None);
    }

    let missing = capability_truth
        .difference(&claimed)
        .cloned()
        .collect::<Vec<_>>();
    let unexpected = claimed
        .difference(capability_truth)
        .cloned()
        .collect::<Vec<_>>();

    let mut pieces = Vec::new();
    if !missing.is_empty() {
        pieces.push(format!("missing capability ids: {}", missing.join(", ")));
    }
    if !unexpected.is_empty() {
        pieces.push(format!(
            "unexpected capability ids: {}",
            unexpected.join(", ")
        ));
    }

    Ok(Some(format!(
        "{} capability claim no longer matches current backend truth ({})",
        path.display(),
        pieces.join("; ")
    )))
}

fn inspect_governance_support_claim(
    path: &Path,
    expected_support_rows: &[SupportRow],
) -> Result<Option<String>, String> {
    let text =
        fs::read_to_string(path).map_err(|err| format!("read({}): {err}", path.display()))?;
    if !text.contains("backend support and UAA support remain `unsupported`") {
        return Ok(None);
    }

    let backend_unsupported = expected_support_rows
        .iter()
        .all(|row| row.backend_support == BackendSupportState::Unsupported);
    let uaa_unsupported = expected_support_rows
        .iter()
        .all(|row| row.uaa_support == UaaSupportState::Unsupported);

    if backend_unsupported && uaa_unsupported {
        Ok(None)
    } else {
        Ok(Some(format!(
            "{} still claims backend and UAA support remain unsupported, but current support truth has advanced",
            path.display()
        )))
    }
}

#[derive(Debug, Deserialize)]
struct MaintenanceCloseoutRecord {
    #[serde(default)]
    resolved_findings: Vec<ResolvedMaintenanceFinding>,
}

#[derive(Debug, Deserialize)]
struct ResolvedMaintenanceFinding {
    category_id: String,
    #[serde(default)]
    surfaces: Vec<String>,
}
