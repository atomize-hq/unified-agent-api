use super::*;
use fixtures::*;

#[path = "../../../agent_api/tests/runtime_advertising/expected.rs"]
mod baseline;

#[test]
fn complete_host_feature_and_config_domain_matches_the_pre_bridge_implementations() {
    let revision = Revision::workspace(&workspace()).unwrap();
    for (i, agent) in AGENTS.iter().enumerate() {
        for os in ["linux", "macos", "windows", "other"] {
            for arch in ["x86_64", "aarch64", "other"] {
                for requested in 0..32 {
                    for default_features in [false, true] {
                        for (write, sandbox, default_config) in [
                            (false, false, false),
                            (true, false, false),
                            (false, true, false),
                            (true, true, false),
                            (false, false, true),
                        ] {
                            let context = Context {
                                os,
                                arch,
                                write,
                                sandbox,
                                requested,
                                default_features,
                                default_config,
                            };
                            let expected = if requested & (1 << i) != 0 {
                                baseline::expected(agent, os, arch, write, sandbox)
                            } else {
                                Default::default()
                            };
                            assert_eq!(
                                revision.capabilities(agent, context).unwrap(),
                                expected,
                                "{agent} {context:?}"
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn default_features_and_opt_in_additions_compare_each_views_own_inputs() {
    let temp = fixture();
    let before = Revision::workspace(temp.path()).unwrap();
    edit(
        temp.path(),
        "crates/agent_api/Cargo.toml",
        "default = []",
        "default = [\"codex\"]",
    );
    let after = Revision::workspace(temp.path()).unwrap();
    let additions = after.additions(&before).unwrap();
    assert!(additions
        .iter()
        .any(|a| a.agent == "codex" && a.requested == 0 && a.default_features));
    assert!(additions.iter().all(|a| a.default_features));
    assert!(before.additions(&after).unwrap().is_empty());

    let temp = fixture();
    let before = Revision::workspace(temp.path()).unwrap();
    edit(
        temp.path(),
        "crates/xtask/data/agent_registry.toml",
        "config_key = \"allow_mcp_write\"",
        "config_key = \"allow_external_sandbox_exec\"",
    );
    regenerate(temp.path());
    let after = Revision::workspace(temp.path()).unwrap();
    assert!(after
        .additions(&before)
        .unwrap()
        .iter()
        .any(|a| a.capability == "agent_api.tools.mcp.add.v1"
            && a.agent == "codex"
            && !a.write
            && a.sandbox));
}

#[test]
fn broader_target_guard_is_an_addition_even_when_the_id_already_exists() {
    let temp = fixture();
    let before = Revision::workspace(temp.path()).unwrap();
    edit(
        temp.path(),
        "crates/xtask/data/agent_registry.toml",
        "capability_id = \"agent_api.tools.mcp.get.v1\"\ntargets = [\"win32-x64\"]",
        "capability_id = \"agent_api.tools.mcp.get.v1\"\ntargets = [\"win32-x64\", \"linux-x64\"]",
    );
    regenerate(temp.path());
    let after = Revision::workspace(temp.path()).unwrap();
    assert!(after
        .additions(&before)
        .unwrap()
        .iter()
        .any(|a| a.agent == "claude_code"
            && a.capability == "agent_api.tools.mcp.get.v1"
            && a.os == "linux"));
}

#[test]
fn alternative_guards_for_one_id_are_unioned_and_unknown_targets_are_not_false() {
    let temp = fixture();
    let text =
        std::fs::read_to_string(temp.path().join("crates/xtask/data/agent_registry.toml")).unwrap();
    let needle = "[agents.publication]";
    let extra = "[[agents.capability_declaration.config_gated]]\ncapability_id = \"agent_api.tools.mcp.add.v1\"\nconfig_key = \"allow_external_sandbox_exec\"\ntargets = [\"x86_64-unknown-linux-musl\"]\n\n";
    std::fs::write(
        temp.path().join("crates/xtask/data/agent_registry.toml"),
        text.replacen(needle, &format!("{extra}{needle}"), 1),
    )
    .unwrap();
    regenerate(temp.path());
    let revision = Revision::workspace(temp.path()).unwrap();
    for (write, sandbox) in [(true, false), (false, true), (true, true)] {
        assert!(revision
            .capabilities(
                "codex",
                Context {
                    os: "linux",
                    arch: "x86_64",
                    write,
                    sandbox,
                    requested: 4,
                    default_features: false,
                    default_config: false
                }
            )
            .unwrap()
            .contains("agent_api.tools.mcp.add.v1"));
    }
    assert!(xtask::runtime_advertising::projection::host_for_target("unsupported-target").is_err());
}
