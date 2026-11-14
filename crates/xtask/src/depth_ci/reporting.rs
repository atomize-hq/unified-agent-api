//! Independently versioned collector observations. Candidate files cannot install this emitter.
use crate::depth_acceptance::admission::{self, *};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

mod dispatcher;
pub(crate) use dispatcher::{capture, dispatch};
pub(crate) const PREFIX: &str = "DEPTH_REPORTING_CONTEXT_V1";
pub(crate) const KIND: &str = "reporting_only";

/// Supplied by the installed host before dispatcher selection, separately from candidate data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Invocation {
    pub claims: Claims,
    pub checkout: Object,
    pub checkout_parents: Vec<String>,
    pub dispatch_inputs: BTreeMap<String, String>,
    pub configuration: BTreeMap<String, String>,
    pub substantive_invoked: bool,
}
impl Invocation {
    pub(crate) fn normalized(mut self) -> Self {
        self.dispatch_inputs.retain(|key, _| key == "depth_mode");
        self.configuration.retain(|key, _| key == "profile");
        self
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ContextRecord {
    pub schema_version: u32,
    pub kind: String,
    pub repository: Repository,
    pub generation: Generation,
    pub enrollment_anchor: Object,
    pub candidate: Object,
    pub predecessor: Object,
    pub predecessor_admissions: Vec<String>,
    pub candidate_record: String,
    pub policy: Policy,
    pub invocation: Invocation,
    pub head_relationship: String,
    pub source_observations: BTreeMap<String, String>,
    pub indexed_tree: String,
    pub worktree_tree: String,
    pub selected_mode: String,
    pub enrolled_substantive_path_not_invoked: bool,
    pub reporting_outcome: String,
    pub collection_complete: bool,
}
impl ContextRecord {
    pub(crate) fn canonical(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let bytes = canonical(self)?;
        if bytes.len() > RECORD_LIMIT {
            return Err("reporting context exceeds 64 KiB".into());
        }
        Ok(bytes)
    }
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > RECORD_LIMIT {
            return Err("reporting context exceeds 64 KiB".into());
        }
        let record: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if record.canonical()? != bytes {
            return Err("noncanonical reporting context".into());
        }
        Ok(record)
    }
    pub(crate) fn identity(&self) -> Result<String, String> {
        Ok(digest(
            b"support-depth-reporting-context-v1\0",
            &self.canonical()?,
        ))
    }
    pub(crate) fn successful(&self) -> bool {
        self.reporting_outcome == "success"
            && self.collection_complete
            && self.enrolled_substantive_path_not_invoked
            && !self.invocation.substantive_invoked
    }
    fn validate(&self) -> Result<(), String> {
        let claims = &self.invocation.claims;
        if self.schema_version != 1
            || self.kind != KIND
            || self.selected_mode != KIND
            || !matches!(
                self.reporting_outcome.as_str(),
                "success" | "failure" | "aborted"
            )
            || !matches!(
                self.head_relationship.as_str(),
                "exact_head" | "pr_test_merge"
            )
            || claims.run == 0
            || claims.attempt == 0
            || claims.attempt > 100
            || claims.job == 0
            || claims.check == 0
            || !matches!(
                claims.event.as_str(),
                "pull_request" | "push" | "merge_group" | "workflow_dispatch"
            )
            || self.repository.id == 0
            || self.repository.owner == 0
            || self.invocation != self.invocation.clone().normalized()
            || self
                .invocation
                .configuration
                .get("profile")
                .map(String::as_str)
                != Some("synthetic-wrapper-v1")
            || self.policy.job_key != "rust-tests"
            || self.predecessor_admissions.windows(2).any(|w| w[0] >= w[1])
        {
            return Err("invalid reporting context version/mode/association".into());
        }
        for object in [
            &self.candidate,
            &self.predecessor,
            &self.enrollment_anchor,
            &self.invocation.checkout,
        ] {
            oid(&object.commit)?;
            oid(&object.tree)?;
        }
        for id in self
            .invocation
            .checkout_parents
            .iter()
            .chain(self.source_observations.values())
            .chain([
                &self.indexed_tree,
                &self.worktree_tree,
                &self.candidate_record,
            ])
            .chain(self.policy.closure.values())
        {
            oid(id)?;
        }
        for id in &self.predecessor_admissions {
            sha256(id)?;
        }
        sha256(&self.generation.policy)?;
        if let Some(obligations) = &self.generation.obligations {
            sha256(obligations)?;
        }
        self.generation.identity()?;
        sha256(&self.policy.implementation)?;
        oid(&self.policy.workflow_commit)?;
        Ok(())
    }
    pub(crate) fn check_objects(&self, root: &Path) -> Result<(), String> {
        let mut objects = admission::objects::Objects::new(root);
        for object in [
            &self.candidate,
            &self.predecessor,
            &self.enrollment_anchor,
            &self.invocation.checkout,
        ] {
            objects.require(object)?;
        }
        if self
            .invocation
            .dispatch_inputs
            .get("depth_mode")
            .map(String::as_str)
            != Some(KIND)
        {
            return Err("reporting dispatch mode mismatch".into());
        }
        let path = format!(
            "cli_manifests/{}/reports/{}/depth-record.json",
            self.generation.selection.agent_id, self.generation.selection.enrollment.version
        );
        let (blob, bytes) = objects
            .blob(&self.candidate, &path)?
            .ok_or("reporting candidate record unavailable")?;
        let depth = crate::depth_record::DepthRecord::parse(
            std::str::from_utf8(&bytes).map_err(|e| e.to_string())?,
            &self.generation.selection.agent_id,
            &self.generation.selection.enrollment.version,
        )?;
        if blob != self.candidate_record || Generation::from_record(&depth) != self.generation {
            return Err("reporting candidate record/generation mismatch".into());
        }
        let candidate = objects.commit(&self.candidate.commit)?;
        let checkout = objects.commit(&self.invocation.checkout.commit)?;
        if candidate.parents != [self.predecessor.commit.clone()]
            || checkout.parents != self.invocation.checkout_parents
            || self.indexed_tree != checkout.object.tree
            || self.worktree_tree != checkout.object.tree
            || self.source_observations
                != BTreeMap::from([
                    ("candidate".into(), self.candidate.tree.clone()),
                    ("checkout".into(), checkout.object.tree.clone()),
                    ("predecessor".into(), self.predecessor.tree.clone()),
                ])
        {
            return Err("reporting checkout/tree/parent observations mismatch".into());
        }
        match self.head_relationship.as_str() {
            "exact_head" if self.invocation.checkout == self.candidate => Ok(()),
            "pr_test_merge"
                if self.invocation.claims.event == "pull_request"
                    && checkout.parents.len() == 2
                    && checkout.parents[1] == self.candidate.commit
                    && checkout.object.tree == self.candidate.tree =>
            {
                Ok(())
            }
            _ => Err("reporting checkout has no sealed permitted head relationship".into()),
        }
    }
}
/// The independently installed IN1 source also identifies the collector implementation closure.
pub(crate) fn authority(publication: &Publication) -> Authority {
    let mut authority = publication.authority.clone();
    authority.policy.job_key = "rust-tests".into();
    authority
}
pub(crate) fn authenticate(
    root: &Path,
    publication: &Publication,
    run: u64,
    attempt: u64,
) -> Result<ContextRecord, String> {
    let (context, producer_success) = authenticate_outcome(root, publication, run, attempt)?;
    if producer_success != (context.reporting_outcome == "success") {
        return Err("reporting context differs from independent host association".into());
    }
    Ok(context)
}
fn authenticate_outcome(
    root: &Path,
    publication: &Publication,
    run: u64,
    attempt: u64,
) -> Result<(ContextRecord, bool), String> {
    let approved = authority(publication);
    let (bytes, details, job, identity) = transport::authenticate_payload(
        publication.host.as_ref(),
        &approved,
        run,
        attempt,
        PREFIX,
        KIND,
    )?;
    let record = ContextRecord::parse(&bytes)?;
    // Historical retries are independently recoverable by exact run/attempt, never the latest.
    let historical = publication
        .host
        .reporting_attempt(run, attempt)?
        .ok_or("reporting historical invocation unavailable")?
        .normalized();
    if record.identity()? != identity
        || record.invocation != historical
        || record.invocation.claims.run != run
        || record.invocation.claims.attempt != attempt
        || job["id"] != record.invocation.claims.job
        || job["check_run_id"] != record.invocation.claims.check
        || details["event"] != record.invocation.claims.event
        || details["head_sha"] != record.candidate.commit
        || record.repository != approved.repository
        || record.enrollment_anchor != approved.anchor
        || record.policy != approved.policy
    {
        return Err("reporting context differs from independent host association".into());
    }
    policy::authenticate(root, &record.policy, &approved.policy)?;
    record.check_objects(root)?;
    Ok((record, job["conclusion"] == "success"))
}

/// Authenticate before classifying. Unexpected execution retains its normal receipt path.
pub(crate) fn resolve_attempt(
    root: &Path,
    publication: &Publication,
    run: u64,
    attempt: u64,
    candidate: &Object,
    binding: &super::profile::GenerationBinding,
) -> Result<Option<(ContextRecord, bool)>, String> {
    let invocation = publication.host.reporting_attempt(run, attempt);
    observations::unavailable("reporting/invocation", &invocation);
    if invocation?.is_none() {
        return Ok(None);
    }
    let (context, producer_success) = authenticate_outcome(root, publication, run, attempt)?;
    if context.candidate != *candidate
        || context.generation != Generation::from_record(&binding.record)
    {
        return Err("reporting context differs from population binding".into());
    }
    Ok(Some((context, producer_success)))
}

#[cfg(test)]
mod tests;

/// Called by the existing collector before its witness/build/execution branch.
pub(super) fn dispatch_installed(
    root: &Path,
    profile: super::profile::Profile,
) -> Result<bool, String> {
    use super::profile::GenerationResolver;
    if !profile.synthetic() {
        return Ok(false);
    }
    let Some(host) = crate::isolated_maintenance::reporting_host(root)? else {
        return Ok(false);
    };
    let publication = host
        .reporting_publication()
        .ok_or("reporting publication unavailable")?;
    let head = super::output(crate::git_inputs::command(root).args(["rev-parse", "HEAD"]))?;
    let candidate = std::str::from_utf8(&head)
        .map_err(|e| e.to_string())?
        .trim();
    if publication.host.reporting_invocation(candidate)?.is_none() {
        return Ok(false);
    }
    let source = crate::runtime_advertising::source::Source::tip(root, candidate)?;
    let binding = profile
        .resolve_generation(&source)?
        .ok_or("reporting generation unavailable")?;
    dispatch(root, publication, candidate, &binding.record)?;
    Ok(true)
}
