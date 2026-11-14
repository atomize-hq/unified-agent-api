//! Pure preparation packet inventory and rendering.
use super::*;

pub(crate) fn render_preparation_packet_docs(
    request: &MaintenanceRequest,
    execution_contract: &ExecutionContract,
    entry: &AgentRegistryEntry,
    rendered_execution_packet: RenderedExecutionPacket,
) -> Result<Vec<RenderedPacketDoc>, String> {
    let packet_owned_contract_inputs =
        build_packet_owned_contract_inputs(request, execution_contract, entry)?;
    let root = &request.maintenance_root;
    let trigger_context = render_trigger_context(request);
    let support_audit = render_support_surface_audit(request);

    let mut docs = vec![
        RenderedPacketDoc {
            relative_path: format!("{root}/README.md"),
            contents: wrap_markdown(&format!(
                "# {} maintenance\n\nThis packet tracks automated upstream-release maintenance for `{}`.\n\n## Request\n\n- request artifact: `{}`\n- trigger kind: `{}`\n- basis ref: `{}`\n- opened from: `{}`\n- recorded at: `{}`\n- request commit: `{}`\n\n## Trigger context\n\n{}\n\n## Support-surface audit\n\n{}\n\n## Canonical execution contract\n\nUse `{}` as the exact contributor execution contract for this lane. The PR body summary under `{}` is derivative only.\n",
                request.agent_id,
                request.agent_id,
                request.relative_path,
                request.trigger_kind.as_str(),
                request.basis_ref,
                request.opened_from,
                request.request_recorded_at,
                request.request_commit,
                trigger_context,
                support_audit,
                rendered_execution_packet.handoff_relative_path,
                rendered_execution_packet.pr_summary_relative_path
            )),
        },
        RenderedPacketDoc {
            relative_path: format!("{root}/scope_brief.md"),
            contents: wrap_markdown(&format!(
                "# Scope brief\n\nThis automated maintenance lane is limited to the frozen shared packet for `{}` and the declared writable surfaces below.\n\n## Support uplift rule\n\nNewly discovered non-TUI surface must land in this run unless the frozen packet records one allowed deferral. Preexisting non-TUI gaps remain valid only when the packet ties them to the committed debt inventory.\n\n## Writable surfaces\n\n{}\n",
                request.agent_id,
                markdown_repo_path_list(&execution_contract.writable_surfaces)
            )),
        },
        RenderedPacketDoc {
            relative_path: format!("{root}/seam_map.md"),
            contents: wrap_markdown(
                "# Seam map\n\nThis maintenance packet has one bounded seam: render a contributor-ready execution contract for the detected upstream release while keeping `HANDOFF.md` canonical and `governance/pr-summary.md` derivative, and while freezing one exact support-surface audit for non-TUI uplift.\n",
            ),
        },
        RenderedPacketDoc {
            relative_path: format!("{root}/threading.md"),
            contents: wrap_markdown(&format!(
                "# Threading\n\n1. Review the auto-generated request at `{}` and the canonical contract at `{}`.\n2. Apply the exact coding-agent prompt from `HANDOFF.md` against branch `{}`.\n3. After the green gates pass, the actor handed the packet PR authors `{}` and runs the exact `close-agent-maintenance` command from `HANDOFF.md`. An agent executing this packet inside a relay session must not run that command.\n",
                request.relative_path,
                rendered_execution_packet.handoff_relative_path,
                execution_contract.recovery.reopen_pr_branch,
                execution_contract.closeout_path
            )),
        },
        RenderedPacketDoc {
            relative_path: format!("{root}/review_surfaces.md"),
            contents: wrap_markdown(&format!(
                "# Review surfaces\n\nSome paths below contain guarded lifecycle command names. An agent executing this packet inside a relay session must not invoke those commands.\n\n## Writable surfaces\n\n{}\n\n## Read-only inputs\n\n{}\n\n## Support debt baseline\n\n- `docs/specs/unified-agent-api/non-tui-support-debt.md`\n",
                markdown_repo_path_list(&execution_contract.writable_surfaces),
                markdown_repo_path_list(&execution_contract.read_only_inputs)
            )),
        },
        RenderedPacketDoc {
            relative_path: format!("{root}/governance/remediation-log.md"),
            contents: wrap_markdown(&format!(
                "# Remediation log\n\nRefresh planned from `{}`.\n\n- basis ref: `{}`\n- trigger kind: `{}`\n- request sha256: `{}`\n- canonical handoff: `{}`\n- derivative pr summary: `{}`\n\nNo maintenance closeout has been recorded yet.\n",
                request.relative_path,
                request.basis_ref,
                request.trigger_kind.as_str(),
                request.sha256,
                rendered_execution_packet.handoff_relative_path,
                rendered_execution_packet.pr_summary_relative_path
            )),
        },
    ];
    docs.extend(packet_owned_contract_inputs);
    docs.push(RenderedPacketDoc {
        relative_path: rendered_execution_packet.handoff_relative_path.clone(),
        contents: rendered_execution_packet.handoff_contents,
    });
    docs.push(RenderedPacketDoc {
        relative_path: rendered_execution_packet.pr_summary_relative_path.clone(),
        contents: rendered_execution_packet.pr_summary_contents,
    });
    Ok(docs)
}

fn build_packet_owned_contract_inputs(
    request: &MaintenanceRequest,
    execution_contract: &ExecutionContract,
    entry: &AgentRegistryEntry,
) -> Result<Vec<RenderedPacketDoc>, String> {
    let prompt_path = packet_owned_prompt_template_path(&request.maintenance_root);
    if execution_contract.prompt_template_path != prompt_path {
        return Ok(Vec::new());
    }

    let detected_release = request.detected_release.as_ref().ok_or_else(|| {
        format!(
            "packet-owned contract inputs require detected_release metadata for `{}`",
            request.relative_path
        )
    })?;
    let handoff_path = format!("{}/HANDOFF.md", request.maintenance_root);

    Ok(vec![
        RenderedPacketDoc {
            relative_path: packet_owned_ops_playbook_path(&request.maintenance_root),
            contents: wrap_markdown(&format!(
                "# Ops playbook\n\nThis packet-owned playbook freezes operator context for `{}` target `{}`.\n\nThe recovery packet-regeneration command is a maintainer action run from outside a relay session. An agent executing this packet inside a relay session must not run it.\n\n- request artifact: `{}`\n- basis ref: `{}`\n- opened from: `{}`\n- branch linkage: `{}`\n- canonical handoff: `{}`\n- recovery packet regeneration: `{}`\n",
                request.agent_id,
                detected_release.target_version,
                request.relative_path,
                request.basis_ref,
                request.opened_from,
                detected_release.branch_name,
                handoff_path,
                execution_contract.recovery.recreate_packet_command
            )),
        },
        RenderedPacketDoc {
            relative_path: packet_owned_workflow_plan_path(&request.maintenance_root),
            contents: wrap_markdown(&format!(
                "# CI workflows plan\n\nThis packet is opened from `{}` and relayed through `{}`.\n\n## Ordered repo commands\n\n{}\n\n## Exact green gates\n\n{}\n",
                request.opened_from,
                handoff_path,
                markdown_command_list(&execution_contract.ordered_commands),
                markdown_command_list(&execution_contract.green_gates)
            )),
        },
        RenderedPacketDoc {
            relative_path: prompt_path,
            contents: packet_pr_prompt_template(entry, &request.maintenance_root),
        },
    ])
}
