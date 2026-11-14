//! Approved ON2 re-derivation from a predecessor-only data view and named invocation inputs.
use super::*;
use crate::depth_acceptance::admission::freeze::authority::Writer;

pub(crate) fn from_predecessor(
    workspace: &Path,
    parent: &Source,
    registry: &AgentRegistry,
    entry: &crate::agent_registry::AgentRegistryEntry,
    writer: &Writer,
) -> Result<Inputs, String> {
    let Writer::Onboarding {
        run_id,
        contract_generated_at,
        packet_generated_at,
        requested_tier,
        minimal_justification_file,
        allow_rich_surface,
    } = writer
    else {
        return Err("freeze writer differs from onboarding generation".into());
    };
    writer.validate()?;
    if let Some(path) = minimal_justification_file {
        parent
            .optional_bytes(path)?
            .ok_or("freeze justification lacks predecessor provenance")?;
    }
    let approval_path = agent_lifecycle::approval_artifact_path_for_entry(entry);
    let raw = parent.read(&approval_path)?;
    let approval = crate::isolated_maintenance::onboarding::approval::read(
        raw.as_bytes(),
        &approval_path,
        entry,
    )?;
    let version = approval
        .support_depth_version
        .as_deref()
        .ok_or("missing freeze version")?;
    let state_path = agent_lifecycle::lifecycle_state_path_for_entry(entry);
    let mut state: LifecycleState =
        serde_json::from_str(&parent.read(&state_path)?).map_err(|e| e.to_string())?;
    state
        .validate_in_workspace(workspace)
        .map_err(|e| e.to_string())?;
    if state.agent_id != approval.agent_id
        || state.onboarding_pack_prefix != approval.onboarding_pack_prefix
        || state.approval_artifact_path
            != agent_lifecycle::approval_artifact_path(&approval.onboarding_pack_prefix)
    {
        return Err("freeze predecessor lifecycle attribution differs from approval".into());
    }
    if state.lifecycle_stage != agent_lifecycle::LifecycleStage::Enrolled {
        return Err("freeze predecessor is beyond enrolled".into());
    }
    let declaration = entry
        .support_depth
        .as_ref()
        .and_then(|d| d.enrollments.as_ref())
        .and_then(|rows| rows.iter().find(|row| row.version == version))
        .ok_or("missing freeze enrollment")?;
    let mut view = GitInputs::new(workspace)?;
    rule_four(
        &mut view,
        parent,
        &entry.agent_id,
        &entry.manifest_root,
        version,
        &declaration.targets,
    )?;
    complete_union(&mut view, &entry.manifest_root, version)?;
    let plan = FreezePlan::onboarding(&mut view, &approval, state.support_depth.as_deref())?;
    if let Some(obligations) = &plan.record.obligations {
        let statement = integration::statement(
            parent,
            &approval.onboarding_pack_prefix,
            &entry.agent_id,
            version,
        )?
        .ok_or("acquisition statement missing at freeze predecessor")?;
        if let Some(reason) = statement.mismatch(&obligations.acquired_inputs) {
            return Err(reason);
        }
    }
    state.support_depth = Some(
        agent_lifecycle::Preserved::new(agent_lifecycle::LifecycleDepth {
            policy_identity: plan.record.policy.identity.clone(),
            obligations_identity: plan.record.obligations.as_ref().map(|o| o.identity.clone()),
        })
        .map_err(|e| e.to_string())?,
    );
    let args = Args {
        approval: approval_path,
        dry_run: true,
        write: false,
        requested_tier: *requested_tier,
        minimal_justification_file: minimal_justification_file.clone(),
        allow_rich_surface: allow_rich_surface.clone(),
        run_id: Some(run_id.clone()),
        codex_binary: None,
    };
    validate_args(&args, workspace).map_err(|e| e.to_string())?;
    let context = packet::context_from_predecessor(
        workspace,
        &args,
        registry,
        state_path,
        state,
        contract_generated_at.clone(),
    )?;
    let publication = crate::support_matrix::load_preparation(workspace, registry, &plan.record)?;
    view.verify()?;
    Ok(Inputs {
        args,
        context,
        record: plan.record,
        publication,
        packet_generated_at: packet_generated_at.clone(),
    })
}

/// Pure complete ON2 writer inputs. Construction stays within the owning derivation module.
pub(crate) struct Inputs {
    args: Args,
    context: RuntimeContext,
    record: crate::depth_record::DepthRecord,
    publication: crate::support_matrix::PreparationInputs,
    packet_generated_at: String,
}
impl Inputs {
    /// Run packets are local preparation artifacts. Only packet paths already tracked or
    /// actually present in the candidate join the committed output inventory. Every such
    /// entry still comes from the complete renderer, never from candidate-selected bytes.
    pub(crate) fn committed_plan(
        &self,
        predecessor: &crate::freeze_transition::Tree,
        acquisition: &crate::freeze_transition::ValidatedAcquisitionDelta,
        candidate: &crate::freeze_transition::Tree,
    ) -> Result<crate::freeze_transition::Plan, String> {
        use crate::freeze_transition::WriterInputs;
        let packet_paths =
            packet::render(&self.args, &self.context, self.packet_generated_at.clone())
                .map_err(|e| e.to_string())?
                .into_iter()
                .map(|(path, _)| path)
                .collect::<std::collections::BTreeSet<_>>();
        crate::freeze_transition::operations(
            predecessor,
            acquisition,
            self.render()?
                .files
                .into_iter()
                .filter(|(path, _)| {
                    !packet_paths.contains(path)
                        || predecessor.contains_key(path)
                        || candidate.contains_key(path)
                })
                .map(|(path, bytes)| (path, Some(bytes))),
        )
    }
}
impl crate::freeze_transition::WriterInputs for Inputs {
    type Request = ();
    fn render(&self) -> Result<crate::freeze_transition::Rendered<Self::Request>, String> {
        let mut files = packet::render(&self.args, &self.context, self.packet_generated_at.clone())
            .map_err(|e| e.to_string())?;
        let selection = &self.record.policy.selection;
        let record_path = format!(
            "{}/reports/{}/depth-record.json",
            self.context.approval.descriptor.manifest_root, selection.enrollment.version
        );
        files.extend(
            reporting::render(
                &self.record,
                &self.context.lifecycle_state,
                &self.context.lifecycle_state_path,
                &record_path,
                self.publication.render()?,
            )
            .map_err(|e| e.to_string())?,
        );
        Ok(crate::freeze_transition::Rendered {
            request: None,
            files,
        })
    }
}
pub(super) fn writer_inputs(
    workspace: &Path,
    args: &Args,
    run_id: &str,
    freeze: &Freeze,
    contract_generated_at: String,
    packet_generated_at: String,
) -> Result<Inputs, Error> {
    let context = packet::context(workspace, args, run_id, freeze, contract_generated_at)?;
    let mut args = args.clone();
    args.run_id = Some(run_id.into());
    let publication = crate::support_matrix::load_preparation(
        workspace,
        freeze.captured.registry(),
        &freeze.plan.record,
    )
    .map_err(Error::Validation)?;
    Ok(Inputs {
        args,
        context,
        record: freeze.plan.record.clone(),
        publication,
        packet_generated_at,
    })
}
