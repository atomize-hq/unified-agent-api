use std::{fs, path::Path};

use sha2::{Digest, Sha256};

use crate::agent_registry::{
    AgentRegistryEntry, ReleaseWatchDispatchKind, ReleaseWatchMetadata, ReleaseWatchSourceKind,
    ReleaseWatchVersionPolicy,
};

use super::request::{
    DetectedRelease, ExecutionContract, ExecutionContractRecovery, MaintenanceRequest,
};

pub(crate) const GENERATED_BY_WORKFLOW: &str =
    ".github/workflows/agent-maintenance-release-watch.yml";
pub(crate) const GENERIC_PACKET_PR_WORKFLOW: &str = "agent-maintenance-open-pr.yml";
pub(crate) const LEGACY_EXECUTOR_ALIAS: &str = "codex";
pub(crate) const EXECUTE_HOST_SURFACE: &str = "execute-agent-maintenance";
pub(crate) const EXECUTION_HOST_LABEL: &str = "local Codex CLI host via execute-agent-maintenance";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DerivedDetectedReleaseFields {
    pub(crate) version_policy: String,
    pub(crate) source_kind: String,
    pub(crate) source_ref: String,
    pub(crate) dispatch_kind: String,
    pub(crate) dispatch_workflow: String,
}

pub(crate) fn derive_detected_release_fields(
    agent_id: &str,
    release_watch: &ReleaseWatchMetadata,
) -> Result<DerivedDetectedReleaseFields, String> {
    Ok(DerivedDetectedReleaseFields {
        version_policy: version_policy_str(release_watch.version_policy).to_string(),
        source_kind: source_kind_str(release_watch.upstream.source_kind).to_string(),
        source_ref: source_ref(release_watch),
        dispatch_kind: dispatch_kind_str(release_watch.dispatch_kind).to_string(),
        dispatch_workflow: dispatch_workflow_value(agent_id, release_watch)?,
    })
}

pub(crate) fn opened_from_path(dispatch_workflow: &str) -> String {
    format!(".github/workflows/{dispatch_workflow}")
}

pub(crate) fn dispatch_workflow_value(
    agent_id: &str,
    release_watch: &ReleaseWatchMetadata,
) -> Result<String, String> {
    match release_watch.dispatch_kind {
        ReleaseWatchDispatchKind::WorkflowDispatch => {
            release_watch
                .dispatch_workflow
                .clone()
                .ok_or_else(|| {
                    format!(
                        "maintenance release-watch requires dispatch_workflow for agent `{agent_id}` when dispatch_kind = workflow_dispatch"
                    )
                })
        }
        ReleaseWatchDispatchKind::PacketPr => Ok(GENERIC_PACKET_PR_WORKFLOW.to_string()),
    }
}

pub(crate) fn dispatch_kind_str(kind: ReleaseWatchDispatchKind) -> &'static str {
    match kind {
        ReleaseWatchDispatchKind::WorkflowDispatch => "workflow_dispatch",
        ReleaseWatchDispatchKind::PacketPr => "packet_pr",
    }
}

pub(crate) fn version_policy_str(value: ReleaseWatchVersionPolicy) -> &'static str {
    match value {
        ReleaseWatchVersionPolicy::LatestStableMinusOne => "latest_stable_minus_one",
    }
}

pub(crate) fn source_kind_str(kind: ReleaseWatchSourceKind) -> &'static str {
    match kind {
        ReleaseWatchSourceKind::GithubReleases => "github_releases",
        ReleaseWatchSourceKind::GcsObjectListing => "gcs_object_listing",
    }
}

pub(crate) fn source_ref(release_watch: &ReleaseWatchMetadata) -> String {
    match release_watch.upstream.source_kind {
        ReleaseWatchSourceKind::GithubReleases => format!(
            "{}/{}",
            release_watch.upstream.owner.as_deref().unwrap_or(""),
            release_watch.upstream.repo.as_deref().unwrap_or("")
        ),
        ReleaseWatchSourceKind::GcsObjectListing => format!(
            "{}/{}",
            release_watch.upstream.bucket.as_deref().unwrap_or(""),
            release_watch.upstream.prefix.as_deref().unwrap_or("")
        ),
    }
}

pub(crate) fn build_execution_contract_for_request(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    request: &MaintenanceRequest,
) -> Result<ExecutionContract, String> {
    let detected_release = request.detected_release.as_ref().ok_or_else(|| {
        format!(
            "automated maintenance request `{}` is missing detected_release metadata",
            request.relative_path
        )
    })?;
    build_execution_contract(
        workspace_root,
        entry,
        &request.relative_path,
        &request.maintenance_root,
        &request.opened_from,
        &detected_release.target_version,
        &detected_release.branch_name,
    )
}

pub(crate) fn build_execution_contract(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    request_path: &str,
    maintenance_root: &str,
    opened_from: &str,
    target_version: &str,
    branch_name: &str,
) -> Result<ExecutionContract, String> {
    let prompt_template_path = format!("{}/PR_BODY_TEMPLATE.md", entry.manifest_root);
    let prompt_contents =
        render_prompt_template(workspace_root, &prompt_template_path, target_version)?;
    let pr_summary_path = format!("{maintenance_root}/governance/pr-summary.md");
    let closeout_path = format!("{maintenance_root}/governance/maintenance-closeout.json");
    let green_gates = green_gates(entry);

    Ok(ExecutionContract {
        executor: EXECUTE_HOST_SURFACE.to_string(),
        prompt_template_path,
        prompt_sha256: hex::encode(Sha256::digest(prompt_contents.as_bytes())),
        pr_summary_path: pr_summary_path.clone(),
        closeout_path,
        requires_manual_closeout: true,
        writable_surfaces: writable_surfaces(entry, maintenance_root, target_version),
        read_only_inputs: vec![
            format!("{}/OPS_PLAYBOOK.md", entry.manifest_root),
            format!("{}/CI_WORKFLOWS_PLAN.md", entry.manifest_root),
            format!("{}/PR_BODY_TEMPLATE.md", entry.manifest_root),
            opened_from.to_string(),
        ],
        ordered_commands: green_gates.clone(),
        green_gates,
        recovery: ExecutionContractRecovery {
            recreate_packet_command: format!(
                "cargo run -p xtask -- refresh-agent --request {request_path} --write"
            ),
            reopen_pr_body_path: pr_summary_path,
            reopen_pr_branch: branch_name.to_string(),
            notes: vec![
                "If PR creation fails after packet generation, rerun packet regeneration from the frozen request and reopen the PR from the generated pr-summary path.".to_string(),
                format!(
                    "If the local execution-host preflight ({EXECUTION_HOST_LABEL}) fails, fix the Codex binary/auth state and rerun `execute-agent-maintenance --dry-run` before write mode."
                ),
            ],
        },
    })
}

pub(crate) fn normalize_detected_release(
    raw: &DetectedRelease,
    derived: &DerivedDetectedReleaseFields,
) -> DetectedRelease {
    DetectedRelease {
        detected_by: raw.detected_by.clone(),
        current_validated: raw.current_validated.clone(),
        target_version: raw.target_version.clone(),
        latest_stable: raw.latest_stable.clone(),
        version_policy: derived.version_policy.clone(),
        source_kind: derived.source_kind.clone(),
        source_ref: derived.source_ref.clone(),
        dispatch_kind: derived.dispatch_kind.clone(),
        dispatch_workflow: derived.dispatch_workflow.clone(),
        branch_name: raw.branch_name.clone(),
    }
}

pub(crate) fn render_prompt_template(
    workspace_root: &Path,
    prompt_template_path: &str,
    target_version: &str,
) -> Result<String, String> {
    let template_path = workspace_root.join(prompt_template_path);
    let template = fs::read_to_string(&template_path)
        .map_err(|err| format!("read {}: {err}", template_path.display()))?;
    Ok(template.replace("{{VERSION}}", target_version))
}

fn green_gates(entry: &AgentRegistryEntry) -> Vec<String> {
    vec![
        "cargo fmt --all".to_string(),
        format!(
            "cargo run -p xtask -- codex-validate --root {}",
            entry.manifest_root
        ),
        "cargo run -p xtask -- support-matrix --check".to_string(),
        "cargo run -p xtask -- capability-matrix --check".to_string(),
        "cargo run -p xtask -- capability-matrix-audit".to_string(),
        "make preflight".to_string(),
    ]
}

fn writable_surfaces(
    entry: &AgentRegistryEntry,
    maintenance_root: &str,
    target_version: &str,
) -> Vec<String> {
    let mut writable_surfaces = vec![
        format!("{maintenance_root}/**"),
        format!("{}/**", entry.crate_path),
        "crates/agent_api/**".to_string(),
        format!("{}/artifacts.lock.json", entry.manifest_root),
        format!("{}/snapshots/{target_version}/**", entry.manifest_root),
        format!("{}/reports/{target_version}/**", entry.manifest_root),
        format!("{}/versions/{target_version}.json", entry.manifest_root),
        format!("{}/wrapper_coverage.json", entry.manifest_root),
        "cli_manifests/support_matrix/current.json".to_string(),
        "docs/specs/unified-agent-api/support-matrix.md".to_string(),
    ];
    if entry.agent_id == "codex" {
        writable_surfaces.push("docs/specs/codex-wrapper-coverage-scenarios-v1.md".to_string());
    }
    writable_surfaces
}
