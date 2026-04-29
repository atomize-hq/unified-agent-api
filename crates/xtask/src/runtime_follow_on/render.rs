use std::io::Write;

use super::{
    models::{RunStatus, RuntimeContext, ValidationReport},
    Error, HANDOFF_FILE_NAME, PROMPT_FILE_NAME, PROMPT_TEMPLATE, SKILL_PATH, WORKFLOW_VERSION,
    WRAPPER_COVERAGE_MANIFEST_PATH,
};
use crate::runtime_follow_on::io::now_rfc3339;

pub(super) fn render_prompt(context: &RuntimeContext) -> String {
    let descriptor = &context.approval.descriptor;
    PROMPT_TEMPLATE
        .replace("{{RUN_ID}}", &context.run_id)
        .replace("{{APPROVAL_PATH}}", &context.approval.relative_path)
        .replace("{{AGENT_ID}}", &descriptor.agent_id)
        .replace("{{DISPLAY_NAME}}", &descriptor.display_name)
        .replace("{{REQUESTED_TIER}}", &context.input_contract.requested_tier)
        .replace(
            "{{MINIMAL_JUSTIFICATION}}",
            context
                .input_contract
                .minimal_justification_text
                .as_deref()
                .unwrap_or("N/A"),
        )
        .replace(
            "{{ALLOW_RICH_SURFACES}}",
            &if context.input_contract.allow_rich_surface.is_empty() {
                "none".to_string()
            } else {
                context.input_contract.allow_rich_surface.join(", ")
            },
        )
        .replace("{{CRATE_PATH}}", &descriptor.crate_path)
        .replace("{{BACKEND_MODULE}}", &descriptor.backend_module)
        .replace("{{MANIFEST_ROOT}}", &descriptor.manifest_root)
        .replace(
            "{{WRAPPER_COVERAGE_SOURCE_PATH}}",
            &descriptor.wrapper_coverage_source_path,
        )
        .replace(
            "{{WRAPPER_COVERAGE_MANIFEST_PATH}}",
            &format!(
                "{}/{}",
                descriptor.wrapper_coverage_source_path, WRAPPER_COVERAGE_MANIFEST_PATH
            ),
        )
        .replace(
            "{{REQUIRED_TEST_PATH}}",
            &context.input_contract.required_agent_api_test,
        )
        .replace("{{RUN_DIR}}", &context.run_dir.to_string_lossy())
        .replace(
            "{{DOCS_TO_READ}}",
            &context.input_contract.docs_to_read.join("\n- "),
        )
        .replace(
            "{{ALLOWED_WRITE_PATHS}}",
            &context.input_contract.allowed_write_paths.join("\n- "),
        )
        .replace(
            "{{REQUIRED_COMMANDS}}",
            &context
                .input_contract
                .required_handoff_commands
                .join("\n- "),
        )
}

pub(super) fn render_dry_run_summary(context: &RuntimeContext) -> String {
    format!(
        "# Runtime Follow-On Dry Run\n\n- run_id: `{}`\n- approval: `{}`\n- agent_id: `{}`\n- requested_tier: `{}`\n- prompt: `{}`\n- handoff contract: `{}`\n",
        context.run_id,
        context.approval.relative_path,
        context.approval.descriptor.agent_id,
        context.input_contract.requested_tier,
        PROMPT_FILE_NAME,
        HANDOFF_FILE_NAME
    )
}

pub(super) fn render_run_summary(
    context: &RuntimeContext,
    report: &ValidationReport,
    written_paths: &[String],
) -> String {
    let mut text = format!(
        "# Runtime Follow-On Validation\n\n- run_id: `{}`\n- status: `{}`\n- agent_id: `{}`\n- requested_tier: `{}`\n",
        context.run_id, report.status, context.approval.descriptor.agent_id, context.input_contract.requested_tier
    );
    text.push_str("\n## Checks\n");
    for check in &report.checks {
        text.push_str(&format!(
            "- {}: {} ({})\n",
            check.name,
            if check.ok { "pass" } else { "fail" },
            check.message
        ));
    }
    text.push_str("\n## Written Paths\n");
    if written_paths.is_empty() {
        text.push_str("- none detected\n");
    } else {
        for path in written_paths {
            text.push_str(&format!("- `{path}`\n"));
        }
    }
    if !report.errors.is_empty() {
        text.push_str("\n## Errors\n");
        for error in &report.errors {
            text.push_str(&format!("- {error}\n"));
        }
    }
    text
}

pub(super) fn render_run_status(
    context: &RuntimeContext,
    mode: &str,
    status: &str,
    validation_passed: bool,
    handoff_ready: bool,
    written_paths: Vec<String>,
    errors: Vec<String>,
) -> RunStatus {
    RunStatus {
        workflow_version: WORKFLOW_VERSION.to_string(),
        generated_at: now_rfc3339().unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()),
        run_id: context.run_id.clone(),
        approval_artifact_path: context.approval.relative_path.clone(),
        agent_id: context.approval.descriptor.agent_id.clone(),
        requested_tier: context.input_contract.requested_tier.clone(),
        host_surface: "xtask runtime-follow-on".to_string(),
        loaded_skill_ref: SKILL_PATH.to_string(),
        mode: mode.to_string(),
        status: status.to_string(),
        validation_passed,
        handoff_ready,
        run_dir: context.run_dir.to_string_lossy().into_owned(),
        written_paths,
        errors,
    }
}

pub(super) fn write_header<W: Write>(
    writer: &mut W,
    context: &RuntimeContext,
    write_mode: bool,
) -> Result<(), Error> {
    writeln!(
        writer,
        "== RUNTIME-FOLLOW-ON {} ==",
        if write_mode { "WRITE" } else { "DRY RUN" }
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "approval: {}", context.approval.relative_path)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "agent_id: {}", context.approval.descriptor.agent_id)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "requested_tier: {}",
        context.input_contract.requested_tier
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}
