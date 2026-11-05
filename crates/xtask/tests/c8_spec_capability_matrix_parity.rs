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

            // Each gated capability the entry declares that projects onto a manifest command
            // gets that command on the gate's targets (all canonical targets when the gate names
            // none). The set derives from the registry, so no agent id is listed here.
            let declaration = &entry.capability_declaration;
            let gates = declaration
                .target_gated
                .iter()
                .map(|gate| (gate.capability_id.as_str(), Some(&gate.targets)))
                .chain(
                    declaration
                        .config_gated
                        .iter()
                        .map(|gate| (gate.capability_id.as_str(), gate.targets.as_ref())),
                );
            let mut commands = Vec::new();
            for (capability_id, targets) in gates {
                let Some(path) =
                    crate::capability_projection::manifest_projected_capability_path(capability_id)
                else {
                    continue;
                };
                let targets: Vec<&str> = targets
                    .filter(|targets| !targets.is_empty())
                    .unwrap_or(&entry.canonical_targets)
                    .iter()
                    .map(String::as_str)
                    .collect();
                commands.push(command(path, &targets));
            }

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

            let inventory =
                collect_inventory_from_registry(&registry, |entry| Ok(valid_manifest_for(entry)))
                    .expect("matching parity should pass");

            let enrolled: Vec<&str> = registry
                .capability_matrix_entries()
                .map(|entry| entry.agent_id.as_str())
                .collect();
            assert!(!enrolled.is_empty());
            assert_eq!(
                inventory
                    .backends
                    .keys()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                {
                    let mut sorted = enrolled.clone();
                    sorted.sort_unstable();
                    sorted
                }
            );
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
            let inventory =
                collect_inventory_from_registry(&registry, |entry| Ok(valid_manifest_for(entry)))
                    .expect("matching parity should pass");

            assert_eq!(
                inventory.canonical_target_header,
                "Canonical publication target profile: `codex=x86_64-unknown-linux-musl`, `claude_code=linux-x64`; `opencode`, `gemini_cli`, `aider` use their default lifecycle-backed target profile.\n"
            );
        }
    }
}
