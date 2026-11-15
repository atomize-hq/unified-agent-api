//! One settlement implementation for the ordinary writer and onboarding predicates.
use super::*;

pub(crate) fn settle_maintenance_readiness(
    entry: &AgentRegistryEntry,
    maintenance: &crate::approval_artifact::ApprovalMaintenance,
    closeout_path: &Path,
) -> Result<MaintenanceSettlement, Error> {
    match &maintenance.mode {
        ApprovalMaintenanceMode::ReleaseWatchEnrolled {
            release_watch_sha256,
            ..
        } => {
            let registry_release_watch = entry.maintenance.release_watch.as_ref().ok_or_else(|| {
                Error::Validation(format!(
                    "{}: approval maintenance mode `release_watch_enrolled` requires registry `maintenance.release_watch` for `{}`",
                    closeout_path.display(),
                    entry.agent_id
                ))
            })?;
            let registry_release_watch_sha256 =
                normalized_release_watch_sha256(registry_release_watch).map_err(|err| {
                    Error::Validation(format!(
                        "{}: registry maintenance.release_watch is invalid for `{}`: {err}",
                        closeout_path.display(),
                        entry.agent_id
                    ))
                })?;
            if registry_release_watch_sha256 != *release_watch_sha256 {
                return Err(Error::Validation(format!(
                    "{}: approval and registry maintenance.release_watch truth diverge for `{}`",
                    closeout_path.display(),
                    entry.agent_id
                )));
            }
            Ok(MaintenanceSettlement {
                mode: MaintenanceSettlementMode::ReleaseWatchEnrolled,
                approval_section_sha256: maintenance.section_sha256.clone(),
                release_watch_sha256: Some(registry_release_watch_sha256),
                deferral_sha256: None,
            })
        }
        ApprovalMaintenanceMode::ExplicitlyDeferred {
            deferral_sha256, ..
        } => {
            if entry.maintenance.release_watch.is_some() {
                return Err(Error::Validation(format!(
                    "{}: approval maintenance mode `explicitly_deferred` forbids registry `maintenance.release_watch` for `{}`",
                    closeout_path.display(),
                    entry.agent_id
                )));
            }
            Ok(MaintenanceSettlement {
                mode: MaintenanceSettlementMode::ExplicitlyDeferred,
                approval_section_sha256: maintenance.section_sha256.clone(),
                release_watch_sha256: None,
                deferral_sha256: Some(deferral_sha256.clone()),
            })
        }
    }
}
