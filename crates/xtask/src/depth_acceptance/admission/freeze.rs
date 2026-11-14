//! Protected IN1 freeze classification. Candidate bytes supply the proposed tree only.
use super::{objects::Objects, *};
use crate::{
    agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH},
    depth_record::{DepthRecord, Event},
    freeze_transition,
};
use std::path::Path;

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

pub(crate) fn classify(
    root: &Path,
    publication: &Publication,
    parent: &Object,
    candidate: &Object,
    record: &DepthRecord,
    observation: Option<&authority::Observation>,
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
    match expected(root, parent, candidate, record, observation) {
        Ok((plan, proposed)) => match plan.require_equal(&proposed) {
            Ok(()) => Classification {
                checkpoint: Checkpoint::RequestFreeze,
                reason: Reason::ExactFreezeTree,
                transition: Some(Transition {
                    planner: digest(
                        b"support-depth-freeze-planner-v1\0",
                        observation.policy.implementation.as_bytes(),
                    ),
                    writer_inputs: observation
                        .identity()
                        .expect("typed freeze observation serializes"),
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

pub(crate) fn expected(
    root: &Path,
    parent: &Object,
    candidate: &Object,
    record: &DepthRecord,
    observation: &authority::Observation,
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
    let acquisition = freeze_transition::acquisition::validate(&predecessor, &proposed, &outputs)?;
    let view = objects::DataView::new(root, parent, &predecessor)?;
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
            freeze_transition::plan(&predecessor, &acquisition, &inputs)?.transition
        }
        _ => return Err("freeze writer identity differs from lifecycle".into()),
    };
    Ok((transition, proposed))
}

#[cfg(test)]
mod tests;
