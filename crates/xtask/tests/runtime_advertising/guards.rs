use super::*;
use fixtures::*;

#[test]
fn source_bypasses_are_refused_without_rewriting_outputs() {
    let cases = [
        (
            "crates/agent_api/src/backends/opencode/mod.rs",
            "const AGENT_KIND: &str = \"opencode\";",
            "const AGENT_KIND: &str = \"aider\";",
            "runtime identity",
        ),
        (
            "crates/agent_api/src/backends/codex/backend.rs",
            "AgentWrapperKind(\"codex\".to_string())",
            "AgentWrapperKind(\"claude_code\".to_string())",
            "runtime kind",
        ),
        (
            "crates/agent_api/src/backends/codex/backend.rs",
            "&crate::runtime_advertising::data::CODEX",
            "&crate::runtime_advertising::data::OPENCODE",
            "capabilities delegate",
        ),
        (
            "crates/agent_api/src/backends/codex/backend.rs",
            "allow_mcp_write: self.config.allow_mcp_write",
            "allow_mcp_write: self.config.allow_external_sandbox_exec",
            "capabilities delegate",
        ),
        (
            "crates/agent_api/src/backends/codex/backend.rs",
            "fn capabilities(&self)",
            "#[cfg(target_os = \"linux\")] fn capabilities(&self)",
            "capabilities delegate",
        ),
        (
            "crates/agent_api/src/backends/codex/mod.rs",
            "Self { config }",
            "Self { config: CodexBackendConfig::default() }",
            "constructor",
        ),
        (
            "crates/agent_api/src/backends/codex/mod.rs",
            "#[derive(Clone, Debug, Default)]",
            "#[derive(Clone, Debug)]",
            "CodexBackendConfig",
        ),
        (
            "crates/agent_api/src/backends/mod.rs",
            "#[cfg(feature = \"codex\")]",
            "#[cfg(any(feature = \"codex\", feature = \"claude_code\"))]",
            "codex",
        ),
        (
            "crates/agent_api/src/lib.rs",
            "mod runtime_advertising;",
            "#[path = \"replacement.rs\"] mod runtime_advertising;",
            "runtime_advertising",
        ),
        (
            "crates/agent_api/src/runtime_advertising/mod.rs",
            "std::env::consts::ARCH",
            "\"x86_64\"",
            "unsupported semantics",
        ),
        (
            "crates/agent_api/src/runtime_advertising/evaluator.rs",
            "if host_matches && config_matches",
            "if host_matches || config_matches",
            "unsupported semantics",
        ),
        (
            "crates/agent_api/Cargo.toml",
            "[features]",
            "[features]\nextras = []",
            "feature keys",
        ),
        (
            "crates/agent_api/Cargo.toml",
            "codex = [\"dep:codex\",",
            "codex = [\"claude_code\", \"dep:codex\", \"codex\",",
            "cyclic",
        ),
    ];
    for (path, old, new, message) in cases {
        let temp = fixture();
        edit(temp.path(), path, old, new);
        let before = std::fs::read(temp.path().join(xtask::runtime_advertising::DATA)).unwrap();
        let error = Revision::workspace(temp.path())
            .err()
            .expect("bypass accepted");
        assert!(error.contains(message), "{path}: {error}");
        for flags in [
            vec!["runtime-advertising-model", "--check"],
            vec!["runtime-advertising-model"],
        ] {
            let out = Command::new(env!("CARGO_BIN_EXE_xtask"))
                .args(flags)
                .current_dir(temp.path())
                .output()
                .unwrap();
            assert_eq!(out.status.code(), Some(2), "{out:?}");
            assert_eq!(
                std::fs::read(temp.path().join(xtask::runtime_advertising::DATA)).unwrap(),
                before
            );
        }
    }
}

#[test]
fn source_only_added_advertising_and_identity_aliases_are_refused() {
    let temp = fixture();
    edit(temp.path(), "crates/agent_api/src/backends/opencode/backend.rs", "        AgentWrapperCapabilities {", "        let mut added = std::collections::BTreeSet::new(); added.insert(\"agent_api.exec.add_dirs.v1\".into()); return AgentWrapperCapabilities { ids: added }; AgentWrapperCapabilities {");
    assert!(Revision::workspace(temp.path())
        .err()
        .unwrap()
        .contains("capabilities delegate"));
    let temp = fixture();
    edit(
        temp.path(),
        "crates/agent_api/src/backends/claude_code/backend.rs",
        "AGENT_KIND};",
        "AGENT_KIND as UNUSED_ID};",
    );
    edit(
        temp.path(),
        "crates/agent_api/src/backends/claude_code/backend.rs",
        "use crate::{",
        "use crate::fake::AGENT_KIND; use crate::{",
    );
    assert!(Revision::workspace(temp.path())
        .err()
        .unwrap()
        .contains("import binding"));
}

#[test]
fn formatting_is_accepted_but_data_or_unknown_input_changes_fail() {
    let temp = fixture();
    let path = temp.path().join(xtask::runtime_advertising::DATA);
    std::fs::write(
        &path,
        format!(
            "// formatting\n{}",
            std::fs::read_to_string(&path).unwrap().replace(';', ";\n")
        ),
    )
    .unwrap();
    Revision::workspace(temp.path()).unwrap();
    edit(
        temp.path(),
        xtask::runtime_advertising::DATA,
        "MODEL_REVISION: u32 = 1",
        "MODEL_REVISION: u32 = 2",
    );
    assert!(Revision::workspace(temp.path())
        .err()
        .unwrap()
        .contains("data is stale"));
    let temp = fixture();
    std::fs::write(
        temp.path().join("crates/agent_api/build.rs"),
        "fn main() {} ",
    )
    .unwrap();
    assert!(Revision::workspace(temp.path())
        .err()
        .unwrap()
        .contains("Cargo/build"));
}

#[test]
fn missing_keys_shared_dependencies_and_root_build_overrides_are_refused() {
    for (path, old, new) in [
        (
            "crates/agent_api/Cargo.toml",
            "opencode = [",
            "extra_opencode = [",
        ),
        (
            "crates/agent_api/Cargo.toml",
            "codex = [\"dep:codex\", \"dep:tokio\",",
            "codex = [\"dep:codex\",",
        ),
        (
            "Cargo.toml",
            "[workspace]",
            "[patch.crates-io]\n\n[workspace]",
        ),
    ] {
        let temp = fixture();
        edit(temp.path(), path, old, new);
        assert!(Revision::workspace(temp.path()).is_err());
    }
    let temp = fixture();
    std::fs::create_dir(temp.path().join(".cargo")).unwrap();
    std::fs::write(temp.path().join(".cargo/config"), "[build]\nrustflags = []").unwrap();
    assert!(Revision::workspace(temp.path())
        .err()
        .unwrap()
        .contains("Cargo configuration"));
}
