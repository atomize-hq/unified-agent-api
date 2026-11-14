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
