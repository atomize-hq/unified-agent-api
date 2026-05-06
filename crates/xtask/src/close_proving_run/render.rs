use xtask::{
    agent_registry::AgentRegistryEntry,
    proving_run_closeout::{DurationTruth, ProvingRunCloseout, ResidualFrictionTruth},
};

const OWNERSHIP_MARKER: &str = "<!-- generated-by: xtask onboard-agent; owner: control-plane -->";
const REGISTRY_ENTRY_PATH: &str = "crates/xtask/data/agent_registry.toml";
const RELEASE_DOC_PATH: &str = "docs/crates-io-release.md";
const PUBLISH_WORKFLOW_PATH: &str = ".github/workflows/publish-crates.yml";
const PUBLISH_SCRIPT_PATH: &str = "scripts/publish_crates.py";
const VALIDATE_PUBLISH_SCRIPT_PATH: &str = "scripts/validate_publish_versions.py";
const CHECK_PUBLISH_READINESS_SCRIPT_PATH: &str = "scripts/check_publish_readiness.py";

pub(super) fn release_touchpoint_lines(entry: &AgentRegistryEntry) -> Vec<String> {
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

pub(super) fn render_markdown_file(body: String) -> String {
    format!("{OWNERSHIP_MARKER}\n\n{body}")
}

pub(super) fn render_readme_body(
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

pub(super) fn render_scope_brief_body(
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
        if closeout.preflight_passed { "passed" } else { "did not pass" }
    )
}

pub(super) fn render_seam_map_body(entry: &AgentRegistryEntry, docs_root_display: &str) -> String {
    format!(
        "# Seam map\n\n- Declaration seam: registry entry for `{}`\n- Docs seam: onboarding pack `{docs_root_display}`\n- Manifest seam: `{}`\n- Runtime seam: wrapper crate `{}` and backend module `{}`\n",
        entry.agent_id,
        entry.manifest_root,
        entry.crate_path,
        entry.backend_module
    )
}

pub(super) fn render_threading_body(entry: &AgentRegistryEntry) -> String {
    format!(
        "# Threading\n\n1. Control-plane onboarding writes for `{}` landed without follow-up packet drift.\n2. Runtime-owned wrapper and backend work landed at `{}` and `{}`.\n3. Manifest evidence and publication artifacts were regenerated from committed runtime outputs.\n4. The proving run closed with `make preflight`.\n",
        entry.agent_id,
        entry.crate_path,
        entry.backend_module
    )
}

pub(super) fn render_review_surfaces_body(
    entry: &AgentRegistryEntry,
    docs_root_display: &str,
) -> String {
    format!(
        "# Review surfaces\n\n- `{REGISTRY_ENTRY_PATH}`\n- `{docs_root_display}`\n- `{}`\n- `{RELEASE_DOC_PATH}`\n- Supporting release rails remained unchanged across the proving run: `{PUBLISH_WORKFLOW_PATH}`, `{PUBLISH_SCRIPT_PATH}`, `{VALIDATE_PUBLISH_SCRIPT_PATH}`, `{CHECK_PUBLISH_READINESS_SCRIPT_PATH}`\n",
        entry.manifest_root
    )
}

pub(super) fn render_remediation_log_body(closeout: &ProvingRunCloseout) -> String {
    format!(
        "# Remediation log\n\n{}\n",
        render_residual_friction_lines(closeout)
    )
}

pub(super) fn render_handoff_body(
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
