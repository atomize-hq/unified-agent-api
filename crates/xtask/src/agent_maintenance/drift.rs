use std::{
    collections::BTreeSet,
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

#[path = "drift_support.rs"]
mod shared;

use crate::{
    agent_registry::{AgentRegistry, AgentRegistryEntry, REGISTRY_RELATIVE_PATH},
    support_matrix::{
        self, validate_publication_consistency, BackendSupportState, SupportMatrixArtifact,
        SupportRow, UaaSupportState,
    },
};
use clap::Parser;
use serde::Deserialize;

const CAPABILITY_MATRIX_PATH: &str = "docs/specs/unified-agent-api/capability-matrix.md";
const SUPPORT_MATRIX_JSON_PATH: &str = "cli_manifests/support_matrix/current.json";
const SUPPORT_MATRIX_MARKDOWN_PATH: &str = "docs/specs/unified-agent-api/support-matrix.md";
const RELEASE_DOC_PATH: &str = "docs/crates-io-release.md";
const RELEASE_DOC_START_MARKER: &str =
    "<!-- generated-by: xtask onboard-agent; section: crates-io-release -->";
const RELEASE_DOC_END_MARKER: &str =
    "<!-- /generated-by: xtask onboard-agent; section: crates-io-release -->";
const SUPPORT_MARKDOWN_START_MARKER: &str = "<!-- support-matrix-published:start -->";
const SUPPORT_MARKDOWN_END_MARKER: &str = "<!-- support-matrix-published:end -->";
const WRAPPER_EVENTS_PACKAGE: &str = "unified-agent-api-wrapper-events";
const AGENT_API_PACKAGE: &str = "unified-agent-api";

const CAPABILITY_MCP_LIST_V1: &str = "agent_api.tools.mcp.list.v1";
const CAPABILITY_MCP_GET_V1: &str = "agent_api.tools.mcp.get.v1";
const CAPABILITY_MCP_ADD_V1: &str = "agent_api.tools.mcp.add.v1";
const CAPABILITY_MCP_REMOVE_V1: &str = "agent_api.tools.mcp.remove.v1";
const CAPABILITY_EXTERNAL_SANDBOX_V1: &str = "agent_api.exec.external_sandbox.v1";

#[derive(Debug, Parser, Clone)]
pub struct Args {
    #[arg(long)]
    pub agent: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DriftCategory {
    CapabilityPublication,
    GovernanceDoc,
    RegistryManifest,
    ReleaseDoc,
    SupportPublication,
}

impl DriftCategory {
    pub fn category_id(self) -> &'static str {
        match self {
            Self::RegistryManifest => "registry_manifest_drift",
            Self::CapabilityPublication => "capability_publication_drift",
            Self::SupportPublication => "support_publication_drift",
            Self::ReleaseDoc => "release_doc_drift",
            Self::GovernanceDoc => "governance_doc_drift",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriftFinding {
    pub category: DriftCategory,
    pub summary: String,
    pub surfaces: Vec<String>,
}

impl DriftFinding {
    pub fn category_id(&self) -> &'static str {
        self.category.category_id()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentDriftReport {
    pub agent_id: String,
    pub findings: Vec<DriftFinding>,
}

impl AgentDriftReport {
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }

    pub fn render(&self) -> String {
        let mut out = String::from("== AGENT DRIFT REPORT ==\n");
        out.push_str(&format!("agent_id: {}\n", self.agent_id));
        out.push_str(if self.findings.is_empty() {
            "status: clean\n"
        } else {
            "status: drift_detected\n"
        });

        for finding in &self.findings {
            out.push('\n');
            out.push_str(&format!("category_id: {}\n", finding.category_id()));
            out.push_str(&format!("summary: {}\n", finding.summary));
            out.push_str("surfaces:\n");
            for surface in &finding.surfaces {
                out.push_str(&format!("  - {surface}\n"));
            }
        }

        out
    }
}

#[derive(Debug)]
pub enum DriftCheckError {
    UnknownAgent { agent_id: String },
    Registry(String),
}

impl fmt::Display for DriftCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownAgent { agent_id } => write!(
                f,
                "agent_id `{agent_id}` does not exist in {REGISTRY_RELATIVE_PATH}"
            ),
            Self::Registry(message) => f.write_str(message),
        }
    }
}

impl DriftCheckError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::UnknownAgent { .. } | Self::Registry(_) => 2,
        }
    }
}

pub fn run(args: Args) -> Result<(), DriftCheckError> {
    let workspace_root = resolve_workspace_root().map_err(DriftCheckError::Registry)?;
    let mut stdout = io::stdout();
    run_in_workspace(&workspace_root, args, &mut stdout)
}

pub fn run_in_workspace<W: Write>(
    workspace_root: &Path,
    args: Args,
    writer: &mut W,
) -> Result<(), DriftCheckError> {
    let report = check_agent_drift(workspace_root, &args.agent)?;
    write!(writer, "{}", report.render())
        .map_err(|err| DriftCheckError::Registry(format!("write stdout: {err}")))?;
    Ok(())
}

pub fn check_agent_drift(
    workspace_root: &Path,
    agent_id: &str,
) -> Result<AgentDriftReport, DriftCheckError> {
    let registry = AgentRegistry::load(workspace_root)
        .map_err(|err| DriftCheckError::Registry(format!("load agent registry: {err}")))?;
    let entry = registry
        .find(agent_id)
        .ok_or_else(|| DriftCheckError::UnknownAgent {
            agent_id: agent_id.to_string(),
        })?;

    let support_rows = support_matrix::derive_rows(workspace_root)
        .map_err(|err| DriftCheckError::Registry(format!("derive support rows: {err}")))?;
    let expected_support_rows = support_rows
        .iter()
        .filter(|row| row.agent == agent_id)
        .cloned()
        .collect::<Vec<_>>();

    let capability_truth = shared::collect_capability_truth(entry, workspace_root);

    let mut findings = Vec::new();
    if let Some(finding) = inspect_registry_manifest(entry, workspace_root) {
        findings.push(finding);
    }
    if let Some(finding) =
        inspect_capability_publication(entry, workspace_root, capability_truth.as_ref())
    {
        findings.push(finding);
    }
    if let Some(finding) =
        inspect_support_publication(entry, workspace_root, &expected_support_rows)
    {
        findings.push(finding);
    }
    if let Some(finding) = inspect_release_doc(entry, workspace_root, &registry) {
        findings.push(finding);
    }
    if let Some(finding) = inspect_governance_docs(
        entry,
        workspace_root,
        capability_truth.as_ref(),
        &expected_support_rows,
    ) {
        findings.push(finding);
    }

    findings.sort_by(|left, right| left.category_id().cmp(right.category_id()));

    Ok(AgentDriftReport {
        agent_id: agent_id.to_string(),
        findings,
    })
}

fn resolve_workspace_root() -> Result<PathBuf, String> {
    let current_dir = std::env::current_dir().map_err(|err| format!("current_dir: {err}"))?;
    for candidate in current_dir.ancestors() {
        let cargo_toml = candidate.join("Cargo.toml");
        let Ok(text) = fs::read_to_string(&cargo_toml) else {
            continue;
        };
        if text.contains("[workspace]") {
            return Ok(candidate.to_path_buf());
        }
    }

    Err(format!(
        "could not resolve workspace root from {}",
        current_dir.display()
    ))
}

fn inspect_registry_manifest(
    entry: &AgentRegistryEntry,
    workspace_root: &Path,
) -> Option<DriftFinding> {
    let current_path = workspace_root
        .join(&entry.manifest_root)
        .join("current.json");
    let mut issues = Vec::new();

    let current = match shared::read_json::<shared::ManifestCurrent>(&current_path) {
        Ok(current) => current,
        Err(err) => {
            issues.push(err);
            return Some(build_finding(
                DriftCategory::RegistryManifest,
                "registry entry no longer matches the committed manifest root.",
                issues,
                shared::build_surfaces(
                    workspace_root,
                    [PathBuf::from(REGISTRY_RELATIVE_PATH), current_path],
                ),
            ));
        }
    };

    if current.expected_targets.is_empty() {
        issues.push(format!(
            "{} no longer declares any expected_targets",
            entry.manifest_root
        ));
    }

    let missing_targets = entry
        .canonical_targets
        .iter()
        .filter(|target| {
            !current
                .expected_targets
                .iter()
                .any(|candidate| candidate == *target)
        })
        .cloned()
        .collect::<Vec<_>>();
    if !missing_targets.is_empty() {
        issues.push(format!(
            "registry canonical targets are absent from {}: {}",
            current_path
                .strip_prefix(workspace_root)
                .unwrap_or(&current_path)
                .display(),
            missing_targets.join(", ")
        ));
    }

    if issues.is_empty() {
        None
    } else {
        Some(build_finding(
            DriftCategory::RegistryManifest,
            "registry entry no longer matches the committed manifest root.",
            issues,
            shared::build_surfaces(
                workspace_root,
                [PathBuf::from(REGISTRY_RELATIVE_PATH), current_path],
            ),
        ))
    }
}

fn inspect_capability_publication(
    entry: &AgentRegistryEntry,
    workspace_root: &Path,
    capability_truth: Result<&BTreeSet<String>, &String>,
) -> Option<DriftFinding> {
    if !entry.publication.capability_matrix_enabled {
        return None;
    }

    let mut issues = Vec::new();
    let truth = match capability_truth {
        Ok(truth) => truth,
        Err(err) => {
            issues.push(err.clone());
            return Some(build_finding(
                DriftCategory::CapabilityPublication,
                "published capability inventory no longer matches modeled backend truth.",
                issues,
                shared::build_surfaces(
                    workspace_root,
                    [
                        PathBuf::from(CAPABILITY_MATRIX_PATH),
                        PathBuf::from(&entry.backend_module),
                    ],
                ),
            ));
        }
    };

    let capability_matrix_path = workspace_root.join(CAPABILITY_MATRIX_PATH);
    let published = match shared::parse_capability_matrix_agent_support(
        &capability_matrix_path,
        &entry.agent_id,
    ) {
        Ok(published) => published,
        Err(err) => {
            issues.push(err);
            return Some(build_finding(
                DriftCategory::CapabilityPublication,
                "published capability inventory no longer matches modeled backend truth.",
                issues,
                shared::build_surfaces(
                    workspace_root,
                    [
                        PathBuf::from(CAPABILITY_MATRIX_PATH),
                        PathBuf::from(&entry.backend_module),
                    ],
                ),
            ));
        }
    };

    let missing = truth.difference(&published).cloned().collect::<Vec<_>>();
    if !missing.is_empty() {
        issues.push(format!(
            "published capability matrix is missing {} capability id(s): {}",
            missing.len(),
            missing.join(", ")
        ));
    }

    let unexpected = published.difference(truth).cloned().collect::<Vec<_>>();
    if !unexpected.is_empty() {
        issues.push(format!(
            "published capability matrix overclaims {} capability id(s): {}",
            unexpected.len(),
            unexpected.join(", ")
        ));
    }

    if issues.is_empty() {
        None
    } else {
        Some(build_finding(
            DriftCategory::CapabilityPublication,
            "published capability inventory no longer matches modeled backend truth.",
            issues,
            shared::build_surfaces(
                workspace_root,
                [
                    PathBuf::from(CAPABILITY_MATRIX_PATH),
                    PathBuf::from(&entry.backend_module),
                ],
            ),
        ))
    }
}

fn inspect_support_publication(
    entry: &AgentRegistryEntry,
    workspace_root: &Path,
    expected_rows: &[SupportRow],
) -> Option<DriftFinding> {
    if !entry.publication.support_matrix_enabled {
        return None;
    }

    let mut issues = Vec::new();
    let json_path = workspace_root.join(SUPPORT_MATRIX_JSON_PATH);
    let json_artifact = match shared::read_json::<SupportMatrixArtifact>(&json_path) {
        Ok(artifact) => artifact,
        Err(err) => {
            issues.push(err);
            return Some(build_finding(
                DriftCategory::SupportPublication,
                "published support artifacts no longer match committed support truth.",
                issues,
                shared::build_surfaces(
                    workspace_root,
                    [
                        PathBuf::from(SUPPORT_MATRIX_JSON_PATH),
                        PathBuf::from(SUPPORT_MATRIX_MARKDOWN_PATH),
                    ],
                ),
            ));
        }
    };

    if let Err(consistency_issues) =
        validate_publication_consistency(workspace_root, &json_artifact.rows)
    {
        let agent_issues = consistency_issues
            .into_iter()
            .filter(|issue| issue.agent == entry.agent_id)
            .map(|issue| issue.message)
            .collect::<Vec<_>>();
        issues.extend(agent_issues);
    }

    let published_rows = json_artifact
        .rows
        .iter()
        .filter(|row| row.agent == entry.agent_id)
        .cloned()
        .collect::<Vec<_>>();
    if published_rows != expected_rows {
        issues.push(
            "published support-matrix JSON rows do not match derived support rows".to_string(),
        );
    }

    let markdown_path = workspace_root.join(SUPPORT_MATRIX_MARKDOWN_PATH);
    let markdown = match fs::read_to_string(&markdown_path) {
        Ok(markdown) => markdown,
        Err(err) => {
            issues.push(format!("read({}): {err}", markdown_path.display()));
            return Some(build_finding(
                DriftCategory::SupportPublication,
                "published support artifacts no longer match committed support truth.",
                issues,
                shared::build_surfaces(
                    workspace_root,
                    [
                        PathBuf::from(SUPPORT_MATRIX_JSON_PATH),
                        PathBuf::from(SUPPORT_MATRIX_MARKDOWN_PATH),
                    ],
                ),
            ));
        }
    };

    let expected_section = shared::render_support_markdown_section(expected_rows);
    match shared::extract_support_markdown_section(&markdown, &entry.agent_id) {
        Ok(section) if section.trim_end() == expected_section.trim_end() => {}
        Ok(_) => issues.push(
            "published support-matrix Markdown section does not match derived support rows"
                .to_string(),
        ),
        Err(err) => issues.push(err),
    }

    if issues.is_empty() {
        None
    } else {
        Some(build_finding(
            DriftCategory::SupportPublication,
            "published support artifacts no longer match committed support truth.",
            issues,
            shared::build_surfaces(
                workspace_root,
                [
                    PathBuf::from(SUPPORT_MATRIX_JSON_PATH),
                    PathBuf::from(SUPPORT_MATRIX_MARKDOWN_PATH),
                ],
            ),
        ))
    }
}

fn inspect_release_doc(
    entry: &AgentRegistryEntry,
    workspace_root: &Path,
    registry: &AgentRegistry,
) -> Option<DriftFinding> {
    if entry.release.docs_release_track != "crates-io" {
        return None;
    }

    let release_path = workspace_root.join(RELEASE_DOC_PATH);
    let text = match fs::read_to_string(&release_path) {
        Ok(text) => text,
        Err(err) => {
            return Some(build_finding(
                DriftCategory::ReleaseDoc,
                "release guide block no longer matches the registry-backed publish order.",
                vec![format!("read({}): {err}", release_path.display())],
                shared::build_surfaces(
                    workspace_root,
                    [
                        PathBuf::from(RELEASE_DOC_PATH),
                        PathBuf::from(REGISTRY_RELATIVE_PATH),
                    ],
                ),
            ));
        }
    };

    let actual = match shared::parse_release_doc_packages(&text) {
        Ok(actual) => actual,
        Err(err) => {
            return Some(build_finding(
                DriftCategory::ReleaseDoc,
                "release guide block no longer matches the registry-backed publish order.",
                vec![err],
                shared::build_surfaces(
                    workspace_root,
                    [
                        PathBuf::from(RELEASE_DOC_PATH),
                        PathBuf::from(REGISTRY_RELATIVE_PATH),
                    ],
                ),
            ));
        }
    };

    let expected = shared::registry_release_packages(registry, &entry.release.docs_release_track);
    let mut issues = Vec::new();
    match package_index(&expected, &entry.package_name) {
        Some(expected_index) => {
            match package_index(&actual.published_crates, &entry.package_name) {
                Some(index) if index == expected_index => {}
                Some(index) => issues.push(format!(
                    "published crates block lists `{}` at position {}, expected {}",
                    entry.package_name,
                    index + 1,
                    expected_index + 1
                )),
                None => issues.push(format!(
                    "published crates block is missing `{}`",
                    entry.package_name
                )),
            }
            match package_index(&actual.publish_order, &entry.package_name) {
                Some(index) if index == expected_index => {}
                Some(index) => issues.push(format!(
                    "publish order block lists `{}` at position {}, expected {}",
                    entry.package_name,
                    index + 1,
                    expected_index + 1
                )),
                None => issues.push(format!(
                    "publish order block is missing `{}`",
                    entry.package_name
                )),
            }
        }
        None => issues.push(format!(
            "registry-derived publish order does not include `{}`",
            entry.package_name
        )),
    }

    if issues.is_empty() {
        None
    } else {
        Some(build_finding(
            DriftCategory::ReleaseDoc,
            "release guide block no longer matches the registry-backed publish order.",
            issues,
            shared::build_surfaces(
                workspace_root,
                [
                    PathBuf::from(RELEASE_DOC_PATH),
                    PathBuf::from(REGISTRY_RELATIVE_PATH),
                ],
            ),
        ))
    }
}

fn inspect_governance_docs(
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

fn closed_governance_surfaces(workspace_root: &Path, entry: &AgentRegistryEntry) -> BTreeSet<String> {
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

fn build_finding(
    category: DriftCategory,
    summary: &str,
    details: Vec<String>,
    mut surfaces: Vec<String>,
) -> DriftFinding {
    surfaces.sort();
    surfaces.dedup();
    let detail_suffix = if details.is_empty() {
        String::new()
    } else {
        format!(" {}", details.join(" "))
    };
    DriftFinding {
        category,
        summary: format!("{summary}{detail_suffix}"),
        surfaces,
    }
}

fn package_index(packages: &[String], package_name: &str) -> Option<usize> {
    packages
        .iter()
        .position(|candidate| candidate == package_name)
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
