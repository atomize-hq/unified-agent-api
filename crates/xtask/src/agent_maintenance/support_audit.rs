use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use crate::agent_registry::AgentRegistryEntry;

use super::request::DetectedRelease;

#[path = "support_audit/authorization.rs"]
mod authorization;
#[path = "support_audit/unmatched_debt.rs"]
mod unmatched_debt;

use authorization::{authorize_gap, validate_authorizations, TargetedGap};
pub(crate) use authorization::{load_debt_inventory, load_targeted_gaps, DebtInventoryRow};

pub(crate) const NON_TUI_SUPPORT_DEBT_PATH: &str =
    "docs/specs/unified-agent-api/non-tui-support-debt.md";
pub(crate) const SUPPORT_MATRIX_DOC_PATH: &str = "docs/specs/unified-agent-api/support-matrix.md";
const SURFACE_KINDS: [&str; 5] = [
    "commands",
    "subcommands",
    "flags",
    "global_flags",
    "positional_args",
];
const EXCLUDED_SURFACE_KINDS: [&str; 1] = ["tui_only"];
const ALLOWED_DEFERRALS: [&str; 5] = [
    "upstream_not_machine_exposed",
    "platform_evidence_missing",
    "requires_new_infra",
    "requires_new_architectural_seam",
    "outside_registry_maintenance_write_envelope",
];
pub(crate) const REQUIRED_WRITES: [&str; 5] = [
    "wrapper",
    "backend",
    "manifest",
    "publication",
    "packet_docs",
];
pub(crate) const ELIGIBILITY_REASONS: [&str; 3] = [
    "adjacent_surface_changed",
    "bounded_write_envelope",
    "no_new_seam_required",
];
pub(crate) const DEBT_OBSERVATIONS: [&str; 3] =
    ["covered_by_wrapper", "excluded_by_rules", "not_observed"];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SurfaceIdentity {
    pub surface_kind: String,
    pub command_path: String,
    pub surface_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceBackedSurface {
    pub surface_kind: String,
    pub command_path: String,
    pub surface_id: String,
    pub evidence_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebtBackedSurface {
    pub surface_kind: String,
    pub command_path: String,
    pub surface_id: String,
    pub debt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnmatchedDebtSurface {
    pub surface_kind: String,
    pub command_path: String,
    pub surface_id: String,
    pub debt_ref: String,
    pub observation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EligibleSurface {
    pub surface_kind: String,
    pub command_path: String,
    pub surface_id: String,
    pub eligibility_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredUplift {
    pub surface_kind: String,
    pub command_path: String,
    pub surface_id: String,
    pub reason: String,
    pub required_writes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferredGap {
    pub surface_kind: String,
    pub command_path: String,
    pub surface_id: String,
    pub defer_reason: String,
    pub blocking_follow_on: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationImpact {
    pub surface_kind: String,
    pub command_path: String,
    pub surface_id: String,
    pub surface_doc: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportSurfaceAudit {
    pub required: bool,
    pub surface_kinds: Vec<String>,
    pub excluded_surface_kinds: Vec<String>,
    pub allowed_deferrals: Vec<String>,
    pub pre_run_debt_count: usize,
    pub expected_post_run_debt_count: usize,
    pub unbaselined_gap_surface: Vec<EvidenceBackedSurface>,
    pub unmatched_debt_surface: Vec<UnmatchedDebtSurface>,
    pub preexisting_unsupported_surface: Vec<DebtBackedSurface>,
    pub eligible_preexisting_surface: Vec<EligibleSurface>,
    pub missing_wrapper_support: Vec<SurfaceIdentity>,
    pub missing_backend_support: Vec<SurfaceIdentity>,
    pub required_uplifts_this_run: Vec<RequiredUplift>,
    pub deferred_preexisting_gaps: Vec<DeferredGap>,
    pub publication_impacts: Vec<PublicationImpact>,
}

impl SurfaceIdentity {
    pub fn new(surface_kind: String, command_path: String, surface_id: String) -> Self {
        Self {
            surface_kind,
            command_path,
            surface_id,
        }
    }

    pub(crate) fn describe(&self) -> String {
        format!(
            "surface_kind={} command_path={} surface_id={}",
            self.surface_kind, self.command_path, self.surface_id
        )
    }
}

impl EvidenceBackedSurface {
    pub(crate) fn identity(&self) -> SurfaceIdentity {
        SurfaceIdentity::new(
            self.surface_kind.clone(),
            self.command_path.clone(),
            self.surface_id.clone(),
        )
    }
}

impl DebtBackedSurface {
    pub(crate) fn identity(&self) -> SurfaceIdentity {
        SurfaceIdentity::new(
            self.surface_kind.clone(),
            self.command_path.clone(),
            self.surface_id.clone(),
        )
    }
}

impl UnmatchedDebtSurface {
    pub(crate) fn identity(&self) -> SurfaceIdentity {
        SurfaceIdentity::new(
            self.surface_kind.clone(),
            self.command_path.clone(),
            self.surface_id.clone(),
        )
    }
}

impl EligibleSurface {
    pub(crate) fn identity(&self) -> SurfaceIdentity {
        SurfaceIdentity::new(
            self.surface_kind.clone(),
            self.command_path.clone(),
            self.surface_id.clone(),
        )
    }
}

impl RequiredUplift {
    pub(crate) fn identity(&self) -> SurfaceIdentity {
        SurfaceIdentity::new(
            self.surface_kind.clone(),
            self.command_path.clone(),
            self.surface_id.clone(),
        )
    }
}

impl DeferredGap {
    pub(crate) fn identity(&self) -> SurfaceIdentity {
        SurfaceIdentity::new(
            self.surface_kind.clone(),
            self.command_path.clone(),
            self.surface_id.clone(),
        )
    }
}

impl PublicationImpact {
    pub(crate) fn identity(&self) -> SurfaceIdentity {
        SurfaceIdentity::new(
            self.surface_kind.clone(),
            self.command_path.clone(),
            self.surface_id.clone(),
        )
    }
}

pub(crate) fn derive_support_surface_audit(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    detected_release: &DetectedRelease,
) -> Result<SupportSurfaceAudit, String> {
    let debt_rows = load_debt_inventory(workspace_root)?
        .into_iter()
        .filter(|row| row.agent_id == entry.agent_id)
        .collect::<Vec<_>>();
    let pre_run_debt_count = debt_rows
        .iter()
        .map(DebtInventoryRow::identity)
        .collect::<BTreeSet<_>>()
        .len();
    validate_authorizations(workspace_root, entry, &debt_rows)?;
    let report =
        load_live_report_if_present(workspace_root, entry, &detected_release.target_version)?;
    let report_ref = report
        .as_ref()
        .map(|report| repo_relative(workspace_root, &report.path))
        .transpose()?;
    let gaps = report
        .as_ref()
        .map(|report| report.gaps.clone())
        .unwrap_or_else(|| {
            debt_rows
                .iter()
                .map(|row| TargetedGap {
                    identity: row.identity(),
                    targets: BTreeSet::new(),
                })
                .collect()
        });
    let surfaces = gaps
        .iter()
        .map(|gap| gap.identity.clone())
        .collect::<Vec<_>>();

    let mut preexisting = Vec::new();
    let mut discovered = Vec::new();
    let mut deferred = Vec::new();
    let mut publication_impacts = Vec::new();
    let missing_wrapper_support = surfaces.clone();
    let missing_backend_support = surfaces.clone();

    for gap in &gaps {
        let surface = &gap.identity;
        publication_impacts.push(PublicationImpact {
            surface_kind: surface.surface_kind.clone(),
            command_path: surface.command_path.clone(),
            surface_id: surface.surface_id.clone(),
            surface_doc: SUPPORT_MATRIX_DOC_PATH.to_string(),
        });
        let authorization = authorize_gap(gap, &detected_release.target_version, &debt_rows);
        let debt_row = debt_rows
            .iter()
            .filter(|row| row.identity() == *surface)
            .min_by(|left, right| left.row_id.cmp(&right.row_id));
        // Without per-target report evidence, an inventory identity keeps the pre-acquisition
        // packet visible but no authorization grant is allowed to apply.
        let fully_authorized = report.is_some()
            && !authorization.applicable_rows.is_empty()
            && authorization.remaining_targets.is_empty();
        if let Some(row) = debt_row {
            preexisting.push(DebtBackedSurface {
                surface_kind: surface.surface_kind.clone(),
                command_path: surface.command_path.clone(),
                surface_id: surface.surface_id.clone(),
                debt_ref: row.debt_ref(),
            });
            deferred.push(DeferredGap {
                surface_kind: surface.surface_kind.clone(),
                command_path: surface.command_path.clone(),
                surface_id: surface.surface_id.clone(),
                defer_reason: row.blocker_class.clone(),
                blocking_follow_on: Some(row.follow_on.clone()),
            });
        }
        if !fully_authorized {
            let evidence_ref = report_ref
                .clone()
                .unwrap_or_else(|| NON_TUI_SUPPORT_DEBT_PATH.to_string());
            discovered.push(EvidenceBackedSurface {
                surface_kind: surface.surface_kind.clone(),
                command_path: surface.command_path.clone(),
                surface_id: surface.surface_id.clone(),
                evidence_ref,
            });
        }
    }

    let unmatched_debt_surface = match &report {
        Some(report) => {
            let mut seen = BTreeSet::new();
            let unmatched = debt_rows
                .iter()
                .filter(|row| !surfaces.contains(&row.identity()))
                .filter(|row| seen.insert(row.identity()))
                .collect::<Vec<_>>();
            unmatched_debt::classify_unmatched_debt_rows(
                workspace_root,
                entry,
                &detected_release.target_version,
                report,
                &unmatched,
            )?
        }
        None => Vec::new(),
    };

    let required_uplifts_this_run = discovered
        .iter()
        .map(|surface| RequiredUplift {
            surface_kind: surface.surface_kind.clone(),
            command_path: surface.command_path.clone(),
            surface_id: surface.surface_id.clone(),
            reason: "unbaselined_gap".to_string(),
            required_writes: REQUIRED_WRITES.iter().map(ToString::to_string).collect(),
        })
        .collect::<Vec<_>>();

    Ok(SupportSurfaceAudit {
        required: true,
        surface_kinds: SURFACE_KINDS.iter().map(ToString::to_string).collect(),
        excluded_surface_kinds: EXCLUDED_SURFACE_KINDS
            .iter()
            .map(ToString::to_string)
            .collect(),
        allowed_deferrals: ALLOWED_DEFERRALS.iter().map(ToString::to_string).collect(),
        pre_run_debt_count,
        expected_post_run_debt_count: preexisting.len(),
        unbaselined_gap_surface: discovered,
        unmatched_debt_surface,
        preexisting_unsupported_surface: preexisting,
        eligible_preexisting_surface: Vec::new(),
        missing_wrapper_support,
        missing_backend_support,
        required_uplifts_this_run,
        deferred_preexisting_gaps: deferred,
        publication_impacts,
    })
}

pub(crate) fn coverage_report_present_for_target(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    target_version: &str,
) -> Result<bool, String> {
    let version_dir = coverage_report_version_dir(workspace_root, entry, target_version);
    if !version_dir.is_dir() {
        return Ok(false);
    }

    let report_path = match select_report_path(&version_dir) {
        Ok(path) => path,
        Err(error) if error.starts_with("no coverage report found under") => return Ok(false),
        Err(error) => return Err(error),
    };
    fs::File::open(&report_path).map_err(|err| format!("open {}: {err}", report_path.display()))?;
    Ok(true)
}

/// The coverage report the audit reads: its path, its deltas, and the gap surfaces they list.
struct LiveReport {
    path: PathBuf,
    deltas: serde_json::Map<String, serde_json::Value>,
    gaps: Vec<TargetedGap>,
    platform_filter_mode: Option<String>,
    upstream_targets: Option<BTreeSet<String>>,
}

fn load_live_report_if_present(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    target_version: &str,
) -> Result<Option<LiveReport>, String> {
    let version_dir = coverage_report_version_dir(workspace_root, entry, target_version);
    if !version_dir.is_dir() {
        return Ok(None);
    }

    match load_live_report(workspace_root, entry, target_version) {
        Ok(result) => Ok(Some(result)),
        Err(error)
            if error.starts_with("no coverage report found under")
                || error.starts_with("read_dir(") =>
        {
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

pub(crate) fn allowed_deferrals() -> Vec<String> {
    ALLOWED_DEFERRALS.iter().map(ToString::to_string).collect()
}

pub(crate) fn surface_kinds() -> Vec<String> {
    SURFACE_KINDS.iter().map(ToString::to_string).collect()
}

pub(crate) fn excluded_surface_kinds() -> Vec<String> {
    EXCLUDED_SURFACE_KINDS
        .iter()
        .map(ToString::to_string)
        .collect()
}

fn parse_key_value(input: &str) -> Option<(&str, &str)> {
    let (key, rest) = input.split_once("`: `")?;
    let value = rest.strip_suffix('`')?;
    Some((key, value))
}

fn load_live_report(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    target_version: &str,
) -> Result<LiveReport, String> {
    let version_dir = coverage_report_version_dir(workspace_root, entry, target_version);
    let report_path = select_report_path(&version_dir)?;
    let text = fs::read_to_string(&report_path)
        .map_err(|err| format!("read {}: {err}", report_path.display()))?;
    let json = serde_json::from_str::<serde_json::Value>(&text)
        .map_err(|err| format!("parse {}: {err}", report_path.display()))?;
    let deltas = json
        .get("deltas")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| format!("{} is missing `deltas` object", report_path.display()))?
        .clone();
    let platform_filter_mode = json
        .pointer("/platform_filter/mode")
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);
    let upstream_targets = json
        .pointer("/inputs/upstream/targets")
        .and_then(serde_json::Value::as_array)
        .and_then(|targets| {
            targets
                .iter()
                .map(|target| target.as_str().map(ToString::to_string))
                .collect::<Option<BTreeSet<_>>>()
        });
    let gaps = load_targeted_gaps(
        &version_dir,
        &entry.agent_id,
        target_version,
        upstream_targets.as_ref().ok_or_else(|| {
            format!(
                "cannot classify unmatched debt rows: report {} has no usable `inputs.upstream.targets`",
                report_path.display()
            )
        })?,
    )?;

    Ok(LiveReport {
        path: report_path,
        deltas,
        gaps,
        platform_filter_mode,
        upstream_targets,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReportRowShape {
    Command,
    Flag,
    Arg,
}

/// A report delta list: its key, the row shape it must hold (any shape when `None`), and whether
/// the report must carry it.
type ReportList = (&'static str, Option<ReportRowShape>, bool);

// `deltas.unsupported` (commands whose wrapper level is `unsupported`) is not a gap list (uaa-0042).
const GAP_LISTS: [ReportList; 4] = [
    ("missing_commands", Some(ReportRowShape::Command), true),
    ("missing_flags", Some(ReportRowShape::Flag), true),
    ("missing_args", Some(ReportRowShape::Arg), true),
    // The report writer omits this list when it is empty.
    ("intentionally_unsupported", None, false),
];

pub(crate) fn surfaces_from_report_deltas(
    agent_id: &str,
    report_path: &Path,
    deltas: &serde_json::Map<String, serde_json::Value>,
) -> Result<Vec<SurfaceIdentity>, String> {
    surfaces_from_report_lists(agent_id, report_path, deltas, &GAP_LISTS)
        .map(|surfaces| surfaces.into_iter().collect())
}

fn surfaces_from_report_lists(
    agent_id: &str,
    report_path: &Path,
    deltas: &serde_json::Map<String, serde_json::Value>,
    lists: &[ReportList],
) -> Result<BTreeSet<SurfaceIdentity>, String> {
    let mut surfaces = BTreeSet::new();
    for &(key, required_shape, list_required) in lists {
        let rows = match deltas.get(key) {
            None if !list_required => continue,
            value => value.and_then(serde_json::Value::as_array).ok_or_else(|| {
                format!("{} is missing `deltas.{key}` array", report_path.display())
            })?,
        };
        for row in rows {
            let (shape, surface) = surface_from_report_value(agent_id, row)?;
            if required_shape.is_some_and(|required| required != shape) {
                return Err(format!(
                    "support-audit report row in `deltas.{key}` has the shape of a {shape:?} row"
                ));
            }
            surfaces.insert(surface);
        }
    }
    Ok(surfaces)
}

fn coverage_report_version_dir(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    target_version: &str,
) -> PathBuf {
    workspace_root
        .join(&entry.manifest_root)
        .join("reports")
        .join(target_version)
}

pub(crate) fn select_report_path(version_dir: &Path) -> Result<PathBuf, String> {
    for preferred in ["coverage.any.json", "coverage.all.json"] {
        let path = version_dir.join(preferred);
        if path.is_file() {
            return Ok(path);
        }
    }
    let mut candidates = fs::read_dir(version_dir)
        .map_err(|err| format!("read_dir({}): {err}", version_dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("coverage.") && name.ends_with(".json"))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    candidates.sort();
    candidates
        .into_iter()
        .next()
        .ok_or_else(|| format!("no coverage report found under {}", version_dir.display()))
}

fn surface_from_report_value(
    agent_id: &str,
    row: &serde_json::Value,
) -> Result<(ReportRowShape, SurfaceIdentity), String> {
    let object = row
        .as_object()
        .ok_or_else(|| "support-audit report row must be an object".to_string())?;
    let path = object
        .get("path")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "support-audit report row missing `path`".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToString::to_string)
                .ok_or_else(|| "support-audit report row `path` values must be strings".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;

    let key = optional_report_row_string(object, "key")?;
    let name = optional_report_row_string(object, "name")?;
    let (shape, leaf) = match (key, name) {
        (Some(_), Some(_)) => {
            return Err("support-audit report row must not carry both `key` and `name`".to_string())
        }
        (Some(flag), None) => (ReportRowShape::Flag, flag),
        (None, Some(arg_name)) => (ReportRowShape::Arg, arg_name),
        (None, None) => (ReportRowShape::Command, ""),
    };

    Ok((shape, surface_identity(agent_id, &path, shape, leaf)))
}

/// Maps a report or union row to its audit identity; `leaf` is the flag key or argument name.
fn surface_identity(
    agent_id: &str,
    path: &[String],
    shape: ReportRowShape,
    leaf: &str,
) -> SurfaceIdentity {
    let command_path = if path.is_empty() {
        agent_id.to_string()
    } else {
        format!("{agent_id} {}", path.join(" "))
    };
    let (surface_kind, surface_id) = match shape {
        ReportRowShape::Flag if path.is_empty() => ("global_flags", leaf.to_string()),
        ReportRowShape::Flag => ("flags", leaf.to_string()),
        ReportRowShape::Arg => ("positional_args", leaf.to_string()),
        ReportRowShape::Command => (
            if path.len() > 1 {
                "subcommands"
            } else {
                "commands"
            },
            // A command row with an empty path is the agent's own root command.
            path.last().cloned().unwrap_or_else(|| agent_id.to_string()),
        ),
    };
    SurfaceIdentity::new(surface_kind.to_string(), command_path, surface_id)
}

fn optional_report_row_string<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<Option<&'a str>, String> {
    object
        .get(field)
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| format!("support-audit report row `{field}` must be a string"))
        })
        .transpose()
}

fn repo_relative(workspace_root: &Path, path: &Path) -> Result<String, String> {
    path.strip_prefix(workspace_root)
        .map(|relative| relative.to_string_lossy().to_string())
        .map_err(|_| format!("{} is outside workspace root", path.display()))
}
