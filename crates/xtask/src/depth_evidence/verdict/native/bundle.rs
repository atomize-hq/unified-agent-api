use super::*;
use host::{Api, Binding, Tree};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Resolved {
    pub claims: Vec<form::Claim>,
    pub graph: DependencyGraph,
    pub environment: Environment,
    pub source: Tree,
}
fn parse<T>(bytes: &[u8], parser: fn(&str) -> Result<T, String>) -> Result<T, String> {
    parser(text(bytes)?)
}
fn canonical<T: serde::Serialize>(file: &FileRef, value: &T) -> Result<(), String> {
    // Native envelopes contain numeric revisions/host ids, unlike number-free P/O parts.
    let value = serde_json::to_value(value).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec(&value).map_err(|e| e.to_string())?;
    if crate::depth_ci::reference(&file.path, &bytes) != *file {
        return Err("native plan/index must use canonical bytes".into());
    }
    Ok(())
}
pub(super) fn approve_inputs(
    root: &Path,
    p: &Profile,
    canonical: &Tree,
    built: &Tree,
    build: &Build,
) -> Result<(), String> {
    let approved = canonical.source(root)?;
    let source = built.source(root)?;
    if built.oid != build.source_tree
        || build.host_target != p.environment.target
        || build.features != p.environment.features
        || build.toolchain != p.toolchain
        || build.covered_inputs != p.graph.inputs
    {
        return Err("native build differs from approved input/target closure".into());
    }
    p.graph.closure()?;
    for file in &p.graph.inputs {
        // Governing data is independently approved on isolated staging, not by enabling production.
        if file.path == p.registry.path {
            if file != &p.registry
                || source
                    .optional_bytes(&file.path)?
                    .as_deref()
                    .map(|b| crate::depth_ci::reference(&file.path, b))
                    .as_ref()
                    != Some(file)
                || source.executable(&file.path)?
            {
                return Err("native governing build input differs".into());
            }
            continue;
        }
        let bytes = approved
            .optional_bytes(&file.path)?
            .ok_or("missing approved producer input")?;
        if crate::depth_ci::reference(&file.path, &bytes) != *file
            || source.optional_bytes(&file.path)?.as_ref() != Some(&bytes)
            || source.executable(&file.path)? != approved.executable(&file.path)?
        {
            return Err("native producer source is not independently approved".into());
        }
    }
    for scope in &p.graph.scopes {
        let prefix = format!("{scope}/");
        let expected: BTreeSet<_> = p
            .graph
            .inputs
            .iter()
            .filter(|f| f.path.starts_with(&prefix))
            .map(|f| f.path.clone())
            .collect();
        if canonical.inventory(scope) != expected
            || built.inventory(scope) != expected
            || approved
                .committed_files(scope)?
                .into_iter()
                .collect::<BTreeSet<_>>()
                != expected
            || source
                .committed_files(scope)?
                .into_iter()
                .collect::<BTreeSet<_>>()
                != expected
        {
            return Err("native producer inventory differs".into());
        }
    }
    for path in &build.absent_inputs {
        if source.optional_bytes(path)?.is_some() || approved.optional_bytes(path)?.is_some() {
            return Err("native absent input became present".into());
        }
    }
    if !p.graph.inputs.contains(&p.collector) || !p.graph.inputs.contains(&p.native.acquisition) {
        return Err("native collector/acquisition lacks source coverage".into());
    }
    Ok(())
}
pub(super) fn resolve(
    root: &Path,
    api: &impl Api,
    p: &Profile,
    binding: &Binding,
    a: &population::Attempt,
    current: &Evaluation,
) -> Result<Resolved, String> {
    let finish = &a.finish;
    if !finish.execution_attested
        || !finish.outcomes_attested
        || !finish.registration_order_attested
        || !finish.completeness_attested
    {
        return Err("native execution/completeness attestation absent".into());
    }
    resolve_content(root, api, p, binding, &a.start, finish, current)
}
// Content inspection is shared with the unposted finish-template preparer. It does not
// authenticate an execution statement or produce evaluator authority.
pub(super) fn resolve_content(
    root: &Path,
    api: &impl Api,
    p: &Profile,
    binding: &Binding,
    start: &form::Start,
    finish: &form::Finish,
    current: &Evaluation,
) -> Result<Resolved, String> {
    host::ancestor(api, &p.repository, &start.packet_head, &binding.packet_head)?;
    host::ancestor(
        api,
        &p.repository,
        &start.packet_head,
        &finish.evidence_commit,
    )?;
    host::ancestor(
        api,
        &p.repository,
        &finish.evidence_commit,
        &binding.packet_head,
    )?;
    let registered = Tree::read(api, &p.repository, &start.packet_head)?;
    let frozen = crate::depth_record::DepthRecord::parse(
        text(&registered.bytes(root, &p.record_path)?)?,
        &current.current.agent_id,
        &current.current.version,
    )?;
    if frozen.event != current.record.event
        || frozen.policy != current.record.policy
        || frozen.obligations != current.record.obligations
        || !super::super::generation(&frozen, &start.scope)
    {
        return Err("native registration did not bind the frozen generation".into());
    }
    let source = Tree::read(api, &p.repository, &finish.evidence_commit)?;
    let index = parse(&source.file(root, &finish.bundle)?, form::Bundle::parse)?;
    canonical(&finish.bundle, &index)?;
    if index.plan != start.plan || index.files.iter().any(|f| f.path == finish.bundle.path) {
        return Err("native bundle plan or cycle differs".into());
    }
    let prefix = format!(
        "cli_manifests/codex/reports/{}/evidence/native-v1/",
        current.current.version
    );
    let mut payloads = BTreeMap::new();
    let mut total = 0;
    for file in &index.files {
        if !file.path.starts_with(&prefix) {
            return Err("native proof is outside its report root".into());
        }
        let bytes = source.file(root, file)?;
        total += bytes.len();
        if total > 16_777_216 {
            return Err("aggregate native proof exceeds bound".into());
        }
        payloads.insert(file.path.clone(), bytes);
    }
    let get = |f: &FileRef| -> Result<&[u8], String> {
        if !index.files.contains(f) {
            return Err("native index reference digest differs".into());
        }
        payloads
            .get(&f.path)
            .map(Vec::as_slice)
            .ok_or("native index omits a referenced payload".into())
    };
    let plan = parse(get(&index.plan)?, form::Plan::parse)?;
    canonical(&index.plan, &plan)?;
    if plan.scope != start.scope
        || plan.packet_head != start.packet_head
        || plan.native != p.native
        || plan.context.sha256 != p.context_intent_sha256
        || index.observed_context.path == plan.context.path
        || plan.collector != p.collector
    {
        return Err("native plan/runtime/context/collector differs".into());
    }
    let built = Tree::read(api, &p.repository, &plan.build.source_commit)?;
    host::ancestor(
        api,
        &p.repository,
        &plan.build.source_commit,
        &start.packet_head,
    )?;
    approve_inputs(root, p, &binding.canonical, &built, &plan.build)?;
    let graph = parse(get(&plan.build.dependency_graph)?, DependencyGraph::parse)?;
    let environment = parse(get(&plan.build.environment)?, Environment::parse)?;
    if graph != p.graph || environment != p.environment || plan.build.toolchain != p.toolchain {
        return Err("native build payloads lack independent approval".into());
    }
    get(&p.toolchain)?;
    get(&plan.context)?;
    get(&index.observed_context)?;
    let expected_cases: BTreeSet<_> = p.specs.values().flatten().map(|a| a.case.clone()).collect();
    let requested: BTreeSet<_> = plan.requested.iter().cloned().collect();
    if requested.is_empty()
        || requested != expected_cases
        || requested.len() != plan.requested.len()
    {
        return Err("native requested population differs from approved cases".into());
    }
    let mut required: BTreeSet<_> = [
        index.plan.clone(),
        plan.build.dependency_graph.clone(),
        plan.build.environment.clone(),
        p.toolchain.clone(),
        plan.context.clone(),
        index.observed_context.clone(),
    ]
    .into_iter()
    .chain(index.claims.clone())
    .collect();
    let mut claims = Vec::new();
    let mut outcomes = BTreeMap::new();
    let mut covered = BTreeSet::new();
    let mut obligations = BTreeSet::new();
    for file in &index.claims {
        let claim = parse(get(file)?, form::Claim::parse)?;
        if !population::same_generation(&claim.scope, &plan.scope)
            || !super::super::generation(&frozen, &claim.scope)
            || claim.build != plan.build
            || claim.mapping.executor_identity != finish.executor_identity
            || claim.mapping.obligation != claim.scope.obligation
            || !claim.provenance.iter().any(|provenance| {
                provenance.kind == ProvenanceKind::Binary && provenance.file == p.native.acquisition
            })
            || !obligations.insert(claim.scope.obligation.clone())
        {
            return Err("native claim differs from frozen plan".into());
        }
        let specs = p
            .specs
            .get(&claim.scope.obligation)
            .ok_or("unsupported native obligation")?;
        if claim.mapping.assertions.len() != specs.len() {
            return Err("native mapping case count differs".into());
        }
        for assertion in &claim.mapping.assertions {
            let expected = specs
                .iter()
                .find(|s| s.case.name == assertion.case.name)
                .ok_or("unknown native assertion")?;
            if assertion.case != expected.case
                || assertion.assertion != expected.assertion
                || assertion.production_inputs != expected.production_inputs
                || assertion.counterfactual != expected.counterfactual
                || !plan.requested.contains(&assertion.case)
            {
                return Err("native substantive mapping differs from source profile".into());
            }
        }
        let requested: BTreeSet<_> = claim.cases.requested.iter().cloned().collect();
        if requested
            != claim
                .mapping
                .assertions
                .iter()
                .map(|a| a.case.clone())
                .collect()
        {
            return Err("native claim case population differs from mapping".into());
        }
        covered.extend(requested.clone());
        for result in &claim.cases.executed {
            if !requested.contains(&result.case) {
                return Err("unexpected native executed case".into());
            }
            if outcomes
                .insert(result.case.clone(), result.outcome.clone())
                .is_some_and(|v| v != result.outcome)
            {
                return Err("inconsistent shared native case outcomes".into());
            }
        }
        required.insert(claim.cases.raw_output.clone());
        get(&claim.cases.raw_output)?;
        claims.push(claim);
    }
    for claim in &claims {
        let expected: BTreeMap<_, _> = outcomes
            .iter()
            .filter(|(case, _)| claim.cases.requested.contains(case))
            .map(|(c, o)| (c.clone(), o.clone()))
            .collect();
        let actual: BTreeMap<_, _> = claim
            .cases
            .executed
            .iter()
            .map(|r| (r.case.clone(), r.outcome.clone()))
            .collect();
        if expected != actual {
            return Err("native shared-case projection is incomplete".into());
        }
    }
    if obligations != p.specs.keys().cloned().collect()
        || covered != plan.requested.iter().cloned().collect()
        || required != index.files.iter().cloned().collect()
    {
        return Err("native aggregate obligations/payload/case population incomplete".into());
    }
    let observed = form::context::Observation::parse(get(&index.observed_context)?)?;
    let executed = outcomes
        .into_iter()
        .map(|(case, outcome)| CaseResult { case, outcome })
        .collect::<Vec<_>>();
    observed.validate(
        &p.context,
        &plan.requested,
        &executed,
        &p.graph.inputs,
        &p.native.executable_sha256,
    )?;
    Ok(Resolved {
        claims,
        graph,
        environment,
        source,
    })
}
