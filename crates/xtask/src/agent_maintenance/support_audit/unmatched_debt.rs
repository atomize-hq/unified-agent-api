use std::{collections::BTreeSet, fs, path::Path};

use serde::Deserialize;

use crate::agent_registry::AgentRegistryEntry;

use super::{
    surface_identity, surfaces_from_report_lists, DebtInventoryRow, LiveReport, ReportList,
    ReportRowShape, SurfaceIdentity, UnmatchedDebtSurface,
};

// The report writer omits each of these lists when it is empty.
const EXCLUDED_LISTS: [ReportList; 3] = [
    ("excluded_commands", Some(ReportRowShape::Command), false),
    ("excluded_flags", Some(ReportRowShape::Flag), false),
    ("excluded_args", Some(ReportRowShape::Arg), false),
];

/// Classifies debt rows that match no gap by what the live evidence shows. A surface absent from
/// the union is `not_observed`, never removed: help output cannot prove a removal, because
/// upstream can hide a surface from help.
pub(super) fn classify_unmatched_debt_rows(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    target_version: &str,
    report: &LiveReport,
    rows: &[&DebtInventoryRow],
) -> Result<Vec<UnmatchedDebtSurface>, String> {
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let excluded = surfaces_from_report_lists(
        &entry.agent_id,
        &report.path,
        &report.deltas,
        &EXCLUDED_LISTS,
    )?;
    let observed = load_union_surfaces(workspace_root, entry, target_version)?;
    Ok(rows
        .iter()
        .map(|row| {
            let identity = row.identity();
            // The audit prefers the any-target report, whose filter keeps every union surface, so
            // an observed surface on no gap or exclusion list is one wrapper coverage accounts
            // for. The exception is an `unsupported` command (uaa-0042).
            let observation = if excluded.contains(&identity) {
                "excluded_by_rules"
            } else if observed.contains(&identity) {
                "covered_by_wrapper"
            } else {
                "not_observed"
            };
            UnmatchedDebtSurface {
                surface_kind: identity.surface_kind,
                command_path: identity.command_path,
                surface_id: identity.surface_id,
                debt_ref: row.debt_ref(),
                observation: observation.to_string(),
            }
        })
        .collect())
}

#[derive(Deserialize)]
struct UnionSurfaces {
    commands: Vec<UnionCommandSurfaces>,
}

#[derive(Deserialize)]
struct UnionCommandSurfaces {
    path: Vec<String>,
    #[serde(default)]
    flags: Vec<UnionFlagSurface>,
    #[serde(default)]
    args: Vec<UnionArgSurface>,
}

#[derive(Deserialize)]
struct UnionFlagSurface {
    key: String,
}

#[derive(Deserialize)]
struct UnionArgSurface {
    name: String,
}

fn load_union_surfaces(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    target_version: &str,
) -> Result<BTreeSet<SurfaceIdentity>, String> {
    let path = workspace_root
        .join(&entry.manifest_root)
        .join("snapshots")
        .join(target_version)
        .join("union.json");
    let text =
        fs::read_to_string(&path).map_err(|err| format!("read {}: {err}", path.display()))?;
    let union = serde_json::from_str::<UnionSurfaces>(&text)
        .map_err(|err| format!("parse {}: {err}", path.display()))?;
    let agent_id = entry.agent_id.as_str();
    let mut observed = BTreeSet::new();
    for command in &union.commands {
        let path = &command.path;
        observed.insert(surface_identity(
            agent_id,
            path,
            ReportRowShape::Command,
            "",
        ));
        for flag in &command.flags {
            observed.insert(surface_identity(
                agent_id,
                path,
                ReportRowShape::Flag,
                &flag.key,
            ));
        }
        for arg in &command.args {
            observed.insert(surface_identity(
                agent_id,
                path,
                ReportRowShape::Arg,
                &arg.name,
            ));
        }
    }
    Ok(observed)
}
