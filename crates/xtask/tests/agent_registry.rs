#[path = "../src/agent_registry.rs"]
mod agent_registry;
mod capability_projection {
    #![allow(dead_code)]

    include!("../src/capability_projection.rs");
}

use std::path::PathBuf;

use agent_registry::{
    AgentRegistry, ReleaseWatchDispatchKind, ReleaseWatchSourceKind, ReleaseWatchVersionPolicy,
    REGISTRY_RELATIVE_PATH,
};

const SEEDED_REGISTRY: &str = include_str!("../data/agent_registry.toml");

#[test]
fn seeded_registry_parses_successfully() {
    let registry = AgentRegistry::parse(SEEDED_REGISTRY).expect("parse seeded registry");

    let agent_ids: Vec<&str> = registry
        .agents
        .iter()
        .map(|agent| agent.agent_id.as_str())
        .collect();
    assert_eq!(
        agent_ids,
        vec!["codex", "claude_code", "opencode", "gemini_cli", "aider"]
    );

    let support_ids: Vec<&str> = registry
        .support_matrix_entries()
        .map(|agent| agent.agent_id.as_str())
        .collect();
    assert_eq!(
        support_ids,
        vec!["codex", "claude_code", "opencode", "gemini_cli", "aider"]
    );

    let capability_ids: Vec<&str> = registry
        .capability_matrix_entries()
        .map(|agent| agent.agent_id.as_str())
        .collect();
    assert_eq!(
        capability_ids,
        vec!["codex", "claude_code", "opencode", "gemini_cli", "aider"]
    );

    let codex = registry.find("codex").expect("seeded codex entry");
    let codex_watch = codex
        .maintenance
        .release_watch
        .as_ref()
        .expect("codex seeded release_watch enrollment");
    assert_eq!(
        codex.capability_declaration.target_gated.len(),
        2,
        "codex seeded target-gated declarations"
    );
    assert_eq!(
        codex.capability_declaration.config_gated.len(),
        3,
        "codex seeded config-gated declarations"
    );
    assert_eq!(
        codex.publication.capability_matrix_target.as_deref(),
        Some("x86_64-unknown-linux-musl")
    );
    assert!(codex_watch.enabled, "codex release watch stays enabled");
    assert_eq!(
        codex_watch.version_policy,
        ReleaseWatchVersionPolicy::LatestStableMinusOne
    );
    assert_eq!(
        codex_watch.dispatch_kind,
        ReleaseWatchDispatchKind::WorkflowDispatch
    );
    assert_eq!(
        codex_watch.dispatch_workflow.as_deref(),
        Some("codex-cli-update-snapshot.yml")
    );
    assert_eq!(
        codex_watch.upstream.source_kind,
        ReleaseWatchSourceKind::GithubReleases
    );
    assert_eq!(codex_watch.upstream.owner.as_deref(), Some("openai"));
    assert_eq!(codex_watch.upstream.repo.as_deref(), Some("codex"));
    assert_eq!(codex_watch.upstream.tag_prefix.as_deref(), Some("rust-v"));

    let claude = registry
        .find("claude_code")
        .expect("seeded claude_code entry");
    let claude_watch = claude
        .maintenance
        .release_watch
        .as_ref()
        .expect("claude_code seeded release_watch enrollment");
    assert!(
        claude_watch.enabled,
        "claude_code release watch stays enabled"
    );
    assert_eq!(
        claude_watch.version_policy,
        ReleaseWatchVersionPolicy::LatestStableMinusOne
    );
    assert_eq!(
        claude_watch.dispatch_kind,
        ReleaseWatchDispatchKind::WorkflowDispatch
    );
    assert_eq!(
        claude_watch.dispatch_workflow.as_deref(),
        Some("claude-code-update-snapshot.yml")
    );
    assert_eq!(
        claude_watch.upstream.source_kind,
        ReleaseWatchSourceKind::GcsObjectListing
    );
    assert_eq!(
        claude_watch.upstream.bucket.as_deref(),
        Some("claude-code-dist-86c565f3-f756-42ad-8dfa-d59b1c096819")
    );
    assert_eq!(
        claude_watch.upstream.prefix.as_deref(),
        Some("claude-code-releases")
    );
    assert_eq!(
        claude_watch.upstream.version_marker.as_deref(),
        Some("manifest.json")
    );

    let release_watch_ids: Vec<&str> = registry
        .agents
        .iter()
        .filter_map(|agent| {
            agent
                .maintenance
                .release_watch
                .as_ref()
                .and_then(|release_watch| release_watch.enabled.then_some(agent.agent_id.as_str()))
        })
        .collect();
    assert_eq!(
        release_watch_ids,
        vec!["codex", "claude_code"],
        "milestone 1 release_watch enrollment stays registry-only for codex and claude_code"
    );

    let opencode = registry.find("opencode").expect("seeded opencode entry");
    assert!(
        opencode.capability_declaration.target_gated.is_empty(),
        "opencode defaults absent target-gated bucket to empty"
    );
    assert!(
        opencode.capability_declaration.config_gated.is_empty(),
        "opencode defaults absent config-gated bucket to empty"
    );
    assert_eq!(
        opencode.maintenance.governance_checks.len(),
        2,
        "opencode seeds explicit governance checks"
    );
    assert!(
        opencode.maintenance.release_watch.is_none(),
        "opencode stays unenrolled in milestone 1"
    );

    let gemini = registry
        .find("gemini_cli")
        .expect("seeded gemini_cli entry");
    assert!(
        gemini.capability_declaration.target_gated.is_empty(),
        "gemini_cli defaults absent target-gated bucket to empty"
    );
    assert!(
        gemini.capability_declaration.config_gated.is_empty(),
        "gemini_cli defaults absent config-gated bucket to empty"
    );
    assert_eq!(
        gemini.maintenance.governance_checks.len(),
        1,
        "gemini_cli seeds approval-artifact governance checks"
    );
    assert_eq!(gemini.publication.capability_matrix_target, None);
    assert!(
        gemini.maintenance.release_watch.is_none(),
        "gemini_cli stays unenrolled in milestone 1"
    );

    let aider = registry.find("aider").expect("seeded aider entry");
    assert!(
        aider.maintenance.release_watch.is_none(),
        "aider stays unenrolled in milestone 1"
    );
}

#[test]
fn workspace_loader_reads_seeded_registry() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .to_path_buf();

    let registry = AgentRegistry::load(&workspace_root).expect("load registry from workspace");
    assert_eq!(
        registry
            .agents
            .iter()
            .map(|agent| agent.agent_id.as_str())
            .collect::<Vec<_>>(),
        vec!["codex", "claude_code", "opencode", "gemini_cli", "aider"]
    );
    assert_eq!(
        workspace_root.join(REGISTRY_RELATIVE_PATH),
        workspace_root.join("crates/xtask/data/agent_registry.toml")
    );
}

#[test]
fn malformed_toml_fails_closed() {
    let err = AgentRegistry::parse("[[agents]\nagent_id = \"codex\"").expect_err("malformed TOML");
    let text = err.to_string();
    assert!(text.contains("parse agent registry TOML"), "{text}");
}

#[test]
fn hyphenated_crate_path_basename_normalizes_into_scaffold_lib_name() {
    let raw = SEEDED_REGISTRY.replacen(
        "crate_path = \"crates/gemini_cli\"",
        "crate_path = \"crates/gemini-cli\"",
        1,
    );

    let registry = AgentRegistry::parse(&raw).expect("parse registry with hyphenated crate path");
    let gemini = registry
        .find("gemini_cli")
        .expect("seeded gemini_cli entry should still exist");

    assert_eq!(gemini.crate_path, "crates/gemini-cli");
    assert_eq!(
        gemini.scaffold_lib_name().expect("normalize lib name"),
        "gemini_cli"
    );
}

#[test]
fn duplicate_identity_and_path_fields_fail_closed() {
    let cases = [
        (
            "duplicate agent_id",
            SEEDED_REGISTRY.replacen("agent_id = \"claude_code\"", "agent_id = \"codex\"", 1),
            "duplicate agent_id `codex`",
        ),
        (
            "duplicate crate_path",
            SEEDED_REGISTRY.replacen(
                "crate_path = \"crates/claude_code\"",
                "crate_path = \"crates/codex\"",
                1,
            ),
            "duplicate crate_path `crates/codex`",
        ),
        (
            "duplicate backend_module",
            SEEDED_REGISTRY.replacen(
                "backend_module = \"crates/agent_api/src/backends/claude_code\"",
                "backend_module = \"crates/agent_api/src/backends/codex\"",
                1,
            ),
            "duplicate backend_module `crates/agent_api/src/backends/codex`",
        ),
        (
            "duplicate manifest_root",
            SEEDED_REGISTRY.replacen(
                "manifest_root = \"cli_manifests/claude_code\"",
                "manifest_root = \"cli_manifests/codex\"",
                1,
            ),
            "duplicate manifest_root `cli_manifests/codex`",
        ),
        (
            "duplicate package_name",
            SEEDED_REGISTRY.replacen(
                "package_name = \"unified-agent-api-claude-code\"",
                "package_name = \"unified-agent-api-codex\"",
                1,
            ),
            "duplicate package_name `unified-agent-api-codex`",
        ),
    ];

    for (label, raw, expected) in cases {
        let err = AgentRegistry::parse(&raw).unwrap_err();
        let text = err.to_string();
        assert!(text.contains(expected), "{label}: {text}");
    }
}

#[test]
fn unsupported_crate_path_basenames_fail_closed() {
    let cases = [
        (
            "dot basename",
            SEEDED_REGISTRY.replacen(
                "crate_path = \"crates/gemini_cli\"",
                "crate_path = \"crates/gemini.cli\"",
                1,
            ),
            "crate_path `crates/gemini.cli`",
        ),
        (
            "space basename",
            SEEDED_REGISTRY.replacen(
                "crate_path = \"crates/gemini_cli\"",
                "crate_path = \"crates/gemini cli\"",
                1,
            ),
            "crate_path `crates/gemini cli`",
        ),
    ];

    for (label, raw, expected_path) in cases {
        let err = AgentRegistry::parse(&raw).expect_err("invalid crate_path basename must fail");
        let text = err.to_string();
        assert!(text.contains(expected_path), "{label}: {text}");
        assert!(
            text.contains("normalized lib name candidate"),
            "{label}: {text}"
        );
        assert!(
            text.contains("ASCII letters, digits, or `_`"),
            "{label}: {text}"
        );
    }
}

#[test]
fn malformed_required_arrays_fail_closed() {
    let cases = [
        (
            "empty canonical_targets",
            SEEDED_REGISTRY.replacen(
                "canonical_targets = [\"x86_64-unknown-linux-musl\"]",
                "canonical_targets = []",
                1,
            ),
            "canonical_targets must contain at least one entry",
        ),
        (
            "missing always_on",
            SEEDED_REGISTRY.replace(
                "[agents.capability_declaration]\nalways_on = [\n  \"agent_api.run\",\n  \"agent_api.events\",\n  \"agent_api.events.live\",\n  \"agent_api.control.cancel.v1\",\n  \"agent_api.tools.structured.v1\",\n  \"agent_api.tools.results.v1\",\n  \"agent_api.artifacts.final_text.v1\",\n  \"agent_api.session.handle.v1\",\n  \"agent_api.session.fork.v1\",\n  \"agent_api.session.resume.v1\",\n  \"agent_api.config.model.v1\",\n  \"agent_api.exec.add_dirs.v1\",\n  \"agent_api.exec.non_interactive\",\n]\nbackend_extensions = [\n  \"backend.codex.exec.approval_policy\",\n  \"backend.codex.exec.sandbox_mode\",\n  \"backend.codex.exec_stream\",\n]",
                "[agents.capability_declaration]\nbackend_extensions = [\n  \"backend.codex.exec.approval_policy\",\n  \"backend.codex.exec.sandbox_mode\",\n  \"backend.codex.exec_stream\",\n]",
            ),
            "missing field `always_on`",
        ),
        (
            "missing backend_extensions",
            SEEDED_REGISTRY.replace(
                "[agents.capability_declaration]\nalways_on = [\n  \"agent_api.run\",\n  \"agent_api.events\",\n  \"agent_api.events.live\",\n  \"agent_api.control.cancel.v1\",\n  \"agent_api.tools.structured.v1\",\n  \"agent_api.tools.results.v1\",\n  \"agent_api.artifacts.final_text.v1\",\n  \"agent_api.session.handle.v1\",\n  \"agent_api.session.fork.v1\",\n  \"agent_api.session.resume.v1\",\n  \"agent_api.config.model.v1\",\n  \"agent_api.exec.add_dirs.v1\",\n  \"agent_api.exec.non_interactive\",\n]\nbackend_extensions = [\n  \"backend.codex.exec.approval_policy\",\n  \"backend.codex.exec.sandbox_mode\",\n  \"backend.codex.exec_stream\",\n]",
                "[agents.capability_declaration]\nalways_on = [\n  \"agent_api.run\",\n  \"agent_api.events\",\n  \"agent_api.events.live\",\n  \"agent_api.control.cancel.v1\",\n  \"agent_api.tools.structured.v1\",\n  \"agent_api.tools.results.v1\",\n  \"agent_api.artifacts.final_text.v1\",\n  \"agent_api.session.handle.v1\",\n  \"agent_api.session.fork.v1\",\n  \"agent_api.session.resume.v1\",\n  \"agent_api.config.model.v1\",\n  \"agent_api.exec.add_dirs.v1\",\n  \"agent_api.exec.non_interactive\",\n]",
            ),
            "missing field `backend_extensions`",
        ),
    ];

    for (label, raw, expected) in cases {
        let err = AgentRegistry::parse(&raw).unwrap_err();
        let text = err.to_string();
        assert!(text.contains(expected), "{label}: {text}");
    }
}

#[test]
fn malformed_gated_declarations_fail_closed() {
    let cases = [
        (
            "empty target-gated targets",
            SEEDED_REGISTRY.replace(
                "[[agents.capability_declaration.target_gated]]\ncapability_id = \"agent_api.tools.mcp.list.v1\"\ntargets = [\"x86_64-unknown-linux-musl\"]",
                "[[agents.capability_declaration.target_gated]]\ncapability_id = \"agent_api.tools.mcp.list.v1\"\ntargets = []",
            ),
            "capability_declaration.target_gated.targets must contain at least one target",
        ),
        (
            "unknown target-gated target",
            SEEDED_REGISTRY.replace(
                "[[agents.capability_declaration.target_gated]]\ncapability_id = \"agent_api.tools.mcp.list.v1\"\ntargets = [\"x86_64-unknown-linux-musl\"]",
                "[[agents.capability_declaration.target_gated]]\ncapability_id = \"agent_api.tools.mcp.list.v1\"\ntargets = [\"linux-x64\"]",
            ),
            "references undeclared canonical target `linux-x64`",
        ),
        (
            "empty config-gated targets",
            SEEDED_REGISTRY.replace(
                "[[agents.capability_declaration.config_gated]]\ncapability_id = \"agent_api.tools.mcp.add.v1\"\nconfig_key = \"allow_mcp_write\"\ntargets = [\"win32-x64\"]",
                "[[agents.capability_declaration.config_gated]]\ncapability_id = \"agent_api.tools.mcp.add.v1\"\nconfig_key = \"allow_mcp_write\"\ntargets = []",
            ),
            "capability_declaration.config_gated.targets must contain at least one target",
        ),
        (
            "empty config key",
            SEEDED_REGISTRY.replacen(
                "config_key = \"allow_external_sandbox_exec\"",
                "config_key = \"\"",
                1,
            ),
            "capability_declaration.config_gated.config_key must not be empty",
        ),
        (
            "unknown config key",
            SEEDED_REGISTRY.replacen(
                "config_key = \"allow_external_sandbox_exec\"",
                "config_key = \"allow_shell_everything\"",
                1,
            ),
            "capability_declaration.config_gated.config_key must be one of",
        ),
    ];

    for (label, raw, expected) in cases {
        let err = AgentRegistry::parse(&raw).unwrap_err();
        let text = err.to_string();
        assert!(text.contains(expected), "{label}: {text}");
    }
}

#[test]
fn capability_matrix_publication_target_validation_fails_closed() {
    let cases = [
        (
            "missing required publication target",
            SEEDED_REGISTRY.replacen(
                "capability_matrix_target = \"x86_64-unknown-linux-musl\"\n",
                "",
                1,
            ),
            "publication.capability_matrix_target must be set when capability-matrix projection uses target-scoped declarations",
        ),
        (
            "unknown publication target",
            SEEDED_REGISTRY.replacen(
                "capability_matrix_target = \"linux-x64\"",
                "capability_matrix_target = \"unknown-target\"",
                1,
            ),
            "publication.capability_matrix_target `unknown-target` must be listed in canonical_targets",
        ),
    ];

    for (label, raw, expected) in cases {
        let err = AgentRegistry::parse(&raw).unwrap_err();
        let text = err.to_string();
        assert!(text.contains(expected), "{label}: {text}");
    }
}

#[test]
fn malformed_governance_checks_fail_closed() {
    let cases = [
        (
            "duplicate governance path",
            SEEDED_REGISTRY.replacen(
                "path = \"docs/integrations/opencode/governance/seam-3-closeout.md\"",
                "path = \"docs/integrations/opencode/governance/seam-2-closeout.md\"",
                1,
            ),
            "maintenance.governance_checks contains duplicate path",
        ),
        (
            "missing markdown start marker",
            SEEDED_REGISTRY.replacen(
                "start_marker = \"<!-- xtask-governance-check:opencode-capabilities:start -->\"\n",
                "",
                1,
            ),
            "missing required `start_marker`",
        ),
        (
            "unsupported extraction mode for descriptor",
            SEEDED_REGISTRY.replacen(
                "comparison_kind = \"approved_agent_descriptor\"",
                "comparison_kind = \"approved_agent_descriptor\"\nstart_marker = \"<!-- nope:start -->\"\nend_marker = \"<!-- nope:end -->\"\nextraction_mode = \"inline_code_ids\"",
                1,
            ),
            "must not declare markdown parser config",
        ),
    ];

    for (label, raw, expected) in cases {
        let err = AgentRegistry::parse(&raw).unwrap_err();
        let text = err.to_string();
        assert!(text.contains(expected), "{label}: {text}");
    }
}

#[test]
fn malformed_release_watch_metadata_fails_closed() {
    let cases = [
        (
            "missing workflow for workflow dispatch",
            SEEDED_REGISTRY.replacen(
                "dispatch_workflow = \"codex-cli-update-snapshot.yml\"\n",
                "",
                1,
            ),
            "dispatch_workflow is required when dispatch_kind = `workflow_dispatch`",
        ),
        (
            "packet pr must not keep workflow field",
            SEEDED_REGISTRY
                .replacen(
                    "dispatch_kind = \"workflow_dispatch\"",
                    "dispatch_kind = \"packet_pr\"",
                    1,
                ),
            "dispatch_workflow must be omitted when dispatch_kind = `packet_pr`",
        ),
        (
            "github release watch missing repo",
            SEEDED_REGISTRY.replacen("repo = \"codex\"\n", "", 1),
            "upstream.repo is required for this upstream source",
        ),
        (
            "gcs object listing missing version marker",
            SEEDED_REGISTRY.replacen("version_marker = \"manifest.json\"\n", "", 1),
            "upstream.version_marker is required for this upstream source",
        ),
        (
            "github release watch must not declare gcs-only field",
            SEEDED_REGISTRY.replacen(
                "tag_prefix = \"rust-v\"",
                "tag_prefix = \"rust-v\"\nbucket = \"unexpected\"",
                1,
            ),
            "must not be set when maintenance.release_watch.upstream.source_kind = `github_releases`",
        ),
        (
            "release watch block may not be present disabled",
            SEEDED_REGISTRY.replacen(
                "enabled = true\nversion_policy = \"latest_stable_minus_one\"",
                "enabled = false\nversion_policy = \"latest_stable_minus_one\"",
                1,
            ),
            "enabled=false is not allowed",
        ),
        (
            "workflow dispatch path still requires source-specific fields",
            SEEDED_REGISTRY
                .replacen(
                    "dispatch_kind = \"workflow_dispatch\"",
                    "dispatch_kind = \"packet_pr\"",
                    1,
                )
                .replacen(
                    "dispatch_workflow = \"codex-cli-update-snapshot.yml\"\n",
                    "",
                    1,
                )
                .replacen("repo = \"codex\"\n", "", 1),
            "upstream.repo is required for this upstream source",
        ),
    ];

    for (label, raw, expected) in cases {
        let err = AgentRegistry::parse(&raw).unwrap_err();
        let text = err.to_string();
        assert!(text.contains(expected), "{label}: {text}");
    }
}

#[test]
fn generic_packet_pr_release_watch_schema_remains_valid() {
    let raw = SEEDED_REGISTRY.replacen(
        "[agents.maintenance]\n\n[[agents.maintenance.governance_checks]]",
        "[agents.maintenance]\n[agents.maintenance.release_watch]\nenabled = true\nversion_policy = \"latest_stable_minus_one\"\ndispatch_kind = \"packet_pr\"\n\n[agents.maintenance.release_watch.upstream]\nsource_kind = \"github_releases\"\nowner = \"example\"\nrepo = \"future-agent\"\ntag_prefix = \"v\"\n\n[[agents.maintenance.governance_checks]]",
        1,
    );

    let registry = AgentRegistry::parse(&raw).expect("packet_pr release_watch should parse");
    let opencode = registry.find("opencode").expect("seeded opencode entry");
    let release_watch = opencode
        .maintenance
        .release_watch
        .as_ref()
        .expect("packet_pr release_watch should be present");

    assert!(release_watch.enabled);
    assert_eq!(
        release_watch.dispatch_kind,
        ReleaseWatchDispatchKind::PacketPr
    );
    assert_eq!(release_watch.dispatch_workflow, None);
    assert_eq!(
        release_watch.upstream.source_kind,
        ReleaseWatchSourceKind::GithubReleases
    );
}

#[test]
fn invalid_governance_comparison_kind_fails_closed() {
    let raw = SEEDED_REGISTRY.replacen(
        "comparison_kind = \"approved_agent_descriptor\"",
        "comparison_kind = \"not_a_real_check\"",
        1,
    );
    let err = AgentRegistry::parse(&raw).unwrap_err();
    let text = err.to_string();
    assert!(text.contains("unknown variant"), "{text}");
}
