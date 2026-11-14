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
    pub prepared: Option<(Record, Option<freeze::Captured>)>,
}
pub(crate) fn object(root: &Path, commit: &str) -> Result<Object, String> {
    Ok(Objects::new(root).commit(commit)?.object)
}
impl Emitter {
    pub(crate) fn finish<T>(
        &self,
        root: &Path,
        outcome: Result<T, String>,
        observed_authority: Result<observations::Authority, String>,
    ) -> Result<T, String> {
        // Invalid invocation bindings have no authenticated predecessor to record. Preserve
        // their existing refusal. A reached owning decision is retained with bounded reasons.
        if self.parent.is_none() && outcome.is_err() {
            return outcome;
        }
        if outcome
            .as_ref()
            .err()
            .is_some_and(|e| e.contains("isolated IN1 binding"))
            && !observed_authority
                .as_ref()
                .is_ok_and(|a| a.incomplete || a.decision.is_some())
        {
            return outcome;
        }
        let incomplete = observed_authority.as_ref().is_ok_and(|a| a.incomplete);
        let outcome = match (incomplete, outcome) {
            (true, Err(error)) => Err(format!("IN1 host observation incomplete: {error}")),
            (true, Ok(_)) => Err("IN1 host observation incomplete".into()),
            (false, outcome) => outcome,
        };
        let retained = observed_authority
            .and_then(|authority| self.retain(root, authority))
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
    /// Resolve every content-addressed input and the authenticated frontier before policy
    /// evaluation. Failed reads leave an incomplete attempt without inventing record inputs.
    pub(crate) fn prepare(&mut self, root: &Path) -> Result<(), String> {
        let prepared = self.read_inputs(root);
        observations::unavailable("in1/evaluated-inputs", &prepared);
        prepared?;
        let stable = self.recheck_freeze();
        observations::unavailable("in1/freeze-stability", &stable);
        stable
    }
    fn recheck_freeze(&self) -> Result<(), String> {
        if let Some((_, Some(captured))) = &self.prepared {
            captured.recheck(self.publication.host.as_ref(), &self.candidate)?;
        }
        Ok(())
    }
    fn read_inputs(&mut self, root: &Path) -> Result<(), String> {
        let mut objects = Objects::new(root);
        let candidate = objects.commit(&self.candidate)?.object;
        let parent = objects
            .commit(
                self.parent
                    .as_deref()
                    .ok_or("IN1 missing authenticated admission parent")?,
            )?
            .object;
        let protected_tip = objects.commit(&self.protected_tip)?.object;
        let history = self
            .publication
            .resolve(root, &[self.protected_tip.clone(), self.candidate.clone()])?;
        let predecessors = history.candidate_predecessors(&candidate.commit)?;
        let mut inputs = BTreeMap::new();
        for path in &self.input_paths {
            if let Some((identity, _)) = objects.blob(&candidate, path)? {
                inputs.insert(path.clone(), identity);
            }
        }
        let depth = objects
            .blob(&candidate, &self.record_path)?
            .map(|(_, bytes)| {
                DepthRecord::parse(
                    std::str::from_utf8(&bytes).map_err(|e| e.to_string())?,
                    &self.agent,
                    &self.version,
                )
            })
            .transpose()?;
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
        let old_state = read_state(&mut objects, &parent)?;
        let new_state = read_state(&mut objects, &candidate)?;
        let captured = depth
            .as_ref()
            .map(|_| freeze::Captured::read(self.publication.host.as_ref(), &self.candidate))
            .transpose();
        if let Ok(Some(captured)) = &captured {
            captured.pin();
        }
        let classification = captured
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|captured| {
                depth
                    .as_ref()
                    .zip(captured.as_ref())
                    .map(|(depth, captured)| {
                        freeze::classify_authenticated(
                            root,
                            &self.publication,
                            &parent,
                            &candidate,
                            depth,
                            captured.observation.as_ref(),
                            &Ok(captured.acquired.clone()),
                        )
                    })
                    .transpose()
            });
        observations::unavailable("in1/freeze-planning", &classification);
        let established = classification.as_ref().ok().and_then(Option::as_ref);
        let bootstrap = predecessors.is_empty();
        self.prepared = Some((
            Record {
                schema_version: 1,
                kind: "in1_admission".into(),
                repository: self.publication.authority.repository.clone(),
                generation_sha256: generation.as_ref().map(Generation::identity).transpose()?,
                generation,
                candidate: candidate.clone(),
                parent,
                checkout: candidate,
                head_relationship: "exact_head".into(),
                old_state,
                new_state,
                checkpoint: established.map_or(
                    if classification.is_ok() && bootstrap {
                        Checkpoint::Bootstrap
                    } else {
                        Checkpoint::Unclassified
                    },
                    |classification| classification.checkpoint,
                ),
                bootstrap: established.is_some() && bootstrap,
                transition: established
                    .and_then(|classification| classification.transition.clone()),
                decision: Decision::Refused,
                reasons: established
                    .iter()
                    .map(|classification| classification.reason)
                    .collect(),
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
                observed_authority: String::new(),
                producer_context: None,
                execution_evidence,
                claims: Claims {
                    run: 0,
                    attempt: 0,
                    job: 0,
                    check: 0,
                    event: self.publication.host.event()?,
                },
            },
            captured.ok().flatten(),
        ));
        classification.map(|_| ())
    }
    fn retain(&self, root: &Path, authority: observations::Authority) -> Result<(), String> {
        let Some(mut record) = self.prepared.as_ref().map(|(record, _)| record.clone()) else {
            self.publication.start(&self.candidate)?;
            return Err("IN1 evaluated inputs unavailable".into());
        };
        record.observed_authority = authority.identity;
        record.decision = authority.decision.unwrap_or(Decision::Refused);
        let classification_reasons = std::mem::take(&mut record.reasons);
        record.reasons = match authority.decision {
            Some(Decision::Admitted) => vec![Reason::OwningPredicatesSatisfied],
            Some(Decision::Refused) => vec![Reason::OwningPredicateRefused],
            None => Vec::new(),
        };
        if authority.decision.is_some() {
            record.reasons.extend(classification_reasons);
        }
        // Reconstruct only to verify continued availability/progression. These reads cannot
        // replace any of the evaluated inputs, even on a reached policy refusal.
        let history = self
            .publication
            .resolve(root, &[self.protected_tip.clone(), self.candidate.clone()]);
        let closing = self
            .recheck_freeze()
            .and_then(|()| history.as_ref().map_err(Clone::clone))
            .and_then(|history| {
                if history.candidate_predecessors(&record.candidate.commit)? != record.predecessors
                {
                    return Err("IN1 predecessor frontier changed after input capture".into());
                }
                if record.decision == Decision::Admitted {
                    if let (Some(generation), Some(state)) = (&record.generation, &record.new_state)
                    {
                        history.check_progression(
                            &record.candidate.commit,
                            generation,
                            state.stage,
                        )?;
                    }
                }
                Ok(())
            });
        if authority.incomplete || closing.is_err() {
            record.reasons.push(Reason::ObservationIncomplete);
        }
        // The scheduler's attempt remains observable even when closing reconstruction fails.
        let session = self.publication.start(&record.candidate.commit)?;
        if session.claims.event != record.claims.event {
            return Err("IN1 emitting event differs from evaluated event".into());
        }
        record.claims = session.claims.clone();
        let publishing = self.recheck_freeze();
        if publishing.is_err() && !record.reasons.contains(&Reason::ObservationIncomplete) {
            record.reasons.push(Reason::ObservationIncomplete);
        }
        session.publish(&mut record, || self.recheck_freeze())?;
        closing?;
        publishing?;
        history?.recheck_publication(&session.claims)
    }
}
