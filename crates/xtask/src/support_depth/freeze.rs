//! Read-only maintenance freeze planning. No caller can use a plan as write admission.
use super::{acquisition, check_current_policy, git_inputs::GitInputs, resolve_policy};
use crate::{
    agent_lifecycle::maintenance_request_path,
    agent_maintenance::support_audit::{SurfaceIdentity, NON_TUI_SUPPORT_DEBT_PATH},
    agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH},
    depth_record::{
        AcceptanceWork, DepthRecord, Event, RequestBinding, DEPTH_RECORD_SCHEMA_VERSION,
    },
};
use serde::Deserialize;
use std::{collections::BTreeSet, path::Path};

#[derive(Debug, Clone)]
pub struct FreezeRequest {
    pub agent: String,
    pub version: String,
    pub event: Event,
}

#[derive(Debug, Clone)]
pub struct FreezePlan {
    pub request: RequestBinding,
    pub record: DepthRecord,
    /// Expected-before bytes for the later guarded writer; this planner never applies them.
    pub previous_request: Option<Vec<u8>>,
    pub previous_record: Option<Vec<u8>>,
    /// Required name-level work is retained even when P-only cannot yet state target applicability.
    pub required_surfaces: BTreeSet<SurfaceIdentity>,
    /// Previous O's due work is retained across generations, including a P-only new generation.
    pub carried_work: Vec<AcceptanceWork>,
}

/// Isolated maintenance planner. None is a version with no declaration or retained record.
/// Events are preparation inputs, not actor authorization. No CLI or production loader calls this.
pub fn plan_freeze(
    workspace: &Path,
    request: &FreezeRequest,
) -> Result<Option<FreezePlan>, String> {
    plan(workspace, request).map_err(|e| {
        format!(
            "support depth freeze agent={} version={}: {e}",
            request.agent, request.version
        )
    })
}

fn plan(workspace: &Path, request: &FreezeRequest) -> Result<Option<FreezePlan>, String> {
    let version = semver::Version::parse(&request.version).map_err(|e| e.to_string())?;
    if !version.pre.is_empty()
        || !version.build.is_empty()
        || version.to_string() != request.version
    {
        return Err("version must be one exact MAJOR.MINOR.PATCH".into());
    }
    if !matches!(request.event, Event::Maintenance { .. }) {
        return Err("onboarding freeze planning needs its approval-artifact integration".into());
    }
    let mut view = GitInputs::new(workspace)?;
    let raw_registry = view.required(REGISTRY_RELATIVE_PATH)?;
    let registry =
        AgentRegistry::parse_unenforced(text(&raw_registry)?).map_err(|e| e.to_string())?;
    let entry = registry.find(&request.agent).ok_or("unknown agent")?;
    let root = &entry.manifest_root;
    let record_path = format!("{root}/reports/{}/depth-record.json", request.version);
    let previous_record = view.optional(&record_path)?;
    let prior = previous_record
        .as_deref()
        .map(|raw| DepthRecord::parse(text(raw)?, &request.agent, &request.version))
        .transpose()?;
    if let Some(prior) = &prior {
        bindings(prior)?;
    }
    let rules = view.required(&format!("{root}/RULES.json"))?;
    let inventory = view.required(NON_TUI_SUPPORT_DEBT_PATH)?;
    let Some(policy) = resolve_policy(
        text(&raw_registry)?,
        &request.agent,
        &request.version,
        text(&rules)?,
        previous_record.as_deref().map(text).transpose()?,
        text(&inventory)?,
    )?
    else {
        view.verify()?;
        return Ok(None);
    };
    let request_path = maintenance_request_path(&request.agent);
    let previous_request = view.optional(&request_path)?;
    let standing = previous_request
        .as_deref()
        .map(|raw| {
            toml_edit::de::from_str::<StandingRequest>(text(raw)?).map_err(|e| e.to_string())
        })
        .transpose()?;
    let mut required = BTreeSet::new();
    if let Some(standing) = &standing {
        if standing.agent_id != request.agent {
            return Err("standing request names another agent".into());
        }
        let standing_binding = standing.binding();
        if standing_binding.event == request.event
            && standing.detected_release.target_version != request.version
        {
            return Err("same Event cannot freeze another target version".into());
        }
        if standing.detected_release.target_version == request.version {
            required.extend(
                standing
                    .support_surface_audit
                    .required_uplifts_this_run
                    .iter()
                    .map(RequiredSurface::identity),
            );
        }
        if standing.support_depth.is_some() {
            let v = &standing.detected_release.target_version;
            let parsed = semver::Version::parse(v).map_err(|e| e.to_string())?;
            if parsed.to_string() != *v || !parsed.pre.is_empty() || !parsed.build.is_empty() {
                return Err("standing request has no exact version".into());
            }
            let bytes = view.required(&format!("{root}/reports/{v}/depth-record.json"))?;
            let record = DepthRecord::parse(text(&bytes)?, &request.agent, v)?;
            bindings(&record)?;
            if let Some(reason) = record.request_mismatch(&standing_binding) {
                return Err(reason.into());
            }
        }
    }
    let same_event = prior.as_ref().is_some_and(|p| p.event == request.event);
    if same_event {
        let standing = standing
            .as_ref()
            .ok_or("re-freeze has no standing request")?;
        if standing.detected_release.target_version != request.version {
            return Err("standing request version differs".into());
        }
        let prior = prior.as_ref().ok_or("missing prior record")?;
        if let Some(reason) = prior.request_mismatch(&standing.binding()) {
            return Err(reason.into());
        }
        check_current_policy(&prior.policy, &policy)?;
    }
    let carried_work = prior
        .as_ref()
        .and_then(|p| p.obligations.as_ref())
        .map(|o| o.acceptance_work.clone())
        .unwrap_or_default();
    required.extend(carried_work.iter().map(|w| w.surface.clone()));
    let obligations = acquisition::derive(
        &mut view,
        root,
        &rules,
        &policy,
        carried_work.clone(),
        &required,
    )?;
    if same_event
        && obligations.is_none()
        && prior.as_ref().is_some_and(|p| p.obligations.is_some())
    {
        return Err("re-freeze cannot lose acquired obligations/due work".into());
    }
    let execution = prior
        .as_ref()
        .filter(|p| same_event && p.obligations == obligations)
        .and_then(|p| p.execution.clone());
    let record = DepthRecord {
        schema_version: DEPTH_RECORD_SCHEMA_VERSION,
        event: request.event.clone(),
        policy,
        obligations,
        execution,
        acceptance_entries: prior.map(|p| p.acceptance_entries).unwrap_or_default(),
    };
    // The standard parser enforces form, canonical set ordering, identities and Event shape.
    let record = DepthRecord::parse(
        &serde_json::to_string(&record).map_err(|e| e.to_string())?,
        &request.agent,
        &request.version,
    )?;
    let binding = RequestBinding {
        event: request.event.clone(),
        policy_identity: Some(record.policy.identity.clone()),
        obligations_identity: record.obligations.as_ref().map(|o| o.identity.clone()),
    };
    if let Some(reason) = record.request_mismatch(&binding) {
        return Err(reason.into());
    }
    view.verify()?;
    Ok(Some(FreezePlan {
        request: binding,
        record,
        previous_request,
        previous_record,
        required_surfaces: required,
        carried_work,
    }))
}

fn bindings(record: &DepthRecord) -> Result<(), String> {
    if let Some(o) = &record.obligations {
        if o.policy_identity != record.policy.identity {
            return Err("prior O does not bind its P".into());
        }
    }
    if let Some(e) = &record.execution {
        if e.policy_identity != record.policy.identity
            || record.obligations.as_ref().map(|o| &o.identity) != Some(&e.obligations_identity)
        {
            return Err("prior E does not bind its P/O".into());
        }
    }
    Ok(())
}
fn text(bytes: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(bytes).map_err(|e| e.to_string())
}

// This is a binding/required-work projection for planning, not a validating execution loader.
// Both production loaders keep their refusal until command integration is reviewed separately.
#[derive(Deserialize)]
struct StandingRequest {
    agent_id: String,
    request_commit: String,
    request_recorded_at: String,
    trigger_kind: String,
    opened_from: String,
    detected_release: Release,
    support_depth: Option<FrozenTable>,
    #[serde(default)]
    support_surface_audit: RequiredAudit,
}
#[derive(Deserialize)]
struct Release {
    target_version: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenTable {
    policy_identity: String,
    obligations_identity: Option<String>,
}
#[derive(Default, Deserialize)]
struct RequiredAudit {
    required_uplifts_this_run: Vec<RequiredSurface>,
}
#[derive(Deserialize)]
struct RequiredSurface {
    surface_kind: String,
    command_path: String,
    surface_id: String,
}
impl RequiredSurface {
    fn identity(&self) -> SurfaceIdentity {
        SurfaceIdentity::new(
            self.surface_kind.clone(),
            self.command_path.clone(),
            self.surface_id.clone(),
        )
    }
}
impl StandingRequest {
    fn binding(&self) -> RequestBinding {
        RequestBinding {
            event: Event::Maintenance {
                request_commit: self.request_commit.clone(),
                request_recorded_at: self.request_recorded_at.clone(),
                trigger_kind: self.trigger_kind.clone(),
                opened_from: self.opened_from.clone(),
            },
            policy_identity: self
                .support_depth
                .as_ref()
                .map(|t| t.policy_identity.clone()),
            obligations_identity: self
                .support_depth
                .as_ref()
                .and_then(|t| t.obligations_identity.clone()),
        }
    }
}
