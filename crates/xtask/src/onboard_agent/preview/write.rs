use std::io::Write;

use crate::agent_registry::REGISTRY_RELATIVE_PATH;

use super::{
    approval_render_input, ApprovalMaintenanceModeSummary, ConfigGate, DraftEntry, Error,
    LifecycleStatePreview, ReleasePreview, TargetGate,
};

pub(super) fn write_input_summary<W: Write>(
    writer: &mut W,
    draft: &DraftEntry,
) -> Result<(), Error> {
    let approval = approval_render_input(draft);
    writeln!(writer, "== INPUT SUMMARY ==")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "agent_id: {}", draft.agent_id)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "display_name: {}", draft.display_name)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "crate_path: {}", draft.crate_path)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "backend_module: {}", draft.backend_module)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "manifest_root: {}", draft.manifest_root)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "package_name: {}", draft.package_name)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    write_list(writer, "canonical_targets", &draft.canonical_targets)?;
    writeln!(
        writer,
        "wrapper_coverage_binding_kind: {}",
        draft.wrapper_coverage_binding_kind
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "wrapper_coverage_source_path: {}",
        draft.wrapper_coverage_source_path
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    write_list(
        writer,
        "always_on_capabilities",
        &draft.always_on_capabilities,
    )?;
    write_gate_summaries(
        writer,
        &draft.target_gated_capabilities,
        &draft.config_gated_capabilities,
    )?;
    write_list(writer, "backend_extensions", &draft.backend_extensions)?;
    writeln!(
        writer,
        "support_matrix_enabled: {}",
        draft.support_matrix_enabled
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "capability_matrix_enabled: {}",
        draft.capability_matrix_enabled
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    if let Some(target) = draft.capability_matrix_target.as_deref() {
        writeln!(writer, "capability_matrix_target: {target}")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    }
    writeln!(writer, "docs_release_track: {}", draft.docs_release_track)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "onboarding_pack_prefix: {}",
        draft.onboarding_pack_prefix
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    if let Some(approval) = approval {
        writeln!(writer, "approval_artifact_path: {}", approval.artifact_path)
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        writeln!(
            writer,
            "approval_artifact_sha256: {}",
            approval.artifact_sha256
        )
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    }
    if let Some(maintenance) = draft.approval_maintenance.as_ref() {
        writeln!(
            writer,
            "approval_maintenance_mode: {}",
            maintenance.mode_name()
        )
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        writeln!(
            writer,
            "approval_maintenance_section_sha256: {}",
            maintenance.section_sha256
        )
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        match &maintenance.mode {
            ApprovalMaintenanceModeSummary::ReleaseWatchEnrolled {
                version_policy,
                dispatch_kind,
                dispatch_workflow,
                upstream,
                release_watch_sha256,
            } => {
                writeln!(
                    writer,
                    "approval_release_watch_version_policy: {version_policy}"
                )
                .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
                writeln!(
                    writer,
                    "approval_release_watch_dispatch_kind: {dispatch_kind}"
                )
                .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
                if let Some(dispatch_workflow) = dispatch_workflow {
                    writeln!(
                        writer,
                        "approval_release_watch_dispatch_workflow: {dispatch_workflow}"
                    )
                    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
                }
                writeln!(writer, "approval_release_watch_upstream: {upstream}")
                    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
                writeln!(
                    writer,
                    "approval_release_watch_sha256: {release_watch_sha256}"
                )
                .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
            }
            ApprovalMaintenanceModeSummary::ExplicitlyDeferred {
                reason,
                follow_up,
                approved_scope,
                deferral_sha256,
            } => {
                writeln!(writer, "approval_maintenance_deferral_reason: {reason}")
                    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
                writeln!(
                    writer,
                    "approval_maintenance_deferral_follow_up: {follow_up}"
                )
                .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
                writeln!(
                    writer,
                    "approval_maintenance_deferral_scope: {approved_scope}"
                )
                .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
                writeln!(
                    writer,
                    "approval_maintenance_deferral_sha256: {deferral_sha256}"
                )
                .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
            }
        }
    }
    writeln!(writer).map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

pub(super) fn write_registry_preview<W: Write>(writer: &mut W, preview: &str) -> Result<(), Error> {
    writeln!(writer, "== REGISTRY ENTRY PREVIEW ==")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "Path: {REGISTRY_RELATIVE_PATH}")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    write_code_block(writer, "toml", preview)?;
    writeln!(writer).map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

pub(super) fn write_docs_preview<W: Write>(
    writer: &mut W,
    previews: &[(String, Option<String>)],
) -> Result<(), Error> {
    write_scaffold_preview(writer, "== DOCS SCAFFOLD PREVIEW ==", "md", previews)
}

pub(super) fn write_manifest_preview<W: Write>(
    writer: &mut W,
    previews: &[(String, Option<String>)],
) -> Result<(), Error> {
    write_scaffold_preview(writer, "== MANIFEST ROOT PREVIEW ==", "json", previews)
}

pub(super) fn write_lifecycle_state_preview<W: Write>(
    writer: &mut W,
    preview: &LifecycleStatePreview,
) -> Result<(), Error> {
    writeln!(writer, "== LIFECYCLE STATE PREVIEW ==")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "Path: {}", preview.path)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    write_code_block(writer, "json", &preview.contents)?;
    writeln!(writer).map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

pub(super) fn write_release_preview<W: Write>(
    writer: &mut W,
    preview: &ReleasePreview,
) -> Result<(), Error> {
    writeln!(writer, "== RELEASE/PUBLICATION TOUCHPOINTS ==")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    for line in &preview.lines {
        writeln!(writer, "{line}")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    }
    writeln!(writer).map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

pub(super) fn write_manual_follow_up<W: Write>(
    writer: &mut W,
    lines: &[String],
) -> Result<(), Error> {
    writeln!(writer, "== MANUAL FOLLOW-UP ==")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    for line in lines {
        writeln!(writer, "- {line}")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    }
    writeln!(writer).map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

fn write_scaffold_preview<W: Write>(
    writer: &mut W,
    header: &str,
    language: &str,
    previews: &[(String, Option<String>)],
) -> Result<(), Error> {
    writeln!(writer, "{header}").map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    for (path, contents) in previews {
        writeln!(writer, "Path: {path}")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        if let Some(contents) = contents {
            write_code_block(writer, language, contents)?;
        } else {
            writeln!(writer, "(empty file)")
                .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        }
    }
    writeln!(writer).map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

fn write_list<W: Write>(writer: &mut W, label: &str, values: &[String]) -> Result<(), Error> {
    writeln!(writer, "{label}:").map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    if values.is_empty() {
        writeln!(writer, "- (none)")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        return Ok(());
    }
    for value in values {
        writeln!(writer, "- {value}")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    }
    Ok(())
}

fn write_gate_summaries<W: Write>(
    writer: &mut W,
    target_gated: &[TargetGate],
    config_gated: &[ConfigGate],
) -> Result<(), Error> {
    writeln!(writer, "target_gated_capabilities:")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    if target_gated.is_empty() {
        writeln!(writer, "- (none)")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    } else {
        for gate in target_gated {
            writeln!(
                writer,
                "- {} => {}",
                gate.capability_id,
                gate.targets.join(",")
            )
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        }
    }
    writeln!(writer, "config_gated_capabilities:")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    if config_gated.is_empty() {
        writeln!(writer, "- (none)")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    } else {
        for gate in config_gated {
            let suffix = gate
                .targets
                .as_ref()
                .map(|targets| format!(" => {}", targets.join(",")))
                .unwrap_or_default();
            writeln!(
                writer,
                "- {}:{}{}",
                gate.capability_id, gate.config_key, suffix
            )
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        }
    }
    Ok(())
}

fn write_code_block<W: Write>(writer: &mut W, language: &str, contents: &str) -> Result<(), Error> {
    writeln!(writer, "```{language}")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    write!(writer, "{contents}").map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    if !contents.ends_with('\n') {
        writeln!(writer).map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    }
    writeln!(writer, "```").map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}
