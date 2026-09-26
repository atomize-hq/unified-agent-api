#![cfg(feature = "codex")]

use std::fs;
use std::path::{Path, PathBuf};

use agent_api::{
    list_runtime_support, resolve_runtime_support, AgentWrapperError, RuntimeSupportRecord,
};
use tempfile::tempdir;

const CODEX_TARGET_TRIPLES: &[&str] = &[
    "aarch64-apple-darwin",
    "aarch64-unknown-linux-musl",
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-linux-musl",
];

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("agent_api crate directory parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn expected_codex_records() -> Vec<RuntimeSupportRecord> {
    let workspace_root = workspace_root();

    CODEX_TARGET_TRIPLES
        .iter()
        .map(|target_triple| RuntimeSupportRecord {
            runtime_family: "codex".to_string(),
            target_triple: (*target_triple).to_string(),
            version: fs::read_to_string(
                workspace_root
                    .join("cli_manifests/codex/pointers/latest_validated")
                    .join(format!("{target_triple}.txt")),
            )
            .expect("read committed latest_validated pointer")
            .trim()
            .to_string(),
        })
        .collect()
}

struct CurrentDirGuard {
    original: PathBuf,
}

impl CurrentDirGuard {
    fn change_to(path: &std::path::Path) -> Self {
        let original = std::env::current_dir().expect("capture current dir");
        std::env::set_current_dir(path).expect("change current dir");
        Self { original }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.original).expect("restore current dir");
    }
}

#[test]
fn codex_runtime_support_is_validated_only_and_embedded() {
    let expected = expected_codex_records();
    let expected_linux_record = expected
        .iter()
        .find(|record| record.target_triple == "x86_64-unknown-linux-musl")
        .cloned()
        .expect("expected linux target");

    for record in &expected {
        let resolved = resolve_runtime_support("codex", &record.target_triple)
            .expect("resolve current codex tuple");
        assert_eq!(resolved, *record);
    }

    let listed = list_runtime_support("codex").expect("list codex tuples");
    assert_eq!(listed, expected);

    let tmp = tempdir().expect("create temp dir");
    let _guard = CurrentDirGuard::change_to(tmp.path());
    let resolved_without_repo = resolve_runtime_support("codex", "x86_64-unknown-linux-musl")
        .expect("resolve without repo checkout");
    let listed_without_repo = list_runtime_support("codex").expect("list without repo checkout");
    assert_eq!(resolved_without_repo, expected_linux_record);
    assert_eq!(listed_without_repo, listed);

    let err = resolve_runtime_support("codex", "linux-x64")
        .expect_err("non-triple alias should fail closed");
    match err {
        AgentWrapperError::UnsupportedTargetTriple {
            runtime_family,
            target_triple,
        } => {
            assert_eq!(runtime_family, "codex");
            assert_eq!(target_triple, "linux-x64");
        }
        other => panic!("expected UnsupportedTargetTriple, got {other:?}"),
    }

    let err = resolve_runtime_support("future_agent", "x86_64-unknown-linux-musl")
        .expect_err("unknown runtime family should fail closed");
    match err {
        AgentWrapperError::UnknownRuntimeFamily { runtime_family } => {
            assert_eq!(runtime_family, "future_agent");
        }
        other => panic!("expected UnknownRuntimeFamily, got {other:?}"),
    }
}
