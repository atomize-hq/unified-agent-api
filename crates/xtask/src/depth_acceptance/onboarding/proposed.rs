//! CP-P3 read-only post-entry evaluation for later ON4/ON5 controllers.
use super::*;
use crate::{
    agent_maintenance::closeout_proposal::{IsolatedView, Proposal},
    isolated_maintenance::CapturedRegistry,
};
use std::path::Path;

/// The caller holds the scratch registry guard and supplies an already coherent entry.
/// This evaluates fresh after that entry, checks every aggregate effect, then borrows the
/// evaluated publication scope for rendering. It delivers no files or admission receipt.
pub(crate) fn evaluate_proposed<T>(
    caller: &Path,
    proposal: &Proposal,
    captured: &CapturedRegistry,
    view: &IsolatedView<'_>,
    kind: Kind,
    render: impl FnOnce(&DepthRecord) -> Result<T, String>,
) -> Result<T, String> {
    let super::super::Authority::Isolated(context) = proposal.authority() else {
        return Err("onboarding proposal requires captured isolated authority".into());
    };
    let host = captured.evidence_host(caller)?;
    let (source, mut inputs) = context.view_inputs(view, &host)?;
    let committed = Source::tip(caller, "HEAD")?;
    let agent = crate::depth_ci::profile::Profile::SyntheticOnboarding.agent_id();
    let candidate = View::with_registry(source.source.clone(), captured.registry(), agent)?;
    let approval_path = crate::agent_lifecycle::approval_artifact_path_for_entry(&candidate.entry);
    captured.onboarding_approval(committed.read(&approval_path)?.as_bytes(), &approval_path)?;
    let version = crate::depth_ci::profile::Profile::SyntheticOnboarding.version();
    let record = candidate
        .records
        .get(version)
        .ok_or("onboarding proposal has no record")?;
    let path = format!(
        "{}/reports/{version}/depth-record.json",
        candidate.entry.manifest_root
    );
    let outcome = (|| {
        let expected = acceptance(
            &candidate,
            &committed,
            record,
            kind,
            || {
                crate::support_depth::current_onboarding_bindings(
                    &mut inputs,
                    &source.source,
                    record,
                )
            },
            || {
                let (evaluated, verdicts) =
                    EvidenceVerdicts::bind_view(view, context, &host, &path)?.evaluate()?;
                if evaluated != *record {
                    return Err("onboarding post-entry record changed during evaluation".into());
                }
                Ok(verdicts)
            },
        )?;
        if kind == Kind::Proving {
            let closeout_path = format!(
                "docs/agents/lifecycle/{}/governance/proving-run-closeout.json",
                candidate.entry.scaffold.onboarding_pack_prefix
            );
            let closeout = candidate.json(&closeout_path)?;
            crate::agent_maintenance::closeout::require_commit_binding(
                caller,
                super::super::input::text(&closeout, "commit")?,
            )
            .map_err(|e| e.to_string())?;
        }
        if !record.acceptance_entries.contains(&expected) {
            return Err("onboarding proposal lacks its exact current owning entry".into());
        }
        effects::check(&committed, &source.source, &candidate, record, kind)?;
        EvidenceVerdicts::with_publication_view(
            caller,
            &source.root,
            proposal,
            captured,
            view,
            || render(record),
        )
    })();
    // Error and unchanged-input paths retain the same closing observations.
    let input_check = inputs.verify();
    let view_check = context.verify_view(view, &source, &host);
    view_check?;
    input_check?;
    outcome
}

/// Build a coherent entry-bearing record in memory. The returned locator has no admission
/// authority. The controller materializes it in scratch before `evaluate_proposed`.
pub(crate) fn coherent_entry(
    caller: &Path,
    proposal: &Proposal,
    captured: &CapturedRegistry,
    view: &IsolatedView<'_>,
    kind: Kind,
) -> Result<DepthRecord, String> {
    let super::super::Authority::Isolated(context) = proposal.authority() else {
        return Err("onboarding proposal requires captured isolated authority".into());
    };
    let host = captured.evidence_host(caller)?;
    let (source, inputs) = context.view_inputs(view, &host)?;
    let committed = Source::tip(caller, "HEAD")?;
    let p = crate::depth_ci::profile::Profile::SyntheticOnboarding;
    let candidate = View::with_registry(source.source.clone(), captured.registry(), p.agent_id())?;
    let approval_path = crate::agent_lifecycle::approval_artifact_path_for_entry(&candidate.entry);
    captured.onboarding_approval(committed.read(&approval_path)?.as_bytes(), &approval_path)?;
    let outcome = (|| {
        let mut record = candidate
            .records
            .get(p.version())
            .ok_or("onboarding proposal has no record")?
            .clone();
        let entry = entry_reference(&candidate, &committed, &record, kind)?;
        if !record.acceptance_entries.contains(&entry) {
            record.acceptance_entries.push(entry);
        }
        Ok(record)
    })();
    let input_check = inputs.verify();
    let view_check = context.verify_view(view, &source, &host);
    view_check?;
    input_check?;
    outcome
}
