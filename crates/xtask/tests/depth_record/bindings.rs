use super::*;
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use xtask::depth_record::bound_evidence;

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn fixture() -> (tempfile::TempDir, DepthRecord) {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    fs::create_dir(root.path().join("proof")).unwrap();
    fs::write(root.path().join("proof/result.json"), "original").unwrap();
    git(root.path(), &["add", "proof/result.json"]);
    let mut record = parse(&sealed(record())).unwrap();
    for result in &mut record.execution.as_mut().unwrap().results {
        for file in &mut result.evidence {
            file.path = "proof/result.json".into();
        }
    }
    (root, record)
}

#[test]
fn bound_evidence_refuses_dirty_bytes_and_unmerged_index() {
    let (root, record) = fixture();
    fs::write(root.path().join("proof/result.json"), "unstaged success").unwrap();
    assert!(bound_evidence(root.path(), [&record])
        .unwrap_err()
        .contains("disagree"));
    let object = git(root.path(), &["rev-parse", ":proof/result.json"]);
    let mut child = Command::new("git")
        .arg("-C")
        .arg(root.path())
        .args(["update-index", "--index-info"])
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            format!(
                "0 {}\tproof/result.json\n100644 {} 2\tproof/result.json\n",
                "0".repeat(40),
                object.trim()
            )
            .as_bytes(),
        )
        .unwrap();
    assert!(child.wait().unwrap().success());
    assert!(bound_evidence(root.path(), [&record])
        .unwrap_err()
        .contains("stage-0"));
}

#[test]
fn bound_evidence_accepts_staged_new_bytes_without_requiring_a_commit() {
    use sha2::{Digest, Sha256};
    let (root, record) = fixture();
    fs::write(root.path().join("proof/result.json"), "staged result").unwrap();
    git(root.path(), &["add", "proof/result.json"]);
    let actual = bound_evidence(root.path(), [&record]).unwrap();
    assert_eq!(
        actual["proof/result.json"],
        hex::encode(Sha256::digest(b"staged result"))
    );
    assert_eq!(actual.len(), 1);
}

#[test]
fn bound_evidence_refuses_a_symlinked_ancestor_even_when_bytes_match() {
    use std::os::unix::fs::symlink;
    let (root, record) = fixture();
    fs::rename(root.path().join("proof"), root.path().join("elsewhere")).unwrap();
    symlink("elsewhere", root.path().join("proof")).unwrap();
    assert!(bound_evidence(root.path(), [&record])
        .unwrap_err()
        .contains("symlink"));
}

#[test]
fn bound_evidence_distinguishes_absence_from_missing_or_untracked_working_files() {
    let (root, record) = fixture();
    fs::remove_file(root.path().join("proof/result.json")).unwrap();
    assert!(bound_evidence(root.path(), [&record]).is_err());
    git(root.path(), &["rm", "--cached", "proof/result.json"]);
    assert!(bound_evidence(root.path(), [&record]).unwrap().is_empty());
    fs::write(root.path().join("proof/result.json"), "untracked").unwrap();
    assert!(bound_evidence(root.path(), [&record])
        .unwrap_err()
        .contains("untracked"));
}

#[test]
fn bound_evidence_uses_literal_paths_and_ignores_unrelated_dirty_files() {
    let (root, mut record) = fixture();
    let path = "proof/[result]*.json";
    fs::rename(
        root.path().join("proof/result.json"),
        root.path().join(path),
    )
    .unwrap();
    git(root.path(), &["--literal-pathspecs", "add", "--", path]);
    fs::write(root.path().join("unrelated"), "local edits").unwrap();
    for result in &mut record.execution.as_mut().unwrap().results {
        for file in &mut result.evidence {
            file.path = path.into();
        }
    }
    let actual = bound_evidence(root.path(), [&record]).unwrap();
    assert_eq!(actual.len(), 1);
    assert!(actual.contains_key(path));
}
