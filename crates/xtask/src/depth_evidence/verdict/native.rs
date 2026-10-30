//! Read-only native admission. The profile is source-owned and deliberately uninstalled.
use super::*;
use crate::depth_evidence::native as form;
use std::collections::BTreeMap;
mod bundle;
pub(super) mod host;
mod population;
pub(super) mod producer;

pub(super) struct Profile {
    pub repository: form::Repository,
    pub canonical: form::Repository,
    pub baseline: String,
    pub registry: FileRef,
    pub selection: crate::agent_registry::support_depth::DepthEnrollmentDeclaration,
    pub record_path: String,
    pub collector: FileRef,
    pub native: form::Runtime,
    pub graph: DependencyGraph,
    pub environment: Environment,
    pub toolchain: FileRef,
    pub context: form::context::Expectations,
    pub context_intent_sha256: String,
    pub definition_sha256: String,
    pub specs: BTreeMap<ObligationRef, Vec<AssertionBinding>>,
}
const LIVE_PROFILE: Option<Profile> = None;

pub(super) fn evaluate(
    root: &Path,
    view: &mut GitInputs,
    selected: &FileRef,
    current: &Evaluation,
    raw: &str,
) -> Result<Verdict, String> {
    let receipt = form::NativeReceipt::parse(raw)?;
    record(current)?;
    crate::depth_evidence::validate::sha256("obligation definition", &current.definition_sha256)?;
    // Form errors in available payloads remain errors even without a live profile.
    for (f, graph) in [
        (&receipt.build.dependency_graph, true),
        (&receipt.build.environment, false),
    ] {
        if let Some(bytes) = read(view, f)? {
            if graph {
                DependencyGraph::parse(text(&bytes)?)?;
            } else {
                Environment::parse(text(&bytes)?)?;
            }
        }
    }
    let Some(profile) = LIVE_PROFILE.as_ref() else {
        return Ok(Verdict::unverified(
            "native evidence profile is not installed",
        ));
    };
    Ok(admit(
        root,
        view,
        selected,
        current,
        &receipt,
        profile,
        &host::Github,
    )
    .unwrap_or_else(|_| {
        Verdict::unverified("native proof is unavailable, incomplete or untrusted")
    }))
}
fn record(current: &Evaluation) -> Result<DepthRecord, String> {
    DepthRecord::parse(
        &serde_json::to_string(&current.record).map_err(|e| e.to_string())?,
        &current.current.agent_id,
        &current.current.version,
    )
}
fn bound(
    view: &mut GitInputs,
    record: &DepthRecord,
) -> Result<BTreeMap<u64, form::CommentRef>, String> {
    let mut ids = BTreeMap::new();
    for reference in record
        .execution
        .iter()
        .flat_map(|e| &e.results)
        .flat_map(|r| &r.evidence)
    {
        let Some(raw) = read(view, reference)? else {
            return Err("missing bound evidence".into());
        };
        if raw.len() > 1_048_576 {
            return Err("bound receipt exceeds size limit".into());
        }
        let value: serde_json::Value =
            serde_json::from_slice(&raw).map_err(|_| "invalid bound receipt")?;
        if value["schema_version"] == 2 {
            let r = form::NativeReceipt::parse(text(&raw)?)?;
            for reference in [r.producer.registration, r.producer.completion] {
                if ids
                    .insert(reference.id, reference.clone())
                    .is_some_and(|prior| prior != reference)
                {
                    return Err("conflicting bound native host body commitments".into());
                }
            }
        }
    }
    Ok(ids)
}
fn registry(
    root: &Path,
    api: &impl host::Api,
    p: &Profile,
    binding: &host::Binding,
    view: &mut GitInputs,
) -> Result<(), String> {
    use crate::agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH};
    view.required(REGISTRY_RELATIVE_PATH)?;
    if p.registry.path != REGISTRY_RELATIVE_PATH
        || view.reference(REGISTRY_RELATIVE_PATH)? != p.registry
    {
        return Err("native governing registry differs".into());
    }
    let tree = host::Tree::read(api, &p.repository, &binding.isolated_tip)?;
    let raw = tree.file(root, &p.registry)?;
    let registry = AgentRegistry::parse_unenforced(text(&raw)?).map_err(|e| e.to_string())?;
    let codex = registry
        .find("codex")
        .ok_or("missing isolated Codex registry")?;
    let depth = codex
        .support_depth
        .as_ref()
        .ok_or("isolated Codex is not declared")?;
    if depth.enabled_paths.as_deref()
        != Some(&[crate::agent_registry::support_depth::LifecyclePath::Maintenance])
        || depth.enrollments.as_deref() != Some(std::slice::from_ref(&p.selection))
        || registry
            .agents
            .iter()
            .any(|a| a.agent_id != "codex" && a.support_depth.is_some())
    {
        return Err("native isolated declaration differs from fixed profile".into());
    }
    Ok(())
}
fn adoption(
    comments: &[population::Comment],
    mapping: &AdequacyMapping,
    executor: &str,
    repository: &str,
    pr: u64,
) -> Result<Adoption, String> {
    for c in comments.iter().filter(|c| c.authorized) {
        let Some(body) = super::github::adoption::body(&c.body)? else {
            continue;
        };
        if body.schema_version != 1 {
            continue;
        }
        crate::depth_evidence::validate::git_identity(&body.source_commit)?;
        for entry in body.mappings {
            if entry.mapping_sha256 == mapping.sha256()?
                && entry.reviewer_identity == mapping.reviewer_identity
                && entry.executor_identity == executor
                && mapping.executor_identity == executor
                && entry.reviewer_identity != executor
                && entry.independence_attested
                && entry.counterfactual_adopted
            {
                return Ok(Adoption {
                    endorsement: Endorsement {
                        kind: EndorsementKind::IssueComment,
                        repository: repository.into(),
                        pull_request: pr,
                        id: c.id,
                        author: c.login.clone(),
                        source_commit: body.source_commit,
                        body_sha256: c.reference().body_sha256,
                        mapping_sha256: entry.mapping_sha256,
                    },
                    reviewer: mapping.reviewer_identity.clone(),
                    executor: executor.into(),
                });
            }
        }
    }
    Err("missing exact independently adopted native mapping".into())
}
// Private adapter entry: neither Profile nor host facts can be supplied by an external caller.
pub(super) fn admit(
    root: &Path,
    view: &mut GitInputs,
    selected: &FileRef,
    current: &Evaluation,
    r: &form::NativeReceipt,
    p: &Profile,
    api: &impl host::Api,
) -> Result<Verdict, String> {
    let rec = record(current)?;
    if !generation(&rec, &current.current)
        || current.current.enrollment_origin != EnrollmentOrigin::Isolated
        || current.current.agent_id != "codex"
        || current.current.version != p.native.version
        || current.current.obligation.target != p.native.target
        || current.current.trust_revision != "maintainer-native-v1"
        || current.current.review_revision != "maintainer-mapping-v1"
        || rec.policy.selection.enrollment != p.selection
        || current.definition_sha256 != p.definition_sha256
        || current.environment != p.environment
        || current.toolchain != p.toolchain
        || r.producer.repository != p.repository
        || !rec.execution.as_ref().is_some_and(|e| {
            e.policy_identity == current.current.policy_identity
                && e.obligations_identity == current.current.obligations_identity
                && e.results.iter().any(|row| {
                    row.obligation == current.current.obligation && row.evidence.contains(selected)
                })
        })
    {
        return Err("unsupported native generation/profile/current inputs".into());
    }
    if crate::depth_record::due(&rec)? != p.specs.keys().cloned().collect() {
        return Err("native profile does not cover the complete due set".into());
    }
    let binding = host::bind(api, p, current)?;
    if r.producer.pull_request != binding.pr {
        return Err("native packet claim differs from host".into());
    }
    registry(root, api, p, &binding, view)?;
    let commitments = bound(view, &rec)?;
    let bound = commitments.keys().copied().collect();
    let comments = host::comments(api, p, binding.pr, &bound)?;
    for (id, expected) in &commitments {
        if !comments
            .iter()
            .any(|c| c.id == *id && c.authorized && c.reference() == *expected)
        {
            return Err("bound native host body changed or lost authority".into());
        }
    }
    let attempts = population::discover(
        &comments,
        &current.current,
        &p.repository,
        binding.pr,
        &bound,
    )?;
    let mut passed = false;
    let mut failed = false;
    let mut incomplete = false;
    let mut found = false;
    for a in attempts {
        let resolved = match bundle::resolve(root, api, p, &binding, &a, current) {
            Ok(v) => v,
            Err(reason) => {
                if a.registration.id == r.producer.registration.id {
                    return Err(reason);
                }
                incomplete = true;
                continue;
            }
        };
        let Some(claim) = resolved
            .claims
            .iter()
            .find(|claim| claim.scope.obligation == current.current.obligation)
        else {
            incomplete = true;
            continue;
        };
        let adoption = match adoption(
            &comments,
            &claim.mapping,
            &a.finish.executor_identity,
            &p.repository.name,
            binding.pr,
        ) {
            Ok(a) => a,
            Err(_) => {
                incomplete = true;
                continue;
            }
        };
        let actual = form::NativeReceipt {
            schema_version: 2,
            scope: claim.scope.clone(),
            producer: form::NativeProducer {
                kind: form::Kind::MaintainerNative,
                repository: p.repository.clone(),
                pull_request: binding.pr,
                registration: a.registration.clone(),
                completion: a.completion.clone(),
                collector: p.collector.clone(),
            },
            build: claim.build.clone(),
            provenance: claim.provenance.clone(),
            cases: claim.cases.clone(),
            adequacy: Adequacy {
                mapping: claim.mapping.clone(),
                endorsement: adoption.endorsement.clone(),
            },
        };
        let is_selected = a.registration.id == r.producer.registration.id;
        if is_selected {
            if *r != actual {
                return Err("selected native receipt differs from authenticated execution".into());
            }
            let index = form::Bundle::parse(text(&resolved.source.file(root, &a.finish.bundle)?)?)?;
            for f in index.files.iter().chain(std::iter::once(&a.finish.bundle)) {
                if read(view, f)?.as_deref() != Some(resolved.source.file(root, f)?.as_slice()) {
                    return Err("selected native proof is not coherently staged".into());
                }
            }
            found = true;
        }
        if qualify_claims(
            root,
            view,
            current,
            actual.claims(),
            Proof {
                graph: &resolved.graph,
                environment: &resolved.environment,
                adoption: Some(&adoption),
                repository: &p.repository.name,
                hosted: true,
            },
        )
        .is_err()
        {
            incomplete = true;
            continue;
        }
        let (complete, counterexample) = case_outcomes(&actual.cases);
        passed |= complete;
        failed |= counterexample;
        incomplete |= !complete && !counterexample;
    }
    if host::comments(api, p, binding.pr, &bound)? != comments
        || host::bind(api, p, current)? != binding
    {
        return Err("native host population/source/authority changed".into());
    }
    view.verify()?;
    if !found {
        return Err("selected native attempt is not in complete authenticated population".into());
    }
    Ok(outcome_verdict(passed, failed, incomplete))
}

pub(super) fn collect(root: &Path, args: form::CollectArgs) -> Result<(), String> {
    let p = LIVE_PROFILE.as_ref().ok_or(
        "native evidence profile is not installed; no registration or native execution started",
    )?;
    producer::collect(root, args, p, &host::Github)
}

pub(super) fn prepare_finish(root: &Path, args: form::PrepareFinishArgs) -> Result<String, String> {
    let p = LIVE_PROFILE
        .as_ref()
        .ok_or("native evidence profile is not installed; no completion template prepared")?;
    producer::prepare_finish(root, args, p, &host::Github)
}
