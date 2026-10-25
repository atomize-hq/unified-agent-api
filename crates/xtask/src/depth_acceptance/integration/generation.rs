//! Reporting-only changes still obey the request and statement-freeze invariants.
use super::super::input::View;
use crate::{
    agent_lifecycle, approval_artifact,
    depth_record::{request::StandingRequest, Event, RequestBinding},
};
use std::path::Path;

pub(super) fn check(root: &Path, old: &View, new: &View) -> Result<(), String> {
    let path = agent_lifecycle::maintenance_request_path(&new.entry.agent_id);
    let request = new
        .source
        .optional(&path)?
        .map(|raw| toml_edit::de::from_str::<StandingRequest>(&raw).map_err(|e| e.to_string()))
        .transpose()?;
    if let Some(request) = &request {
        let version = request
            .detected_release
            .as_ref()
            .and_then(|d| d.target_version.as_ref());
        let table = request.support_depth.as_ref();
        if let Some(record) = version.and_then(|v| new.records.get(v)) {
            let binding = RequestBinding {
                event: Event::Maintenance {
                    request_commit: request.request_commit.clone(),
                    request_recorded_at: request.request_recorded_at.clone(),
                    trigger_kind: request.trigger_kind.clone(),
                    opened_from: request.opened_from.clone(),
                },
                policy_identity: table.and_then(|t| t.policy_identity.clone()),
                obligations_identity: table.and_then(|t| t.obligations_identity.clone()),
            };
            if let Some(why) = record.request_mismatch(&binding) {
                return Err(format!("{why} (request generation)"));
            }
        } else if table.is_some() {
            return Err("request froze P without its depth record".into());
        }
    }
    for (version, record) in &new.records {
        if old
            .records
            .get(version)
            .map(|prior| prior.stated_sha256())
            .transpose()?
            == Some(record.stated_sha256()?)
        {
            // Entries alone change no statement; old working files may be replaced or absent.
            continue;
        }
        let frozen = match &record.event {
            Event::Maintenance { .. } => request.as_ref().is_some_and(|r| {
                r.detected_release
                    .as_ref()
                    .and_then(|d| d.target_version.as_ref())
                    == Some(version)
                    && r.support_depth
                        .as_ref()
                        .and_then(|t| t.policy_identity.as_ref())
                        .is_some_and(|p| !p.is_empty())
            }),
            Event::Onboarding { .. } => {
                let path = agent_lifecycle::approval_artifact_path_for_entry(&new.entry);
                if new.source.optional(&path)?.is_none() {
                    false
                } else {
                    let approval = approval_artifact::load_approval_artifact(root, &path)
                        .map_err(|e| e.to_string())?;
                    approval.descriptor.agent_id == new.entry.agent_id
                        && approval.descriptor.manifest_root == new.entry.manifest_root
                        && approval.support_depth_version.as_ref() == Some(version)
                }
            }
        };
        if !frozen {
            return Err(
                "new or changed statement lacks its own version and lifecycle freeze".into(),
            );
        }
    }
    Ok(())
}
