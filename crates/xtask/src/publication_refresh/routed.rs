//! ON4 prepares a fully checked branch candidate before any caller delivery.
use super::*;
use crate::{
    depth_acceptance::onboarding::{self, Kind},
    isolated_maintenance::{CapturedRegistry, Route},
    runtime_advertising::source::Source,
};
use std::collections::BTreeMap;

mod gates;
#[cfg(test)]
mod test_hooks;
#[cfg(test)]
pub(crate) use test_hooks::with_delivery_failure;

pub(super) fn run(
    root: &Path,
    args: Args,
    route: Route,
    writer: &mut impl Write,
) -> Result<(), Error> {
    if !route.context().bootstrap_declared_onboarding() {
        crate::depth_routes::refuse_workspace(root, "publication_refresh")
            .map_err(Error::Validation)?;
        return Err(Error::Validation(
            "ON4 requires declared onboarding authority".into(),
        ));
    }
    if args.check == args.write {
        return Err(Error::Validation(
            "choose exactly one of --check or --write".into(),
        ));
    }
    let captured = route.capture(root).map_err(Error::Validation)?;
    let proposal = captured.proposal(root).map_err(Error::Validation)?;
    let context_result = context(root, &args, &captured);
    captured.recheck(root).map_err(Error::Validation)?;
    let context = context_result?;
    // Freeze the finalized packet before creating the entry. The committed ON3 packet
    // already binds the exact publication-ready predecessor and selected runtime evidence.
    validate_packet_freshness(root, &context)?;
    if args.check {
        let outputs = publication_outputs(root, &context)?;
        let stale = stale_output_paths(root, &outputs)?;
        if !stale.is_empty() {
            return Err(Error::Validation(format!(
                "publication-owned outputs are stale: {}",
                stale.join(", ")
            )));
        }
        captured.recheck(root).map_err(Error::Validation)?;
        writeln!(writer, "OK: refresh-publication check passed.")
            .map_err(|e| Error::Internal(e.to_string()))?;
        return Ok(());
    }
    let packet_bytes = Source::tip(root, "HEAD")
        .and_then(|source| source.read(&context.publication_packet_path))
        .map_err(Error::Validation)?;
    let next = build_next_lifecycle_state(
        &context,
        hex::encode(Sha256::digest(packet_bytes.as_bytes())),
    )?;
    let version = context
        .approval
        .support_depth_version
        .as_deref()
        .ok_or_else(|| Error::Validation("missing approved depth version".into()))?;
    let record_path = format!(
        "{}/reports/{version}/depth-record.json",
        context.entry.manifest_root
    );
    let mut files = vec![
        (
            PathBuf::from(&context.lifecycle_state_path),
            serialize_json_pretty(&next)?,
        ),
        (
            PathBuf::from(&context.publication_packet_path),
            packet_bytes.into_bytes(),
        ),
    ];
    let scratch = tempfile::tempdir().map_err(|e| Error::Internal(e.to_string()))?;
    let view = proposal
        .isolated_view(root, scratch.path())
        .map_err(Error::Validation)?;
    let outcome = (|| -> Result<Vec<PlannedMutation>, Error> {
        let _binding = captured.bind_view(&view).map_err(Error::Validation)?;
        materialize(scratch.path(), &files)?;
        let record =
            onboarding::coherent_entry(root, &proposal, &captured, &view, Kind::Publication)
                .map_err(Error::Validation)?;
        files.push((record_path.into(), serialize_json_pretty(&record)?));
        materialize(scratch.path(), &files)?;
        let outputs = onboarding::evaluate_proposed(
            root,
            &proposal,
            &captured,
            &view,
            Kind::Publication,
            |_| publication_outputs(scratch.path(), &context).map_err(|e| e.to_string()),
        )
        .map_err(Error::Validation)?;
        files.extend(
            outputs
                .into_iter()
                .map(|file| (file.relative_path.into(), file.contents)),
        );
        materialize(scratch.path(), &files)?;
        let gate_inputs =
            crate::candidate_execution::tree::capture(scratch.path(), &Default::default())
                .map_err(|e| Error::Validation(e.to_string()))?;
        crate::readonly_gates::check_proposed(root, &proposal, &captured, &view, &record)
            .map_err(Error::Validation)?;
        let repository = proposal
            .preflight_repository(root)
            .map_err(Error::Validation)?
            .ok_or_else(|| {
                Error::Validation("ON4 proposal lacks its captured repository".into())
            })?;
        gates::run(
            root,
            scratch.path(),
            repository,
            &files.iter().map(|(p, _)| p.clone()).collect::<Vec<_>>(),
        )
        .map_err(Error::Validation)?;
        // Gates may alter inputs. Re-authenticate and require exact output bytes afterwards.
        onboarding::evaluate_proposed(root, &proposal, &captured, &view, Kind::Publication, |_| {
            let outputs =
                publication_outputs(scratch.path(), &context).map_err(|e| e.to_string())?;
            if !stale_output_paths(scratch.path(), &outputs)
                .map_err(|e| e.to_string())?
                .is_empty()
            {
                return Err("ON4 gates changed publication outputs".into());
            }
            Ok(())
        })
        .map_err(Error::Validation)?;
        if crate::candidate_execution::tree::capture(scratch.path(), &Default::default())
            .map_err(|e| Error::Validation(e.to_string()))?
            != gate_inputs
        {
            return Err(Error::Validation(
                "ON4 gates changed proposed inputs".into(),
            ));
        }
        proposal
            .plan_with(root, files, |_, _, _, _| Ok(BTreeMap::new()))
            .map_err(Error::Validation)
    })();
    view.verify_current(root).map_err(Error::Validation)?;
    captured.recheck(root).map_err(Error::Validation)?;
    let mutations = outcome?;
    deliver(root, &mutations)?;
    writeln!(
        writer,
        "OK: refresh-publication local candidate prepared.\npublication_packet: {}",
        context.publication_packet_path
    )
    .map_err(|e| Error::Internal(e.to_string()))
}

fn context(root: &Path, args: &Args, captured: &CapturedRegistry) -> Result<RefreshContext, Error> {
    crate::prepare_publication::acquired_inputs::check(root, &args.approval)
        .map_err(Error::Validation)?;
    let committed = Source::tip(root, "HEAD").map_err(Error::Validation)?;
    let raw = committed.read(&args.approval).map_err(Error::Validation)?;
    if Source::workspace(root)
        .and_then(|s| s.read(&args.approval))
        .map_err(Error::Validation)?
        != raw
    {
        return Err(Error::Validation(
            "ON4 approval differs from committed authority".into(),
        ));
    }
    let approval = crate::runtime_follow_on::depth::approval::execution(
        root,
        raw.as_bytes(),
        &args.approval,
        captured,
    )
    .map_err(Error::Validation)?;
    let entry = captured
        .registry()
        .find(&approval.descriptor.agent_id)
        .cloned()
        .ok_or_else(|| Error::Validation("unknown onboarding agent".into()))?;
    context_for(root, approval, entry)
}

fn materialize(root: &Path, files: &[(PathBuf, Vec<u8>)]) -> Result<(), Error> {
    let jail = WorkspacePathJail::new(root)?;
    for (path, bytes) in files {
        let path = jail.resolve(path)?;
        fs::create_dir_all(
            path.parent()
                .ok_or_else(|| Error::Validation("output has no parent".into()))?,
        )
        .map_err(|e| Error::Internal(e.to_string()))?;
        fs::write(path, bytes).map_err(|e| Error::Internal(e.to_string()))?;
    }
    Ok(())
}

fn deliver(root: &Path, mutations: &[PlannedMutation]) -> Result<(), Error> {
    #[cfg(test)]
    if let Some(index) = test_hooks::delivery_failure() {
        return crate::workspace_mutation::delivery::apply_admitted_mutations_with_test_failure(
            root, mutations, index,
        )
        .map(drop)
        .map_err(Into::into);
    }
    crate::workspace_mutation::apply_admitted_mutations(root, mutations)
        .map(drop)
        .map_err(Into::into)
}

/// Depth-only rendering uses the attested scratch root for capability inventory too. The
/// ordinary shared planner retains its established production workspace resolution.
fn publication_outputs(
    root: &Path,
    context: &RefreshContext,
) -> Result<Vec<PublicationArtifactFile>, Error> {
    let packet = &context.publication_packet;
    let expected = expected_publication_output_paths(
        packet.support_publication_enabled,
        packet.capability_publication_enabled,
    );
    if packet.required_publication_outputs != expected {
        return Err(Error::Validation(format!(
            "`{}` required_publication_outputs do not match the frozen publication planning seam",
            context.publication_packet_path,
        )));
    }
    let mut files =
        build_publication_artifact_plan(root, packet.support_publication_enabled, false)
            .map_err(Error::Validation)?;
    if packet.capability_publication_enabled {
        files.push(PublicationArtifactFile {
            relative_path: CAPABILITY_MATRIX_OUTPUT_PATH.into(),
            contents: capability_matrix::generate_markdown_for_workspace(root)
                .map_err(Error::Validation)?
                .into_bytes(),
        });
    }
    if files
        .iter()
        .map(|file| &file.relative_path)
        .collect::<Vec<_>>()
        != packet
            .required_publication_outputs
            .iter()
            .collect::<Vec<_>>()
    {
        return Err(Error::Validation(
            "ON4 planned publication paths differ from its packet".into(),
        ));
    }
    Ok(files)
}
