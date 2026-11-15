use super::*;

pub(super) fn validate_closeout_prerequisites(
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
        LifecycleStage::Published => {}
        LifecycleStage::PublicationReady
            if agent_lifecycle::is_legacy_post_refresh_publication_ready_state(&lifecycle_state) => {}
        LifecycleStage::PublicationReady => {
            return Err(Error::Validation(format!(
                "close-proving-run rejects ordinary prepare-time `publication_ready` at `{}`; run `refresh-publication --approval {} --write` first",
                lifecycle_state_path,
                approval.relative_path
            )))
        }
        other => {
            return Err(Error::Validation(format!(
                "close-proving-run requires lifecycle stage `published`, or the explicit legacy post-refresh `publication_ready` compatibility shape, at `{}` (found `{}`)",
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
    if has_unresolved_placeholders(closeout) {
        return Err(Error::Validation(format!(
            "{} contains unresolved prepared-closeout placeholders in: {}",
            closeout_path.display(),
            unresolved_placeholder_fields(closeout).join(", ")
        )));
    }
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
    validate_capability_matrix_audit_green(workspace_root)?;
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
