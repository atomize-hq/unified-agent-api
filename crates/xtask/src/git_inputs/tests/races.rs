use super::*;

fn stage_and_remove(root: &Path, path: &str) {
    fs::write(root.join(path), "staged during observation").unwrap();
    git(root, &["add", path]).unwrap();
    fs::remove_file(root.join(path)).unwrap();
}

#[test]
fn verification_detects_index_changes_between_directory_checks() {
    for changed in [vec!["z/new"], vec!["z/new", "a/new"]] {
        let root = fixture();
        for dir in ["a", "z"] {
            fs::create_dir(root.path().join(dir)).unwrap();
        }
        let mut view = GitInputs::new(root.path()).unwrap();
        view.list("a").unwrap();
        view.list("z").unwrap();
        let hook_root = root.path().to_owned();
        let expected = if changed.len() == 1 { "z" } else { "a" };
        DIRECTORY_HOOK.with(|hook| {
            *hook.borrow_mut() = Some((
                "a".into(),
                Box::new(move || {
                    for path in changed {
                        stage_and_remove(&hook_root, path);
                    }
                }),
            ));
        });
        assert_eq!(
            view.verify(),
            Err(format!(
                "acquisition directory changed during freeze: {expected}"
            ))
        );
        DIRECTORY_HOOK.with(|hook| assert!(hook.borrow().is_none()));
    }
}

#[test]
fn traversal_detects_index_changes_between_directory_checks() {
    let root = fixture();
    for dir in ["scope/a", "scope/z"] {
        fs::create_dir_all(root.path().join(dir)).unwrap();
    }
    let mut view = GitInputs::new(root.path()).unwrap();
    let hook_root = root.path().to_owned();
    DIRECTORY_HOOK.with(|hook| {
        *hook.borrow_mut() = Some((
            "scope/a".into(),
            Box::new(move || {
                stage_and_remove(&hook_root, "scope/z/new");
            }),
        ));
    });
    assert_eq!(
        view.files_under("scope"),
        Err("acquisition directory changed during freeze: scope/z".into())
    );
    DIRECTORY_HOOK.with(|hook| assert!(hook.borrow().is_none()));
}

#[test]
fn verification_detects_index_changes_during_the_final_directory_pass() {
    let root = fixture();
    for dir in ["a", "z"] {
        fs::create_dir(root.path().join(dir)).unwrap();
    }
    let mut view = GitInputs::new(root.path()).unwrap();
    view.list("a").unwrap();
    view.list("z").unwrap();
    let hook_root = root.path().to_owned();
    DIRECTORY_HOOK.with(|hook| {
        *hook.borrow_mut() = Some((
            "a".into(),
            Box::new(move || {
                DIRECTORY_HOOK.with(|hook| {
                    *hook.borrow_mut() = Some((
                        "a".into(),
                        Box::new(move || {
                            stage_and_remove(&hook_root, "z/new");
                        }),
                    ));
                });
            }),
        ));
    });
    assert_eq!(
        view.verify(),
        Err("acquisition directory changed during freeze: z".into())
    );
    DIRECTORY_HOOK.with(|hook| assert!(hook.borrow().is_none()));
}

#[test]
fn verification_detects_staged_object_changes_between_streamed_reads() {
    let root = fixture();
    for path in ["a", "z"] {
        fs::write(root.path().join(path), "initial").unwrap();
    }
    git(root.path(), &["add", "."]).unwrap();
    let mut view = GitInputs::new(root.path()).unwrap();
    view.required("a").unwrap();
    view.required("z").unwrap();
    let hook_root = root.path().to_owned();
    FILE_HOOK.with(|hook| {
        *hook.borrow_mut() = Some((
            "a".into(),
            Box::new(move || {
                fs::write(hook_root.join("z"), "replacement").unwrap();
                git(&hook_root, &["add", "z"]).unwrap();
                fs::write(hook_root.join("z"), "initial").unwrap();
            }),
        ));
    });
    assert_eq!(
        view.verify(),
        Err("bound input changed during freeze: z".into())
    );
    FILE_HOOK.with(|hook| assert!(hook.borrow().is_none()));
}

#[cfg(target_os = "linux")]
#[test]
fn deep_traversal_keeps_only_one_live_batch_child() {
    let root = fixture();
    let mut deepest = String::from("scope");
    for _ in 0..64 {
        fs::create_dir_all(root.path().join(&deepest)).unwrap();
        fs::write(root.path().join(format!("{deepest}/a")), "initial").unwrap();
        deepest.push_str("/d");
    }
    fs::create_dir_all(root.path().join(&deepest)).unwrap();
    git(root.path(), &["add", "."]).unwrap();
    let mut view = GitInputs::new(root.path()).unwrap();
    DIRECTORY_HOOK.with(|hook| {
        *hook.borrow_mut() = Some((
            deepest,
            Box::new(|| {
                let children = fs::read_to_string("/proc/thread-self/children").unwrap();
                let count = children
                    .split_whitespace()
                    .filter(|pid| {
                        fs::read(format!("/proc/{pid}/cmdline"))
                            .is_ok_and(|cmd| cmd.split(|b| *b == 0).any(|arg| arg == b"--batch"))
                    })
                    .count();
                assert_eq!(count, 1, "live cat-file children at maximum depth");
            }),
        ));
    });
    assert_eq!(view.files_under("scope").unwrap().len(), 64);
    DIRECTORY_HOOK.with(|hook| assert!(hook.borrow().is_none()));
}

#[test]
fn verification_refuses_warm_packed_object_loss_after_the_last_read() {
    let root = fixture();
    fs::write(root.path().join("a"), "initial").unwrap();
    fs::write(root.path().join("z"), "initial").unwrap();
    git(root.path(), &["add", "."]).unwrap();
    batch::commit(root.path());
    git(root.path(), &["repack", "-ad"]).unwrap();
    git(root.path(), &["prune-packed"]).unwrap();
    let object = String::from_utf8(git(root.path(), &["rev-parse", ":z"]).unwrap()).unwrap();
    let mut view = GitInputs::new(root.path()).unwrap();
    view.required("a").unwrap();
    view.required("z").unwrap();
    let hook_root = root.path().to_owned();
    FILE_HOOK.with(|hook| {
        *hook.borrow_mut() = Some((
            "z".into(),
            Box::new(move || {
                for entry in fs::read_dir(hook_root.join(".git/objects/pack")).unwrap() {
                    fs::remove_file(entry.unwrap().path()).unwrap();
                }
            }),
        ));
    });
    let expected = git(root.path(), &["cat-file", "blob", object.trim()]);
    assert!(expected.is_ok());
    let result = view.verify();
    let expected = git(root.path(), &["cat-file", "blob", object.trim()]);
    assert!(expected.is_err());
    assert_eq!(result, expected.map(|_| ()));
    FILE_HOOK.with(|hook| assert!(hook.borrow().is_none()));
}

#[test]
fn traversal_detects_staged_object_changes_between_streamed_reads() {
    let root = fixture();
    fs::create_dir_all(root.path().join("scope/d")).unwrap();
    for path in ["scope/a", "scope/z"] {
        fs::write(root.path().join(path), "initial").unwrap();
    }
    git(root.path(), &["add", "."]).unwrap();
    let mut view = GitInputs::new(root.path()).unwrap();
    let hook_root = root.path().to_owned();
    DIRECTORY_HOOK.with(|hook| {
        *hook.borrow_mut() = Some((
            "scope/d".into(),
            Box::new(move || {
                fs::write(hook_root.join("scope/z"), "replacement").unwrap();
                git(&hook_root, &["add", "scope/z"]).unwrap();
                fs::write(hook_root.join("scope/z"), "initial").unwrap();
            }),
        ));
    });
    assert_eq!(
        view.files_under("scope"),
        Err("bound input changed during freeze: scope/z".into())
    );
    DIRECTORY_HOOK.with(|hook| assert!(hook.borrow().is_none()));
}

#[test]
fn traversal_detects_index_changes_during_the_final_directory_pass() {
    let root = fixture();
    for dir in ["scope/a", "scope/z"] {
        fs::create_dir_all(root.path().join(dir)).unwrap();
    }
    let mut view = GitInputs::new(root.path()).unwrap();
    let hook_root = root.path().to_owned();
    DIRECTORY_HOOK.with(|hook| {
        *hook.borrow_mut() = Some((
            "scope/a".into(),
            Box::new(move || {
                DIRECTORY_HOOK.with(|hook| {
                    *hook.borrow_mut() = Some((
                        "scope/a".into(),
                        Box::new(move || {
                            stage_and_remove(&hook_root, "scope/z/new");
                        }),
                    ));
                });
            }),
        ));
    });
    assert_eq!(
        view.files_under("scope"),
        Err("acquisition directory changed during freeze: scope/z".into())
    );
    DIRECTORY_HOOK.with(|hook| assert!(hook.borrow().is_none()));
}
