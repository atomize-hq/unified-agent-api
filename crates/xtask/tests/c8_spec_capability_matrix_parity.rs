mod agent_registry {
    pub use xtask::agent_registry::*;
}

mod capability_publication {
    pub use xtask::capability_publication::*;
}

mod capability_projection {
    #![allow(dead_code)]

    include!("../src/capability_projection.rs");
}

mod capability_matrix {
    #![allow(dead_code)]

    include!("../src/capability_matrix.rs");

    #[cfg(test)]
    mod parity_tests {
        use super::*;
        use crate::agent_registry::AgentRegistry;

        #[derive(Debug)]
        struct LocalInventory {
            backends: std::collections::BTreeMap<String, agent_api::AgentWrapperCapabilities>,
            canonical_target_header: String,
        }

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
                "opencode" | "gemini_cli" | "aider" => Vec::new(),
                other => panic!("unexpected agent `{other}`"),
            };

            UnionManifest {
                expected_targets: entry.canonical_targets.clone(),
                commands,
            }
        }

        fn collect_inventory_from_registry<F>(
            registry: &AgentRegistry,
            mut manifest_loader: F,
        ) -> Result<LocalInventory, String>
        where
            F: FnMut(&AgentRegistryEntry) -> Result<UnionManifest, String>,
        {
            let enrolled_entries: Vec<&AgentRegistryEntry> =
                registry.capability_matrix_entries().collect();
            let mut backends = std::collections::BTreeMap::new();
            for entry in enrolled_entries.iter().copied() {
                let manifest = manifest_loader(entry)?;
                validate_capability_publication_target(entry, &manifest)?;
                let capabilities = projected_advertised_capabilities(entry, &manifest)?;
                backends.insert(
                    entry.agent_id.clone(),
                    agent_api::AgentWrapperCapabilities { ids: capabilities },
                );
            }

            Ok(LocalInventory {
                backends,
                canonical_target_header: render_canonical_target_header(&enrolled_entries)?,
            })
        }

        #[test]
        fn capability_matrix_parity_accepts_seeded_registry_with_matching_manifests() {
            let registry = seeded_registry();

            let inventory = collect_inventory_from_registry(&registry, |entry| {
                Ok(valid_manifest_for(entry))
            })
            .expect("matching parity should pass");

            assert!(inventory.backends.contains_key("codex"));
            assert!(inventory.backends.contains_key("claude_code"));
            assert!(inventory.backends.contains_key("opencode"));
            assert!(inventory.backends.contains_key("gemini_cli"));
            assert!(inventory.backends.contains_key("aider"));
        }

        #[test]
        fn capability_matrix_parity_rejects_missing_primary_expected_target() {
            let registry = seeded_registry();

            let err = collect_inventory_from_registry(&registry, |entry| {
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
        fn capability_matrix_parity_header_tracks_shared_publication_semantics() {
            let registry = seeded_registry();
            let inventory = collect_inventory_from_registry(&registry, |entry| {
                Ok(valid_manifest_for(entry))
            })
            .expect("matching parity should pass");

            assert_eq!(
                inventory.canonical_target_header,
                "Canonical publication target profile: `codex=x86_64-unknown-linux-musl`, `claude_code=linux-x64`; `opencode`, `gemini_cli`, `aider` use their default lifecycle-backed target profile.\n"
            );
        }
    }
}
