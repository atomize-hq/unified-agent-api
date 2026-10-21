use super::*;
use fixtures::*;

fn append(root: &std::path::Path, path: &str, text: &str) {
    use std::io::Write;
    std::fs::OpenOptions::new()
        .append(true)
        .open(root.join(path))
        .unwrap()
        .write_all(text.as_bytes())
        .unwrap();
}

#[test]
fn guarded_producers_aliases_and_expansion_are_refused_across_the_runtime_graph() {
    let cases = [
        ("backends/codex/mod.rs", "impl CodexBackendConfig { pub fn default() -> Self { Self { allow_external_sandbox_exec: true, ..<Self as Default>::default() } } }"),
        ("backends/codex/mod.rs", "impl crate::backends::codex::CodexBackend { pub fn alternative(config: CodexBackendConfig) -> Self { Self {config} } }"),
        ("backends/codex/mod.rs", "macro_rules! producer { () => { impl CodexBackend { pub fn alternative(config: CodexBackendConfig) -> Self { Self {config} } } }; } producer!();"),
        ("backends/codex/exec.rs", "impl super::CodexBackendConfig { pub fn default() -> Self { Self { allow_external_sandbox_exec: true, ..<Self as Default>::default() } } }"),
        ("backends/codex/exec.rs", "type Alias = super::CodexBackendConfig; impl Alias { pub fn default() -> Self { <Self as Default>::default() } }"),
        ("backends/codex/exec.rs", "use super::CodexBackendConfig as Alias; impl Alias { pub fn default() -> Self { <Self as Default>::default() } }"),
        ("backends/codex/exec.rs", "fn nested() { impl super::CodexBackendConfig { pub fn default() -> Self { <Self as Default>::default() } } }"),
        ("backends/codex/exec.rs", "mod nested { impl crate::backends::codex::CodexBackendConfig { pub fn default() -> Self { <Self as Default>::default() } } }"),
        ("backends/codex/exec.rs", "fn opaque() { let _ = serde_json::json!({\"x\": { impl super::CodexBackendConfig { pub fn default() -> Self { <Self as Default>::default() } } 1 }}); }"),
        ("backends/codex/exec.rs", "fn opaque() { let _ = serde_json::json!({\"x\": evil::hidden!() }); }"),
        ("backends/codex/exec.rs", "fn opaque() { use evil::hidden as format; let _ = format!(\"x\"); }"),
        ("backends/codex/exec.rs", "fn opaque() { use evil::Hidden as Default; #[derive(Default)] struct Hidden; }"),
        ("backends/codex/exec.rs", "fn opaque() { use evil::*; }"),
        ("backends/codex/exec.rs", "#[custom::derive_defaults] struct Hidden;"),
        ("backends/codex/exec.rs", "#[cfg_attr(unix, custom::derive_defaults)] struct Hidden;"),
        ("lib.rs", "mod std { pub use ::std::*; }"),
        ("runtime_support_data.rs", "impl crate::backends::codex::CodexBackendConfig { pub fn default() -> Self { <Self as Default>::default() } }"),
    ];
    let opaque_functions = [
        "fn opaque() { let _ = vec![{ #[custom::producer] fn marker() {} 0 }]; }",
        "fn opaque() { let _ = vec![{ #[custom::producer] const MARKER: usize = 0; MARKER }]; }",
        "fn opaque() { let _ = vec![{ #[custom::producer] static MARKER: usize = 0; MARKER }]; }",
        "fn opaque() { let _ = vec![{ #[custom::producer] let x = 0; x }]; }",
    ];
    for addition in opaque_functions {
        let temp = fixture();
        append(
            temp.path(),
            "crates/agent_api/src/backends/codex/exec.rs",
            &format!("\n{addition}\n"),
        );
        assert!(
            Revision::workspace(temp.path()).is_err(),
            "accepted opaque function/attribute: {addition}"
        );
    }
    for (relative, addition) in cases {
        let temp = fixture();
        append(
            temp.path(),
            &format!("crates/agent_api/src/{relative}"),
            &format!("\n{addition}\n"),
        );
        assert!(
            Revision::workspace(temp.path()).is_err(),
            "accepted {relative}: {addition}"
        );
    }
}

#[test]
fn unrelated_method_changes_pass_and_new_modules_are_checked_in_both_views() {
    let temp = fixture();
    let path = "crates/agent_api/src/backends/codex/exec.rs";
    append(temp.path(), path, "\nfn unrelated() -> usize { 1 }\n");
    Revision::workspace(temp.path()).unwrap();
    edit(
        temp.path(),
        path,
        "fn unrelated() -> usize { 1 }",
        "fn unrelated() -> usize { 2 }",
    );
    Revision::workspace(temp.path()).unwrap();
    git(temp.path(), &["init", "-q"]);
    commit(temp.path());
    append(temp.path(), path, "\nmod additional;\n");
    let extra = temp
        .path()
        .join("crates/agent_api/src/backends/codex/exec/additional.rs");
    std::fs::create_dir_all(extra.parent().unwrap()).unwrap();
    std::fs::write(extra, "impl crate::backends::codex::CodexBackendConfig { pub fn default() -> Self { <Self as Default>::default() } }").unwrap();
    let corrupt = commit(temp.path());
    assert!(Revision::workspace(temp.path()).is_err());
    git(temp.path(), &["checkout", "-q", "HEAD^", "--", path]);
    Revision::workspace(temp.path()).unwrap();
    assert!(Revision::tip(temp.path(), &corrupt).is_err());
}

#[test]
fn implicit_optional_dependency_features_are_rejected() {
    for table in ["dependencies", "build-dependencies", "dev-dependencies"] {
        let temp = fixture();
        if table == "dependencies" || table == "dev-dependencies" {
            edit(
                temp.path(),
                "crates/agent_api/Cargo.toml",
                &format!("[{table}]"),
                &format!("[{table}]\nserde = {{ workspace = true, optional = true }}"),
            );
        } else {
            append(
                temp.path(),
                "crates/agent_api/Cargo.toml",
                &format!("\n[{table}]\nserde = {{ workspace = true, optional = true }}\n"),
            );
        }
        assert!(Revision::workspace(temp.path())
            .err()
            .unwrap()
            .contains("implicit runtime feature"));
    }
}
