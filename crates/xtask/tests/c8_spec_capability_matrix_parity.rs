mod agent_registry {
    pub use xtask::agent_registry::*;
}

mod capability_matrix {
    #![allow(dead_code)]

    include!("../src/capability_matrix.rs");

    #[cfg(test)]
    mod parity_tests {
        use super::*;

        const SEEDED_REGISTRY: &str = include_str!("../data/agent_registry.toml");

        fn seeded_registry() -> AgentRegistry {
            AgentRegistry::parse(SEEDED_REGISTRY).expect("parse seeded registry")
        }

        fn valid_manifest_for(entry: &AgentRegistryEntry) -> UnionManifest {
            let command = |path: &[&str], available_on: &[&str]| UnionCommand {
                path: path.iter().map(|segment| (*segment).to_string()).collect(),
                available_on: available_on
                    .iter()
                    .map(|target| (*target).to_string())
                    .collect(),
            };

            let commands = match entry.agent_id.as_str() {
                "codex" => vec![
                    command(&["mcp", "list"], &["x86_64-unknown-linux-musl"]),
                    command(&["mcp", "get"], &["x86_64-unknown-linux-musl"]),
                    command(&["mcp", "add"], &["x86_64-unknown-linux-musl"]),
                    command(&["mcp", "remove"], &["x86_64-unknown-linux-musl"]),
                ],
                "claude_code" => vec![
                    command(
                        &["mcp", "list"],
                        &["linux-x64", "darwin-arm64", "win32-x64"],
                    ),
                    command(&["mcp", "get"], &["win32-x64"]),
                    command(&["mcp", "add"], &["win32-x64"]),
                    command(&["mcp", "remove"], &["win32-x64"]),
                ],
                "opencode" | "gemini_cli" => Vec::new(),
                other => panic!("unexpected agent `{other}`"),
            };

            UnionManifest {
                expected_targets: entry.canonical_targets.clone(),
                commands,
            }
        }

        #[test]
        fn capability_matrix_parity_accepts_seeded_registry_with_matching_manifests() {
            let registry = seeded_registry();

            let inventory = collect_builtin_backend_inventory_from_registry(&registry, |entry| {
                Ok(valid_manifest_for(entry))
            })
            .expect("matching parity should pass");

            assert!(inventory.backends.contains_key("codex"));
            assert!(inventory.backends.contains_key("claude_code"));
            assert!(inventory.backends.contains_key("opencode"));
            assert!(inventory.backends.contains_key("gemini_cli"));
        }

        #[test]
        fn capability_matrix_parity_rejects_missing_primary_expected_target() {
            let registry = seeded_registry();

            let err = collect_builtin_backend_inventory_from_registry(&registry, |entry| {
                let mut manifest = valid_manifest_for(entry);
                if entry.agent_id == "codex" {
                    manifest.expected_targets = vec![
                        "aarch64-apple-darwin".to_string(),
                        "x86_64-pc-windows-msvc".to_string(),
                    ];
                }
                Ok(manifest)
            })
            .expect_err("missing primary target should fail closed");

            assert!(err.contains("cli_manifests/codex/current.json"));
            assert!(err.contains("x86_64-unknown-linux-musl"));
            assert!(err.contains("expected_targets"));
        }

        #[test]
        fn capability_matrix_parity_rejects_declared_capability_beyond_runtime_truth() {
            let raw = SEEDED_REGISTRY.replace(
                "always_on = [\n  \"agent_api.run\",\n  \"agent_api.events\",\n  \"agent_api.events.live\",\n  \"agent_api.config.model.v1\",\n  \"agent_api.session.resume.v1\",\n  \"agent_api.session.fork.v1\",\n]",
                "always_on = [\n  \"agent_api.run\",\n  \"agent_api.events\",\n  \"agent_api.events.live\",\n  \"agent_api.config.model.v1\",\n  \"agent_api.session.resume.v1\",\n  \"agent_api.session.fork.v1\",\n  \"agent_api.tools.mcp.list.v1\",\n]",
            );
            let registry = AgentRegistry::parse(&raw).expect("parse mutated registry");

            let err = collect_builtin_backend_inventory_from_registry(&registry, |entry| {
                Ok(valid_manifest_for(entry))
            })
            .expect_err("declared/runtime mismatch should fail closed");

            assert!(err.contains("opencode"));
            assert!(err.contains("agent_api.tools.mcp.list.v1"));
            assert!(err.contains("modeled runtime truth"));
        }
    }
}
