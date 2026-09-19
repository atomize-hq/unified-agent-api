use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path},
};

use semver::Version;
use serde::Deserialize;

use crate::agent_registry::AgentRegistryEntry;

use super::{
    parse_key_value, surface_from_report_value, surfaces_from_report_deltas, SurfaceIdentity,
    ALLOWED_DEFERRALS, GAP_LISTS, NON_TUI_SUPPORT_DEBT_PATH,
};

pub(super) const CONTRACT_MARKER_ROW_ID: &str =
    "support-debt-authorization-contract-target-version-v1";

const REQUIRED_KEYS: [&str; 13] = [
    "agent_id",
    "surface_kind",
    "command_path",
    "surface_id",
    "current_reason",
    "blocker_class",
    "owner",
    "milestone",
    "follow_on",
    "evidence_ref",
    "scope_target_triples",
    "authorized_at_version",
    "authorization_evidence_ref",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebtInventoryRow {
    pub row_id: String,
    pub agent_id: String,
    pub surface_kind: String,
    pub command_path: String,
    pub surface_id: String,
    pub current_reason: String,
    pub blocker_class: String,
    pub owner: String,
    pub milestone: String,
    pub follow_on: String,
    pub evidence_ref: String,
    pub scope_target_triples: BTreeSet<String>,
    pub authorized_at_version: String,
    pub authorization_evidence_ref: String,
}

impl DebtInventoryRow {
    pub fn identity(&self) -> SurfaceIdentity {
        SurfaceIdentity::new(
            self.surface_kind.clone(),
            self.command_path.clone(),
            self.surface_id.clone(),
        )
    }

    pub fn debt_ref(&self) -> String {
        format!("{NON_TUI_SUPPORT_DEBT_PATH}#{}", self.row_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TargetedGap {
    pub identity: SurfaceIdentity,
    pub targets: BTreeSet<String>,
}

pub(super) struct GapAuthorization<'a> {
    pub applicable_rows: Vec<&'a DebtInventoryRow>,
    pub remaining_targets: BTreeSet<String>,
}

pub(crate) fn load_debt_inventory(workspace_root: &Path) -> Result<Vec<DebtInventoryRow>, String> {
    let path = workspace_root.join(NON_TUI_SUPPORT_DEBT_PATH);
    let text = fs::read_to_string(&path)
        .map_err(|err| inventory_error(format!("read {}: {err}", path.display())))?;
    let mut rows = Vec::new();
    let mut current_row_id: Option<String> = None;
    let mut current_fields = BTreeMap::<String, String>::new();
    let mut marker_seen = false;

    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("### `") {
            if let Some(row_id) = current_row_id.take() {
                rows.push(build_debt_row(&row_id, &current_fields)?);
                current_fields.clear();
            }
            let row_id = rest.strip_suffix('`').ok_or_else(|| {
                inventory_error(format!("invalid debt heading in {}", path.display()))
            })?;
            if row_id == CONTRACT_MARKER_ROW_ID {
                if marker_seen {
                    return Err(inventory_error("duplicate contract marker"));
                }
                marker_seen = true;
            } else {
                current_row_id = Some(row_id.to_string());
            }
            continue;
        }
        if current_row_id.is_some() {
            if let Some(rest) = line.strip_prefix("- `") {
                let (key, value) = parse_key_value(rest).ok_or_else(|| {
                    inventory_error(format!(
                        "invalid debt row field `{line}` in {}",
                        path.display()
                    ))
                })?;
                if !REQUIRED_KEYS.contains(&key) {
                    return Err(inventory_error(format!(
                        "debt row `{}` has unknown field `{key}`",
                        current_row_id.as_deref().unwrap_or_default()
                    )));
                }
                if current_fields
                    .insert(key.to_string(), value.to_string())
                    .is_some()
                {
                    return Err(inventory_error(format!(
                        "debt row `{}` repeats field `{key}`",
                        current_row_id.as_deref().unwrap_or_default()
                    )));
                }
            }
        }
    }

    if let Some(row_id) = current_row_id.take() {
        rows.push(build_debt_row(&row_id, &current_fields)?);
    }
    if !marker_seen {
        return Err(inventory_error(format!(
            "{} is missing required contract marker `{CONTRACT_MARKER_ROW_ID}`",
            path.display()
        )));
    }
    Ok(rows)
}

pub(super) fn validate_authorizations(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    rows: &[DebtInventoryRow],
) -> Result<(), String> {
    let expected_targets = load_expected_targets(workspace_root, entry)?;
    let mut granted = BTreeMap::<(SurfaceIdentity, String, String), &str>::new();
    for row in rows {
        for target in &row.scope_target_triples {
            if !expected_targets.contains(target) {
                return Err(inventory_error(format!(
                    "debt row `{}` scope_target_triples contains target `{target}` absent from {}/RULES.json union.expected_targets",
                    row.row_id, entry.manifest_root
                )));
            }
        }
        validate_authorization_evidence(workspace_root, entry, row)?;
        for target in &row.scope_target_triples {
            let key = (
                row.identity(),
                row.authorized_at_version.clone(),
                target.clone(),
            );
            if let Some(first) = granted.insert(key, &row.row_id) {
                return Err(inventory_error(format!(
                    "debt rows `{first}` and `{}` overlap on {} version={} target={target}",
                    row.row_id,
                    row.identity().describe(),
                    row.authorized_at_version
                )));
            }
        }
    }
    Ok(())
}

pub(super) fn authorize_gap<'a>(
    gap: &TargetedGap,
    version: &str,
    rows: &'a [DebtInventoryRow],
) -> GapAuthorization<'a> {
    let mut applicable_rows = rows
        .iter()
        .filter(|row| row.identity() == gap.identity && row.authorized_at_version == version)
        .filter(|row| !row.scope_target_triples.is_disjoint(&gap.targets))
        .collect::<Vec<_>>();
    applicable_rows.sort_by(|left, right| left.row_id.cmp(&right.row_id));
    let authorized = applicable_rows
        .iter()
        .flat_map(|row| row.scope_target_triples.iter().cloned())
        .collect::<BTreeSet<_>>();
    GapAuthorization {
        applicable_rows,
        remaining_targets: gap.targets.difference(&authorized).cloned().collect(),
    }
}

pub(crate) fn load_targeted_gaps(
    version_dir: &Path,
    agent_id: &str,
    version: &str,
    targets: &BTreeSet<String>,
) -> Result<Vec<TargetedGap>, String> {
    let mut by_identity = BTreeMap::<SurfaceIdentity, BTreeSet<String>>::new();
    for target in targets {
        let path = version_dir.join(format!("coverage.{target}.json"));
        let json = read_json(&path)?;
        require_report_version(&path, &json, version)?;
        let mode = json
            .pointer("/platform_filter/mode")
            .and_then(serde_json::Value::as_str);
        let report_target = json
            .pointer("/platform_filter/target_triple")
            .and_then(serde_json::Value::as_str);
        if mode != Some("exact_target") || report_target != Some(target) {
            return Err(inventory_error(format!(
                "{} must select platform_filter.mode=exact_target and target_triple={target}",
                path.display()
            )));
        }
        let input_targets = json
            .pointer("/inputs/upstream/targets")
            .and_then(serde_json::Value::as_array)
            .and_then(|values| {
                values
                    .iter()
                    .map(|value| value.as_str().map(ToString::to_string))
                    .collect::<Option<BTreeSet<_>>>()
            });
        if input_targets != Some(BTreeSet::from([target.clone()])) {
            return Err(inventory_error(format!(
                "{} inputs.upstream.targets must contain only `{target}`",
                path.display()
            )));
        }
        let deltas = json
            .get("deltas")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| format!("{} is missing `deltas` object", path.display()))?;
        for identity in surfaces_from_report_deltas(agent_id, &path, deltas)? {
            by_identity
                .entry(identity)
                .or_default()
                .insert(target.clone());
        }
    }
    Ok(by_identity
        .into_iter()
        .map(|(identity, targets)| TargetedGap { identity, targets })
        .collect())
}

fn build_debt_row(
    row_id: &str,
    fields: &BTreeMap<String, String>,
) -> Result<DebtInventoryRow, String> {
    let get = |key: &str| {
        fields
            .get(key)
            .filter(|value| !value.trim().is_empty())
            .cloned()
            .ok_or_else(|| {
                inventory_error(format!(
                    "debt row `{row_id}` is missing or empty required field `{key}`"
                ))
            })
    };
    let blocker_class = get("blocker_class")?;
    if !ALLOWED_DEFERRALS.contains(&blocker_class.as_str()) {
        return Err(inventory_error(format!(
            "debt row `{row_id}` has invalid blocker_class `{blocker_class}`"
        )));
    }
    let scope_target_triples = parse_scope(row_id, &get("scope_target_triples")?)?;
    let authorized_at_version = get("authorized_at_version")?;
    let parsed_version = Version::parse(&authorized_at_version).map_err(|err| {
        inventory_error(format!(
            "debt row `{row_id}` has malformed authorized_at_version `{authorized_at_version}`: {err}"
        ))
    })?;
    if parsed_version.to_string() != authorized_at_version {
        return Err(inventory_error(format!(
            "debt row `{row_id}` authorized_at_version must be one canonical exact version"
        )));
    }
    Ok(DebtInventoryRow {
        row_id: row_id.to_string(),
        agent_id: get("agent_id")?,
        surface_kind: get("surface_kind")?,
        command_path: get("command_path")?,
        surface_id: get("surface_id")?,
        current_reason: get("current_reason")?,
        blocker_class,
        owner: get("owner")?,
        milestone: get("milestone")?,
        follow_on: get("follow_on")?,
        evidence_ref: get("evidence_ref")?,
        scope_target_triples,
        authorized_at_version,
        authorization_evidence_ref: get("authorization_evidence_ref")?,
    })
}

fn parse_scope(row_id: &str, value: &str) -> Result<BTreeSet<String>, String> {
    let parts = value.split(',').map(str::trim).collect::<Vec<_>>();
    if parts.is_empty() || parts.iter().any(|part| part.is_empty()) {
        return Err(inventory_error(format!(
            "debt row `{row_id}` has malformed scope_target_triples `{value}`"
        )));
    }
    let scope = parts
        .iter()
        .map(|part| (*part).to_string())
        .collect::<BTreeSet<_>>();
    if scope.len() != parts.len() {
        return Err(inventory_error(format!(
            "debt row `{row_id}` scope_target_triples contains a duplicate target"
        )));
    }
    Ok(scope)
}

#[derive(Deserialize)]
struct RulesFile {
    union: RulesUnion,
}

#[derive(Deserialize)]
struct RulesUnion {
    expected_targets: BTreeSet<String>,
}

fn load_expected_targets(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
) -> Result<BTreeSet<String>, String> {
    let path = workspace_root.join(&entry.manifest_root).join("RULES.json");
    let text = fs::read_to_string(&path)
        .map_err(|err| inventory_error(format!("read {}: {err}", path.display())))?;
    let rules = serde_json::from_str::<RulesFile>(&text)
        .map_err(|err| inventory_error(format!("parse {}: {err}", path.display())))?;
    if rules.union.expected_targets.is_empty() {
        return Err(inventory_error(format!(
            "{} union.expected_targets must not be empty",
            path.display()
        )));
    }
    Ok(rules.union.expected_targets)
}

fn validate_authorization_evidence(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    row: &DebtInventoryRow,
) -> Result<(), String> {
    let relative = Path::new(&row.authorization_evidence_ref);
    if relative.is_absolute()
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(inventory_error(format!(
            "debt row `{}` authorization_evidence_ref must be a repository-relative coverage report",
            row.row_id
        )));
    }
    let expected_prefix = Path::new(&entry.manifest_root)
        .join("reports")
        .join(&row.authorized_at_version);
    if relative.parent() != Some(expected_prefix.as_path())
        || relative
            .file_name()
            .and_then(|name| name.to_str())
            .map_or(true, |name| {
                !name.starts_with("coverage.") || !name.ends_with(".json")
            })
    {
        return Err(inventory_error(format!(
            "debt row `{}` authorization_evidence_ref `{}` is not a coverage report for agent `{}` at version `{}`",
            row.row_id, row.authorization_evidence_ref, row.agent_id, row.authorized_at_version
        )));
    }
    let path = workspace_root.join(relative);
    let json = read_json(&path)?;
    require_report_version(&path, &json, &row.authorized_at_version)?;
    let report_targets = json
        .pointer("/inputs/upstream/targets")
        .and_then(serde_json::Value::as_array)
        .and_then(|targets| {
            targets
                .iter()
                .map(|target| target.as_str().map(ToString::to_string))
                .collect::<Option<BTreeSet<_>>>()
        })
        .ok_or_else(|| {
            inventory_error(format!(
                "{} has no usable `inputs.upstream.targets`",
                path.display()
            ))
        })?;
    let deltas = json
        .get("deltas")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| inventory_error(format!("{} is missing `deltas` object", path.display())))?;
    let mut observed_targets = BTreeSet::new();
    for &(key, _, required) in &GAP_LISTS {
        let Some(rows) = deltas.get(key) else {
            if required {
                return Err(inventory_error(format!(
                    "{} is missing `deltas.{key}` array",
                    path.display()
                )));
            }
            continue;
        };
        let rows = rows.as_array().ok_or_else(|| {
            inventory_error(format!(
                "{} is missing `deltas.{key}` array",
                path.display()
            ))
        })?;
        for report_row in rows {
            let (_, identity) = surface_from_report_value(&entry.agent_id, report_row)?;
            if identity != row.identity() {
                continue;
            }
            let targets = report_row
                .get("upstream_available_on")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| {
                    inventory_error(format!(
                        "{} evidence row for `{}` is missing upstream_available_on",
                        path.display(),
                        row.row_id
                    ))
                })?;
            for target in targets {
                let target = target.as_str().ok_or_else(|| {
                    inventory_error(format!(
                        "{} evidence row upstream_available_on values must be strings",
                        path.display()
                    ))
                })?;
                if report_targets.contains(target) {
                    observed_targets.insert(target.to_string());
                }
            }
        }
    }
    if !row.scope_target_triples.is_subset(&observed_targets) {
        let excess = row
            .scope_target_triples
            .difference(&observed_targets)
            .cloned()
            .collect::<Vec<_>>();
        return Err(inventory_error(format!(
            "debt row `{}` scope exceeds surface observations in `{}`: {}",
            row.row_id,
            row.authorization_evidence_ref,
            excess.join(", ")
        )));
    }
    Ok(())
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| inventory_error(format!("read {}: {err}", path.display())))?;
    serde_json::from_str(&text)
        .map_err(|err| inventory_error(format!("parse {}: {err}", path.display())))
}

fn require_report_version(
    path: &Path,
    json: &serde_json::Value,
    expected: &str,
) -> Result<(), String> {
    let observed = json
        .pointer("/inputs/upstream/semantic_version")
        .and_then(serde_json::Value::as_str);
    if observed != Some(expected) {
        return Err(inventory_error(format!(
            "{} upstream semantic_version {:?} does not equal `{expected}`",
            path.display(),
            observed
        )));
    }
    Ok(())
}

fn inventory_error(message: impl Into<String>) -> String {
    format!("parse support debt inventory: {}", message.into())
}
