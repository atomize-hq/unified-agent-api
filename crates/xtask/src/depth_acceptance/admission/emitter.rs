//! Terminal real-IN1 emission. Publication is part of successful admission completion.
use super::{objects::Objects, *};
use crate::depth_record::DepthRecord;
use std::{collections::BTreeMap, path::Path};

pub(crate) struct Emitter {
    pub context: crate::isolated_maintenance::Context,
    pub parent: Option<String>,
    pub publication: Publication,
    pub protected_tip: String,
    pub canonical_tip: String,
    pub registry_blob: String,
    pub candidate: String,
    pub state_path: Option<String>,
    pub record_path: String,
    pub agent: String,
    pub version: String,
    pub input_paths: Vec<String>,
}
pub(crate) fn object(root: &Path, commit: &str) -> Result<Object, String> {
    Ok(Objects::new(root).commit(commit)?.object)
}
impl Emitter {
    pub(crate) fn finish<T>(&self, root: &Path, outcome: Result<T, String>) -> Result<T, String> {
        // Invalid invocation bindings have no authenticated predecessor to record. Preserve
        // their existing refusal. A reached owning decision is retained with bounded reasons.
        if self.parent.is_none() && outcome.is_err() {
            return outcome;
        }
        if outcome
            .as_ref()
            .err()
            .is_some_and(|e| e.contains("isolated IN1 binding"))
        {
            return outcome;
        }
        let retained = self
            .retain(root, outcome.is_ok())
            .and_then(|()| self.context.recheck(root));
        match (outcome, retained) {
            (Ok(value), Ok(())) => Ok(value),
            (Ok(_), Err(error)) => Err(format!("IN1 admission publication incomplete: {error}")),
            (Err(error), Ok(())) => Err(error),
            (Err(error), Err(publication)) => Err(format!(
                "{error}; IN1 refusal publication incomplete: {publication}"
            )),
        }
    }
    fn retain(&self, root: &Path, admitted: bool) -> Result<(), String> {
        let parent = self
            .parent
            .as_deref()
            .ok_or("IN1 missing authenticated admission parent")?;
        let mut objects = Objects::new(root);
        let candidate = objects.commit(&self.candidate)?.object;
        let parent = objects.commit(parent)?.object;
        let protected_tip = objects.commit(&self.protected_tip)?.object;
        let history = self
            .publication
            .resolve(root, &[self.protected_tip.clone(), self.candidate.clone()]);
        // The scheduler's attempt remains observable if reconstruction or later emission fails.
        let session = self.publication.start(&candidate.commit)?;
        if admitted {
            history.as_ref().map_err(Clone::clone)?;
        }
        let predecessors = match &history {
            Ok(history) => history.candidate_predecessors(&candidate.commit)?,
            Err(_) => Vec::new(),
        };
        let mut inputs = BTreeMap::new();
        for path in &self.input_paths {
            if let Some((identity, _)) = objects.blob(&candidate, path)? {
                inputs.insert(path.clone(), identity);
            }
        }
        let depth = objects
            .blob(&candidate, &self.record_path)?
            .and_then(|(_, bytes)| {
                std::str::from_utf8(&bytes)
                    .ok()
                    .and_then(|raw| DepthRecord::parse(raw, &self.agent, &self.version).ok())
            });
        let generation = depth.as_ref().map(Generation::from_record);
        let mut execution_evidence = Vec::new();
        if let Some(depth) = &depth {
            let mut paths = Vec::new();
            if let Some(obligations) = &depth.obligations {
                paths.extend(
                    obligations
                        .acquired_inputs
                        .iter()
                        .map(|input| input.path.clone()),
                );
            }
            if let Some(execution) = &depth.execution {
                for evidence in execution.results.iter().flat_map(|row| &row.evidence) {
                    execution_evidence.push(evidence.sha256.clone());
                    paths.push(evidence.path.clone());
                }
            }
            for path in paths {
                if let Some((identity, _)) = objects.blob(&candidate, &path)? {
                    inputs.insert(path, identity);
                }
            }
        }
        execution_evidence.sort();
        execution_evidence.dedup();
        let read_state = |objects: &mut Objects<'_>, object: &Object| {
            self.state_path
                .as_ref()
                .map(|path| objects.state(object, path))
                .transpose()
                .map(Option::flatten)
        };
        let old_state = read_state(&mut objects, &parent);
        let new_state = read_state(&mut objects, &candidate);
        if admitted {
            old_state.as_ref().map_err(Clone::clone)?;
            if let (Some(generation), Some(state)) =
                (&generation, new_state.as_ref().map_err(Clone::clone)?)
            {
                history.as_ref().map_err(Clone::clone)?.check_progression(
                    &candidate.commit,
                    generation,
                    state.stage,
                )?;
            }
        }
        session.publish(&Record {
            schema_version: 1,
            kind: "in1_admission".into(),
            repository: self.publication.authority.repository.clone(),
            generation_sha256: generation.as_ref().map(Generation::identity).transpose()?,
            generation,
            candidate: candidate.clone(),
            parent,
            checkout: candidate,
            head_relationship: "exact_head".into(),
            old_state: old_state.unwrap_or(None),
            new_state: new_state.unwrap_or(None),
            checkpoint: if predecessors.is_empty() && history.is_ok() {
                Checkpoint::Bootstrap
            } else {
                Checkpoint::Unclassified
            },
            decision: if admitted {
                Decision::Admitted
            } else {
                Decision::Refused
            },
            reasons: vec![if admitted {
                Reason::OwningPredicatesSatisfied
            } else {
                Reason::OwningPredicateRefused
            }],
            predecessors,
            enrollment_anchor: self.publication.authority.anchor.clone(),
            policy: self.publication.authority.policy.clone(),
            protected_tip,
            canonical_tip: self.canonical_tip.clone(),
            accepted_origin: format!(
                "https://github.com/{}.git",
                self.publication.authority.repository.name
            ),
            registry_blob: self.registry_blob.clone(),
            inputs,
            producer_context: None,
            execution_evidence,
            claims: session.claims.clone(),
        })?;
        if let Ok(history) = history {
            history.recheck_publication(&session.claims)?;
        }
        Ok(())
    }
}
