//! Concurrent edits occur after actual ON4 delivery starts, before a later write fails.
use super::*;
use crate::{
    candidate_execution::tree::{self, Node, Tree},
    workspace_mutation::{PlannedMutation, WorkspaceMutationError},
};
use std::{cell::RefCell, os::unix::fs::PermissionsExt};

thread_local! {
    static CHANGE: RefCell<&'static str> = const { RefCell::new("") };
    static CONCURRENT: RefCell<Option<(String, Option<Node>)>> = const { RefCell::new(None) };
}

fn interrupt_delivery(
    root: &Path,
    mutations: &[PlannedMutation],
    index: usize,
) -> Result<(), WorkspaceMutationError> {
    if index != 2 {
        return Ok(());
    }
    // Identical outputs are skipped by the writer. Find two outputs already applied by
    // comparing their actual bytes to the clean committed invocation snapshot.
    let committed = crate::runtime_advertising::source::Source::tip(root, "HEAD").unwrap();
    let applied: Vec<_> = mutations
        .iter()
        .filter(|m| {
            let bytes = fs::read(root.join(m.relative_path())).unwrap();
            committed
                .optional_bytes(m.relative_path().to_str().unwrap())
                .unwrap()
                .as_ref()
                != Some(&bytes)
        })
        .collect();
    assert_eq!(applied.len(), 2, "failure must follow two actual writes");
    let path = applied[0].relative_path().to_str().unwrap().to_owned();
    let full = root.join(&path);
    let delivered = fs::read(&full).unwrap();
    let mode = fs::metadata(&full).unwrap().permissions().mode() & 0o7777;
    let node = CHANGE.with(|change| match *change.borrow() {
        "replacement" => {
            fs::write(&full, b"concurrent replacement\n").unwrap();
            Some(Node::File(b"concurrent replacement\n".to_vec(), mode))
        }
        "removal" => {
            fs::remove_file(&full).unwrap();
            None
        }
        "mode" => {
            let changed_mode = mode ^ 0o100;
            fs::set_permissions(&full, fs::Permissions::from_mode(changed_mode)).unwrap();
            Some(Node::File(delivered, changed_mode))
        }
        "symlink" => {
            fs::remove_file(&full).unwrap();
            std::os::unix::fs::symlink("concurrent-target", &full).unwrap();
            Some(Node::Link("concurrent-target".into()))
        }
        _ => unreachable!(),
    });
    CONCURRENT.with(|slot| slot.replace(Some((path, node))));
    Err(WorkspaceMutationError::Internal(
        "injected later ON4 write failure".into(),
    ))
}

pub(super) fn check(root: &Path, before: &Tree) {
    publication_refresh::set_test_gate_runner(Some(green));
    for change in ["replacement", "removal", "mode", "symlink"] {
        eprintln!("ON4 partial-delivery regression case {change}");
        CHANGE.with(|slot| slot.replace(change));
        let error =
            publication_refresh::routed::with_delivery_hook(interrupt_delivery, || refresh(root))
                .unwrap_err()
                .to_string();
        let (path, node) = CONCURRENT
            .with(|slot| slot.take())
            .expect("delivery hook never ran");
        assert!(
            error.contains("preserving intervening change") || error.contains("symlink"),
            "{change}: {error}"
        );
        let mut expected = before.clone();
        match node {
            Some(node) => {
                expected.insert(path.clone(), node);
            }
            None => {
                expected.remove(&path);
            }
        }
        let mut actual = tree::capture(root, &Default::default()).unwrap();
        // A rollback conflict deliberately retains original bytes in its recovery directory.
        let recovery: Vec<_> = actual
            .keys()
            .filter(|p| p.starts_with(".xtask-onboard-agent-") && !p.contains('/'))
            .cloned()
            .collect();
        assert_eq!(recovery.len(), 1);
        let Node::File(original, original_mode) = &before[&path] else {
            panic!("output was not a file")
        };
        assert!(
            fs::read_dir(root.join(&recovery[0])).unwrap().any(|entry| {
                let entry = entry.unwrap();
                entry.path().extension().is_some_and(|ext| ext == "bak")
                    && fs::read(entry.path()).unwrap() == *original
            }),
            "original output must remain recoverable"
        );
        actual.retain(|p, _| !p.starts_with(".xtask-onboard-agent-"));
        assert_eq!(
            actual, expected,
            "{change}: other owned outputs must be restored"
        );
        for directory in recovery {
            fs::remove_dir_all(root.join(directory)).unwrap();
        }
        // Restore only test-owned concurrency before exercising the next sibling.
        if fs::symlink_metadata(root.join(&path)).is_ok() {
            fs::remove_file(root.join(&path)).unwrap();
        }
        write(root, &path, original);
        fs::set_permissions(root.join(&path), fs::Permissions::from_mode(*original_mode)).unwrap();
        assert_eq!(&tree::capture(root, &Default::default()).unwrap(), before);
    }
    publication_refresh::set_test_gate_runner(None);
}
