#![cfg(feature = "codex")]

use std::path::PathBuf;

use agent_api::{
    list_runtime_support, resolve_runtime_support, AgentWrapperError, RuntimeSupportRecord,
};
use tempfile::tempdir;

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
    let resolved = resolve_runtime_support("codex", "x86_64-unknown-linux-musl")
        .expect("resolve current codex linux tuple");
    assert_eq!(
        resolved,
        RuntimeSupportRecord {
            runtime_family: "codex".to_string(),
            target_triple: "x86_64-unknown-linux-musl".to_string(),
            version: "0.125.0".to_string(),
        }
    );

    let listed = list_runtime_support("codex").expect("list codex tuples");
    assert_eq!(listed, vec![resolved.clone()]);

    let tmp = tempdir().expect("create temp dir");
    let _guard = CurrentDirGuard::change_to(tmp.path());
    let resolved_without_repo = resolve_runtime_support("codex", "x86_64-unknown-linux-musl")
        .expect("resolve without repo checkout");
    let listed_without_repo = list_runtime_support("codex").expect("list without repo checkout");
    assert_eq!(resolved_without_repo, resolved);
    assert_eq!(listed_without_repo, vec![resolved.clone()]);

    let err = resolve_runtime_support("codex", "aarch64-apple-darwin")
        .expect_err("darwin target should not publish latest_validated");
    match err {
        AgentWrapperError::MissingValidatedRuntime {
            runtime_family,
            target_triple,
        } => {
            assert_eq!(runtime_family, "codex");
            assert_eq!(target_triple, "aarch64-apple-darwin");
        }
        other => panic!("expected MissingValidatedRuntime, got {other:?}"),
    }

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
