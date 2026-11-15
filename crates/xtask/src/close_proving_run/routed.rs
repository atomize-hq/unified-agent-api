//! Final ON5 owns one exact proving entry and its closed-baseline assertion.
use super::*;
use crate::{
    depth_acceptance::onboarding::{self, on5, Kind},
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
        crate::depth_routes::refuse_workspace(root, "close_proving_run")
            .map_err(Error::Validation)?;
        return Err(Error::Validation(
            "final ON5 requires declared onboarding authority".into(),
        ));
    }
    let captured = route.capture(root).map_err(Error::Validation)?;
    let proposal = captured.proposal(root).map_err(Error::Validation)?;
    let head = on5::head(root).map_err(Error::Validation)?;
    let outcome = (|| {
        let approval_path = normalize_repo_relative_path(root, &args.approval, "--approval")?;
        let approval = on5::authority(
            root,
            &approval_path.to_string_lossy(),
            &captured,
            "close_proving_run",
        )
        .map_err(Error::Validation)?;
        let entry = captured
            .registry()
            .find(&approval.descriptor.agent_id)
            .ok_or_else(|| Error::Validation("unknown ON5 agent".into()))?;
        let closeout_path = normalize_repo_relative_path(root, &args.closeout, "--closeout")?;
        let expected =
            agent_lifecycle::proving_run_closeout_path(&entry.scaffold.onboarding_pack_prefix);
        if closeout_path != Path::new(&expected) {
            return Err(Error::Validation(
                "ON5 requires its exact canonical closeout path".into(),
            ));
        }
        let closeout = on5::read_prepared(root, &expected, &approval)
            .map_err(Error::Validation)?
            .ok_or_else(|| Error::Validation("final ON5 requires its prepared closeout".into()))?;
        crate::depth_evidence::EvidenceVerdicts::with_integration_publication(
            root,
            &captured,
            || {
                validate_closeout_prerequisites(root, entry, &approval, &closeout_path, &closeout)
                    .map_err(|e| e.to_string())
            },
        )
        .map_err(Error::Validation)?;
        let finalized =
            finalize_closeout_using(entry, &approval, &closeout, &closeout_path, || head)?;
        let state = next_lifecycle_baseline(root, entry, &approval, &closeout_path)?;
        let state_path = agent_lifecycle::lifecycle_state_path_for_entry(entry);
        let mut files = vec![
            (
                closeout_path.clone(),
                render_closeout_json(&finalized)
                    .map_err(map_closeout_error)?
                    .into_bytes(),
            ),
            (state_path.into(), encode(&state)?),
        ];
        files.extend(
            build_docs_preview(entry, &finalized, &closeout_path)
                .into_iter()
                .map(|(path, text)| (path.into(), text.unwrap_or_default().into_bytes())),
        );
        let scratch = tempfile::tempdir().map_err(|e| Error::Internal(e.to_string()))?;
        let view = proposal
            .isolated_view(root, scratch.path())
            .map_err(Error::Validation)?;
        let planned = (|| {
            let _binding = captured.bind_view(&view).map_err(Error::Validation)?;
            on5::materialize(scratch.path(), &files).map_err(Error::Validation)?;
            let record =
                onboarding::coherent_entry(root, &proposal, &captured, &view, Kind::Proving)
                    .map_err(Error::Validation)?;
            let version = &record.policy.selection.enrollment.version;
            files.push((
                format!(
                    "{}/reports/{version}/depth-record.json",
                    entry.manifest_root
                )
                .into(),
                encode(&record)?,
            ));
            on5::materialize(scratch.path(), &files).map_err(Error::Validation)?;
            #[cfg(test)]
            {
                on5::proposal_hooks::apply(root, &mut files).map_err(Error::Validation)?;
                on5::materialize(scratch.path(), &files).map_err(Error::Validation)?;
            }
            let bundle = onboarding::evaluate_proposed(
                root,
                &proposal,
                &captured,
                &view,
                Kind::Proving,
                |_| crate::support_matrix::generate_publication_artifacts(scratch.path()),
            )
            .map_err(Error::Validation)?;
            files.extend([
                (
                    crate::support_matrix::JSON_OUTPUT_PATH.into(),
                    bundle.json.into_bytes(),
                ),
                (
                    crate::support_matrix::MARKDOWN_OUTPUT_PATH.into(),
                    bundle.markdown.into_bytes(),
                ),
                (
                    crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH.into(),
                    bundle.runtime_support_data.into_bytes(),
                ),
            ]);
            on5::materialize(scratch.path(), &files).map_err(Error::Validation)?;
            onboarding::evaluate_proposed(root, &proposal, &captured, &view, Kind::Proving, |_| {
                Ok(())
            })
            .map_err(Error::Validation)?;
            proposal
                .plan_with(root, files, |_, _, _, _| Ok(BTreeMap::new()))
                .map_err(Error::Validation)
        })();
        view.verify_current(root).map_err(Error::Validation)?;
        captured.recheck(root).map_err(Error::Validation)?;
        planned
    })();
    captured.recheck(root).map_err(Error::Validation)?;
    on5::deliver(root, &outcome?).map_err(Error::Validation)?;
    writeln!(writer, "OK: close-proving-run local candidate prepared.")
        .map_err(|e| Error::Internal(e.to_string()))
}

fn encode(value: &impl serde::Serialize) -> Result<Vec<u8>, Error> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| Error::Internal(e.to_string()))?;
    bytes.push(b'\n');
    Ok(bytes)
}
