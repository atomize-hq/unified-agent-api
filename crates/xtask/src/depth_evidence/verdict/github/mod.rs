//! The only installed host profile: canonical GitHub CI, isolated source-contract proof.
use super::*;
use crate::depth_ci::profile::Profile;
use crate::{
    depth_ci::{self, Witness},
    depth_record::Event,
};
use serde_json::{json, Value};
pub(super) mod adoption;
mod artifact;
pub(super) mod assembly;
mod population;
mod profile;
use population::resolve_profile_inputs;
#[path = "readonly_transport.rs"]
pub(super) mod transport;
#[path = "transport.rs"]
mod transport_impl;
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
    resolve_profile_in(api, root, p, None)
}
/// Channel and CORE read the canonical repository and require whole-workflow completion. The
/// synthetic profile reads only the isolated repository its authenticated IR-P1 binding names,
/// and admits an attempt once its producer job is terminal (IR-P3).
fn resolve_profile_in(
    api: &impl Api,
    root: &Path,
    p: Profile,
    host: Option<&crate::isolated_maintenance::EvidenceHost>,
) -> Result<Resolved, String> {
    resolve_profile_inputs(api, root, p, host, None)
}

fn receipt_ref(r: &EvidenceReceipt) -> Result<FileRef, String> {
    let p = match (r.scope.agent_id.as_str(), r.scope.version.as_str()) {
        ("codex", "0.0.0") => Profile::Channel,
        ("codex", "0.156.1") => Profile::Core,
        ("depth_fixture", "0.2.0") => Profile::for_event(&r.scope.event),
        _ => return Err("receipt names no compiled isolated profile".into()),
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
        if current.current.enrollment_origin != EnrollmentOrigin::Isolated {
            return Ok(None);
        }
        let (p, host) = match (
            current.current.agent_id.as_str(),
            current.current.version.as_str(),
        ) {
            ("codex", "0.0.0") => (Profile::Channel, None),
            ("codex", "0.156.1") => (Profile::Core, None),
            // Only the installed IR-P1 binding names the isolated repository; uninstalled, none.
            ("depth_fixture", "0.2.0") => {
                match crate::isolated_maintenance::evidence_host(self.root) {
                    Ok(host) => (Profile::for_event(&current.current.event), Some(host)),
                    Err(_) => return Ok(None),
                }
            }
            _ => return Ok(None),
        };
        let resolved = match &host {
            Some(h) => resolve_profile_in(&Isolated::new(h), self.root, p, Some(h)),
            None => resolve_profile_in(&CANONICAL, self.root, p, None),
        };
        // Read failures are absence of authenticated authority, never successful evidence.
        match resolved {
            Ok(resolved) => resolved.population(current),
            Err(_) => Ok(None),
        }
    }
}

/// Read-only live proof in a disposable checkout. It materializes no authoritative repository
/// records, modifies no packet/host policy, and cannot enroll a production version.
pub fn prove(root: &Path) -> Result<String, String> {
    prove_profile(root, &CANONICAL, Profile::Channel)
}
#[cfg(test)]
fn prove_with(root: &Path, api: &impl Api) -> Result<String, String> {
    prove_profile(root, api, Profile::Channel)
}
pub fn prove_core(root: &Path) -> Result<String, String> {
    prove_profile(root, &CANONICAL, Profile::Core)
}
fn prove_profile(root: &Path, api: &impl Api, p: Profile) -> Result<String, String> {
    prove_profile_in(root, api, p, None)
}
fn prove_profile_in(
    root: &Path,
    api: &impl Api,
    p: Profile,
    host: Option<&crate::isolated_maintenance::EvidenceHost>,
) -> Result<String, String> {
    let resolved = resolve_profile_in(api, root, p, host)?;
    let target = match &resolved.original.obligations {
        Some(o) if o.targets.len() == 1 => o.targets[0].clone(),
        _ => return Err("isolated proof requires exactly one frozen target".into()),
    };
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
        source(
            scratch.path(),
            api.repository(),
            &run.receipt.build.source_commit,
        )?;
        source(
            scratch.path(),
            api.repository(),
            &run.receipt.adequacy.endorsement.source_commit,
        )?;
    }
    let jail = crate::workspace_mutation::WorkspacePathJail::new(scratch.path())
        .map_err(|e| e.to_string())?;
    let assembly = assembly::assemble(&resolved)?;
    for (reference, bytes) in &assembly.files {
        let path = jail
            .resolve(Path::new(&reference.path))
            .map_err(|e| e.to_string())?;
        std::fs::create_dir_all(path.parent().ok_or("missing proof parent")?)
            .map_err(|e| e.to_string())?;
        if path.exists() && std::fs::read(&path).map_err(|e| e.to_string())? != *bytes {
            return Err("conflicting hosted payload path".into());
        }
        std::fs::write(path, bytes).map_err(|e| e.to_string())?;
    }
    crate::depth_ci::output(
        std::process::Command::new("git")
            .arg("-C")
            .arg(scratch.path())
            .args(["add", "--", &p.evidence()]),
    )?;
    let selected = assembly.selected;
    let mut record = resolved.original.clone();
    record.execution = Some(assembly.execution);
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
        &target,
        &|path| {
            std::fs::read(scratch.path().join(path))
                .ok()
                .map(|b| depth_ci::reference(path, &b).sha256)
        },
        &results,
    )?;
    let row = crate::support_matrix::SupportRow {
        agent: p.agent_id().into(),
        version: p.version().into(),
        target: target.clone(),
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
#[cfg(all(test, target_os = "linux"))]
pub(crate) use tests::host::{synthetic_host, SyntheticHost};

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
        self.decisions_with(record, |selected, current| {
            evaluate(root, selected, current, self)
        })
    }
    fn decisions_with(
        &self,
        record: &DepthRecord,
        mut evaluate_selected: impl FnMut(&FileRef, &Evaluation) -> Result<Verdict, String>,
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
                    outcomes.push(evaluate_selected(selected, &current)?);
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
/// Read-only bridge for the fixed isolated profiles: CORE from the canonical repository, and the
/// synthetic profile only through the installed IR-P1 binding. Production consumers never call it.
/// It always re-authenticates the complete population, whatever E or a staged bundle claims.
/// With `borrowed`, it evaluates through the caller's host and never resolves one from `root`.
pub(in crate::depth_evidence) fn isolated_record_results(
    root: &Path,
    record: &DepthRecord,
    borrowed: Option<&crate::isolated_maintenance::EvidenceHost>,
) -> Result<super::super::EvidenceVerdicts, String> {
    if let Some(host) = borrowed {
        return borrowed_record_results(root, record, host);
    }
    let selection = &record.policy.selection;
    let resolved = match (
        selection.agent_id.as_str(),
        selection.enrollment.version.as_str(),
    ) {
        ("depth_fixture", "0.2.0") => {
            crate::isolated_maintenance::evidence_host(root).and_then(|host| {
                resolve_profile_in(
                    &Isolated::new(&host),
                    root,
                    Profile::for_event(&record.event),
                    Some(&host),
                )
            })
        }
        _ => resolve_profile(&CANONICAL, root, Profile::Core),
    };
    match resolved {
        Ok(resolved) => resolved.evaluate_record(root, record),
        Err(_) => Ok(super::super::EvidenceVerdicts::default()),
    }
}
/// Owning-predicate side (CP-P1): only the compiled synthetic profile has an isolated evidence
/// host, so only its record is evaluated. Failures to authenticate refuse; they never read as
/// evidence and never fall back to a binding resolved from `root`.
fn borrowed_record_results(
    root: &Path,
    record: &DepthRecord,
    host: &crate::isolated_maintenance::EvidenceHost,
) -> Result<super::super::EvidenceVerdicts, String> {
    let profile = Profile::for_event(&record.event);
    let selection = &record.policy.selection;
    if selection.agent_id != profile.agent_id() || selection.enrollment.version != profile.version()
    {
        return Err("a borrowed isolated evidence host evaluates only its compiled profile".into());
    }
    resolve_profile_in(&Isolated::new(host), root, profile, Some(host))?
        .evaluate_record(root, record)
}
/// MA4 host side (B.4 part 2): authenticate the complete applicable population through the
/// binding, then assemble reporting E and its referenced files. Only the synthetic profile has an
/// isolated evidence host. The result is staged for the executor; it grants nothing.
pub(crate) fn isolated_assembly(
    root: &Path,
    host: &crate::isolated_maintenance::EvidenceHost,
) -> Result<assembly::Assembly, String> {
    let p = Profile::synthetic_in(&crate::runtime_advertising::source::Source::workspace(
        root,
    )?)?;
    let resolved = resolve_profile_in(&Isolated::new(host), root, p, Some(host))?;
    assembly::assemble(&resolved)
}

/// CP-P3: all proposed file reads use the bound view, while all immutable host provenance
/// resolves through the caller's object store. Never resolve an evidence host from either root.
pub(super) fn view_record_results(
    inputs: &mut GitInputs,
    record: &DepthRecord,
    host: &crate::isolated_maintenance::EvidenceHost,
) -> Result<super::super::EvidenceVerdicts, String> {
    let p = Profile::for_event(&record.event);
    let selection = &record.policy.selection;
    if selection.agent_id != p.agent_id() || selection.enrollment.version != p.version() {
        return Err("a borrowed isolated evidence host evaluates only its compiled profile".into());
    }
    let _read_only = transport::read_only(inputs)?;
    let caller = inputs.object_root().to_path_buf();
    let resolved =
        resolve_profile_inputs(&Isolated::new(host), &caller, p, Some(host), Some(inputs))?;
    let (results, _) = resolved.decisions_with(record, |selected, current| {
        evaluate_inputs(&caller, inputs, selected, current, &resolved)
    })?;
    inputs.verify()?;
    Ok(results)
}
