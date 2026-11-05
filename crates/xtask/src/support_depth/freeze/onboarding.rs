//! Read-only create-lane call site for the shared policy resolver and O derivation.
use super::*;
use crate::{
    agent_registry::support_depth::LifecyclePath,
    depth_record::onboarding::{onboarding_binding, LifecycleDepth, OnboardingApproval},
};
use std::collections::BTreeMap;

impl FreezePlan {
    /// Planning is not write admission. The private controller authenticates the installed route,
    /// checks charter rule 4 and acquisition provenance, then verifies this same input view.
    pub(crate) fn onboarding(
        view: &mut GitInputs,
        approval: &OnboardingApproval,
        object: Option<&LifecycleDepth>,
    ) -> Result<Self, String> {
        let version = approval
            .support_depth_version
            .as_deref()
            .ok_or("charter rule 3 unresolved: approval states no depth version")?;
        let raw_registry = view.required(REGISTRY_RELATIVE_PATH)?;
        let registry =
            AgentRegistry::parse_unenforced(text(&raw_registry)?).map_err(|e| e.to_string())?;
        let entry = registry
            .find(&approval.agent_id)
            .ok_or("unknown approval agent")?;
        if let Some(why) = approval.attribution_mismatch(
            &entry.agent_id,
            &entry.manifest_root,
            &entry.scaffold.onboarding_pack_prefix,
        ) {
            return Err(why);
        }
        let declared = entry.support_depth.as_ref().is_some_and(|table| {
            table
                .enabled_paths
                .as_ref()
                .is_some_and(|p| p.contains(&LifecyclePath::Onboarding))
                && table.enrollments.as_ref().is_some_and(|rows| {
                    rows.iter().any(|row| {
                        row.version == version && row.lifecycle_path == LifecyclePath::Onboarding
                    })
                })
        });
        if !declared {
            return Err("charter rule 3 unresolved: no enabled onboarding enrollment for the approved version".into());
        }
        let root = &entry.manifest_root;
        let record_path = format!("{root}/reports/{version}/depth-record.json");
        let previous_record = view.optional(&record_path)?;
        let prior = previous_record
            .as_deref()
            .map(|raw| DepthRecord::parse(text(raw)?, &approval.agent_id, version))
            .transpose()?;
        let records = prior
            .clone()
            .map(|p| BTreeMap::from([(version.to_owned(), p)]))
            .unwrap_or_default();
        onboarding_binding(object, approval, &records).map_err(|faults| {
            faults
                .into_iter()
                .map(|f| f.message)
                .collect::<Vec<_>>()
                .join("; ")
        })?;
        if let Some(prior) = &prior {
            bindings(prior)?;
        }
        let rules = view.required(&format!("{root}/RULES.json"))?;
        let inventory = view.required(NON_TUI_SUPPORT_DEBT_PATH)?;
        let policy = resolve(
            text(&raw_registry)?,
            &approval.agent_id,
            version,
            text(&rules)?,
            previous_record.as_deref().map(text).transpose()?,
            text(&inventory)?,
            ResolutionContext {
                intake: true,
                ..ResolutionContext::default()
            },
        )?
        .ok_or("charter rule 3 unresolved: no policy")?;
        let same_event = prior.as_ref().is_some_and(|p| p.event == approval.event);
        if let Some(prior) = prior.as_ref().filter(|_| same_event) {
            check_current_policy(&prior.policy, &policy)?;
        }
        let carried_work = prior
            .as_ref()
            .and_then(|p| p.obligations.as_ref())
            .map(|o| o.acceptance_work.clone())
            .unwrap_or_default();
        let required: BTreeSet<_> = carried_work.iter().map(|w| w.surface.clone()).collect();
        // OF-P6: maintenance and onboarding bind exactly the same acquired inputs.
        let acquired =
            acquisition::derive(view, root, &rules, &policy, carried_work.clone(), &required)?;
        let (obligations, classification_bases) = match acquired {
            Some(a) => (Some(a.obligations), a.classification_bases),
            None => (None, Vec::new()),
        };
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
            event: approval.event.clone(),
            policy,
            obligations,
            execution,
            acceptance_entries: prior.map(|p| p.acceptance_entries).unwrap_or_default(),
        };
        let record = DepthRecord::parse(
            &serde_json::to_string(&record).map_err(|e| e.to_string())?,
            &approval.agent_id,
            version,
        )?;
        let request = RequestBinding {
            event: approval.event.clone(),
            policy_identity: Some(record.policy.identity.clone()),
            obligations_identity: record.obligations.as_ref().map(|o| o.identity.clone()),
        };
        view.verify()?;
        Ok(Self {
            request,
            record,
            previous_record,
            previous_request: None,
            required_surfaces: required,
            carried_work,
            classification_bases,
        })
    }
}
