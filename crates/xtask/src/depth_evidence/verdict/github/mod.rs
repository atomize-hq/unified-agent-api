//! The only installed host profile: canonical GitHub CI, isolated source-contract proof.
use super::*;
use crate::depth_ci::profile::Profile;
use crate::{
    depth_ci::{self, Witness},
    depth_record::Event,
};
use serde_json::{json, Value};
mod adoption;
mod artifact;
mod profile;
mod transport;
use transport::*;

#[derive(Debug)]
pub(super) struct AuthenticatedRun {
    pub receipt: EvidenceReceipt,
    pub witness: Witness,
}
#[derive(Debug)]
pub(super) struct Resolved {
    pub profile: Profile,
    pub original: DepthRecord,
    pub definition: String,
    pub runs: Vec<AuthenticatedRun>,
}

#[cfg(test)]
fn resolve(api: &impl Api, root: &Path) -> Result<Resolved, String> {
    resolve_profile(api, root, Profile::Channel)
}
fn resolve_profile(api: &impl Api, root: &Path, p: Profile) -> Result<Resolved, String> {
    let mut view = GitInputs::new(root)?;
    let original =
        p.record(std::str::from_utf8(&view.required(p.fixture())?).map_err(|e| e.to_string())?)?;
    let definition = depth_ci::reference("", &view.required(p.contract())?).sha256;
    let anchor_value = api.json(&endpoint("git/ref/heads/staging"))?;
    let anchor = sha(required_string(&anchor_value["object"], "sha")?)?;
    let anchor_source = source(root, anchor)?;
    // Include all compiled producer code and its dependencies, not only the named collector.
    let mut paths = view.files_under("crates")?;
    let anchor_paths = anchor_source.committed_files("crates")?;
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
        _ => return Err("unsupported initial CI lifecycle".into()),
    };
    let pulls = list(api, &format!("commits/{request}/pulls"), None)?;
    let pulls = pulls
        .iter()
        .filter(|p| {
            p["base"]["repo"]["full_name"] == depth_ci::REPOSITORY && p["base"]["ref"] == "staging"
        })
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
    for run in &inventory {
        if run["path"] != profile::WORKFLOW {
            return Err("workflow population path mismatch".into());
        }
        if run["head_repository"]["full_name"] != depth_ci::REPOSITORY {
            continue;
        }
        let head = source(root, sha(required_string(run, "head_sha")?)?)?;
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
        profile::approved(root, head_sha, head_sha, anchor)?;
        let id = run["id"].as_u64().ok_or("missing run id")?;
        let count = run["run_attempt"]
            .as_u64()
            .filter(|n| *n > 0 && *n <= 100)
            .ok_or("unsupported attempt population")?;
        for attempt in 1..=count {
            let attempt_run =
                api.json(&endpoint(&format!("actions/runs/{id}/attempts/{attempt}")))?;
            if attempt_run["id"] != id
                || attempt_run["run_attempt"] != attempt
                || attempt_run["head_sha"] != run["head_sha"]
                || attempt_run["path"] != profile::WORKFLOW
                || attempt_run["status"] != "completed"
            {
                return Err("missing/unstable/nonterminal applicable attempt".into());
            }
            let w = artifact::witness(api, &decoder, &attempt_run, attempt, p)?;
            let built_original =
                profile::validate_witness(root, &w, &attempt_run, attempt, anchor, p)?;
            if built_original != original {
                return Err(
                    "actual checkout disagrees with independently applicable generation".into(),
                );
            }
            let adoptions = adoption::resolve(api, root, pr, &w, p)?;
            if adoptions.is_empty() {
                return Err(format!("missing exact authenticated adequacy adoption on PR #{pr}; proposal: ```depth-adequacy-v1\n{}\n```",serde_json::to_string_pretty(&adoption::proposal_for(root,&w,"REPLACE_WITH_REVIEWER_INVOCATION",p)?).map_err(|e|e.to_string())?));
            }
            for (mapping, adoption) in adoptions {
                let scope = profile::scope_for(&built_original, p, mapping.obligation.clone());
                let mapped: std::collections::BTreeSet<_> =
                    mapping.assertions.iter().map(|a| a.case.clone()).collect();
                let mut cases = w.cases.clone();
                cases.requested.retain(|c| mapped.contains(c));
                cases.executed.retain(|c| mapped.contains(&c.case));
                let built = source(root, &w.build.source_commit)?;
                let receipt = EvidenceReceipt {
                    schema_version: 1,
                    scope,
                    producer: Producer {
                        kind: ProducerKind::GithubActions,
                        repository: depth_ci::REPOSITORY.into(),
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
    // Do not allow an attempt created during discovery to disappear from the population.
    if inventory != list(api, &query, Some("workflow_runs"))?
        || anchor_value != api.json(&endpoint("git/ref/heads/staging"))?
    {
        return Err("host population or approved anchor changed during read".into());
    }
    view.verify()?;
    Ok(Resolved {
        profile: p,
        original,
        definition,
        runs,
    })
}

fn receipt_ref(r: &EvidenceReceipt) -> Result<FileRef, String> {
    let p = if r.scope.version == "0.0.0" {
        Profile::Channel
    } else {
        Profile::Core
    };
    let suffix = if p == Profile::Channel {
        String::new()
    } else {
        format!(
            "/{}",
            r.scope
                .obligation
                .id
                .map(|id| serde_json::to_value(id)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_owned())
                .unwrap_or_else(|| "M".into())
        )
    };
    Ok(depth_ci::reference(
        format!(
            "{}/{}-{}{}/receipt.json",
            p.evidence(),
            r.producer.run_id,
            r.producer.run_attempt,
            suffix
        ),
        &serde_json::to_vec(r).map_err(|e| e.to_string())?,
    ))
}
impl Resolved {
    fn population(&self, current: &Evaluation) -> Result<Option<Population>, String> {
        if profile::scope_for(
            &self.original,
            self.profile,
            current.current.obligation.clone(),
        ) != current.current
            || !self
                .profile
                .obligations(&self.original)
                .contains(&current.current.obligation)
            || self.definition != current.definition_sha256
            || current.record.event != self.original.event
            || current.record.policy != self.original.policy
            || current.record.obligations != self.original.obligations
        {
            return Ok(None);
        }
        let applicable: Vec<_> = self
            .runs
            .iter()
            .filter(|r| r.receipt.scope.obligation == current.current.obligation)
            .collect();
        let Some(first) = applicable.first() else {
            return Ok(None);
        };
        let first = &first.witness;
        if applicable.iter().any(|r| {
            r.witness.environment != first.environment
                || r.witness.build.toolchain != first.build.toolchain
        }) {
            return Ok(None);
        }
        Ok(Some(Population {
            current: profile::scope_for(
                &self.original,
                self.profile,
                current.current.obligation.clone(),
            ),
            definition_sha256: self.definition.clone(),
            environment: first.environment.clone(),
            toolchain: first.build.toolchain.clone(),
            runs: applicable
                .into_iter()
                .map(|r| {
                    Ok(Run {
                        evidence: receipt_ref(&r.receipt)?,
                        scope: r.receipt.scope.clone(),
                        definition_sha256: self.definition.clone(),
                        producer: r.receipt.producer.clone(),
                        build: r.receipt.build.clone(),
                        graph: r.witness.graph.clone(),
                        environment: r.witness.environment.clone(),
                        provenance: r.receipt.provenance.clone(),
                        cases: r.receipt.cases.clone(),
                        adoption: Some(Adoption {
                            endorsement: r.receipt.adequacy.endorsement.clone(),
                            reviewer: r.receipt.adequacy.mapping.reviewer_identity.clone(),
                            executor: r.receipt.adequacy.mapping.executor_identity.clone(),
                        }),
                        hosted: Some(r.receipt.clone()),
                    })
                })
                .collect::<Result<_, String>>()?,
        }))
    }
}
pub(super) struct Adapter<'a> {
    pub root: &'a Path,
}
impl Host for Adapter<'_> {
    fn population(&self, current: &Evaluation) -> Result<Option<Population>, String> {
        if current.current.enrollment_origin != EnrollmentOrigin::Isolated
            || current.current.agent_id != "codex"
        {
            return Ok(None);
        }
        let p = match current.current.version.as_str() {
            "0.0.0" => Profile::Channel,
            "0.156.1" => Profile::Core,
            _ => return Ok(None),
        };
        // Read failures are absence of authenticated authority, never successful evidence.
        match resolve_profile(&Github, self.root, p) {
            Ok(resolved) => resolved.population(current),
            Err(_) => Ok(None),
        }
    }
}

/// Read-only live proof in a disposable checkout. It materializes no authoritative repository
/// records, modifies no packet/host policy, and cannot enroll a production version.
pub fn prove(root: &Path) -> Result<String, String> {
    prove_profile(root, &Github, Profile::Channel)
}
#[cfg(test)]
fn prove_with(root: &Path, api: &impl Api) -> Result<String, String> {
    prove_profile(root, api, Profile::Channel)
}
pub fn prove_core(root: &Path) -> Result<String, String> {
    prove_profile(root, &Github, Profile::Core)
}
fn prove_profile(root: &Path, api: &impl Api, p: Profile) -> Result<String, String> {
    let resolved = resolve_profile(api, root, p)?;
    let scratch = tempfile::tempdir().map_err(|e| e.to_string())?;
    let commit = resolved.runs[0].witness.build.source_commit.clone();
    crate::depth_ci::output(
        std::process::Command::new("git")
            .args(["clone", "--quiet", "--no-hardlinks", "--no-checkout"])
            .arg(root)
            .arg(scratch.path()),
    )?;
    crate::depth_ci::output(
        std::process::Command::new("git")
            .arg("-C")
            .arg(scratch.path())
            .args(["checkout", "--quiet", "--detach", &commit]),
    )?;
    // All run/endorsement objects used by currentness must exist in the disposable checkout.
    for run in &resolved.runs {
        source(scratch.path(), &run.receipt.build.source_commit)?;
        source(
            scratch.path(),
            &run.receipt.adequacy.endorsement.source_commit,
        )?;
    }
    let jail = crate::workspace_mutation::WorkspacePathJail::new(scratch.path())
        .map_err(|e| e.to_string())?;
    let mut selected = std::collections::BTreeMap::new();
    for run in &resolved.runs {
        let r = receipt_ref(&run.receipt)?;
        selected
            .entry(run.receipt.scope.obligation.clone())
            .or_insert(r.clone());
        let mut files = profile::payloads(&run.witness)?;
        files.push((
            r,
            serde_json::to_vec(&run.receipt).map_err(|e| e.to_string())?,
        ));
        for (reference, bytes) in files {
            super::super::validate::file(&reference)?;
            let path = jail
                .resolve(Path::new(&reference.path))
                .map_err(|e| e.to_string())?;
            std::fs::create_dir_all(path.parent().ok_or("missing proof parent")?)
                .map_err(|e| e.to_string())?;
            if path.exists() && std::fs::read(&path).map_err(|e| e.to_string())? != bytes {
                return Err("conflicting hosted payload path".into());
            }
            std::fs::write(path, bytes).map_err(|e| e.to_string())?;
        }
    }
    crate::depth_ci::output(
        std::process::Command::new("git")
            .arg("-C")
            .arg(scratch.path())
            .args(["add", "--", &p.evidence()]),
    )?;
    let mut record = resolved.original.clone();
    let scope = profile::scope_for(&record, p, p.obligations(&record).remove(0));
    let mut e = crate::depth_record::ExecutionPart {
        identity: String::new(),
        policy_identity: scope.policy_identity.clone(),
        obligations_identity: scope.obligations_identity.clone(),
        results: selected
            .iter()
            .map(
                |(obligation, evidence)| crate::depth_record::ObligationResult {
                    obligation: obligation.clone(),
                    result: DepthResult::Verified,
                    rationale: None,
                    evidence: vec![evidence.clone()],
                },
            )
            .collect(),
        reuse_bindings: vec![],
        debt_grants: vec![],
        debt_retired: vec![],
    };
    // Annex B derived arrays sort by canonical serialized entries, not Rust enum order.
    e.results
        .sort_by_key(|r| serde_json::to_vec(&serde_json::to_value(r).unwrap()).unwrap());
    e.identity = crate::depth_record::part_identity(&e)?;
    record.execution = Some(e);
    let (results, decisions) = resolved.decisions(scratch.path(), &record)?;
    let verdicts=selected.keys().map(|obligation|json!({"obligation":obligation,"result":results.result(&record,obligation)})).collect::<Vec<_>>();
    let verdict = Verdict {
        result: if verdicts.iter().all(|v| v["result"] == "verified") {
            DepthResult::Verified
        } else if verdicts.iter().any(|v| v["result"] == "failed") {
            DepthResult::Failed
        } else {
            DepthResult::Unverified
        },
        reason: decisions
            .values()
            .map(|v| v.reason.as_str())
            .collect::<Vec<_>>()
            .join("; "),
    };
    let facts = crate::support_matrix::depth::derive_evaluated_depth_facts(
        Some(&record),
        true,
        depth_ci::TARGET,
        &|path| {
            std::fs::read(scratch.path().join(path))
                .ok()
                .map(|b| depth_ci::reference(path, &b).sha256)
        },
        &results,
    )?;
    let row = crate::support_matrix::SupportRow {
        agent: "codex".into(),
        version: p.version().into(),
        target: depth_ci::TARGET.into(),
        manifest_support: crate::support_matrix::ManifestSupportState::Unsupported,
        backend_support: crate::support_matrix::BackendSupportState::Partial,
        uaa_support: crate::support_matrix::UaaSupportState::Partial,
        pointer_promotion: crate::support_matrix::PointerPromotionState::None,
        evidence_notes: vec![
            "Isolated source-contract diagnostic; no CLI release or production enrollment".into(),
        ],
        depth: facts.clone(),
    };
    let publication = crate::support_matrix::render_publication_bundle(&[row], String::new())?;
    serde_json::to_string_pretty(&json!({"origin":"isolated","obligations":verdicts,"result":verdict.result,"reason":verdict.reason,"publisher":facts,"publication_json":serde_json::from_str::<Value>(&publication.json).map_err(|e|e.to_string())?,"publication_markdown":publication.markdown,"production_enrollment":false})).map_err(|e|e.to_string())
}

#[cfg(test)]
mod tests;

impl Host for Resolved {
    fn population(&self, current: &Evaluation) -> Result<Option<Population>, String> {
        self.population(current)
    }
}
impl Resolved {
    fn evaluate_record(
        &self,
        root: &Path,
        record: &DepthRecord,
    ) -> Result<super::super::EvidenceVerdicts, String> {
        self.decisions(root, record).map(|(results, _)| results)
    }
    fn decisions(
        &self,
        root: &Path,
        record: &DepthRecord,
    ) -> Result<
        (
            super::super::EvidenceVerdicts,
            std::collections::BTreeMap<ObligationRef, Verdict>,
        ),
        String,
    > {
        let first = self.runs.first().ok_or("empty authenticated population")?;
        let mut results = std::collections::BTreeMap::new();
        let mut decisions = std::collections::BTreeMap::new();
        for obligation in self.profile.obligations(&self.original) {
            let current = Evaluation {
                current: profile::scope_for(&self.original, self.profile, obligation.clone()),
                definition_sha256: self.definition.clone(),
                record: record.clone(),
                environment: first.witness.environment.clone(),
                toolchain: first.witness.build.toolchain.clone(),
            };
            let row = record
                .execution
                .as_ref()
                .and_then(|e| e.results.iter().find(|r| r.obligation == obligation));
            let mut outcomes = vec![];
            if let Some(row) = row {
                for selected in &row.evidence {
                    outcomes.push(evaluate(root, selected, &current, self)?);
                }
            }
            let result =
                if outcomes.is_empty() || outcomes.iter().any(|r| r.result != outcomes[0].result) {
                    DepthResult::Unverified
                } else {
                    outcomes[0].result
                };
            let reason = if outcomes.is_empty() {
                "missing selected evidence for due obligation".into()
            } else if outcomes.iter().any(|r| r.result != outcomes[0].result) {
                "contradictory selected evidence results".into()
            } else {
                outcomes[0].reason.clone()
            };
            decisions.insert(obligation.clone(), Verdict { result, reason });
            results.insert(obligation, result);
        }
        Ok((
            super::super::results::evaluated_many(record, results)?,
            decisions,
        ))
    }
}
/// Read-only bridge for the fixed isolated CORE profile. Production consumers never call this.
pub(in crate::depth_evidence) fn isolated_record_results(
    root: &Path,
    record: &DepthRecord,
) -> Result<super::super::EvidenceVerdicts, String> {
    match resolve_profile(&Github, root, Profile::Core) {
        Ok(resolved) => resolved.evaluate_record(root, record),
        Err(_) => Ok(super::super::EvidenceVerdicts::default()),
    }
}
