use crate::{
    agent_lifecycle::LifecycleStage,
    depth_record::{DepthRecord, Event, Selection},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub(crate) const RECORD_LIMIT: usize = 65_536;
pub(crate) const DOMAIN: &[u8] = b"support-depth-in1-admission-v1\0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Generation {
    pub event: Event,
    pub selection: Selection,
    pub policy: String,
    pub obligations: Option<String>,
}
impl Generation {
    pub(crate) fn identity(&self) -> Result<String, String> {
        Ok(digest(
            b"support-depth-in1-generation-v1\0",
            &canonical(self)?,
        ))
    }
    pub(crate) fn from_record(record: &DepthRecord) -> Self {
        Self {
            event: record.event.clone(),
            selection: record.policy.selection.clone(),
            policy: record.policy.identity.clone(),
            obligations: record.obligations.as_ref().map(|o| o.identity.clone()),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Object {
    pub commit: String,
    pub tree: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Repository {
    pub name: String,
    pub id: u64,
    pub owner: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Policy {
    pub implementation: String,
    pub workflow_path: String,
    pub workflow_commit: String,
    pub workflow_id: u64,
    pub job_key: String,
    pub app_id: u64,
    pub closure: BTreeMap<String, String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Claims {
    pub run: u64,
    pub attempt: u64,
    pub job: u64,
    pub check: u64,
    pub event: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Decision {
    Admitted,
    Refused,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Checkpoint {
    Bootstrap,
    Unclassified,
    RequestFreeze,
    ExecutionRequired,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Reason {
    OwningPredicatesSatisfied,
    OwningPredicateRefused,
    ObservationIncomplete,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct State {
    pub path: String,
    pub blob: String,
    pub stage: LifecycleStage,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Record {
    pub schema_version: u32,
    pub kind: String,
    pub repository: Repository,
    pub generation: Option<Generation>,
    pub generation_sha256: Option<String>,
    pub candidate: Object,
    pub parent: Object,
    pub checkout: Object,
    pub head_relationship: String,
    pub old_state: Option<State>,
    pub new_state: Option<State>,
    pub checkpoint: Checkpoint,
    pub decision: Decision,
    pub reasons: Vec<Reason>,
    pub predecessors: Vec<String>,
    pub enrollment_anchor: Object,
    pub policy: Policy,
    pub protected_tip: Object,
    pub canonical_tip: String,
    pub accepted_origin: String,
    pub registry_blob: String,
    /// Path -> Git blob identity for every authoritative input consumed by this decision.
    pub inputs: BTreeMap<String, String>,
    /// Canonical commitment to independently read host facts, including observed absence.
    pub observed_authority: String,
    /// FR2/FR3 supply authenticated freeze and producer-context references.
    pub producer_context: Option<String>,
    pub execution_evidence: Vec<String>,
    pub claims: Claims,
}
impl Record {
    pub(crate) fn canonical(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let bytes = canonical(self)?;
        if bytes.len() > RECORD_LIMIT {
            return Err("IN1 control record exceeds 64 KiB".into());
        }
        Ok(bytes)
    }
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > RECORD_LIMIT {
            return Err("IN1 control record exceeds 64 KiB".into());
        }
        let record: Self = serde_json::from_slice(bytes).map_err(|e| format!("IN1 record: {e}"))?;
        if record.canonical()? != bytes {
            return Err("noncanonical IN1 control record".into());
        }
        Ok(record)
    }
    pub(crate) fn identity(&self) -> Result<String, String> {
        Ok(digest(DOMAIN, &self.canonical()?))
    }
    /// A retry's host association is excluded. Root snapshots and all evaluated inputs remain.
    pub(crate) fn inputs_identity(&self) -> Result<String, String> {
        let mut inputs = self.clone();
        inputs.claims = Claims {
            run: 0,
            attempt: 0,
            job: 0,
            check: 0,
            event: self.claims.event.clone(),
        };
        inputs.decision = Decision::Admitted;
        inputs.reasons.clear();
        Ok(digest(
            b"support-depth-in1-inputs-v1\0",
            &canonical(&inputs)?,
        ))
    }
    /// A reached admission with failed closing observations cannot grant historical authority.
    pub(crate) fn successful(&self) -> bool {
        self.decision == Decision::Admitted
            && !self.reasons.contains(&Reason::ObservationIncomplete)
    }
    fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 || self.kind != "in1_admission" {
            return Err("unsupported IN1 control record version/kind".into());
        }
        if self.reasons.contains(&Reason::ObservationIncomplete)
            && !matches!(
                (self.decision, self.reasons.as_slice()),
                (Decision::Refused, [Reason::ObservationIncomplete])
                    | (
                        Decision::Refused,
                        [
                            Reason::OwningPredicateRefused,
                            Reason::ObservationIncomplete
                        ]
                    )
                    | (
                        Decision::Admitted,
                        [
                            Reason::OwningPredicatesSatisfied,
                            Reason::ObservationIncomplete
                        ]
                    )
            )
        {
            return Err(
                "IN1 incomplete observation has inconsistent reached decision/reasons".into(),
            );
        }
        for object in [
            &self.candidate,
            &self.parent,
            &self.checkout,
            &self.enrollment_anchor,
            &self.protected_tip,
        ] {
            oid(&object.commit)?;
            oid(&object.tree)?;
        }
        oid(&self.canonical_tip)?;
        oid(&self.registry_blob)?;
        oid(&self.policy.workflow_commit)?;
        sha256(&self.policy.implementation)?;
        sha256(&self.observed_authority)?;
        if self.policy.workflow_id == 0
            || self.policy.app_id == 0
            || self.policy.job_key.is_empty()
            || self.policy.workflow_path.is_empty()
        {
            return Err("invalid IN1 policy provenance".into());
        }
        for identity in self.inputs.values().chain(self.policy.closure.values()) {
            oid(identity)?;
        }
        for identity in self
            .predecessors
            .iter()
            .chain(self.producer_context.iter())
            .chain(self.execution_evidence.iter())
        {
            sha256(identity)?;
        }
        if self.predecessors.windows(2).any(|w| w[0] >= w[1])
            || self.predecessors.len() > 1000
            || self.reasons.is_empty()
            || self.reasons.len() > 16
            || self.claims.run == 0
            || self.claims.attempt == 0
            || self.claims.attempt > 100
            || self.claims.job == 0
            || self.claims.check == 0
            || !matches!(
                self.claims.event.as_str(),
                "pull_request" | "merge_group" | "push" | "workflow_dispatch"
            )
            || self.repository.id == 0
            || self.repository.owner == 0
            || self.head_relationship != "exact_head"
            || self.checkout != self.candidate
        {
            return Err("invalid IN1 decision inputs or association claims".into());
        }
        if self
            .generation
            .as_ref()
            .map(Generation::identity)
            .transpose()?
            != self.generation_sha256
        {
            return Err("IN1 generation identity mismatch".into());
        }
        if let Some(generation) = &self.generation {
            sha256(&generation.policy)?;
            if let Some(o) = &generation.obligations {
                sha256(o)?;
            }
        }
        for state in self.old_state.iter().chain(&self.new_state) {
            oid(&state.blob)?;
        }
        Ok(())
    }
}
pub(crate) fn canonical(value: &impl Serialize) -> Result<Vec<u8>, String> {
    // serde_json::Value uses a BTreeMap. Integers have one decimal representation. All fields
    // are typed, and round-trip equality rejects whitespace, duplicate keys and alternate escapes.
    serde_json::to_vec(&serde_json::to_value(value).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
pub(crate) fn digest(domain: &[u8], bytes: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(domain);
    hash.update(bytes);
    hex::encode(hash.finalize())
}
pub(crate) fn oid(value: &str) -> Result<(), String> {
    identity(value, 40)
}
pub(crate) fn sha256(value: &str) -> Result<(), String> {
    identity(value, 64)
}
fn identity(value: &str, size: usize) -> Result<(), String> {
    if value.len() != size
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("invalid IN1 object/digest identity".into());
    }
    Ok(())
}
