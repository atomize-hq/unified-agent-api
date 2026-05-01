use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Write},
    path::{Component, Path, PathBuf},
};

use agent_api::AgentWrapperCapabilities;
use clap::Parser;
use thiserror::Error;
use xtask::workspace_mutation::{
    apply_mutations, plan_create_or_replace, WorkspaceMutationError, WorkspacePathJail,
};
use xtask::{
    agent_lifecycle::{
        self, file_sha256, load_lifecycle_state, load_publication_ready_packet,
        required_evidence_for_stage, write_lifecycle_state, LifecycleStage, LifecycleState,
        PublicationReadyPacket, SideState, SupportTier,
    },
    agent_maintenance::drift::{self, DriftCategory},
    agent_registry::{AgentRegistry, AgentRegistryEntry, REGISTRY_RELATIVE_PATH},
    approval_artifact::{load_approval_artifact, ApprovalArtifact, ApprovalArtifactError},
    proving_run_closeout::{
        self, DurationTruth, ProvingRunCloseout, ProvingRunCloseoutError,
        ProvingRunCloseoutExpected, ResidualFrictionTruth,
    },
};

const OWNERSHIP_MARKER: &str = "<!-- generated-by: xtask onboard-agent; owner: control-plane -->";
const DOCS_NEXT_ROOT: &str = "docs/agents/lifecycle";
const REGISTRY_ENTRY_PATH: &str = "crates/xtask/data/agent_registry.toml";
const RELEASE_DOC_PATH: &str = "docs/crates-io-release.md";
const PUBLISH_WORKFLOW_PATH: &str = ".github/workflows/publish-crates.yml";
const PUBLISH_SCRIPT_PATH: &str = "scripts/publish_crates.py";
const VALIDATE_PUBLISH_SCRIPT_PATH: &str = "scripts/validate_publish_versions.py";
const CHECK_PUBLISH_READINESS_SCRIPT_PATH: &str = "scripts/check_publish_readiness.py";
const NEXT_MAINTENANCE_COMMAND_TEMPLATE: &str = "check-agent-drift --agent {agent_id}";
const AGENT_API_ORTHOGONALITY_ALLOWLIST: [&str; 8] = [
    "agent_api.run",
    "agent_api.events",
    "agent_api.events.live",
    "agent_api.exec.non_interactive",
    "agent_api.tools.mcp.list.v1",
    "agent_api.tools.mcp.get.v1",
    "agent_api.tools.mcp.add.v1",
    "agent_api.tools.mcp.remove.v1",
];

#[derive(Debug, Parser, Clone)]
pub struct Args {
    /// Path to the proving-run approval artifact.
    #[arg(long)]
    pub approval: PathBuf,

    /// Path to the proving-run closeout JSON.
    #[arg(long)]
    pub closeout: PathBuf,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Internal(String),
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Validation(_) => 2,
            Self::Internal(_) => 1,
        }
    }
}

impl From<WorkspaceMutationError> for Error {
    fn from(err: WorkspaceMutationError) -> Self {
        match err {
            WorkspaceMutationError::Validation(message) => Self::Validation(message),
            WorkspaceMutationError::Internal(message) => Self::Internal(message),
        }
    }
}

pub fn run(args: Args) -> Result<(), Error> {
    let workspace_root = resolve_workspace_root()?;
    let mut stdout = io::stdout();
    run_in_workspace(&workspace_root, args, &mut stdout)
}

pub fn run_in_workspace<W: Write>(
    workspace_root: &Path,
    args: Args,
    writer: &mut W,
) -> Result<(), Error> {
    let approval_path = normalize_repo_relative_path(workspace_root, &args.approval, "--approval")?;
    let closeout_path = normalize_repo_relative_path(workspace_root, &args.closeout, "--closeout")?;
    let onboarding_pack_prefix = onboarding_pack_prefix_from_closeout_path(&closeout_path)?;
    let jail = WorkspacePathJail::new(workspace_root)?;
    let resolved_closeout_path = jail.resolve(&closeout_path)?;
    let approval = load_approval_artifact(workspace_root, &approval_path.display().to_string())
        .map_err(map_approval_error)?;
    let closeout = load_validated_closeout(
        workspace_root,
        &closeout_path,
        &resolved_closeout_path,
        &approval_path,
        &onboarding_pack_prefix,
    )?;
    let registry = AgentRegistry::load(workspace_root)
        .map_err(|err| Error::Internal(format!("load {REGISTRY_RELATIVE_PATH}: {err}")))?;
    let entry = registry
        .agents
        .iter()
        .find(|entry| entry.scaffold.onboarding_pack_prefix == onboarding_pack_prefix)
        .ok_or_else(|| {
            Error::Validation(format!(
                "no agent registry entry owns onboarding_pack_prefix `{onboarding_pack_prefix}`"
            ))
        })?;
    validate_closeout_prerequisites(workspace_root, entry, &approval, &closeout_path, &closeout)?;

    let docs_preview = build_docs_preview(entry, &closeout, &closeout_path);
    write_docs(workspace_root, &docs_preview)?;
    update_lifecycle_baseline(workspace_root, entry, &approval, &closeout_path)?;

    writeln!(writer, "OK: close-proving-run write complete.")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "Refreshed {} docs files for `{}` from `{}`.",
        docs_preview.len(),
        entry.agent_id,
        display_repo_relative_path(&closeout_path)
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;

    Ok(())
}

fn resolve_workspace_root() -> Result<PathBuf, Error> {
    let current_dir = std::env::current_dir()
        .map_err(|err| Error::Internal(format!("resolve current directory: {err}")))?;
    for candidate in current_dir.ancestors() {
        let cargo_toml = candidate.join("Cargo.toml");
        let Ok(text) = fs::read_to_string(&cargo_toml) else {
            continue;
        };
        if text.contains("[workspace]") {
            return Ok(candidate.to_path_buf());
        }
    }

    Err(Error::Internal(format!(
        "could not resolve workspace root from {}",
        current_dir.display()
    )))
}

fn build_docs_preview(
    entry: &AgentRegistryEntry,
    closeout: &ProvingRunCloseout,
    closeout_path: &Path,
) -> Vec<(String, Option<String>)> {
    let docs_root = docs_pack_root(&entry.scaffold.onboarding_pack_prefix);
    let docs_root_display = docs_root.display().to_string();
    let closeout_path_display = display_repo_relative_path(closeout_path);
    let release_touchpoints = release_touchpoint_lines(entry)
        .into_iter()
        .map(|line| format!("- {line}"))
        .collect::<Vec<_>>()
        .join("\n");

    vec![
        (
            docs_root.join("README.md").display().to_string(),
            Some(render_markdown_file(render_readme_body(
                entry,
                closeout,
                &closeout_path_display,
            ))),
        ),
        (
            docs_root.join("scope_brief.md").display().to_string(),
            Some(render_markdown_file(render_scope_brief_body(
                entry,
                &docs_root_display,
                closeout,
                &closeout_path_display,
            ))),
        ),
        (
            docs_root.join("seam_map.md").display().to_string(),
            Some(render_markdown_file(render_seam_map_body(
                entry,
                &docs_root_display,
            ))),
        ),
        (
            docs_root.join("threading.md").display().to_string(),
            Some(render_markdown_file(render_threading_body(entry))),
        ),
        (
            docs_root.join("review_surfaces.md").display().to_string(),
            Some(render_markdown_file(render_review_surfaces_body(
                entry,
                &docs_root_display,
            ))),
        ),
        (
            docs_root
                .join("governance/remediation-log.md")
                .display()
                .to_string(),
            Some(render_markdown_file(render_remediation_log_body(closeout))),
        ),
        (
            docs_root.join("HANDOFF.md").display().to_string(),
            Some(render_markdown_file(render_handoff_body(
                entry,
                closeout,
                &closeout_path_display,
                &release_touchpoints,
            ))),
        ),
    ]
}

fn write_docs(
    workspace_root: &Path,
    docs_preview: &[(String, Option<String>)],
) -> Result<(), Error> {
    let jail = WorkspacePathJail::new(workspace_root)?;
    let mutations = docs_preview
        .iter()
        .map(|(relative_path, contents)| {
            plan_create_or_replace(
                &jail,
                PathBuf::from(relative_path),
                contents.clone().unwrap_or_default().into_bytes(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    apply_mutations(workspace_root, &mutations)?;
    Ok(())
}

fn validate_closeout_prerequisites(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    approval: &ApprovalArtifact,
    closeout_path: &Path,
    closeout: &ProvingRunCloseout,
) -> Result<(), Error> {
    let lifecycle_state_path =
        agent_lifecycle::lifecycle_state_path(&entry.scaffold.onboarding_pack_prefix);
    let lifecycle_state = load_lifecycle_state(workspace_root, &lifecycle_state_path)
        .map_err(|err| Error::Validation(err.to_string()))?;
    if lifecycle_state.approval_artifact_path != approval.relative_path {
        return Err(Error::Validation(format!(
            "`{lifecycle_state_path}` approval_artifact_path `{}` does not match `{}`",
            lifecycle_state.approval_artifact_path, approval.relative_path
        )));
    }
    if lifecycle_state.approval_artifact_sha256 != approval.sha256 {
        return Err(Error::Validation(format!(
            "`{lifecycle_state_path}` approval_artifact_sha256 does not match `{}`",
            approval.relative_path
        )));
    }
    match lifecycle_state.lifecycle_stage {
        LifecycleStage::PublicationReady | LifecycleStage::Published => {}
        other => {
            return Err(Error::Validation(format!(
                "close-proving-run requires lifecycle stage `publication_ready` or legacy `published` at `{}` (found `{}`)",
                lifecycle_state_path,
                other.as_str()
            )))
        }
    }

    let packet_path =
        agent_lifecycle::publication_ready_path(&entry.scaffold.onboarding_pack_prefix);
    validate_publication_packet_continuity(
        workspace_root,
        &packet_path,
        &lifecycle_state,
        approval,
        &entry.manifest_root,
    )?;
    if !closeout.preflight_passed {
        return Err(Error::Validation(format!(
            "{} records `preflight_passed = false`; close-proving-run requires a green preflight gate",
            closeout_path.display()
        )));
    }

    let drift_report =
        drift::check_agent_drift(workspace_root, &entry.agent_id).map_err(|err| {
            Error::Validation(format!(
                "drift re-check failed for `{}`: {err}",
                entry.agent_id
            ))
        })?;
    let blocking_findings = drift_report
        .findings
        .iter()
        .filter(|finding| {
            matches!(
                finding.category,
                DriftCategory::RegistryManifest
                    | DriftCategory::CapabilityPublication
                    | DriftCategory::SupportPublication
            )
        })
        .collect::<Vec<_>>();
    if !blocking_findings.is_empty() {
        return Err(Error::Validation(format!(
            "green publication surfaces are required before closeout: {}",
            blocking_findings
                .iter()
                .map(|finding| finding.summary.clone())
                .collect::<Vec<_>>()
                .join(" | ")
        )));
    }
    validate_capability_matrix_audit_green()?;
    Ok(())
}

fn validate_publication_packet_continuity(
    workspace_root: &Path,
    packet_path: &str,
    lifecycle_state: &LifecycleState,
    approval: &ApprovalArtifact,
    manifest_root: &str,
) -> Result<(), Error> {
    match lifecycle_state.lifecycle_stage {
        LifecycleStage::PublicationReady => {
            let packet = load_publication_ready_packet(workspace_root, packet_path)
                .map_err(|err| Error::Validation(err.to_string()))?;
            validate_packet_identity(packet_path, &packet, approval, manifest_root)?;
        }
        LifecycleStage::Published => {
            let packet_bytes = fs::read(workspace_root.join(packet_path))
                .map_err(|err| Error::Validation(format!("read {packet_path}: {err}")))?;
            let packet: PublicationReadyPacket = serde_json::from_slice(&packet_bytes)
                .map_err(|err| Error::Validation(format!("parse {packet_path}: {err}")))?;
            packet
                .validate()
                .map_err(|err| Error::Validation(err.to_string()))?;
            validate_packet_identity(packet_path, &packet, approval, manifest_root)?;
        }
        _ => unreachable!("caller validates allowed lifecycle stages"),
    }
    Ok(())
}

fn validate_packet_identity(
    packet_path: &str,
    packet: &PublicationReadyPacket,
    approval: &ApprovalArtifact,
    manifest_root: &str,
) -> Result<(), Error> {
    if packet.approval_artifact_path != approval.relative_path {
        return Err(Error::Validation(format!(
            "`{packet_path}` approval_artifact_path `{}` does not match `{}`",
            packet.approval_artifact_path, approval.relative_path
        )));
    }
    if packet.approval_artifact_sha256 != approval.sha256 {
        return Err(Error::Validation(format!(
            "`{packet_path}` approval_artifact_sha256 does not match `{}`",
            approval.relative_path
        )));
    }
    if packet.agent_id != approval.descriptor.agent_id {
        return Err(Error::Validation(format!(
            "`{packet_path}` agent_id `{}` does not match `{}`",
            packet.agent_id, approval.descriptor.agent_id
        )));
    }
    if packet.manifest_root != manifest_root {
        return Err(Error::Validation(format!(
            "`{packet_path}` manifest_root `{}` does not match `{}`",
            packet.manifest_root, manifest_root
        )));
    }
    Ok(())
}

fn update_lifecycle_baseline(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    approval: &ApprovalArtifact,
    closeout_path: &Path,
) -> Result<(), Error> {
    let lifecycle_state_path =
        agent_lifecycle::lifecycle_state_path(&entry.scaffold.onboarding_pack_prefix);
    let packet_path =
        agent_lifecycle::publication_ready_path(&entry.scaffold.onboarding_pack_prefix);
    let mut lifecycle_state = load_lifecycle_state(workspace_root, &lifecycle_state_path)
        .map_err(|err| Error::Validation(err.to_string()))?;
    if lifecycle_state.approval_artifact_path != approval.relative_path
        || lifecycle_state.approval_artifact_sha256 != approval.sha256
    {
        return Err(Error::Validation(format!(
            "`{}` no longer matches approval continuity for `{}`",
            lifecycle_state_path, approval.relative_path
        )));
    }

    lifecycle_state.lifecycle_stage = LifecycleStage::ClosedBaseline;
    lifecycle_state.support_tier =
        if matches!(lifecycle_state.support_tier, SupportTier::FirstClass) {
            SupportTier::FirstClass
        } else {
            SupportTier::PublicationBacked
        };
    lifecycle_state.current_owner_command = "close-proving-run --write".to_string();
    lifecycle_state.expected_next_command =
        NEXT_MAINTENANCE_COMMAND_TEMPLATE.replace("{agent_id}", &entry.agent_id);
    lifecycle_state.last_transition_at =
        agent_lifecycle::now_rfc3339().map_err(|err| Error::Internal(err.to_string()))?;
    lifecycle_state.last_transition_by = "xtask close-proving-run --write".to_string();
    lifecycle_state.required_evidence =
        required_evidence_for_stage(LifecycleStage::ClosedBaseline).to_vec();
    lifecycle_state.satisfied_evidence =
        required_evidence_for_stage(LifecycleStage::ClosedBaseline).to_vec();
    lifecycle_state.side_states.retain(|state| {
        !matches!(
            state,
            SideState::Blocked | SideState::FailedRetryable | SideState::Drifted
        )
    });
    lifecycle_state.blocking_issues.clear();
    lifecycle_state.retryable_failures.clear();
    lifecycle_state.publication_packet_path = Some(packet_path.clone());
    lifecycle_state.publication_packet_sha256 = Some(
        file_sha256(workspace_root, &packet_path)
            .map_err(|err| Error::Validation(err.to_string()))?,
    );
    lifecycle_state.closeout_baseline_path = Some(closeout_path.display().to_string());

    write_lifecycle_state(workspace_root, &lifecycle_state_path, &lifecycle_state)
        .map_err(|err| Error::Internal(format!("write lifecycle state: {err}")))
}

fn validate_capability_matrix_audit_green() -> Result<(), Error> {
    let backends = xtask::capability_matrix::collect_builtin_backend_capabilities()
        .map_err(Error::Validation)?;
    let all_capability_ids = backends
        .values()
        .flat_map(|caps| caps.ids.iter().cloned())
        .collect::<BTreeSet<_>>();

    let mut violations = Vec::new();
    for capability_id in &all_capability_ids {
        if !capability_id.starts_with("agent_api.") {
            continue;
        }
        if AGENT_API_ORTHOGONALITY_ALLOWLIST.contains(&capability_id.as_str()) {
            continue;
        }
        let supported_by = supported_backends(&backends, capability_id);
        if supported_by.len() < 2 {
            violations.push(format!(
                "{capability_id}: supported by {} backend(s): [{}]",
                supported_by.len(),
                supported_by.join(", ")
            ));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(Error::Validation(format!(
            "capability-matrix-audit failed: {}",
            violations.join(" | ")
        )))
    }
}

fn supported_backends(
    backends: &BTreeMap<String, AgentWrapperCapabilities>,
    capability_id: &str,
) -> Vec<String> {
    backends
        .iter()
        .filter_map(|(backend_id, capabilities)| {
            if capabilities.contains(capability_id) {
                Some(backend_id.clone())
            } else {
                None
            }
        })
        .collect()
}

fn load_validated_closeout(
    workspace_root: &Path,
    closeout_path: &Path,
    resolved_closeout_path: &Path,
    approval_path: &Path,
    onboarding_pack_prefix: &str,
) -> Result<ProvingRunCloseout, Error> {
    let expected = ProvingRunCloseoutExpected {
        approval_path: Some(approval_path),
        onboarding_pack_prefix,
    };
    proving_run_closeout::load_validated_closeout(
        workspace_root,
        closeout_path,
        resolved_closeout_path,
        expected,
    )
    .map_err(map_closeout_error)
}

fn map_closeout_error(err: ProvingRunCloseoutError) -> Error {
    match err {
        ProvingRunCloseoutError::Validation(message) => Error::Validation(message),
        ProvingRunCloseoutError::Internal(message) => Error::Internal(message),
    }
}

fn map_approval_error(err: ApprovalArtifactError) -> Error {
    match err {
        ApprovalArtifactError::Validation(message) => Error::Validation(message),
        ApprovalArtifactError::Internal(message) => Error::Internal(message),
    }
}

fn onboarding_pack_prefix_from_closeout_path(closeout_path: &Path) -> Result<String, Error> {
    let components = closeout_path.components().collect::<Vec<_>>();
    let expected_prefix = [
        Component::Normal("docs".as_ref()),
        Component::Normal("agents".as_ref()),
        Component::Normal("lifecycle".as_ref()),
    ];
    if components.len() != 6
        || components[0..3] != expected_prefix
        || components[4] != Component::Normal("governance".as_ref())
        || components[5] != Component::Normal("proving-run-closeout.json".as_ref())
    {
        return Err(Error::Validation(format!(
            "{} must point to docs/agents/lifecycle/<prefix>/governance/proving-run-closeout.json",
            closeout_path.display()
        )));
    }
    path_component_to_string(components[3])
}

fn path_component_to_string(component: Component<'_>) -> Result<String, Error> {
    let Component::Normal(value) = component else {
        return Err(Error::Validation(
            "onboarding pack prefix must be a normal path component".to_string(),
        ));
    };
    Ok(value.to_string_lossy().into_owned())
}

fn docs_pack_root(prefix: &str) -> PathBuf {
    Path::new(DOCS_NEXT_ROOT).join(prefix)
}

fn normalize_repo_relative_path(
    workspace_root: &Path,
    path: &Path,
    flag_name: &str,
) -> Result<PathBuf, Error> {
    let relative = if path.is_absolute() {
        path.strip_prefix(workspace_root)
            .map(Path::to_path_buf)
            .map_err(|_| {
                Error::Validation(format!(
                    "{flag_name} `{}` must be inside workspace root {}",
                    path.display(),
                    workspace_root.display()
                ))
            })?
    } else {
        path.to_path_buf()
    };

    if relative.components().next().is_none()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(Error::Validation(format!(
            "{flag_name} `{}` must be a repo-relative path with only normal components",
            path.display()
        )));
    }

    Ok(relative)
}

fn display_repo_relative_path(path: &Path) -> String {
    path.display().to_string()
}

fn release_touchpoint_lines(entry: &AgentRegistryEntry) -> Vec<String> {
    vec![
        format!(
            "Path: Cargo.toml will ensure workspace member `{}` is enrolled.",
            entry.crate_path
        ),
        format!(
            "Path: {RELEASE_DOC_PATH} will ensure the generated release block includes `{}` on release track `{}`.",
            entry.package_name, entry.release.docs_release_track
        ),
        format!(
            "Workflow and script files remain unchanged: {PUBLISH_WORKFLOW_PATH}, {PUBLISH_SCRIPT_PATH}, {VALIDATE_PUBLISH_SCRIPT_PATH}, {CHECK_PUBLISH_READINESS_SCRIPT_PATH}."
        ),
    ]
}

fn render_markdown_file(body: String) -> String {
    format!("{OWNERSHIP_MARKER}\n\n{body}")
}

fn render_readme_body(
    entry: &AgentRegistryEntry,
    closeout: &ProvingRunCloseout,
    closeout_path: &str,
) -> String {
    format!(
        "# {} onboarding pack\n\nThis packet records the closed proving run for `{}`.\n\n- Packet state: `closed_proving_run`\n- Agent id: `{}`\n- Wrapper crate: `{}`\n- Backend module: `{}`\n- Manifest root: `{}`\n- Closeout metadata is recorded in `{}`.\n- Approval linkage: `{}` via `{}` (`sha256: {}`)\n",
        entry.display_name,
        entry.display_name,
        entry.agent_id,
        entry.crate_path,
        entry.backend_module,
        entry.manifest_root,
        closeout_path,
        closeout.approval_source,
        closeout.approval_ref,
        closeout.approval_sha256
    )
}

fn render_scope_brief_body(
    entry: &AgentRegistryEntry,
    docs_root_display: &str,
    closeout: &ProvingRunCloseout,
    closeout_path: &str,
) -> String {
    format!(
        "# Scope brief\n\nThis packet records the closed proving run for `{}`.\n\n- Registry enrollment in `{REGISTRY_ENTRY_PATH}`\n- Docs pack in `{docs_root_display}`\n- Manifest root in `{}`\n- Closeout metadata in `{}`\n- Approval linkage via `{}` (`{}`, sha256 `{}`)\n\nCloseout status: `make preflight` {} for this proving run.\n",
        entry.agent_id,
        entry.manifest_root,
        closeout_path,
        closeout.approval_ref,
        closeout.approval_source,
        closeout.approval_sha256,
        if closeout.preflight_passed {
            "passed"
        } else {
            "did not pass"
        }
    )
}

fn render_seam_map_body(entry: &AgentRegistryEntry, docs_root_display: &str) -> String {
    format!(
        "# Seam map\n\n- Declaration seam: registry entry for `{}`\n- Docs seam: onboarding pack `{docs_root_display}`\n- Manifest seam: `{}`\n- Runtime seam: wrapper crate `{}` and backend module `{}`\n",
        entry.agent_id,
        entry.manifest_root,
        entry.crate_path,
        entry.backend_module
    )
}

fn render_threading_body(entry: &AgentRegistryEntry) -> String {
    format!(
        "# Threading\n\n1. Control-plane onboarding writes for `{}` landed without follow-up packet drift.\n2. Runtime-owned wrapper and backend work landed at `{}` and `{}`.\n3. Manifest evidence and publication artifacts were regenerated from committed runtime outputs.\n4. The proving run closed with `make preflight`.\n",
        entry.agent_id,
        entry.crate_path,
        entry.backend_module
    )
}

fn render_review_surfaces_body(entry: &AgentRegistryEntry, docs_root_display: &str) -> String {
    format!(
        "# Review surfaces\n\n- `{REGISTRY_ENTRY_PATH}`\n- `{docs_root_display}`\n- `{}`\n- `{RELEASE_DOC_PATH}`\n- Supporting release rails remained unchanged across the proving run: `{PUBLISH_WORKFLOW_PATH}`, `{PUBLISH_SCRIPT_PATH}`, `{VALIDATE_PUBLISH_SCRIPT_PATH}`, `{CHECK_PUBLISH_READINESS_SCRIPT_PATH}`\n",
        entry.manifest_root
    )
}

fn render_remediation_log_body(closeout: &ProvingRunCloseout) -> String {
    format!(
        "# Remediation log\n\n{}\n",
        render_residual_friction_lines(closeout)
    )
}

fn render_handoff_body(
    entry: &AgentRegistryEntry,
    closeout: &ProvingRunCloseout,
    closeout_path: &str,
    release_touchpoints: &str,
) -> String {
    format!(
        "# Handoff\n\nThis packet records the closed proving run for `{}`.\n\n## Release touchpoints\n\n{}\n\n## Proving-run closeout\n\n- approval ref: `{}`\n- approval source: `{}`\n- approval artifact sha256: `{}`\n- manual control-plane file edits by maintainers: `{}`\n- partial-write incidents: `{}`\n- ambiguous ownership incidents: `{}`\n- approved-agent to repo-ready control-plane mutation time: `{}`\n- proving-run closeout passes `make preflight`: `{}`\n- recorded at: `{}`\n- commit: `{}`\n- closeout metadata: `{}`\n\n## Residual friction\n\n{}\n\n## Status\n\nNo open runtime next step remains in this packet.\n",
        entry.agent_id,
        release_touchpoints,
        closeout.approval_ref,
        closeout.approval_source,
        closeout.approval_sha256,
        closeout.manual_control_plane_edits,
        closeout.partial_write_incidents,
        closeout.ambiguous_ownership_incidents,
        render_closeout_duration(closeout),
        closeout.preflight_passed,
        closeout.recorded_at,
        closeout.commit,
        closeout_path,
        render_residual_friction_lines(closeout)
    )
}

fn render_closeout_duration(closeout: &ProvingRunCloseout) -> String {
    match &closeout.duration {
        DurationTruth::Seconds(seconds) => format!("{seconds}s"),
        DurationTruth::MissingReason(reason) => format!("missing ({reason})"),
    }
}

fn render_residual_friction_lines(closeout: &ProvingRunCloseout) -> String {
    let mut lines = match &closeout.residual_friction {
        ResidualFrictionTruth::Items(items) => items
            .iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>(),
        ResidualFrictionTruth::ExplicitNone(reason) => {
            vec![format!("- No residual friction recorded: {reason}")]
        }
    };
    if let DurationTruth::MissingReason(reason) = &closeout.duration {
        lines.push(format!("- Duration missing reason: {reason}"));
    }
    lines.join("\n")
}
