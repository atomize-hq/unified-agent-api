//! Private reporting-only integration. The public builder/apply retain production refusal.
use super::super::support_audit::{RequiredUplift, REQUIRED_WRITES};
use super::*;
use crate::{
    depth_record::{DepthRecord, Event, RequestBinding},
    support_depth::{plan_freeze_with_work, FreezePlan, FreezeRequest},
};

pub(super) fn freeze(
    root: &Path,
    entry: &AgentRegistryEntry,
    args: &Args,
    audit: &mut SupportSurfaceAudit,
) -> Result<Option<FreezePlan>, Error> {
    if entry.support_depth.is_none()
        && !root
            .join(format!(
                "{}/reports/{}/depth-record.json",
                entry.manifest_root, args.target_version
            ))
            .exists()
    {
        return Ok(None);
    }
    let required = audit
        .required_uplifts_this_run
        .iter()
        .map(|row| {
            (
                row.surface_kind.clone(),
                row.command_path.clone(),
                row.surface_id.clone(),
            )
        })
        .collect::<Vec<_>>();
    let plan = plan_freeze_with_work(
        root,
        &FreezeRequest {
            agent: args.agent.clone(),
            version: args.target_version.clone(),
            event: Event::Maintenance {
                request_commit: args.request_commit.clone(),
                request_recorded_at: args.request_recorded_at.clone(),
                trigger_kind: "upstream_release_detected".into(),
                opened_from: args.opened_from.display().to_string(),
            },
        },
        &required,
    )
    .map_err(Error::Validation)?;
    if let Some(plan) = &plan {
        if plan.record.obligations.is_none() && !plan.carried_work.is_empty() {
            return Err(Error::Validation("cannot durably preserve prior targeted work in a P-only replacement; restore acquisition before preparing".into()));
        }
        if let Some(previous) = &plan.previous_record {
            let old = DepthRecord::parse(
                std::str::from_utf8(previous).map_err(|e| Error::Validation(e.to_string()))?,
                &args.agent,
                &args.target_version,
            )
            .map_err(Error::Validation)?;
            if plan.record.acceptance_entries != old.acceptance_entries
                || !plan.record.lacking_from(&old).is_empty()
            {
                return Err(Error::Validation(
                    "preparation must preserve entries and non-shrinking selection".into(),
                ));
            }
        }
        let old: Option<serde_json::Value> = plan
            .previous_request
            .as_deref()
            .map(|raw| {
                toml_edit::de::from_str(
                    std::str::from_utf8(raw).map_err(|e| Error::Validation(e.to_string()))?,
                )
                .map_err(|e| Error::Validation(e.to_string()))
            })
            .transpose()?;
        let retained = old
            .as_ref()
            .filter(|raw| raw["detected_release"]["target_version"] == args.target_version)
            .and_then(|raw| raw["support_surface_audit"]["required_uplifts_this_run"].as_array());
        for surface in &plan.required_surfaces {
            let prior = retained
                .into_iter()
                .flatten()
                .find(|row| {
                    row["surface_kind"] == surface.surface_kind
                        && row["command_path"] == surface.command_path
                        && row["surface_id"] == surface.surface_id
                })
                .map(|row| {
                    serde_json::from_value::<RequiredUplift>(row.clone())
                        .map_err(|e| Error::Validation(e.to_string()))
                })
                .transpose()?;
            let had_prior = prior.is_some();
            let row = prior.unwrap_or_else(|| RequiredUplift {
                surface_kind: surface.surface_kind.clone(),
                command_path: surface.command_path.clone(),
                surface_id: surface.surface_id.clone(),
                reason: "retained_required_work".into(),
                required_writes: REQUIRED_WRITES.iter().map(ToString::to_string).collect(),
            });
            if row.reason.trim().is_empty()
                || row.required_writes.is_empty()
                || row
                    .required_writes
                    .iter()
                    .any(|w| !REQUIRED_WRITES.contains(&w.as_str()))
            {
                return Err(Error::Validation("invalid retained required row".into()));
            }
            if let Some(current) = audit.required_uplifts_this_run.iter().find(|row| {
                row.surface_kind == surface.surface_kind
                    && row.command_path == surface.command_path
                    && row.surface_id == surface.surface_id
            }) {
                // A current live gap can coincide with retained due work. Preserve its actual
                // standing metadata, and refuse a contradictory standing row rather than drop it.
                if had_prior && current != &row {
                    return Err(Error::Validation(
                        "contradictory retained required row".into(),
                    ));
                }
            } else {
                audit.required_uplifts_this_run.push(row);
            }
        }
        audit
            .required_uplifts_this_run
            .sort_by_key(RequiredUplift::identity);
    }
    Ok(plan)
}

pub(super) fn append(
    root: &Path,
    registry: &AgentRegistry,
    plan: &FreezePlan,
    files: &mut Vec<PreparedFile>,
) -> Result<(), Error> {
    let selection = &plan.record.policy.selection;
    let entry = registry
        .find(&selection.agent_id)
        .ok_or_else(|| Error::Validation("missing depth agent".into()))?;
    let request_path = crate::agent_lifecycle::maintenance_request_path(&selection.agent_id);
    let proposed_request = files
        .iter()
        .find(|file| file.relative_path == request_path)
        .ok_or_else(|| Error::Validation("missing proposed request".into()))?;
    let value: serde_json::Value = toml_edit::de::from_str(
        std::str::from_utf8(&proposed_request.contents)
            .map_err(|e| Error::Validation(e.to_string()))?,
    )
    .map_err(|e| Error::Validation(e.to_string()))?;
    let text = |key: &str| {
        value[key]
            .as_str()
            .map(ToString::to_string)
            .ok_or_else(|| Error::Validation(format!("missing proposed request {key}")))
    };
    if value["artifact_version"] != "2"
        || value["agent_id"] != selection.agent_id
        || value["detected_release"]["target_version"] != selection.enrollment.version
    {
        return Err(Error::Validation(
            "proposed request does not bind the depth tuple".into(),
        ));
    }
    let table: request::SupportDepthBinding =
        serde_json::from_value(value["support_depth"].clone())
            .map_err(|e| Error::Validation(e.to_string()))?;
    let binding = RequestBinding {
        event: Event::Maintenance {
            request_commit: text("request_commit")?,
            request_recorded_at: text("request_recorded_at")?,
            trigger_kind: text("trigger_kind")?,
            opened_from: text("opened_from")?,
        },
        policy_identity: Some(table.policy_identity),
        obligations_identity: table.obligations_identity,
    };
    if let Some(reason) = plan.record.request_mismatch(&binding) {
        return Err(Error::Validation(reason.into()));
    }
    let record = format!(
        "{}\n",
        serde_json::to_string_pretty(&plan.record).map_err(|e| Error::Internal(e.to_string()))?
    );
    // Reparse the exact proposed serialization before it participates in publication.
    DepthRecord::parse(&record, &selection.agent_id, &selection.enrollment.version)
        .map_err(Error::Validation)?;
    files.push(PreparedFile {
        relative_path: format!(
            "{}/reports/{}/depth-record.json",
            entry.manifest_root, selection.enrollment.version
        ),
        contents: record.into_bytes(),
    });
    let bundle = crate::support_matrix::preview_preparation(root, registry, &plan.record)
        .map_err(Error::Validation)?;
    for (path, contents) in [
        (crate::support_matrix::JSON_OUTPUT_PATH, bundle.json),
        (crate::support_matrix::MARKDOWN_OUTPUT_PATH, bundle.markdown),
        (
            crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
            bundle.runtime_support_data,
        ),
    ] {
        files.push(PreparedFile {
            relative_path: path.into(),
            contents: contents.into_bytes(),
        });
    }
    let proposed = files
        .iter()
        .map(|file| {
            Ok((
                file.relative_path.clone(),
                String::from_utf8(file.contents.clone())
                    .map_err(|e| Error::Validation(e.to_string()))?,
            ))
        })
        .collect::<Result<_, Error>>()?;
    crate::depth_acceptance::check_reporting_preparation(root, registry, proposed)
        .map_err(Error::Validation)
}
