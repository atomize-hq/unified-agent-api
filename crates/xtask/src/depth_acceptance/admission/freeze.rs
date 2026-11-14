//! Protected IN1 freeze classification. Candidate bytes supply the proposed tree only.
use super::{objects::Objects, *};
use crate::{
    agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH},
    depth_record::{DepthRecord, Event},
    freeze_transition,
};
use std::path::Path;

pub(crate) use crate::isolated_maintenance::acquisition;
mod acquisition_checks;
mod captured;
pub(crate) use captured::Captured;
pub(crate) mod authority;
mod objects;

pub(crate) struct Classification {
    pub checkpoint: Checkpoint,
    pub reason: Reason,
    pub transition: Option<Transition>,
}
impl Classification {
    fn required(reason: Reason) -> Self {
        Self {
            checkpoint: Checkpoint::ExecutionRequired,
            reason,
            transition: None,
        }
    }
}

#[cfg(test)]
pub(crate) fn classify(
    root: &Path,
    publication: &Publication,
    parent: &Object,
    candidate: &Object,
    record: &DepthRecord,
    observation: Option<&authority::Observation>,
) -> Classification {
    classify_authenticated(
        root,
        publication,
        parent,
        candidate,
        record,
        observation,
        &Ok(None),
    )
}

pub(crate) fn classify_authenticated(
    root: &Path,
    publication: &Publication,
    parent: &Object,
    candidate: &Object,
    record: &DepthRecord,
    observation: Option<&authority::Observation>,
    acquired: &Result<Option<acquisition::Retained>, String>,
) -> Classification {
    let Some(observation) = observation else {
        return Classification::required(Reason::FreezeAuthorityUnavailable);
    };
    if observation.repository != publication.authority.repository
        || observation.predecessor != *parent
        || observation.policy != publication.authority.policy
        || observation.generation != Generation::from_record(record)
    {
        return Classification::required(Reason::FreezeAuthorityMismatch);
    }
    if observation.writer.validate().is_err() {
        return Classification::required(Reason::FreezePlanningRefused);
    }
    let acquired = match acquired {
        Ok(value) => value.as_ref(),
        Err(_) => return Classification::required(Reason::AcquisitionAuthorityUnavailable),
    };
    match expected_authenticated(
        root,
        parent,
        candidate,
        record,
        observation,
        Some(publication),
        acquired,
    ) {
        Ok((plan, proposed)) => match plan.require_equal(&proposed) {
            Ok(()) => Classification {
                checkpoint: Checkpoint::RequestFreeze,
                reason: Reason::ExactFreezeTree,
                transition: Some(Transition {
                    planner: digest(
                        b"support-depth-freeze-planner-v1\0",
                        observation.policy.implementation.as_bytes(),
                    ),
                    writer_inputs: match acquired {
                        Some(acquired) => digest(
                            b"support-depth-freeze-acquisition-inputs-v1\0",
                            &super::canonical(&(observation, &acquired.observation))
                                .expect("typed acquisition observation serializes"),
                        ),
                        None => observation
                            .identity()
                            .expect("typed freeze observation serializes"),
                    },
                }),
            },
            Err(_) => Classification::required(Reason::FreezeTreeMismatch),
        },
        Err(error) if error == "acquisition authority unavailable" => {
            Classification::required(Reason::AcquisitionAuthorityUnavailable)
        }
        Err(_) => Classification::required(Reason::FreezePlanningRefused),
    }
}

#[cfg(test)]
pub(crate) fn expected(
    root: &Path,
    parent: &Object,
    candidate: &Object,
    record: &DepthRecord,
    observation: &authority::Observation,
) -> Result<(freeze_transition::Plan, freeze_transition::Tree), String> {
    expected_authenticated(root, parent, candidate, record, observation, None, None)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn expected_authenticated(
    root: &Path,
    parent: &Object,
    candidate: &Object,
    record: &DepthRecord,
    observation: &authority::Observation,
    publication: Option<&Publication>,
    acquired: Option<&acquisition::Retained>,
) -> Result<(freeze_transition::Plan, freeze_transition::Tree), String> {
    let mut objects = Objects::new(root);
    let predecessor = objects::read(&mut objects, parent)?;
    let proposed = objects::read(&mut objects, candidate)?;
    let source = crate::runtime_advertising::source::Source::captured_inventory(
        root,
        predecessor
            .iter()
            .filter(|(_, entry)| entry.regular())
            .map(|(path, entry)| (path.clone(), (entry.bytes.clone(), entry.mode == "100755")))
            .collect(),
        predecessor
            .iter()
            .filter(|(_, entry)| !entry.regular())
            .map(|(path, _)| path.clone())
            .collect(),
    )?;
    let registry = AgentRegistry::parse_unenforced(&source.read(REGISTRY_RELATIVE_PATH)?)
        .map_err(|e| e.to_string())?;
    let selection = &record.policy.selection;
    let entry = registry
        .find(&selection.agent_id)
        .ok_or("missing predecessor freeze agent")?;
    let version = &selection.enrollment.version;
    let rules: serde_json::Value =
        serde_json::from_str(&source.read(&format!("{}/RULES.json", entry.manifest_root))?)
            .map_err(|e| e.to_string())?;
    let targets: Vec<String> = serde_json::from_value(rules["union"]["expected_targets"].clone())
        .map_err(|e| e.to_string())?;
    let (_, outputs) =
        crate::manifest_acquisition::plan::delivery_paths(&entry.manifest_root, version, &targets);
    let view = objects::DataView::new(root, parent, &predecessor)?;
    let acquisition = if let Some(acquired) = acquired {
        let checked = || -> Result<_, String> {
            if !matches!(
                (&record.event, &observation.writer),
                (Event::Maintenance { .. }, authority::Writer::Maintenance)
            ) {
                return Err("ON2 acquisition must precede freeze".into());
            }
            let plan = crate::manifest_acquisition::plan_with_registry(
                view.root(),
                &selection.agent_id,
                version,
                &registry,
            )
            .map_err(|e| e.to_string())?;
            let request_path =
                crate::agent_lifecycle::maintenance_request_path(&selection.agent_id);
            let args = crate::agent_maintenance::prepare::args_from_request_in_workspace(
                view.root(),
                Path::new(&request_path),
                false,
                true,
            )
            .map_err(|e| e.to_string())?;
            acquired.require_binding(
                root,
                publication.ok_or("missing protected acquisition publication")?,
                parent,
                candidate,
                &Generation::from_record(record),
                &plan,
                &args.branch_name,
            )?;
            let delta = freeze_transition::acquisition::authenticated(
                &predecessor,
                &proposed,
                &plan.output_paths,
                &acquired.observation.outputs,
            )?;
            let combined = delta.apply(&predecessor)?;
            view.overlay(&predecessor, &combined)?;
            acquisition_checks::validate(view.root(), &predecessor, &plan)?;
            let obligations = record.obligations.as_ref().ok_or("acquisition lacks O")?;
            for reference in &obligations.acquired_inputs {
                let entry = combined
                    .get(&reference.path)
                    .filter(|e| e.regular())
                    .ok_or("missing authenticated O input")?;
                if reference.sha256 != digest(b"", &entry.bytes) {
                    return Err("O acquisition digest differs".into());
                }
            }
            Ok(delta)
        };
        checked().map_err(|_| "acquisition authority unavailable")?
    } else {
        freeze_transition::acquisition::validate(&predecessor, &proposed, &outputs)?
    };
    let transition = match (&record.event, &observation.writer) {
        (Event::Maintenance { .. }, authority::Writer::Maintenance) => {
            let request_path =
                crate::agent_lifecycle::maintenance_request_path(&selection.agent_id);
            let args = crate::agent_maintenance::prepare::args_from_request_in_workspace(
                view.root(),
                Path::new(&request_path),
                false,
                true,
            )
            .map_err(|e| e.to_string())?;
            let inputs =
                crate::agent_maintenance::prepare::derive_inputs(view.root(), &args, &registry)
                    .map_err(|e| e.to_string())?;
            freeze_transition::plan(&predecessor, &acquisition, &inputs)?.transition
        }
        (Event::Onboarding { .. }, authority::Writer::Onboarding { .. }) => {
            let inputs = crate::runtime_follow_on::depth::planner::from_predecessor(
                view.root(),
                &source,
                &registry,
                entry,
                &observation.writer,
            )?;
            inputs.committed_plan(&predecessor, &acquisition, &proposed)?
        }
        _ => return Err("freeze writer identity differs from lifecycle".into()),
    };
    Ok((transition, proposed))
}

#[cfg(test)]
mod tests;
