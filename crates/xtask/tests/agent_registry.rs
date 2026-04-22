#[path = "../src/agent_registry.rs"]
mod agent_registry;

use std::path::PathBuf;

use agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH};

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
        vec!["codex", "claude_code", "opencode", "gemini_cli"]
    );

    let support_ids: Vec<&str> = registry
        .support_matrix_entries()
        .map(|agent| agent.agent_id.as_str())
        .collect();
    assert_eq!(
        support_ids,
        vec!["codex", "claude_code", "opencode", "gemini_cli"]
    );

    let capability_ids: Vec<&str> = registry
        .capability_matrix_entries()
        .map(|agent| agent.agent_id.as_str())
        .collect();
    assert_eq!(
        capability_ids,
        vec!["codex", "claude_code", "opencode", "gemini_cli"]
    );

    let codex = registry.find("codex").expect("seeded codex entry");
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
        vec!["codex", "claude_code", "opencode", "gemini_cli"]
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
                "path = \"docs/project_management/next/opencode-implementation/governance/seam-3-closeout.md\"",
                "path = \"docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md\"",
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
