use super::*;
use std::io::Write;
use std::process::Stdio;

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
