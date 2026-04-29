mod agent_registry {
    pub use xtask::agent_registry::*;
}

mod capability_projection {
    #![allow(dead_code)]

    include!("../src/capability_projection.rs");
}

mod capability_matrix {
    #![allow(dead_code)]

    include!("../src/capability_matrix.rs");

    const SEEDED_REGISTRY: &str = include_str!("../data/agent_registry.toml");

    fn manifest_with_commands(commands: &[(&[&str], &[&str])]) -> UnionManifest {
        let expected_targets = commands
            .iter()
            .flat_map(|(_, available_on)| available_on.iter().copied())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .map(str::to_string)
            .collect();

        UnionManifest {
            expected_targets,
            commands: commands
                .iter()
                .map(|(path, available_on)| UnionCommand {
                    path: path.iter().map(|segment| (*segment).to_string()).collect(),
                    available_on: available_on
                        .iter()
                        .map(|target| (*target).to_string())
                        .collect(),
                })
                .collect(),
        }
    }

    fn seeded_registry() -> AgentRegistry {
        AgentRegistry::parse(SEEDED_REGISTRY).expect("parse seeded registry")
    }

    fn capability_matrix_backend_ids(registry: &AgentRegistry) -> Vec<String> {
        registry
            .capability_matrix_entries()
            .map(|entry| entry.agent_id.clone())
            .collect()
    }

    #[test]
    fn command_available_on_target_matches_exact_path_and_target() {
        let manifest = manifest_with_commands(&[
            (&["mcp", "list"], &["linux-x64"]),
            (&["mcp", "get"], &["win32-x64"]),
        ]);

        assert!(command_available_on_target(
            &manifest,
            &["mcp", "list"],
            "linux-x64"
        ));
        assert!(!command_available_on_target(
            &manifest,
            &["mcp"],
            "linux-x64"
        ));
        assert!(!command_available_on_target(
            &manifest,
            &["mcp", "get"],
            "linux-x64"
        ));
    }

    #[test]
    fn manifest_projection_adds_read_caps_and_clears_unavailable_caps() {
        let manifest = manifest_with_commands(&[
            (&["mcp", "list"], &["x86_64-unknown-linux-musl"]),
            (&["mcp", "get"], &["x86_64-unknown-linux-musl"]),
        ]);
        let registry = seeded_registry();
        let codex = registry.find("codex").expect("seeded codex entry");
        let projected =
            projected_advertised_capabilities(codex, &manifest).expect("projected capabilities");

        assert!(projected.contains("agent_api.tools.mcp.list.v1"));
        assert!(projected.contains("agent_api.tools.mcp.get.v1"));
        assert!(!projected.contains("agent_api.tools.mcp.add.v1"));
        assert!(!projected.contains("agent_api.tools.mcp.remove.v1"));
    }

    #[test]
    fn resolve_output_path_defaults_to_workspace_root() {
        assert_eq!(
            resolve_output_path(None).expect("resolve default output path"),
            resolve_workspace_root()
                .expect("resolve workspace root")
                .join(DEFAULT_OUT_PATH)
        );
    }

    #[test]
    fn resolve_output_path_preserves_absolute_path() {
        let absolute = std::env::temp_dir().join("capability-matrix-absolute.md");
        assert_eq!(
            resolve_output_path(Some(absolute.as_path())).expect("resolve absolute output path"),
            absolute
        );
    }

    #[test]
    fn resolve_output_path_preserves_explicit_relative_path() {
        let relative = Path::new("tmp/capability-matrix.md");
        assert_eq!(
            resolve_output_path(Some(relative)).expect("resolve relative output path"),
            relative
        );
    }

    #[test]
    fn collect_builtin_backend_capabilities_includes_opencode() {
        let backends = collect_builtin_backend_capabilities().expect("collect backends");

        assert!(backends.contains_key("opencode"));
        assert_eq!(
            backends["opencode"].ids,
            [
                "agent_api.run".to_string(),
                "agent_api.events".to_string(),
                "agent_api.events.live".to_string(),
                "agent_api.config.model.v1".to_string(),
                "agent_api.session.resume.v1".to_string(),
                "agent_api.session.fork.v1".to_string(),
            ]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn capability_matrix_backend_ids_follow_registry_enrollment_order() {
        let registry = seeded_registry();

        assert_eq!(
            capability_matrix_backend_ids(&registry),
            vec![
                "codex".to_string(),
                "claude_code".to_string(),
                "opencode".to_string(),
                "gemini_cli".to_string(),
                "aider".to_string()
            ]
        );
    }

    #[test]
    fn runtime_backend_kinds_match_seeded_registry_agent_ids() {
        let registry = seeded_registry();

        for entry in registry.capability_matrix_entries() {
            let (backend_kind, _) =
                runtime_backend_capabilities(&entry.agent_id).expect("runtime backend");
            assert_eq!(backend_kind, entry.agent_id);
        }
    }

    #[test]
    fn canonical_target_header_uses_registry_publication_targets() {
        let registry = seeded_registry();
        let entries: Vec<&AgentRegistryEntry> = registry.capability_matrix_entries().collect();

        assert_eq!(
            render_canonical_target_header(&entries).expect("render header"),
            "Canonical target profile: `codex=x86_64-unknown-linux-musl`, `claude_code=linux-x64`; `opencode`, `gemini_cli`, `aider` use the default built-in backend config.\n"
        );
    }

    #[test]
    fn registry_driven_mcp_projection_uses_explicit_publication_target() {
        let registry = seeded_registry();
        let claude = registry
            .find("claude_code")
            .expect("seeded claude_code entry");
        let manifest = manifest_with_commands(&[
            (&["mcp", "list"], &["linux-x64", "win32-x64"]),
            (&["mcp", "get"], &["linux-x64", "win32-x64"]),
            (&["mcp", "add"], &["linux-x64", "win32-x64"]),
            (&["mcp", "remove"], &["linux-x64", "win32-x64"]),
        ]);
        let mut capabilities = AgentWrapperCapabilities {
            ids: [
                "agent_api.tools.mcp.list.v1".to_string(),
                "agent_api.tools.mcp.get.v1".to_string(),
                "agent_api.tools.mcp.add.v1".to_string(),
                "agent_api.tools.mcp.remove.v1".to_string(),
            ]
            .into_iter()
            .collect(),
        };

        let advertised =
            projected_advertised_capabilities(claude, &manifest).expect("projected capabilities");

        capabilities.ids = advertised;
        assert!(capabilities.contains("agent_api.tools.mcp.list.v1"));
        assert!(!capabilities.contains("agent_api.tools.mcp.get.v1"));
        assert!(!capabilities.contains("agent_api.tools.mcp.add.v1"));
        assert!(!capabilities.contains("agent_api.tools.mcp.remove.v1"));
    }
}
