//! Pure MA2 request and complete preparation packet rendering.
use super::*;

#[allow(clippy::too_many_arguments)]
fn render_packet(
    workspace_root: &Path,
    args: &Args,
    entry: &AgentRegistryEntry,
    detected_release: DetectedRelease,
    execution_contract: ExecutionContract,
    support_surface_audit: SupportSurfaceAudit,
    freeze: Option<&crate::support_depth::FreezePlan>,
    prompt: String,
) -> Result<(MaintenanceRequest, Vec<PreparedFile>), Error> {
    let request_path = maintenance_request_path(&args.agent);
    let maintenance_root = format!("docs/agents/lifecycle/{}-maintenance", args.agent);
    let basis_ref = format!("{}/latest_validated.txt", entry.manifest_root);
    let support_depth = freeze.map(|plan| request::SupportDepthBinding {
        policy_identity: plan.record.policy.identity.clone(),
        obligations_identity: plan.record.obligations.as_ref().map(|o| o.identity.clone()),
    });
    let request_bytes = render_request_toml(
        args,
        &basis_ref,
        &detected_release,
        &support_surface_audit,
        &execution_contract,
        support_depth.as_ref(),
    )?
    .into_bytes();
    let request = MaintenanceRequest {
        relative_path: request_path.clone(),
        canonical_path: workspace_root.join(&request_path),
        sha256: hex::encode(Sha256::digest(&request_bytes)),
        maintenance_pack_prefix: format!("{}-maintenance", args.agent),
        maintenance_root,
        agent_id: args.agent.clone(),
        trigger_kind: TriggerKind::UpstreamReleaseDetected,
        basis_ref,
        opened_from: args.opened_from.display().to_string(),
        requested_control_plane_actions: vec![MaintenanceAction::PacketDocRefresh],
        runtime_followup_required: RuntimeFollowupRequired {
            required: false,
            items: Vec::new(),
        },
        detected_release: Some(detected_release),
        support_surface_audit: Some(support_surface_audit),
        support_depth,
        request_recorded_at: args.request_recorded_at.clone(),
        request_commit: args.request_commit.clone(),
    };
    let mut files = vec![PreparedFile {
        relative_path: request.relative_path.clone(),
        contents: request_bytes,
    }];
    for doc in super::super::docs::render_preparation_packet_docs(
        &request,
        &execution_contract,
        entry,
        super::super::docs::render_execution_packet_from_prompt(
            &request,
            &execution_contract,
            prompt,
        )
        .map_err(Error::Validation)?,
    )
    .map_err(Error::Validation)?
    {
        files.push(PreparedFile {
            relative_path: doc.relative_path,
            contents: doc.contents.into_bytes(),
        });
    }
    Ok((request, files))
}

/// Derived from one checked writer view. Fields stay private to the owning module.
pub(crate) struct Inputs {
    workspace: PathBuf,
    args: Args,
    entry: AgentRegistryEntry,
    detected: DetectedRelease,
    contract: ExecutionContract,
    audit: SupportSurfaceAudit,
    freeze: Option<crate::support_depth::FreezePlan>,
    prompt: String,
    publication: Option<crate::support_matrix::PreparationInputs>,
}
impl Inputs {
    pub(super) fn classification_bases(&self) -> Vec<(String, String)> {
        self.freeze
            .iter()
            .flat_map(|plan| &plan.classification_bases)
            .map(|basis| (basis.operation.clone(), basis.sha256.clone()))
            .collect()
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        workspace: &Path,
        args: &Args,
        entry: &AgentRegistryEntry,
        detected: DetectedRelease,
        contract: ExecutionContract,
        audit: SupportSurfaceAudit,
        freeze: Option<crate::support_depth::FreezePlan>,
        prompt: String,
        publication: Option<crate::support_matrix::PreparationInputs>,
    ) -> Self {
        Self {
            workspace: workspace.to_owned(),
            args: args.clone(),
            entry: entry.clone(),
            detected,
            contract,
            audit,
            freeze,
            prompt,
            publication,
        }
    }
}
impl crate::freeze_transition::WriterInputs for Inputs {
    type Request = MaintenanceRequest;
    fn render(&self) -> Result<crate::freeze_transition::Rendered<Self::Request>, String> {
        let (request, mut files) = render_packet(
            &self.workspace,
            &self.args,
            &self.entry,
            self.detected.clone(),
            self.contract.clone(),
            self.audit.clone(),
            self.freeze.as_ref(),
            self.prompt.clone(),
        )
        .map_err(|e| e.to_string())?;
        if let Some(freeze) = &self.freeze {
            let bundle = self
                .publication
                .as_ref()
                .ok_or("missing freeze reporting inputs")?
                .render()?;
            depth::append(freeze, &self.entry, bundle, &mut files).map_err(|e| e.to_string())?;
        }
        Ok(crate::freeze_transition::Rendered {
            request: Some(request),
            files: files
                .into_iter()
                .map(|file| (file.relative_path, file.contents))
                .collect(),
        })
    }
}
