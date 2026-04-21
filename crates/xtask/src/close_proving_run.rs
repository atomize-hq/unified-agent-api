use std::{
    fs,
    io::{self, Write},
    path::{Component, Path, PathBuf},
};

use clap::Parser;
use serde::Deserialize;
use thiserror::Error;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use xtask::agent_registry::{AgentRegistry, AgentRegistryEntry, REGISTRY_RELATIVE_PATH};
use xtask::approval_artifact::{self, ApprovalArtifact, ApprovalArtifactError};

const OWNERSHIP_MARKER: &str = "<!-- generated-by: xtask onboard-agent; owner: control-plane -->";
const DOCS_NEXT_ROOT: &str = "docs/project_management/next";
const REGISTRY_ENTRY_PATH: &str = "crates/xtask/data/agent_registry.toml";
const RELEASE_DOC_PATH: &str = "docs/crates-io-release.md";
const PUBLISH_WORKFLOW_PATH: &str = ".github/workflows/publish-crates.yml";
const PUBLISH_SCRIPT_PATH: &str = "scripts/publish_crates.py";
const VALIDATE_PUBLISH_SCRIPT_PATH: &str = "scripts/validate_publish_versions.py";
const CHECK_PUBLISH_READINESS_SCRIPT_PATH: &str = "scripts/check_publish_readiness.py";

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

#[derive(Debug, Clone)]
enum DurationTruth {
    Seconds(u64),
    MissingReason(String),
}

#[derive(Debug, Clone)]
enum ResidualFrictionTruth {
    Items(Vec<String>),
    ExplicitNone(String),
}

#[derive(Debug, Clone)]
struct ProvingRunCloseout {
    approval_ref: String,
    approval_sha256: String,
    approval_source: String,
    manual_control_plane_edits: u64,
    partial_write_incidents: u64,
    ambiguous_ownership_incidents: u64,
    duration: DurationTruth,
    residual_friction: ResidualFrictionTruth,
    preflight_passed: bool,
    recorded_at: String,
    commit: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProvingRunCloseout {
    state: String,
    approval_ref: Option<String>,
    approval_sha256: Option<String>,
    approval_source: Option<String>,
    manual_control_plane_edits: u64,
    partial_write_incidents: u64,
    ambiguous_ownership_incidents: u64,
    duration_seconds: Option<u64>,
    duration_missing_reason: Option<String>,
    residual_friction: Option<Vec<String>>,
    explicit_none_reason: Option<String>,
    preflight_passed: bool,
    recorded_at: String,
    commit: String,
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
    let closeout_path = absolutize(workspace_root, &args.closeout);
    let onboarding_pack_prefix =
        onboarding_pack_prefix_from_closeout_path(workspace_root, &closeout_path)?;
    let closeout = load_validated_closeout(
        workspace_root,
        &closeout_path,
        &args.approval,
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

    let docs_preview = build_docs_preview(workspace_root, entry, &closeout, &closeout_path);
    write_docs(workspace_root, &docs_preview)?;

    writeln!(writer, "OK: close-proving-run write complete.")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "Refreshed {} docs files for `{}` from `{}`.",
        docs_preview.len(),
        entry.agent_id,
        display_workspace_path(workspace_root, &closeout_path)
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
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    closeout: &ProvingRunCloseout,
    closeout_path: &Path,
) -> Vec<(String, Option<String>)> {
    let docs_root = docs_pack_root(&entry.scaffold.onboarding_pack_prefix);
    let docs_root_display = docs_root.display().to_string();
    let closeout_path_display = display_workspace_path(workspace_root, closeout_path);
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
    for (relative_path, contents) in docs_preview {
        let path = workspace_root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| Error::Internal(format!("create {}: {err}", parent.display())))?;
        }
        fs::write(&path, contents.clone().unwrap_or_default())
            .map_err(|err| Error::Internal(format!("write {}: {err}", path.display())))?;
    }
    Ok(())
}

fn load_validated_closeout(
    workspace_root: &Path,
    closeout_path: &Path,
    approval_path: &Path,
    onboarding_pack_prefix: &str,
) -> Result<ProvingRunCloseout, Error> {
    let closeout_text = fs::read_to_string(closeout_path)
        .map_err(|err| Error::Validation(format!("read {}: {err}", closeout_path.display())))?;
    let raw = serde_json::from_str::<RawProvingRunCloseout>(&closeout_text)
        .map_err(|err| Error::Validation(format!("parse {}: {err}", closeout_path.display())))?;
    validate_closeout(
        workspace_root,
        closeout_path,
        raw,
        approval_path,
        onboarding_pack_prefix,
    )
}

fn validate_closeout(
    workspace_root: &Path,
    closeout_path: &Path,
    raw: RawProvingRunCloseout,
    approval_path: &Path,
    onboarding_pack_prefix: &str,
) -> Result<ProvingRunCloseout, Error> {
    if raw.state != "closed" {
        return Err(Error::Validation(format!(
            "{}: state must equal `closed`",
            closeout_path.display()
        )));
    }

    let approval_ref = required_string(closeout_path, "approval_ref", raw.approval_ref.as_deref())?;
    let approval_sha256 = required_string(
        closeout_path,
        "approval_sha256",
        raw.approval_sha256.as_deref(),
    )?;
    let approval_source = required_string(
        closeout_path,
        "approval_source",
        raw.approval_source.as_deref(),
    )?;
    validate_lower_hex_sha256(closeout_path, &approval_sha256)?;
    validate_recorded_at(closeout_path, &raw.recorded_at)?;
    validate_commit(closeout_path, &raw.commit)?;

    let duration = match (raw.duration_seconds, raw.duration_missing_reason) {
        (Some(seconds), None) => DurationTruth::Seconds(seconds),
        (None, Some(reason)) => DurationTruth::MissingReason(non_empty_field(
            closeout_path,
            "duration_missing_reason",
            &reason,
        )?),
        _ => {
            return Err(Error::Validation(format!(
                "{}: exactly one of `duration_seconds` or `duration_missing_reason` is required",
                closeout_path.display()
            )));
        }
    };

    let residual_friction = match (raw.residual_friction, raw.explicit_none_reason) {
        (Some(items), None) => {
            let items = items
                .into_iter()
                .map(|item| non_empty_field(closeout_path, "residual_friction[]", &item))
                .collect::<Result<Vec<_>, _>>()?;
            if items.is_empty() {
                return Err(Error::Validation(format!(
                    "{}: `residual_friction` must not be empty when present",
                    closeout_path.display()
                )));
            }
            ResidualFrictionTruth::Items(items)
        }
        (None, Some(reason)) => ResidualFrictionTruth::ExplicitNone(non_empty_field(
            closeout_path,
            "explicit_none_reason",
            &reason,
        )?),
        _ => {
            return Err(Error::Validation(format!(
                "{}: exactly one of `residual_friction` or `explicit_none_reason` is required",
                closeout_path.display()
            )));
        }
    };

    let provided_approval = load_approval_artifact(workspace_root, approval_path, closeout_path)?;
    let linked_approval =
        load_approval_artifact(workspace_root, Path::new(&approval_ref), closeout_path)?;
    validate_same_approval_artifact(closeout_path, &provided_approval, &linked_approval)?;
    validate_approval_hash(closeout_path, &provided_approval, &approval_sha256)?;
    validate_approval_pack_prefix(closeout_path, &provided_approval, onboarding_pack_prefix)?;

    Ok(ProvingRunCloseout {
        approval_ref,
        approval_sha256,
        approval_source,
        manual_control_plane_edits: raw.manual_control_plane_edits,
        partial_write_incidents: raw.partial_write_incidents,
        ambiguous_ownership_incidents: raw.ambiguous_ownership_incidents,
        duration,
        residual_friction,
        preflight_passed: raw.preflight_passed,
        recorded_at: raw.recorded_at,
        commit: raw.commit,
    })
}

fn required_string(path: &Path, field_name: &str, value: Option<&str>) -> Result<String, Error> {
    let Some(value) = value else {
        return Err(Error::Validation(format!(
            "{}: missing required field `{field_name}`",
            path.display()
        )));
    };
    non_empty_field(path, field_name, value)
}

fn non_empty_field(path: &Path, field_name: &str, value: &str) -> Result<String, Error> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(Error::Validation(format!(
            "{}: `{field_name}` must not be empty",
            path.display()
        )));
    }
    Ok(trimmed.to_string())
}

fn validate_lower_hex_sha256(path: &Path, value: &str) -> Result<(), Error> {
    if value.len() != 64 || !value.chars().all(|ch| matches!(ch, '0'..='9' | 'a'..='f')) {
        return Err(Error::Validation(format!(
            "{}: `approval_sha256` must be 64 lowercase hex characters",
            path.display()
        )));
    }
    Ok(())
}

fn validate_recorded_at(path: &Path, value: &str) -> Result<(), Error> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|err| {
        Error::Validation(format!(
            "{}: `recorded_at` must be RFC3339 ({err})",
            path.display()
        ))
    })?;
    Ok(())
}

fn validate_commit(path: &Path, value: &str) -> Result<(), Error> {
    let valid = (7..=40).contains(&value.len())
        && value.chars().all(|ch| matches!(ch, '0'..='9' | 'a'..='f'));
    if !valid {
        return Err(Error::Validation(format!(
            "{}: `commit` must be 7-40 lowercase hex characters",
            path.display()
        )));
    }
    Ok(())
}

fn load_approval_artifact(
    workspace_root: &Path,
    approval_path: &Path,
    closeout_path: &Path,
) -> Result<ApprovalArtifact, Error> {
    let approval_path = approval_path.to_string_lossy();
    approval_artifact::load_approval_artifact(workspace_root, &approval_path)
        .map_err(|err| map_approval_artifact_error(closeout_path, err))
}

fn validate_same_approval_artifact(
    closeout_path: &Path,
    provided_approval: &ApprovalArtifact,
    linked_approval: &ApprovalArtifact,
) -> Result<(), Error> {
    if provided_approval.canonical_path != linked_approval.canonical_path {
        return Err(Error::Validation(format!(
            "{}: approval_ref `{}` does not match --approval `{}`",
            closeout_path.display(),
            linked_approval.relative_path,
            provided_approval.relative_path
        )));
    }
    Ok(())
}

fn validate_approval_hash(
    closeout_path: &Path,
    approval: &ApprovalArtifact,
    expected_sha256: &str,
) -> Result<(), Error> {
    if approval.sha256 != expected_sha256 {
        return Err(Error::Validation(format!(
            "{}: approval_sha256 does not match {}",
            closeout_path.display(),
            approval.relative_path
        )));
    }
    Ok(())
}

fn validate_approval_pack_prefix(
    closeout_path: &Path,
    approval: &ApprovalArtifact,
    onboarding_pack_prefix: &str,
) -> Result<(), Error> {
    if approval.descriptor.onboarding_pack_prefix != onboarding_pack_prefix {
        return Err(Error::Validation(format!(
            "{}: approval artifact `{}` belongs to onboarding_pack_prefix `{}` instead of `{}`",
            closeout_path.display(),
            approval.relative_path,
            approval.descriptor.onboarding_pack_prefix,
            onboarding_pack_prefix
        )));
    }
    Ok(())
}

fn map_approval_artifact_error(closeout_path: &Path, err: ApprovalArtifactError) -> Error {
    match err {
        ApprovalArtifactError::Validation(message) => {
            Error::Validation(format!("{}: {message}", closeout_path.display()))
        }
        ApprovalArtifactError::Internal(message) => Error::Internal(message),
    }
}

fn onboarding_pack_prefix_from_closeout_path(
    workspace_root: &Path,
    closeout_path: &Path,
) -> Result<String, Error> {
    let relative = closeout_path.strip_prefix(workspace_root).map_err(|_| {
        Error::Validation(format!(
            "{} is not contained by workspace root {}",
            closeout_path.display(),
            workspace_root.display()
        ))
    })?;
    let components = relative.components().collect::<Vec<_>>();
    let expected_prefix = [
        Component::Normal("docs".as_ref()),
        Component::Normal("project_management".as_ref()),
        Component::Normal("next".as_ref()),
    ];
    if components.len() != 6
        || components[0..3] != expected_prefix
        || components[4] != Component::Normal("governance".as_ref())
        || components[5] != Component::Normal("proving-run-closeout.json".as_ref())
    {
        return Err(Error::Validation(format!(
            "{} must point to docs/project_management/next/<prefix>/governance/proving-run-closeout.json",
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

fn absolutize(workspace_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    }
}

fn display_workspace_path(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .display()
        .to_string()
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
