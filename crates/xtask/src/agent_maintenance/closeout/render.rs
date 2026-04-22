use serde::Serialize;

use super::{
    DeferredFindingsTruth, LinkedMaintenanceCloseout, MaintenanceCloseout,
    MaintenanceCloseoutError, MaintenanceFinding,
};

#[derive(Debug, Serialize)]
struct SerializableMaintenanceCloseout<'a> {
    request_ref: &'a str,
    request_sha256: &'a str,
    resolved_findings: Vec<SerializableMaintenanceFinding<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deferred_findings: Option<Vec<SerializableMaintenanceFinding<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    explicit_none_reason: Option<&'a str>,
    preflight_passed: bool,
    recorded_at: &'a str,
    commit: &'a str,
}

#[derive(Debug, Serialize)]
struct SerializableMaintenanceFinding<'a> {
    category_id: &'a str,
    summary: &'a str,
    surfaces: &'a [String],
}

pub(crate) fn serialize_closeout_json(
    closeout: &MaintenanceCloseout,
) -> Result<Vec<u8>, MaintenanceCloseoutError> {
    let serializable = SerializableMaintenanceCloseout {
        request_ref: &closeout.request_ref,
        request_sha256: &closeout.request_sha256,
        resolved_findings: closeout
            .resolved_findings
            .iter()
            .map(SerializableMaintenanceFinding::from)
            .collect(),
        deferred_findings: match &closeout.deferred_findings {
            DeferredFindingsTruth::Findings(findings) => Some(
                findings
                    .iter()
                    .map(SerializableMaintenanceFinding::from)
                    .collect(),
            ),
            DeferredFindingsTruth::ExplicitNone(_) => None,
        },
        explicit_none_reason: match &closeout.deferred_findings {
            DeferredFindingsTruth::Findings(_) => None,
            DeferredFindingsTruth::ExplicitNone(reason) => Some(reason.as_str()),
        },
        preflight_passed: closeout.preflight_passed,
        recorded_at: &closeout.recorded_at,
        commit: &closeout.commit,
    };
    let mut bytes = serde_json::to_vec_pretty(&serializable).map_err(|err| {
        MaintenanceCloseoutError::Internal(format!("serialize maintenance closeout: {err}"))
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub(crate) fn render_markdown_file(body: String) -> String {
    format!("{}\n\n{body}", super::OWNERSHIP_MARKER)
}

pub(crate) fn render_handoff_body(linked: &LinkedMaintenanceCloseout) -> String {
    let actions = linked
        .request
        .requested_control_plane_actions
        .iter()
        .map(|action| format!("- `{}`", action.as_str()))
        .collect::<Vec<_>>()
        .join("\n");
    let resolved_findings = render_findings(&linked.closeout.resolved_findings);
    let deferred_findings = match &linked.closeout.deferred_findings {
        DeferredFindingsTruth::Findings(findings) => render_findings(findings),
        DeferredFindingsTruth::ExplicitNone(reason) => {
            format!("- No deferred findings remain: {reason}")
        }
    };
    let runtime_followup = if linked.request.runtime_followup_required.required {
        linked
            .request
            .runtime_followup_required
            .items
            .iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        "- No runtime follow-up is currently required.".to_string()
    };

    format!(
        "# Handoff\n\nThis packet records the closed maintenance run for `{}`.\n\n## Request linkage\n\n- request ref: `{}`\n- request sha256: `{}`\n- trigger kind: `{}`\n- basis ref: `{}`\n- opened from: `{}`\n- requested control-plane actions:\n{}\n\n## Closeout\n\n- closeout metadata: `{}`\n- preflight passed: `{}`\n- recorded at: `{}`\n- commit: `{}`\n\n## Resolved findings\n\n{}\n\n## Deferred findings\n\n{}\n\n## Runtime follow-up\n\n{}\n",
        linked.request.agent_id,
        linked.closeout.request_ref,
        linked.request_sha256,
        linked.request.trigger_kind.as_str(),
        linked.request.basis_ref,
        linked.request.opened_from,
        actions,
        linked.closeout_path.display(),
        linked.closeout.preflight_passed,
        linked.closeout.recorded_at,
        linked.closeout.commit,
        resolved_findings,
        deferred_findings,
        runtime_followup
    )
}

pub(crate) fn render_remediation_log_body(linked: &LinkedMaintenanceCloseout) -> String {
    let resolved_findings = render_findings(&linked.closeout.resolved_findings);
    let deferred_findings = match &linked.closeout.deferred_findings {
        DeferredFindingsTruth::Findings(findings) => render_findings(findings),
        DeferredFindingsTruth::ExplicitNone(reason) => {
            format!("- No deferred findings remain: {reason}")
        }
    };

    format!(
        "# Remediation log\n\n## Request\n\n- request ref: `{}`\n- request sha256: `{}`\n- request recorded at: `{}`\n- request commit: `{}`\n\n## Resolved findings\n\n{}\n\n## Deferred findings\n\n{}\n",
        linked.closeout.request_ref,
        linked.request_sha256,
        linked.request.request_recorded_at,
        linked.request.request_commit,
        resolved_findings,
        deferred_findings
    )
}

fn render_findings(findings: &[MaintenanceFinding]) -> String {
    findings
        .iter()
        .map(|finding| {
            let surfaces = finding
                .surfaces
                .iter()
                .map(|surface| format!("  - {surface}"))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "- [{}] {}\n  surfaces:\n{}",
                finding.category_id.as_id(),
                finding.summary,
                surfaces
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

impl<'a> From<&'a MaintenanceFinding> for SerializableMaintenanceFinding<'a> {
    fn from(value: &'a MaintenanceFinding) -> Self {
        Self {
            category_id: value.category_id.as_id(),
            summary: &value.summary,
            surfaces: &value.surfaces,
        }
    }
}
