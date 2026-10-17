//! The depth facts of a support row, as revision 2 of the support matrix artifact states them
//! (`docs/specs/unified-agent-api/support-matrix.md`, "Support depth").
//!
//! This module derives a row's facts from the version's depth record and the content identities
//! of the evidence the record binds. It decides what the record decides: the published results
//! of record invariant 3, qualification under invariants 3 and 4, and depth qualification under
//! shared mapping rule 5. Whether a route may write a change to those facts is depth admission,
//! and is not decided here. The generator does not call it yet.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::UaaSupportState;
use crate::agent_registry::support_depth::{ObligationId, OperationDeclaration};
use crate::depth_record::{DepthRecord, DepthResult, ObligationRef};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DepthFacts {
    /// False until a frozen O covers the row's target.
    pub obligations_frozen: bool,
    pub promises: Vec<PromiseFacts>,
    pub mode_exclusions: Vec<ModeExclusionFacts>,
    pub capabilities: Vec<CapabilityFacts>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromiseFacts {
    pub operation: String,
    pub templates: Vec<String>,
    pub qualified: bool,
    pub outcomes: Vec<Outcome>,
    pub obligations: Vec<PublishedObligation>,
}

/// An obligation's published result. It states `id`, or `capability` for the `M` obligation of
/// that capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishedObligation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<ObligationId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability: Option<String>,
    pub result: DepthResult,
}

/// The outcomes of the support-depth contract's Publication table that can apply to a promise of
/// a depth-enrolled row. Qualification is stated by `qualified`; a mode exclusion, a row that is
/// not depth-enrolled and an unresolved enrollment are not stated per promise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    InsufficientDepth,
    AuthorizedDebt,
    ClassificationRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModeExclusionFacts {
    pub mode: String,
    pub operation: String,
    pub result: DepthResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityFacts {
    pub id: String,
    pub depth_qualified: bool,
}

/// The facts of the row for `target`, or `None` when the row is not depth-enrolled: the record
/// covers no mode on the target and `declared` (the registry declares the record's version and
/// lists the target) is false. `evidence_sha256` gives the content identity of a committed file,
/// or `None` when the revision does not hold it.
pub fn derive_depth_facts(
    record: Option<&DepthRecord>,
    declared: bool,
    target: &str,
    evidence_sha256: &dyn Fn(&str) -> Option<String>,
) -> Result<Option<DepthFacts>, String> {
    let covered = record.is_some_and(|record| covers(record, target));
    match record {
        // A declared target the record does not cover still lists the record's capabilities.
        Some(record) if covered || declared => Facts::new(record, target, evidence_sha256).derive(),
        None if declared => Ok(Some(not_yet_frozen())),
        _ => Ok(None),
    }
}

/// Rule 6: a row that the name-level derivation makes `supported` is `partial` unless every
/// capability its record claims is depth-qualified for the row's target.
pub fn apply_depth(uaa_support: UaaSupportState, facts: Option<&DepthFacts>) -> UaaSupportState {
    match facts {
        Some(facts)
            if uaa_support == UaaSupportState::Supported
                && facts.capabilities.iter().any(|c| !c.depth_qualified) =>
        {
            UaaSupportState::Partial
        }
        _ => uaa_support,
    }
}

/// Whether the record's selection selects a mode on the target.
fn covers(record: &DepthRecord, target: &str) -> bool {
    let enrollment = &record.policy.selection.enrollment;
    enrollment.modes.iter().any(|mode| {
        mode.targets
            .as_ref()
            .unwrap_or(&enrollment.targets)
            .iter()
            .any(|t| t == target)
    })
}

fn not_yet_frozen() -> DepthFacts {
    DepthFacts {
        obligations_frozen: false,
        promises: Vec::new(),
        mode_exclusions: Vec::new(),
        capabilities: Vec::new(),
    }
}

struct Facts<'a> {
    record: &'a DepthRecord,
    target: &'a str,
    evidence_sha256: &'a dyn Fn(&str) -> Option<String>,
    frozen: bool,
    /// E counts only while it references the P and O the record states.
    current: bool,
    accepted: bool,
}

impl<'a> Facts<'a> {
    fn new(
        record: &'a DepthRecord,
        target: &'a str,
        evidence_sha256: &'a dyn Fn(&str) -> Option<String>,
    ) -> Self {
        let obligations = record.obligations.as_ref();
        // O frozen for a target the selection does not cover freezes nothing that row publishes.
        let frozen = covers(record, target)
            && obligations.is_some_and(|o| o.targets.iter().any(|t| t == target));
        let current = match (obligations, &record.execution) {
            (Some(o), Some(e)) => {
                e.policy_identity == record.policy.identity && e.obligations_identity == o.identity
            }
            _ => false,
        };
        // Invariant 4: qualification stands only while an entry was made for what the record
        // states now.
        let accepted = record.stated_sha256().is_ok_and(|stated| {
            record
                .acceptance_entries
                .iter()
                .any(|entry| entry.record_sha256() == stated)
        });
        Self {
            record,
            target,
            evidence_sha256,
            frozen,
            current,
            accepted,
        }
    }

    fn enrollment(&self) -> &'a crate::agent_registry::support_depth::DepthEnrollmentDeclaration {
        &self.record.policy.selection.enrollment
    }

    /// The operations that serve a mode selected on the target.
    fn on_target(&self) -> BTreeMap<&'a str, &'a OperationDeclaration> {
        let enrollment = self.enrollment();
        let modes: BTreeSet<&str> = enrollment
            .modes
            .iter()
            .filter(|mode| {
                mode.targets
                    .as_ref()
                    .unwrap_or(&enrollment.targets)
                    .iter()
                    .any(|t| t == self.target)
            })
            .map(|mode| mode.selector.as_str())
            .collect();
        enrollment
            .operations
            .iter()
            .filter(|op| op.serves.iter().any(|mode| modes.contains(mode.as_str())))
            .map(|op| (op.id.as_str(), op))
            .collect()
    }

    fn derive(&self) -> Result<Option<DepthFacts>, String> {
        let on_target = self.on_target();
        let mut promises = Vec::new();
        let mut mode_exclusions = Vec::new();
        let mut qualified = BTreeSet::new();
        for (id, op) in &on_target {
            let stated = self.stated_obligations(op)?;
            if op.is_excluded() {
                for mode in &op.serves {
                    mode_exclusions.push(ModeExclusionFacts {
                        mode: mode.clone(),
                        operation: id.to_string(),
                        result: self
                            .published(op, &self.reference(id, Some(ObligationId::Ex1), None)),
                    });
                }
                continue;
            }
            let promise = self.promise(op, stated.as_ref());
            if promise.qualified {
                qualified.insert(*id);
            }
            promises.push(promise);
        }

        let mut capabilities = Vec::new();
        for mapping in &self.enrollment().capabilities {
            // Shared mapping rule 5: every operation of the mapping is enrolled and qualified on
            // the target, and one of the capability's `M` obligations is verified there. Each of
            // those operations carries this capability's `M` obligation, so its qualification
            // already requires that obligation to be verified.
            let all = mapping
                .operations
                .iter()
                .all(|op| qualified.contains(op.as_str()));
            capabilities.push(CapabilityFacts {
                id: mapping.id.clone(),
                depth_qualified: all,
            });
        }
        capabilities.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(Some(DepthFacts {
            obligations_frozen: self.frozen,
            promises,
            mode_exclusions,
            capabilities,
        }))
    }

    /// O's obligation set for the operation while O is frozen for the target. It must be the
    /// set P resolves, so that no promise qualifies on a shorter list than its own.
    fn stated_obligations(
        &self,
        op: &OperationDeclaration,
    ) -> Result<Option<BTreeSet<ObligationId>>, String> {
        if !self.frozen {
            return Ok(None);
        }
        let stated = self
            .record
            .obligations
            .iter()
            .flat_map(|o| &o.operations)
            .find(|stated| stated.id == op.id)
            .ok_or_else(|| {
                format!(
                    "depth record: the obligations part states no obligation set for \
                     operation `{}`",
                    op.id
                )
            })?;
        let policy = &self.record.policy;
        let mut resolved: BTreeSet<ObligationId> = policy
            .templates
            .iter()
            .filter(|rule| op.templates.contains(&rule.template))
            .flat_map(|rule| rule.obligations.iter().copied())
            .collect();
        if !op.is_excluded() {
            resolved.extend(policy.base.iter().copied());
        }
        let stated: BTreeSet<ObligationId> = stated.obligations.iter().copied().collect();
        if stated != resolved {
            return Err(format!(
                "depth record: the obligations part states an obligation set for operation `{}` \
                 that is not the one its policy resolves",
                op.id
            ));
        }
        Ok(Some(stated))
    }

    fn promise(
        &self,
        op: &OperationDeclaration,
        stated: Option<&BTreeSet<ObligationId>>,
    ) -> PromiseFacts {
        let mut obligations = Vec::new();
        if let Some(ids) = stated {
            for &id in ids {
                let reference = self.reference(&op.id, Some(id), None);
                obligations.push(PublishedObligation {
                    id: Some(id),
                    capability: None,
                    result: self.published(op, &reference),
                });
            }
            let mut mapped: Vec<&str> = self
                .enrollment()
                .capabilities
                .iter()
                .filter(|mapping| mapping.operations.contains(&op.id))
                .map(|mapping| mapping.id.as_str())
                .collect();
            mapped.sort_unstable();
            for capability in mapped {
                let reference = self.reference(&op.id, None, Some(capability));
                obligations.push(PublishedObligation {
                    id: None,
                    capability: Some(capability.to_string()),
                    result: self.published(op, &reference),
                });
            }
        }

        let mut outcomes = BTreeSet::new();
        let insufficient = !self.frozen
            || obligations
                .iter()
                .any(|o| matches!(o.result, DepthResult::Unverified | DepthResult::Failed));
        if insufficient {
            outcomes.insert(Outcome::InsufficientDepth);
        }
        let classification_required = self.record.obligations.iter().any(|o| {
            o.classification_required
                .iter()
                .any(|unit| unit.operation == op.id)
        });
        if classification_required {
            outcomes.insert(Outcome::ClassificationRequired);
        }
        if self.authorized_debt(op) {
            outcomes.insert(Outcome::AuthorizedDebt);
        }

        PromiseFacts {
            operation: op.id.clone(),
            templates: op.templates.iter().map(name).collect(),
            qualified: !insufficient && !classification_required && self.accepted,
            outcomes: outcomes.into_iter().collect(),
            obligations,
        }
    }

    /// A debt row in force for the version on the target covers a surface the operation owns.
    /// The rows in force are P's baseline, with each row E grants or retires replaced by E's
    /// grant or removed, while E references the P and O the record states.
    fn authorized_debt(&self, op: &OperationDeclaration) -> bool {
        let Some(stated) = self
            .record
            .obligations
            .iter()
            .flat_map(|o| &o.operations)
            .find(|stated| stated.id == op.id)
        else {
            return false;
        };
        let (grants, retired) = match &self.record.execution {
            Some(e) if self.current => (e.debt_grants.as_slice(), e.debt_retired.as_slice()),
            _ => (&[][..], &[][..]),
        };
        let baseline = self.record.policy.debt_baseline.iter().filter(|row| {
            !retired.contains(&row.row_id) && !grants.iter().any(|g| g.row_id == row.row_id)
        });
        let version = &self.enrollment().version;
        grants.iter().chain(baseline).any(|row| {
            row.authorized_at_version == *version
                && row.scope_target_triples.contains(self.target)
                && stated
                    .surfaces
                    .iter()
                    .any(|surface| *surface == row.identity())
        })
    }

    fn reference(
        &self,
        op: &str,
        id: Option<ObligationId>,
        capability: Option<&str>,
    ) -> ObligationRef {
        ObligationRef {
            operation: op.to_string(),
            id,
            capability: capability.map(str::to_string),
            target: self.target.to_string(),
        }
    }

    /// Record invariant 3: the stated result, or `unverified`. `not_applicable` stands only
    /// where P permits it, and no promise permits it for an `M` obligation.
    fn published(&self, op: &OperationDeclaration, reference: &ObligationRef) -> DepthResult {
        let Some(e) = self
            .record
            .execution
            .as_ref()
            .filter(|_| self.current && self.frozen)
        else {
            return DepthResult::Unverified;
        };
        let Some(stated) = e.results.iter().find(|r| r.obligation == *reference) else {
            return DepthResult::Unverified;
        };
        match stated.result {
            DepthResult::Verified => {
                let bound = !stated.evidence.is_empty()
                    && stated.evidence.iter().all(|file| {
                        (self.evidence_sha256)(&file.path).as_deref() == Some(file.sha256.as_str())
                    });
                if bound {
                    DepthResult::Verified
                } else {
                    DepthResult::Unverified
                }
            }
            DepthResult::NotApplicable => {
                let permitted = reference
                    .id
                    .is_some_and(|id| op.not_applicable_permitted.contains(&id));
                if permitted {
                    DepthResult::NotApplicable
                } else {
                    DepthResult::Unverified
                }
            }
            other => other,
        }
    }
}

/// The Markdown projection of one row's facts. It states every value the JSON states.
pub(super) fn render_markdown(
    agent: &str,
    version: &str,
    target: &str,
    facts: &DepthFacts,
) -> String {
    let yes_no = |value: bool| if value { "yes" } else { "no" };
    let mut out = format!(
        "\n#### Depth facts: `{agent}` `{version}` `{target}`\n\nObligations frozen: {}\n",
        yes_no(facts.obligations_frozen)
    );
    if !facts.promises.is_empty() {
        out.push_str("\n| promise | templates | qualified | outcomes | obligations |\n");
        out.push_str("|---|---|---|---|---|\n");
        for promise in &facts.promises {
            let templates = promise
                .templates
                .iter()
                .map(|t| format!("`{t}`"))
                .collect::<Vec<_>>()
                .join(", ");
            let outcomes = list_or_dash(promise.outcomes.iter().map(name));
            let obligations = list_or_dash(promise.obligations.iter().map(|o| {
                let id = match (&o.id, &o.capability) {
                    (Some(id), _) => name(id),
                    (None, Some(capability)) => format!("M {capability}"),
                    (None, None) => String::new(),
                };
                format!("`{id}` {}", name(&o.result))
            }));
            out.push_str(&format!(
                "| `{}` | {templates} | {} | {outcomes} | {obligations} |\n",
                promise.operation,
                yes_no(promise.qualified)
            ));
        }
    }
    if !facts.mode_exclusions.is_empty() {
        out.push_str("\n| mode exclusion | operation | `EX1` |\n|---|---|---|\n");
        for exclusion in &facts.mode_exclusions {
            out.push_str(&format!(
                "| `{}` | `{}` | {} |\n",
                exclusion.mode,
                exclusion.operation,
                name(&exclusion.result)
            ));
        }
    }
    if !facts.capabilities.is_empty() {
        out.push_str("\n| capability | depth_qualified |\n|---|---|\n");
        for capability in &facts.capabilities {
            out.push_str(&format!(
                "| `{}` | {} |\n",
                capability.id,
                yes_no(capability.depth_qualified)
            ));
        }
    }
    out
}

/// The JSON name of a unit enum value.
fn name<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_default()
}

fn list_or_dash(items: impl Iterator<Item = String>) -> String {
    let items: Vec<_> = items.collect();
    if items.is_empty() {
        "—".to_string()
    } else {
        items.join("; ")
    }
}
