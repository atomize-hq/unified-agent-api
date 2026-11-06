//! The governing data `onboard-agent` scaffolds passes `manifest-validate` as written, and drives
//! the real manifest commands.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::{json as value, Value};

use super::{
    governing_data::{depth_fixture_args, json, GNU},
    harness::{fixture_root, run_xtask, seed_release_touchpoints, write_args},
    run_cli,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

/// Unrelated workspace prerequisites: the committed manifest roots of the registry's other agents
/// and the support publication (JSON artifact and its Markdown page) derived from them, which the
/// validator's workspace-wide publication check reads. Nothing of the onboarded agent is seeded.
fn seed_published_roots(fixture: &Path) {
    let repo = repo_root();
    let listed = Command::new("git")
        .args([
            "ls-files",
            "-z",
            "cli_manifests",
            "docs/specs/unified-agent-api/support-matrix.md",
        ])
        .current_dir(&repo)
        .output()
        .expect("git ls-files");
    assert!(listed.status.success());
    for path in String::from_utf8(listed.stdout)
        .expect("utf-8")
        .split('\0')
        .filter(|p| !p.is_empty())
    {
        let to = fixture.join(path);
        fs::create_dir_all(to.parent().expect("parent")).expect("mkdir");
        fs::copy(repo.join(path), to).expect("copy tracked file");
    }
}

fn validate_ok(fixture: &Path, root: &str) {
    let validate = run_xtask(fixture, ["manifest-validate", "--root", root]);
    assert_eq!(
        validate.exit_code, 0,
        "manifest-validate {root}:\n{}\n{}",
        validate.stdout, validate.stderr
    );
    assert!(validate.stdout.contains("OK: manifest-validate"));
}

#[test]
fn scaffolded_manifest_root_passes_manifest_validate_as_written() {
    let fixture = fixture_root("onboard-agent-governing-validate");
    seed_release_touchpoints(&fixture);
    seed_published_roots(&fixture);
    let output = run_cli(write_args("cursor"), &fixture);
    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);

    let root = fixture.join("cli_manifests/cursor");
    let rules = json(&root.join("RULES.json"));
    assert_eq!(
        json(&root.join("current.json")),
        value!({"expected_targets": rules["union"]["expected_targets"], "inputs": []})
    );
    assert_eq!(
        rules["versioning"]["pointers"]["root_pointers_allow_none"],
        true
    );
    assert_eq!(
        fs::read_to_string(root.join("latest_validated.txt")).unwrap(),
        "none\n"
    );
    let governance = fixture.join("docs/agents/lifecycle/cursor-cli-onboarding/governance");
    assert!(json(&governance.join("lifecycle-state.json"))
        .get("support_depth")
        .is_none());
    assert!(!governance.join("approved-agent.toml").exists());

    validate_ok(&fixture, "cli_manifests/cursor");
}

#[test]
fn bootstrap_current_exception_requires_the_exact_pointer_and_bootstrap_rules() {
    let fixture = fixture_root("onboard-agent-bootstrap-guards");
    seed_release_touchpoints(&fixture);
    seed_published_roots(&fixture);
    let output = run_cli(write_args("cursor"), &fixture);
    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    let root = fixture.join("cli_manifests/cursor");
    let current = json(&root.join("current.json"));
    let rules = json(&root.join("RULES.json"));
    let mut extra = current.clone();
    extra["mode"] = value!("union");
    let mut wrong_targets = current.clone();
    wrong_targets["expected_targets"] = value!(["other-target"]);
    let mut inputs = current.clone();
    inputs["inputs"] = value!([{}]);

    for (pointer, allow_none, latest) in [
        (extra, true, "none\n"),
        (wrong_targets, true, "none\n"),
        (inputs, true, "none\n"),
        (current.clone(), false, "none\n"),
        (current, true, "1.2.3\n"),
    ] {
        let mut rules = rules.clone();
        rules["versioning"]["pointers"]["root_pointers_allow_none"] = value!(allow_none);
        fs::write(root.join("RULES.json"), rules.to_string()).unwrap();
        fs::write(root.join("current.json"), pointer.to_string()).unwrap();
        fs::write(root.join("latest_validated.txt"), latest).unwrap();
        let output = run_xtask(
            &fixture,
            ["manifest-validate", "--root", "cli_manifests/cursor"],
        );
        assert_eq!(output.exit_code, 2, "{}", output.stderr);
        for code in ["CURRENT_WRONG_KIND", "CURRENT_SCHEMA_INVALID"] {
            assert!(
                output.stderr.contains(code),
                "expected {code}:\n{}",
                output.stderr
            );
        }
    }
}

/// The schemas are the committed ones and RULES is the synthetic subject's, without what
/// onboarding cannot know (acquisition and parity exclusions) and without Annex B adoption.
#[test]
fn scaffolded_governing_data_matches_the_committed_root_shape() {
    let fixture = fixture_root("onboard-agent-governing-shape");
    seed_release_touchpoints(&fixture);
    let output = run_cli(depth_fixture_args("--write"), &fixture);
    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    let root = fixture.join("cli_manifests/depth_fixture");
    let template = |name: &str| {
        json(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/depth-synthetic/governing/manifest-root")
                .join(name),
        )
    };

    for name in ["SCHEMA.json", "VERSION_METADATA_SCHEMA.json"] {
        let mut generated = json(&root.join(name));
        let mut committed = template(name);
        for value in [&mut generated, &mut committed] {
            let object = value.as_object_mut().expect("object");
            object.remove("title");
            object.remove("description");
        }
        assert_eq!(generated, committed, "{name}");
    }
    // The synthetic subject's install-time pointers are the scaffold's.
    for pointer in [
        "min_supported.txt".to_string(),
        "latest_validated.txt".to_string(),
        format!("pointers/latest_supported/{GNU}.txt"),
        format!("pointers/latest_validated/{GNU}.txt"),
    ] {
        let committed = fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/depth-synthetic/governing/manifest-root")
                .join(&pointer),
        )
        .expect("read template pointer");
        assert_eq!(
            fs::read_to_string(root.join(&pointer)).expect("read pointer"),
            committed,
            "{pointer}"
        );
    }

    let mut generated = json(&root.join("RULES.json"));
    let mut committed = template("RULES.json");
    let object = committed.as_object_mut().expect("object");
    object.remove("acquisition");
    object.remove("parity_exclusions");
    committed["storage"]["committed"]["reports"]
        .as_object_mut()
        .expect("reports")
        .remove("depth_record");
    for value in [&mut generated, &mut committed] {
        let globals = value["globals"].as_object_mut().expect("globals");
        globals.remove("meaning");
        globals.remove("notes");
        value["globals"]["effective_flags_model"]
            .as_object_mut()
            .expect("model")
            .remove("reporting");
        value["union"]["promotion_policy"]
            .as_object_mut()
            .expect("policy")
            .remove("notes");
    }
    assert_eq!(generated, committed);
}

fn fixture_binary() -> PathBuf {
    let output = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args([
            "build",
            "-q",
            "-p",
            "unified-agent-api-depth-fixture",
            "--bin",
            "depth_fixture",
            "--message-format=json",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("build fixture");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find_map(|m| m["executable"].as_str().map(PathBuf::from))
        .expect("cargo reported the fixture executable")
}

fn xtask_ok(cwd: &Path, args: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .current_dir(cwd)
        .env("SOURCE_DATE_EPOCH", "1793664000")
        .output()
        .expect("run xtask");
    assert!(
        output.status.success(),
        "xtask {args:?} failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// The unmodified scaffold drives the real manifest commands, as the synthetic subject's
/// templates do: it validates before acquisition and again once a version is reported.
#[test]
fn scaffolded_governing_data_drives_the_real_manifest_commands() {
    let binary = fixture_binary();
    let version_output = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("fixture version");
    let version = String::from_utf8(version_output.stdout).expect("utf-8");
    let version = version
        .trim()
        .strip_prefix("depth_fixture ")
        .expect("version prefix")
        .to_owned();
    let v = version.as_str();

    let fixture = fixture_root("onboard-agent-governing-pipeline");
    seed_release_touchpoints(&fixture);
    seed_published_roots(&fixture);
    let output = run_cli(depth_fixture_args("--write"), &fixture);
    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    let root = fixture.join("cli_manifests/depth_fixture");
    let root_arg = root.to_str().expect("utf-8 root");
    validate_ok(&fixture, root_arg);

    let per_target = root.join(format!("snapshots/{v}/{GNU}.json"));
    xtask_ok(
        &fixture,
        &[
            "depth-fixture-snapshot",
            "--depth-fixture-binary",
            binary.to_str().expect("utf-8 binary"),
            "--out-file",
            per_target.to_str().expect("utf-8 out"),
            "--capture-raw-help",
            "--raw-help-target",
            GNU,
        ],
    );
    xtask_ok(
        &fixture,
        &["manifest-union", "--root", root_arg, "--version", v],
    );
    xtask_ok(
        &fixture,
        &[
            "depth-fixture-wrapper-coverage",
            "--out",
            root.join("wrapper_coverage.json").to_str().expect("utf-8"),
            "--rules",
            root.join("RULES.json").to_str().expect("utf-8"),
        ],
    );
    xtask_ok(
        &fixture,
        &["manifest-report", "--root", root_arg, "--version", v],
    );
    let report = json(&root.join(format!("reports/{v}/coverage.any.json")));
    // Without parity exclusions every upstream unit is in parity: nothing is excluded.
    assert!(
        report["deltas"].get("excluded_commands").is_none(),
        "{report}"
    );
    xtask_ok(
        &fixture,
        &[
            "manifest-version-metadata",
            "--root",
            root_arg,
            "--version",
            v,
            "--status",
            "reported",
        ],
    );
    // A reported version changes the workspace-wide support publication, which the publication
    // lane regenerates; the root's own files are not touched.
    xtask_ok(&fixture, &["support-matrix"]);
    validate_ok(&fixture, root_arg);
    assert_eq!(
        json(&root.join(format!("versions/{v}.json")))["status"],
        "reported"
    );
}
