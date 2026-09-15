use std::{collections::BTreeSet, fs, io, path::Path};

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
    match report.platform_filter_mode.as_deref() {
        Some("any") => {}
        Some(mode) => {
            return Err(format!(
                "cannot classify unmatched debt rows: report {} has `platform_filter.mode` `{mode}`, expected `any`",
                report.path.display()
            ));
        }
        None => {
            return Err(format!(
                "cannot classify unmatched debt rows: report {} is missing `platform_filter.mode`",
                report.path.display()
            ));
        }
    }
    // Absent and malformed both arrive as `None`, so the message names the field either way.
    let report_targets = report.upstream_targets.as_ref().ok_or_else(|| {
        format!(
            "cannot classify unmatched debt rows: report {} has no usable `inputs.upstream.targets`",
            report.path.display()
        )
    })?;
    let union = load_union(workspace_root, entry, target_version)?;
    if report_targets != &union.input_targets {
        return Err(format!(
            "cannot classify unmatched debt rows: report {} `inputs.upstream.targets` {:?} differs from union input targets {:?}",
            report.path.display(),
            report_targets,
            union.input_targets
        ));
    }
    let excluded = surfaces_from_report_lists(
        &entry.agent_id,
        &report.path,
        &report.deltas,
        &EXCLUDED_LISTS,
    )?;
    Ok(rows
        .iter()
        .map(|row| {
            let identity = row.identity();
            // The audit prefers the any-target report, whose filter keeps every union surface, so
            // an observed surface on no gap or exclusion list is one wrapper coverage accounts
            // for. The exception is an `unsupported` command (uaa-0042).
            let observation = if excluded.contains(&identity) {
                "excluded_by_rules"
            } else if union.surfaces.contains(&identity) {
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
    inputs: Vec<UnionInput>,
    commands: Vec<UnionCommandSurfaces>,
}

#[derive(Deserialize)]
struct UnionInput {
    target_triple: String,
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

struct UnionEvidence {
    input_targets: BTreeSet<String>,
    surfaces: BTreeSet<SurfaceIdentity>,
}

fn load_union(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    target_version: &str,
) -> Result<UnionEvidence, String> {
    let path = workspace_root
        .join(&entry.manifest_root)
        .join("snapshots")
        .join(target_version)
        .join("union.json");
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Err(format!(
                "cannot classify unmatched debt rows: union {} is missing",
                path.display()
            ));
        }
        Err(err) => return Err(format!("read {}: {err}", path.display())),
    };
    let union = serde_json::from_str::<UnionSurfaces>(&text)
        .map_err(|err| format!("parse {}: {err}", path.display()))?;
    let agent_id = entry.agent_id.as_str();
    let mut surfaces = BTreeSet::new();
    for command in &union.commands {
        let path = &command.path;
        surfaces.insert(surface_identity(
            agent_id,
            path,
            ReportRowShape::Command,
            "",
        ));
        for flag in &command.flags {
            surfaces.insert(surface_identity(
                agent_id,
                path,
                ReportRowShape::Flag,
                &flag.key,
            ));
        }
        for arg in &command.args {
            surfaces.insert(surface_identity(
                agent_id,
                path,
                ReportRowShape::Arg,
                &arg.name,
            ));
        }
    }
    Ok(UnionEvidence {
        input_targets: union
            .inputs
            .into_iter()
            .map(|input| input.target_triple)
            .collect(),
        surfaces,
    })
}
