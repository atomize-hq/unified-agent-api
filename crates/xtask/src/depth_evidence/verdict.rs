//! Pure evidence decisions. Host facts stay private; there is no live producer profile in D1b.
use super::{payload::*, *};
use crate::{
    depth_record::{DepthRecord, DepthResult, FileRef},
    git_inputs::GitInputs,
};
use std::{collections::BTreeSet, path::Path};

/// Expected current inputs, not authority. The host adapter must independently corroborate them.
#[derive(Debug, Clone)]
pub struct Evaluation {
    pub current: Scope,
    pub definition_sha256: String,
    pub record: DepthRecord,
    pub environment: Environment,
    pub toolchain: FileRef,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub result: DepthResult,
    pub reason: String,
}
impl Verdict {
    fn unverified(reason: impl Into<String>) -> Self {
        Self {
            result: DepthResult::Unverified,
            reason: reason.into(),
        }
    }
}

/// Read a bound receipt and evaluate without installing a producer or granting write authority.
/// D2 will provide the private authenticated host adapter. Public claims cannot substitute for it.
pub fn evaluate_workspace(
    workspace: &Path,
    evidence: &FileRef,
    current: &Evaluation,
) -> Result<Verdict, String> {
    evaluate(workspace, evidence, current, &Unavailable)
}

// Only the selected repository adapter may construct these facts. No serialized `trusted` flag,
// caller population list or public constructor is a trust source. A resolved population is complete
// for the independently established generation/obligation/build, not just the E-selected run.
trait Host {
    fn population(&self, current: &Evaluation) -> Result<Option<Population>, String>;
}
struct Unavailable;
impl Host for Unavailable {
    fn population(&self, _: &Evaluation) -> Result<Option<Population>, String> {
        Ok(None)
    }
}
struct Population {
    current: Scope,
    definition_sha256: String,
    environment: Environment,
    toolchain: FileRef,
    runs: Vec<Run>,
}
struct Run {
    evidence: FileRef,
    scope: Scope,
    definition_sha256: String,
    producer: Producer,
    build: Build,
    graph: DependencyGraph,
    environment: Environment,
    provenance: Vec<Provenance>,
    cases: Cases,
    adoption: Option<Adoption>,
}
// Constructed only after authenticated maintainer permission, reviewed source, exact digest,
// independent invocations and substantive per-assertion counterfactual adoption are established.
struct Adoption {
    endorsement: Endorsement,
    reviewer: String,
    executor: String,
}

fn text(bytes: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(bytes).map_err(|e| e.to_string())
}
fn read(view: &mut GitInputs, reference: &FileRef) -> Result<Option<Vec<u8>>, String> {
    super::validate::file(reference)?;
    let Some(bytes) = view.optional(&reference.path)? else {
        return Ok(None);
    };
    if view.reference(&reference.path)? != *reference {
        return Ok(None);
    }
    Ok(Some(bytes))
}

fn evaluate(
    workspace: &Path,
    selected: &FileRef,
    current: &Evaluation,
    host: &impl Host,
) -> Result<Verdict, String> {
    let mut view = GitInputs::new(workspace)?;
    let Some(raw) = read(&mut view, selected)? else {
        view.verify()?;
        return Ok(Verdict::unverified("missing or changed selected receipt"));
    };
    let receipt = EvidenceReceipt::parse(text(&raw)?)?;
    // Malformed referenced payloads are form errors even when no producer is installed.
    let payloads = load_payloads(&mut view, &receipt)?;
    let record = DepthRecord::parse(
        &serde_json::to_string(&current.record).map_err(|e| e.to_string())?,
        &current.current.agent_id,
        &current.current.version,
    )?;
    super::validate::sha256("obligation definition", &current.definition_sha256)?;
    let verdict = if !generation(&record, &current.current) {
        Verdict::unverified("current generation/obligation disagrees with record")
    } else if !record.execution.as_ref().is_some_and(|e| {
        e.policy_identity == current.current.policy_identity
            && e.obligations_identity == current.current.obligations_identity
            && e.results.iter().any(|r| {
                r.obligation == current.current.obligation && r.evidence.contains(selected)
            })
    }) {
        Verdict::unverified("selected evidence is not bound by current E")
    } else if let Some(population) = host.population(current)? {
        reconcile(workspace, &mut view, selected, current, population)?
    } else {
        // Reading and hashing the graph cannot authenticate its completeness or producer.
        let _ = payloads;
        Verdict::unverified("no independently resolved complete producer population")
    };
    view.verify()?;
    Ok(verdict)
}

fn generation(record: &DepthRecord, scope: &Scope) -> bool {
    record.policy.path_enabled
        && scope.rules_revision == crate::agent_registry::support_depth::GOVERNING_RULES_REVISION
        && record.policy.selection.enrollment.version == scope.version
        && record.event == scope.event
        && record.policy.selection.agent_id == scope.agent_id
        && record.policy.identity == scope.policy_identity
        && record.policy.rules_revision == scope.rules_revision
        && record.obligations.as_ref().is_some_and(|o| {
            o.identity == scope.obligations_identity
                && o.policy_identity == scope.policy_identity
                && o.targets.contains(&scope.obligation.target)
                && o.operations.iter().any(|op| {
                    op.id == scope.obligation.operation
                        && match (&scope.obligation.id, &scope.obligation.capability) {
                            (Some(id), None) => op.obligations.contains(id),
                            (None, Some(capability)) => {
                                record.policy.selection.enrollment.capabilities.iter().any(
                                    |mapping| {
                                        &mapping.id == capability
                                            && mapping
                                                .operations
                                                .contains(&scope.obligation.operation)
                                    },
                                )
                            }
                            _ => false,
                        }
                })
        })
}

fn load_payloads(
    view: &mut GitInputs,
    receipt: &EvidenceReceipt,
) -> Result<Option<(DependencyGraph, Environment)>, String> {
    let graph = read(view, &receipt.build.dependency_graph)?;
    let environment = read(view, &receipt.build.environment)?;
    // Parse every available matching payload, including when its partner is absent.
    let graph = graph
        .as_deref()
        .map(|b| DependencyGraph::parse(text(b)?))
        .transpose()?;
    let environment = environment
        .as_deref()
        .map(|b| Environment::parse(text(b)?))
        .transpose()?;
    Ok(graph.zip(environment))
}

fn reconcile(
    workspace: &Path,
    view: &mut GitInputs,
    selected: &FileRef,
    current: &Evaluation,
    population: Population,
) -> Result<Verdict, String> {
    if population.current != current.current
        || population.definition_sha256 != current.definition_sha256
        || population.environment != current.environment
        || population.toolchain != current.toolchain
    {
        return Ok(Verdict::unverified(
            "current inputs lack independent host corroboration",
        ));
    }
    if population.runs.is_empty() || !population.runs.iter().any(|r| r.evidence == *selected) {
        return Ok(Verdict::unverified(
            "selected run is not in resolved population",
        ));
    }
    let mut seen = BTreeSet::new();
    let mut passed = false;
    let mut failed = false;
    let mut incomplete = false;
    for run in population.runs {
        if !seen.insert((
            run.producer.repository.clone(),
            run.producer.run_id,
            run.producer.run_attempt,
            run.producer.job.clone(),
        )) {
            return Ok(Verdict::unverified("duplicate host run/attempt/job"));
        }
        let Some(bytes) = read(view, &run.evidence)? else {
            incomplete = true;
            continue;
        };
        let receipt = EvidenceReceipt::parse(text(&bytes)?)?;
        let Some((graph, environment)) = load_payloads(view, &receipt)? else {
            incomplete = true;
            continue;
        };
        if let Err(_reason) = qualifying(
            workspace,
            view,
            current,
            &receipt,
            &graph,
            &environment,
            &run,
        ) {
            incomplete = true;
            continue;
        }
        let requested: BTreeSet<_> = receipt.cases.requested.iter().collect();
        let executed: BTreeSet<_> = receipt.cases.executed.iter().map(|r| &r.case).collect();
        // A qualifying observed counterexample remains a failure even alongside missing cases.
        let counterexample = receipt
            .cases
            .executed
            .iter()
            .any(|r| requested.contains(&r.case) && r.outcome == CaseOutcome::Failed);
        let complete = !requested.is_empty()
            && requested == executed
            && receipt
                .cases
                .executed
                .iter()
                .all(|r| r.outcome == CaseOutcome::Passed);
        failed |= counterexample;
        passed |= complete;
        incomplete |= !counterexample && !complete;
    }
    Ok(if passed && failed {
        Verdict::unverified("contradictory qualifying runs require investigation")
    } else if failed {
        Verdict {
            result: DepthResult::Failed,
            reason: "qualifying counterexample".into(),
        }
    } else if incomplete || !passed {
        Verdict::unverified("missing, stale, nonterminal or nonqualifying evidence")
    } else {
        Verdict {
            result: DepthResult::Verified,
            reason: "current complete execution and independently adopted adequacy".into(),
        }
    })
}

fn qualifying(
    workspace: &Path,
    view: &mut GitInputs,
    current: &Evaluation,
    r: &EvidenceReceipt,
    graph: &DependencyGraph,
    environment: &Environment,
    run: &Run,
) -> Result<(), String> {
    let c = &current.current;
    if r.scope != run.scope
        || r.producer != run.producer
        || r.build != run.build
        || r.provenance != run.provenance
        || r.cases != run.cases
        || *graph != run.graph
        || *environment != run.environment
        || *environment != current.environment
        || r.build.toolchain != current.toolchain
        || r.build.host_target != c.obligation.target
        || r.build.features != environment.features
        || environment.target != c.obligation.target
        || r.scope.agent_id != c.agent_id
        || r.scope.version != c.version
        || r.scope.obligation != c.obligation
        || r.scope.rules_revision != c.rules_revision
        || r.scope.trust_revision != c.trust_revision
        || r.scope.review_revision != c.review_revision
        || run.definition_sha256 != current.definition_sha256
        || (c.enrollment_origin == EnrollmentOrigin::Production
            && r.scope.enrollment_origin != EnrollmentOrigin::Production)
    {
        return Err("scope/build/provenance lacks independent current binding".into());
    }
    if r.scope.policy_identity != c.policy_identity
        || r.scope.obligations_identity != c.obligations_identity
        || r.scope.event != c.event
    {
        let reused = current.record.execution.as_ref().is_some_and(|e| {
            e.reuse_bindings.iter().any(|b| {
                b.obligation == c.obligation
                    && b.policy_identity == r.scope.policy_identity
                    && b.obligations_identity == r.scope.obligations_identity
            })
        });
        if !reused {
            return Err("missing explicit original P/O reuse binding".into());
        }
    }
    graph.closure()?;
    let mut optional: Vec<String> = [
        ".cargo/config",
        ".cargo/config.toml",
        "rust-toolchain",
        "rust-toolchain.toml",
    ]
    .map(str::to_owned)
    .to_vec();
    for package in &graph.packages {
        if let PackageOrigin::Local { directory } = &package.origin {
            optional.push(format!("{directory}/build.rs"));
        }
    }
    if optional.iter().any(|path| {
        !graph.inputs.iter().any(|f| &f.path == path) && !r.build.absent_inputs.contains(path)
    }) {
        return Err("missing optional configuration/build-script provenance".into());
    }
    let inputs: BTreeSet<_> = graph.inputs.iter().collect();
    if inputs != r.build.covered_inputs.iter().collect() || r.provenance.is_empty() {
        return Err("incomplete covered build/provenance inputs".into());
    }
    let adoption = run
        .adoption
        .as_ref()
        .ok_or("missing authenticated adequacy adoption")?;
    let mapping = &r.adequacy.mapping;
    if adoption.endorsement != r.adequacy.endorsement
        || adoption.reviewer != mapping.reviewer_identity
        || adoption.executor != mapping.executor_identity
        || adoption.reviewer == adoption.executor
        || mapping.obligation != c.obligation
        || mapping.assertions.is_empty()
        || adoption.endorsement.source_commit != r.build.source_commit
        || adoption.endorsement.repository != r.producer.repository
    {
        return Err("adequacy adoption does not bind this execution/mapping".into());
    }
    let mapped: BTreeSet<_> = mapping.assertions.iter().map(|a| &a.case).collect();
    if mapped != r.cases.requested.iter().collect() {
        return Err("adequacy does not cover the exact requested cases".into());
    }
    for assertion in &mapping.assertions {
        if assertion.production_inputs.is_empty()
            || !inputs.contains(&assertion.assertion)
            || assertion
                .production_inputs
                .iter()
                .any(|p| !inputs.contains(p))
        {
            return Err("assertion/production dependency is not covered".into());
        }
    }
    if r.provenance.iter().any(|p| !inputs.contains(&p.file)) {
        return Err("upstream provenance is not covered".into());
    }
    super::currentness::check(workspace, view, r, graph)
}

#[cfg(test)]
mod tests;
