//! Read-only receipt form from `docs/specs/support-depth-evidence-v1.md`.
//! A parsed receipt is an attributed claim, not trusted execution, adequacy or admission.

use serde::{Deserialize, Serialize};

use crate::depth_record::{Event, FileRef, ObligationRef};

mod currentness;
mod results;
pub(crate) use results::production_results;
#[cfg(test)]
pub(crate) use results::test_results;
pub use results::{evaluate_isolated_record, EvidenceVerdicts};
pub mod native;
pub mod payload;
mod validate;
mod verdict;
pub use verdict::{evaluate_workspace, Evaluation, Verdict};
pub use verdict::{prove_isolated_ci, prove_isolated_core};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceReceipt {
    pub schema_version: u64,
    pub scope: Scope,
    pub producer: Producer,
    pub build: Build,
    pub provenance: Vec<Provenance>,
    pub cases: Cases,
    pub adequacy: Adequacy,
}

impl EvidenceReceipt {
    /// Parse bounded JSON and check form/internal mapping identity only. No host is queried and
    /// no referenced evidence is trusted or opened. Nonqualifying runs can be recorded truthfully.
    pub fn parse(raw: &str) -> Result<Self, String> {
        if raw.len() > 1_048_576 {
            return Err("depth evidence exceeds 1048576 bytes".into());
        }
        let receipt: Self = serde_json::from_str(raw).map_err(|e| e.to_string())?;
        // Deserialize first so duplicate fields are still rejected. Serde structs also accept
        // positional sequences; compare container shapes to their named-object representation.
        // This also rejects explicit null in Annex B's reused optional obligation members.
        check_shape(
            &serde_json::from_str(raw).map_err(|e| e.to_string())?,
            &serde_json::to_value(&receipt).map_err(|e| e.to_string())?,
        )?;
        receipt.validate()?;
        Ok(receipt)
    }
}

fn check_shape(raw: &serde_json::Value, typed: &serde_json::Value) -> Result<(), String> {
    use serde_json::Value;
    match (raw, typed) {
        (Value::Null, _) => return Err("depth evidence does not permit null members".into()),
        (Value::Array(values), Value::Array(expected)) => {
            for (value, expected) in values.iter().zip(expected) {
                check_shape(value, expected)?;
            }
        }
        (Value::Object(values), Value::Object(expected)) => {
            for (name, value) in values {
                check_shape(
                    value,
                    expected.get(name).ok_or("unexpected evidence member")?,
                )?;
            }
        }
        (Value::Array(_) | Value::Object(_), _) | (_, Value::Array(_) | Value::Object(_)) => {
            return Err("depth evidence requires named objects, not positional arrays".into());
        }
        _ => {}
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub agent_id: String,
    pub version: String,
    pub event: Event,
    pub enrollment_origin: EnrollmentOrigin,
    pub obligation: ObligationRef,
    pub policy_identity: String,
    pub obligations_identity: String,
    pub rules_revision: String,
    pub trust_revision: String,
    pub review_revision: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnrollmentOrigin {
    Production,
    Isolated,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Producer {
    pub kind: ProducerKind,
    pub repository: String,
    pub workflow_path: String,
    pub workflow_commit: String,
    pub collector: FileRef,
    pub run_id: u64,
    pub run_attempt: u64,
    pub job: String,
    pub artifact_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProducerKind {
    GithubActions,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Build {
    pub source_commit: String,
    pub source_tree: String,
    pub host_target: String,
    pub toolchain: FileRef,
    pub features: Vec<String>,
    pub environment: FileRef,
    pub dependency_graph: FileRef,
    pub covered_inputs: Vec<FileRef>,
    pub absent_inputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub kind: ProvenanceKind,
    pub origin: String,
    pub file: FileRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceKind {
    Binary,
    Source,
    Fixture,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Cases {
    pub requested: Vec<CaseIdentity>,
    pub executed: Vec<CaseResult>,
    pub raw_output: FileRef,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaseIdentity {
    pub name: String,
    pub executable_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaseResult {
    pub case: CaseIdentity,
    pub outcome: CaseOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseOutcome {
    Passed,
    Failed,
    Ignored,
    Skipped,
    Aborted,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Adequacy {
    pub mapping: AdequacyMapping,
    pub endorsement: Endorsement,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdequacyMapping {
    pub obligation: ObligationRef,
    pub reviewer_identity: String,
    pub executor_identity: String,
    pub assertions: Vec<AssertionBinding>,
}

impl AdequacyMapping {
    /// SHA-256 of canonical JSON over the whole mapping. This binds content, not review authority.
    pub fn sha256(&self) -> Result<String, String> {
        // No member is named `identity`, so the existing Annex B helper omits nothing here.
        crate::depth_record::part_identity(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssertionBinding {
    pub case: CaseIdentity,
    pub assertion: FileRef,
    pub production_inputs: Vec<FileRef>,
    pub counterfactual: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Endorsement {
    pub kind: EndorsementKind,
    pub repository: String,
    pub pull_request: u64,
    pub id: u64,
    pub author: String,
    pub source_commit: String,
    pub body_sha256: String,
    pub mapping_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EndorsementKind {
    PrReview,
    IssueComment,
}

// Borrowed common claims, never authenticated facts or a public admission constructor.
#[derive(Clone, Copy)]
struct Claims<'a> {
    scope: &'a Scope,
    build: &'a Build,
    provenance: &'a [Provenance],
    cases: &'a Cases,
    adequacy: &'a Adequacy,
    collector: &'a FileRef,
}
impl EvidenceReceipt {
    fn claims(&self) -> Claims<'_> {
        Claims {
            scope: &self.scope,
            build: &self.build,
            provenance: &self.provenance,
            cases: &self.cases,
            adequacy: &self.adequacy,
            collector: &self.producer.collector,
        }
    }
}
