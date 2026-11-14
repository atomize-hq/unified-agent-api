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

#[cfg(test)]
thread_local! {
    pub(crate) static DATA_VIEW_TEMP_ROOT: std::cell::RefCell<Option<std::path::PathBuf>> = const { std::cell::RefCell::new(None) };
}

#[derive(Debug)]
pub(crate) struct Classification {
    pub checkpoint: Checkpoint,
    pub reason: Reason,
    pub transition: Option<Transition>,
}

/// Only errors from fully read, pure content checks establish ineligibility. Legacy readers
/// that mix validation and I/O errors must propagate their failures as incomplete capture.
#[derive(Debug)]
pub(super) enum PlanningFailure {
    Incomplete(String),
    Ineligible(Reason, String),
}
impl PlanningFailure {
    fn planning(error: impl ToString) -> Self {
        Self::Ineligible(Reason::FreezePlanningRefused, error.to_string())
    }
    fn acquisition(error: impl ToString) -> Self {
        Self::Ineligible(Reason::AcquisitionAuthorityUnavailable, error.to_string())
    }
    #[cfg(test)]
    fn message(self) -> String {
        match self {
            Self::Incomplete(message) | Self::Ineligible(_, message) => message,
        }
    }
}
impl From<String> for PlanningFailure {
    fn from(error: String) -> Self {
        Self::Incomplete(error)
    }
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
    .expect("fixture freeze classification requires complete input reads")
}

pub(crate) fn classify_authenticated(
    root: &Path,
    publication: &Publication,
    parent: &Object,
    candidate: &Object,
    record: &DepthRecord,
    observation: Option<&authority::Observation>,
    acquired: &Result<Option<acquisition::Retained>, String>,
) -> Result<Classification, String> {
    let acquired = acquired.as_ref().map_err(Clone::clone)?.as_ref();
    let Some(observation) = observation else {
        return Ok(Classification::required(Reason::FreezeAuthorityUnavailable));
    };
    if observation.repository != publication.authority.repository
        || observation.predecessor != *parent
        || observation.policy != publication.authority.policy
        || observation.generation != Generation::from_record(record)
    {
        return Ok(Classification::required(Reason::FreezeAuthorityMismatch));
    }
    if observation.writer.validate().is_err() {
        return Ok(Classification::required(Reason::FreezePlanningRefused));
    }
    let (plan, proposed) = match plan_authenticated(
        root,
        parent,
        candidate,
        record,
        observation,
        Some(publication),
        acquired,
    ) {
        Ok(expected) => expected,
        Err(PlanningFailure::Ineligible(reason, _detail)) => {
            return Ok(Classification::required(reason));
        }
        Err(PlanningFailure::Incomplete(error)) => return Err(error),
    };
    if plan.require_equal(&proposed).is_err() {
        return Ok(Classification::required(Reason::FreezeTreeMismatch));
    }
    Ok(Classification {
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
    })
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
#[cfg(test)]
pub(crate) fn expected_authenticated(
    root: &Path,
    parent: &Object,
    candidate: &Object,
    record: &DepthRecord,
    observation: &authority::Observation,
    publication: Option<&Publication>,
    acquired: Option<&acquisition::Retained>,
) -> Result<(freeze_transition::Plan, freeze_transition::Tree), String> {
    plan_authenticated(
        root,
        parent,
        candidate,
        record,
        observation,
        publication,
        acquired,
    )
    .map_err(PlanningFailure::message)
}

#[allow(clippy::too_many_arguments)]
fn plan_authenticated(
    root: &Path,
    parent: &Object,
    candidate: &Object,
    record: &DepthRecord,
    observation: &authority::Observation,
    publication: Option<&Publication>,
    acquired: Option<&acquisition::Retained>,
) -> Result<(freeze_transition::Plan, freeze_transition::Tree), PlanningFailure> {
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
    let registry = AgentRegistry::parse_unenforced(
        &source
            .read(REGISTRY_RELATIVE_PATH)
            .map_err(PlanningFailure::planning)?,
    )
    .map_err(PlanningFailure::planning)?;
    let selection = &record.policy.selection;
    let entry = registry
        .find(&selection.agent_id)
        .ok_or_else(|| PlanningFailure::planning("missing predecessor freeze agent"))?;
    let version = &selection.enrollment.version;
    let rules: serde_json::Value = serde_json::from_str(
        &source
            .read(&format!("{}/RULES.json", entry.manifest_root))
            .map_err(PlanningFailure::planning)?,
    )
    .map_err(PlanningFailure::planning)?;
    let targets: Vec<String> = serde_json::from_value(rules["union"]["expected_targets"].clone())
        .map_err(PlanningFailure::planning)?;
    let (_, outputs) =
        crate::manifest_acquisition::plan::delivery_paths(&entry.manifest_root, version, &targets);
    let view = objects::DataView::new(root, parent, &predecessor)?;
    let acquisition = if let Some(acquired) = acquired {
        if !matches!(
            (&record.event, &observation.writer),
            (Event::Maintenance { .. }, authority::Writer::Maintenance)
        ) {
            return Err(PlanningFailure::acquisition(
                "ON2 acquisition must precede freeze",
            ));
        }
        let plan = crate::manifest_acquisition::plan_with_registry(
            view.root(),
            &selection.agent_id,
            version,
            &registry,
        )
        .map_err(|error| match error {
            crate::manifest_acquisition::AcquisitionError::Io(_)
            | crate::manifest_acquisition::AcquisitionError::Registry(_)
            | crate::manifest_acquisition::AcquisitionError::RoutedRegistry(_) => {
                PlanningFailure::Incomplete(error.to_string())
            }
            _ => PlanningFailure::acquisition(error),
        })?;
        let request_path = crate::agent_lifecycle::maintenance_request_path(&selection.agent_id);
        let args = crate::agent_maintenance::prepare::args_from_request_in_workspace(
            view.root(),
            Path::new(&request_path),
            false,
            true,
        )
        .map_err(|e| e.to_string())?;
        let publication = publication.ok_or_else(|| {
            PlanningFailure::Incomplete("missing protected acquisition publication".into())
        })?;
        super::policy::authenticate_inputs(root, &acquired.approved, &publication.authority.policy)
            .map_err(|failure| match failure {
                super::policy::AuthenticationFailure::SourceRead(error) => {
                    PlanningFailure::Incomplete(error)
                }
                super::policy::AuthenticationFailure::Mismatch(error) => {
                    PlanningFailure::acquisition(error)
                }
            })?;
        acquired
            .require_association(
                publication,
                parent,
                candidate,
                &Generation::from_record(record),
                &plan,
                &args.branch_name,
            )
            .map_err(PlanningFailure::acquisition)?;
        let delta = freeze_transition::acquisition::authenticated(
            &predecessor,
            &proposed,
            &plan.output_paths,
            &acquired.observation.outputs,
        )
        .map_err(PlanningFailure::acquisition)?;
        let combined = delta
            .apply(&predecessor)
            .map_err(PlanningFailure::acquisition)?;
        view.overlay(&predecessor, &combined)?;
        acquisition_checks::validate(view.root(), &predecessor, &plan)?;
        let obligations = record
            .obligations
            .as_ref()
            .ok_or_else(|| PlanningFailure::acquisition("acquisition lacks O"))?;
        for reference in &obligations.acquired_inputs {
            let entry = combined
                .get(&reference.path)
                .filter(|e| e.regular())
                .ok_or_else(|| PlanningFailure::acquisition("missing authenticated O input"))?;
            if reference.sha256 != digest(b"", &entry.bytes) {
                return Err(PlanningFailure::acquisition("O acquisition digest differs"));
            }
        }
        delta
    } else {
        freeze_transition::acquisition::validate(&predecessor, &proposed, &outputs)
            .map_err(PlanningFailure::acquisition)?
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
            freeze_transition::plan(&predecessor, &acquisition, &inputs)
                .map_err(PlanningFailure::planning)?
                .transition
        }
        (Event::Onboarding { .. }, authority::Writer::Onboarding { .. }) => {
            let inputs = crate::runtime_follow_on::depth::planner::from_predecessor(
                view.root(),
                &source,
                &registry,
                entry,
                &observation.writer,
            )?;
            inputs
                .committed_plan(&predecessor, &acquisition, &proposed)
                .map_err(PlanningFailure::planning)?
        }
        _ => {
            return Err(PlanningFailure::planning(
                "freeze writer identity differs from lifecycle",
            ))
        }
    };
    Ok((transition, proposed))
}

#[cfg(test)]
mod tests;
