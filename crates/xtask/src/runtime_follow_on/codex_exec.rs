use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::approval_artifact::{ApprovalArtifactError, ApprovalDescriptor};

use super::{
    io::{now_rfc3339, write_string},
    models::{CodexExecutionEvidence, InputContract, RuntimeContext},
    Args, Error, CODEX_BINARY_ENV, CODEX_STDERR_FILE_NAME, CODEX_STDOUT_FILE_NAME,
    PROMPT_FILE_NAME, RUNTIME_RUNS_ROOT, WORKFLOW_VERSION,
};

pub(super) fn required_agent_api_test(agent_id: &str) -> String {
    format!("crates/agent_api/tests/c1_{agent_id}_runtime_follow_on.rs")
}

pub(super) fn docs_to_read(agent_id: &str) -> Vec<String> {
    vec![
        "docs/cli-agent-onboarding-factory-operator-guide.md".to_string(),
        "docs/specs/cli-agent-onboarding-charter.md".to_string(),
        "docs/adr/0013-agent-api-backend-harness.md".to_string(),
        format!(
            "docs/agents/lifecycle/{}-onboarding/HANDOFF.md",
            agent_id.replace('_', "-")
        ),
        "crates/agent_api/src/backends/opencode".to_string(),
        "crates/opencode".to_string(),
    ]
}

pub(super) fn allowed_write_paths(descriptor: &ApprovalDescriptor) -> Vec<String> {
    vec![
        "Cargo.lock".to_string(),
        format!("{}/**", descriptor.crate_path),
        format!("{}/**", descriptor.backend_module),
        "crates/agent_api/Cargo.toml".to_string(),
        "crates/agent_api/src/backends/mod.rs".to_string(),
        "crates/agent_api/src/lib.rs".to_string(),
        required_agent_api_test(&descriptor.agent_id),
        format!(
            "crates/agent_api/tests/c1_{}_runtime_follow_on/**",
            descriptor.agent_id
        ),
        format!("crates/agent_api/src/bin/fake_{}*", descriptor.agent_id),
        format!("crates/agent_api/src/bin/fake_{}*/**", descriptor.agent_id),
        format!("{}/src/**", descriptor.wrapper_coverage_source_path),
        format!("{}/snapshots/**", descriptor.manifest_root),
        format!("{}/supplement/**", descriptor.manifest_root),
        format!("{}/**", RUNTIME_RUNS_ROOT),
    ]
}

pub(super) fn execute_codex_write(
    workspace_root: &Path,
    context: &RuntimeContext,
    args: &Args,
    prompt: &str,
) -> Result<CodexExecutionEvidence, Error> {
    let binary = resolve_codex_binary(args);
    let argv = vec![
        "exec".to_string(),
        "--skip-git-repo-check".to_string(),
        "--dangerously-bypass-approvals-and-sandbox".to_string(),
        "--cd".to_string(),
        workspace_root.to_string_lossy().into_owned(),
    ];
    let mut child = Command::new(&binary)
        .current_dir(workspace_root)
        .args(&argv)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| Error::Internal(format!("spawn codex binary `{binary}`: {err}")))?;
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| Error::Internal("codex exec stdin was not captured".to_string()))?;
        stdin
            .write_all(prompt.as_bytes())
            .map_err(|err| Error::Internal(format!("write codex prompt to stdin: {err}")))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|err| Error::Internal(format!("wait for codex exec: {err}")))?;
    let stdout_path = context.run_dir.join(CODEX_STDOUT_FILE_NAME);
    let stderr_path = context.run_dir.join(CODEX_STDERR_FILE_NAME);
    write_string(&stdout_path, &String::from_utf8_lossy(&output.stdout))?;
    write_string(&stderr_path, &String::from_utf8_lossy(&output.stderr))?;

    Ok(CodexExecutionEvidence {
        workflow_version: WORKFLOW_VERSION.to_string(),
        generated_at: now_rfc3339()?,
        run_id: context.run_id.clone(),
        binary,
        argv,
        prompt_path: context
            .run_dir
            .join(PROMPT_FILE_NAME)
            .to_string_lossy()
            .into_owned(),
        stdout_path: stdout_path.to_string_lossy().into_owned(),
        stderr_path: stderr_path.to_string_lossy().into_owned(),
        exit_code: output.status.code().unwrap_or(1),
    })
}

pub(super) fn approval_descriptor_view(input_contract: &InputContract) -> ApprovalDescriptor {
    ApprovalDescriptor {
        agent_id: input_contract.agent_id.clone(),
        display_name: input_contract.display_name.clone(),
        crate_path: input_contract.crate_path.clone(),
        backend_module: input_contract.backend_module.clone(),
        manifest_root: input_contract.manifest_root.clone(),
        package_name: String::new(),
        canonical_targets: input_contract.canonical_targets.clone(),
        wrapper_coverage_binding_kind: String::new(),
        wrapper_coverage_source_path: input_contract.wrapper_coverage_source_path.clone(),
        always_on_capabilities: input_contract.always_on_capabilities.clone(),
        target_gated_capabilities: input_contract.target_gated_capabilities.clone(),
        config_gated_capabilities: input_contract.config_gated_capabilities.clone(),
        backend_extensions: input_contract.backend_extensions.clone(),
        support_matrix_enabled: input_contract.support_matrix_enabled,
        capability_matrix_enabled: input_contract.capability_matrix_enabled,
        capability_matrix_target: input_contract.capability_matrix_target.clone(),
        docs_release_track: String::new(),
        onboarding_pack_prefix: String::new(),
    }
}

pub(super) fn is_allowed_write_path(path: &str, descriptor: &ApprovalDescriptor) -> bool {
    let required_test = required_agent_api_test(&descriptor.agent_id);
    let runtime_test_dir = format!(
        "crates/agent_api/tests/c1_{}_runtime_follow_on/",
        descriptor.agent_id
    );
    let fake_bin_prefix = format!("crates/agent_api/src/bin/fake_{}", descriptor.agent_id);

    path == "crates/agent_api/Cargo.toml"
        || path == "Cargo.lock"
        || path == "crates/agent_api/src/backends/mod.rs"
        || path == "crates/agent_api/src/lib.rs"
        || path == required_test
        || path.starts_with(&(descriptor.crate_path.clone() + "/"))
        || path.starts_with(&(descriptor.backend_module.clone() + "/"))
        || path.starts_with(&runtime_test_dir)
        || path == fake_bin_prefix
        || path.starts_with(&fake_bin_prefix)
        || path.starts_with(&(descriptor.wrapper_coverage_source_path.clone() + "/src/"))
        || path.starts_with(&(descriptor.manifest_root.clone() + "/snapshots/"))
        || path.starts_with(&(descriptor.manifest_root.clone() + "/supplement/"))
}

pub(super) fn is_publication_owned_manifest_path(path: &str, manifest_root: &str) -> bool {
    path.starts_with(&(manifest_root.to_string() + "/"))
        && !path.starts_with(&(manifest_root.to_string() + "/snapshots/"))
        && !path.starts_with(&(manifest_root.to_string() + "/supplement/"))
}

pub(super) fn map_approval_error(err: ApprovalArtifactError) -> Error {
    match err {
        ApprovalArtifactError::Validation(message) => Error::Validation(message),
        ApprovalArtifactError::Internal(message) => Error::Internal(message),
    }
}

pub(super) fn resolve_workspace_root() -> Result<PathBuf, Error> {
    let current_dir = std::env::current_dir()
        .map_err(|err| Error::Internal(format!("resolve current directory: {err}")))?;
    for candidate in current_dir.ancestors() {
        let cargo_toml = candidate.join("Cargo.toml");
        let Ok(text) = fs::read_to_string(&cargo_toml) else {
            continue;
        };
        if text.contains("[workspace]") {
            return Ok(candidate.to_path_buf());
        }
    }
    Err(Error::Internal(format!(
        "could not resolve workspace root from {}",
        current_dir.display()
    )))
}

fn resolve_codex_binary(args: &Args) -> String {
    args.codex_binary
        .clone()
        .or_else(|| std::env::var(CODEX_BINARY_ENV).ok())
        .unwrap_or_else(|| "codex".to_string())
}
