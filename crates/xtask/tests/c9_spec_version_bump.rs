use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn make_temp_dir(prefix: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after unix epoch");
    let unique = format!("{}-{}-{}", prefix, std::process::id(), now.as_nanos());
    let dir = std::env::temp_dir().join(unique);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn write_text(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dir");
    }
    fs::write(path, contents).expect("write fixture file");
}

fn fixture_workspace(root: &Path) {
    write_text(&root.join("VERSION"), "0.2.0\n");
    write_text(
        &root.join("CHANGELOG.md"),
        r#"# Changelog

## [Unreleased]

### Added

- Pending release note.

## [0.2.0] - 2026-04-14

### Added

- Previous release note.
"#,
    );
    write_text(
        &root.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/leaf", "crates/root", "crates/nonpub"]
resolver = "2"

[workspace.package]
version = "0.2.0"
edition = "2021"
rust-version = "1.78"
"#,
    );
    write_text(
        &root.join("crates/leaf/Cargo.toml"),
        r#"[package]
name = "leaf"
version.workspace = true
edition = "2021"

[lib]
path = "src/lib.rs"
"#,
    );
    write_text(&root.join("crates/leaf/src/lib.rs"), "pub fn leaf() {}\n");

    write_text(
        &root.join("crates/root/Cargo.toml"),
        r#"[package]
name = "root"
version = "0.2.0"
edition = "2021"

[dependencies]
leaf = { path = "../leaf", version = "=0.2.0" }

[lib]
path = "src/lib.rs"
"#,
    );
    write_text(&root.join("crates/root/src/lib.rs"), "pub fn root() {}\n");

    write_text(
        &root.join("crates/nonpub/Cargo.toml"),
        r#"[package]
name = "nonpub"
version.workspace = true
edition = "2021"
publish = false

[dependencies]
leaf = { path = "../leaf", version = "=0.2.0" }
root = { path = "../root", version = "=0.2.0" }

[lib]
path = "src/lib.rs"
"#,
    );
    write_text(
        &root.join("crates/nonpub/src/lib.rs"),
        "pub fn nonpub() {}\n",
    );
}

#[test]
fn c9_spec_version_bump_updates_release_surfaces_in_one_pass() {
    let xtask_bin = PathBuf::from(env!("CARGO_BIN_EXE_xtask"));
    let root = make_temp_dir("version-bump");
    fixture_workspace(&root);

    let output = Command::new(&xtask_bin)
        .arg("version-bump")
        .arg("0.3.1")
        .arg("--root")
        .arg(&root)
        .output()
        .expect("spawn xtask version-bump");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "version-bump failed\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );

    let version = fs::read_to_string(root.join("VERSION")).expect("read VERSION");
    assert_eq!(version, "0.3.1\n");

    let root_manifest = fs::read_to_string(root.join("Cargo.toml")).expect("read root Cargo");
    assert!(root_manifest.contains("version = \"0.3.1\""));

    let changelog = fs::read_to_string(root.join("CHANGELOG.md")).expect("read changelog");
    assert!(changelog.contains("## [Unreleased]\n\n## [0.3.1] - "));
    assert!(changelog.contains("- Pending release note."));
    assert!(changelog.contains("## [0.2.0] - 2026-04-14"));

    let root_crate_manifest =
        fs::read_to_string(root.join("crates/root/Cargo.toml")).expect("read root crate");
    assert!(root_crate_manifest.contains("version = \"0.3.1\""));
    assert!(root_crate_manifest.contains("version = \"=0.3.1\""));

    let nonpub_manifest =
        fs::read_to_string(root.join("crates/nonpub/Cargo.toml")).expect("read nonpub crate");
    assert!(nonpub_manifest.contains("version = \"=0.3.1\""));
}

#[test]
fn c9_spec_version_bump_rejects_invalid_semver() {
    let xtask_bin = PathBuf::from(env!("CARGO_BIN_EXE_xtask"));
    let root = make_temp_dir("version-bump-invalid");
    fixture_workspace(&root);

    let output = Command::new(&xtask_bin)
        .arg("version-bump")
        .arg("not-a-version")
        .arg("--root")
        .arg(&root)
        .output()
        .expect("spawn xtask version-bump");

    assert!(!output.status.success(), "invalid version should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid semver version"),
        "unexpected stderr:\n{}",
        stderr
    );
}
