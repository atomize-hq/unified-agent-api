use super::*;

#[test]
fn terminal_gate_covers_all_existing_jobs_without_duplicate_test_runs() {
    use std::collections::BTreeSet;
    use std::process::Command;

    let yml = read_repo_file(".github/workflows/ci.yml");
    let jobs = yml.split_once("jobs:\n").unwrap().1;
    let actual: BTreeSet<_> = Regex::new(r"(?m)^  ([a-z][a-z0-9-]*):$")
        .unwrap()
        .captures_iter(jobs)
        .map(|c| c[1].to_string())
        .filter(|id| id != "staging-integration-gate")
        .collect();
    assert_eq!(actual.len(), 17);
    let block = jobs
        .split_once("\n  staging-integration-gate:\n")
        .unwrap()
        .1;
    let needs = section_between(block, "    needs:\n", "    steps:\n", "ci.yml");
    let declared: Vec<_> = Regex::new(r"(?m)^      - ([a-z][a-z0-9-]*)$")
        .unwrap()
        .captures_iter(needs)
        .map(|c| c[1].to_string())
        .collect();
    assert_eq!(declared.len(), actual.len());
    assert_eq!(declared.into_iter().collect::<BTreeSet<_>>(), actual);
    let output = Command::new("python3")
        .current_dir(repo_root().join("scripts"))
        .args([
            "-c",
            "import json, check_staging_integration as g; print(json.dumps(sorted(g.JOBS)))",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let evaluator: BTreeSet<String> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(evaluator, actual);
    for required in [
        "if: ${{ always() }}",
        "CI_NEEDS: ${{ toJSON(needs) }}",
        "ref: ${{ github.sha }}",
        "fetch-depth: 0",
        "github.event.pull_request.base.ref == 'staging'",
        "github.event.merge_group.base_ref == 'refs/heads/staging'",
        "'Staging integration gate' || 'CI summary (non-staging)'",
        "python3 scripts/check_staging_integration.py gate",
    ] {
        assert!(block.contains(required), "gate missing {required}");
    }
    assert!(!yml.contains("continue-on-error"));
    assert!(yml.contains("  merge_group:\n    types: [checks_requested]\n    branches: [staging]"));
    assert!(read_repo_file("Makefile")
        .contains("python3 -m unittest discover -s scripts -p 'test_*.py'"));
    assert!(
        !yml.contains("python3 -m unittest discover"),
        "preflight owns the Python suite, even when readiness skips"
    );
}

#[test]
#[cfg(unix)]
fn queue_validation_passes_the_supplied_base_to_every_manifest_root() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    use std::process::Command;

    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source");
    fs::create_dir(&source).unwrap();
    let git = |cwd: &std::path::Path, args: &[&str]| {
        let output = Command::new("git")
            .current_dir(cwd)
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
    fs::write(source.join("fixture"), "base").unwrap();
    git(&source, &["add", "."]);
    git(&source, &["commit", "-qm", "base"]);
    let base = git(&source, &["rev-parse", "HEAD"]);
    fs::write(source.join("fixture"), "group").unwrap();
    git(&source, &["commit", "-qam", "group"]);
    let head = git(&source, &["rev-parse", "HEAD"]);
    let candidate = dir.path().join("candidate");
    git(
        dir.path(),
        &[
            "clone",
            "-q",
            source.to_str().unwrap(),
            candidate.to_str().unwrap(),
        ],
    );
    symlink(repo_root().join("scripts"), candidate.join("scripts")).unwrap();
    let event = dir.path().join("event.json");
    let queue_event = serde_json::json!({
        "action": "checks_requested",
        "merge_group": {"base_ref": "refs/heads/staging", "base_sha": base, "head_sha": head},
    });
    let bin = dir.path().join("bin");
    fs::create_dir(&bin).unwrap();
    let cargo = bin.join("cargo");
    fs::write(&cargo, "#!/bin/bash\nprintf '%s\\n' \"$*\" >> \"$TRACE\"\n").unwrap();
    fs::set_permissions(&cargo, fs::Permissions::from_mode(0o755)).unwrap();
    let yml = read_repo_file(".github/workflows/ci.yml");
    for (job, root) in [
        ("codex", "codex"),
        ("claude", "claude_code"),
        ("opencode", "opencode"),
    ] {
        let tail = yml
            .split_once(&format!("\n  {job}-validate-artifacts:\n"))
            .unwrap()
            .1;
        let body = tail.split_once("        run: |\n").unwrap().1;
        let shell = body
            .lines()
            .take_while(|line| line.is_empty() || line.starts_with("          "))
            .map(|line| line.strip_prefix("          ").unwrap_or(line))
            .collect::<Vec<_>>()
            .join("\n");
        let trace = dir.path().join(format!("{job}.trace"));
        let run = |name| {
            Command::new("bash")
                .args(["--noprofile", "--norc", "-c", &shell])
                .current_dir(&candidate)
                .env(
                    "PATH",
                    format!("{}:{}", bin.display(), std::env::var("PATH").unwrap()),
                )
                .env("GITHUB_EVENT_NAME", name)
                .env("GITHUB_EVENT_PATH", &event)
                .env("GITHUB_SHA", &head)
                .env("DEPTH_TIP", "")
                .env("TRACE", &trace)
                .output()
                .unwrap()
        };
        fs::write(&event, queue_event.to_string()).unwrap();
        let output = run("merge_group");
        assert!(output.status.success(), "{job}: {output:?}");
        let recorded = fs::read_to_string(&trace).unwrap();
        assert!(
            recorded.contains(&format!(
                "manifest-validate --root cli_manifests/{root} --tip {base}"
            )),
            "{job}: {recorded}"
        );
        fs::remove_file(&trace).unwrap();
        for (key, value) in [
            ("base_sha", ""),
            ("base_sha", "1111111111111111111111111111111111111111"),
            ("head_sha", base.as_str()),
            ("base_ref", "refs/heads/main"),
        ] {
            let mut bad = queue_event.clone();
            bad["merge_group"][key] = value.into();
            fs::write(&event, bad.to_string()).unwrap();
            let output = run("merge_group");
            assert!(
                !output.status.success(),
                "{job}: invalid {key} reached validation: {output:?}"
            );
            assert!(
                !trace.exists(),
                "{job}: invalid context must stop before cargo"
            );
        }
        fs::write(
            &event,
            serde_json::json!({"pull_request": {"base": {"ref": "main"}}}).to_string(),
        )
        .unwrap();
        let output = run("pull_request");
        assert!(
            output.status.success(),
            "{job}: ordinary main PR: {output:?}"
        );
        let recorded = fs::read_to_string(&trace).unwrap();
        assert!(recorded.contains(&format!("manifest-validate --root cli_manifests/{root}")));
        assert!(
            !recorded.contains("--tip"),
            "non-staging validation remains one-tree"
        );
    }
}
