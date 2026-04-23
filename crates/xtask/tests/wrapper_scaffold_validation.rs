use std::{
    fs,
    path::{Path, PathBuf},
};

#[path = "support/onboard_agent_harness.rs"]
mod harness;

use harness::{fixture_root, run_xtask, snapshot_files, wrapper_scaffold_args, write_text};

fn scaffold_fixture(prefix: &str) -> PathBuf {
    let fixture = fixture_root(prefix);
    fs::remove_dir_all(fixture.join("crates/gemini_cli")).expect("remove seeded gemini crate");
    fixture
}

fn rewrite_registry<F>(fixture: &Path, edit: F)
where
    F: FnOnce(String) -> String,
{
    let registry_path = fixture.join("crates/xtask/data/agent_registry.toml");
    let registry = fs::read_to_string(&registry_path).expect("read registry");
    write_text(&registry_path, &edit(registry));
}

#[test]
fn scaffold_wrapper_crate_rejects_unknown_agent() {
    let fixture = fixture_root("wrapper-scaffold-unknown-agent");
    let before = snapshot_files(&fixture);

    let output = run_xtask(&fixture, wrapper_scaffold_args("--dry-run", "cursor"));
    let after = snapshot_files(&fixture);

    assert_eq!(output.exit_code, 2, "stderr:\n{}", output.stderr);
    assert!(
        output.stderr.contains("cursor"),
        "stderr did not mention unknown agent:\n{}",
        output.stderr
    );
    assert_eq!(before, after, "unknown-agent failure must not write files");
}

#[test]
fn scaffold_wrapper_crate_rejects_missing_crate_path() {
    let fixture = scaffold_fixture("wrapper-scaffold-missing-crate-path");
    rewrite_registry(&fixture, |registry| {
        registry.replacen("crate_path = \"crates/gemini_cli\"\n", "", 1)
    });
    let before = snapshot_files(&fixture);

    let output = run_xtask(&fixture, wrapper_scaffold_args("--dry-run", "gemini_cli"));
    let after = snapshot_files(&fixture);

    assert_eq!(output.exit_code, 2, "stderr:\n{}", output.stderr);
    assert!(
        output.stderr.contains("crate_path"),
        "stderr did not mention crate_path:\n{}",
        output.stderr
    );
    assert_eq!(before, after, "missing crate_path must not write files");
}

#[test]
fn scaffold_wrapper_crate_rejects_invalid_crate_path() {
    let fixture = scaffold_fixture("wrapper-scaffold-invalid-crate-path");
    rewrite_registry(&fixture, |registry| {
        registry.replacen(
            "crate_path = \"crates/gemini_cli\"\n",
            "crate_path = \"\"\n",
            1,
        )
    });
    let before = snapshot_files(&fixture);

    let output = run_xtask(&fixture, wrapper_scaffold_args("--dry-run", "gemini_cli"));
    let after = snapshot_files(&fixture);

    assert_eq!(output.exit_code, 2, "stderr:\n{}", output.stderr);
    assert!(
        output.stderr.contains("crate_path"),
        "stderr did not mention invalid crate_path:\n{}",
        output.stderr
    );
    assert_eq!(before, after, "invalid crate_path must not write files");
}

#[test]
fn scaffold_wrapper_crate_rejects_path_escape() {
    let fixture = scaffold_fixture("wrapper-scaffold-path-escape");
    rewrite_registry(&fixture, |registry| {
        registry.replacen(
            "crate_path = \"crates/gemini_cli\"\n",
            "crate_path = \"../escape\"\n",
            1,
        )
    });
    let before = snapshot_files(&fixture);

    let output = run_xtask(&fixture, wrapper_scaffold_args("--write", "gemini_cli"));
    let after = snapshot_files(&fixture);

    assert_eq!(output.exit_code, 2, "stderr:\n{}", output.stderr);
    assert!(
        output.stderr.contains("crate_path") || output.stderr.contains(".."),
        "stderr did not mention path escape:\n{}",
        output.stderr
    );
    assert_eq!(before, after, "path escape must not write files");
}

#[cfg(unix)]
#[test]
fn scaffold_wrapper_crate_rejects_symlinked_crate_path() {
    use std::os::unix::fs::symlink;

    let fixture = scaffold_fixture("wrapper-scaffold-symlink");
    let outside = fixture_root("wrapper-scaffold-symlink-outside");
    let outside_before = snapshot_files(&outside);
    fs::create_dir_all(fixture.join("crates")).expect("create crates dir");
    symlink(&outside, fixture.join("crates/gemini_cli")).expect("create symlink");
    let before = snapshot_files(&fixture);

    let output = run_xtask(&fixture, wrapper_scaffold_args("--write", "gemini_cli"));
    let after = snapshot_files(&fixture);
    let outside_after = snapshot_files(&outside);

    assert_eq!(output.exit_code, 2, "stderr:\n{}", output.stderr);
    assert!(
        output.stderr.contains("symlink"),
        "stderr did not mention symlinked path:\n{}",
        output.stderr
    );
    assert_eq!(
        before, after,
        "symlink rejection must not mutate the fixture"
    );
    assert_eq!(
        outside_before, outside_after,
        "symlink rejection must not mutate outside paths"
    );
}
