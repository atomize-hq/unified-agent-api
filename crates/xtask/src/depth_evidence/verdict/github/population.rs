//! Independently discover every matching Event/P/O run and attempt before inspecting outcomes.
use super::*;

pub(super) fn resolve_profile_inputs(
    api: &impl Api,
    root: &Path,
    p: Profile,
    host: Option<&crate::isolated_maintenance::EvidenceHost>,
    borrowed: Option<&mut GitInputs>,
) -> Result<Resolved, String> {
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
            Some(h) if !h.same_repository(&api.json(&api.identity_endpoint())?) => {
                Err("isolated repository id, owner id or name differs from the binding".into())
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
    let original =
        p.record(std::str::from_utf8(&view.required(p.fixture())?).map_err(|e| e.to_string())?)?;
    depth_ci::onboarding::check_inputs(p, &original, |path| {
        String::from_utf8(view.required(path)?).map_err(|e| e.to_string())
    })?;
    let definition = depth_ci::reference("", &view.required(p.contract())?).sha256;
    let anchor_value = api.json(&api.endpoint("git/ref/heads/staging"))?;
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
    if pulls.len() != 1 {
        return Err("generation has no unique authenticated packet PR".into());
    }
    let pr = pulls[0]["number"]
        .as_u64()
        .ok_or("missing packet PR number")?;
    let query = format!("actions/workflows/ci.yml/runs?created=%3E%3D{date}");
    let inventory = list(api, &query, Some("workflow_runs"))?;
    let mut runs = vec![];
    let mut progression = progression::Onboarding::default();
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
        let Some(raw) = head.optional(p.fixture())? else {
            continue;
        };
        let candidate = p.record(&raw)?;
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
        // Approve the API-derived execution head before trusting any witness source claims.
        let head_sha = sha(required_string(run, "head_sha")?)?;
        progression.observe(root, head_sha, &head, p, &candidate)?;
        profile::approved(root, repository, head_sha, head_sha, anchor, p)?;
        let id = run["id"].as_u64().ok_or("missing run id")?;
        let count = run["run_attempt"]
            .as_u64()
            .filter(|n| *n > 0 && *n <= 100)
            .ok_or("unsupported attempt population")?;
        for attempt in 1..=count {
            let attempt_run =
                api.json(&api.endpoint(&format!("actions/runs/{id}/attempts/{attempt}")))?;
            if attempt_run["id"] != id
                || attempt_run["run_attempt"] != attempt
                || attempt_run["head_sha"] != run["head_sha"]
                || attempt_run["path"] != profile::WORKFLOW
                // IR-P3: the enclosing workflow may still run; the producer job must be terminal,
                // which the artifact binding checks for every profile.
                || (!ir_p3 && attempt_run["status"] != "completed")
            {
                return Err("missing/unstable/nonterminal applicable attempt".into());
            }
            let w = artifact::witness(api, &decoder, &attempt_run, attempt, p)?;
            let built_original =
                profile::validate_witness(root, repository, &w, &attempt_run, attempt, anchor, p)?;
            if built_original != original {
                return Err(
                    "actual checkout disagrees with independently applicable generation".into(),
                );
            }
            if p == Profile::SyntheticOnboarding && w.build.source_commit != head_sha {
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
    }
    if runs.is_empty() {
        return Err("no applicable producer attempts".into());
    }
    check_stable(api, &query, &inventory, &anchor_value, ir_p3)?;
    same_host()?;
    view.verify()?;
    Ok(Resolved {
        profile: p,
        original,
        definition,
        runs,
    })
}

/// Recheck discovery facts independently of already authenticated immutable source/artifact bytes.
pub(super) fn check_stable(
    api: &impl Api,
    query: &str,
    inventory: &[Value],
    anchor_value: &Value,
    ir_p3: bool,
) -> Result<(), String> {
    // Do not allow an attempt created during discovery to disappear from the population. Under
    // IR-P3 unrelated workflow progress may continue, so only run identities must hold still.
    let identities = |rows: &[Value]| -> Vec<Value> {
        rows.iter()
            .map(|r| {
                if ir_p3 {
                    json!([
                        r["id"],
                        r["run_attempt"],
                        r["head_sha"],
                        r["path"],
                        r["head_repository"]["full_name"],
                        r["event"]
                    ])
                } else {
                    r.clone()
                }
            })
            .collect()
    };
    if identities(inventory) != identities(&list(api, query, Some("workflow_runs"))?)
        || *anchor_value != api.json(&api.endpoint("git/ref/heads/staging"))?
    {
        return Err("host population or approved anchor changed during read".into());
    }
    Ok(())
}
