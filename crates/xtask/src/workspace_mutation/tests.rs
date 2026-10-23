use super::*;

#[test]
fn resolve_rejects_parent_dir_escape() {
    let root = temp_root();
    let jail = WorkspacePathJail::new(&root).expect("build jail");
    let err = jail
        .resolve(Path::new("../escape.txt"))
        .expect_err("parent-dir escape should fail");
    assert!(err.to_string().contains("must stay repo-relative"));
}

#[cfg(unix)]
#[test]
fn resolve_rejects_symlink_escape() {
    use std::os::unix::fs::symlink;

    let root = temp_root();
    let outside = temp_root();
    symlink(&outside, root.join("linked")).expect("create symlink");

    let jail = WorkspacePathJail::new(&root).expect("build jail");
    let err = jail
        .resolve(Path::new("linked/file.txt"))
        .expect_err("symlink escape should fail");
    assert!(err.to_string().contains("symlinked component"));
}

#[test]
fn rollback_preserves_intervening_file_edits() {
    for replacing in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("output");
        let temp = root.path().join("proposed");
        let backup = root.path().join("backup");
        fs::write(&temp, b"planned").unwrap();
        let mutation = if replacing {
            fs::write(&path, b"before").unwrap();
            PlannedMutation::replace("output", b"before".to_vec(), b"planned".to_vec())
        } else {
            PlannedMutation::create("output", b"planned".to_vec())
        };
        let applied = apply_one(&path, &mutation, &temp, &backup).unwrap();
        fs::write(&path, b"intervening edit").unwrap();
        assert!(rollback(
            &WorkspacePathJail::new(root.path()).unwrap(),
            &[applied],
            &[]
        )
        .is_err());
        assert_eq!(fs::read(&path).unwrap(), b"intervening edit");
    }
}

#[test]
fn injected_write_failure_restores_owned_bytes_but_preserves_observed_conflicts() {
    for replacing in [false, true] {
        for conflict in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let path = root.path().join("first");
            let first = if replacing {
                fs::write(&path, b"before").unwrap();
                PlannedMutation::replace("first", b"before".to_vec(), b"planned".to_vec())
            } else {
                PlannedMutation::create("first", b"planned".to_vec())
            };
            let mutations = [first, PlannedMutation::create("second", b"second".to_vec())];
            let error = apply_with_hook(root.path(), &mutations, |index| {
                if index == 1 {
                    if conflict {
                        fs::write(&path, b"other writer").unwrap();
                    }
                    return Err(WorkspaceMutationError::Internal(
                        "injected write failure".into(),
                    ));
                }
                Ok(())
            })
            .unwrap_err();
            assert!(!root.path().join("second").exists());
            if conflict {
                assert!(error.to_string().contains("rollback conflict"));
                assert_eq!(fs::read(&path).unwrap(), b"other writer");
            } else if replacing {
                assert_eq!(fs::read(&path).unwrap(), b"before");
            } else {
                assert!(!path.exists());
            }
        }
    }
}

#[cfg(unix)]
#[test]
fn rollback_refuses_an_intervening_symlink() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("output");
    let temp = root.path().join("proposed");
    fs::write(&temp, b"planned").unwrap();
    let mutation = PlannedMutation::create("output", b"planned".to_vec());
    let applied = apply_one(&path, &mutation, &temp, &root.path().join("backup")).unwrap();
    fs::remove_file(&path).unwrap();
    fs::write(root.path().join("other"), b"other writer").unwrap();
    symlink("other", &path).unwrap();
    assert!(rollback(
        &WorkspacePathJail::new(root.path()).unwrap(),
        &[applied],
        &[]
    )
    .is_err());
    assert!(fs::symlink_metadata(&path)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(
        fs::read(root.path().join("other")).unwrap(),
        b"other writer"
    );
}

fn temp_root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "xtask-workspace-mutation-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(&root).expect("create temp root");
    root
}

#[test]
fn rollback_continues_past_conflict_to_restore_other_owned_files() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("first"), b"before").unwrap();
    let mutations = [
        PlannedMutation::replace("first", b"before".to_vec(), b"after".to_vec()),
        PlannedMutation::create("second", b"second".to_vec()),
        PlannedMutation::create("third", b"third".to_vec()),
    ];
    let error = apply_with_hook(root.path(), &mutations, |index| {
        if index == 2 {
            fs::write(root.path().join("second"), b"concurrent").unwrap();
            return Err(WorkspaceMutationError::Internal("injected".into()));
        }
        Ok(())
    })
    .unwrap_err();
    assert!(error.to_string().contains("rollback conflict"));
    assert_eq!(fs::read(root.path().join("first")).unwrap(), b"before");
    assert_eq!(fs::read(root.path().join("second")).unwrap(), b"concurrent");
    assert!(!root.path().join("third").exists());
}
#[cfg(unix)]
#[test]
fn replacement_preserves_mode_and_rollback_preserves_concurrent_chmod() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("first");
    fs::write(&path, b"before").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    apply_mutations(
        root.path(),
        &[PlannedMutation::replace(
            "first",
            b"before".to_vec(),
            b"after".to_vec(),
        )],
    )
    .unwrap();
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o755
    );
    let mutations = [
        PlannedMutation::replace("first", b"after".to_vec(), b"next".to_vec()),
        PlannedMutation::create("second", vec![]),
    ];
    assert!(apply_with_hook(root.path(), &mutations, |index| {
        if index == 1 {
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
            return Err(WorkspaceMutationError::Internal("injected".into()));
        }
        Ok(())
    })
    .is_err());
    assert_eq!(fs::read(&path).unwrap(), b"next");
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o700
    );
}

#[cfg(unix)]
#[test]
fn mode_precondition_preserves_a_chmod_seen_before_apply() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("output");
    fs::write(&path, b"before").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    let mutation = PlannedMutation::replace("output", b"before".to_vec(), b"after".to_vec())
        .with_expected_mode(0o644);
    let result = apply_with_hook(root.path(), &[mutation], |_| {
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(fs::read(&path).unwrap(), b"before");
    assert_eq!(file_mode(&fs::metadata(&path).unwrap()), 0o600);
}
