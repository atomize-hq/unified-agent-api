//! ON2 repair owns runtime reporting and preserves the successful generation's inventory.
use super::*;
use crate::{
    candidate_execution::tree::{self, Node},
    isolated_maintenance::{CapturedRegistry, Route},
    runtime_advertising::source::Source,
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn run(
    root: &Path,
    args: Args,
    route: Route,
    writer: &mut impl Write,
) -> Result<(), Error> {
    if !route.context().bootstrap_declared_onboarding() {
        // Maintenance and unresolved bootstrap scopes confer no repair authority.
        crate::depth_routes::refuse_workspace(root, "repair_runtime_evidence")
            .map_err(Error::Validation)?;
        return Err(Error::Validation(
            "repair requires a declared onboarding generation".into(),
        ));
    }
    if args.check == args.write {
        return Err(Error::Validation(
            "choose exactly one of --check or --write".into(),
        ));
    }
    let captured = route.capture(root).map_err(Error::Validation)?;
    let proposal = captured.proposal(root).map_err(Error::Validation)?;
    let (context, inventory) = context(root, &args, &captured)?;
    let run_id = repair_run_id(&context.approval.descriptor.agent_id);
    let relative = format!("{RUNTIME_RUNS_ROOT}/{run_id}");
    write_header(writer, &context, &run_id, args.write)?;
    let scratch = tempfile::tempdir().map_err(|e| Error::Internal(e.to_string()))?;
    let view = proposal
        .isolated_view(root, scratch.path())
        .map_err(Error::Validation)?;
    let before = tree::capture(scratch.path(), &BTreeSet::new()).map_err(internal)?;
    let files = {
        let _binding = captured.bind_view(&view).map_err(Error::Validation)?;
        prepare(scratch.path(), &context, &run_id, &relative, &inventory)?;
        let after = tree::capture(scratch.path(), &BTreeSet::new()).map_err(internal)?;
        outputs(&before, &after, &context.lifecycle_state_path, &relative)?
    };
    let mutations = proposal
        .plan_with(root, files, |workspace, before, after, _| {
            // Authenticate the actual proposed lifecycle, record, acquired inputs and aggregate
            // reporting effects. Command names and a copied contract supply no admission.
            crate::depth_acceptance::check_captured_relay(before, after, workspace)?;
            Ok(BTreeMap::new())
        })
        .map_err(Error::Validation)?;
    captured.recheck(root).map_err(Error::Validation)?;
    view.verify_current(root).map_err(Error::Validation)?;
    if args.write {
        deliver(root, &mutations).map_err(|e| Error::Internal(e.to_string()))?;
    }
    writeln!(
        writer,
        "OK: repair-runtime-evidence {}.\nrun_id: {run_id}\nrun_dir: {relative}\nwritten_paths: {}",
        if args.write {
            "write complete"
        } else {
            "check passed"
        },
        inventory_paths(&inventory)?.len()
    )
    .map_err(|e| Error::Internal(e.to_string()))
}

fn context(
    root: &Path,
    args: &Args,
    captured: &CapturedRegistry,
) -> Result<(RepairContext, Vec<u8>), Error> {
    let source = Source::workspace(root).map_err(Error::Validation)?;
    let raw = source
        .optional_bytes(&args.approval)
        .map_err(Error::Validation)?
        .ok_or_else(|| Error::Validation("missing onboarding approval".into()))?;
    let approval =
        crate::runtime_follow_on::depth::approval::execution(root, &raw, &args.approval, captured)
            .map_err(Error::Validation)?;
    let path = agent_lifecycle::lifecycle_state_path(&approval.descriptor.onboarding_pack_prefix);
    let state = agent_lifecycle::load_lifecycle_state(root, &path)
        .map_err(|e| Error::Validation(e.to_string()))?;
    validate_approval_continuity(&state, &path, &approval)?;
    if state.agent_id != approval.descriptor.agent_id
        || state.onboarding_pack_prefix != approval.descriptor.onboarding_pack_prefix
        || state.lifecycle_stage != LifecycleStage::RuntimeIntegrated
    {
        return Err(Error::Validation(
            "routed repair requires the frozen runtime_integrated generation".into(),
        ));
    }
    let depth = SupportDepthContract::copy_from(&approval, &state).map_err(Error::Validation)?;
    crate::prepare_publication::acquired_inputs::check(root, &args.approval)
        .map_err(Error::Validation)?;
    let host = captured.evidence_host(root).map_err(Error::Validation)?;
    let tip = Source::tip(root, host.tip()).map_err(Error::Validation)?;
    let prior: LifecycleState = serde_json::from_str(&tip.read(&path).map_err(Error::Validation)?)
        .map_err(|e| Error::Validation(e.to_string()))?;
    if tip
        .optional_bytes(&args.approval)
        .map_err(Error::Validation)?
        .as_ref()
        != Some(&raw)
    {
        return Err(Error::Validation(
            "repair approval differs from protected generation".into(),
        ));
    }
    // Authenticate success against protected state before preserving or repointing its binding.
    if prior.lifecycle_stage != LifecycleStage::RuntimeIntegrated {
        return Err(Error::Validation(
            "routed repair requires the protected runtime_integrated generation".into(),
        ));
    }
    if prior.support_depth != state.support_depth
        || prior.active_runtime_evidence_run_id != state.active_runtime_evidence_run_id
        || prior.written_paths_sha256 != state.written_paths_sha256
    {
        return Err(Error::Validation(
            "repair success state differs from protected generation".into(),
        ));
    }
    let record = format!(
        "{}/reports/{}/depth-record.json",
        approval.descriptor.manifest_root,
        approval
            .support_depth_version
            .as_deref()
            .ok_or_else(|| Error::Validation("missing depth version".into()))?
    );
    if source.optional_bytes(&record).map_err(Error::Validation)?
        != tip.optional_bytes(&record).map_err(Error::Validation)?
    {
        return Err(Error::Validation(
            "repair must preserve the protected depth record and entries".into(),
        ));
    }
    let run_id = state
        .active_runtime_evidence_run_id
        .as_deref()
        .ok_or_else(|| Error::Validation("repair requires the successful runtime run id".into()))?;
    crate::runtime_evidence_run::validate_run_id(run_id).map_err(Error::Validation)?;
    let run = format!("{RUNTIME_RUNS_ROOT}/{run_id}");
    let contract: serde_json::Value = serde_json::from_str(
        &source
            .read(&format!("{run}/input-contract.json"))
            .map_err(Error::Validation)?,
    )
    .map_err(|e| Error::Validation(e.to_string()))?;
    if contract["workflow_version"] != WORKFLOW_VERSION
        || contract["run_id"] != run_id
        || contract["approval_artifact_path"] != approval.relative_path
        || contract["approval_artifact_sha256"] != approval.sha256
        || contract["agent_id"] != approval.descriptor.agent_id
        || contract["manifest_root"] != approval.descriptor.manifest_root
        || contract["support_depth"]
            != serde_json::to_value(&depth).map_err(|e| Error::Internal(e.to_string()))?
    {
        return Err(Error::Validation(
            "repair input contract differs from frozen success state".into(),
        ));
    }
    let inventory = source
        .optional_bytes(&format!("{run}/written-paths.json"))
        .map_err(Error::Validation)?
        .ok_or_else(|| Error::Validation("missing lifecycle-bound written paths".into()))?;
    if state.written_paths_sha256.as_deref().map(|v| v.as_str())
        != Some(hex::encode(Sha256::digest(&inventory)).as_str())
    {
        return Err(Error::Validation(
            "repair written-paths digest differs from lifecycle success state".into(),
        ));
    }
    for path in inventory_paths(&inventory)? {
        if source
            .optional_bytes(&path)
            .map_err(Error::Validation)?
            .is_none()
        {
            return Err(Error::Validation(format!(
                "missing successful runtime output: {path}"
            )));
        }
    }
    captured.recheck(root).map_err(Error::Validation)?;
    Ok((
        RepairContext {
            approval,
            lifecycle_state_path: path,
            lifecycle_state: state,
            support_depth: depth,
        },
        inventory,
    ))
}

fn prepare(
    root: &Path,
    context: &RepairContext,
    run_id: &str,
    relative: &str,
    inventory: &[u8],
) -> Result<(), Error> {
    let run = root.join(relative);
    write_runtime_evidence_bundle_at(
        root,
        &run,
        relative,
        Path::new(relative),
        &context.approval,
        &context.lifecycle_state,
        &RuntimeEvidenceBundleSpec {
            run_id,
            host_surface: HOST_SURFACE,
            loaded_skill_ref: LOADED_SKILL_REF,
            mode: "repair",
            source_label: "repair-runtime-evidence",
            summary_title: "Runtime Evidence Repair",
            validation_check_name: "runtime_evidence_repair",
            validation_message:
                "runtime evidence was reconstructed from committed runtime-owned outputs",
        },
    )
    .map_err(map_bundle_error)?;
    copy_depth_contract(&run, context.support_depth.as_ref())?;
    runtime_evidence_bundle::preserve_written_paths(&run, relative, &context.approval, inventory)
        .map_err(map_bundle_error)?;
    let mut next = context.lifecycle_state.clone();
    next.active_runtime_evidence_run_id = Some(run_id.into());
    next.current_owner_command = "repair-runtime-evidence --write".into();
    next.last_transition_at = now_rfc3339().map_err(|e| Error::Internal(e.to_string()))?;
    next.last_transition_by = "xtask repair-runtime-evidence --write".into();
    fs::write(
        root.join(&context.lifecycle_state_path),
        serde_json::to_vec_pretty(&next).map_err(|e| Error::Internal(e.to_string()))?,
    )
    .map_err(|e| Error::Internal(e.to_string()))?;
    maybe_mutate_staged_bundle(&run)?;
    if load_lifecycle_state_relaxed(root, &context.lifecycle_state_path)? != next {
        return Err(Error::Validation(
            "repair must preserve lifecycle stage, depth object and written-path integrity".into(),
        ));
    }
    crate::prepare_publication::acquired_inputs::check(root, &context.approval.relative_path)
        .map_err(Error::Validation)?;
    validate_runtime_evidence_run_for_approval(root, &context.approval, run_id)
        .map_err(|e| Error::Validation(e.to_string()))?;
    // The general historical validator deliberately reads a smaller contract projection.
    let contract: serde_json::Value = serde_json::from_slice(
        &fs::read(run.join("input-contract.json")).map_err(|e| Error::Internal(e.to_string()))?,
    )
    .map_err(|e| Error::Validation(e.to_string()))?;
    if contract["support_depth"]
        != serde_json::to_value(&context.support_depth)
            .map_err(|e| Error::Internal(e.to_string()))?
        || contract["run_id"] != run_id
        || contract["workflow_version"] != WORKFLOW_VERSION
    {
        return Err(Error::Validation(
            "repair changed frozen runtime contract".into(),
        ));
    }
    Ok(())
}

fn inventory_paths(raw: &[u8]) -> Result<Vec<String>, Error> {
    let paths: Vec<String> =
        serde_json::from_slice(raw).map_err(|e| Error::Validation(e.to_string()))?;
    if paths.is_empty() {
        return Err(Error::Validation(
            "repair requires nonempty lifecycle-bound written paths".into(),
        ));
    }
    Ok(paths)
}

fn outputs(
    before: &tree::Tree,
    after: &tree::Tree,
    state: &str,
    run: &str,
) -> Result<Vec<(PathBuf, Vec<u8>)>, Error> {
    let allowed: BTreeSet<_> = crate::runtime_evidence_run::REQUIRED_RUNTIME_EVIDENCE_FILES
        .iter()
        .map(|name| format!("{run}/{name}"))
        .chain([state.to_owned()])
        .collect();
    let mut files = Vec::new();
    for path in tree::changed(before, after) {
        if let Some(Node::Directory(_)) = after.get(&path) {
            if !before.contains_key(&path)
                && allowed.iter().any(|p| p.starts_with(&format!("{path}/")))
            {
                continue;
            }
        }
        let Some(Node::File(bytes, _)) = after.get(&path).filter(|_| allowed.contains(&path))
        else {
            return Err(Error::Validation(format!(
                "repair changed protected output: {path}"
            )));
        };
        files.push((path.into(), bytes.clone()));
    }
    Ok(files)
}

fn internal(error: crate::candidate_execution::Error) -> Error {
    Error::Internal(error.to_string())
}

#[cfg(test)]
thread_local! {
    pub(crate) static DELIVERY_FAILURE: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
}
fn deliver(
    root: &Path,
    mutations: &[crate::workspace_mutation::PlannedMutation],
) -> Result<
    crate::workspace_mutation::ApplySummary,
    crate::workspace_mutation::WorkspaceMutationError,
> {
    #[cfg(test)]
    if let Some(index) = DELIVERY_FAILURE.with(|slot| slot.get()) {
        return crate::workspace_mutation::delivery::apply_admitted_mutations_with_test_failure(
            root, mutations, index,
        );
    }
    crate::workspace_mutation::apply_admitted_mutations(root, mutations)
}
