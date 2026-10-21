use super::{
    effect,
    input::{text, View},
    Effect,
};
use crate::{
    agent_lifecycle::{self, LifecycleState},
    depth_record::{AcceptanceEntry, Event},
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Maintenance,
    Proving,
    Publication,
}
struct Artifact {
    path: String,
    digest: String,
    value: Value,
    binding: String,
    binding_digest: String,
    kind: Kind,
}
pub(super) fn digest(raw: &str) -> String {
    hex::encode(Sha256::digest(raw.as_bytes()))
}
fn semantic(v: &Value) -> Value {
    match v {
        Value::Array(items) => {
            let mut items: Vec<_> = items.iter().map(semantic).collect();
            items.sort_by_key(Value::to_string);
            Value::Array(items)
        }
        Value::Object(items) => Value::Object(
            items
                .iter()
                .map(|(k, v)| (k.clone(), semantic(v)))
                .collect(),
        ),
        _ => v.clone(),
    }
}
fn maintenance(view: &View) -> String {
    format!(
        "docs/agents/lifecycle/{}-maintenance/governance/maintenance-closeout.json",
        view.entry.agent_id
    )
}
fn governance(view: &View) -> String {
    format!(
        "docs/agents/lifecycle/{}/governance/",
        view.entry.scaffold.onboarding_pack_prefix
    )
}
fn artifact(view: &View, path: &str, kind: Kind) -> Result<Option<Artifact>, String> {
    let prefix = if kind == Kind::Maintenance {
        format!(
            "docs/agents/lifecycle/{}-maintenance/governance/",
            view.entry.agent_id
        )
    } else {
        governance(view)
    };
    if !path.starts_with(&prefix) {
        return Err(format!("{path}: cross-agent artifact path"));
    }
    let Some(raw) = view.source.optional(path)? else {
        return Ok(None);
    };
    let value = super::input::json(&raw)?;
    let (binding, hash) = match kind {
        Kind::Maintenance => {
            crate::agent_maintenance::closeout::check_closeout_form(&raw)?;
            ("request_ref", "request_sha256")
        }
        Kind::Proving => {
            crate::proving_run_closeout::check_closeout_form(&raw)?;
            if !["prepared", "closed"].contains(&text(&value, "state")?) {
                return Err("unsupported proving closeout state".into());
            }
            ("approval_ref", "approval_sha256")
        }
        Kind::Publication => {
            let packet: agent_lifecycle::PublicationReadyPacket =
                serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
            packet.validate().map_err(|e| e.to_string())?;
            if packet.agent_id != view.entry.agent_id
                || packet.manifest_root != view.entry.manifest_root
            {
                return Err("publication packet belongs to another agent/root".into());
            }
            ("approval_artifact_path", "approval_artifact_sha256")
        }
    };
    let binding = text(&value, binding)?.to_owned();
    let binding_digest = text(&value, hash)?.to_owned();
    if binding_digest.len() != 64
        || !binding_digest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("invalid artifact binding digest".into());
    }
    Ok(Some(Artifact {
        path: path.into(),
        digest: digest(&raw),
        value,
        binding,
        binding_digest,
        kind,
    }))
}
fn owner(view: &View, artifact: &Artifact) -> Result<Option<String>, String> {
    let mut owners = BTreeSet::new();
    for (version, record) in &view.records {
        for entry in &record.acceptance_entries {
            let reference = match (artifact.kind, entry) {
                (Kind::Maintenance, AcceptanceEntry::MaintenanceCloseout { closeout, .. })
                | (Kind::Proving, AcceptanceEntry::ProvingRunCloseout { closeout, .. }) => {
                    Some(closeout)
                }
                (Kind::Publication, AcceptanceEntry::Publication { packet, .. }) => Some(packet),
                _ => None,
            };
            if reference.is_some_and(|r| r.path == artifact.path && r.sha256 == artifact.digest) {
                owners.insert(version.clone());
            }
        }
    }
    let binding_prefix = if artifact.kind == Kind::Maintenance {
        format!(
            "docs/agents/lifecycle/{}-maintenance/governance/",
            view.entry.agent_id
        )
    } else {
        governance(view)
    };
    if !artifact.binding.starts_with(&binding_prefix) {
        return Err(format!("{}: cross-agent binding", artifact.path));
    }
    let live = match view.source.optional(&artifact.binding)? {
        Some(raw) if digest(&raw) == artifact.binding_digest => {
            Some(binding_owner(view, artifact, &raw)?)
        }
        _ => None,
    };
    if owners.len() > 1 {
        return Err(format!("{}: conflicting durable owners", artifact.path));
    }
    if let Some(live) = live {
        if !owners.is_empty() && (live.is_none() || owners.iter().any(|v| Some(v) != live.as_ref()))
        {
            return Err(format!(
                "{}: binding/durable ownership conflict",
                artifact.path
            ));
        }
        return Ok(live);
    }
    owners.into_iter().next().map(Some).ok_or_else(|| {
        format!(
            "{}: generation scope unresolved (no exact binding or durable reference)",
            artifact.path
        )
    })
}
fn binding_owner(view: &View, artifact: &Artifact, raw: &str) -> Result<Option<String>, String> {
    if artifact.kind != Kind::Maintenance {
        let approval = crate::approval_artifact::parse_approval_attribution(raw, &artifact.binding)
            .map_err(|e| e.to_string())?;
        if approval.agent_id != view.entry.agent_id
            || approval.manifest_root != view.entry.manifest_root
            || approval.onboarding_pack_prefix != view.entry.scaffold.onboarding_pack_prefix
        {
            return Err("approval belongs to another agent/root".into());
        }
        return Ok(approval.support_depth_version);
    }
    let request: Value = toml_edit::de::from_str(raw).map_err(|e| e.to_string())?;
    if text(&request, "agent_id")? != view.entry.agent_id {
        return Err("request belongs to another agent".into());
    }
    let version = request
        .pointer("/detected_release/target_version")
        .and_then(Value::as_str)
        .filter(|v| semver::Version::parse(v).is_ok())
        .ok_or("request has no exact target version")?;
    let event = Event::Maintenance {
        request_commit: text(&request, "request_commit")?.into(),
        request_recorded_at: text(&request, "request_recorded_at")?.into(),
        trigger_kind: text(&request, "trigger_kind")?.into(),
        opened_from: text(&request, "opened_from")?.into(),
    };
    if view.records.get(version).is_some_and(|r| r.event != event) {
        return Err("request Event contradicts recorded generation".into());
    }
    Ok(Some(version.into()))
}
fn lifecycle(view: &View) -> Result<Option<LifecycleState>, String> {
    view.optional_json(&agent_lifecycle::lifecycle_state_path_for_entry(
        &view.entry,
    ))?
    .map(|v| {
        let state: LifecycleState = serde_json::from_value(v).map_err(|e| e.to_string())?;
        state.validate().map_err(|e| e.to_string())?;
        if state.agent_id != view.entry.agent_id
            || state.onboarding_pack_prefix != view.entry.scaffold.onboarding_pack_prefix
        {
            return Err("lifecycle identity mismatch".into());
        }
        Ok(state)
    })
    .transpose()
}
fn assertion(
    view: &View,
    state: Option<&LifecycleState>,
    field: &str,
) -> Result<Option<Artifact>, String> {
    let Some(state) = state else { return Ok(None) };
    match field {
        "published" => state
            .publication_packet_path
            .as_ref()
            .map(|path| {
                let a =
                    artifact(view, path, Kind::Publication)?.ok_or("reported packet missing")?;
                if state.publication_packet_sha256.as_ref() != Some(&a.digest) {
                    return Err("reported packet hash disagrees with bytes".into());
                }
                Ok(a)
            })
            .transpose(),
        "closed_baseline" | "proving_run_closeout_written" => state
            .closeout_baseline_path
            .as_ref()
            .map(|path| {
                artifact(view, path, Kind::Proving)?
                    .ok_or_else(|| "reported proving closeout missing".into())
            })
            .transpose(),
        _ => artifact(view, &maintenance(view), Kind::Maintenance),
    }
}
fn asserted(state: Option<&LifecycleState>, field: &str) -> bool {
    use agent_lifecycle::{EvidenceId as E, LifecycleStage as S, SideState};
    state.is_some_and(|s| match field {
        "published" => s.lifecycle_stage == S::Published,
        "closed_baseline" => s.lifecycle_stage == S::ClosedBaseline,
        "proving_run_closeout_written" => {
            s.satisfied_evidence.contains(&E::ProvingRunCloseoutWritten)
        }
        "maintenance_closeout_written" => s
            .satisfied_evidence
            .contains(&E::MaintenanceCloseoutWritten),
        _ => !s.side_states.contains(&SideState::Drifted),
    })
}
fn charge(
    view: &View,
    a: &Artifact,
    effects: &mut BTreeSet<Effect>,
    kind: &str,
) -> Result<(), String> {
    if let Some(version) = owner(view, a)? {
        effect(effects, &version, None, &a.path, kind);
    }
    Ok(())
}

pub(super) fn detect(
    before: &View,
    after: &View,
    effects: &mut BTreeSet<Effect>,
) -> Result<(), String> {
    for (kind, path, old_path) in [
        (Kind::Maintenance, maintenance(after), maintenance(before)),
        (
            Kind::Proving,
            format!("{}proving-run-closeout.json", governance(after)),
            format!("{}proving-run-closeout.json", governance(before)),
        ),
    ] {
        let old = artifact(before, &old_path, kind)?;
        let now = artifact(after, &path, kind)?;
        if let Some(now) = now {
            let changed = old.as_ref().map(|a| semantic(&a.value)) != Some(semantic(&now.value));
            if changed && (kind == Kind::Maintenance || now.value["state"] == "closed") {
                charge(after, &now, effects, "closeout semantic change")?;
            }
        }
    }
    let old = lifecycle(before)?;
    let now = lifecycle(after)?;
    for field in [
        "published",
        "closed_baseline",
        "proving_run_closeout_written",
        "maintenance_closeout_written",
        "absence_of_drifted",
    ] {
        let previously = asserted(old.as_ref(), field);
        let current = asserted(now.as_ref(), field);
        let was_drifted = old
            .as_ref()
            .is_some_and(|s| s.side_states.contains(&agent_lifecycle::SideState::Drifted));
        let drift_cleared = field == "absence_of_drifted"
            && was_drifted
            && !now
                .as_ref()
                .is_some_and(|s| s.side_states.contains(&agent_lifecycle::SideState::Drifted));
        if !current && !drift_cleared {
            continue;
        }
        let candidate = assertion(after, now.as_ref(), field)?;
        // Removing the lifecycle file cannot evade D3: its canonical candidate closeout is required.
        let candidate = if drift_cleared && candidate.is_none() {
            artifact(after, &maintenance(after), Kind::Maintenance)?
        } else {
            candidate
        };
        let previous = if previously {
            assertion(before, old.as_ref(), field)?
        } else {
            None
        };
        let retarget = previously
            && current
            && candidate.as_ref().is_some_and(|a| {
                previous.as_ref().map(|p| (&p.path, &p.digest)) != Some((&a.path, &a.digest))
            });
        if drift_cleared || (field != "absence_of_drifted" && !previously) || retarget {
            if let Some(candidate) = candidate {
                charge(after, &candidate, effects, field)?;
            } else if field != "absence_of_drifted" || drift_cleared {
                return Err(format!(
                    "{field}: generation scope unresolved; reported artifact absent"
                ));
            }
        }
    }
    Ok(())
}
