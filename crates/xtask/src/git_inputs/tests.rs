use super::*;
use std::io::Write;
use std::process::Stdio;

mod batch;
mod immutable;

thread_local! {
    pub(crate) static GIT_COMMANDS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]).unwrap();
    fs::write(root.path().join("bound"), "initial").unwrap();
    git(root.path(), &["add", "bound"]).unwrap();
    root
}

#[test]
fn referenced_input_changes_and_new_artifacts_are_detected() {
    let root = fixture();
    let mut view = GitInputs::new(root.path()).unwrap();
    view.required("bound").unwrap();
    fs::write(root.path().join("bound"), "replacement").unwrap();
    git(root.path(), &["add", "bound"]).unwrap();
    assert!(view.verify().unwrap_err().contains("changed"));
    let mut view = GitInputs::new(root.path()).unwrap();
    assert!(view.list("acquired").unwrap().is_empty());
    fs::create_dir(root.path().join("acquired")).unwrap();
    fs::write(root.path().join("acquired/new.json"), "{}").unwrap();
    assert!(view.verify().unwrap_err().contains("changed"));
}

#[test]
fn unmerged_index_and_missing_working_file_refuse() {
    let root = fixture();
    let object = String::from_utf8(git(root.path(), &["rev-parse", ":bound"]).unwrap())
        .unwrap()
        .trim()
        .to_owned();
    let mut command = Command::new("git")
        .arg("-C")
        .arg(root.path())
        .args(["update-index", "--index-info"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    command.stdin.take().unwrap().write_all(format!("0 {}\tbound\n100644 {object} 1\tbound\n100644 {object} 2\tbound\n100644 {object} 3\tbound\n","0".repeat(40)).as_bytes()).unwrap();
    assert!(command.wait().unwrap().success());
    let mut view = GitInputs::new(root.path()).unwrap();
    assert!(view.required("bound").unwrap_err().contains("unmerged"));
    let root = fixture();
    fs::remove_file(root.path().join("bound")).unwrap();
    let mut view = GitInputs::new(root.path()).unwrap();
    assert!(view.required("bound").is_err());
}

#[cfg(unix)]
#[test]
fn executable_mode_changes_are_part_of_the_coherent_recheck() {
    use std::os::unix::fs::PermissionsExt;
    let root = fixture();
    let mut view = GitInputs::new(root.path()).unwrap();
    view.required("bound").unwrap();
    fs::set_permissions(root.path().join("bound"), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(view.verify().unwrap_err().contains("executable mode"));
}

// Retain the original traversal as an independent oracle for results, error order and the
// partial freeze state. Batch failures must be consumed at the original optional-read boundary.
fn single_files_under(view: &mut GitInputs, path: &str) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    for name in view.list(path)? {
        let child = format!("{path}/{name}");
        let full = safe_path(&view.root, &child)?;
        if fs::symlink_metadata(full).is_ok_and(|m| m.is_dir()) {
            result.extend(single_files_under(view, &child)?);
        } else {
            view.required(&child)?;
            result.push(child);
        }
    }
    result.sort();
    Ok(result)
}

fn equivalent_traversal(view: &GitInputs) -> GitInputs {
    let mut single = view.clone();
    let mut batched = view.clone();
    assert_eq!(
        batched.files_under("scope"),
        single_files_under(&mut single, "scope")
    );
    assert!(batched.files == single.files);
    assert_eq!(batched.directories, single.directories);
    batched
}

#[test]
fn files_under_preserves_freeze_recording_proposals_and_error_order() {
    let root = fixture();
    fs::create_dir_all(root.path().join("scope/subdir")).unwrap();
    for path in ["scope/a", "scope/subdir/b", "scope/z"] {
        fs::write(root.path().join(path), path).unwrap();
    }
    git(root.path(), &["add", "."]).unwrap();
    let view = GitInputs::new(root.path()).unwrap();
    let frozen = equivalent_traversal(&view);
    GIT_COMMANDS.with(|count| count.set(0));
    frozen.verify().unwrap();
    assert_eq!(GIT_COMMANDS.with(|count| count.get()), 3); // Two file reads, one directory listing.
    let mut proposed = view.clone();
    proposed.proposed = Some(
        crate::runtime_advertising::source::Source::captured(
            root.path(),
            ["scope/a", "scope/subdir/b", "scope/z"]
                .into_iter()
                .map(|p| (p.into(), (p.as_bytes().to_vec(), false)))
                .collect(),
        )
        .unwrap(),
    );
    let frozen_proposal = equivalent_traversal(&proposed);
    fs::write(root.path().join("scope/a"), "changed").unwrap();
    git(root.path(), &["add", "scope/a"]).unwrap();
    fs::write(root.path().join("scope/z"), "unstaged").unwrap();
    assert_eq!(
        frozen.verify(),
        Err("bound input changed during freeze: scope/a".into())
    );
    let changed = equivalent_traversal(&frozen);
    assert!(changed.files == frozen.files);
    equivalent_traversal(&proposed); // Frozen-source comparison precedes later unstaged bytes.
    let mut cached = equivalent_traversal(&frozen_proposal);
    assert!(cached.files_under("scope").is_ok()); // Proposed optional reads reuse observations.
    assert!(cached.verify().is_err());
    fs::remove_file(root.path().join("scope/z")).unwrap();
    fs::create_dir(root.path().join("scope/z")).unwrap();
    equivalent_traversal(&view);
}

#[test]
fn files_under_batches_direct_files_and_verify_batches_all_files() {
    let root = fixture();
    fs::create_dir(root.path().join("scope")).unwrap();
    for i in 0..64 {
        fs::write(root.path().join(format!("scope/{i}")), "file").unwrap();
    }
    git(root.path(), &["add", "."]).unwrap();
    let mut view = GitInputs::new(root.path()).unwrap();
    GIT_COMMANDS.with(|count| count.set(0));
    assert_eq!(view.files_under("scope").unwrap().len(), 64);
    assert_eq!(GIT_COMMANDS.with(|count| count.get()), 3);
    GIT_COMMANDS.with(|count| count.set(0));
    view.verify().unwrap();
    assert_eq!(GIT_COMMANDS.with(|count| count.get()), 3);
    fs::write(root.path().join("scope/extra"), "untracked").unwrap();
    assert_eq!(
        view.verify(),
        Err("acquisition directory changed during freeze: scope".into())
    );
    equivalent_traversal(&GitInputs::new(root.path()).unwrap());
}

#[cfg(unix)]
#[test]
fn files_under_preserves_path_error_before_an_invalid_borrowed_binding() {
    use std::os::unix::fs::symlink;
    let root = fixture();
    fs::create_dir(root.path().join("scope")).unwrap();
    fs::write(root.path().join("scope/a"), "a").unwrap();
    fs::write(root.path().join("scope/z"), "z").unwrap();
    git(root.path(), &["add", "."]).unwrap();
    let borrowed = batch::commit(root.path());
    let mut view = GitInputs::new(root.path()).unwrap();
    view.borrowed = Some(borrowed); // Deliberately lacks this thread's scratch binding.
    fs::remove_file(root.path().join("scope/a")).unwrap();
    symlink("z", root.path().join("scope/a")).unwrap();
    equivalent_traversal(&view);
}
