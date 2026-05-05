use std::{fs, path::Path};

use crate::agent_registry::{AgentRegistry, AgentRegistryEntry};

use super::request::{DetectedRelease, MaintenanceRequest};

const OWNERSHIP_MARKER: &str = "<!-- generated-by: xtask refresh-agent; owner: control-plane -->";
const PR_SUMMARY_FILE_NAME: &str = "governance/pr-summary.md";
const CLOSEOUT_FILE_NAME: &str = "governance/maintenance-closeout.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedPacketDoc {
    pub relative_path: String,
    pub contents: String,
}

pub fn build_packet_docs(
    workspace_root: &Path,
    request: &MaintenanceRequest,
) -> Result<Vec<RenderedPacketDoc>, String> {
    if request.is_automated_watch_request() {
        build_automated_packet_docs(workspace_root, request)
    } else {
        Ok(build_manual_packet_docs(request))
    }
}

fn build_manual_packet_docs(request: &MaintenanceRequest) -> Vec<RenderedPacketDoc> {
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

fn build_automated_packet_docs(
    workspace_root: &Path,
    request: &MaintenanceRequest,
) -> Result<Vec<RenderedPacketDoc>, String> {
    let entry = load_registry_entry(workspace_root, &request.agent_id)?;
    let detected_release = request.detected_release.as_ref().ok_or_else(|| {
        "automated maintenance request missing detected_release metadata".to_string()
    })?;
    let rendered_template = render_pr_body_template(workspace_root, &entry, detected_release)?;
    let writable_surfaces = automated_writable_surfaces(request, &entry, detected_release);
    let read_only_inputs = automated_read_only_inputs(request, &entry);
    let excluded_surfaces = automated_excluded_surfaces(&entry);
    let ordered_commands = automated_ordered_commands(request, &entry, detected_release);
    let green_gates = automated_green_gates(&entry);
    let trigger_context = render_trigger_context(request);
    let closeout_path = format!("{}/{}", request.maintenance_root, CLOSEOUT_FILE_NAME);
    let root = &request.maintenance_root;

    Ok(vec![
        RenderedPacketDoc {
            relative_path: format!("{root}/README.md"),
            contents: wrap_markdown(&format!(
                "# {} maintenance\n\nThis packet tracks automated upstream-release maintenance for `{}`.\n\n## Request\n\n- request artifact: `{}`\n- trigger kind: `{}`\n- basis ref: `{}`\n- opened from: `{}`\n- recorded at: `{}`\n- request commit: `{}`\n\n## Trigger context\n\n{}\n\n## Canonical execution contract\n\nUse `{}/HANDOFF.md` as the exact contributor execution contract for this lane. The PR body summary under `{}/{}` is derivative only.\n",
                request.agent_id,
                request.agent_id,
                request.relative_path,
                request.trigger_kind.as_str(),
                request.basis_ref,
                request.opened_from,
                request.request_recorded_at,
                request.request_commit,
                trigger_context,
                request.maintenance_root,
                request.maintenance_root,
                PR_SUMMARY_FILE_NAME
            )),
        },
        RenderedPacketDoc {
            relative_path: format!("{root}/scope_brief.md"),
            contents: wrap_markdown(&format!(
                "# Scope brief\n\nThis automated maintenance lane is limited to the contributor-ready packet and the worker-owned parity surfaces for `{}`.\n\n## Writable surfaces\n\n{}\n\n## Explicit exclusions\n\n{}\n",
                request.agent_id,
                markdown_list(&writable_surfaces),
                markdown_list(&excluded_surfaces)
            )),
        },
        RenderedPacketDoc {
            relative_path: format!("{root}/seam_map.md"),
            contents: wrap_markdown(
                "# Seam map\n\nThis maintenance packet has one bounded seam: render a contributor-ready execution contract for the detected upstream release while keeping `HANDOFF.md` canonical and `governance/pr-summary.md` derivative.\n",
            ),
        },
        RenderedPacketDoc {
            relative_path: format!("{root}/threading.md"),
            contents: wrap_markdown(&format!(
                "# Threading\n\n1. Review the auto-generated request at `{}` and the canonical contract at `{}/HANDOFF.md`.\n2. Apply the exact coding-agent prompt from `HANDOFF.md` against branch `{}`.\n3. Author `{}` and run the exact closeout command from `HANDOFF.md` after the green gates pass.\n",
                request.relative_path,
                request.maintenance_root,
                detected_release.branch_name,
                closeout_path
            )),
        },
        RenderedPacketDoc {
            relative_path: format!("{root}/review_surfaces.md"),
            contents: wrap_markdown(&format!(
                "# Review surfaces\n\n## Writable surfaces\n\n{}\n\n## Read-only inputs\n\n{}\n\n## Explicit exclusions\n\n{}\n",
                markdown_list(&writable_surfaces),
                markdown_list(&read_only_inputs),
                markdown_list(&excluded_surfaces)
            )),
        },
        RenderedPacketDoc {
            relative_path: format!("{root}/HANDOFF.md"),
            contents: wrap_markdown(&format!(
                "# Handoff\n\nThis file is the canonical contributor execution contract for `{}` maintenance.\n\n## Packet origin\n\n{}\n\n## Writable surfaces\n\n{}\n\n## Read-only inputs\n\n{}\n\n## Explicit exclusions\n\n{}\n\n## Ordered repo commands\n\n{}\n\n## Exact green gates\n\n{}\n\n## Exact closeout command\n\n```sh\ncargo run -p xtask -- close-agent-maintenance --request {} --closeout {}\n```\n\n## Exact coding-agent prompt\n\n```md\n{}\n```\n",
                request.agent_id,
                trigger_context,
                markdown_list(&writable_surfaces),
                markdown_list(&read_only_inputs),
                markdown_list(&excluded_surfaces),
                markdown_list(&ordered_commands),
                markdown_list(&green_gates),
                request.relative_path,
                closeout_path,
                rendered_template
            )),
        },
        RenderedPacketDoc {
            relative_path: format!("{root}/{PR_SUMMARY_FILE_NAME}"),
            contents: wrap_markdown(&format!(
                "# PR summary\n\nAutomated maintenance packet for `{}` target `{}`.\n\n- canonical execution contract: `{}/HANDOFF.md`\n- request artifact: `{}`\n- branch: `{}`\n- opened from: `{}`\n\n## Next step\n\nFollow `{}/HANDOFF.md` exactly. This PR summary is derivative from the same packet renderer context.\n\n{}\n",
                request.agent_id,
                detected_release.target_version,
                request.maintenance_root,
                request.relative_path,
                detected_release.branch_name,
                request.opened_from,
                request.maintenance_root,
                rendered_template
            )),
        },
        RenderedPacketDoc {
            relative_path: format!("{root}/governance/remediation-log.md"),
            contents: wrap_markdown(&format!(
                "# Remediation log\n\nRefresh planned from `{}`.\n\n- basis ref: `{}`\n- trigger kind: `{}`\n- request sha256: `{}`\n- canonical handoff: `{}/HANDOFF.md`\n- derivative pr summary: `{}/{}`\n\nNo maintenance closeout has been recorded yet.\n",
                request.relative_path,
                request.basis_ref,
                request.trigger_kind.as_str(),
                request.sha256,
                request.maintenance_root,
                request.maintenance_root,
                PR_SUMMARY_FILE_NAME
            )),
        },
    ])
}

fn load_registry_entry(
    workspace_root: &Path,
    agent_id: &str,
) -> Result<AgentRegistryEntry, String> {
    let registry =
        AgentRegistry::load(workspace_root).map_err(|err| format!("load agent registry: {err}"))?;
    registry
        .find(agent_id)
        .cloned()
        .ok_or_else(|| format!("unknown agent_id `{agent_id}` in maintenance packet rendering"))
}

fn render_pr_body_template(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    detected_release: &DetectedRelease,
) -> Result<String, String> {
    let template_path = workspace_root
        .join(&entry.manifest_root)
        .join("PR_BODY_TEMPLATE.md");
    let template = fs::read_to_string(&template_path)
        .map_err(|err| format!("read {}: {err}", template_path.display()))?;
    Ok(template.replace("{{VERSION}}", &detected_release.target_version))
}

fn automated_writable_surfaces(
    request: &MaintenanceRequest,
    entry: &AgentRegistryEntry,
    detected_release: &DetectedRelease,
) -> Vec<String> {
    vec![
        format!(
            "maintenance packet docs under `{}`",
            request.maintenance_root
        ),
        format!("wrapper crate sources under `{}/**`", entry.crate_path),
        format!("backend module sources under `{}/**`", entry.backend_module),
        format!(
            "artifact pins `{}/artifacts.lock.json`",
            entry.manifest_root
        ),
        format!(
            "versioned snapshots under `{}/snapshots/{}/**`",
            entry.manifest_root, detected_release.target_version
        ),
        format!(
            "versioned reports under `{}/reports/{}/**`",
            entry.manifest_root, detected_release.target_version
        ),
        format!(
            "version metadata `{}/versions/{}.json`",
            entry.manifest_root, detected_release.target_version
        ),
        format!(
            "generated wrapper coverage `{}/wrapper_coverage.json`",
            entry.manifest_root
        ),
    ]
}

fn automated_read_only_inputs(
    request: &MaintenanceRequest,
    entry: &AgentRegistryEntry,
) -> Vec<String> {
    vec![
        format!("maintenance request `{}`", request.relative_path),
        format!("baseline pointer `{}`", request.basis_ref),
        format!(
            "agent-local prompt/tail template `{}/PR_BODY_TEMPLATE.md`",
            entry.manifest_root
        ),
        format!(
            "agent-local playbook `{}/OPS_PLAYBOOK.md`",
            entry.manifest_root
        ),
        format!(
            "agent-local workflow plan `{}/CI_WORKFLOWS_PLAN.md`",
            entry.manifest_root
        ),
        format!("worker workflow `{}`", request.opened_from),
    ]
}

fn automated_excluded_surfaces(entry: &AgentRegistryEntry) -> Vec<String> {
    vec![
        format!(
            "promotion pointer `{}/latest_validated.txt`",
            entry.manifest_root
        ),
        format!(
            "promotion policy `{}/min_supported.txt`",
            entry.manifest_root
        ),
        "support publication `cli_manifests/support_matrix/current.json`".to_string(),
        "support publication markdown `docs/specs/unified-agent-api/support-matrix.md`"
            .to_string(),
        "capability publication `docs/specs/unified-agent-api/capability-matrix.md`"
            .to_string(),
        "release documentation `docs/crates-io-release.md`".to_string(),
        "note: those support/capability/release-doc surfaces remain part of the broader maintenance framework, but this automated upstream-release lane is packet-only and does not request them".to_string(),
    ]
}

fn automated_ordered_commands(
    request: &MaintenanceRequest,
    entry: &AgentRegistryEntry,
    detected_release: &DetectedRelease,
) -> Vec<String> {
    let closeout_path = format!("{}/{}", request.maintenance_root, CLOSEOUT_FILE_NAME);
    vec![
        format!(
            "run the agent-specific implementation steps exactly as rendered in the prompt block for target `{}`",
            detected_release.target_version
        ),
        format!(
            "cargo run -p xtask -- codex-validate --root {}",
            entry.manifest_root
        ),
        "cargo run -p xtask -- support-matrix --check".to_string(),
        "cargo run -p xtask -- capability-matrix --check".to_string(),
        "cargo run -p xtask -- capability-matrix-audit".to_string(),
        "make preflight".to_string(),
        format!(
            "cargo run -p xtask -- close-agent-maintenance --request {} --closeout {}",
            request.relative_path, closeout_path
        ),
    ]
}

fn automated_green_gates(entry: &AgentRegistryEntry) -> Vec<String> {
    vec![
        format!(
            "cargo run -p xtask -- codex-validate --root {}",
            entry.manifest_root
        ),
        "cargo run -p xtask -- support-matrix --check".to_string(),
        "cargo run -p xtask -- capability-matrix --check".to_string(),
        "cargo run -p xtask -- capability-matrix-audit".to_string(),
        "make preflight".to_string(),
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
