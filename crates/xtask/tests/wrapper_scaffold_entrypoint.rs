use std::{
    fs,
    path::{Path, PathBuf},
};

#[path = "support/onboard_agent_harness.rs"]
mod harness;

use harness::{fixture_root, run_xtask, snapshot_files, wrapper_scaffold_args};

fn scaffold_fixture(prefix: &str) -> PathBuf {
    let fixture = fixture_root(prefix);
    fs::remove_dir_all(fixture.join("crates/gemini_cli")).expect("remove seeded gemini crate");
    fixture
}

fn assert_scaffold_shell(root: &Path) {
    let crate_root = root.join("crates/gemini_cli");
    assert!(crate_root.join("Cargo.toml").is_file());
    assert!(crate_root.join("README.md").is_file());
    assert!(crate_root.join("LICENSE-APACHE").is_file());
    assert!(crate_root.join("LICENSE-MIT").is_file());
    assert!(crate_root.join("src/lib.rs").is_file());
}

#[test]
fn scaffold_wrapper_crate_dry_run_previews_exact_file_set() {
    let fixture = scaffold_fixture("wrapper-scaffold-dry-run");
    let before = snapshot_files(&fixture);

    let first = run_xtask(&fixture, wrapper_scaffold_args("--dry-run", "gemini_cli"));
    let second = run_xtask(&fixture, wrapper_scaffold_args("--dry-run", "gemini_cli"));
    let after = snapshot_files(&fixture);

    assert_eq!(first.exit_code, 0, "stderr:\n{}", first.stderr);
    assert_eq!(second.exit_code, 0, "stderr:\n{}", second.stderr);
    assert_eq!(
        first.stdout, second.stdout,
        "dry-run output must be deterministic"
    );
    assert_eq!(before, after, "dry-run must not mutate the fixture");

    for path in [
        "crates/gemini_cli/Cargo.toml",
        "crates/gemini_cli/README.md",
        "crates/gemini_cli/LICENSE-APACHE",
        "crates/gemini_cli/LICENSE-MIT",
        "crates/gemini_cli/src/lib.rs",
    ] {
        assert!(
            first.stdout.contains(path),
            "dry-run stdout must mention {path}:\n{}",
            first.stdout
        );
    }
}

#[test]
fn scaffold_wrapper_crate_write_creates_minimal_publishable_shell() {
    let fixture = scaffold_fixture("wrapper-scaffold-write");
    let before = snapshot_files(&fixture);

    let output = run_xtask(&fixture, wrapper_scaffold_args("--write", "gemini_cli"));
    let after = snapshot_files(&fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert_ne!(before, after, "write mode must mutate the fixture");
    assert_scaffold_shell(&fixture);

    let manifest =
        fs::read_to_string(fixture.join("crates/gemini_cli/Cargo.toml")).expect("read manifest");
    assert!(manifest.contains("name = \"unified-agent-api-gemini-cli\""));
    assert!(manifest.contains("version.workspace = true"));
    assert!(manifest.contains("edition = \"2021\""));
    assert!(manifest.contains("rust-version = \"1.78\""));
    assert!(manifest.contains("license = \"MIT OR Apache-2.0\""));
    assert!(manifest.contains("repository = \"https://github.com/atomize-hq/unified-agent-api\""));
    assert!(manifest.contains("homepage = \"https://github.com/atomize-hq/unified-agent-api\""));
    assert!(manifest.contains("keywords = [\"gemini\", \"cli\", \"wrapper\", \"agent\"]"));
    assert!(manifest.contains("categories = [\"api-bindings\", \"command-line-interface\"]"));
    assert!(manifest.contains("readme = \"README.md\""));

    let root_apache = fs::read(fixture.join("LICENSE-APACHE")).expect("read root apache");
    let root_mit = fs::read(fixture.join("LICENSE-MIT")).expect("read root mit");
    let crate_apache =
        fs::read(fixture.join("crates/gemini_cli/LICENSE-APACHE")).expect("read crate apache");
    let crate_mit =
        fs::read(fixture.join("crates/gemini_cli/LICENSE-MIT")).expect("read crate mit");
    assert_eq!(crate_apache, root_apache);
    assert_eq!(crate_mit, root_mit);
}

#[test]
fn scaffold_wrapper_crate_replay_is_noop() {
    let fixture = scaffold_fixture("wrapper-scaffold-replay");

    let first = run_xtask(&fixture, wrapper_scaffold_args("--write", "gemini_cli"));
    let after_first = snapshot_files(&fixture);
    let second = run_xtask(&fixture, wrapper_scaffold_args("--write", "gemini_cli"));
    let after_second = snapshot_files(&fixture);

    assert_eq!(first.exit_code, 0, "stderr:\n{}", first.stderr);
    assert_eq!(second.exit_code, 0, "stderr:\n{}", second.stderr);
    assert_eq!(
        after_first, after_second,
        "identical replay must not change the filesystem"
    );
}

#[test]
fn scaffold_wrapper_crate_divergent_file_fails_without_partial_writes() {
    let fixture = scaffold_fixture("wrapper-scaffold-divergent");
    let first = run_xtask(&fixture, wrapper_scaffold_args("--write", "gemini_cli"));
    assert_eq!(first.exit_code, 0, "stderr:\n{}", first.stderr);

    fs::write(
        fixture.join("crates/gemini_cli/README.md"),
        "# Tampered README\n",
    )
    .expect("tamper README");
    let before = snapshot_files(&fixture);

    let second = run_xtask(&fixture, wrapper_scaffold_args("--write", "gemini_cli"));
    let after = snapshot_files(&fixture);

    assert_eq!(second.exit_code, 2, "stderr:\n{}", second.stderr);
    assert!(
        second.stderr.contains("crates/gemini_cli/README.md"),
        "stderr did not mention divergent README:\n{}",
        second.stderr
    );
    assert_eq!(
        before, after,
        "divergent replay must not leave partial writes"
    );
}
