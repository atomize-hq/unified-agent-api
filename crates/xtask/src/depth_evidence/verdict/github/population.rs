//! Independently discover every matching Event/P/O run and attempt before inspecting outcomes.
use super::*;
use crate::depth_acceptance::admission::observations;
use crate::depth_ci::profile::GenerationResolver;

pub(super) fn resolve_profile_inputs(
    api: &impl Api,
    root: &Path,
    p: Profile,
    host: Option<&crate::isolated_maintenance::EvidenceHost>,
    borrowed: Option<&mut GitInputs>,
) -> Result<Resolved, String> {
    resolve_generation_inputs(api, root, host, borrowed, &p)
}

/// Discovery and the checkpoint join consume resolved bindings from this single interface.
pub(super) fn resolve_generation_inputs(
    api: &impl Api,
    root: &Path,
    host: Option<&crate::isolated_maintenance::EvidenceHost>,
    borrowed: Option<&mut GitInputs>,
    resolver: &impl GenerationResolver,
) -> Result<Resolved, String> {
    let p = resolver.profile();
    let observed = super::transport_impl::Observed(api);
    let api = &observed;
    let ir_p3 = match (p, host) {
        (Profile::Synthetic | Profile::SyntheticOnboarding, Some(h))
            if api.repository() == h.repository() && h.repository() != depth_ci::REPOSITORY =>
        {
            true
        }
        (Profile::Synthetic | Profile::SyntheticOnboarding, _) => {
            return Err(
                "synthetic-wrapper-v1 requires an authenticated isolated evidence host".into(),
            )
        }
        (_, None) if api.repository() == depth_ci::REPOSITORY => false,
        _ => return Err("channel/CORE evidence is bound to the canonical repository".into()),
    };
    let repository = api.repository();
    // IR-P1 authenticated ids, not the name alone, before and after discovery.
    let same_host = || -> Result<(), String> {
        match host {
            Some(h) => {
                let value = api.json(&api.identity_endpoint())?;
                observations::fact(
                    &api.identity_endpoint(),
                    "repository",
                    &json!([value["id"], value["owner"]["id"], value["full_name"]]),
                );
                if !h.same_repository(&value) {
                    return Err(
                        "isolated repository id, owner id or name differs from the binding".into(),
                    );
                }
                Ok(())
            }
            _ => Ok(()),
        }
    };
    same_host()?;
    let mut owned;
    let view = match borrowed {
        Some(view) => view,
        None => {
            owned = GitInputs::new(root)?;
            &mut owned
        }
    };
    let original = resolver.current_generation(view)?.record;
    depth_ci::onboarding::check_inputs(p, &original, |path| {
        String::from_utf8(view.required(path)?).map_err(|e| e.to_string())
    })?;
    let definition = depth_ci::reference("", &view.required(p.contract())?).sha256;
    let anchor_value = api.json(&api.endpoint("git/ref/heads/staging"))?;
    observations::fact(
        &api.endpoint("git/ref/heads/staging"),
        "anchor",
        &anchor_value["object"]["sha"],
    );
    let anchor = sha(required_string(&anchor_value["object"], "sha")?)?;
    if host.is_some_and(|h| h.tip() != anchor) {
        return Err("isolated protected staging differs from the authenticated binding".into());
    }
    let anchor_source = source(root, repository, anchor)?;
    // Include all compiled producer code and its dependencies, not only the named collector.
    let mut paths = view.files_under("crates")?;
    let mut anchor_paths = anchor_source.committed_files("crates")?;
    if let Some(scope) = p.requirements() {
        paths.extend(view.files_under(scope)?);
        anchor_paths.extend(anchor_source.committed_files(scope)?);
    }
    if paths != anchor_paths {
        return Err("current producer inventory differs from approved staging".into());
    }
    paths.extend(
        [
            "Cargo.toml",
            "Cargo.lock",
            profile::WORKFLOW,
            depth_ci::ARCHIVE,
        ]
        .map(str::to_owned),
    );
    for path in paths {
        if anchor_source.optional_bytes(&path)?.as_deref() != Some(view.required(&path)?.as_slice())
            || anchor_source.executable(&path)? != view.executable(&path)?
        {
            return Err("current producer is not the reviewed staging profile".into());
        }
    }
    for path in profile::OPTIONAL {
        if view.optional(path)?.is_some() || anchor_source.optional_bytes(path)?.is_some() {
            return Err(
                "initial producer requires absent root Cargo/toolchain configuration".into(),
            );
        }
    }
    let decoder = anchor_source.read(depth_ci::ARCHIVE)?;
    let (request, date) = match &original.event {
        Event::Maintenance {
            request_commit,
            request_recorded_at,
            ..
        } => (
            sha(request_commit)?,
            request_recorded_at
                .get(..10)
                .ok_or("missing generation date")?,
        ),
        Event::Onboarding {
            approval_commit,
            approval_recorded_at,
        } if p == Profile::SyntheticOnboarding => (
            sha(approval_commit)?,
            approval_recorded_at
                .get(..10)
                .ok_or("missing generation date")?,
        ),
        _ => return Err("unsupported initial CI lifecycle".into()),
    };
    let pulls = list(api, &format!("commits/{request}/pulls"), None)?;
    let pulls = pulls
        .iter()
        .filter(|p| p["base"]["repo"]["full_name"] == repository && p["base"]["ref"] == "staging")
        .collect::<Vec<_>>();
    observations::population(
        &api.endpoint(&format!("commits/{request}/pulls")),
        "packet_pr",
        pulls
            .iter()
            .map(|p| {
                json!([
                    p["number"],
                    p["base"]["repo"]["full_name"],
                    p["base"]["ref"]
                ])
            })
            .collect(),
    );
    if pulls.len() != 1 {
        return Err("generation has no unique authenticated packet PR".into());
    }
    let pr = pulls[0]["number"]
        .as_u64()
        .ok_or("missing packet PR number")?;
    let association_pr = (p == Profile::Synthetic).then_some(pr);
    let query = format!("actions/workflows/ci.yml/runs?created=%3E%3D{date}");
    let inventory = list(api, &query, Some("workflow_runs"))?;
    let mut applicable = vec![];
    for run in &inventory {
        if run["path"] != profile::WORKFLOW {
            return Err("workflow population path mismatch".into());
        }
        if run["head_repository"]["full_name"] != repository {
            continue;
        }
        let head = source(root, repository, sha(required_string(run, "head_sha")?)?)?;
        // Applicability precedes looking for artifacts or outcomes. Missing witnesses cannot
        // erase an otherwise applicable attempt. Other generations are genuinely out of scope.
        let associated = run["pull_requests"]
            .as_array()
            .is_some_and(|pulls| pulls.iter().any(|pull| pull["number"] == pr));
        let Some(binding) =
            resolver.resolve_inputs(&mut head.clone(), Some(&original), associated)?
        else {
            if let Some(history) = host
                .map(|host| host.admission_history(root, &[]))
                .transpose()?
                .flatten()
            {
                if history.associates_generation(
                    root,
                    sha(required_string(run, "head_sha")?)?,
                    &crate::depth_acceptance::admission::Generation::from_record(&original),
                )? {
                    return Err("associated head replaced maintenance generation".into());
                }
                history.recheck()?;
            }
            continue;
        };
        let candidate = binding.record;
        if candidate.event != original.event
            || candidate.policy != original.policy
            || candidate.obligations != original.obligations
        {
            continue;
        }
        if !matches!(
            run["event"].as_str(),
            Some("pull_request" | "merge_group" | "workflow_dispatch" | "push")
        ) {
            return Err("unhandled applicable producer event".into());
        }
        applicable.push((run, head, candidate));
    }
    let applicable_runs: Vec<_> = applicable
        .iter()
        .map(|(run, _, _)| (*run).clone())
        .collect();
    observations::population(
        &api.endpoint(&query),
        "runs",
        run_facts(&applicable_runs, ir_p3, association_pr),
    );
    let candidates = applicable
        .iter()
        .map(|(run, _, _)| sha(required_string(run, "head_sha")?).map(str::to_owned))
        .collect::<Result<Vec<_>, _>>()?;
    let admissions = host
        .map(|host| host.admission_history(root, &candidates))
        .transpose()?
        .flatten();
    let mut runs = vec![];
    let mut reporting = vec![];
    let mut reporting_conflicts = BTreeSet::new();
    let mut progression = progression::Onboarding::default();
    let mut attempts = vec![];
    for (run, head, candidate) in applicable {
        // Approve the API-derived execution head before trusting any witness source claims.
        let head_sha = sha(required_string(run, "head_sha")?)?;
        if let Some(history) = &admissions {
            if p == Profile::SyntheticOnboarding {
                history.check_candidate(root, head_sha, &candidate, depth_ci::onboarding::STATE)?;
            }
        }
        progression.observe(root, head_sha, &head, p, &candidate)?;
        let id = run["id"].as_u64().ok_or("missing run id")?;
        let has_reporting = host
            .and_then(|host| host.reporting_publication())
            .map(|publication| publication.host.reporting_attempt(id, 1))
            .transpose()?
            .flatten()
            .is_some();
        if !has_reporting {
            profile::approved(root, repository, head_sha, head_sha, anchor, p)?;
        }
        let count = run["run_attempt"]
            .as_u64()
            .filter(|n| *n > 0 && *n <= 100)
            .ok_or("unsupported attempt population")?;
        for attempt in 1..=count {
            let attempt_run =
                api.json(&api.endpoint(&format!("actions/runs/{id}/attempts/{attempt}")))?;
            observations::population(
                &api.endpoint(&format!("actions/runs/{id}/attempts/{attempt}")),
                "attempt",
                run_facts(std::slice::from_ref(&attempt_run), ir_p3, association_pr),
            );
            if attempt_run["id"] != id
                || attempt_run["run_attempt"] != attempt
                || attempt_run["head_sha"] != run["head_sha"]
                || attempt_run["path"] != profile::WORKFLOW
                || attempt_run["event"] != run["event"]
                || attempt_run["workflow_id"] != run["workflow_id"]
                || attempt_run["head_repository"]["full_name"] != run["head_repository"]["full_name"]
                // IR-P3: the enclosing workflow may still run; the producer job must be terminal,
                // which the artifact binding checks for every profile.
                || (!ir_p3 && attempt_run["status"] != "completed")
            {
                return Err("missing/unstable/nonterminal applicable attempt".into());
            }
            attempts.push((candidate.clone(), attempt_run, attempt));
        }
    }
    // Exhaust the matching attempt population before opening any substantive artifacts.
    for (candidate, attempt_run, attempt) in attempts {
        let run = &attempt_run;
        let head_sha = sha(required_string(run, "head_sha")?)?;
        let id = run["id"].as_u64().ok_or("missing run id")?;
        if let Some((publication, history)) = host
            .and_then(|host| host.reporting_publication())
            .zip(admissions.as_ref())
        {
            let candidate_object =
                crate::depth_acceptance::admission::emitter::object(root, head_sha)?;
            let resolved_context = depth_ci::reporting::resolve_attempt(
                root,
                publication,
                id,
                attempt,
                &candidate_object,
                &crate::depth_ci::profile::GenerationBinding {
                    record: candidate.clone(),
                },
            )?;
            let context_fact = resolved_context
                .as_ref()
                .map(|(context, success)| context.identity().map(|identity| (identity, *success)))
                .transpose()?;
            observations::fact(
                &format!("reporting/{id}/{attempt}"),
                "context",
                &context_fact,
            );
            if let Some((context, producer_success)) = resolved_context {
                if attempt_run["event"] != context.invocation.claims.event
                    || attempt_run["workflow_id"] != context.policy.workflow_id
                    || attempt_run["head_repository"]["full_name"] != context.repository.name
                {
                    return Err("reporting population event mismatch".into());
                }
                let artifacts = list(
                    api,
                    &format!("actions/runs/{id}/artifacts"),
                    Some("artifacts"),
                )?;
                let witness_name = format!("{}-{id}-{attempt}", p.artifact());
                let execution = publication.host.reporting_execution(id, attempt);
                observations::unavailable("reporting/execution", &execution);
                let execution = execution?;
                observations::fact(
                    &format!("reporting/{id}/{attempt}"),
                    "execution",
                    &execution,
                );
                let unexpected = artifacts.iter().any(|a| a["name"] == witness_name)
                    || execution
                    || context.invocation.substantive_invoked
                    || !context.enrolled_substantive_path_not_invoked;
                if !unexpected {
                    if !context.successful() || !producer_success {
                        return Err("failed reporting producer or incomplete collection".into());
                    }
                    history.require_reporting(&context)?;
                    reporting.push(context);
                    continue;
                }
                reporting.push(context);
                reporting_conflicts.insert((id, attempt));
            }
        }
        if admissions.as_ref().is_some_and(|history| {
            history.is_request_freeze(
                head_sha,
                &crate::depth_acceptance::admission::Generation::from_record(&candidate),
            )
        }) {
            reporting_conflicts.insert((id, attempt));
        }
        profile::approved(root, repository, head_sha, head_sha, anchor, p)?;
        let w = artifact::witness(api, &decoder, &attempt_run, attempt, p)?;
        let built_original =
            profile::validate_witness(root, repository, &w, &attempt_run, attempt, anchor, p)?;
        if built_original != original {
            return Err(
                "actual checkout disagrees with independently applicable generation".into(),
            );
        }
        if p == Profile::SyntheticOnboarding && w.build.source_commit != head_sha {
            if let Some(host) = host {
                if let Some(history) =
                    host.admission_history(root, std::slice::from_ref(&w.build.source_commit))?
                {
                    history.check_candidate(
                        root,
                        &w.build.source_commit,
                        &built_original,
                        depth_ci::onboarding::STATE,
                    )?;
                    history.recheck()?;
                }
            }
            progression.observe(
                root,
                &w.build.source_commit,
                &source(root, repository, &w.build.source_commit)?,
                p,
                &built_original,
            )?;
        }
        let adoptions = adoption::resolve(api, root, pr, &w, p)?;
        if adoptions.is_empty() {
            return Err(format!("missing exact authenticated adequacy adoption on PR #{pr}; proposal: ```depth-adequacy-v1\n{}\n```",serde_json::to_string_pretty(&adoption::proposal_for(root,repository,&w,"REPLACE_WITH_REVIEWER_INVOCATION",p)?).map_err(|e|e.to_string())?));
        }
        for (mapping, adoption) in adoptions {
            let scope = profile::scope_for(&built_original, p, mapping.obligation.clone());
            let mapped: std::collections::BTreeSet<_> =
                mapping.assertions.iter().map(|a| a.case.clone()).collect();
            let mut cases = w.cases.clone();
            cases.requested.retain(|c| mapped.contains(c));
            cases.executed.retain(|c| mapped.contains(&c.case));
            let built = source(root, repository, &w.build.source_commit)?;
            let receipt = EvidenceReceipt {
                schema_version: 1,
                scope,
                producer: Producer {
                    kind: ProducerKind::GithubActions,
                    repository: repository.into(),
                    workflow_path: profile::WORKFLOW.into(),
                    workflow_commit: w.workflow_commit.clone(),
                    collector: depth_ci::reference(
                        depth_ci::COLLECTOR,
                        built.read(depth_ci::COLLECTOR)?.as_bytes(),
                    ),
                    run_id: id,
                    run_attempt: attempt,
                    job: "rust-tests".into(),
                    artifact_name: format!("{}-{id}-{attempt}", p.artifact()),
                },
                build: w.build.clone(),
                provenance: w.provenance.clone(),
                cases,
                adequacy: Adequacy {
                    mapping,
                    endorsement: adoption.endorsement,
                },
            };
            let receipt = EvidenceReceipt::parse(
                &serde_json::to_string(&receipt).map_err(|e| e.to_string())?,
            )?;
            runs.push(AuthenticatedRun {
                receipt,
                witness: w.clone(),
            });
        }
    }
    if runs.is_empty() && reporting.is_empty() {
        return Err("no applicable producer attempts".into());
    }
    check_stable(
        api,
        &query,
        &inventory,
        &anchor_value,
        ir_p3,
        association_pr,
    )?;
    same_host()?;
    if let Some(history) = admissions {
        history.recheck()?;
    }
    view.verify()?;
    Ok(Resolved {
        profile: p,
        original,
        definition,
        runs,
        reporting,
        reporting_conflicts,
    })
}

/// Recheck discovery facts independently of already authenticated immutable source/artifact bytes.
pub(super) fn check_stable(
    api: &impl Api,
    query: &str,
    inventory: &[Value],
    anchor_value: &Value,
    ir_p3: bool,
    association_pr: Option<u64>,
) -> Result<(), String> {
    // Do not allow an attempt created during discovery to disappear from the population. Under
    // IR-P3 unrelated workflow progress may continue, so only run identities must hold still.
    let current = list(api, query, Some("workflow_runs"))?;
    let anchor = api.json(&api.endpoint("git/ref/heads/staging"))?;
    observations::fact(
        &api.endpoint("git/ref/heads/staging"),
        "anchor",
        &anchor["object"]["sha"],
    );
    let stable =
        run_facts(inventory, ir_p3, association_pr) == run_facts(&current, ir_p3, association_pr);
    observations::fact(&api.endpoint(query), "population_stable", &stable);
    if !stable || anchor_value["object"]["sha"] != anchor["object"]["sha"] {
        return Err("host population or approved anchor changed during read".into());
    }
    Ok(())
}

/// The enclosing workflow may progress under IR-P3. Producer terminal state is job-owned.
fn run_facts(rows: &[Value], ir_p3: bool, association_pr: Option<u64>) -> Vec<Value> {
    let mut facts: Vec<_> = rows
        .iter()
        .map(|r| {
            json!([
                r["id"],
                r["run_attempt"],
                r["head_sha"],
                r["path"],
                r["head_repository"]["full_name"],
                r["event"],
                r["workflow_id"],
                association_pr.map(|pr| r["pull_requests"]
                    .as_array()
                    .is_some_and(|rows| rows.iter().any(|r| r["number"] == pr))),
                if ir_p3 {
                    Value::Null
                } else {
                    r["status"].clone()
                }
            ])
        })
        .collect();
    facts.sort_by_cached_key(Value::to_string);
    facts
}
