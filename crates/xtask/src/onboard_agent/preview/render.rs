use std::path::{Path, PathBuf};

use crate::agent_registry::REGISTRY_RELATIVE_PATH;
use crate::proving_run_closeout::{DurationTruth, ProvingRunCloseout, ResidualFrictionTruth};

use super::super::{
    CHECK_PUBLISH_READINESS_SCRIPT_PATH, OWNERSHIP_MARKER, PUBLISH_SCRIPT_PATH,
    PUBLISH_WORKFLOW_PATH, RELEASE_DOC_PATH, VALIDATE_PUBLISH_SCRIPT_PATH,
};
use super::ApprovalRenderInput;
use super::DraftEntry;
const DOCS_NEXT_ROOT: &str = "docs/agents/lifecycle";
const PROVING_RUN_CLOSEOUT_RELATIVE_PATH: &str = "governance/proving-run-closeout.json";

#[derive(Debug, Clone, Copy)]
pub(in crate::onboard_agent) enum PacketPhase<'a> {
    Execution,
    Closeout(&'a ProvingRunCloseout),
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
        "# Seam map\n\n- Declaration seam: registry entry for `{}`\n- Docs seam: onboarding pack `{docs_root_display}`\n- Manifest seam: `{}`\n- Runtime seam: wrapper crate shell `{}` via `scaffold-wrapper-crate` and backend module `{}`\n",
        draft.agent_id,
        draft.manifest_root,
        draft.crate_path,
        draft.backend_module
    )
}

fn render_threading_body(draft: &DraftEntry, phase: PacketPhase<'_>) -> String {
    match phase {
        PacketPhase::Execution => format!(
            "# Threading\n\n1. Apply the control-plane onboarding packet with `onboard-agent --write`.\n2. Run `cargo run -p xtask -- scaffold-wrapper-crate --agent {} --write` to create the runtime-owned wrapper crate shell at `{}`; `onboard-agent` does not create the wrapper crate.\n3. Materialize the bounded runtime packet with `runtime-follow-on --dry-run`, then implement backend/runtime details in `{}` and `{}`.\n4. Keep runtime evidence inside `{}/snapshots/**` and `{}/supplement/**`, complete `runtime-follow-on --write`, then hand publication refresh and proving-run closeout to the next lane.\n",
            draft.agent_id,
            draft.crate_path,
            draft.crate_path,
            draft.backend_module,
            draft.manifest_root,
            draft.manifest_root
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
            "# Handoff\n\nThis packet captures the next executable onboarding step for `{}`.\n\n## Release touchpoints\n\n{}\n\n{}\n## Next executable runtime step\n\nRun `cargo run -p xtask -- scaffold-wrapper-crate --agent {} --write` to create the runtime-owned wrapper crate shell at `{}`. `onboard-agent` does not create the wrapper crate.\n\n## Remaining runtime checklist\n\n- After scaffolding, materialize the bounded runtime packet with `runtime-follow-on --dry-run` for this approval artifact.\n- Implement backend/runtime details in `{}` and `{}`.\n- Author wrapper coverage input at `{}` for binding kind `{}`.\n- Populate committed runtime evidence only under `{}/snapshots/**` and `{}/supplement/**`.\n- Complete `runtime-follow-on --write`; publication refresh and proving-run closeout stay in the next lane.\n",
            draft.agent_id,
            release_touchpoints,
            render_execution_handoff_approval(closeout_path, approval),
            draft.agent_id,
            draft.crate_path,
            draft.crate_path,
            draft.backend_module,
            draft.wrapper_coverage_source_path,
            draft.wrapper_coverage_binding_kind,
            draft.manifest_root,
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
