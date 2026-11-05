//! CP-P3: private read-only evaluation of an attested post-entry proposal.
use super::*;
use crate::{
    agent_maintenance::closeout_proposal::IsolatedView,
    depth_evidence::EvidenceVerdicts,
    git_inputs::EvaluationSource,
    isolated_maintenance::{Context, EvidenceHost},
};

/// Invocation-local binding; neither the view nor the registry guard supplies evidence authority.
pub(crate) struct ViewEvaluation<'a> {
    view: &'a IsolatedView<'a>,
    context: &'a Context,
    host: &'a EvidenceHost,
    source: EvaluationSource,
    inputs: GitInputs,
    record: DepthRecord,
}
impl EvidenceVerdicts {
    /// Socket-free companion fixture: use the real authenticated MA4 assembler, without
    /// claiming to exercise its delivery channel. The connected test still exercises MA4.
    #[cfg(all(test, target_os = "linux"))]
    pub(crate) fn assemble_view_fixture(
        caller: &Path,
        host: &EvidenceHost,
    ) -> Result<github::assembly::Assembly, String> {
        github::isolated_assembly(caller, host)
    }

    /// Borrow authenticated caller authority, then freeze the exact proposed post-entry record.
    /// Requires the caller's CP-P2 registry/Git-input guard to remain installed on this thread.
    pub(crate) fn bind_view<'a>(
        view: &'a IsolatedView<'a>,
        context: &'a Context,
        host: &'a EvidenceHost,
        record_path: &str,
    ) -> Result<ViewEvaluation<'a>, String> {
        let (source, inputs) = context.view_inputs(view, host)?;
        let p = crate::depth_ci::profile::Profile::Synthetic;
        if record_path
            != format!(
                "cli_manifests/{}/reports/{}/depth-record.json",
                p.agent_id(),
                p.version()
            )
        {
            return Err("view evaluation requires its compiled profile record".into());
        }
        let record = p.record(&source.source.read(record_path)?)?;
        context.verify_view(view, &source, host)?;
        Ok(ViewEvaluation {
            view,
            context,
            host,
            source,
            inputs,
            record,
        })
    }
}
impl ViewEvaluation<'_> {
    #[cfg(test)]
    pub(crate) fn probe_provenance(&self, root: &Path, commit: &str) -> Result<(), String> {
        let _read_only = github::transport::read_only(&self.inputs)?;
        github::transport::source(root, self.host.repository(), commit).map(drop)
    }

    /// Re-authenticate the complete population for this exact record, without any writes.
    pub(crate) fn evaluate(mut self) -> Result<(DepthRecord, EvidenceVerdicts), String> {
        self.context
            .verify_view(self.view, &self.source, self.host)?;
        self.inputs.verify()?;
        let outcome = github::view_record_results(&mut self.inputs, &self.record, self.host);
        // Recheck even when authentication fails; a failure never bypasses freshness.
        let inputs_current = self.inputs.verify();
        let view_current = self.context.verify_view(self.view, &self.source, self.host);
        view_current?;
        inputs_current?;
        Ok((self.record, outcome?))
    }
}
