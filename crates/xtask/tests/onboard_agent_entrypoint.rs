use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use clap::{CommandFactory, Parser, Subcommand};
use xtask::onboard_agent;

const SEEDED_REGISTRY: &str = include_str!("../data/agent_registry.toml");

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Project automation tasks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Preview the next control-plane onboarding packet without writing files.
    OnboardAgent(onboard_agent::Args),
}

#[derive(Debug)]
struct HarnessOutput {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

#[test]
fn onboard_agent_help_text_includes_required_surface() {
    let top_help = Cli::command().render_help().to_string();
    assert!(top_help.contains("onboard-agent"));

    let err = Cli::try_parse_from(["xtask", "onboard-agent", "--help"])
        .expect_err("subcommand help should short-circuit parsing");
    assert_eq!(err.exit_code(), 0);
    let help_text = err.to_string();
    assert!(help_text.contains("--dry-run"));
    assert!(help_text.contains("--write"));
    assert!(help_text.contains("--agent-id"));
    assert!(help_text.contains("--canonical-target"));
    assert!(help_text.contains("--always-on-capability"));
    assert!(help_text.contains("--target-gated-capability"));
    assert!(help_text.contains("--config-gated-capability"));
    assert!(help_text.contains("--backend-extension"));
    assert!(help_text.contains("--support-matrix-enabled"));
    assert!(help_text.contains("--capability-matrix-enabled"));
    assert!(help_text.contains("--docs-release-track"));
    assert!(help_text.contains("--onboarding-pack-prefix"));
}

#[test]
fn onboard_agent_requires_one_mode_flag() {
    let output = run_cli(
        [
            "xtask",
            "onboard-agent",
            "--agent-id",
            "cursor",
            "--display-name",
            "Cursor CLI",
            "--crate-path",
            "crates/cursor",
            "--backend-module",
            "crates/agent_api/src/backends/cursor",
            "--manifest-root",
            "cli_manifests/cursor",
            "--package-name",
            "unified-agent-api-cursor",
            "--canonical-target",
            "linux-x64",
            "--wrapper-coverage-binding-kind",
            "generated_from_wrapper_crate",
            "--wrapper-coverage-source-path",
            "crates/cursor",
            "--always-on-capability",
            "agent_api.run",
            "--support-matrix-enabled",
            "true",
            "--capability-matrix-enabled",
            "true",
            "--docs-release-track",
            "crates-io",
            "--onboarding-pack-prefix",
            "cursor-cli-onboarding",
        ],
        &fixture_root("onboard-agent-missing-dry-run"),
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("--dry-run"));
    assert!(output.stderr.contains("--write"));
}

#[test]
fn onboard_agent_rejects_conflicting_mode_flags() {
    let output = run_cli(
        base_args_with_mode("cursor", "unified-agent-api-cursor", "--dry-run", true),
        &fixture_root("onboard-agent-conflicting-modes"),
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("--dry-run"));
    assert!(output.stderr.contains("--write"));
}

#[test]
fn onboard_agent_duplicate_agent_id_exits_with_validation_code() {
    let fixture = fixture_root("onboard-agent-duplicate-id");
    let output = run_cli(base_args("codex"), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(
        output
            .stderr
            .contains("agent_id `codex` already exists in crates/xtask/data/agent_registry.toml"),
        "stderr did not mention duplicate agent id:\n{}",
        output.stderr
    );
}

#[test]
fn onboard_agent_duplicate_registry_package_name_exits_with_validation_code() {
    let fixture = fixture_root("onboard-agent-duplicate-package-name");
    let output = run_cli(
        base_args_with_package_name("cursor", "unified-agent-api-codex"),
        &fixture,
    );

    assert_eq!(output.exit_code, 2);
    assert!(
        output
            .stderr
            .contains("package_name `unified-agent-api-codex` is already owned by agent `codex`"),
        "stderr did not mention duplicate package name:\n{}",
        output.stderr
    );
}

#[test]
fn onboard_agent_duplicate_workspace_package_name_exits_with_validation_code() {
    let fixture = fixture_root("onboard-agent-workspace-package-name");
    let output = run_cli(
        base_args_with_package_name("cursor", "unified-agent-api"),
        &fixture,
    );

    assert_eq!(output.exit_code, 2);
    assert!(
        output.stderr.contains(
            "package_name `unified-agent-api` already exists in workspace member `crates/agent_api` (crates/agent_api/Cargo.toml)"
        ),
        "stderr did not mention workspace package name conflict:\n{}",
        output.stderr
    );
}

#[test]
fn onboard_agent_preexisting_target_conflict_exits_with_validation_code() {
    let fixture = fixture_root("onboard-agent-target-conflict");
    write_text(
        &fixture.join("cli_manifests/cursor/current.json"),
        "{\n  \"expected_targets\": [\"darwin-arm64\"],\n  \"inputs\": []\n}\n",
    );

    let output = run_cli(base_args("cursor"), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("conflicts with proposed canonical_targets"));
}

#[test]
fn onboard_agent_dry_run_preview_is_deterministic_and_writes_nothing() {
    let fixture = fixture_root("onboard-agent-preview");
    write_text(
        &fixture.join("docs/crates-io-release.md"),
        "# Release docs\n\nManual contract text.\n",
    );
    write_text(
        &fixture.join(".github/workflows/publish-crates.yml"),
        "name: publish-crates\n",
    );
    write_text(
        &fixture.join("scripts/publish_crates.py"),
        "print('publish')\n",
    );
    write_text(
        &fixture.join("scripts/validate_publish_versions.py"),
        "print('validate')\n",
    );
    write_text(
        &fixture.join("scripts/check_publish_readiness.py"),
        "print('readiness')\n",
    );

    let before = snapshot_files(&fixture);
    let first = run_cli(base_args("cursor"), &fixture);
    let second = run_cli(base_args("cursor"), &fixture);
    let after = snapshot_files(&fixture);

    assert_eq!(first.exit_code, 0, "stderr:\n{}", first.stderr);
    assert_eq!(second.exit_code, 0, "stderr:\n{}", second.stderr);
    assert_eq!(
        first.stdout, second.stdout,
        "dry-run output must be deterministic"
    );
    assert_eq!(before, after, "dry-run must not write any files");

    let sections = [
        "== ONBOARD-AGENT DRY RUN ==",
        "== INPUT SUMMARY ==",
        "== REGISTRY ENTRY PREVIEW ==",
        "== DOCS SCAFFOLD PREVIEW ==",
        "== MANIFEST ROOT PREVIEW ==",
        "== RELEASE/PUBLICATION TOUCHPOINTS ==",
        "== MANUAL FOLLOW-UP ==",
        "== RESULT ==",
    ];
    let mut cursor = 0usize;
    for section in sections {
        let found = first.stdout[cursor..]
            .find(section)
            .map(|offset| cursor + offset)
            .unwrap_or_else(|| panic!("missing section `{section}` in stdout:\n{}", first.stdout));
        cursor = found + section.len();
    }

    assert!(first
        .stdout
        .contains("Path: crates/xtask/data/agent_registry.toml"));
    assert!(first
        .stdout
        .contains("Path: docs/project_management/next/cursor-cli-onboarding/README.md"));
    assert!(first
        .stdout
        .contains("Path: cli_manifests/cursor/current.json"));
    assert!(first
        .stdout
        .contains("Path: Cargo.toml will ensure workspace member `crates/cursor` is enrolled."));
    assert!(first.stdout.contains("Path: docs/crates-io-release.md will ensure the generated release block includes `unified-agent-api-cursor` on release track `crates-io`."));
    assert!(first
        .stdout
        .contains("Workflow and script files remain unchanged:"));
    assert!(first
        .stdout
        .contains("<!-- generated-by: xtask onboard-agent; owner: control-plane -->"));
    assert!(first.stdout.contains("## Manual Runtime Follow-Up"));
    assert!(first
        .stdout
        .contains("Shared onboarding plan preview; no filesystem writes performed."));
}

#[test]
fn onboard_agent_write_applies_plan_and_replays_identically() {
    let fixture = fixture_root("onboard-agent-write");
    seed_release_touchpoints(&fixture);

    let before = snapshot_files(&fixture);
    let first = run_cli(write_args("cursor"), &fixture);
    let after_first = snapshot_files(&fixture);
    let second = run_cli(write_args("cursor"), &fixture);
    let after_second = snapshot_files(&fixture);

    assert_eq!(first.exit_code, 0, "stderr:\n{}", first.stderr);
    assert_eq!(second.exit_code, 0, "stderr:\n{}", second.stderr);
    assert_ne!(before, after_first, "write mode must mutate the workspace");
    assert_eq!(
        after_first, after_second,
        "identical replay must be a no-op for filesystem state"
    );

    let sections = [
        "== ONBOARD-AGENT WRITE ==",
        "== INPUT SUMMARY ==",
        "== REGISTRY ENTRY PREVIEW ==",
        "== DOCS SCAFFOLD PREVIEW ==",
        "== MANIFEST ROOT PREVIEW ==",
        "== RELEASE/PUBLICATION TOUCHPOINTS ==",
        "== MANUAL FOLLOW-UP ==",
        "== RESULT ==",
    ];
    assert_sections_in_order(&first.stdout, &sections);
    assert_sections_in_order(&second.stdout, &sections);

    assert!(first
        .stdout
        .contains("Shared onboarding plan preview before apply."));
    assert!(first.stdout.contains("OK: onboard-agent write complete."));
    assert!(first
        .stdout
        .contains("Mutation summary: 15 written, 0 identical, 15 total planned."));
    assert!(second.stdout.contains("OK: onboard-agent write complete."));
    assert!(second
        .stdout
        .contains("Mutation summary: 0 written, 15 identical, 15 total planned."));

    let registry = fs::read_to_string(fixture.join("crates/xtask/data/agent_registry.toml"))
        .expect("read registry");
    assert!(registry.contains("agent_id = \"cursor\""));
    assert!(registry.contains("package_name = \"unified-agent-api-cursor\""));

    let root_manifest = fs::read_to_string(fixture.join("Cargo.toml")).expect("read root manifest");
    assert!(root_manifest.contains("\"crates/cursor\""));
    let wrapper_events_index = root_manifest
        .find("\"crates/wrapper_events\"")
        .expect("wrapper_events member");
    let cursor_index = root_manifest
        .find("\"crates/cursor\"")
        .expect("cursor member");
    assert!(cursor_index < wrapper_events_index);

    let readme = fs::read_to_string(
        fixture.join("docs/project_management/next/cursor-cli-onboarding/README.md"),
    )
    .expect("read docs README");
    assert!(readme.contains("# Cursor CLI onboarding pack"));

    let current_json = fs::read_to_string(fixture.join("cli_manifests/cursor/current.json"))
        .expect("read current");
    assert!(current_json.contains("\"expected_targets\": ["));
    let release_doc =
        fs::read_to_string(fixture.join("docs/crates-io-release.md")).expect("read release doc");
    assert!(release_doc
        .contains("<!-- generated-by: xtask onboard-agent; section: crates-io-release -->"));
    assert!(release_doc.contains("`unified-agent-api-cursor`"));
    assert!(release_doc.contains("## Publish order"));
    assert!(fixture
        .join("cli_manifests/cursor/versions/.gitkeep")
        .is_file());
    assert!(fixture
        .join("cli_manifests/cursor/pointers/latest_supported/.gitkeep")
        .is_file());
}

#[test]
fn onboard_agent_write_rejects_divergent_replay_without_mutating_other_files() {
    let fixture = fixture_root("onboard-agent-divergent-replay");
    seed_release_touchpoints(&fixture);

    let first = run_cli(write_args("cursor"), &fixture);
    assert_eq!(first.exit_code, 0, "stderr:\n{}", first.stderr);

    write_text(
        &fixture.join("cli_manifests/cursor/current.json"),
        "{\n  \"tampered\": true\n}\n",
    );
    let before = snapshot_files(&fixture);
    let second = run_cli(write_args("cursor"), &fixture);
    let after = snapshot_files(&fixture);

    assert_eq!(second.exit_code, 2);
    assert!(second.stderr.contains("cli_manifests/cursor/current.json"));
    assert!(second.stderr.contains("divergent"));
    assert_eq!(
        before, after,
        "divergent replay must not leave partial writes"
    );
}

#[test]
fn onboard_agent_write_allows_preexisting_runtime_owned_directories() {
    let fixture = fixture_root("onboard-agent-preexisting-runtime-dirs");
    seed_release_touchpoints(&fixture);
    fs::create_dir_all(fixture.join("crates/cursor")).expect("create runtime crate dir");
    fs::create_dir_all(fixture.join("crates/agent_api/src/backends/cursor"))
        .expect("create backend module dir");

    let output = run_cli(write_args("cursor"), &fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert!(output.stdout.contains("OK: onboard-agent write complete."));
}

#[cfg(unix)]
#[test]
fn onboard_agent_write_rejects_symlink_escape_paths() {
    use std::os::unix::fs::symlink;

    let fixture = fixture_root("onboard-agent-symlink-escape");
    seed_release_touchpoints(&fixture);
    let outside = fixture_root("onboard-agent-symlink-outside");
    fs::create_dir_all(fixture.join("docs/project_management/next")).expect("create docs parent");
    symlink(
        &outside,
        fixture.join("docs/project_management/next/linked"),
    )
    .expect("create symlink");

    let output = run_cli(
        args_with_overrides(
            "--write",
            "cursor",
            "unified-agent-api-cursor",
            &[("--onboarding-pack-prefix", "linked/escape-pack")],
            false,
        ),
        &fixture,
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("symlinked component"));
    assert!(output
        .stderr
        .contains("docs/project_management/next/linked/escape-pack"));
}

fn run_cli<I, S>(argv: I, workspace_root: &Path) -> HarnessOutput
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = argv
        .into_iter()
        .map(|arg| arg.as_ref().to_string())
        .collect::<Vec<_>>();

    match Cli::try_parse_from(args) {
        Ok(cli) => {
            let mut stdout = Vec::new();
            let mut stderr = String::new();
            let exit_code = match cli.command {
                Command::OnboardAgent(args) => {
                    match onboard_agent::run_in_workspace(workspace_root, args, &mut stdout) {
                        Ok(()) => 0,
                        Err(err) => {
                            stderr = format!("{err}\n");
                            err.exit_code()
                        }
                    }
                }
            };
            HarnessOutput {
                exit_code,
                stdout: String::from_utf8(stdout).expect("stdout must be utf-8"),
                stderr,
            }
        }
        Err(err) => HarnessOutput {
            exit_code: err.exit_code(),
            stdout: String::new(),
            stderr: err.to_string(),
        },
    }
}

fn base_args(agent_id: &str) -> Vec<String> {
    base_args_with_package_name(agent_id, "unified-agent-api-cursor")
}

fn base_args_with_package_name(agent_id: &str, package_name: &str) -> Vec<String> {
    base_args_with_mode(agent_id, package_name, "--dry-run", false)
}

fn write_args(agent_id: &str) -> Vec<String> {
    base_args_with_mode(agent_id, "unified-agent-api-cursor", "--write", false)
}

fn base_args_with_mode(
    agent_id: &str,
    package_name: &str,
    mode_flag: &str,
    include_other_mode: bool,
) -> Vec<String> {
    args_with_overrides(mode_flag, agent_id, package_name, &[], include_other_mode)
}

fn args_with_overrides(
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

fn fixture_root(prefix: &str) -> PathBuf {
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

fn write_text(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

fn seed_release_touchpoints(root: &Path) {
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

fn assert_sections_in_order(stdout: &str, sections: &[&str]) {
    let mut cursor = 0usize;
    for section in sections {
        let found = stdout[cursor..]
            .find(section)
            .map(|offset| cursor + offset)
            .unwrap_or_else(|| panic!("missing section `{section}` in stdout:\n{stdout}"));
        cursor = found + section.len();
    }
}

fn snapshot_files(root: &Path) -> BTreeMap<String, Vec<u8>> {
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
