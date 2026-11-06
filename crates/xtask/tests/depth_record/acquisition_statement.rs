//! The OF-P4(B) acquisition statement and its reader, which reads it from a named integration ref
//! only (child module of `depth_record.rs`).

use std::{fs, path::Path, process::Command};

use xtask::depth_record::{
    acquisition::{read_from_ref, statement_path, AcquisitionStatement},
    FileRef,
};

use super::*;

const PACK: &str = "depth-fixture-onboarding";
const AGENT: &str = "depth_fixture";
const INPUT: &str = "cli_manifests/depth_fixture/snapshots/0.2.0/union.json";

fn statement(version: &str, agent: &str) -> String {
    format!(
        "schema_version = 1\nagent_id = \"{agent}\"\nversion = \"{version}\"\n\n[[inputs]]\n\
         path = \"{INPUT}\"\nsha256 = \"{SHA}\"\n\
         command = \"cargo run -p xtask -- manifest-union --root cli_manifests/depth_fixture\"\n"
    )
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// A repository whose `integration` branch holds the statement for 0.2.0.
fn repository() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q", "-b", "integration"]);
    write(
        root.path(),
        &statement_path(PACK, "0.2.0"),
        &statement("0.2.0", AGENT),
    );
    git(root.path(), &["add", "-A"]);
    git(root.path(), &["commit", "-q", "-m", "statement"]);
    root
}

fn read(root: &Path, git_ref: &str, version: &str) -> Result<Option<AcquisitionStatement>, String> {
    read_from_ref(root, git_ref, PACK, AGENT, version)
}

#[test]
fn a_statement_states_the_agent_version_and_each_input() {
    let parsed = AcquisitionStatement::parse(&statement("0.2.0", AGENT), AGENT, "0.2.0.toml")
        .expect("parse statement");
    assert_eq!(parsed.inputs.len(), 1);
    assert_eq!(
        statement_path(PACK, "0.2.0"),
        "docs/agents/lifecycle/depth-fixture-onboarding/governance/depth-acquisition/0.2.0.toml"
    );

    let base = statement("0.2.0", AGENT);
    let cases: &[(String, &str, &str)] = &[
        (
            format!("{base}declared_by = \"x\"\n"),
            "0.2.0.toml",
            "unknown field",
        ),
        (
            base.replace("[[inputs]]", "[[inputs]]\nextra = 1"),
            "0.2.0.toml",
            "unknown field",
        ),
        (
            base.replace("schema_version = 1", "schema_version = 2"),
            "0.2.0.toml",
            "schema_version",
        ),
        (statement("0.2.0", "other"), "0.2.0.toml", "agent_id"),
        (base.clone(), "0.2.1.toml", "file name"),
        (base.clone(), "0.2.0.json", "named"),
        (base.replace(SHA, "abc"), "0.2.0.toml", "SHA-256"),
        (
            base.replace(INPUT, "../outside.json"),
            "0.2.0.toml",
            "repository-relative",
        ),
        (
            base.replace(
                "cargo run -p xtask -- manifest-union --root cli_manifests/depth_fixture",
                " ",
            ),
            "0.2.0.toml",
            "producing command",
        ),
        (
            format!(
                "{base}\n[[inputs]]\npath = \"{INPUT}\"\nsha256 = \"{SHA}\"\ncommand = \"c\"\n"
            ),
            "0.2.0.toml",
            "twice",
        ),
        (
            "schema_version = 1\nagent_id = \"depth_fixture\"\nversion = \"0.2.0\"\ninputs = []\n"
                .to_string(),
            "0.2.0.toml",
            "no acquired input",
        ),
    ];
    for (text, name, needle) in cases {
        let err = AcquisitionStatement::parse(text, AGENT, name).expect_err(needle);
        assert!(err.contains(needle), "{needle}: {err}");
    }
}

#[test]
fn bound_inputs_must_equal_the_statement_exactly() {
    let parsed =
        AcquisitionStatement::parse(&statement("0.2.0", AGENT), AGENT, "0.2.0.toml").unwrap();
    let file = |path: &str, sha256: &str| FileRef {
        path: path.to_string(),
        sha256: sha256.to_string(),
    };
    assert_eq!(parsed.mismatch(&[file(INPUT, SHA)]), None);
    let other = "1111111111111111111111111111111111111111111111111111111111111111";
    // Differing bytes, an input the statement does not cover, and an entry with no input.
    for bound in [
        vec![file(INPUT, other)],
        vec![file(INPUT, SHA), file("x.json", SHA)],
        vec![],
    ] {
        let why = parsed.mismatch(&bound).expect("mismatch");
        assert!(why.contains("the acquisition statement"), "{why}");
    }
}

#[test]
fn the_reader_reads_the_named_ref_and_never_the_working_tree() {
    let root = repository();
    let found = read(root.path(), "integration", "0.2.0")
        .unwrap()
        .expect("statement");
    assert_eq!(found.version, "0.2.0");
    // No statement for another version is an absence, not an error.
    assert_eq!(read(root.path(), "integration", "0.3.0").unwrap(), None);

    // A statement carried on the onboarding branch does not count before it merges.
    git(root.path(), &["checkout", "-q", "-b", "onboarding"]);
    write(
        root.path(),
        &statement_path(PACK, "0.3.0"),
        &statement("0.3.0", AGENT),
    );
    assert_eq!(read(root.path(), "integration", "0.3.0").unwrap(), None);
    git(root.path(), &["add", "-A"]);
    git(root.path(), &["commit", "-q", "-m", "onboarding statement"]);
    assert_eq!(read(root.path(), "integration", "0.3.0").unwrap(), None);
    assert!(read(root.path(), "onboarding", "0.3.0").unwrap().is_some());

    // A working-tree edit of the statement at the ref changes nothing that is read.
    write(root.path(), &statement_path(PACK, "0.2.0"), "not toml");
    assert!(read(root.path(), "integration", "0.2.0").unwrap().is_some());
}

#[test]
fn the_reader_refuses_a_directory_it_cannot_read_in_full() {
    let root = repository();
    let refuse = |root: &Path, needle: &str| {
        let err = read(root, "integration", "0.2.0").expect_err(needle);
        assert!(err.contains(needle), "{needle}: {err}");
    };
    // An unreadable ref is an error, never an absence.
    let err = read(root.path(), "missing", "0.2.0").unwrap_err();
    assert!(err.contains("git ls-tree"), "{err}");
    assert!(read(root.path(), "--all", "0.2.0").is_err());
    assert!(read(root.path(), "integration", "../0.2.0").is_err());

    let dir = statement_path(PACK, "x").replace("/x.toml", "");
    for (name, text, needle) in [
        ("v0.2.1.toml", statement("0.2.1", AGENT), "file name"),
        ("0.2.1.toml", statement("0.2.1", "other"), "agent_id"),
        (
            "nested/0.2.1.toml",
            statement("0.2.1", AGENT),
            "regular statement file",
        ),
    ] {
        let root = repository();
        write(root.path(), &format!("{dir}/{name}"), &text);
        git(root.path(), &["add", "-A"]);
        git(root.path(), &["commit", "-q", "-m", "sibling"]);
        refuse(root.path(), needle);
    }

    #[cfg(unix)]
    {
        let root = repository();
        std::os::unix::fs::symlink("0.2.0.toml", root.path().join(format!("{dir}/0.2.1.toml")))
            .unwrap();
        git(root.path(), &["add", "-A"]);
        git(root.path(), &["commit", "-q", "-m", "link"]);
        refuse(root.path(), "regular statement file");
    }
}
