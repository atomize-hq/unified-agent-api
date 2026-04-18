use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use semver::Version;
use serde::Deserialize;

use crate::wrapper_coverage_shared::RootIntakeLayout;

use super::{
    BackendSupportState, ManifestSupportState, PointerPromotionState, SupportRow, UaaSupportState,
    CURRENT_AGENT_ROOTS,
};

#[derive(Debug, Clone)]
pub(super) struct AgentRoot {
    pub(super) agent: String,
    pub(super) root: PathBuf,
}

#[derive(Debug, Clone)]
pub(super) struct LoadedAgentRoot {
    pub(super) agent: String,
    pub(super) posture: CurrentRootPosture,
    pub(super) pointers: PointerSet,
    pub(super) layout: RootIntakeLayout,
    pub(super) versions: Vec<VersionMetadata>,
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

pub(crate) fn derive_rows(workspace_root: &Path) -> Result<Vec<SupportRow>, String> {
    let roots = CURRENT_AGENT_ROOTS
        .iter()
        .map(|(agent, rel_root)| AgentRoot {
            agent: (*agent).to_string(),
            root: workspace_root.join(rel_root),
        })
        .collect::<Vec<_>>();
    derive_rows_for_roots(&roots)
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn derive_rows_for_test_roots(
    workspace_root: &Path,
    roots: &[(&str, &str)],
) -> Result<Vec<SupportRow>, String> {
    let roots = roots
        .iter()
        .map(|(agent, rel_root)| AgentRoot {
            agent: (*agent).to_string(),
            root: workspace_root.join(rel_root),
        })
        .collect::<Vec<_>>();
    derive_rows_for_roots(&roots)
}

fn derive_rows_for_roots(roots: &[AgentRoot]) -> Result<Vec<SupportRow>, String> {
    let loaded_roots = roots
        .iter()
        .map(load_agent_root)
        .collect::<Result<Vec<_>, _>>()?;
    derive_rows_for_loaded_roots(&loaded_roots)
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
    let mut rows = Vec::new();

    for metadata in &root.versions {
        let supported_targets = metadata
            .coverage
            .supported_targets
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();

        for target in &root.posture.expected_targets {
            let manifest_support = if supported_targets.contains(target) {
                ManifestSupportState::Supported
            } else {
                ManifestSupportState::Unsupported
            };

            let report = load_support_report(&root.layout, &metadata.semantic_version, target)?;
            let backend_support = classify_backend_support(report.as_ref());
            let pointer_promotion =
                classify_pointer_promotion(&root.pointers, target, &metadata.semantic_version);
            let evidence_notes = build_evidence_notes(
                report.as_ref(),
                &root.posture,
                target,
                &metadata.semantic_version,
            );
            let uaa_support =
                classify_uaa_support(manifest_support, backend_support, &evidence_notes);

            rows.push(SupportRow {
                agent: root.agent.clone(),
                version: metadata.semantic_version.clone(),
                target: target.clone(),
                manifest_support,
                backend_support,
                uaa_support,
                pointer_promotion,
                evidence_notes,
            });
        }
    }

    Ok(rows)
}

pub(super) fn load_agent_root(root: &AgentRoot) -> Result<LoadedAgentRoot, String> {
    let posture = read_current_root_posture(&root.root)?;
    let pointers = read_pointers(&root.root, &posture.expected_targets)?;
    let layout = RootIntakeLayout::new(root.root.clone());
    let versions = read_version_metadata(&layout)?;

    Ok(LoadedAgentRoot {
        agent: root.agent.clone(),
        posture,
        pointers,
        layout,
        versions,
    })
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

fn read_pointers(root: &Path, expected_targets: &[String]) -> Result<PointerSet, String> {
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
