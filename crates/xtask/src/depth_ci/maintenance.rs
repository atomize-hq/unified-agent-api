//! Canonical MA2 input evidence. Checkpoint authority belongs to protected IN1 and its join.
use super::profile::{GenerationBinding, Profile};
use crate::{
    agent_maintenance::{request::SupportDepthBinding, support_audit::SurfaceIdentity},
    depth_record::{DepthRecord, Event, RequestBinding},
    support_depth::acquisition_inputs::Inputs,
};
use serde::Deserialize;
use std::collections::BTreeSet;

pub(crate) fn resolve(
    inputs: &mut impl Inputs,
    p: Profile,
    target: Option<&DepthRecord>,
    associated: bool,
) -> Result<Option<GenerationBinding>, String> {
    let request_path = crate::agent_lifecycle::maintenance_request_path(p.agent_id());
    // Capture both reads before parsing. An unavailable read never means absent association.
    let request = inputs.optional(&request_path)?;
    let report = inputs.optional(super::onboarding::RECORD)?;
    let request_hint = request
        .as_deref()
        .map(|raw| {
            toml_edit::de::from_str::<serde_json::Value>(text(raw)?).map_err(|e| e.to_string())
        })
        .transpose();
    let report_hint = report
        .as_deref()
        .map(|raw| serde_json::from_slice::<serde_json::Value>(raw).map_err(|e| e.to_string()))
        .transpose();
    let request = request
        .as_deref()
        .map(|raw| toml_edit::de::from_str::<Request>(text(raw)?).map_err(|e| e.to_string()))
        .transpose();
    let report = report
        .as_deref()
        .map(|raw| {
            let raw = text(raw)?;
            DepthRecord::parse(raw, p.agent_id(), p.version())
        })
        .transpose();
    let relevant = target.map_or(true, |target| {
        let request_event = request_hint
            .as_ref()
            .ok()
            .and_then(Option::as_ref)
            .and_then(|r| {
                serde_json::from_value::<Event>(serde_json::json!({
                "lifecycle_path":"maintenance", "request_commit":r["request_commit"],
                "request_recorded_at":r["request_recorded_at"], "trigger_kind":r["trigger_kind"],
                "opened_from":r["opened_from"],
            })).ok()
            });
        let report_event = report_hint
            .as_ref()
            .ok()
            .and_then(Option::as_ref)
            .and_then(|r| serde_json::from_value::<Event>(r["event"].clone()).ok());
        associated
            || request_event.as_ref() == Some(&target.event)
            || report_event.as_ref() == Some(&target.event)
    });
    if !relevant {
        return Ok(None);
    }
    let request = request
        .map_err(|e| format!("invalid maintenance generation request: {e}"))?
        .ok_or("maintenance generation request missing")?;
    let mut record = report
        .map_err(|e| format!("invalid maintenance generation report: {e}"))?
        .ok_or("maintenance generation report missing")?;
    if request.artifact_version != "2"
        || request.agent_id != p.agent_id()
        || request.detected_release.target_version != p.version()
        || record.policy.selection.enrollment.lifecycle_path
            != crate::agent_registry::support_depth::LifecyclePath::Maintenance
    {
        return Err("maintenance generation version, agent or enrollment mismatch".into());
    }
    if let Some(reason) = record.request_mismatch(&RequestBinding {
        event: request.event(),
        policy_identity: Some(request.support_depth.policy_identity.clone()),
        obligations_identity: request.support_depth.obligations_identity.clone(),
    }) {
        return Err(format!(
            "maintenance generation request/report mismatch: {reason}"
        ));
    }
    let different_generation = target.is_some_and(|t| {
        t.event != record.event || t.policy != record.policy || t.obligations != record.obligations
    });
    if different_generation && associated {
        return Err("associated head replaced maintenance generation".into());
    }
    // Reject duplicate generation claims across versioned reports, independently of entry order.
    for version in inputs.list("cli_manifests/depth_fixture/reports")? {
        if version == p.version() {
            continue;
        }
        if let Some(raw) = inputs.optional(&format!(
            "cli_manifests/depth_fixture/reports/{version}/depth-record.json"
        ))? {
            let other = DepthRecord::parse(text(&raw)?, p.agent_id(), &version)?;
            if other.event == record.event {
                return Err("ambiguous maintenance generation reports".into());
            }
        }
    }
    for input in &record
        .obligations
        .as_ref()
        .ok_or("maintenance generation missing frozen O")?
        .acquired_inputs
    {
        if inputs.reference(&input.path)? != *input {
            return Err("maintenance generation has stale acquired input".into());
        }
    }
    if request.trigger_kind == "isolated_synthetic_wrapper" {
        // The original finite machinery profile has a compiled P/O, with no CLI acquisition.
        let expected = p.record(include_str!(
            "../../tests/fixtures/depth-synthetic/record.json"
        ))?;
        if record.policy != expected.policy || record.obligations != expected.obligations {
            return Err("maintenance generation differs from compiled machinery P/O".into());
        }
    } else {
        let required = request
            .support_surface_audit
            .required_uplifts_this_run
            .into_iter()
            .map(|r| SurfaceIdentity::new(r.surface_kind, r.command_path, r.surface_id))
            .collect::<BTreeSet<_>>();
        crate::support_depth::generation::check(inputs, &record, &required)?;
    }
    if different_generation {
        return Ok(None);
    }
    record.execution = None;
    record.acceptance_entries.clear();
    Ok(Some(GenerationBinding { record }))
}
fn text(bytes: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(bytes).map_err(|e| e.to_string())
}
#[derive(Deserialize)]
struct Request {
    artifact_version: String,
    agent_id: String,
    request_commit: String,
    request_recorded_at: String,
    trigger_kind: String,
    opened_from: String,
    detected_release: Release,
    support_depth: SupportDepthBinding,
    #[serde(default)]
    support_surface_audit: Audit,
}
#[derive(Deserialize)]
struct Release {
    target_version: String,
}
#[derive(Default, Deserialize)]
struct Audit {
    #[serde(default)]
    required_uplifts_this_run: Vec<Surface>,
}
#[derive(Deserialize)]
struct Surface {
    surface_kind: String,
    command_path: String,
    surface_id: String,
}
impl Request {
    fn event(&self) -> Event {
        Event::Maintenance {
            request_commit: self.request_commit.clone(),
            request_recorded_at: self.request_recorded_at.clone(),
            trigger_kind: self.trigger_kind.clone(),
            opened_from: self.opened_from.clone(),
        }
    }
}
