mod governance;
mod publication;
mod runtime_evidence;
mod shared;

use std::{
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::{
    agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH},
    support_matrix,
};
use clap::Parser;

use super::finding_signature::FindingSignature;

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
    RuntimeEvidence,
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
            Self::RuntimeEvidence => "runtime_evidence_drift",
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

    pub fn signature(&self) -> FindingSignature {
        FindingSignature::new(self.category_id(), &self.surfaces)
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
    Validation(String),
    Internal(String),
}

impl fmt::Display for DriftCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(message) | Self::Internal(message) => f.write_str(message),
        }
    }
}

impl DriftCheckError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Validation(_) => 2,
            Self::Internal(_) => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriftCheckOutcome {
    Clean(AgentDriftReport),
    DriftDetected(AgentDriftReport),
}

impl DriftCheckOutcome {
    pub fn report(&self) -> &AgentDriftReport {
        match self {
            Self::Clean(report) | Self::DriftDetected(report) => report,
        }
    }
}

pub fn run(args: Args) -> Result<DriftCheckOutcome, DriftCheckError> {
    let workspace_root = resolve_workspace_root().map_err(DriftCheckError::Internal)?;
    let mut stdout = io::stdout();
    run_in_workspace(&workspace_root, args, &mut stdout)
}

pub fn run_in_workspace<W: Write>(
    workspace_root: &Path,
    args: Args,
    writer: &mut W,
) -> Result<DriftCheckOutcome, DriftCheckError> {
    let report = check_agent_drift(workspace_root, &args.agent)?;
    write!(writer, "{}", report.render())
        .map_err(|err| DriftCheckError::Internal(format!("write stdout: {err}")))?;
    if report.is_clean() {
        Ok(DriftCheckOutcome::Clean(report))
    } else {
        Ok(DriftCheckOutcome::DriftDetected(report))
    }
}

pub fn check_agent_drift(
    workspace_root: &Path,
    agent_id: &str,
) -> Result<AgentDriftReport, DriftCheckError> {
    let registry = AgentRegistry::load(workspace_root)
        .map_err(|err| DriftCheckError::Validation(format!("load agent registry: {err}")))?;
    let entry = registry.find(agent_id).ok_or_else(|| {
        DriftCheckError::Validation(format!(
            "agent_id `{agent_id}` does not exist in {REGISTRY_RELATIVE_PATH}"
        ))
    })?;

    let expected_support_rows = if entry.publication.support_matrix_enabled {
        support_matrix::derive_rows_for_agent_root(
            workspace_root,
            &entry.agent_id,
            &entry.manifest_root,
        )
        .map_err(|err| format!("derive support rows: {err}"))
    } else {
        Ok(Vec::new())
    };

    let capability_truth = shared::collect_capability_truth(entry, workspace_root);

    let mut findings = Vec::new();
    if let Some(finding) = publication::inspect_registry_manifest(entry, workspace_root) {
        findings.push(finding);
    }
    if let Some(finding) = publication::inspect_capability_publication(
        entry,
        workspace_root,
        capability_truth.as_ref(),
    ) {
        findings.push(finding);
    }
    if let Some(finding) = publication::inspect_support_publication(
        entry,
        workspace_root,
        expected_support_rows.as_ref(),
    ) {
        findings.push(finding);
    }
    if let Some(finding) = publication::inspect_release_doc(entry, workspace_root, &registry) {
        findings.push(finding);
    }
    let runtime_integrated =
        runtime_evidence::has_runtime_integrated_lifecycle(entry, workspace_root);
    if let Some(finding) = runtime_evidence::inspect_runtime_evidence(entry, workspace_root) {
        findings.push(finding);
    }
    if !runtime_integrated {
        if let Some(finding) = governance::inspect_governance_docs(
            entry,
            workspace_root,
            capability_truth.as_ref(),
            expected_support_rows.as_ref(),
        ) {
            findings.push(finding);
        }
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

pub(super) fn build_finding(
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
