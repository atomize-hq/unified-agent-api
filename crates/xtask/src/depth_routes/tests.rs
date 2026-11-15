use super::*;
use std::fs;

#[cfg(unix)]
#[test]
fn checked_delivery_refuses_final_and_directory_alias_retargeting() {
    use std::os::unix::fs::symlink;
    for replacement in ["final", "directory_alias", "parent_identity"] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir(root.join("ordinary")).unwrap();
        fs::create_dir_all(root.join("protected/reports/0.2.0")).unwrap();
        let protected = root.join("protected/reports/0.2.0/depth-record.json");
        fs::write(&protected, b"protected record\n").unwrap();
        let output = if replacement == "directory_alias" {
            symlink("ordinary", root.join("alias")).unwrap();
            root.join("alias/depth-record.json")
        } else {
            root.join("ordinary/matrix.md")
        };
        if replacement == "final" {
            fs::write(&output, b"ordinary before replacement\n").unwrap();
        }
        let retargeted = output.clone();
        let changed_root = root.to_owned();
        let target = protected.clone();
        destination::BEFORE_WRITE.with(|hook| {
            *hook.borrow_mut() = Some(Box::new(move || match replacement {
                "final" => {
                    fs::remove_file(&retargeted).unwrap();
                    symlink(&target, &retargeted).unwrap();
                }
                "directory_alias" => {
                    fs::remove_file(changed_root.join("alias")).unwrap();
                    symlink("protected/reports/0.2.0", changed_root.join("alias")).unwrap();
                }
                _ => {
                    fs::rename(changed_root.join("ordinary"), changed_root.join("old")).unwrap();
                    fs::create_dir(changed_root.join("ordinary")).unwrap();
                }
            }));
        });
        let error =
            write_file_destination(&output, b"matrix bytes\n", "capability-matrix").unwrap_err();
        assert!(
            error.contains("output destination changed after validation"),
            "{replacement}: {error}"
        );
        assert_eq!(fs::read(protected).unwrap(), b"protected record\n");
        if replacement == "final" {
            assert!(fs::symlink_metadata(&output)
                .unwrap()
                .file_type()
                .is_symlink());
        } else {
            assert!(!root.join("ordinary/matrix.md").exists());
        }
    }
}

#[cfg(unix)]
#[test]
fn resolution_refuses_loops_and_io_errors_and_keeps_ordinary_delivery_bytes() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    symlink("loop", root.join("loop")).unwrap();
    assert_eq!(
        refuse_file_destination(&root.join("loop"), b"bytes", "test").unwrap_err(),
        "too many output symlinks"
    );
    fs::write(root.join("file"), b"sentinel").unwrap();
    assert!(refuse_file_destination(&root.join("file/child"), b"bytes", "test").is_err());
    for suffix in ["file/", "file/."] {
        assert!(write_file_destination(&root.join(suffix), b"wrong bytes", "test").is_err());
        assert_eq!(fs::read(root.join("file")).unwrap(), b"sentinel");
    }
    symlink("file/", root.join("directory-alias")).unwrap();
    assert!(write_file_destination(&root.join("directory-alias"), b"wrong bytes", "test").is_err());
    assert_eq!(fs::read(root.join("file")).unwrap(), b"sentinel");
    fs::create_dir(root.join("new")).unwrap();
    symlink("new/target", root.join("alias")).unwrap();
    write_file_destination(&root.join("alias"), b"literal ordinary bytes\n", "test").unwrap();
    assert_eq!(
        fs::read(root.join("new/target")).unwrap(),
        b"literal ordinary bytes\n"
    );
    assert!(fs::symlink_metadata(root.join("alias"))
        .unwrap()
        .file_type()
        .is_symlink());
}

#[cfg(unix)]
#[test]
fn checked_delivery_opens_without_following_final_symlinks() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("protected/reports/0.2.0")).unwrap();
    let protected = root.join("protected/reports/0.2.0/depth-record.json");
    fs::write(&protected, b"protected bytes\n").unwrap();
    let output = root.join("matrix.md");
    let changed_output = output.clone();
    let target = protected.clone();
    destination::BEFORE_OPEN.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move || {
            std::os::unix::fs::symlink(target, changed_output).unwrap();
        }));
    });
    let error = write_file_destination(&output, b"matrix\n", "capability-matrix").unwrap_err();
    assert!(error.contains("open checked output:"), "{error}");
    assert_eq!(fs::read(protected).unwrap(), b"protected bytes\n");
}

#[cfg(unix)]
#[test]
fn checked_delivery_rechecks_parent_identity_after_opening() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir(root.join("ordinary")).unwrap();
    fs::write(root.join("ordinary/matrix.md"), b"original bytes\n").unwrap();
    let changed_root = root.to_owned();
    destination::AFTER_OPEN.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move || {
            fs::rename(changed_root.join("ordinary"), changed_root.join("moved")).unwrap();
            fs::create_dir(changed_root.join("ordinary")).unwrap();
            fs::write(
                changed_root.join("ordinary/matrix.md"),
                b"replacement bytes\n",
            )
            .unwrap();
        }));
    });
    let error = write_file_destination(
        &root.join("ordinary/matrix.md"),
        b"matrix\n",
        "capability-matrix",
    )
    .unwrap_err();
    assert_eq!(error, "output parent changed after opening");
    assert_eq!(
        fs::read(root.join("moved/matrix.md")).unwrap(),
        b"original bytes\n"
    );
    assert_eq!(
        fs::read(root.join("ordinary/matrix.md")).unwrap(),
        b"replacement bytes\n"
    );
}
