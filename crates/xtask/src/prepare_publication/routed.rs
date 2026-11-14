//! Controller-owned ON3 reporting, checked as one invocation-local proposal before delivery.
use super::*;
use crate::{
    depth_evidence::EvidenceVerdicts,
    isolated_maintenance::{CapturedRegistry, Route},
    runtime_advertising::source::Source,
};
use std::collections::BTreeMap;

pub(super) fn run(
    root: &Path,
    args: Args,
    route: Route,
    writer: &mut impl Write,
) -> Result<(), Error> {
    if !route.context().bootstrap_declared_onboarding() {
        return Err(Error::Validation(
            "prepare-publication requires a declared onboarding generation".into(),
        ));
    }
    if args.check == args.write {
        return Err(Error::Validation(
            "choose exactly one of --check or --write".into(),
        ));
    }
    let captured = route.capture(root).map_err(Error::Validation)?;
    let proposal = captured.proposal(root).map_err(Error::Validation)?;
    let context = context(root, &args, &captured)?;
    write_header(writer, &context, args.write)?;
    if args.check {
        validate_check_mode(root, &context)?;
        if context.lifecycle_state.lifecycle_stage == LifecycleStage::PublicationReady {
            let version = context
                .approval
                .support_depth_version
                .as_deref()
                .ok_or_else(|| Error::Validation("missing approved depth version".into()))?;
            let record = crate::depth_record::DepthRecord::parse(
                &Source::workspace(root)
                    .and_then(|s| {
                        s.read(&format!(
                            "{}/reports/{version}/depth-record.json",
                            context.entry.manifest_root
                        ))
                    })
                    .map_err(Error::Validation)?,
                &context.entry.agent_id,
                version,
            )
            .map_err(Error::Validation)?;
            let host = captured.evidence_host(root).map_err(Error::Validation)?;
            EvidenceVerdicts::check_reporting_assembly(root, &record, &host)
                .map_err(Error::Validation)?;
        }
        captured.recheck(root).map_err(Error::Validation)?;
        writeln!(writer, "OK: prepare-publication check passed.")
            .map_err(|e| Error::Internal(e.to_string()))?;
        return Ok(());
    }
    let (next, packet) = transition(&context)?;
    if root.join(&context.publication_packet_path).exists() {
        return Err(Error::Validation(
            "publication-ready packet already exists".into(),
        ));
    }
    let host = captured.evidence_host(root).map_err(Error::Validation)?;
    let assembly = EvidenceVerdicts::assemble_reporting(root, &host).map_err(Error::Validation)?;
    let version = context
        .approval
        .support_depth_version
        .as_deref()
        .ok_or_else(|| Error::Validation("missing approved depth version".into()))?;
    let record_path = format!(
        "{}/reports/{version}/depth-record.json",
        context.entry.manifest_root
    );
    let mut record = crate::depth_record::DepthRecord::parse(
        &Source::workspace(root)
            .and_then(|s| s.read(&record_path))
            .map_err(Error::Validation)?,
        &context.entry.agent_id,
        version,
    )
    .map_err(Error::Validation)?;
    let mut frozen = record.clone();
    frozen.execution = None;
    frozen.acceptance_entries.clear();
    if assembly.original != frozen
        || assembly.profile != crate::depth_ci::profile::Profile::SyntheticOnboarding
        || !record.acceptance_entries.is_empty()
    {
        return Err(Error::Validation(
            "ON3 reporting assembly differs from its frozen onboarding generation".into(),
        ));
    }
    record.execution = Some(assembly.execution);
    let mut files: Vec<(PathBuf, Vec<u8>)> = assembly
        .files
        .into_iter()
        .map(|(reference, bytes)| (reference.path.into(), bytes))
        .collect();
    files.extend([
        (record_path.into(), serialize_json_pretty(&record)?),
        (
            context.lifecycle_state_path.clone().into(),
            serialize_json_pretty(&next)?,
        ),
        (
            context.publication_packet_path.clone().into(),
            serialize_json_pretty(&packet)?,
        ),
    ]);
    let scratch = tempfile::tempdir().map_err(|e| Error::Internal(e.to_string()))?;
    let view = proposal
        .isolated_view(root, scratch.path())
        .map_err(Error::Validation)?;
    {
        let _binding = captured.bind_view(&view).map_err(Error::Validation)?;
        let jail = crate::workspace_mutation::WorkspacePathJail::new(scratch.path())
            .map_err(|e| Error::Validation(e.to_string()))?;
        for (path, bytes) in &files {
            let path = jail
                .resolve(path)
                .map_err(|e| Error::Validation(e.to_string()))?;
            fs::create_dir_all(
                path.parent()
                    .ok_or_else(|| Error::Validation("output has no parent".into()))?,
            )
            .map_err(|e| Error::Internal(e.to_string()))?;
            fs::write(path, bytes).map_err(|e| Error::Internal(e.to_string()))?;
        }
        let bundle = EvidenceVerdicts::with_publication_view(
            root,
            scratch.path(),
            &proposal,
            &captured,
            &view,
            || crate::support_matrix::generate_publication_artifacts(scratch.path()),
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
        for (path, bytes) in &files {
            let path = jail
                .resolve(path)
                .map_err(|e| Error::Validation(e.to_string()))?;
            fs::write(path, bytes).map_err(|e| Error::Internal(e.to_string()))?;
        }
        // The aggregate detector mediates every registered destination, including retained rows.
        let proposed = files
            .iter()
            .map(|(path, bytes)| {
                Ok((
                    path.to_str().ok_or("non-UTF8 reporting output")?.to_owned(),
                    String::from_utf8(bytes.clone()).map_err(|e| e.to_string())?,
                ))
            })
            .collect::<Result<_, String>>()
            .map_err(Error::Validation)?;
        crate::depth_acceptance::check_reporting_preparation(root, captured.registry(), proposed)
            .map_err(Error::Validation)?;
    }
    let mutations = proposal
        .plan_with(root, files, |_workspace, before, after, _| {
            // The controller owns reporting only. The relay checks P/O/entries, acquisition and
            // aggregate effects. Rendered facts are authenticated again against the proposed view.
            let _binding = captured.bind_view(&view)?;
            EvidenceVerdicts::with_publication_view(
                root,
                scratch.path(),
                &proposal,
                &captured,
                &view,
                || crate::depth_acceptance::check_captured_relay(before, after, scratch.path()),
            )?;
            Ok(BTreeMap::new())
        })
        .map_err(Error::Validation)?;
    view.verify_current(root).map_err(Error::Validation)?;
    captured.recheck(root).map_err(Error::Validation)?;
    crate::workspace_mutation::apply_admitted_mutations(root, &mutations)
        .map_err(|e| Error::Internal(e.to_string()))?;
    writeln!(
        writer,
        "OK: prepare-publication write complete.\npublication_packet: {}\nruntime_evidence_run: {}",
        context.publication_packet_path, context.runtime_evidence.run_id
    )
    .map_err(|e| Error::Internal(e.to_string()))
}

fn context(
    root: &Path,
    args: &Args,
    captured: &CapturedRegistry,
) -> Result<PublicationContext, Error> {
    acquired_inputs::check(root, &args.approval).map_err(Error::Validation)?;
    let source = Source::workspace(root).map_err(Error::Validation)?;
    let raw = source
        .optional_bytes(&args.approval)
        .map_err(Error::Validation)?
        .ok_or_else(|| Error::Validation("missing onboarding approval".into()))?;
    let approval =
        crate::runtime_follow_on::depth::approval::execution(root, &raw, &args.approval, captured)
            .map_err(Error::Validation)?;
    let entry = captured
        .registry()
        .find(&approval.descriptor.agent_id)
        .cloned()
        .ok_or_else(|| Error::Validation("unknown onboarding agent".into()))?;
    let context = context_for(root, approval, entry)?;
    let host = captured.evidence_host(root).map_err(Error::Validation)?;
    let tip = Source::tip(root, host.tip()).map_err(Error::Validation)?;
    if tip
        .optional_bytes(&args.approval)
        .map_err(Error::Validation)?
        .as_ref()
        != Some(&raw)
    {
        return Err(Error::Validation(
            "ON3 approval differs from protected generation".into(),
        ));
    }
    let prior: LifecycleState = serde_json::from_str(
        &tip.read(&context.lifecycle_state_path)
            .map_err(Error::Validation)?,
    )
    .map_err(|e| Error::Validation(e.to_string()))?;
    let runtime_matches = match (
        prior.lifecycle_stage,
        context.lifecycle_state.lifecycle_stage,
    ) {
        (LifecycleStage::RuntimeIntegrated, LifecycleStage::RuntimeIntegrated) => {
            prior.active_runtime_evidence_run_id
                == context.lifecycle_state.active_runtime_evidence_run_id
        }
        (LifecycleStage::RuntimeIntegrated, LifecycleStage::PublicationReady) => {
            prior.active_runtime_evidence_run_id.as_deref()
                == Some(context.runtime_evidence.run_id.as_str())
        }
        (LifecycleStage::PublicationReady, LifecycleStage::PublicationReady) => true,
        _ => false,
    };
    if prior.support_depth != context.lifecycle_state.support_depth
        || prior.written_paths_sha256 != context.lifecycle_state.written_paths_sha256
        || !runtime_matches
    {
        return Err(Error::Validation(
            "ON3 runtime state differs from protected generation".into(),
        ));
    }
    Ok(context)
}
