use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use semver::Version;
use serde::Deserialize;

use agent_api::RuntimeSupportRecord;

use crate::agent_registry::AgentRegistry;
use crate::depth_record::{self, DepthRecord};
use crate::root_intake_layout::RootIntakeLayout;

use super::runtime_support::{
    derive_runtime_support_projection_for_loaded_roots,
    derive_validated_runtime_support_for_loaded_roots,
    render_agent_api_runtime_support_data_from_projection, EMBEDDED_RUNTIME_SUPPORT_FAMILIES,
};
use super::{
    depth, BackendSupportState, ManifestSupportState, PointerPromotionState, SupportRow,
    UaaSupportState,
};

#[derive(Debug, Clone)]
pub(super) struct AgentRoot {
    pub(super) agent: String,
    pub(super) root: PathBuf,
    /// Evidence a depth record binds is named relative to the workspace.
    workspace: PathBuf,
}

impl AgentRoot {
    fn new(workspace_root: &Path, agent: &str, manifest_root: &str) -> Self {
        Self {
            agent: agent.to_string(),
            root: workspace_root.join(manifest_root),
            workspace: workspace_root.to_path_buf(),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct LoadedAgentRoot {
    pub(super) agent: String,
    pub(super) posture: CurrentRootPosture,
    pub(super) pointers: PointerSet,
    pub(super) layout: RootIntakeLayout,
    pub(super) versions: Vec<VersionMetadata>,
    records: BTreeMap<String, DepthRecord>,
    workspace: PathBuf,
}

#[derive(Debug, Deserialize)]
struct CurrentUnion {
    #[serde(default)]
    expected_targets: Vec<String>,
    #[serde(default)]
    inputs: Vec<CurrentUnionInput>,
}

#[derive(Debug, Deserialize)]
struct CurrentUnionInput {
    target_triple: String,
    #[serde(default)]
    binary: CurrentUnionBinary,
}

#[derive(Debug, Default, Deserialize)]
struct CurrentUnionBinary {
    semantic_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct VersionMetadata {
    pub(super) semantic_version: String,
    #[serde(default)]
    pub(super) status: Option<String>,
    #[serde(default)]
    coverage: VersionCoverage,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct VersionCoverage {
    #[serde(default)]
    supported_targets: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct SupportReport {
    #[serde(default)]
    inputs: SupportReportInputs,
    #[serde(default)]
    deltas: SupportReportDeltas,
}

#[derive(Debug, Default, Deserialize)]
struct SupportReportInputs {
    #[serde(default)]
    upstream: SupportReportUpstream,
}

#[derive(Debug, Default, Deserialize)]
struct SupportReportUpstream {
    #[serde(default)]
    targets: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct SupportReportDeltas {
    #[serde(default)]
    missing_commands: Vec<serde_json::Value>,
    #[serde(default)]
    missing_flags: Vec<serde_json::Value>,
    #[serde(default)]
    missing_args: Vec<serde_json::Value>,
    #[serde(default)]
    passthrough_candidates: Vec<serde_json::Value>,
    #[serde(default)]
    intentionally_unsupported: Vec<serde_json::Value>,
    #[serde(default)]
    wrapper_only_commands: Vec<serde_json::Value>,
    #[serde(default)]
    wrapper_only_flags: Vec<serde_json::Value>,
    #[serde(default)]
    wrapper_only_args: Vec<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub(super) struct CurrentRootPosture {
    pub(super) expected_targets: Vec<String>,
    pub(super) current_version: Option<String>,
    pub(super) current_targets: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub(super) struct PointerSet {
    pub(super) latest_supported: BTreeMap<String, Option<String>>,
    pub(super) latest_validated: BTreeMap<String, Option<String>>,
}

pub fn derive_rows(workspace_root: &Path) -> Result<Vec<SupportRow>, String> {
    let roots = enrolled_agent_roots(workspace_root)?;
    derive_rows_for_roots(&roots)
}

pub fn derive_validated_runtime_support(
    workspace_root: &Path,
) -> Result<Vec<RuntimeSupportRecord>, String> {
    let roots = enrolled_agent_roots(workspace_root)?;
    derive_validated_runtime_support_for_roots(&roots)
}

pub fn render_agent_api_runtime_support_data(workspace_root: &Path) -> Result<String, String> {
    let roots = enrolled_agent_roots(workspace_root)?;
    render_agent_api_runtime_support_data_for_roots(&roots)
}

pub fn render_agent_api_runtime_support_data_for_agent_root(
    workspace_root: &Path,
    agent: &str,
    manifest_root: &str,
) -> Result<String, String> {
    let roots = [AgentRoot::new(workspace_root, agent, manifest_root)];
    render_agent_api_runtime_support_data_for_roots(&roots)
}

pub fn derive_rows_for_agent_root(
    workspace_root: &Path,
    agent: &str,
    manifest_root: &str,
) -> Result<Vec<SupportRow>, String> {
    let roots = [AgentRoot::new(workspace_root, agent, manifest_root)];
    derive_rows_for_roots(&roots)
}

pub fn derive_validated_runtime_support_for_agent_root(
    workspace_root: &Path,
    agent: &str,
    manifest_root: &str,
) -> Result<Vec<RuntimeSupportRecord>, String> {
    let roots = [AgentRoot::new(workspace_root, agent, manifest_root)];
    derive_validated_runtime_support_for_roots(&roots)
}

#[doc(hidden)]
pub fn render_agent_api_runtime_support_data_for_test_roots(
    workspace_root: &Path,
    roots: &[(&str, &str)],
) -> Result<String, String> {
    let roots = roots
        .iter()
        .map(|(agent, rel_root)| AgentRoot::new(workspace_root, agent, rel_root))
        .collect::<Vec<_>>();
    render_agent_api_runtime_support_data_for_roots(&roots)
}

#[doc(hidden)]
pub fn derive_rows_for_test_roots(
    workspace_root: &Path,
    roots: &[(&str, &str)],
) -> Result<Vec<SupportRow>, String> {
    let roots = roots
        .iter()
        .map(|(agent, rel_root)| AgentRoot::new(workspace_root, agent, rel_root))
        .collect::<Vec<_>>();
    derive_rows_for_roots(&roots)
}

#[doc(hidden)]
pub fn derive_validated_runtime_support_for_test_roots(
    workspace_root: &Path,
    roots: &[(&str, &str)],
) -> Result<Vec<RuntimeSupportRecord>, String> {
    let roots = roots
        .iter()
        .map(|(agent, rel_root)| AgentRoot::new(workspace_root, agent, rel_root))
        .collect::<Vec<_>>();
    derive_validated_runtime_support_for_roots(&roots)
}

pub fn runtime_support_family_is_embedded(runtime_family: &str) -> bool {
    EMBEDDED_RUNTIME_SUPPORT_FAMILIES.contains(&runtime_family)
}

fn derive_rows_for_roots(roots: &[AgentRoot]) -> Result<Vec<SupportRow>, String> {
    let loaded_roots = roots
        .iter()
        .map(load_agent_root)
        .collect::<Result<Vec<_>, _>>()?;
    derive_rows_for_loaded_roots(&loaded_roots)
}

fn derive_validated_runtime_support_for_roots(
    roots: &[AgentRoot],
) -> Result<Vec<RuntimeSupportRecord>, String> {
    let loaded_roots = roots
        .iter()
        .map(load_agent_root)
        .collect::<Result<Vec<_>, _>>()?;
    derive_validated_runtime_support_for_loaded_roots(&loaded_roots)
}

fn render_agent_api_runtime_support_data_for_roots(roots: &[AgentRoot]) -> Result<String, String> {
    let loaded_roots = roots
        .iter()
        .map(load_agent_root)
        .collect::<Result<Vec<_>, _>>()?;
    let projections = derive_runtime_support_projection_for_loaded_roots(&loaded_roots)?;
    Ok(render_agent_api_runtime_support_data_from_projection(
        &projections,
        EMBEDDED_RUNTIME_SUPPORT_FAMILIES,
    ))
}

pub(super) fn enrolled_agent_roots(workspace_root: &Path) -> Result<Vec<AgentRoot>, String> {
    let registry =
        AgentRegistry::load(workspace_root).map_err(|err| format!("load agent registry: {err}"))?;

    Ok(registry
        .support_matrix_entries()
        .map(|entry| AgentRoot::new(workspace_root, &entry.agent_id, &entry.manifest_root))
        .collect())
}

pub(super) fn derive_rows_for_loaded_roots(
    roots: &[LoadedAgentRoot],
) -> Result<Vec<SupportRow>, String> {
    let mut rows = Vec::new();
    for root in roots {
        rows.extend(derive_rows_for_loaded_root(root)?);
    }

    rows.sort_by(compare_rows);
    Ok(rows)
}

fn derive_rows_for_loaded_root(root: &LoadedAgentRoot) -> Result<Vec<SupportRow>, String> {
    let metadata = root
        .versions
        .iter()
        .map(|metadata| (metadata.semantic_version.as_str(), metadata))
        .collect::<BTreeMap<_, _>>();
    let versions = metadata
        .keys()
        .copied()
        .chain(root.records.keys().map(String::as_str))
        .collect::<BTreeSet<_>>();
    let evidence = |path: &str| depth_record::file_sha256(&root.workspace.join(path));
    let mut rows = Vec::new();

    for version in versions {
        let metadata = metadata.get(version);
        let record = root.records.get(version);
        // Support depth rule 4: a target the record covers implies a row, whether or not the
        // version has metadata and whether or not `expected_targets` lists the target.
        let mut targets = record.map(depth::covered_targets).unwrap_or_default();
        if metadata.is_some() {
            targets.extend(root.posture.expected_targets.iter().map(String::as_str));
        }

        for target in targets {
            let manifest_support = match metadata {
                Some(metadata)
                    if metadata
                        .coverage
                        .supported_targets
                        .iter()
                        .any(|t| t == target) =>
                {
                    ManifestSupportState::Supported
                }
                _ => ManifestSupportState::Unsupported,
            };

            let report = load_support_report(&root.layout, version, target)?;
            let backend_support = classify_backend_support(report.as_ref());
            let pointer_promotion = classify_pointer_promotion(&root.pointers, target, version);
            let evidence_notes =
                build_evidence_notes(report.as_ref(), &root.posture, target, version);
            // ponytail: `declared` is false until the registry admits a depth declaration, which
            // route wiring brings; rule 4's declared rows come with it.
            let depth =
                depth::derive_depth_facts(record, false, target, &evidence).map_err(|err| {
                    format!(
                        "agent `{}` version `{version}` target `{target}`: {err}",
                        root.agent
                    )
                })?;
            let uaa_support = depth::apply_depth(
                classify_uaa_support(manifest_support, backend_support, &evidence_notes),
                depth.as_ref(),
            );

            rows.push(SupportRow {
                agent: root.agent.clone(),
                version: version.to_string(),
                target: target.to_string(),
                manifest_support,
                backend_support,
                uaa_support,
                pointer_promotion,
                evidence_notes,
                depth,
            });
        }
    }

    Ok(rows)
}

pub(super) fn load_agent_root(root: &AgentRoot) -> Result<LoadedAgentRoot, String> {
    let posture = read_current_root_posture(&root.root)?;
    let records = read_depth_records(&root.root, &root.agent)?;
    let covered = records
        .values()
        .flat_map(depth::covered_targets)
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let pointers = read_pointers(&root.root, &posture.expected_targets, &covered)?;
    let layout = RootIntakeLayout::new(root.root.clone());
    let versions = read_version_metadata(&layout)?;

    Ok(LoadedAgentRoot {
        agent: root.agent.clone(),
        posture,
        pointers,
        layout,
        versions,
        records,
        workspace: root.workspace.clone(),
    })
}

/// The root's depth records by version. A record that does not read fails publication rather than
/// leaving its version's rows without depth facts (Support depth rule 3), and so does one in a root
/// whose `RULES.json` does not adopt Annex B (record invariant 1).
fn read_depth_records(root: &Path, agent: &str) -> Result<BTreeMap<String, DepthRecord>, String> {
    let files = depth_record::record_files(root);
    if files.is_empty() {
        return Ok(BTreeMap::new());
    }
    let rules = fs::read_to_string(root.join("RULES.json"))
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok());
    let adopted = rules.is_some_and(|rules| {
        rules
            .pointer("/storage/committed/reports/depth_record")
            .is_some_and(serde_json::Value::is_string)
    });
    if !adopted {
        return Err(format!(
            "{}: holds a depth record, and its RULES.json names no storage.committed.reports.depth_record, so the root does not adopt Annex B",
            root.display()
        ));
    }
    files
        .into_iter()
        .map(|(version, path)| {
            let record = fs::read_to_string(&path)
                .map_err(|err| err.to_string())
                .and_then(|raw| DepthRecord::parse(&raw, agent, &version))
                .map_err(|err| format!("{}: {err}", path.display()))?;
            Ok((version, record))
        })
        .collect()
}

fn read_version_metadata(layout: &RootIntakeLayout) -> Result<Vec<VersionMetadata>, String> {
    let versions_dir = layout.versions_dir();
    let mut versions = Vec::new();

    for entry in fs::read_dir(&versions_dir)
        .map_err(|err| format!("read_dir({}): {err}", versions_dir.display()))?
    {
        let entry = entry.map_err(|err| format!("read_dir entry error: {err}"))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        versions.push(read_json(&path)?);
    }

    Ok(versions)
}

fn read_current_root_posture(root: &Path) -> Result<CurrentRootPosture, String> {
    let current_path = root.join("current.json");
    let current: CurrentUnion = read_json(&current_path)?;
    if current.expected_targets.is_empty() {
        return Err(format!(
            "{}: expected_targets must not be empty",
            current_path.display()
        ));
    }

    let mut versions = BTreeSet::new();
    let current_targets = current
        .inputs
        .iter()
        .map(|input| {
            if let Some(version) = input.binary.semantic_version.clone() {
                versions.insert(version);
            }
            input.target_triple.clone()
        })
        .collect::<BTreeSet<_>>();

    let current_version = match versions.len() {
        0 => None,
        1 => versions.into_iter().next(),
        _ => {
            return Err(format!(
                "{}: current.json contained multiple semantic versions",
                current_path.display()
            ));
        }
    };

    Ok(CurrentRootPosture {
        expected_targets: current.expected_targets,
        current_version,
        current_targets,
    })
}

/// The pointers of every expected target, whose files must exist, and of each other target a
/// depth record covers, which has no pointer when it has no file (Support depth rule 4).
fn read_pointers(
    root: &Path,
    expected_targets: &[String],
    covered: &BTreeSet<String>,
) -> Result<PointerSet, String> {
    let layout = RootIntakeLayout::new(root.to_path_buf());
    let mut latest_supported = BTreeMap::new();
    let mut latest_validated = BTreeMap::new();

    for target in expected_targets {
        latest_supported.insert(
            target.clone(),
            read_pointer_file(&layout.latest_supported_pointer_path(target))?,
        );
        latest_validated.insert(
            target.clone(),
            read_pointer_file(&layout.latest_validated_pointer_path(target))?,
        );
    }
    for target in covered.iter().filter(|t| !expected_targets.contains(t)) {
        for (pointers, path) in [
            (
                &mut latest_supported,
                layout.latest_supported_pointer_path(target),
            ),
            (
                &mut latest_validated,
                layout.latest_validated_pointer_path(target),
            ),
        ] {
            if path.exists() {
                pointers.insert(target.clone(), read_pointer_file(&path)?);
            }
        }
    }

    Ok(PointerSet {
        latest_supported,
        latest_validated,
    })
}

fn read_pointer_file(path: &Path) -> Result<Option<String>, String> {
    let raw = fs::read_to_string(path).map_err(|err| format!("read({}): {err}", path.display()))?;
    let value = raw.trim();
    if value.is_empty() || value == "none" {
        Ok(None)
    } else {
        Ok(Some(value.to_string()))
    }
}

pub(super) fn load_support_report(
    layout: &RootIntakeLayout,
    version: &str,
    target: &str,
) -> Result<Option<SupportReport>, String> {
    let version_dir = layout.reports_version_dir(version);
    let exact_target = version_dir.join(format!("coverage.{target}.json"));
    if exact_target.exists() {
        return read_json(&exact_target).map(Some);
    }

    for candidate in [
        version_dir.join("coverage.any.json"),
        version_dir.join("coverage.all.json"),
    ] {
        if !candidate.exists() {
            continue;
        }

        let report: SupportReport = read_json(&candidate)?;
        if report_mentions_target(&report, target) {
            return Ok(Some(report));
        }
    }
    Ok(None)
}

fn report_mentions_target(report: &SupportReport, target: &str) -> bool {
    report.inputs.upstream.targets.is_empty()
        || report
            .inputs
            .upstream
            .targets
            .iter()
            .any(|candidate| candidate == target)
}

fn classify_backend_support(report: Option<&SupportReport>) -> BackendSupportState {
    let Some(report) = report else {
        return BackendSupportState::Unsupported;
    };

    if report_has_missing_surface(report) {
        return BackendSupportState::Unsupported;
    }

    if report_has_partial_caveat(report) {
        return BackendSupportState::Partial;
    }

    BackendSupportState::Supported
}

fn report_has_missing_surface(report: &SupportReport) -> bool {
    !report.deltas.missing_commands.is_empty()
        || !report.deltas.missing_flags.is_empty()
        || !report.deltas.missing_args.is_empty()
}

fn report_has_partial_caveat(report: &SupportReport) -> bool {
    !report.deltas.intentionally_unsupported.is_empty()
        || !report.deltas.wrapper_only_commands.is_empty()
        || !report.deltas.wrapper_only_flags.is_empty()
        || !report.deltas.wrapper_only_args.is_empty()
}

pub(super) fn classify_pointer_promotion(
    pointers: &PointerSet,
    target: &str,
    version: &str,
) -> PointerPromotionState {
    let latest_supported = pointers
        .latest_supported
        .get(target)
        .and_then(|value| value.as_deref())
        == Some(version);
    let latest_validated = pointers
        .latest_validated
        .get(target)
        .and_then(|value| value.as_deref())
        == Some(version);

    match (latest_supported, latest_validated) {
        (true, true) => PointerPromotionState::LatestSupportedAndValidated,
        (true, false) => PointerPromotionState::LatestSupported,
        (false, true) => PointerPromotionState::LatestValidated,
        (false, false) => PointerPromotionState::None,
    }
}

pub(super) fn build_evidence_notes(
    report: Option<&SupportReport>,
    posture: &CurrentRootPosture,
    target: &str,
    version: &str,
) -> Vec<String> {
    let mut notes = Vec::new();

    if let Some(report) = report {
        if !report.deltas.intentionally_unsupported.is_empty() {
            notes.push(
                "backend report includes intentionally unsupported surface outside unified support"
                    .to_string(),
            );
        }
        if !report.deltas.passthrough_candidates.is_empty() {
            notes.push(
                "backend report includes passthrough surface outside unified support".to_string(),
            );
        }
        if !report.deltas.wrapper_only_commands.is_empty()
            || !report.deltas.wrapper_only_flags.is_empty()
            || !report.deltas.wrapper_only_args.is_empty()
        {
            notes.push(
                "backend report includes backend-only surface outside unified support".to_string(),
            );
        }
    }

    if posture.current_version.as_deref() == Some(version)
        && !posture.current_targets.contains(target)
    {
        notes.push("current root snapshot omits this target".to_string());
    }

    notes
}

fn classify_uaa_support(
    manifest_support: ManifestSupportState,
    backend_support: BackendSupportState,
    evidence_notes: &[String],
) -> UaaSupportState {
    match (manifest_support, backend_support) {
        (ManifestSupportState::Supported, BackendSupportState::Supported)
            if evidence_notes.is_empty() =>
        {
            UaaSupportState::Supported
        }
        (ManifestSupportState::Supported, BackendSupportState::Unsupported)
        | (ManifestSupportState::Unsupported, BackendSupportState::Partial)
        | (ManifestSupportState::Unsupported, BackendSupportState::Supported)
        | (ManifestSupportState::Unsupported, BackendSupportState::Unsupported) => {
            UaaSupportState::Unsupported
        }
        _ => UaaSupportState::Partial,
    }
}

fn compare_rows(left: &SupportRow, right: &SupportRow) -> Ordering {
    left.agent
        .cmp(&right.agent)
        .then_with(|| left.target.cmp(&right.target))
        .then_with(|| compare_semver_desc(&left.version, &right.version))
}

fn compare_semver_desc(left: &str, right: &str) -> Ordering {
    match (Version::parse(left), Version::parse(right)) {
        (Ok(left), Ok(right)) => right.cmp(&left),
        _ => right.cmp(left),
    }
}

fn read_json<T>(path: &Path) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    let text =
        fs::read_to_string(path).map_err(|err| format!("read({}): {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("parse({}): {err}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn evidence_notes_include_passthrough_caveat_in_stable_order() {
        let mut report = SupportReport::default();
        report
            .deltas
            .intentionally_unsupported
            .push(json!({ "path": ["unsupported"] }));
        report
            .deltas
            .passthrough_candidates
            .push(json!({ "path": ["app-server"] }));
        report
            .deltas
            .wrapper_only_commands
            .push(json!({ "path": ["backend-only"] }));

        let posture = CurrentRootPosture {
            expected_targets: vec!["linux-x64".to_string(), "win32-x64".to_string()],
            current_version: Some("1.0.0".to_string()),
            current_targets: BTreeSet::from(["linux-x64".to_string()]),
        };

        let notes = build_evidence_notes(Some(&report), &posture, "win32-x64", "1.0.0");

        assert_eq!(
            notes,
            vec![
                "backend report includes intentionally unsupported surface outside unified support"
                    .to_string(),
                "backend report includes passthrough surface outside unified support".to_string(),
                "backend report includes backend-only surface outside unified support".to_string(),
                "current root snapshot omits this target".to_string(),
            ]
        );
    }
}
