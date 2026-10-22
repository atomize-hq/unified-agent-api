use super::*;

pub(in super::super) fn render_request_toml(
    args: &Args,
    basis_ref: &str,
    detected_release: &DetectedRelease,
    support_surface_audit: &SupportSurfaceAudit,
    execution_contract: &ExecutionContract,
    depth: Option<&request::SupportDepthBinding>,
) -> Result<String, Error> {
    let mut out = String::new();
    push_toml_line(&mut out, "artifact_version", AUTOMATED_ARTIFACT_VERSION);
    push_toml_line(&mut out, "agent_id", &args.agent);
    push_toml_line(&mut out, "trigger_kind", "upstream_release_detected");
    push_toml_line(&mut out, "basis_ref", basis_ref);
    push_toml_line(
        &mut out,
        "opened_from",
        &args.opened_from.display().to_string(),
    );
    push_toml_array(
        &mut out,
        "requested_control_plane_actions",
        &["packet_doc_refresh".to_string()],
    );
    push_toml_line(&mut out, "request_recorded_at", &args.request_recorded_at);
    push_toml_line(&mut out, "request_commit", &args.request_commit);
    out.push('\n');

    out.push_str("[runtime_followup_required]\n");
    out.push_str("required = false\n");
    out.push_str("items = []\n\n");

    out.push_str("[detected_release]\n");
    push_toml_line(&mut out, "detected_by", &detected_release.detected_by);
    push_toml_line(
        &mut out,
        "current_validated",
        &detected_release.current_validated,
    );
    push_toml_line(&mut out, "target_version", &detected_release.target_version);
    push_toml_line(&mut out, "latest_stable", &detected_release.latest_stable);
    push_toml_line(&mut out, "version_policy", &detected_release.version_policy);
    push_toml_line(&mut out, "source_kind", &detected_release.source_kind);
    push_toml_line(&mut out, "source_ref", &detected_release.source_ref);
    push_toml_line(&mut out, "dispatch_kind", &detected_release.dispatch_kind);
    push_toml_line(
        &mut out,
        "dispatch_workflow",
        &detected_release.dispatch_workflow,
    );
    push_toml_line(&mut out, "branch_name", &detected_release.branch_name);
    out.push('\n');

    if let Some(depth) = depth {
        out.push_str(&depth.render().map_err(Error::Validation)?);
    }
    render_support_surface_audit(&mut out, support_surface_audit);
    out.push('\n');

    out.push_str("[execution_contract]\n");
    push_toml_line(&mut out, "executor", &execution_contract.executor);
    push_toml_line(
        &mut out,
        "prompt_template_path",
        &execution_contract.prompt_template_path,
    );
    push_toml_line(&mut out, "prompt_sha256", &execution_contract.prompt_sha256);
    push_toml_line(
        &mut out,
        "pr_summary_path",
        &execution_contract.pr_summary_path,
    );
    push_toml_line(&mut out, "closeout_path", &execution_contract.closeout_path);
    out.push_str("requires_manual_closeout = true\n");
    push_toml_array(
        &mut out,
        "writable_surfaces",
        &execution_contract.writable_surfaces,
    );
    push_toml_array(
        &mut out,
        "read_only_inputs",
        &execution_contract.read_only_inputs,
    );
    push_toml_array(
        &mut out,
        "ordered_commands",
        &execution_contract.ordered_commands,
    );
    push_toml_array(&mut out, "green_gates", &execution_contract.green_gates);
    out.push('\n');

    out.push_str("[execution_contract.recovery]\n");
    push_toml_line(
        &mut out,
        "recreate_packet_command",
        &execution_contract.recovery.recreate_packet_command,
    );
    push_toml_line(
        &mut out,
        "reopen_pr_body_path",
        &execution_contract.recovery.reopen_pr_body_path,
    );
    push_toml_line(
        &mut out,
        "reopen_pr_branch",
        &execution_contract.recovery.reopen_pr_branch,
    );
    push_toml_array(&mut out, "notes", &execution_contract.recovery.notes);
    Ok(out)
}
