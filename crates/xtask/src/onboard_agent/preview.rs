mod release;
mod render;
mod write;

use std::{fmt::Write as _, path::Path};

use crate::agent_registry::AgentRegistryEntry;
use crate::proving_run_closeout::{
    load_validated_closeout_if_present_with_states, ProvingRunCloseout, ProvingRunCloseoutError,
    ProvingRunCloseoutExpected, ProvingRunCloseoutState,
};
use crate::workspace_mutation::WorkspacePathJail;

use self::render::{
    build_closeout_docs_preview as render_closeout_docs_preview,
    build_docs_preview as render_docs_preview, closeout_relative_path, CloseoutPacketRenderInput,
};
use super::{
    ApprovalMaintenanceModeSummary, ConfigGate, DraftEntry, Error, LifecycleStatePreview,
    TargetGate, RELEASE_DOC_PATH,
};

const RELEASE_DOC_START_MARKER: &str =
    "<!-- generated-by: xtask onboard-agent; section: crates-io-release -->";
const RELEASE_DOC_END_MARKER: &str =
    "<!-- /generated-by: xtask onboard-agent; section: crates-io-release -->";
const WRAPPER_EVENTS_PACKAGE_NAME: &str = "unified-agent-api-wrapper-events";
const ROOT_AGENT_API_PACKAGE_NAME: &str = "unified-agent-api";

#[derive(Debug, Clone, Copy)]
pub(super) struct ApprovalRenderInput<'a> {
    pub(super) artifact_path: &'a str,
    pub(super) artifact_sha256: &'a str,
}

#[derive(Debug)]
pub(super) struct ReleasePreview {
    pub(super) lines: Vec<String>,
    pub(super) workspace_manifest: TextMutationPlan,
    pub(super) release_doc: TextMutationPlan,
}

#[derive(Debug, Clone)]
pub(super) struct TextMutationPlan {
    pub(super) path: String,
    pub(super) expected_before: String,
    pub(super) desired_after: String,
}

pub(super) fn build_release_preview(
    workspace_root: &Path,
    draft: &DraftEntry,
) -> Result<ReleasePreview, Error> {
    release::build_release_preview(workspace_root, draft)
}

pub(super) fn load_proving_run_metrics(
    workspace_root: &Path,
    draft: &DraftEntry,
) -> Result<Option<ProvingRunCloseout>, Error> {
    let closeout_path = draft
        .docs_pack_root()
        .join("governance/proving-run-closeout.json");
    let jail = WorkspacePathJail::new(workspace_root)?;
    let resolved_closeout_path = jail.resolve(&closeout_path)?;
    let expected = ProvingRunCloseoutExpected {
        approval_path: draft.approval_identity().map(|(path, _)| Path::new(path)),
        onboarding_pack_prefix: &draft.onboarding_pack_prefix,
    };
    load_validated_closeout_if_present_with_states(
        workspace_root,
        Path::new(&closeout_relative_path(draft)),
        &resolved_closeout_path,
        expected,
        &[
            ProvingRunCloseoutState::Prepared,
            ProvingRunCloseoutState::Closed,
        ],
    )
    .map_err(map_closeout_error)
}

fn map_closeout_error(err: ProvingRunCloseoutError) -> Error {
    match err {
        ProvingRunCloseoutError::Validation(message) => Error::Validation(message),
        ProvingRunCloseoutError::Internal(message) => Error::Internal(message),
    }
}

pub(super) fn write_input_summary<W: std::io::Write>(
    writer: &mut W,
    draft: &DraftEntry,
) -> Result<(), Error> {
    write::write_input_summary(writer, draft)
}

pub(super) fn write_registry_preview<W: std::io::Write>(
    writer: &mut W,
    preview: &str,
) -> Result<(), Error> {
    write::write_registry_preview(writer, preview)
}

pub(super) fn write_docs_preview<W: std::io::Write>(
    writer: &mut W,
    previews: &[(String, Option<String>)],
) -> Result<(), Error> {
    write::write_docs_preview(writer, previews)
}

pub(super) fn write_manifest_preview<W: std::io::Write>(
    writer: &mut W,
    previews: &[(String, Option<String>)],
) -> Result<(), Error> {
    write::write_manifest_preview(writer, previews)
}

pub(super) fn write_lifecycle_state_preview<W: std::io::Write>(
    writer: &mut W,
    preview: &LifecycleStatePreview,
) -> Result<(), Error> {
    write::write_lifecycle_state_preview(writer, preview)
}

pub(super) fn write_release_preview<W: std::io::Write>(
    writer: &mut W,
    preview: &ReleasePreview,
) -> Result<(), Error> {
    write::write_release_preview(writer, preview)
}

pub(super) fn write_manual_follow_up<W: std::io::Write>(
    writer: &mut W,
    lines: &[String],
) -> Result<(), Error> {
    write::write_manual_follow_up(writer, lines)
}

pub(super) fn render_registry_entry_preview(draft: &DraftEntry) -> String {
    let mut out = String::new();
    writeln!(&mut out, "[[agents]]").expect("write String");
    writeln!(&mut out, "agent_id = {:?}", draft.agent_id).expect("write String");
    writeln!(&mut out, "display_name = {:?}", draft.display_name).expect("write String");
    writeln!(&mut out, "crate_path = {:?}", draft.crate_path).expect("write String");
    writeln!(&mut out, "backend_module = {:?}", draft.backend_module).expect("write String");
    writeln!(&mut out, "manifest_root = {:?}", draft.manifest_root).expect("write String");
    writeln!(&mut out, "package_name = {:?}", draft.package_name).expect("write String");
    writeln!(
        &mut out,
        "canonical_targets = {}",
        render_string_array(&draft.canonical_targets)
    )
    .expect("write String");
    writeln!(&mut out).expect("write String");

    writeln!(&mut out, "[agents.wrapper_coverage]").expect("write String");
    writeln!(
        &mut out,
        "binding_kind = {:?}",
        draft.wrapper_coverage_binding_kind
    )
    .expect("write String");
    writeln!(
        &mut out,
        "source_path = {:?}",
        draft.wrapper_coverage_source_path
    )
    .expect("write String");
    writeln!(&mut out).expect("write String");

    writeln!(&mut out, "[agents.capability_declaration]").expect("write String");
    writeln!(
        &mut out,
        "always_on = {}",
        render_string_array(&draft.always_on_capabilities)
    )
    .expect("write String");
    writeln!(
        &mut out,
        "backend_extensions = {}",
        render_string_array(&draft.backend_extensions)
    )
    .expect("write String");
    if !draft.target_gated_capabilities.is_empty() {
        writeln!(&mut out).expect("write String");
    }

    for target_gate in &draft.target_gated_capabilities {
        writeln!(&mut out, "[[agents.capability_declaration.target_gated]]").expect("write String");
        writeln!(&mut out, "capability_id = {:?}", target_gate.capability_id)
            .expect("write String");
        writeln!(
            &mut out,
            "targets = {}",
            render_string_array(&target_gate.targets)
        )
        .expect("write String");
        writeln!(&mut out).expect("write String");
    }

    for config_gate in &draft.config_gated_capabilities {
        writeln!(&mut out, "[[agents.capability_declaration.config_gated]]").expect("write String");
        writeln!(&mut out, "capability_id = {:?}", config_gate.capability_id)
            .expect("write String");
        writeln!(&mut out, "config_key = {:?}", config_gate.config_key).expect("write String");
        if let Some(targets) = &config_gate.targets {
            writeln!(&mut out, "targets = {}", render_string_array(targets)).expect("write String");
        }
        writeln!(&mut out).expect("write String");
    }

    writeln!(&mut out, "[agents.publication]").expect("write String");
    writeln!(
        &mut out,
        "support_matrix_enabled = {}",
        draft.support_matrix_enabled
    )
    .expect("write String");
    writeln!(
        &mut out,
        "capability_matrix_enabled = {}",
        draft.capability_matrix_enabled
    )
    .expect("write String");
    if let Some(target) = draft.capability_matrix_target.as_deref() {
        writeln!(&mut out, "capability_matrix_target = {:?}", target).expect("write String");
    }
    writeln!(&mut out).expect("write String");

    writeln!(&mut out, "[agents.release]").expect("write String");
    writeln!(
        &mut out,
        "docs_release_track = {:?}",
        draft.docs_release_track
    )
    .expect("write String");
    writeln!(&mut out).expect("write String");

    writeln!(&mut out, "[agents.scaffold]").expect("write String");
    writeln!(
        &mut out,
        "onboarding_pack_prefix = {:?}",
        draft.onboarding_pack_prefix
    )
    .expect("write String");
    out
}

pub(super) fn build_docs_preview(
    draft: &DraftEntry,
    release_preview: &ReleasePreview,
    closeout: Option<&ProvingRunCloseout>,
) -> Vec<(String, Option<String>)> {
    match closeout {
        Some(closeout) => {
            let input = CloseoutPacketRenderInput::from_draft(draft);
            render_closeout_docs_preview(&input, closeout)
        }
        None => {
            let approval = approval_render_input(draft);
            render_docs_preview(draft, &release_preview.lines, approval)
        }
    }
}

pub(crate) fn build_closeout_docs_preview_for_entry(
    entry: &AgentRegistryEntry,
    closeout: &ProvingRunCloseout,
) -> Vec<(String, Option<String>)> {
    let input = CloseoutPacketRenderInput::from_registry_entry(entry);
    render_closeout_docs_preview(&input, closeout)
}

fn approval_render_input(draft: &DraftEntry) -> Option<ApprovalRenderInput<'_>> {
    let (artifact_path, artifact_sha256) = draft.approval_identity()?;
    Some(ApprovalRenderInput {
        artifact_path,
        artifact_sha256,
    })
}

pub(super) fn build_manifest_preview(draft: &DraftEntry) -> Vec<(String, Option<String>)> {
    vec![
        (
            Path::new(&draft.manifest_root)
                .join("current.json")
                .display()
                .to_string(),
            Some(render_current_json(draft)),
        ),
        (
            Path::new(&draft.manifest_root)
                .join("versions/.gitkeep")
                .display()
                .to_string(),
            None,
        ),
        (
            Path::new(&draft.manifest_root)
                .join("pointers/latest_supported/.gitkeep")
                .display()
                .to_string(),
            None,
        ),
        (
            Path::new(&draft.manifest_root)
                .join("pointers/latest_validated/.gitkeep")
                .display()
                .to_string(),
            None,
        ),
        (
            Path::new(&draft.manifest_root)
                .join("reports/.gitkeep")
                .display()
                .to_string(),
            None,
        ),
    ]
}

fn render_current_json(draft: &DraftEntry) -> String {
    let targets = draft
        .canonical_targets
        .iter()
        .map(|target| format!("    \"{target}\""))
        .collect::<Vec<_>>()
        .join(",\n");
    format!("{{\n  \"expected_targets\": [\n{targets}\n  ],\n  \"inputs\": []\n}}\n")
}

pub(super) fn build_manual_follow_up(
    draft: &DraftEntry,
    closeout: Option<&ProvingRunCloseout>,
) -> Vec<String> {
    let mut approval_maintenance_lines = Vec::new();
    if let Some(maintenance) = draft.approval_maintenance.as_ref() {
        match &maintenance.mode {
            ApprovalMaintenanceModeSummary::ReleaseWatchEnrolled {
                dispatch_kind,
                dispatch_workflow,
                ..
            } => {
                let dispatch = dispatch_workflow
                    .as_deref()
                    .map(|workflow| format!("{dispatch_kind} via `{workflow}`"))
                    .unwrap_or_else(|| dispatch_kind.to_string());
                approval_maintenance_lines.push(format!(
                    "Approval maintenance is already enrolled for release-watch handling ({dispatch}); keep that truth intact in downstream lanes."
                ));
            }
            ApprovalMaintenanceModeSummary::ExplicitlyDeferred {
                reason, follow_up, ..
            } => {
                approval_maintenance_lines.push(format!(
                    "Approval maintenance is explicitly deferred for this onboarding packet; do not write release-watch enrollment from this lane. Reason: {reason}"
                ));
                approval_maintenance_lines.push(format!(
                    "Deferred maintenance follow-up remains manual until closeout: {follow_up}"
                ));
            }
        }
    }

    match closeout {
        Some(_) => {
            vec!["No open runtime follow-up remains; the proving run is closed.".to_string()]
        }
        None => {
            let mut lines = approval_maintenance_lines;
            lines.extend([
                format!(
                    "Next executable runtime step: run `cargo run -p xtask -- scaffold-wrapper-crate --agent {} --write` to create the runtime-owned wrapper crate shell at `{}`; `onboard-agent` does not create the wrapper crate.",
                    draft.agent_id, draft.crate_path
                ),
                "Then materialize the bounded runtime packet with `runtime-follow-on --dry-run`."
                    .to_string(),
                format!(
                    "Implement backend/runtime details in `{}` and `{}`.",
                    draft.crate_path, draft.backend_module
                ),
                format!(
                    "Author wrapper coverage input at `{}` for binding kind `{}`.",
                    draft.wrapper_coverage_source_path, draft.wrapper_coverage_binding_kind
                ),
                format!(
                    "Populate committed runtime evidence only under `{}/snapshots/**` and `{}/supplement/**`.",
                    draft.manifest_root, draft.manifest_root
                ),
                "Complete `runtime-follow-on --write`; publication refresh and `make preflight` stay in the next lane."
                    .to_string(),
            ]);
            lines
        }
    }
}

fn render_string_array(values: &[String]) -> String {
    let rendered = values
        .iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>();
    format!("[{}]", rendered.join(", "))
}
