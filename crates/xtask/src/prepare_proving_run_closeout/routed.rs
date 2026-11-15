//! Prepared ON5 retains reporting authority through aggregate admission and checked delivery.
use super::*;
use crate::{
    depth_acceptance::onboarding::on5, depth_evidence::EvidenceVerdicts,
    isolated_maintenance::Route,
};
use std::collections::BTreeMap;

pub(super) fn run(
    root: &Path,
    args: Args,
    route: Route,
    writer: &mut impl Write,
) -> Result<(), Error> {
    if !route.context().bootstrap_declared_onboarding() {
        crate::depth_routes::refuse_workspace(root, "prepare_proving_run_closeout")
            .map_err(Error::Validation)?;
        return Err(Error::Validation(
            "prepared ON5 requires declared onboarding authority".into(),
        ));
    }
    if args.check == args.write {
        return Err(Error::Validation(
            "choose exactly one of --check or --write".into(),
        ));
    }
    let captured = route.capture(root).map_err(Error::Validation)?;
    let proposal = captured.proposal(root).map_err(Error::Validation)?;
    let head = on5::head(root).map_err(Error::Validation)?;
    let outcome = (|| -> Result<_, Error> {
        let approval = on5::authority(
            root,
            &args.approval,
            &captured,
            "prepare_proving_run_closeout",
        )
        .map_err(Error::Validation)?;
        let entry = captured
            .registry()
            .find(&approval.descriptor.agent_id)
            .cloned()
            .ok_or_else(|| Error::Validation("unknown ON5 agent".into()))?;
        let context = context_for(root, approval, entry, |path, approval| {
            on5::read_prepared(root, path, approval).map_err(Error::Validation)
        })?;
        #[allow(unused_mut)]
        let mut files = prepare_files_at(
            &context,
            now_rfc3339().map_err(|e| Error::Internal(e.to_string()))?,
            head,
        )?;
        #[cfg(test)]
        on5::proposal_hooks::apply(root, &mut files).map_err(Error::Validation)?;
        let scratch = tempfile::tempdir().map_err(|e| Error::Internal(e.to_string()))?;
        let view = proposal
            .isolated_view(root, scratch.path())
            .map_err(Error::Validation)?;
        let planned = (|| {
            let _binding = captured.bind_view(&view).map_err(Error::Validation)?;
            on5::materialize(scratch.path(), &files).map_err(Error::Validation)?;
            proposal
                .plan_with(root, files, |_, before, after, _| {
                    on5::prepared_batch(before.clone(), after.clone(), captured.registry())?;
                    EvidenceVerdicts::with_publication_view(
                        root,
                        scratch.path(),
                        &proposal,
                        &captured,
                        &view,
                        || {
                            crate::depth_acceptance::check_captured_relay(
                                before,
                                after,
                                scratch.path(),
                            )
                        },
                    )?;
                    Ok(BTreeMap::new())
                })
                .map_err(Error::Validation)
        })();
        view.verify_current(root).map_err(Error::Validation)?;
        planned
    })();
    captured.recheck(root).map_err(Error::Validation)?;
    let mutations = outcome?;
    if args.write {
        on5::deliver(root, &mutations).map_err(Error::Validation)?;
    }
    writeln!(
        writer,
        "OK: prepare-proving-run-closeout local proposal checked."
    )
    .map_err(|e| Error::Internal(e.to_string()))
}
