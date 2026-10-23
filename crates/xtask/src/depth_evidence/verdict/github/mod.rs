//! The only installed host profile: canonical GitHub CI, isolated source-contract proof.
use super::*;
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
    pub original: DepthRecord,
    pub definition: String,
    pub runs: Vec<AuthenticatedRun>,
}

fn resolve(api: &impl Api, root: &Path) -> Result<Resolved, String> {
    let mut view = GitInputs::new(root)?;
    let original = depth_ci::record(
        std::str::from_utf8(&view.required(depth_ci::FIXTURE)?).map_err(|e| e.to_string())?,
    )?;
    let definition = depth_ci::reference("", &view.required(depth_ci::CONTRACT)?).sha256;
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
        let Some(raw) = head.optional(depth_ci::FIXTURE)? else {
            continue;
        };
        let candidate = depth_ci::record(&raw)?;
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
            let w = artifact::witness(api, root, &attempt_run, attempt)?;
            let built_original =
                profile::validate_witness(root, &w, &attempt_run, attempt, anchor)?;
            if built_original != original {
                return Err(
                    "actual checkout disagrees with independently applicable generation".into(),
                );
            }
            let Some((mapping, adoption)) = adoption::resolve(api, root, pr, &w)? else {
                return Err(format!("missing exact authenticated adequacy adoption on PR #{pr}; proposal: ```depth-adequacy-v1\n{}\n```",serde_json::to_string_pretty(&adoption::proposal(&w,"REPLACE_WITH_REVIEWER_INVOCATION")?).map_err(|e|e.to_string())?));
            };
            let built = source(root, &w.build.source_commit)?;
            let receipt = EvidenceReceipt {
                schema_version: 1,
                scope: profile::scope(&built_original),
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
                    artifact_name: format!("depth-channel-v1-{id}-{attempt}"),
                },
                build: w.build.clone(),
                provenance: w.provenance.clone(),
                cases: w.cases.clone(),
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
                witness: w,
            });
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
        original,
        definition,
        runs,
    })
}

fn receipt_ref(r: &EvidenceReceipt) -> Result<FileRef, String> {
    Ok(depth_ci::reference(
        format!(
            "cli_manifests/codex/reports/0.0.0/evidence/channel-v1/{}-{}/receipt.json",
            r.producer.run_id, r.producer.run_attempt
        ),
        &serde_json::to_vec(r).map_err(|e| e.to_string())?,
    ))
}
impl Resolved {
    fn population(&self, current: &Evaluation) -> Result<Option<Population>, String> {
        if profile::scope(&self.original) != current.current
            || self.definition != current.definition_sha256
            || current.record.event != self.original.event
            || current.record.policy != self.original.policy
            || current.record.obligations != self.original.obligations
        {
            return Ok(None);
        }
        let first = &self.runs[0].witness;
        if self.runs.iter().any(|r| {
            r.witness.environment != first.environment
                || r.witness.build.toolchain != first.build.toolchain
        }) {
            return Ok(None);
        }
        Ok(Some(Population {
            current: profile::scope(&self.original),
            definition_sha256: self.definition.clone(),
            environment: first.environment.clone(),
            toolchain: first.build.toolchain.clone(),
            runs: self
                .runs
                .iter()
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
            || current.current.version != "0.0.0"
        {
            return Ok(None);
        }
        // Read failures are absence of authenticated authority, never successful evidence.
        match resolve(&Github, self.root) {
            Ok(resolved) => resolved.population(current),
            Err(_) => Ok(None),
        }
    }
}

/// Read-only live proof in a disposable checkout. It materializes no authoritative repository
/// records, modifies no packet/host policy, and cannot enroll a production version.
pub fn prove(root: &Path) -> Result<String, String> {
    prove_with(root, &Github)
}
fn prove_with(root: &Path, api: &impl Api) -> Result<String, String> {
    let resolved = resolve(api, root)?;
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
    let mut selected = None;
    for run in &resolved.runs {
        let r = receipt_ref(&run.receipt)?;
        selected.get_or_insert(r.clone());
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
            .args([
                "add",
                "--",
                "cli_manifests/codex/reports/0.0.0/evidence/channel-v1",
            ]),
    )?;
    let selected = selected.ok_or("no selected proof")?;
    let mut record = resolved.original.clone();
    let scope = profile::scope(&record);
    let mut e = crate::depth_record::ExecutionPart {
        identity: String::new(),
        policy_identity: scope.policy_identity.clone(),
        obligations_identity: scope.obligations_identity.clone(),
        results: vec![crate::depth_record::ObligationResult {
            obligation: scope.obligation.clone(),
            result: DepthResult::Verified,
            rationale: None,
            evidence: vec![selected.clone()],
        }],
        reuse_bindings: vec![],
        debt_grants: vec![],
        debt_retired: vec![],
    };
    e.identity = crate::depth_record::part_identity(&e)?;
    record.execution = Some(e);
    let first = &resolved.runs[0].witness;
    let current = Evaluation {
        current: scope.clone(),
        definition_sha256: resolved.definition.clone(),
        record: record.clone(),
        environment: first.environment.clone(),
        toolchain: first.build.toolchain.clone(),
    };
    struct Snapshot(Resolved);
    impl Host for Snapshot {
        fn population(&self, c: &Evaluation) -> Result<Option<Population>, String> {
            self.0.population(c)
        }
    }
    let verdict = evaluate(scratch.path(), &selected, &current, &Snapshot(resolved))?;
    let results = super::super::results::evaluated(&record, scope.obligation, verdict.result)?;
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
        version: "0.0.0".into(),
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
    serde_json::to_string_pretty(&json!({"origin":"isolated","result":verdict.result,"reason":verdict.reason,"publisher":facts,"publication_json":serde_json::from_str::<Value>(&publication.json).map_err(|e|e.to_string())?,"publication_markdown":publication.markdown,"production_enrollment":false})).map_err(|e|e.to_string())
}

#[cfg(test)]
mod tests;
