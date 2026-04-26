use std::{fs, path::Path};

#[path = "support/onboard_agent_harness.rs"]
mod harness;

use harness::{
    hyphenated_scaffold_fixture_root, nested_scaffold_fixture_root, run_xtask,
    scaffold_fixture_root, snapshot_files, wrapper_scaffold_args, HYPHENATED_GEMINI_CRATE_PATH,
    NESTED_GEMINI_CRATE_PATH, SEEDED_GEMINI_CRATE_PATH,
};

fn assert_scaffold_shell(root: &Path, crate_path: &str) {
    let crate_root = root.join(crate_path);
    assert!(crate_root.join("Cargo.toml").is_file());
    assert!(crate_root.join("README.md").is_file());
    assert!(crate_root.join("LICENSE-APACHE").is_file());
    assert!(crate_root.join("LICENSE-MIT").is_file());
    assert!(crate_root.join("src/lib.rs").is_file());
}

#[test]
fn scaffold_wrapper_crate_dry_run_previews_exact_file_set() {
    let fixture = scaffold_fixture_root("wrapper-scaffold-dry-run");
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
    let fixture = scaffold_fixture_root("wrapper-scaffold-write");
    let before = snapshot_files(&fixture);

    let output = run_xtask(&fixture, wrapper_scaffold_args("--write", "gemini_cli"));
    let after = snapshot_files(&fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert_ne!(before, after, "write mode must mutate the fixture");
    assert_scaffold_shell(&fixture, SEEDED_GEMINI_CRATE_PATH);

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
    let fixture = scaffold_fixture_root("wrapper-scaffold-replay");

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
    let fixture = scaffold_fixture_root("wrapper-scaffold-divergent");
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

#[test]
fn scaffold_wrapper_crate_supports_nested_registry_crate_path() {
    let fixture = nested_scaffold_fixture_root("wrapper-scaffold-nested-path");
    let before = snapshot_files(&fixture);

    let dry_run = run_xtask(&fixture, wrapper_scaffold_args("--dry-run", "gemini_cli"));
    let after_dry_run = snapshot_files(&fixture);

    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    assert_eq!(before, after_dry_run, "dry-run must not mutate the fixture");

    for path in [
        "crates/runtime/gemini_shell/Cargo.toml",
        "crates/runtime/gemini_shell/README.md",
        "crates/runtime/gemini_shell/LICENSE-APACHE",
        "crates/runtime/gemini_shell/LICENSE-MIT",
        "crates/runtime/gemini_shell/src/lib.rs",
    ] {
        assert!(
            dry_run.stdout.contains(path),
            "dry-run stdout must mention {path}:\n{}",
            dry_run.stdout
        );
    }

    let first = run_xtask(&fixture, wrapper_scaffold_args("--write", "gemini_cli"));
    assert_eq!(first.exit_code, 0, "stderr:\n{}", first.stderr);
    assert_scaffold_shell(&fixture, NESTED_GEMINI_CRATE_PATH);

    let manifest = fs::read_to_string(fixture.join(NESTED_GEMINI_CRATE_PATH).join("Cargo.toml"))
        .expect("read nested manifest");
    assert!(manifest.contains("name = \"unified-agent-api-gemini-cli\""));

    let after_first = snapshot_files(&fixture);
    let second = run_xtask(&fixture, wrapper_scaffold_args("--write", "gemini_cli"));
    let after_second = snapshot_files(&fixture);

    assert_eq!(second.exit_code, 0, "stderr:\n{}", second.stderr);
    assert_eq!(
        after_first, after_second,
        "identical replay must not change the filesystem"
    );

    fs::write(
        fixture.join(NESTED_GEMINI_CRATE_PATH).join("README.md"),
        "# Tampered Nested README\n",
    )
    .expect("tamper nested README");
    let before_divergence = snapshot_files(&fixture);

    let divergent = run_xtask(&fixture, wrapper_scaffold_args("--write", "gemini_cli"));
    let after_divergence = snapshot_files(&fixture);

    assert_eq!(divergent.exit_code, 2, "stderr:\n{}", divergent.stderr);
    assert!(
        divergent
            .stderr
            .contains("crates/runtime/gemini_shell/README.md"),
        "stderr did not mention divergent nested README:\n{}",
        divergent.stderr
    );
    assert_eq!(
        before_divergence, after_divergence,
        "divergent replay must not leave partial writes"
    );
}

#[test]
fn scaffold_wrapper_crate_normalizes_hyphenated_lib_name() {
    let fixture = hyphenated_scaffold_fixture_root("wrapper-scaffold-hyphenated-path");
    let before = snapshot_files(&fixture);

    let dry_run = run_xtask(&fixture, wrapper_scaffold_args("--dry-run", "gemini_cli"));
    let after_dry_run = snapshot_files(&fixture);

    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    assert_eq!(before, after_dry_run, "dry-run must not mutate the fixture");
    assert!(
        dry_run
            .stdout
            .contains("crates/gemini-cli/Cargo.toml [create]"),
        "dry-run stdout must mention the hyphenated crate path:\n{}",
        dry_run.stdout
    );

    let write = run_xtask(&fixture, wrapper_scaffold_args("--write", "gemini_cli"));
    assert_eq!(write.exit_code, 0, "stderr:\n{}", write.stderr);
    assert_scaffold_shell(&fixture, HYPHENATED_GEMINI_CRATE_PATH);

    let manifest = fs::read_to_string(
        fixture
            .join(HYPHENATED_GEMINI_CRATE_PATH)
            .join("Cargo.toml"),
    )
    .expect("read hyphenated manifest");
    assert!(manifest.contains("name = \"unified-agent-api-gemini-cli\""));
    assert!(manifest.contains("[lib]\nname = \"gemini_cli\"\n"));

    let readme = fs::read_to_string(fixture.join(HYPHENATED_GEMINI_CRATE_PATH).join("README.md"))
        .expect("read hyphenated readme");
    assert!(readme.contains("- Rust library crate: `gemini_cli`"));
}
