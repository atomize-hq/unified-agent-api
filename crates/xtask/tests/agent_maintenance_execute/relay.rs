use super::*;
const REQUEST: &str = "docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml";
const MARKER: &str =
    "docs/agents/lifecycle/codex-maintenance/governance/automation-stand-down/0.98.0.toml";
const NOTE: &str = "docs/agents/lifecycle/codex-maintenance/runtime-note.md";

fn executor(fixture: &Path, action: &str) -> std::path::PathBuf {
    let binary = fake_execute_codex_binary(fixture);
    let raw = fs::read_to_string(&binary).unwrap();
    let body = format!(
        "  relay)\n    printf 'candidate output\\n' > '{NOTE}'\n    {action}\n    ;;\n  success)"
    );
    harness::write_text(&binary, &raw.replace("  success)", &body));
    binary
}
fn prepare(fixture: &Path, binary: &Path) {
    let dry = run_execute_cli(execute_args("--dry-run", Some(binary)), fixture);
    assert_eq!(dry.exit_code, 0, "{}", dry.stderr);
    write_fake_execute_codex_scenario(fixture, "relay");
}
fn failed(fixture: &Path, binary: &Path, expected: &str) {
    let before = snapshot_without_execute_runs(fixture);
    let out = run_execute_cli(execute_args("--write", Some(binary)), fixture);
    assert_ne!(out.exit_code, 0, "accepted {expected}");
    assert!(out.stderr.contains(expected), "{}", out.stderr);
    assert_eq!(
        snapshot_without_execute_runs(fixture),
        before,
        "rejected output leaked"
    );
    let report = read_json(
        &fixture
            .join(EXECUTE_RUNS_ROOT)
            .join(EXECUTE_WRITE_RUN_ID)
            .join("validation-report.json"),
    );
    assert_eq!(report["status"], "fail");
    let recovery = fs::read_to_string(
        fixture
            .join(EXECUTE_RUNS_ROOT)
            .join(EXECUTE_WRITE_RUN_ID)
            .join("recovery-candidate.txt"),
    )
    .unwrap();
    assert!(Path::new(recovery.trim()).join(NOTE).exists());
}
#[test]
fn rejected_shapes_and_failed_execution_never_deliver_candidate_files() {
    for (name, action, expected) in [
        ("nonzero", "exit 23".to_owned(), "exit code 23"),
        ("request-edit", format!("printf forged > '{REQUEST}'"), "protected request"),
        ("request-delete", format!("rm '{REQUEST}'"), "protected request"),
        ("request-add", "mkdir -p docs/new; printf forged > docs/new/maintenance-request.toml".into(), "protected request"),
        ("marker-edit", format!("printf forged > '{MARKER}'"), "protected request"),
        ("marker-delete", format!("rm '{MARKER}'"), "protected request"),
        ("marker-add", format!("printf forged > '{}.new'", MARKER), "protected request"),
        ("delete", "rm cli_manifests/codex/versions/0.98.0.json".into(), "unsupported deletion"),
        ("mode", format!("chmod +x '{NOTE}'"), "unsupported new file mode"),
        ("type", "rm cli_manifests/codex/versions/0.98.0.json; mkdir cli_manifests/codex/versions/0.98.0.json".into(), "unsupported deletion/type change"),
        ("commit", "git add .; git commit -qm forged".into(), "changed candidate HEAD"),
        ("other-run", "mkdir -p docs/agents/.uaa-temp/agent-maintenance/runs/other; printf escaped > docs/agents/.uaa-temp/agent-maintenance/runs/other/state.json".into(), "write boundary violation"),
        ("closeout", "printf forged > docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json".into(), "closeout remains manual"),
    ] {
        let fixture = prepare_execute_fixture(&format!("relay-{name}"));
        harness::write_text(&fixture.join(MARKER), "reason = 'operator frozen'\n");
        let binary = executor(&fixture, &action);
        prepare(&fixture, &binary);
        failed(&fixture, &binary, expected);
    }
}
#[test]
fn staged_reporting_output_is_delivered_without_changing_caller_git() {
    let fixture = prepare_execute_fixture("relay-staged-report");
    let binary = executor(&fixture, "git add .");
    prepare(&fixture, &binary);
    let out = run_execute_cli(execute_args("--write", Some(&binary)), &fixture);
    assert_eq!(out.exit_code, 0, "{}", out.stderr);
    assert_eq!(
        fs::read_to_string(fixture.join(NOTE)).unwrap(),
        "candidate output\n"
    );
    assert!(!fixture.join(".git").exists());
}
#[test]
fn gate_error_after_candidate_writes_does_not_deliver() {
    let fixture = prepare_execute_fixture("relay-gate-error");
    harness::write_text(&fixture.join("gate-command.sh"), "#!/bin/sh\nexit 47\n");
    let binary = executor(&fixture, ":");
    prepare(&fixture, &binary);
    failed(&fixture, &binary, "green gate failed");
}
#[test]
fn validation_error_after_candidate_writes_does_not_deliver() {
    let fixture = prepare_execute_fixture("relay-validation-error");
    let binary = executor(
        &fixture,
        "printf malformed > crates/xtask/data/agent_registry.toml",
    );
    prepare(&fixture, &binary);
    failed(&fixture, &binary, "write boundary violation");
}
#[test]
fn concurrent_caller_edits_are_preserved_and_refuse_delivery() {
    let fixture = prepare_execute_fixture("relay-concurrent-caller");
    // Only this fake executor is given an absolute caller path to simulate another writer.
    let binary = executor(
        &fixture,
        &format!(
            "printf concurrent > '{}'",
            fixture.join("operator.txt").display()
        ),
    );
    prepare(&fixture, &binary);
    let out = run_execute_cli(execute_args("--write", Some(&binary)), &fixture);
    assert_ne!(out.exit_code, 0);
    assert!(out.stderr.contains("caller changed"), "{}", out.stderr);
    assert_eq!(
        fs::read(fixture.join("operator.txt")).unwrap(),
        b"concurrent"
    );
    assert!(!fixture.join(NOTE).exists());
}
#[cfg(unix)]
#[test]
fn symlink_candidate_does_not_modify_external_target() {
    let fixture = prepare_execute_fixture("relay-symlink");
    let external = tempfile::tempdir().unwrap();
    fs::write(external.path().join("external"), b"outside").unwrap();
    let binary = executor(
        &fixture,
        &format!(
            "rm '{NOTE}'; ln -s '{}' '{NOTE}'",
            external.path().join("external").display()
        ),
    );
    prepare(&fixture, &binary);
    let out = run_execute_cli(execute_args("--write", Some(&binary)), &fixture);
    assert_ne!(out.exit_code, 0);
    assert!(out.stderr.contains("symlink/type change"), "{}", out.stderr);
    assert!(!fixture.join(NOTE).exists());
    assert_eq!(
        fs::read(external.path().join("external")).unwrap(),
        b"outside"
    );
}

#[test]
fn tracked_cache_paths_cannot_escape_the_guard() {
    let fixture = prepare_execute_fixture("relay-tracked-cache");
    let binary = executor(&fixture, "mkdir -p scripts/__pycache__; printf hidden > scripts/__pycache__/input.pyc; git add -f scripts/__pycache__/input.pyc");
    prepare(&fixture, &binary);
    failed(&fixture, &binary, "write boundary violation");
}
#[test]
fn caller_index_change_refuses_delivery_without_overwriting_it() {
    let fixture = prepare_execute_fixture("relay-caller-index");
    for args in [
        vec!["init", "-q"],
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
            "commit",
            "-qm",
            "baseline",
        ],
    ] {
        assert!(std::process::Command::new("git")
            .current_dir(&fixture)
            .args(args)
            .status()
            .unwrap()
            .success());
    }
    let binary = executor(
        &fixture,
        &format!(
            "git -C '{}' update-index --assume-unchanged Cargo.toml",
            fixture.display()
        ),
    );
    prepare(&fixture, &binary);
    let out = run_execute_cli(execute_args("--write", Some(&binary)), &fixture);
    assert_ne!(out.exit_code, 0);
    assert!(out.stderr.contains("caller changed"), "{}", out.stderr);
    assert!(!fixture.join(NOTE).exists());
    let state = std::process::Command::new("git")
        .current_dir(&fixture)
        .args(["ls-files", "-v", "Cargo.toml"])
        .output()
        .unwrap();
    assert!(state.stdout.starts_with(b"h "));
}
#[test]
fn candidate_git_object_corruption_is_rejected_and_preserved() {
    let fixture = prepare_execute_fixture("relay-git-corruption");
    let binary = executor(&fixture, "printf corrupt > .git/objects/corrupt");
    prepare(&fixture, &binary);
    // Corrupt a named object, not an irrelevant non-object file.
    let raw = fs::read_to_string(&binary).unwrap().replace(
        "printf corrupt > .git/objects/corrupt",
        "obj=$(git rev-parse HEAD); chmod u+w .git/objects/${obj:0:2}/${obj:2}; printf corrupt > .git/objects/${obj:0:2}/${obj:2}",
    );
    harness::write_text(&binary, &raw);
    failed(&fixture, &binary, "fsck failed");
}

#[test]
fn spawn_failure_after_candidate_preflight_still_finalizes_without_delivery() {
    let fixture = prepare_execute_fixture("relay-spawn-failure");
    let binary = fake_execute_codex_binary(&fixture);
    prepare(&fixture, &binary);
    let raw = fs::read_to_string(&binary).unwrap();
    harness::write_text(&binary, &raw.replace("case \"$scenario\" in", "if [[ \"$workspace_root\" == *uaa-relay-* ]]; then chmod 644 \"$0\"; fi\ncase \"$scenario\" in"));
    let before = snapshot_without_execute_runs(&fixture);
    let out = run_execute_cli(execute_args("--write", Some(&binary)), &fixture);
    assert_ne!(out.exit_code, 0);
    assert!(out.stderr.contains("spawn codex binary"), "{}", out.stderr);
    assert!(out.stderr.contains("candidate retained"));
    assert_eq!(snapshot_without_execute_runs(&fixture), before);
    let report = read_json(
        &fixture
            .join(EXECUTE_RUNS_ROOT)
            .join(EXECUTE_WRITE_RUN_ID)
            .join("validation-report.json"),
    );
    assert_eq!(report["status"], "fail");
}
