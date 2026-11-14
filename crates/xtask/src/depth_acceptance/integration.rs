//! Final integration reuses owning predicates; entries and route labels grant no authority.
use super::{admission::observations, input::View, manual, promotion};
use crate::{
    agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH},
    depth_record::AcceptanceEntry,
    runtime_advertising::source::Source,
};
use std::{collections::BTreeSet, path::Path};

mod binding;
mod bootstrap;
mod generation;

pub(super) fn check(root: &Path, revision: &str) -> Result<(), String> {
    // Resolve repository-wide authority before enumerating or narrowing to any manifest root.
    let Some(route) = crate::isolated_maintenance::route(root)? else {
        return check_with(root, revision, &manual::Production, &promotion::Production);
    };
    route.with_admitted_publication(root, revision, || Ok(()))
}

impl crate::isolated_maintenance::Route {
    /// The public --tip consistency boundary and check_integration share authenticated IN1
    /// admission and fresh B.7a evaluation under the same captured caller authority.
    pub(crate) fn with_admitted_publication<T>(
        self,
        root: &Path,
        revision: &str,
        validate: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        let mut emitter = self.context().admission_emitter()?;
        let observations = super::admission::observations::Capture::start()?;
        let outcome = self.evaluate_admitted_publication(root, revision, validate, &mut emitter);
        let outcome = observations::after_decision(outcome, || Ok(()));
        emitter.finish(root, outcome, observations.finish())
    }

    fn evaluate_admitted_publication<T>(
        self,
        root: &Path,
        revision: &str,
        validate: impl FnOnce() -> Result<T, String>,
        emitter: &mut super::admission::emitter::Emitter,
    ) -> Result<T, String> {
        // Only bootstrap lacks a resolved enrollment. Existing routes must retain their
        // capture -> proposal -> binding error order, including competing input failures.
        if self.context().bootstrap_pending()? {
            let initial = binding::capture(root, revision, self.context())?;
            emitter.parent = Some(initial.parent.clone());
            emitter.prepare(root)?;
            if let Some(checkpoint) = self.context().bootstrap_checkpoint(root, &initial.parent)? {
                if checkpoint.publication_changed {
                    checkpoint
                        .with_view(root, |view| bootstrap::publication(view, &initial.parent))?;
                }
                let outcome = checkpoint.with_registry(root, validate);
                return observations::after_decision(outcome, || {
                    initial.recheck(root, self.context())
                });
            }
        }
        let captured = self.capture(root)?;
        let proposal = captured.proposal(root)?;
        let super::Authority::Isolated(context) = proposal.authority() else {
            return Err("isolated IN1 binding lost its captured authority".into());
        };
        let binding = binding::capture(root, revision, context)?;
        emitter.parent = Some(binding.parent.clone());
        emitter.prepare(root)?;
        // Only onboarding can observe a first declaration. Maintenance keeps its existing
        // admission even when its parent lacks the subject entry or depth table.
        let checkpoint = if context.bootstrap_declared_onboarding() {
            context.bootstrap_checkpoint(root, &binding.parent)?
        } else {
            None
        };
        if let Some(checkpoint) = checkpoint {
            if checkpoint.publication_changed {
                checkpoint.with_view(root, |view| bootstrap::publication(view, &binding.parent))?;
            }
            let outcome = checkpoint.with_registry(root, validate);
            return observations::after_decision(outcome, || binding.recheck(root, context));
        }
        let outcome = crate::depth_evidence::EvidenceVerdicts::with_integration_publication(
            root,
            &captured,
            || {
                admit_all(
                    root,
                    &binding.parent,
                    &super::isolated::Host::closeout(context),
                    &super::isolated::Host::promotion(context),
                    |view| super::publication::validate_current(root, view),
                )?;
                validate()
            },
        );
        observations::after_decision(outcome, || binding.recheck(root, context))
    }
}

// Per-root IN1 readers borrow the same routed scope instead of re-entering production-only
// parsers. These are read adapters; all-root `check` remains the owning admission.
impl crate::isolated_maintenance::Route {
    pub(crate) fn integration_registry(
        &self,
        root: &Path,
        revision: &str,
    ) -> Result<AgentRegistry, String> {
        self.recheck(root)?;
        let source = Source::tip(root, revision)?;
        let registry = AgentRegistry::parse_unenforced(&source.read(REGISTRY_RELATIVE_PATH)?)
            .map_err(|e| e.to_string())?;
        self.recheck(root)?;
        Ok(registry)
    }

    pub(crate) fn integration_comparison(
        self,
        root: &Path,
        revision: &str,
        agent: &str,
    ) -> Result<super::Comparison, String> {
        let registry = self.registry(root)?.clone();
        let earlier = self.integration_registry(root, revision)?;
        let before = View::with_registry(Source::tip(root, revision)?, &earlier, agent)?;
        let after = View::with_registry(Source::workspace(root)?, &registry, agent)?;
        let mut comparison = super::compare_views(&before, &after)?;
        let captured = self.capture(root)?;
        if !before.records.is_empty() || !after.records.is_empty() {
            comparison.input_issues =
                crate::depth_evidence::EvidenceVerdicts::with_integration_publication(
                    root,
                    &captured,
                    || super::publication::validate_current(root, &after),
                )
                .err()
                .into_iter()
                .collect();
        }
        captured.recheck(root)?;
        Ok(comparison)
    }
}

pub(super) fn check_with(
    root: &Path,
    revision: &str,
    closeout_host: &impl manual::Host,
    promotion_host: &impl manual::Host,
) -> Result<(), String> {
    let tip = resolve(root, revision)?;
    let head = resolve(root, "HEAD")?;
    admit_all(root, &tip, closeout_host, promotion_host, |view| {
        super::publication::validate_current(root, view)
    })?;
    if resolve(root, revision)? != tip || resolve(root, "HEAD")? != head {
        return Err("integration revision moved during admission".into());
    }
    Ok(())
}

/// Admission consumes the already pinned parent. In particular it must never resolve a
/// caller's mutable parent ref again after the isolated binding has authenticated P -> H.
fn admit_all(
    root: &Path,
    parent: &str,
    closeout_host: &impl manual::Host,
    promotion_host: &impl manual::Host,
    validate_publication: impl FnOnce(&View) -> Result<(), String>,
) -> Result<(), String> {
    let before = Source::tip(root, parent)?;
    let after = Source::workspace(root)?;
    let old_registry = AgentRegistry::parse_unenforced(&before.read(REGISTRY_RELATIVE_PATH)?)
        .map_err(|e| e.to_string())?;
    let registry = AgentRegistry::parse_unenforced(&after.read(REGISTRY_RELATIVE_PATH)?)
        .map_err(|e| e.to_string())?;
    let agents: BTreeSet<_> = old_registry
        .agents
        .iter()
        .chain(&registry.agents)
        .map(|e| e.agent_id.as_str())
        .collect();
    let mut publication = None;
    for agent in agents {
        let old = View::with_registry(
            before.clone(),
            if old_registry.find(agent).is_some() {
                &old_registry
            } else {
                &registry
            },
            agent,
        )?;
        let new = View::with_registry(
            after.clone(),
            if registry.find(agent).is_some() {
                &registry
            } else {
                &old_registry
            },
            agent,
        )?;
        generation::check(root, &old, &new).map_err(|e| format!("{agent}: {e}"))?;
        if old.records.is_empty() && new.records.is_empty() {
            continue;
        }
        let declaration = new.json(&format!("{}/RULES.json", new.entry.manifest_root))?;
        let expected = format!(
            "{}/reports/<version>/depth-record.json",
            new.entry.manifest_root
        );
        if declaration
            .pointer("/storage/committed/reports/depth_record")
            .and_then(serde_json::Value::as_str)
            != Some(expected.as_str())
        {
            return Err(format!(
                "{agent}: retained root does not adopt the canonical depth record path"
            ));
        }
        if registry.find(agent).is_none() {
            return Err("retained agent lost its registry anchor".into());
        }
        admit(root, &old, &new, &before, closeout_host, promotion_host)
            .map_err(|e| format!("{agent}: {e}"))?;
        publication = Some(new);
    }
    if let Some(view) = publication {
        validate_publication(&view)?;
    }
    Ok(())
}

fn resolve(root: &Path, revision: &str) -> Result<String, String> {
    let out = crate::git_inputs::command(root)
        .args([
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{revision}^{{commit}}"),
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err("unreadable integration revision".into());
    }
    String::from_utf8(out.stdout)
        .map(|s| s.trim().into())
        .map_err(|e| e.to_string())
}

pub(super) fn admit(
    root: &Path,
    old: &View,
    new: &View,
    tip: &Source,
    closeout_host: &impl manual::Host,
    promotion_host: &impl manual::Host,
) -> Result<(), String> {
    let comparison = super::compare_views(old, new)?;
    let qualifications = super::publication::qualifications(new)?;
    let candidate_promotion = candidate_has_promotion(old, new)?;
    for (version, prior) in &old.records {
        let record = new
            .records
            .get(version)
            .ok_or("integration removed a retained record")?;
        if !record
            .acceptance_entries
            .starts_with(&prior.acceptance_entries)
            || !record.lacking_from(prior).is_empty()
        {
            return Err("integration changed retained entries or shrank selection".into());
        }
    }
    for (version, record) in &new.records {
        let digest = record.stated_sha256()?;
        // Dependency-only changes can invalidate a still-published qualification without
        // changing the record, its entries or any newly accepting value.
        if qualifications
            .iter()
            .any(|(agent, v, _, _)| agent == &new.entry.agent_id && v == version)
        {
            if record.policy.selection.enrollment.lifecycle_path
                != crate::agent_registry::support_depth::LifecyclePath::Maintenance
                || !record.policy.path_enabled
            {
                return Err("qualified publication lacks maintenance lifecycle authority".into());
            }
            manual::admitted(root, new, record, promotion_host)?;
            if !record
                .acceptance_entries
                .iter()
                .any(|e| e.record_sha256() == digest)
            {
                return Err("qualified publication has no current entry".into());
            }
        }
        let previous = old
            .records
            .get(version)
            .map(|r| r.acceptance_entries.as_slice())
            .unwrap_or(&[]);
        let added = &record.acceptance_entries[previous.len()..];
        let effects: Vec<_> = comparison
            .effects
            .iter()
            .filter(|e| e.version == *version)
            .collect();
        if added.is_empty() && effects.is_empty() {
            continue;
        }
        if added.iter().any(|e| e.record_sha256() != digest) {
            return Err("new acceptance entry must describe the current record".into());
        }
        let closeout_path = format!(
            "docs/agents/lifecycle/{}-maintenance/governance/maintenance-closeout.json",
            new.entry.agent_id
        );
        let new_closeout = added.iter().any(|e| {
            matches!(e, AcceptanceEntry::MaintenanceCloseout { .. }) && !previous.contains(e)
        });
        let mut closeout = new_closeout;
        let mut promote = added
            .iter()
            .any(|e| matches!(e, AcceptanceEntry::Promotion { .. }));
        for e in &effects {
            match e.kind.as_str() {
                "published" | "closed_baseline" | "proving_run_closeout_written" => {
                    return Err("unsupported onboarding lifecycle authority at integration".into());
                }
                "closeout semantic change"
                | "maintenance_closeout_written"
                | "absence_of_drifted" => {
                    if e.path != closeout_path {
                        return Err("wrong lifecycle closeout authority".into());
                    }
                    closeout = true;
                }
                _ if super::publication::is_qualification(e)
                    && new_closeout
                    && !candidate_promotion => {}
                _ => promote = true,
            }
        }
        let manual_entry = if closeout {
            let entry = manual::acceptance(root, new, record, tip, closeout_host)?;
            if !record.acceptance_entries.contains(&entry)
                || (new_closeout && (!added.contains(&entry) || previous.contains(&entry)))
            {
                return Err("integration closeout lacks its exact new current entry".into());
            }
            Some(entry)
        } else {
            None
        };
        let promotion_entry = if promote {
            // Only an already admitted tip entry or this candidate's independently admitted
            // closeout can supply retained authority. A fabricated locator is not a closeout.
            let retained = record.acceptance_entries.iter().any(|e| matches!(e,
                AcceptanceEntry::MaintenanceCloseout { record_sha256, .. }
                if *record_sha256 == digest && (previous.contains(e) || manual_entry.as_ref() == Some(e))));
            if !retained {
                return Err("promotion has no admitted retained closeout".into());
            }
            let entry =
                promotion::acceptance(root, new, record, tip, &comparison.effects, promotion_host)?;
            if !record.acceptance_entries.contains(&entry) {
                return Err("integration promotion lacks its own current target entry".into());
            }
            Some(entry)
        } else {
            None
        };
        for entry in added {
            if manual_entry.as_ref() != Some(entry) && promotion_entry.as_ref() != Some(entry) {
                return Err("new acceptance entry has no supported owning authority".into());
            }
        }
    }
    Ok(())
}

// Qualification alone is attributed below. Every other accepting effect (or new Promotion
// entry), anywhere in the candidate, prevents borrowing closeout authority for qualification.
fn candidate_has_promotion(old: &View, new: &View) -> Result<bool, String> {
    for entry in old.registry.agents.iter().chain(&new.registry.agents) {
        let registry = if new.registry.find(&entry.agent_id).is_some() {
            &new.registry
        } else {
            &old.registry
        };
        let prior_registry = if old.registry.find(&entry.agent_id).is_some() {
            &old.registry
        } else {
            &new.registry
        };
        let before = View::with_registry(old.source.clone(), prior_registry, &entry.agent_id)?;
        let after = View::with_registry(new.source.clone(), registry, &entry.agent_id)?;
        if after.records.iter().any(|(version, record)| {
            record.acceptance_entries.iter().any(|e| {
                matches!(e, AcceptanceEntry::Promotion { .. })
                    && !before
                        .records
                        .get(version)
                        .is_some_and(|r| r.acceptance_entries.contains(e))
            })
        }) {
            return Ok(true);
        }
        if super::compare_views(&before, &after)?
            .effects
            .iter()
            .any(|e| {
                !super::publication::is_qualification(e)
                    && !matches!(
                        e.kind.as_str(),
                        "closeout semantic change"
                            | "maintenance_closeout_written"
                            | "absence_of_drifted"
                    )
            })
        {
            return Ok(true);
        }
    }
    Ok(false)
}
