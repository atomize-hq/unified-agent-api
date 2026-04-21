use crate::agent_registry::REGISTRY_RELATIVE_PATH;
use serde::Deserialize;

use super::{
    DraftEntry, CHECK_PUBLISH_READINESS_SCRIPT_PATH, PUBLISH_SCRIPT_PATH, PUBLISH_WORKFLOW_PATH,
    RELEASE_DOC_PATH, VALIDATE_PUBLISH_SCRIPT_PATH,
};

#[derive(Debug, Clone, Deserialize)]
pub(in crate::onboard_agent) struct ProvingRunMetrics {
    pub(super) manual_control_plane_edits: u64,
    pub(super) partial_write_incidents: u64,
    pub(super) ambiguous_ownership_incidents: u64,
    pub(super) control_plane_mutation_duration_seconds: Option<u64>,
    pub(super) control_plane_mutation_duration_recorded: bool,
    pub(super) control_plane_mutation_duration_note: Option<String>,
    pub(super) preflight_passed: bool,
    pub(super) residual_friction: Vec<String>,
    pub(super) recorded_at: String,
    pub(super) commit: String,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum PacketPhase<'a> {
    Execution,
    Closeout(&'a ProvingRunMetrics),
}

pub(super) fn render_readme_body(
    draft: &DraftEntry,
    phase: PacketPhase<'_>,
    metrics_path: &str,
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
                "Closeout metadata becomes authoritative at `{metrics_path}` once the proving run closes."
            )
        }
        PacketPhase::Closeout(_) => {
            format!("Closeout metadata is recorded in `{metrics_path}`.")
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

pub(super) fn render_scope_brief_body(
    draft: &DraftEntry,
    phase: PacketPhase<'_>,
    docs_root_display: &str,
    metrics_path: &str,
) -> String {
    match phase {
        PacketPhase::Execution => format!(
            "# Scope brief\n\nThis packet covers the control-plane-owned onboarding surfaces for `{}`.\n\n- Registry enrollment in `{REGISTRY_RELATIVE_PATH}`\n- Docs pack in `{docs_root_display}`\n- Manifest root in `{}`\n- Release/workspace touchpoints in `Cargo.toml` and `{RELEASE_DOC_PATH}`\n\nCurrent proving-run target: complete the runtime-owned wrapper/backend lane, commit manifest evidence, regenerate publication artifacts, and close with `make preflight`.\n",
            draft.agent_id,
            draft.manifest_root
        ),
        PacketPhase::Closeout(metrics) => format!(
            "# Scope brief\n\nThis packet records the closed proving run for `{}`.\n\n- Registry enrollment in `{REGISTRY_RELATIVE_PATH}`\n- Docs pack in `{docs_root_display}`\n- Manifest root in `{}`\n- Closeout metadata in `{metrics_path}`\n\nCloseout status: `make preflight` {} for this proving run.\n",
            draft.agent_id,
            draft.manifest_root,
            if metrics.preflight_passed { "passed" } else { "did not pass" }
        ),
    }
}

pub(super) fn render_threading_body(draft: &DraftEntry, phase: PacketPhase<'_>) -> String {
    match phase {
        PacketPhase::Execution => format!(
            "# Threading\n\n1. Apply the control-plane onboarding packet with `onboard-agent --write`.\n2. {}.\n3. Populate manifest evidence under `{}` from committed runtime outputs.\n4. Regenerate support/capability artifacts and close the proving run with `make preflight`.\n",
            execution_next_runtime_step(draft),
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

pub(super) fn render_review_surfaces_body(
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

pub(super) fn render_remediation_log_body(phase: PacketPhase<'_>) -> String {
    match phase {
        PacketPhase::Execution => {
            "# Remediation log\n\nNo remediations are recorded yet. Capture residual friction or follow-up decisions here if the proving run surfaces them.\n".to_string()
        }
        PacketPhase::Closeout(metrics) => format!(
            "# Remediation log\n\n{}\n",
            render_residual_friction_lines(metrics)
        ),
    }
}

pub(super) fn execution_next_runtime_step(draft: &DraftEntry) -> String {
    format!(
        "Implement the runtime-owned wrapper crate at `{}` and backend module `{}`",
        draft.crate_path, draft.backend_module
    )
}

fn render_closeout_duration(metrics: &ProvingRunMetrics) -> String {
    match (
        metrics.control_plane_mutation_duration_seconds,
        metrics.control_plane_mutation_duration_recorded,
    ) {
        (Some(seconds), true) => format!("{seconds}s"),
        (Some(seconds), false) => format!("{seconds}s (manual backfill)"),
        (None, _) => "not recorded".to_string(),
    }
}

fn render_residual_friction_lines(metrics: &ProvingRunMetrics) -> String {
    let mut lines = Vec::new();
    if metrics.residual_friction.is_empty() {
        lines.push("- No residual friction recorded.".to_string());
    } else {
        lines.extend(
            metrics
                .residual_friction
                .iter()
                .map(|item| format!("- {item}")),
        );
    }
    if let Some(note) = &metrics.control_plane_mutation_duration_note {
        lines.push(format!("- Timing note: {note}"));
    }
    lines.join("\n")
}

pub(super) fn render_handoff_body(
    draft: &DraftEntry,
    phase: PacketPhase<'_>,
    metrics_path: &str,
    release_touchpoints: &str,
    manual_follow_up: &[String],
) -> String {
    match phase {
        PacketPhase::Execution => format!(
            "# Handoff\n\nThis packet captures the next executable onboarding step for `{}`.\n\n## Release touchpoints\n\n{}\n\n## Next executable runtime step\n\n{}.\n\n## Remaining runtime checklist\n\n{}\n",
            draft.agent_id,
            release_touchpoints,
            execution_next_runtime_step(draft),
            manual_follow_up
                .iter()
                .skip(1)
                .map(|line| format!("- {line}"))
                .collect::<Vec<_>>()
                .join("\n")
        ),
        PacketPhase::Closeout(metrics) => format!(
            "# Handoff\n\nThis packet records the closed proving run for `{}`.\n\n## Release touchpoints\n\n{}\n\n## Proving-run closeout\n\n- manual control-plane file edits by maintainers: `{}`\n- partial-write incidents: `{}`\n- ambiguous ownership incidents: `{}`\n- approved-agent to repo-ready control-plane mutation time: `{}`\n- proving-run closeout passes `make preflight`: `{}`\n- recorded at: `{}`\n- commit: `{}`\n- closeout metadata: `{}`\n\n## Residual friction\n\n{}\n\n## Status\n\nNo open runtime next step remains in this packet.\n",
            draft.agent_id,
            release_touchpoints,
            metrics.manual_control_plane_edits,
            metrics.partial_write_incidents,
            metrics.ambiguous_ownership_incidents,
            render_closeout_duration(metrics),
            metrics.preflight_passed,
            metrics.recorded_at,
            metrics.commit,
            metrics_path,
            render_residual_friction_lines(metrics)
        ),
    }
}

pub(super) fn render_current_json(draft: &DraftEntry) -> String {
    let targets = draft
        .canonical_targets
        .iter()
        .map(|target| format!("    \"{target}\""))
        .collect::<Vec<_>>()
        .join(",\n");
    format!("{{\n  \"expected_targets\": [\n{targets}\n  ],\n  \"inputs\": []\n}}\n")
}
