use super::*;

#[cfg(unix)]
use std::{os::unix::fs::PermissionsExt, process::Command};

#[test]
fn integration_depth_validation_checks_candidate_context_for_every_root_without_skips() {
    let yml = read_repo_file(".github/workflows/ci.yml");
    for (job, next, root) in [
        (
            "codex-validate-artifacts",
            "claude-validate-artifacts",
            "codex",
        ),
        (
            "claude-validate-artifacts",
            "opencode-validate-artifacts",
            "claude_code",
        ),
        ("opencode-validate-artifacts", "versions", "opencode"),
    ] {
        let block = section_between(
            &yml,
            &format!("\n  {job}:"),
            &format!("\n  {next}:"),
            "ci.yml",
        );
        assert!(
            block.contains("fetch-depth: 0"),
            "{job}: fetch the complete candidate history for PRs and queue groups"
        );
        assert!(
            !block.contains("has_versions") && !block.contains("hashFiles"),
            "{job}: records can outlive metadata"
        );
        assert!(block.contains("DEPTH_TIP=\"$(python3 scripts/check_staging_integration.py tip)\""));
        assert_text_order(
            block,
            "check_staging_integration.py tip",
            "cargo run -p xtask -- manifest-validate",
            "ci.yml",
        );
        assert!(block.contains("ARGS+=(--tip \"$DEPTH_TIP\")"));
        assert!(block.contains(&format!("ARGS=(--root cli_manifests/{root})")));
        assert!(block.contains("cargo run -p xtask -- manifest-validate \"${ARGS[@]}\""));
        assert!(
            !block.contains("pull_request.head.sha") && !block.contains("pull_request.base.sha")
        );
    }
}

#[test]
fn staging_push_checks_the_event_transition_with_only_the_light_checks() {
    let yml = read_repo_file(".github/workflows/depth-tip.yml");
    for required in [
        "on:\n  push:\n    branches: [staging]",
        "ref: ${{ github.sha }}",
        "TIP_SHA: ${{ github.event.before }}",
        "git fetch --no-tags --depth=1 origin \"$TIP_SHA\"",
        "for agent in codex claude_code opencode; do",
        "cargo run -p xtask -- manifest-validate --root \"cli_manifests/$agent\" --tip \"$TIP_SHA\"",
        "cargo run -p xtask -- support-matrix --check",
    ] {
        assert!(yml.contains(required), "missing push input/check: {required}");
    }
    assert!(
        !yml.contains("HEAD^1"),
        "a multi-commit push must compare with event.before"
    );
    for forbidden in [
        "paths:",
        "make preflight",
        "cargo test",
        "contents: write",
        "workflow_dispatch",
    ] {
        assert!(!yml.contains(forbidden), "light push check: {forbidden}");
    }
    assert_text_order(&yml, "git fetch", "--tip \"$TIP_SHA\"", "depth-tip.yml");
}

#[test]
#[cfg(unix)]
fn push_shell_fetches_a_multicommit_before_object_and_refuses_missing_baselines() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    fs::create_dir(&source).unwrap();
    let git = |root: &std::path::Path, args: &[&str]| {
        let output = Command::new("git")
            .current_dir(root)
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.hooksPath=/dev/null",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success(), "git {args:?}: {output:?}");
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    };
    git(&source, &["init", "-q", "-b", "staging"]);
    fs::write(source.join("file"), "before").unwrap();
    git(&source, &["add", "."]);
    git(&source, &["commit", "-qm", "before"]);
    let before = git(&source, &["rev-parse", "HEAD"]);
    for change in ["one", "two"] {
        fs::write(source.join("file"), change).unwrap();
        git(&source, &["commit", "-qam", change]);
    }
    let candidate = temp.path().join("candidate");
    git(
        temp.path(),
        &[
            "clone",
            "-q",
            "--depth=1",
            &format!("file://{}", source.display()),
            candidate.to_str().unwrap(),
        ],
    );
    let absent = Command::new("git")
        .current_dir(&candidate)
        .args(["cat-file", "-e", &before])
        .output()
        .unwrap();
    assert!(
        !absent.status.success(),
        "the prior push tip must initially be outside the shallow clone"
    );
    let bin = temp.path().join("bin");
    fs::create_dir(&bin).unwrap();
    let cargo = bin.join("cargo");
    fs::write(&cargo, "#!/bin/bash\nprintf '%s\\n' \"$*\" >> \"$TRACE\"\n").unwrap();
    fs::set_permissions(&cargo, fs::Permissions::from_mode(0o755)).unwrap();
    let yml = read_repo_file(".github/workflows/depth-tip.yml");
    let body = yml.split_once("        run: |\n").unwrap().1;
    let script = body
        .lines()
        .map(|line| line.strip_prefix("          ").unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n");
    let trace = temp.path().join("trace");
    let run = |tip: &str| {
        Command::new("bash")
            .args(["--noprofile", "--norc", "-c", &script])
            .current_dir(&candidate)
            .env("TIP_SHA", tip)
            .env("TRACE", &trace)
            .env(
                "PATH",
                format!("{}:{}", bin.display(), std::env::var("PATH").unwrap()),
            )
            .output()
            .unwrap()
    };
    let result = run(&before);
    assert!(result.status.success(), "{result:?}");
    git(&candidate, &["cat-file", "-e", &before]);
    let calls = fs::read_to_string(&trace).unwrap();
    assert_eq!(calls.lines().count(), 4);
    for agent in ["codex", "claude_code", "opencode"] {
        assert!(calls.contains(&format!(
            "manifest-validate --root cli_manifests/{agent} --tip {before}"
        )));
    }
    assert!(calls.contains("support-matrix --check"));
    for bad in ["".to_string(), "0".repeat(40), "invalid".to_string()] {
        fs::remove_file(&trace).ok();
        let result = run(&bad);
        assert!(!result.status.success(), "bad baseline {bad}: {result:?}");
        assert!(
            !trace.exists(),
            "no validation may run with a substituted baseline"
        );
    }
}
