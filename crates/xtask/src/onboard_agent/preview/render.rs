use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::agent_registry::REGISTRY_RELATIVE_PATH;
use crate::approval_artifact;
use serde::Deserialize;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use super::super::{
    CHECK_PUBLISH_READINESS_SCRIPT_PATH, OWNERSHIP_MARKER, PUBLISH_SCRIPT_PATH,
    PUBLISH_WORKFLOW_PATH, RELEASE_DOC_PATH, VALIDATE_PUBLISH_SCRIPT_PATH,
};
use super::ApprovalRenderInput;
use super::DraftEntry;
const DOCS_NEXT_ROOT: &str = "docs/project_management/next";
const PROVING_RUN_CLOSEOUT_RELATIVE_PATH: &str = "governance/proving-run-closeout.json";

#[derive(Debug, Clone)]
pub(in crate::onboard_agent) struct ProvingRunCloseout {
    pub(in crate::onboard_agent) approval_ref: String,
    pub(in crate::onboard_agent) approval_sha256: String,
    pub(in crate::onboard_agent) approval_source: String,
    pub(in crate::onboard_agent) manual_control_plane_edits: u64,
    pub(in crate::onboard_agent) partial_write_incidents: u64,
    pub(in crate::onboard_agent) ambiguous_ownership_incidents: u64,
    pub(in crate::onboard_agent) duration: DurationTruth,
    pub(in crate::onboard_agent) residual_friction: ResidualFrictionTruth,
    pub(in crate::onboard_agent) preflight_passed: bool,
    pub(in crate::onboard_agent) recorded_at: String,
    pub(in crate::onboard_agent) commit: String,
}

#[derive(Debug, Clone)]
pub(in crate::onboard_agent) enum DurationTruth {
    Seconds(u64),
    MissingReason(String),
}

#[derive(Debug, Clone)]
pub(in crate::onboard_agent) enum ResidualFrictionTruth {
    Items(Vec<String>),
    ExplicitNone(String),
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

#[derive(Debug, Clone, Copy)]
pub(in crate::onboard_agent) enum PacketPhase<'a> {
    Execution,
    Closeout(&'a ProvingRunCloseout),
}

pub(in crate::onboard_agent) fn load_validated_closeout_if_present(
    workspace_root: &Path,
    draft: &DraftEntry,
) -> Result<Option<ProvingRunCloseout>, String> {
    let closeout_path = workspace_root.join(closeout_relative_path(draft));
    let closeout_text = match fs::read_to_string(&closeout_path) {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(format!("read {}: {err}", closeout_path.display()));
        }
    };
    let raw = match serde_json::from_str::<RawProvingRunCloseout>(&closeout_text) {
        Ok(raw) => raw,
        Err(_) => return Ok(None),
    };
    Ok(validate_closeout(workspace_root, draft, raw).ok())
}

pub(in crate::onboard_agent) fn closeout_relative_path(draft: &DraftEntry) -> String {
    docs_pack_root(&draft.onboarding_pack_prefix)
        .join(PROVING_RUN_CLOSEOUT_RELATIVE_PATH)
        .display()
        .to_string()
}

pub(in crate::onboard_agent) fn build_docs_preview(
    draft: &DraftEntry,
    release_touchpoints: &[String],
    phase: PacketPhase<'_>,
    approval: Option<ApprovalRenderInput<'_>>,
) -> Vec<(String, Option<String>)> {
    let docs_root = draft.docs_pack_root();
    let docs_root_display = docs_root.display().to_string();
    let closeout_path = closeout_relative_path(draft);
    let release_touchpoints = release_touchpoints
        .iter()
        .map(|line| format!("- {line}"))
        .collect::<Vec<_>>()
        .join("\n");

    vec![
        (
            docs_root.join("README.md").display().to_string(),
            Some(render_markdown_file(render_readme_body(
                draft,
                phase,
                &closeout_path,
                approval,
            ))),
        ),
        (
            docs_root.join("scope_brief.md").display().to_string(),
            Some(render_markdown_file(render_scope_brief_body(
                draft,
                phase,
                &docs_root_display,
                &closeout_path,
                approval,
            ))),
        ),
        (
            docs_root.join("seam_map.md").display().to_string(),
            Some(render_markdown_file(render_seam_map_body(
                draft,
                &docs_root_display,
            ))),
        ),
        (
            docs_root.join("threading.md").display().to_string(),
            Some(render_markdown_file(render_threading_body(draft, phase))),
        ),
        (
            docs_root.join("review_surfaces.md").display().to_string(),
            Some(render_markdown_file(render_review_surfaces_body(
                draft,
                phase,
                &docs_root_display,
            ))),
        ),
        (
            docs_root
                .join("governance/remediation-log.md")
                .display()
                .to_string(),
            Some(render_markdown_file(render_remediation_log_body(phase))),
        ),
        (
            docs_root.join("HANDOFF.md").display().to_string(),
            Some(render_markdown_file(render_handoff_body(
                draft,
                phase,
                &closeout_path,
                &release_touchpoints,
                approval,
            ))),
        ),
    ]
}

pub(in crate::onboard_agent) fn release_touchpoint_lines(draft: &DraftEntry) -> Vec<String> {
    vec![
        format!(
            "Path: Cargo.toml will ensure workspace member `{}` is enrolled.",
            draft.crate_path
        ),
        format!(
            "Path: {RELEASE_DOC_PATH} will ensure the generated release block includes `{}` on release track `{}`.",
            draft.package_name, draft.docs_release_track
        ),
        format!(
            "Workflow and script files remain unchanged: {PUBLISH_WORKFLOW_PATH}, {PUBLISH_SCRIPT_PATH}, {VALIDATE_PUBLISH_SCRIPT_PATH}, {CHECK_PUBLISH_READINESS_SCRIPT_PATH}."
        ),
    ]
}

fn validate_closeout(
    workspace_root: &Path,
    draft: &DraftEntry,
    raw: RawProvingRunCloseout,
) -> Result<ProvingRunCloseout, ()> {
    if raw.state != "closed" {
        return Err(());
    }

    let approval_ref = required_string(raw.approval_ref.as_deref())?;
    let approval_sha256 = required_string(raw.approval_sha256.as_deref())?;
    if approval_sha256.len() != 64
        || !approval_sha256
            .chars()
            .all(|ch| matches!(ch, '0'..='9' | 'a'..='f'))
    {
        return Err(());
    }
    let approval_source = required_string(raw.approval_source.as_deref())?;
    let approval =
        approval_artifact::load_approval_artifact(workspace_root, &approval_ref).map_err(|_| ())?;
    if approval.sha256 != approval_sha256 {
        return Err(());
    }
    if approval.descriptor.onboarding_pack_prefix != draft.onboarding_pack_prefix {
        return Err(());
    }
    OffsetDateTime::parse(&raw.recorded_at, &Rfc3339).map_err(|_| ())?;
    validate_commit(&raw.commit)?;

    let duration = match (raw.duration_seconds, raw.duration_missing_reason) {
        (Some(seconds), None) => DurationTruth::Seconds(seconds),
        (None, Some(reason)) => DurationTruth::MissingReason(required_string(Some(&reason))?),
        _ => return Err(()),
    };

    let residual_friction = match (raw.residual_friction, raw.explicit_none_reason) {
        (Some(items), None) if !items.is_empty() => ResidualFrictionTruth::Items(
            items
                .into_iter()
                .map(|item| required_string(Some(&item)))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        (None, Some(reason)) => {
            ResidualFrictionTruth::ExplicitNone(required_string(Some(&reason))?)
        }
        _ => return Err(()),
    };

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

fn required_string(value: Option<&str>) -> Result<String, ()> {
    let Some(value) = value else {
        return Err(());
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(());
    }
    Ok(trimmed.to_string())
}

fn validate_commit(value: &str) -> Result<(), ()> {
    let valid = (7..=40).contains(&value.len())
        && value.chars().all(|ch| matches!(ch, '0'..='9' | 'a'..='f'));
    if valid {
        Ok(())
    } else {
        Err(())
    }
}

fn render_markdown_file(body: String) -> String {
    format!("{OWNERSHIP_MARKER}\n\n{body}")
}

fn render_readme_body(
    draft: &DraftEntry,
    phase: PacketPhase<'_>,
    closeout_path: &str,
    approval: Option<ApprovalRenderInput<'_>>,
) -> String {
    let packet_state = match phase {
        PacketPhase::Execution => "execution",
        PacketPhase::Closeout(_) => "closed_proving_run",
    };
    let summary = match phase {
        PacketPhase::Execution => format!(
            "This packet records the current onboarding handoff for `{}`.",
            draft.display_name
        ),
        PacketPhase::Closeout(_) => format!(
            "This packet records the closed proving run for `{}`.",
            draft.display_name
        ),
    };
    let closeout_line = match phase {
        PacketPhase::Execution => {
            format!(
                "Closeout metadata becomes authoritative at `{closeout_path}` once the proving run closes."
            ) + &render_execution_approval_linkage(approval)
        }
        PacketPhase::Closeout(closeout) => {
            format!("Closeout metadata is recorded in `{closeout_path}`.")
                + &format!(
                    "\n- Approval linkage: `{}` via `{}` (`sha256: {}`)",
                    closeout.approval_source, closeout.approval_ref, closeout.approval_sha256
                )
        }
    };

    format!(
        "# {} onboarding pack\n\n{}\n\n- Packet state: `{}`\n- Agent id: `{}`\n- Wrapper crate: `{}`\n- Backend module: `{}`\n- Manifest root: `{}`\n- {}\n",
        draft.display_name,
        summary,
        packet_state,
        draft.agent_id,
        draft.crate_path,
        draft.backend_module,
        draft.manifest_root,
        closeout_line
    )
}

fn render_scope_brief_body(
    draft: &DraftEntry,
    phase: PacketPhase<'_>,
    docs_root_display: &str,
    closeout_path: &str,
    approval: Option<ApprovalRenderInput<'_>>,
) -> String {
    match phase {
        PacketPhase::Execution => format!(
            "# Scope brief\n\nThis packet covers the control-plane-owned onboarding surfaces for `{}`.\n\n- Registry enrollment in `{REGISTRY_RELATIVE_PATH}`\n- Docs pack in `{docs_root_display}`\n- Manifest root in `{}`\n- Release/workspace touchpoints in `Cargo.toml` and `{RELEASE_DOC_PATH}`{}\n\nCurrent proving-run target: complete the runtime-owned wrapper/backend lane, commit manifest evidence, regenerate publication artifacts, and close with `make preflight`.\n",
            draft.agent_id,
            draft.manifest_root,
            render_execution_scope_approval(approval),
        ),
        PacketPhase::Closeout(closeout) => format!(
            "# Scope brief\n\nThis packet records the closed proving run for `{}`.\n\n- Registry enrollment in `{REGISTRY_RELATIVE_PATH}`\n- Docs pack in `{docs_root_display}`\n- Manifest root in `{}`\n- Closeout metadata in `{closeout_path}`\n- Approval linkage via `{}` (`{}`, sha256 `{}`)\n\nCloseout status: `make preflight` {} for this proving run.\n",
            draft.agent_id,
            draft.manifest_root,
            closeout.approval_ref,
            closeout.approval_source,
            closeout.approval_sha256,
            if closeout.preflight_passed { "passed" } else { "did not pass" }
        ),
    }
}

fn render_seam_map_body(draft: &DraftEntry, docs_root_display: &str) -> String {
    format!(
        "# Seam map\n\n- Declaration seam: registry entry for `{}`\n- Docs seam: onboarding pack `{docs_root_display}`\n- Manifest seam: `{}`\n- Runtime seam: wrapper crate `{}` and backend module `{}`\n",
        draft.agent_id,
        draft.manifest_root,
        draft.crate_path,
        draft.backend_module
    )
}

fn render_threading_body(draft: &DraftEntry, phase: PacketPhase<'_>) -> String {
    match phase {
        PacketPhase::Execution => format!(
            "# Threading\n\n1. Apply the control-plane onboarding packet with `onboard-agent --write`.\n2. Implement the runtime-owned wrapper crate at `{}` and backend module `{}`.\n3. Populate manifest evidence under `{}` from committed runtime outputs.\n4. Regenerate support/capability artifacts and close the proving run with `make preflight`.\n",
            draft.crate_path, draft.backend_module, draft.manifest_root
        ),
        PacketPhase::Closeout(_) => format!(
            "# Threading\n\n1. Control-plane onboarding writes for `{}` landed without follow-up packet drift.\n2. Runtime-owned wrapper and backend work landed at `{}` and `{}`.\n3. Manifest evidence and publication artifacts were regenerated from committed runtime outputs.\n4. The proving run closed with `make preflight`.\n",
            draft.agent_id,
            draft.crate_path,
            draft.backend_module
        ),
    }
}

fn render_review_surfaces_body(
    draft: &DraftEntry,
    phase: PacketPhase<'_>,
    docs_root_display: &str,
) -> String {
    let release_rails = match phase {
        PacketPhase::Execution => "Supporting release rails reviewed for this onboarding run",
        PacketPhase::Closeout(_) => {
            "Supporting release rails remained unchanged across the proving run"
        }
    };
    format!(
        "# Review surfaces\n\n- `{REGISTRY_RELATIVE_PATH}`\n- `{docs_root_display}`\n- `{}`\n- `{RELEASE_DOC_PATH}`\n- {}: `{PUBLISH_WORKFLOW_PATH}`, `{PUBLISH_SCRIPT_PATH}`, `{VALIDATE_PUBLISH_SCRIPT_PATH}`, `{CHECK_PUBLISH_READINESS_SCRIPT_PATH}`\n",
        draft.manifest_root, release_rails
    )
}

fn render_remediation_log_body(phase: PacketPhase<'_>) -> String {
    match phase {
        PacketPhase::Execution => {
            "# Remediation log\n\nNo remediations are recorded yet. Capture residual friction or follow-up decisions here if the proving run surfaces them.\n".to_string()
        }
        PacketPhase::Closeout(closeout) => format!(
            "# Remediation log\n\n{}\n",
            render_residual_friction_lines(closeout)
        ),
    }
}

fn render_handoff_body(
    draft: &DraftEntry,
    phase: PacketPhase<'_>,
    closeout_path: &str,
    release_touchpoints: &str,
    approval: Option<ApprovalRenderInput<'_>>,
) -> String {
    match phase {
        PacketPhase::Execution => format!(
            "# Handoff\n\nThis packet captures the next executable onboarding step for `{}`.\n\n## Release touchpoints\n\n{}\n\n{}\n## Next executable runtime step\n\nImplement the runtime-owned wrapper crate at `{}` and backend module `{}`.\n\n## Remaining runtime checklist\n\n- Author wrapper coverage input at `{}` for binding kind `{}`.\n- Populate `{}/current.json`, pointers, versions, and reports from committed runtime evidence.\n- Regenerate support and capability publication artifacts, then run `make preflight`.\n",
            draft.agent_id,
            release_touchpoints,
            render_execution_handoff_approval(closeout_path, approval),
            draft.crate_path,
            draft.backend_module,
            draft.wrapper_coverage_source_path,
            draft.wrapper_coverage_binding_kind,
            draft.manifest_root
        ),
        PacketPhase::Closeout(closeout) => format!(
            "# Handoff\n\nThis packet records the closed proving run for `{}`.\n\n## Release touchpoints\n\n{}\n\n## Proving-run closeout\n\n- approval ref: `{}`\n- approval source: `{}`\n- approval artifact sha256: `{}`\n- manual control-plane file edits by maintainers: `{}`\n- partial-write incidents: `{}`\n- ambiguous ownership incidents: `{}`\n- approved-agent to repo-ready control-plane mutation time: `{}`\n- proving-run closeout passes `make preflight`: `{}`\n- recorded at: `{}`\n- commit: `{}`\n- closeout metadata: `{}`\n\n## Residual friction\n\n{}\n\n## Status\n\nNo open runtime next step remains in this packet.\n",
            draft.agent_id,
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
        ),
    }
}

fn render_closeout_duration(closeout: &ProvingRunCloseout) -> String {
    match &closeout.duration {
        DurationTruth::Seconds(seconds) => format!("{seconds}s"),
        DurationTruth::MissingReason(reason) => format!("missing ({reason})"),
    }
}

fn render_execution_approval_linkage(approval: Option<ApprovalRenderInput<'_>>) -> String {
    match approval {
        Some(approval) => format!(
            "\n- Approval linkage: `{}` (`sha256: {}`)",
            approval.artifact_path, approval.artifact_sha256
        ),
        None => String::new(),
    }
}

fn render_execution_scope_approval(approval: Option<ApprovalRenderInput<'_>>) -> String {
    match approval {
        Some(approval) => format!(
            "\n- Approval linkage via `{}` (`sha256: {}`)",
            approval.artifact_path, approval.artifact_sha256
        ),
        None => String::new(),
    }
}

fn render_execution_handoff_approval(
    closeout_path: &str,
    approval: Option<ApprovalRenderInput<'_>>,
) -> String {
    match approval {
        Some(approval) => format!(
            "## Approval provenance\n\n- approval ref: `{}`\n- approval artifact sha256: `{}`\n- closeout metadata will become authoritative at `{}`.\n\n",
            approval.artifact_path, approval.artifact_sha256, closeout_path
        ),
        None => String::new(),
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

fn docs_pack_root(prefix: &str) -> PathBuf {
    Path::new(DOCS_NEXT_ROOT).join(prefix)
}
