use super::request::MaintenanceRequest;

const OWNERSHIP_MARKER: &str = "<!-- generated-by: xtask refresh-agent; owner: control-plane -->";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedPacketDoc {
    pub relative_path: String,
    pub contents: String,
}

pub fn build_packet_docs(request: &MaintenanceRequest) -> Vec<RenderedPacketDoc> {
    let root = &request.maintenance_root;
    let actions = markdown_list(
        &request
            .requested_control_plane_actions
            .iter()
            .map(|action| format!("`{}`", action.as_str()))
            .collect::<Vec<_>>(),
    );
    let runtime_followup = if request.runtime_followup_required.required {
        markdown_list(&request.runtime_followup_required.items)
    } else {
        "- none recorded".to_string()
    };
    let trigger_context = render_trigger_context(request);
    let threading = if request.is_automated_watch_request() {
        format!(
            "# Threading\n\n1. Review the auto-generated request at `{}`.\n2. Confirm the detected release metadata and staged branch name under `{}`.\n3. Follow the packet to complete the maintenance work and close the lane with `close-agent-maintenance`.\n",
            request.relative_path, request.maintenance_root
        )
    } else {
        format!(
            "# Threading\n\n1. Run `check-agent-drift --agent {}`.\n2. Record the maintainer-authored request at `{}`.\n3. Apply `refresh-agent --dry-run` and `refresh-agent --write` using that request.\n4. Close the maintenance run with `close-agent-maintenance` once findings are resolved or explicitly deferred.\n",
            request.agent_id, request.relative_path
        )
    };

    vec![
        RenderedPacketDoc {
            relative_path: format!("{root}/README.md"),
            contents: wrap_markdown(&format!(
                "# {} maintenance\n\nThis packet tracks control-plane maintenance for `{}`.\n\n## Request\n\n- request artifact: `{}`\n- trigger kind: `{}`\n- basis ref: `{}`\n- opened from: `{}`\n- recorded at: `{}`\n- request commit: `{}`\n\n## Trigger context\n\n{}\n\n## Requested control-plane actions\n\n{}\n",
                request.agent_id,
                request.agent_id,
                request.relative_path,
                request.trigger_kind.as_str(),
                request.basis_ref,
                request.opened_from,
                request.request_recorded_at,
                request.request_commit,
                trigger_context,
                actions
            )),
        },
        RenderedPacketDoc {
            relative_path: format!("{root}/scope_brief.md"),
            contents: wrap_markdown(&format!(
                "# Scope brief\n\nThis maintenance lane is limited to control-plane docs and generated publication surfaces for `{}`.\n\nAllowed write envelope:\n\n- maintenance packet docs under `{}`\n- `cli_manifests/support_matrix/current.json`\n- `docs/specs/unified-agent-api/support-matrix.md`\n- `docs/specs/unified-agent-api/capability-matrix.md`\n- `docs/crates-io-release.md`\n\nHistorical onboarding and implementation packet docs remain read-only inputs.\n",
                request.agent_id,
                request.maintenance_root
            )),
        },
        RenderedPacketDoc {
            relative_path: format!("{root}/seam_map.md"),
            contents: wrap_markdown(
                "# Seam map\n\nThis maintenance packet has one bounded seam: reconcile maintenance-owned docs and generated publication surfaces with the detector-emitted drift basis.\n",
            ),
        },
        RenderedPacketDoc {
            relative_path: format!("{root}/threading.md"),
            contents: wrap_markdown(&threading),
        },
        RenderedPacketDoc {
            relative_path: format!("{root}/review_surfaces.md"),
            contents: wrap_markdown(&format!(
                "# Review surfaces\n\n- `{}`\n- `{}`\n- `docs/specs/unified-agent-api/support-matrix.md`\n- `cli_manifests/support_matrix/current.json`\n- `docs/specs/unified-agent-api/capability-matrix.md`\n- `docs/crates-io-release.md`\n- historical packet docs are detector inputs only and remain read-only\n",
                request.basis_ref, request.opened_from
            )),
        },
        RenderedPacketDoc {
            relative_path: format!("{root}/HANDOFF.md"),
            contents: wrap_markdown(&format!(
                "# Handoff\n\nCurrent maintenance request: `{}`.\n\n## Packet origin\n\n{}\n\n## Runtime follow-up\n\n- required: `{}`\n{}\n\n## Operator note\n\nMaintenance closeout is not finalized by `refresh-agent`; runtime-owned changes, if any, stay outside this write set.\n",
                request.relative_path,
                trigger_context,
                if request.runtime_followup_required.required {
                    "true"
                } else {
                    "false"
                },
                runtime_followup
            )),
        },
        RenderedPacketDoc {
            relative_path: format!("{root}/governance/remediation-log.md"),
            contents: wrap_markdown(&format!(
                "# Remediation log\n\nRefresh planned from `{}`.\n\n- basis ref: `{}`\n- trigger kind: `{}`\n- request sha256: `{}`\n\nNo maintenance closeout has been recorded yet.\n",
                request.relative_path,
                request.basis_ref,
                request.trigger_kind.as_str(),
                request.sha256
            )),
        },
    ]
}

fn wrap_markdown(body: &str) -> String {
    format!("{OWNERSHIP_MARKER}\n\n{body}")
}

fn markdown_list(items: &[String]) -> String {
    items
        .iter()
        .map(|item| format!("- {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_trigger_context(request: &MaintenanceRequest) -> String {
    if let Some(detected_release) = &request.detected_release {
        format!(
            "- detected_by: `{}`\n- current_validated: `{}`\n- target_version: `{}`\n- latest_stable: `{}`\n- version_policy: `{}`\n- source_kind: `{}`\n- source_ref: `{}`\n- dispatch_kind: `{}`\n- dispatch_workflow: `{}`\n- branch_name: `{}`",
            detected_release.detected_by,
            detected_release.current_validated,
            detected_release.target_version,
            detected_release.latest_stable,
            detected_release.version_policy,
            detected_release.source_kind,
            detected_release.source_ref,
            detected_release.dispatch_kind,
            detected_release.dispatch_workflow,
            detected_release.branch_name
        )
    } else {
        "- no automated release detection metadata recorded".to_string()
    }
}
