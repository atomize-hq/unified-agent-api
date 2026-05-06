use std::{fs, path::Path};

use sha2::{Digest, Sha256};

use crate::{agent_registry::AgentRegistryEntry, workspace_mutation::WorkspacePathJail};

use super::{
    raw::{RawDetectedRelease, RawExecutionContract},
    validate::{
        validate_existing_repo_relative_string_array, validate_non_empty_scalar,
        validate_non_empty_string_array, validate_repo_relative_glob_path,
        validate_repo_relative_reference, validate_repo_relative_string_array,
        validate_sha256_value,
    },
    DetectedRelease, ExecutionContract, ExecutionContractRecovery, MaintenanceAction,
    MaintenanceRequestError, TriggerKind, AUTOMATED_ARTIFACT_VERSION,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn validate_execution_contract(
    workspace_root: &Path,
    jail: &WorkspacePathJail,
    request_path: &Path,
    maintenance_root: &Path,
    registry_entry: &AgentRegistryEntry,
    trigger_kind: TriggerKind,
    detected_release: Option<&DetectedRelease>,
    raw: Option<RawExecutionContract>,
) -> Result<Option<ExecutionContract>, MaintenanceRequestError> {
    match (trigger_kind, raw) {
        (TriggerKind::UpstreamReleaseDetected, None) => Ok(None),
        (TriggerKind::UpstreamReleaseDetected, Some(raw_execution_contract)) => {
            let detected_release = detected_release.ok_or_else(|| {
                MaintenanceRequestError::Internal(format!(
                    "maintenance request `{}` is missing detected_release while validating execution_contract",
                    request_path.display()
                ))
            })?;

            validate_non_empty_scalar(
                request_path,
                "execution_contract.executor",
                &raw_execution_contract.executor,
            )?;
            if raw_execution_contract.executor != "codex" {
                return Err(MaintenanceRequestError::Validation(format!(
                    "maintenance request `{}` field `execution_contract.executor` must be `codex` in milestone 1",
                    request_path.display()
                )));
            }

            let expected_prompt_template_path =
                format!("{}/PR_BODY_TEMPLATE.md", registry_entry.manifest_root);
            validate_repo_relative_reference(
                jail,
                request_path,
                "execution_contract.prompt_template_path",
                &raw_execution_contract.prompt_template_path,
            )?;
            if raw_execution_contract.prompt_template_path != expected_prompt_template_path {
                return Err(MaintenanceRequestError::Validation(format!(
                    "maintenance request `{}` field `execution_contract.prompt_template_path` must be `{expected_prompt_template_path}` for agent `{}`",
                    request_path.display(),
                    registry_entry.agent_id
                )));
            }
            validate_sha256_value(
                request_path,
                "execution_contract.prompt_sha256",
                &raw_execution_contract.prompt_sha256,
            )?;
            let rendered_prompt = render_execution_prompt(
                workspace_root,
                &raw_execution_contract.prompt_template_path,
                &detected_release.target_version,
            )?;
            let rendered_prompt_sha256 = hex::encode(Sha256::digest(rendered_prompt.as_bytes()));
            if raw_execution_contract.prompt_sha256 != rendered_prompt_sha256 {
                return Err(MaintenanceRequestError::Validation(format!(
                    "maintenance request `{}` field `execution_contract.prompt_sha256` must match the rendered prompt template digest `{rendered_prompt_sha256}`",
                    request_path.display()
                )));
            }

            let expected_pr_summary_path =
                format!("{}/governance/pr-summary.md", maintenance_root.display());
            validate_repo_relative_glob_path(
                request_path,
                "execution_contract.pr_summary_path",
                &raw_execution_contract.pr_summary_path,
            )?;
            if raw_execution_contract.pr_summary_path != expected_pr_summary_path {
                return Err(MaintenanceRequestError::Validation(format!(
                    "maintenance request `{}` field `execution_contract.pr_summary_path` must be `{expected_pr_summary_path}` under the same maintenance root",
                    request_path.display()
                )));
            }

            let expected_closeout_path = format!(
                "{}/governance/maintenance-closeout.json",
                maintenance_root.display()
            );
            validate_repo_relative_glob_path(
                request_path,
                "execution_contract.closeout_path",
                &raw_execution_contract.closeout_path,
            )?;
            if raw_execution_contract.closeout_path != expected_closeout_path {
                return Err(MaintenanceRequestError::Validation(format!(
                    "maintenance request `{}` field `execution_contract.closeout_path` must be `{expected_closeout_path}` under the same maintenance root",
                    request_path.display()
                )));
            }

            if !raw_execution_contract.requires_manual_closeout {
                return Err(MaintenanceRequestError::Validation(format!(
                    "maintenance request `{}` field `execution_contract.requires_manual_closeout` must be `true` in milestone 1",
                    request_path.display()
                )));
            }

            let writable_surfaces = validate_repo_relative_string_array(
                request_path,
                "execution_contract.writable_surfaces",
                &raw_execution_contract.writable_surfaces,
                true,
            )?;
            let read_only_inputs = validate_existing_repo_relative_string_array(
                jail,
                request_path,
                "execution_contract.read_only_inputs",
                &raw_execution_contract.read_only_inputs,
            )?;
            let ordered_commands = validate_non_empty_string_array(
                request_path,
                "execution_contract.ordered_commands",
                &raw_execution_contract.ordered_commands,
                true,
            )?;
            let green_gates = validate_non_empty_string_array(
                request_path,
                "execution_contract.green_gates",
                &raw_execution_contract.green_gates,
                true,
            )?;

            validate_non_empty_scalar(
                request_path,
                "execution_contract.recovery.recreate_packet_command",
                &raw_execution_contract.recovery.recreate_packet_command,
            )?;
            validate_repo_relative_glob_path(
                request_path,
                "execution_contract.recovery.reopen_pr_body_path",
                &raw_execution_contract.recovery.reopen_pr_body_path,
            )?;
            if raw_execution_contract.recovery.reopen_pr_body_path != expected_pr_summary_path {
                return Err(MaintenanceRequestError::Validation(format!(
                    "maintenance request `{}` field `execution_contract.recovery.reopen_pr_body_path` must match `execution_contract.pr_summary_path` `{expected_pr_summary_path}`",
                    request_path.display()
                )));
            }
            validate_non_empty_scalar(
                request_path,
                "execution_contract.recovery.reopen_pr_branch",
                &raw_execution_contract.recovery.reopen_pr_branch,
            )?;
            if raw_execution_contract.recovery.reopen_pr_branch != detected_release.branch_name {
                return Err(MaintenanceRequestError::Validation(format!(
                    "maintenance request `{}` field `execution_contract.recovery.reopen_pr_branch` must match `detected_release.branch_name` `{}`",
                    request_path.display(),
                    detected_release.branch_name
                )));
            }
            let recovery_notes = validate_non_empty_string_array(
                request_path,
                "execution_contract.recovery.notes",
                &raw_execution_contract.recovery.notes,
                true,
            )?;

            Ok(Some(ExecutionContract {
                executor: raw_execution_contract.executor,
                prompt_template_path: raw_execution_contract.prompt_template_path,
                prompt_sha256: raw_execution_contract.prompt_sha256,
                pr_summary_path: raw_execution_contract.pr_summary_path,
                closeout_path: raw_execution_contract.closeout_path,
                requires_manual_closeout: raw_execution_contract.requires_manual_closeout,
                writable_surfaces,
                read_only_inputs,
                ordered_commands,
                green_gates,
                recovery: ExecutionContractRecovery {
                    recreate_packet_command: raw_execution_contract
                        .recovery
                        .recreate_packet_command,
                    reopen_pr_body_path: raw_execution_contract.recovery.reopen_pr_body_path,
                    reopen_pr_branch: raw_execution_contract.recovery.reopen_pr_branch,
                    notes: recovery_notes,
                },
            }))
        }
        (_, Some(_)) => Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` may only include `[execution_contract]` when `trigger_kind = \"upstream_release_detected\"`",
            request_path.display()
        ))),
        (_, None) => Ok(None),
    }
}

pub(super) fn validate_detected_release(
    request_path: &Path,
    trigger_kind: TriggerKind,
    raw: Option<RawDetectedRelease>,
) -> Result<Option<DetectedRelease>, MaintenanceRequestError> {
    match (trigger_kind, raw) {
        (TriggerKind::UpstreamReleaseDetected, Some(raw)) => {
            validate_non_empty_scalar(request_path, "detected_release.detected_by", &raw.detected_by)?;
            validate_non_empty_scalar(
                request_path,
                "detected_release.current_validated",
                &raw.current_validated,
            )?;
            validate_non_empty_scalar(
                request_path,
                "detected_release.target_version",
                &raw.target_version,
            )?;
            validate_non_empty_scalar(
                request_path,
                "detected_release.latest_stable",
                &raw.latest_stable,
            )?;
            validate_non_empty_scalar(
                request_path,
                "detected_release.version_policy",
                &raw.version_policy,
            )?;
            validate_non_empty_scalar(
                request_path,
                "detected_release.source_kind",
                &raw.source_kind,
            )?;
            validate_non_empty_scalar(request_path, "detected_release.source_ref", &raw.source_ref)?;
            validate_non_empty_scalar(
                request_path,
                "detected_release.dispatch_kind",
                &raw.dispatch_kind,
            )?;
            validate_non_empty_scalar(
                request_path,
                "detected_release.dispatch_workflow",
                &raw.dispatch_workflow,
            )?;
            validate_non_empty_scalar(
                request_path,
                "detected_release.branch_name",
                &raw.branch_name,
            )?;
            Ok(Some(DetectedRelease {
                detected_by: raw.detected_by,
                current_validated: raw.current_validated,
                target_version: raw.target_version,
                latest_stable: raw.latest_stable,
                version_policy: raw.version_policy,
                source_kind: raw.source_kind,
                source_ref: raw.source_ref,
                dispatch_kind: raw.dispatch_kind,
                dispatch_workflow: raw.dispatch_workflow,
                branch_name: raw.branch_name,
            }))
        }
        (TriggerKind::UpstreamReleaseDetected, None) => Err(MaintenanceRequestError::Validation(
            format!(
                "maintenance request `{}` trigger_kind `upstream_release_detected` requires a `[detected_release]` table",
                request_path.display()
            ),
        )),
        (_, Some(_)) => Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` may only include `[detected_release]` when `trigger_kind = \"upstream_release_detected\"`",
            request_path.display()
        ))),
        (_, None) => Ok(None),
    }
}

pub(super) fn validate_automated_watch_request(
    request_path: &Path,
    artifact_version: &str,
    trigger_kind: TriggerKind,
    requested_control_plane_actions: &[MaintenanceAction],
) -> Result<(), MaintenanceRequestError> {
    if trigger_kind != TriggerKind::UpstreamReleaseDetected {
        return Ok(());
    }
    if artifact_version != AUTOMATED_ARTIFACT_VERSION {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` trigger_kind `upstream_release_detected` requires `artifact_version = \"{AUTOMATED_ARTIFACT_VERSION}\"`",
            request_path.display()
        )));
    }
    if requested_control_plane_actions != [MaintenanceAction::PacketDocRefresh] {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` trigger_kind `upstream_release_detected` requires `requested_control_plane_actions = [\"packet_doc_refresh\"]`",
            request_path.display()
        )));
    }
    Ok(())
}

fn render_execution_prompt(
    workspace_root: &Path,
    prompt_template_path: &str,
    target_version: &str,
) -> Result<String, MaintenanceRequestError> {
    let prompt_template =
        fs::read_to_string(workspace_root.join(prompt_template_path)).map_err(|err| {
            MaintenanceRequestError::Validation(format!(
                "read execution contract prompt template `{prompt_template_path}`: {err}"
            ))
        })?;
    Ok(prompt_template.replace("{{VERSION}}", target_version))
}
