//! Read-only preparation preview using the ordinary loaded-root derivation and renderers.
use super::{
    derive::{derive_rows_for_loaded_roots, load_agent_root, AgentRoot},
    render_publication_artifacts,
    runtime_support::{
        derive_runtime_support_projection_for_loaded_roots,
        render_agent_api_runtime_support_data_from_projection, EMBEDDED_RUNTIME_SUPPORT_FAMILIES,
    },
    PublicationBundle, SupportMatrixArtifact, SupportRow, JSON_OUTPUT_PATH, MARKDOWN_OUTPUT_PATH,
};
use crate::depth_record::DepthResult;
use crate::{agent_registry::AgentRegistry, depth_record::DepthRecord};
use std::{collections::BTreeSet, fs, path::Path};

/// Proposed data is not admission. Command writers must retain their guarded registry loader.
pub fn preview_preparation(
    workspace: &Path,
    registry: &AgentRegistry,
    record: &DepthRecord,
) -> Result<PublicationBundle, String> {
    let selection = &record.policy.selection;
    let mut roots = Vec::new();
    for entry in registry
        .agents
        .iter()
        .filter(|e| !e.publication.support_matrix_enabled)
    {
        if !crate::depth_record::record_files(&workspace.join(&entry.manifest_root)).is_empty() {
            return Err(format!(
                "depth record root for {} is not published",
                entry.agent_id
            ));
        }
    }
    for entry in registry.support_matrix_entries() {
        let mut loaded = load_agent_root(&AgentRoot::new(
            workspace,
            &entry.agent_id,
            &entry.manifest_root,
        ))?;
        if let Some(depth) = &entry.support_depth {
            for declaration in depth.enrollments.iter().flatten() {
                loaded.declared.insert(
                    declaration.version.clone(),
                    declaration.targets.iter().cloned().collect(),
                );
            }
        }
        if entry.agent_id == selection.agent_id {
            loaded
                .records
                .insert(selection.enrollment.version.clone(), record.clone());
        }
        for version in loaded.records.keys() {
            if !loaded.declared.contains_key(version) {
                return Err(format!(
                    "missing policy for retained depth record {}/{version}",
                    entry.agent_id
                ));
            }
        }
        roots.push(loaded);
    }
    if !roots.iter().any(|root| root.agent == selection.agent_id) {
        return Err("depth preparation agent is not published".into());
    }
    let rows = derive_rows_for_loaded_roots(&roots)?;
    let before: SupportMatrixArtifact = serde_json::from_slice(
        &fs::read(workspace.join(JSON_OUTPUT_PATH)).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if let Some(error) = before.revision_error() {
        return Err(error);
    }
    // C can retain E mechanically, but cannot newly assert its execution/reuse/debt verdicts.
    // These reporting-only limits are stricter than check 9's qualified-acceptance categories.
    let previous = positive_results(&before.rows)?;
    if let Some(fact) = positive_results(&rows)?.difference(&previous).next() {
        return Err(format!(
            "preparation requires a deferred execution/debt verdict for {fact:?}"
        ));
    }
    let projection = derive_runtime_support_projection_for_loaded_roots(&roots)?;
    render_publication_artifacts(
        &fs::read_to_string(workspace.join(MARKDOWN_OUTPUT_PATH)).map_err(|e| e.to_string())?,
        &rows,
        render_agent_api_runtime_support_data_from_projection(
            &projection,
            EMBEDDED_RUNTIME_SUPPORT_FAMILIES,
        ),
    )
}

type PositiveResult = (String, String, String, String);
fn positive_results(rows: &[SupportRow]) -> Result<BTreeSet<PositiveResult>, String> {
    let mut result = BTreeSet::new();
    for row in rows {
        let Some(depth) = &row.depth else {
            continue;
        };
        let mut atoms = Vec::new();
        for promise in &depth.promises {
            for obligation in &promise.obligations {
                if matches!(
                    obligation.result,
                    DepthResult::Verified | DepthResult::NotApplicable
                ) {
                    atoms.push(
                        serde_json::to_string(&("obligation", &promise.operation, obligation))
                            .map_err(|e| e.to_string())?,
                    );
                }
            }
            if promise
                .outcomes
                .contains(&super::depth::Outcome::AuthorizedDebt)
            {
                atoms.push(
                    serde_json::to_string(&("authorized_debt", &promise.operation))
                        .map_err(|e| e.to_string())?,
                );
            }
        }
        for exclusion in &depth.mode_exclusions {
            if matches!(
                exclusion.result,
                DepthResult::Verified | DepthResult::NotApplicable
            ) {
                atoms.push(
                    serde_json::to_string(&("exclusion", exclusion)).map_err(|e| e.to_string())?,
                );
            }
        }
        result.extend(atoms.into_iter().map(|atom| {
            (
                row.agent.clone(),
                row.version.clone(),
                row.target.clone(),
                atom,
            )
        }));
    }
    Ok(result)
}
