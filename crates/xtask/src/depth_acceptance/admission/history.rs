//! Independent admission DAG, rooted in source-installed enrollment and authenticated tips.
use super::{objects::Objects, *};
use crate::{agent_lifecycle::LifecycleStage, depth_record::DepthRecord};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(crate) struct History {
    records: Vec<Record>,
    protected: String,
    anchor: String,
    ancestors: BTreeMap<String, BTreeSet<String>>,
    frontiers: BTreeMap<String, BTreeSet<String>>,
    parents: BTreeMap<String, Vec<String>>,
    pub(crate) incomplete: BTreeSet<String>,
    pub(crate) revoked: BTreeSet<String>,
    pub(super) observation: Option<(Publication, super::transport::Snapshot)>,
}
impl History {
    fn authoritative(&self, record: &Record) -> bool {
        record.successful()
            && record
                .identity()
                .is_ok_and(|id| !self.incomplete.contains(&id))
    }
    pub(crate) fn recheck(&self) -> Result<(), String> {
        if let Some((publication, snapshot)) = &self.observation {
            snapshot.recheck(publication.host.as_ref(), &publication.authority)?;
        }
        Ok(())
    }
    pub(crate) fn recheck_publication(&self, claims: &Claims) -> Result<(), String> {
        if let Some((publication, snapshot)) = &self.observation {
            snapshot.recheck_publication(publication, claims)?;
        }
        Ok(())
    }
    pub(crate) fn check_candidate(
        &self,
        root: &Path,
        commit: &str,
        record: &DepthRecord,
        state_path: &str,
    ) -> Result<(), String> {
        let mut objects = Objects::new(root);
        let object = objects.commit(commit)?.object;
        let state = objects
            .state(&object, state_path)?
            .ok_or("IN1 missing candidate lifecycle state")?;
        let generation = Generation::from_record(record);
        self.check_progression(commit, &generation, state.stage)?;
        self.require_generation_history(root, commit, record, state_path)
    }
    fn require_generation_history(
        &self,
        root: &Path,
        candidate: &str,
        expected: &DepthRecord,
        state_path: &str,
    ) -> Result<(), String> {
        let generation = Generation::from_record(expected);
        let mut objects = Objects::new(root);
        let mut lineage = self
            .ancestors
            .get(&self.protected)
            .ok_or("IN1 missing protected lineage")?
            .clone();
        lineage.insert(self.protected.clone());
        // Progression also relies on unmerged ancestors through every candidate parent.
        // A pending candidate's own transition does not require its future admission.
        lineage.extend(
            self.ancestors
                .get(candidate)
                .ok_or("IN1 missing candidate lineage")?
                .iter()
                .cloned(),
        );
        let tip = objects.commit(&self.protected)?.object;
        let path = generation_path(&mut objects, &tip, &generation)?;
        let mut stages = BTreeMap::new();
        for commit in &lineage {
            let object = objects.commit(commit)?.object;
            let Some((_, bytes)) = objects.blob(&object, &path)? else {
                continue;
            };
            let depth = DepthRecord::parse(
                std::str::from_utf8(&bytes).map_err(|e| e.to_string())?,
                &generation.selection.agent_id,
                &generation.selection.enrollment.version,
            )?;
            if Generation::from_record(&depth) == generation {
                let state = objects
                    .state(&object, state_path)?
                    .ok_or("IN1 history gap in lifecycle state")?;
                stages.insert(commit.clone(), state.stage);
            }
        }
        for (commit, stage) in &stages {
            if *commit == self.anchor && *stage <= LifecycleStage::Enrolled {
                continue;
            }
            let parents = &objects.commits[commit].parents;
            let changed = parents.iter().all(|p| !stages.contains_key(p))
                || parents
                    .iter()
                    .filter_map(|p| stages.get(p))
                    .any(|prior| prior != stage);
            if changed
                && !self.records.iter().any(|r| {
                    r.candidate.commit == *commit
                        && r.generation.as_ref() == Some(&generation)
                        && self.authoritative(r)
                })
            {
                return Err(
                    "IN1 generation history gap: lifecycle transition has no retained admission"
                        .into(),
                );
            }
        }
        Ok(())
    }
    pub(crate) fn candidate_predecessors(&self, candidate: &str) -> Result<Vec<String>, String> {
        let mut frontier = BTreeSet::new();
        for parent in self
            .parents
            .get(candidate)
            .ok_or("IN1 missing candidate parent history")?
        {
            frontier.extend(self.predecessors(parent)?);
        }
        Ok(frontier.into_iter().collect())
    }
    pub(crate) fn predecessors(&self, parent: &str) -> Result<Vec<String>, String> {
        Ok(self
            .frontiers
            .get(parent)
            .ok_or("IN1 parent absent from admission history")?
            .iter()
            .cloned()
            .collect())
    }
    /// FR2/FR3 may select current checkpoint authority only after consulting revocation.
    #[allow(dead_code)] // FR2/FR3 consume current eligibility independently of historical stage.
    pub(crate) fn eligible(&self, identity: &str) -> Result<bool, String> {
        Ok(!self.revoked.contains(identity)
            && !self.incomplete.contains(identity)
            && self
                .records
                .iter()
                .any(|r| r.identity().as_deref() == Ok(identity) && self.authoritative(r)))
    }
    /// Structural checkpoint identity supplies no receipt-free authority by itself.
    pub(crate) fn is_request_freeze(&self, commit: &str, generation: &Generation) -> bool {
        self.records.iter().any(|record| {
            self.authoritative(record)
                && record.candidate.commit == commit
                && record.generation.as_ref() == Some(generation)
                && record.checkpoint == Checkpoint::RequestFreeze
                && record.reasons.contains(&Reason::ExactFreezeTree)
        })
    }
    pub(crate) fn require_reporting(
        &self,
        context: &crate::depth_ci::reporting::ContextRecord,
    ) -> Result<(), String> {
        let identity = context.identity()?;
        let protected = &self.ancestors[&self.protected];
        if context.candidate.commit != self.protected
            && !protected.contains(&context.candidate.commit)
        {
            return Err("reporting checkpoint absent from protected lineage".into());
        }
        let admitted = self.records.iter().any(|record| {
            self.authoritative(record)
                && !self
                    .revoked
                    .contains(&record.identity().unwrap_or_default())
                && record.candidate == context.candidate
                && record.parent == context.predecessor
                && record.generation.as_ref() == Some(&context.generation)
                && record.checkpoint == Checkpoint::RequestFreeze
                && record.reasons.contains(&Reason::ExactFreezeTree)
                && record.producer_context.as_ref() == Some(&identity)
                && record.execution_evidence.is_empty()
        });
        if !admitted {
            return Err("reporting context lacks later exact-transition IN1 admission".into());
        }
        for ready in self.records.iter().filter(|record| {
            self.authoritative(record)
                && record.generation.as_ref() == Some(&context.generation)
                && record.checkpoint == Checkpoint::ExecutionRequired
        }) {
            if !self.ancestors[&ready.candidate.commit].contains(&context.candidate.commit) {
                return Err("reporting checkpoint cannot reopen irreversible readiness".into());
            }
        }
        Ok(())
    }
    pub(crate) fn check_progression(
        &self,
        commit: &str,
        generation: &Generation,
        next: LifecycleStage,
    ) -> Result<(), String> {
        let ancestors = self
            .ancestors
            .get(commit)
            .ok_or("IN1 candidate absent from authenticated history")?;
        for record in &self.records {
            if !self.authoritative(record) || record.generation.as_ref() != Some(generation) {
                continue;
            }
            if let Some(prior) = &record.new_state {
                if ancestors.contains(&record.candidate.commit) {
                    crate::runtime_follow_on::depth::check_progression(prior.stage, next)?;
                }
            }
        }
        Ok(())
    }
}
pub(super) fn resolve(
    root: &Path,
    authority: &Authority,
    records: &[Record],
    roots: &[String],
) -> Result<History, String> {
    let mut objects = Objects::new(root);
    objects.require(&authority.anchor)?;
    super::policy::authenticate(root, &authority.policy, &authority.policy)?;
    let mut pending = roots.to_vec();
    // Every retained attempt is independently authenticated, including attempts on other PRs.
    pending.extend(records.iter().map(|r| r.candidate.commit.clone()));
    let mut order = vec![];
    let mut seen = BTreeSet::new();
    let mut stack: Vec<_> = pending.into_iter().map(|commit| (commit, false)).collect();
    while let Some((commit, expanded)) = stack.pop() {
        if expanded {
            order.push(commit);
            continue;
        }
        if !seen.insert(commit.clone()) {
            continue;
        }
        let object = objects.commit(&commit)?;
        stack.push((commit.clone(), true));
        if commit == authority.anchor.commit {
            continue;
        }
        if object.parents.is_empty() {
            return Err("IN1 history gap before authenticated enrollment anchor".into());
        }
        for parent in object.parents.iter().rev() {
            stack.push((parent.clone(), false));
        }
    }
    // Independent roots can share subgraphs. Compute topological order without assuming API
    // order, wall time or first-parent-only lineage.
    let mut remaining: BTreeSet<_> = order.into_iter().collect();
    let mut ancestors: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut frontiers: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut by_commit: BTreeMap<String, Vec<&Record>> = BTreeMap::new();
    let mut decisions = BTreeMap::new();
    let mut incomplete = BTreeSet::new();
    let mut identities = BTreeSet::new();
    let mut by_identity = BTreeMap::new();
    for record in records {
        let id = record.identity()?;
        by_identity.insert(id.clone(), record);
        if record.reasons.contains(&Reason::ObservationIncomplete) {
            incomplete.insert(id.clone());
        }
        if !identities.insert(id) {
            return Err("IN1 duplicate decision identity".into());
        }
        if record.policy != authority.policy
            || record.repository != authority.repository
            || record.enrollment_anchor != authority.anchor
        {
            return Err("IN1 authority mismatch".into());
        }
        verify_record(&mut objects, record)?;
        by_commit
            .entry(record.candidate.commit.clone())
            .or_default()
            .push(record);
    }
    for record in records {
        let predecessors: Vec<_> = record
            .predecessors
            .iter()
            .filter_map(|id| by_identity.get(id).copied())
            .collect();
        let key = record.inputs_identity(&predecessors)?;
        if record.reasons != [Reason::ObservationIncomplete]
            && decisions
                .insert(key, record.decision)
                .is_some_and(|prior| prior != record.decision)
        {
            return Err("IN1 conflicting decisions over identical inputs".into());
        }
    }
    while !remaining.is_empty() {
        let ready = remaining
            .iter()
            .find(|commit| {
                **commit == authority.anchor.commit
                    || objects.commits[*commit]
                        .parents
                        .iter()
                        .all(|p| ancestors.contains_key(p))
            })
            .cloned()
            .ok_or("IN1 cyclic/incomplete parent history")?;
        remaining.remove(&ready);
        let mut lineage = BTreeSet::new();
        let mut frontier = BTreeSet::new();
        if ready != authority.anchor.commit {
            for parent in &objects.commits[&ready].parents {
                lineage.extend(ancestors[parent].iter().cloned());
                lineage.insert(parent.clone());
                frontier.extend(frontiers[parent].iter().cloned());
            }
        }
        if let Some(attempts) = by_commit.get(&ready) {
            // All retries bind the same predecessor frontier. A later retry must not cite its
            // own earlier attempt as its predecessor.
            for record in attempts {
                let expected: BTreeSet<_> = frontier
                    .iter()
                    .map(|id| by_identity[id].candidate.commit.as_str())
                    .collect();
                let mut referenced = BTreeSet::new();
                let mut references_complete = true;
                for id in &record.predecessors {
                    match by_identity.get(id) {
                        Some(prior)
                            if prior.successful()
                                && !incomplete.contains(id)
                                && lineage.contains(&prior.candidate.commit) =>
                        {
                            referenced.insert(prior.candidate.commit.as_str());
                        }
                        _ => references_complete = false,
                    }
                }
                if !lineage.contains(&record.parent.commit) {
                    return Err("IN1 inconsistent admission parent".into());
                }
                let frontier_complete = references_complete
                    && referenced == expected
                    && (expected.is_empty()
                        == (record.checkpoint == Checkpoint::Bootstrap || record.bootstrap));
                if !frontier_complete {
                    // A retained attempt remains accountable even when later parent admissions
                    // supersede its captured frontier. Quarantine it without blocking recovery.
                    incomplete.insert(record.identity()?);
                }
            }
            // A predecessor references a retained successful attempt at each frontier commit.
            // Late retries are authenticated separately and cannot rewrite an earlier child's
            // predecessor references. Keep one deterministic representative for new emissions.
            if let Some(admitted) = attempts
                .iter()
                .filter(|r| {
                    r.successful() && r.identity().is_ok_and(|id| !incomplete.contains(&id))
                })
                .min_by_key(|r| (r.claims.run, r.claims.attempt, r.claims.job))
            {
                frontier.clear();
                frontier.insert(admitted.identity()?);
            }
        }
        ancestors.insert(ready.clone(), lineage);
        frontiers.insert(ready, frontier);
    }
    let history = History {
        records: records.to_vec(),
        anchor: authority.anchor.commit.clone(),
        protected: roots
            .first()
            .cloned()
            .ok_or("IN1 missing protected history root")?,
        ancestors,
        frontiers,
        parents: objects
            .commits
            .iter()
            .map(|(id, commit)| (id.clone(), commit.parents.clone()))
            .collect(),
        incomplete,
        revoked: BTreeSet::new(),
        observation: None,
    };
    for record in records {
        if history.authoritative(record) {
            if let (Some(generation), Some(state)) = (&record.generation, &record.new_state) {
                history.check_progression(&record.candidate.commit, generation, state.stage)?;
            }
        }
    }
    Ok(history)
}
fn verify_record(objects: &mut Objects<'_>, record: &Record) -> Result<(), String> {
    if (record.protected_tip != record.parent && record.protected_tip != record.candidate)
        || record.accepted_origin != format!("https://github.com/{}.git", record.repository.name)
    {
        return Err("IN1 protected-tip/origin relationship mismatch".into());
    }
    for object in [
        &record.candidate,
        &record.parent,
        &record.checkout,
        &record.protected_tip,
    ] {
        objects.require(object)?;
    }
    objects.commit(&record.canonical_tip)?;
    for (path, expected) in &record.inputs {
        if objects
            .blob(&record.candidate, path)?
            .as_ref()
            .map(|(id, _)| id)
            != Some(expected)
        {
            return Err("IN1 authoritative input blob mismatch".into());
        }
    }
    if objects
        .blob(
            &record.protected_tip,
            crate::agent_registry::REGISTRY_RELATIVE_PATH,
        )?
        .as_ref()
        .map(|(id, _)| id)
        != Some(&record.registry_blob)
    {
        return Err("IN1 registry anchor blob mismatch".into());
    }
    for (object, state) in [
        (&record.parent, &record.old_state),
        (&record.candidate, &record.new_state),
    ] {
        if let Some(state) = state {
            if objects.state(object, &state.path)?.as_ref() != Some(state) {
                return Err("IN1 lifecycle state blob mismatch".into());
            }
        }
    }
    if let Some(generation) = &record.generation {
        let path = generation_path(objects, &record.candidate, generation)?;
        let (_, bytes) = objects
            .blob(&record.candidate, &path)?
            .ok_or("IN1 generation record missing")?;
        let depth = DepthRecord::parse(
            std::str::from_utf8(&bytes).map_err(|e| e.to_string())?,
            &generation.selection.agent_id,
            &generation.selection.enrollment.version,
        )?;
        if Generation::from_record(&depth) != *generation {
            return Err("IN1 generation input mismatch".into());
        }
    }
    Ok(())
}

fn generation_path(
    objects: &mut Objects<'_>,
    object: &Object,
    generation: &Generation,
) -> Result<String, String> {
    let (_, registry) = objects
        .blob(object, crate::agent_registry::REGISTRY_RELATIVE_PATH)?
        .ok_or("IN1 missing generation registry")?;
    let registry = crate::agent_registry::AgentRegistry::parse_unenforced(
        std::str::from_utf8(&registry).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let entry = registry
        .find(&generation.selection.agent_id)
        .ok_or("IN1 generation has no registry anchor")?;
    Ok(format!(
        "{}/reports/{}/depth-record.json",
        entry.manifest_root, generation.selection.enrollment.version
    ))
}
