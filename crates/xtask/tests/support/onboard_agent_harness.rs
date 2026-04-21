#![allow(dead_code)]

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

const SEEDED_REGISTRY: &str = include_str!("../../data/agent_registry.toml");

#[derive(Debug)]
pub struct HarnessOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn base_args(agent_id: &str) -> Vec<String> {
    base_args_with_package_name(agent_id, "unified-agent-api-cursor")
}

pub fn base_args_with_package_name(agent_id: &str, package_name: &str) -> Vec<String> {
    base_args_with_mode(agent_id, package_name, "--dry-run", false)
}

pub fn write_args(agent_id: &str) -> Vec<String> {
    base_args_with_mode(agent_id, "unified-agent-api-cursor", "--write", false)
}

pub fn base_args_with_mode(
    agent_id: &str,
    package_name: &str,
    mode_flag: &str,
    include_other_mode: bool,
) -> Vec<String> {
    args_with_overrides(mode_flag, agent_id, package_name, &[], include_other_mode)
}

pub fn args_with_overrides(
    mode_flag: &str,
    agent_id: &str,
    package_name: &str,
    overrides: &[(&str, &str)],
    include_other_mode: bool,
) -> Vec<String> {
    let mut args = vec![
        "xtask".to_string(),
        "onboard-agent".to_string(),
        mode_flag.to_string(),
    ];
    if include_other_mode {
        args.push(if mode_flag == "--dry-run" {
            "--write".to_string()
        } else {
            "--dry-run".to_string()
        });
    }

    args.extend([
        "--agent-id".to_string(),
        agent_id.to_string(),
        "--display-name".to_string(),
        "Cursor CLI".to_string(),
        "--crate-path".to_string(),
        "crates/cursor".to_string(),
        "--backend-module".to_string(),
        "crates/agent_api/src/backends/cursor".to_string(),
        "--manifest-root".to_string(),
        "cli_manifests/cursor".to_string(),
        "--package-name".to_string(),
        package_name.to_string(),
        "--canonical-target".to_string(),
        "linux-x64".to_string(),
        "--wrapper-coverage-binding-kind".to_string(),
        "generated_from_wrapper_crate".to_string(),
        "--wrapper-coverage-source-path".to_string(),
        "crates/cursor".to_string(),
        "--always-on-capability".to_string(),
        "agent_api.run".to_string(),
        "--target-gated-capability".to_string(),
        "agent_api.tools.mcp.list.v1:linux-x64".to_string(),
        "--config-gated-capability".to_string(),
        "agent_api.exec.external_sandbox.v1:allow_external_sandbox_exec".to_string(),
        "--support-matrix-enabled".to_string(),
        "true".to_string(),
        "--capability-matrix-enabled".to_string(),
        "true".to_string(),
        "--docs-release-track".to_string(),
        "crates-io".to_string(),
        "--onboarding-pack-prefix".to_string(),
        "cursor-cli-onboarding".to_string(),
    ]);

    for (flag, value) in overrides {
        let position = args
            .iter()
            .position(|existing| existing == flag)
            .expect("override flag must exist");
        args[position + 1] = (*value).to_string();
    }

    args
}

pub fn approval_args(mode_flag: &str, approval_path: &str) -> Vec<String> {
    vec![
        "xtask".to_string(),
        "onboard-agent".to_string(),
        mode_flag.to_string(),
        "--approval".to_string(),
        approval_path.to_string(),
    ]
}

pub fn gemini_dry_run_args() -> Vec<String> {
    vec![
        "xtask".to_string(),
        "onboard-agent".to_string(),
        "--dry-run".to_string(),
        "--agent-id".to_string(),
        "gemini_cli".to_string(),
        "--display-name".to_string(),
        "Gemini CLI".to_string(),
        "--crate-path".to_string(),
        "crates/gemini_cli".to_string(),
        "--backend-module".to_string(),
        "crates/agent_api/src/backends/gemini_cli".to_string(),
        "--manifest-root".to_string(),
        "cli_manifests/gemini_cli".to_string(),
        "--package-name".to_string(),
        "unified-agent-api-gemini-cli".to_string(),
        "--canonical-target".to_string(),
        "darwin-arm64".to_string(),
        "--wrapper-coverage-binding-kind".to_string(),
        "generated_from_wrapper_crate".to_string(),
        "--wrapper-coverage-source-path".to_string(),
        "crates/gemini_cli".to_string(),
        "--always-on-capability".to_string(),
        "agent_api.config.model.v1".to_string(),
        "--always-on-capability".to_string(),
        "agent_api.events".to_string(),
        "--always-on-capability".to_string(),
        "agent_api.events.live".to_string(),
        "--always-on-capability".to_string(),
        "agent_api.run".to_string(),
        "--support-matrix-enabled".to_string(),
        "true".to_string(),
        "--capability-matrix-enabled".to_string(),
        "true".to_string(),
        "--docs-release-track".to_string(),
        "crates-io".to_string(),
        "--onboarding-pack-prefix".to_string(),
        "gemini-cli-onboarding".to_string(),
    ]
}

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

pub fn fixture_root(prefix: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time after unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(&root).expect("create temp fixture");
    write_text(
        &root.join("Cargo.toml"),
        "[workspace]\nmembers = [\n  \"crates/agent_api\",\n  \"crates/codex\",\n  \"crates/claude_code\",\n  \"crates/opencode\",\n  \"crates/wrapper_events\",\n  \"crates/xtask\",\n]\n",
    );
    write_text(
        &root.join("crates/xtask/data/agent_registry.toml"),
        SEEDED_REGISTRY,
    );
    write_text(
        &root.join("crates/agent_api/Cargo.toml"),
        "[package]\nname = \"unified-agent-api\"\nversion = \"0.2.3\"\nedition = \"2021\"\n",
    );
    write_text(
        &root.join("crates/codex/Cargo.toml"),
        "[package]\nname = \"unified-agent-api-codex\"\nversion = \"0.2.3\"\nedition = \"2021\"\n",
    );
    write_text(
        &root.join("crates/claude_code/Cargo.toml"),
        "[package]\nname = \"unified-agent-api-claude-code\"\nversion = \"0.2.3\"\nedition = \"2021\"\n",
    );
    write_text(
        &root.join("crates/opencode/Cargo.toml"),
        "[package]\nname = \"unified-agent-api-opencode\"\nversion = \"0.2.3\"\nedition = \"2021\"\n",
    );
    write_text(
        &root.join("crates/wrapper_events/Cargo.toml"),
        "[package]\nname = \"unified-agent-api-wrapper-events\"\nversion = \"0.2.3\"\nedition = \"2021\"\n",
    );
    write_text(
        &root.join("crates/xtask/Cargo.toml"),
        "[package]\nname = \"xtask\"\nversion = \"0.2.3\"\nedition = \"2021\"\npublish = false\n",
    );
    root
}

pub fn write_text(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

pub fn seed_release_touchpoints(root: &Path) {
    write_text(
        &root.join("docs/crates-io-release.md"),
        "# Release docs\n\nManual contract text.\n",
    );
    write_text(
        &root.join(".github/workflows/publish-crates.yml"),
        "name: publish-crates\n",
    );
    write_text(
        &root.join("scripts/publish_crates.py"),
        "print('publish')\n",
    );
    write_text(
        &root.join("scripts/validate_publish_versions.py"),
        "print('validate')\n",
    );
    write_text(
        &root.join("scripts/check_publish_readiness.py"),
        "print('readiness')\n",
    );
}

pub fn seed_approval_artifact(
    root: &Path,
    relative_path: &str,
    recommended_agent_id: &str,
    approved_agent_id: &str,
    override_reason: Option<&str>,
) -> String {
    seed_approval_artifact_with_pack_prefix(
        root,
        relative_path,
        recommended_agent_id,
        approved_agent_id,
        override_reason,
        "cursor-cli-onboarding",
    )
}

pub fn seed_approval_artifact_with_pack_prefix(
    root: &Path,
    relative_path: &str,
    recommended_agent_id: &str,
    approved_agent_id: &str,
    override_reason: Option<&str>,
    onboarding_pack_prefix: &str,
) -> String {
    let mut contents = format!(
        concat!(
            "artifact_version = \"1\"\n",
            "comparison_ref = \"compare/cursor\"\n",
            "selection_mode = \"factory_validation\"\n",
            "recommended_agent_id = \"{recommended_agent_id}\"\n",
            "approved_agent_id = \"{approved_agent_id}\"\n",
            "approval_commit = \"test-approval-commit\"\n",
            "approval_recorded_at = \"2026-04-21T11:23:09Z\"\n",
        ),
        recommended_agent_id = recommended_agent_id,
        approved_agent_id = approved_agent_id,
    );
    if let Some(override_reason) = override_reason {
        contents.push_str(&format!("override_reason = \"{override_reason}\"\n"));
    }
    contents.push_str(&format!(
        concat!(
            "\n",
            "[descriptor]\n",
            "agent_id = \"cursor\"\n",
            "display_name = \"Cursor CLI\"\n",
            "crate_path = \"crates/cursor\"\n",
            "backend_module = \"crates/agent_api/src/backends/cursor\"\n",
            "manifest_root = \"cli_manifests/cursor\"\n",
            "package_name = \"unified-agent-api-cursor\"\n",
            "canonical_targets = [\"linux-x64\"]\n",
            "wrapper_coverage_binding_kind = \"generated_from_wrapper_crate\"\n",
            "wrapper_coverage_source_path = \"crates/cursor\"\n",
            "always_on_capabilities = [\"agent_api.run\"]\n",
            "backend_extensions = []\n",
            "support_matrix_enabled = true\n",
            "capability_matrix_enabled = true\n",
            "docs_release_track = \"crates-io\"\n",
            "onboarding_pack_prefix = \"{onboarding_pack_prefix}\"\n",
            "\n",
            "[[descriptor.target_gated_capabilities]]\n",
            "capability_id = \"agent_api.tools.mcp.list.v1\"\n",
            "targets = [\"linux-x64\"]\n",
            "\n",
            "[[descriptor.config_gated_capabilities]]\n",
            "capability_id = \"agent_api.exec.external_sandbox.v1\"\n",
            "config_key = \"allow_external_sandbox_exec\"\n",
        ),
        onboarding_pack_prefix = onboarding_pack_prefix,
    ));
    write_text(&root.join(relative_path), &contents);
    relative_path.to_string()
}

pub fn sha256_hex(path: &Path) -> String {
    let bytes = fs::read(path).expect("read approval artifact");
    hex::encode(Sha256::digest(bytes))
}

pub fn assert_sections_in_order(stdout: &str, sections: &[&str]) {
    let mut cursor = 0usize;
    for section in sections {
        let found = stdout[cursor..]
            .find(section)
            .map(|offset| cursor + offset)
            .unwrap_or_else(|| panic!("missing section `{section}` in stdout:\n{stdout}"));
        cursor = found + section.len();
    }
}

pub fn snapshot_files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    snapshot_files_recursive(root, root, &mut out);
    out
}

fn snapshot_files_recursive(root: &Path, current: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
    let entries = fs::read_dir(current).expect("read dir");
    for entry in entries {
        let entry = entry.expect("read dir entry");
        let path = entry.path();
        let file_type = entry.file_type().expect("read file type");
        if file_type.is_dir() {
            snapshot_files_recursive(root, &path, out);
        } else if file_type.is_file() {
            let rel = path
                .strip_prefix(root)
                .expect("path relative to root")
                .display()
                .to_string();
            out.insert(rel, fs::read(&path).expect("read file"));
        }
    }
}
