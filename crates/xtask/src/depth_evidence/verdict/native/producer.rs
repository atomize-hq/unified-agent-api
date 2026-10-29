//! Foreground native attempt production; this module never installs a trust profile.
use super::*;
use crate::depth_evidence::native::CollectArgs;
use std::{fs, path::PathBuf};
pub(in crate::depth_evidence::verdict) mod execution;
pub(in crate::depth_evidence::verdict) mod registration;
mod retained;
use registration::{register, StartApi};
use retained::Retained;
const INTENT: &str = "crates/xtask/data/depth-native-context-intent-v1.json";
const MAPPINGS: &str = "crates/xtask/data/depth-native-mappings-v1.json";
pub(super) fn canonical(value: &impl serde::Serialize) -> Result<Vec<u8>, String> {
    Ok(serde_json::to_vec(
        &serde_json::to_value(value).map_err(|_| "cannot encode native payload")?,
    )
    .map_err(|_| "cannot encode native payload")?)
}
fn fence(tag: &str, value: &impl serde::Serialize) -> Result<String, String> {
    let bytes = canonical(value)?;
    if bytes.len() > 65_536 {
        return Err("native host block exceeds bound".into());
    }
    Ok(format!(
        "```{tag}\n{}\n```",
        std::str::from_utf8(&bytes).map_err(|_| "invalid native block")?
    ))
}
fn directory(p: &Profile, attempt: &str) -> Result<String, String> {
    if attempt.is_empty()
        || attempt.len() > 80
        || !attempt
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("attempt must be a fresh alphanumeric/hyphen leaf (max 80 bytes)".into());
    }
    Ok(format!(
        "cli_manifests/codex/reports/{}/evidence/native-v1/{attempt}",
        p.native.version
    ))
}
fn current(view: &mut GitInputs, p: &Profile) -> Result<Evaluation, String> {
    let rec = DepthRecord::parse(
        text(&view.required(&p.record_path)?)?,
        "codex",
        &p.native.version,
    )?;
    let due = crate::depth_record::due(&rec)?;
    if p.native.version != "0.156.1"
        || p.native.target != "x86_64-unknown-linux-musl"
        || p.environment.target != crate::depth_ci::TARGET
        || due.iter().any(|o| o.target != p.native.target)
    {
        return Err("unsupported native version/target pair".into());
    }

    if due.is_empty()
        || due != p.specs.keys().cloned().collect()
        || rec.policy.selection.enrollment != p.selection
    {
        return Err("native packet selection differs from fixed profile".into());
    }
    let scope = Scope {
        agent_id: "codex".into(),
        version: p.native.version.clone(),
        event: rec.event.clone(),
        enrollment_origin: EnrollmentOrigin::Isolated,
        obligation: due.into_iter().next().unwrap(),
        policy_identity: rec.policy.identity.clone(),
        obligations_identity: rec
            .obligations
            .as_ref()
            .ok_or("native packet has no frozen O")?
            .identity
            .clone(),
        rules_revision: rec.policy.rules_revision.clone(),
        trust_revision: "maintainer-native-v1".into(),
        review_revision: "maintainer-mapping-v1".into(),
    };
    Ok(Evaluation {
        current: scope,
        definition_sha256: p.definition_sha256.clone(),
        record: rec,
        environment: p.environment.clone(),
        toolchain: p.toolchain.clone(),
    })
}
fn mapped(view: &mut GitInputs, p: &Profile) -> Result<Vec<AdequacyMapping>, String> {
    for path in [INTENT, MAPPINGS] {
        if !p.graph.inputs.iter().any(|f| f.path == path) {
            return Err("native intent/mappings lack approved build coverage".into());
        }
    }
    let raw = view.required(MAPPINGS)?;
    let mappings: Vec<AdequacyMapping> =
        serde_json::from_slice(&raw).map_err(|_| "invalid fixed native mappings")?;
    if raw.len() > 1_048_576 || mappings.len() > 4096 || mappings.len() != p.specs.len() {
        return Err("native mapping population differs".into());
    }
    let mut seen = BTreeSet::new();
    for m in &mappings {
        if !seen.insert(m.obligation.clone()) || p.specs.get(&m.obligation) != Some(&m.assertions) {
            return Err("native assertion mapping differs".into());
        }
        crate::depth_evidence::validate::mapping(m)?;
    }
    Ok(mappings)
}
fn source_check(
    root: &Path,
    api: &impl host::Api,
    p: &Profile,
    b: &host::Binding,
    current: &Evaluation,
    view: &mut GitInputs,
    build: &Build,
) -> Result<(), String> {
    registry(root, api, p, b, view)?;
    for file in &p.graph.inputs {
        view.required(&file.path)?;
        if view.reference(&file.path)? != *file {
            return Err("native producer working/index input changed".into());
        }
    }
    for scope in &p.graph.scopes {
        let expected: BTreeSet<_> = p
            .graph
            .inputs
            .iter()
            .filter(|f| f.path.starts_with(&format!("{scope}/")))
            .map(|f| f.path.clone())
            .collect();
        if view
            .files_under(scope)?
            .into_iter()
            .collect::<BTreeSet<_>>()
            != expected
        {
            return Err("native producer working inventory changed".into());
        }
    }

    let packet = host::Tree::read(api, &p.repository, &b.packet_head)?;
    let frozen = DepthRecord::parse(
        text(&packet.bytes(root, &p.record_path)?)?,
        "codex",
        &p.native.version,
    )?;
    if frozen.event != current.record.event
        || frozen.policy != current.record.policy
        || frozen.obligations != current.record.obligations
    {
        return Err("native local packet differs from authenticated frozen packet".into());
    }
    let request_path = crate::agent_lifecycle::maintenance_request_path("codex");
    let raw = view.required(&request_path)?;
    if packet.bytes(root, &request_path)? != raw {
        return Err("native request differs from authenticated packet".into());
    }
    let request: crate::depth_record::request::StandingRequest =
        toml_edit::de::from_str(text(&raw)?).map_err(|_| "invalid native standing request")?;
    let depth = request
        .support_depth
        .ok_or("native request did not freeze depth")?;
    let request_binding = crate::depth_record::RequestBinding {
        event: crate::depth_record::Event::Maintenance {
            request_commit: request.request_commit,
            request_recorded_at: request.request_recorded_at,
            trigger_kind: request.trigger_kind,
            opened_from: request.opened_from,
        },
        policy_identity: depth.policy_identity,
        obligations_identity: depth.obligations_identity,
    };
    if current.record.request_mismatch(&request_binding).is_some()
        || request
            .detected_release
            .and_then(|r| r.target_version)
            .as_deref()
            != Some(p.native.version.as_str())
    {
        return Err("native standing request/Event/P/O/version differs".into());
    }
    let source = host::Tree::read(api, &p.repository, &build.source_commit)?;
    host::ancestor(api, &p.repository, &build.source_commit, &b.packet_head)?;
    bundle::approve_inputs(root, p, &b.canonical, &source, build)?;
    view.verify()
}
fn preview(
    root: &Path,
    graph: &DependencyGraph,
    environment: &Environment,
    toolchain: &FileRef,
    prefix: &str,
) -> Result<Build, String> {
    let mut c = std::process::Command::new("git");
    let commit = String::from_utf8(crate::depth_ci::output(
        c.arg("--no-replace-objects")
            .arg("-C")
            .arg(root)
            .args(["rev-parse", "HEAD"]),
    )?)
    .map_err(|_| "invalid source identity")?
    .trim()
    .to_owned();
    let source = crate::runtime_advertising::source::Source::tip(root, &commit)?;
    Ok(Build {
        source_commit: commit,
        source_tree: source.tree()?,
        host_target: environment.target.clone(),
        toolchain: toolchain.clone(),
        features: environment.features.clone(),
        environment: crate::depth_ci::reference(
            format!("{prefix}/environment.json"),
            &serde_json::to_vec(environment).map_err(|_| "invalid environment")?,
        ),
        dependency_graph: crate::depth_ci::reference(
            format!("{prefix}/graph.json"),
            &serde_json::to_vec(graph).map_err(|_| "invalid graph")?,
        ),
        covered_inputs: graph.inputs.clone(),
        absent_inputs: [
            ".cargo/config",
            ".cargo/config.toml",
            "rust-toolchain",
            "rust-toolchain.toml",
            "crates/xtask/build.rs",
        ]
        .map(str::to_owned)
        .to_vec(),
    })
}
pub(super) fn collect(
    root: &Path,
    args: CollectArgs,
    p: &Profile,
    api: &impl StartApi,
) -> Result<(), String> {
    collect_with(
        root,
        args,
        p,
        api,
        |root, prefix| {
            crate::depth_ci::native::build(root, prefix, &p.graph, &p.environment, &p.toolchain)
        },
        execution::run,
    )
}
pub(in crate::depth_evidence::verdict) fn collect_with(
    root: &Path,
    args: CollectArgs,
    p: &Profile,
    api: &impl StartApi,
    build: impl FnOnce(&Path, &str) -> Result<crate::depth_ci::native::Built, String>,
    run: impl FnOnce(
        &Path,
        &CollectArgs,
        &Path,
        &crate::depth_ci::native::Built,
        &form::Plan,
    ) -> Result<execution::Execution, String>,
) -> Result<(), String> {
    if !args.live || !args.register {
        return Err(
            "native execution and start registration require explicit --live --register".into(),
        );
    }
    let root = root
        .canonicalize()
        .map_err(|_| "native workspace unavailable")?;
    let prefix = directory(p, &args.attempt)?;
    let mut view = GitInputs::new(&root)?;
    let current = current(&mut view, p)?;
    let binding = host::bind(api, p, &current)?;
    let mappings = mapped(&mut view, p)?;
    let intent = view.required(INTENT)?;
    if crate::depth_ci::reference("", &intent).sha256 != p.context_intent_sha256 {
        return Err("native intent differs from fixed profile".into());
    }
    let binary = args
        .binary
        .canonicalize()
        .map_err(|_| "native executable unavailable")?;
    if execution::fingerprint(&binary)? != p.native.executable_sha256 {
        return Err("native executable differs from pinned acquisition".into());
    }
    // Merely measuring executable bytes is not a native version/config probe.
    // The build uses a fresh closed environment, never the caller's native credentials.
    let preview = preview(&root, &p.graph, &p.environment, &p.toolchain, &prefix)?;
    source_check(&root, api, p, &binding, &current, &mut view, &preview)?;
    let built = build(&root, &prefix)?;
    if built.build != preview {
        return Err("measured native build differs from pre-build source closure".into());
    }
    source_check(&root, api, p, &binding, &current, &mut view, &built.build)?;
    let requested = execution::requested(&built.executable_sha256);
    if requested.iter().cloned().collect::<BTreeSet<_>>()
        != p.specs.values().flatten().map(|a| a.case.clone()).collect()
    {
        return Err("compiled native case population differs from reviewed assertions".into());
    }
    let mut retained = Retained::reserve(&root, &prefix)?;
    for (file, bytes) in &built.payloads {
        if file.path.starts_with(&format!("{prefix}/")) {
            retained.put(
                file.path
                    .rsplit('/')
                    .next()
                    .ok_or("invalid build payload")?,
                bytes,
            )?;
        } else {
            retained.include(file, bytes)?;
        }
    }
    let context = retained.put("intent.json", &intent)?;
    let plan = form::Plan {
        schema_version: 1,
        scope: current.current.clone(),
        packet_head: binding.packet_head.clone(),
        build: built.build.clone(),
        native: p.native.clone(),
        context,
        collector: p.collector.clone(),
        requested,
    };
    let plan_bytes = canonical(&plan)?;
    form::Plan::parse(text(&plan_bytes)?)?;
    let plan_ref = retained.put("plan.json", &plan_bytes)?;
    retained.verify()?;
    if host::bind(api, p, &current)? != binding {
        return Err("native packet/source binding changed before registration".into());
    }
    let start = form::Start {
        schema_version: 1,
        profile: "isolated-codex-native-v1".into(),
        repository: p.repository.clone(),
        pull_request: binding.pr,
        packet_head: binding.packet_head.clone(),
        scope: plan.scope.clone(),
        plan: plan_ref.clone(),
    };
    let registration = register(api, p, binding.pr, &start)?;
    retained.registration(&canonical(&registration.reference())?)?;
    // All post-registration failures leave the registered plan intact. Never create another attempt.
    let ready = (|| -> Result<(), String> {
        source_check(&root, api, p, &binding, &current, &mut view, &built.build)?;
        retained.verify()?;
        if host::bind(api, p, &current)? != binding
            || execution::fingerprint(&binary)? != p.native.executable_sha256
            || mappings.iter().any(|m| {
                m.executor_identity
                    != format!("github:{}:{}", registration.login, registration.author_id)
            })
        {
            return Err("registered executor, packet or executable differs".into());
        }
        Ok(())
    })();
    let result = ready.and_then(|()| run(&root, &args, &binary, &built, &plan));
    let run = match result {
        Ok(run) => run,
        Err(_) => execution::incomplete(&plan),
    };
    let observed_context = retained.put("observed-context.json", &canonical(&run.context)?)?;
    let raw = retained.put("observations.json", &canonical(&run.observations)?)?;
    let mut claims = Vec::new();
    for (i, m) in mappings.into_iter().enumerate() {
        let names: BTreeSet<_> = m.assertions.iter().map(|a| a.case.clone()).collect();
        let cases = Cases {
            requested: plan
                .requested
                .iter()
                .filter(|c| names.contains(*c))
                .cloned()
                .collect(),
            executed: run
                .cases
                .executed
                .iter()
                .filter(|r| names.contains(&r.case))
                .cloned()
                .collect(),
            raw_output: raw.clone(),
        };
        let mut scope = plan.scope.clone();
        scope.obligation = m.obligation.clone();
        let claim = form::Claim {
            scope,
            build: plan.build.clone(),
            provenance: vec![Provenance {
                kind: crate::depth_evidence::ProvenanceKind::Binary,
                origin: "pinned native acquisition".into(),
                file: p.native.acquisition.clone(),
            }],
            cases,
            mapping: m,
        };
        let bytes = canonical(&claim)?;
        form::Claim::parse(text(&bytes)?)?;
        claims.push(retained.put(&format!("claim-{i}.json"), &bytes)?);
    }
    let index = retained.index(plan_ref, observed_context, claims)?;
    println!("Native attempt retained at {}. No completion statement was posted; evidence remains unverified.",index.path);
    // Closing drift is a retained refusal, never a discarded attempt or a green outcome.
    retained.verify()?;
    source_check(&root, api, p, &binding, &current, &mut view, &built.build)?;
    if host::bind(api, p, &current)? != binding
        || execution::fingerprint(&binary)? != p.native.executable_sha256
        || !run.complete
    {
        return Err("native attempt is incomplete, failed or changed; retained registration and outcomes require review".into());
    }
    Ok(())
}
#[cfg(test)]
mod tests;

pub(in crate::depth_evidence::verdict) fn prepare_finish(
    root: &Path,
    args: form::PrepareFinishArgs,
    p: &Profile,
    api: &impl host::Api,
) -> Result<String, String> {
    crate::depth_evidence::validate::git_identity(&args.evidence_commit)?;
    let prefix = directory(p, &args.attempt)?;
    let mut view = GitInputs::new(root)?;
    let current = current(&mut view, p)?;
    let binding = host::bind(api, p, &current)?;
    let tree = host::Tree::read(api, &p.repository, &args.evidence_commit)?;
    let bytes = tree.bytes(root, &format!("{prefix}/bundle.json"))?;
    let index = form::Bundle::parse(text(&bytes)?)?;
    if bytes != canonical(&index)? {
        return Err("native bundle is not canonical".into());
    }
    let bundle = crate::depth_ci::reference(format!("{prefix}/bundle.json"), &bytes);
    let registration: form::CommentRef = crate::depth_evidence::payload::parse(text(
        &tree.bytes(root, &format!("{prefix}/registration.json"))?,
    )?)?;
    let bound = BTreeSet::from([registration.id]);
    let comments = host::comments(api, p, binding.pr, &bound)?;
    let own = comments
        .iter()
        .find(|c| c.id == registration.id && c.authorized && c.reference() == registration)
        .ok_or("registered execution author/body is unavailable")?;
    let start = form::Start::parse(
        population::block(&own.body, "depth-native-start-v1")?.ok_or("missing native start")?,
    )?;
    if start.repository != p.repository
        || start.pull_request != binding.pr
        || start.profile != "isolated-codex-native-v1"
        || !population::same_generation(&start.scope, &current.current)
    {
        return Err("completion generation/packet differs".into());
    }
    let registrations = population::snapshot(
        &comments,
        &start.scope,
        &p.repository,
        binding.pr,
        &registration,
    )?;
    let finish = form::Finish {
        schema_version: 1,
        registration,
        evidence_commit: args.evidence_commit,
        bundle,
        executor_identity: format!("github:{}:{}", own.login, own.author_id),
        population_through_id: comments
            .iter()
            .map(|c| c.id)
            .max()
            .ok_or("empty native attempt snapshot")?,
        registrations,
        execution_attested: false,
        outcomes_attested: false,
        registration_order_attested: false,
        completeness_attested: false,
    };
    form::Finish::parse(text(&canonical(&finish)?)?)?;
    // Full immutable bundle/source/claim checks are shared with admission, but no human
    // attestation, adoption, receipt or opaque verdict is manufactured here.
    bundle::resolve_content(root, api, p, &binding, &start, &finish, &current)?;
    view.verify()?;
    if host::bind(api, p, &current)? != binding
        || host::comments(api, p, binding.pr, &bound)? != comments
    {
        return Err("native finish snapshot changed".into());
    }
    Ok(format!("Unposted template. Review execution, outcomes, ordering and completeness before explicitly attesting; this command posts nothing.\n\n{}",fence("depth-native-finish-v1",&finish)?))
}
