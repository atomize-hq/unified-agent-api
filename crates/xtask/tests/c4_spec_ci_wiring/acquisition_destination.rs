use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::{Command, Output};

use super::*;

const WORKFLOW: &str = ".github/workflows/parity-acquire.yml";

pub(super) fn shell(step: &str) -> String {
    step.split_once("        run: |\n")
        .expect("workflow shell")
        .1
        .lines()
        .take_while(|line| line.is_empty() || line.starts_with("          "))
        .map(|line| line.strip_prefix("          ").unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn destination(commit: &str, reference: &str) -> (Output, String) {
    let yml = read_repo_file(WORKFLOW);
    let plan = section_between(&yml, "\n  plan:\n", "\n  snapshot:\n", WORKFLOW);
    let first = plan.split_once("    steps:\n").expect("plan steps").1;
    let temp = tempfile::tempdir().unwrap();
    let outputs = temp.path().join("outputs");
    fs::write(&outputs, "").unwrap();
    let output = Command::new("bash")
        .args(["--noprofile", "--norc", "-c", &shell(first)])
        .current_dir(temp.path())
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap())
        .env("COMMIT_ARTIFACTS", commit)
        .env("HAS_AUTOMATION_TOKEN", "true")
        .env("ACQUISITION_REF", reference)
        .env("GITHUB_OUTPUT", &outputs)
        .output()
        .unwrap();
    (output, fs::read_to_string(outputs).unwrap())
}

#[test]
fn integration_destinations_are_refused_before_checkout_even_with_a_token() {
    for reference in ["staging", "refs/heads/staging", "main", "refs/heads/main"] {
        let (output, outputs) = destination("true", reference);
        assert!(!output.status.success(), "{reference}: {output:?}");
        assert!(
            outputs.is_empty(),
            "refused input must not emit a destination"
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("commit=false"));
    }
}

#[test]
fn packet_and_custom_destinations_are_canonical_literal_branches() {
    for branch in [
        "automation/codex-maintenance-1.2.3",
        "custom/acquisition",
        "origin/staging",
        "Staging",
        "stаging", // Cyrillic a: a distinct literal branch, not staging.
        "odd;`touch${IFS}pwn`",
        "0123456789abcdef0123456789abcdef01234567",
    ] {
        for reference in [branch.to_string(), format!("refs/heads/{branch}")] {
            let (output, outputs) = destination("true", &reference);
            assert!(output.status.success(), "{reference}: {output:?}");
            assert_eq!(
                outputs,
                format!("branch_name={branch}\ndelivery_ref=refs/heads/{branch}\n")
            );
        }
    }
}

#[test]
fn invalid_delivery_refs_fail_without_emitting_outputs() {
    for reference in [
        "",
        "refs/heads/",
        "HEAD",
        "@",
        "@{-1}",
        "-bad",
        "refs/heads/-bad",
        "refs/tags/staging",
        "refs/remotes/origin/staging",
        "refs/pull/1/head",
        "refs/heads/refs/heads/staging",
        "staging^0",
        "staging:other",
        "staging~1",
        "/staging",
        "staging//alias",
        "staging/",
        "stage.lock",
        "stage..alias",
        " staging",
        "staging ",
        "stage\nalias",
        "stage\talias",
        "stage\u{7f}alias",
        "stage\u{a0}alias",
        "stage\u{9f}alias",
    ] {
        let (output, outputs) = destination("true", reference);
        assert!(!output.status.success(), "{reference:?}: {output:?}");
        assert!(outputs.is_empty());
    }
    for commit in ["", "1", "TRUE"] {
        assert!(!destination(commit, "custom").0.status.success());
    }
}

#[test]
fn read_only_acquisition_does_not_restrict_the_checkout_ref_or_emit_delivery_outputs() {
    for reference in [
        "staging",
        "refs/heads/main",
        "refs/tags/v1.2.3",
        "HEAD",
        "@{-1}",
    ] {
        let (output, outputs) = destination("false", reference);
        assert!(output.status.success(), "{reference}: {output:?}");
        assert!(outputs.is_empty());
    }
}

pub(super) fn job<'a>(yml: &'a str, name: &str) -> &'a str {
    let tail = yml.split_once(&format!("\n  {name}:\n")).unwrap().1;
    let end = tail
        .find("\n  snapshot:\n")
        .or_else(|| tail.find("\n  union:\n"));
    &tail[..end.unwrap_or(tail.len())]
}

pub(super) fn step<'a>(job: &'a str, name: &str) -> &'a str {
    let tail = job
        .split_once(&format!("      - name: {name}\n"))
        .unwrap()
        .1;
    &tail[..tail.find("\n      - ").unwrap_or(tail.len())]
}

pub(super) fn run(script: &str, cwd: &Path, env: &[(&str, &str)]) -> Output {
    Command::new("bash")
        .args(["--noprofile", "--norc", "-c", script])
        .current_dir(cwd)
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap())
        .envs(env.iter().copied())
        .output()
        .unwrap()
}

#[test]
fn every_checkout_checks_outputs_before_using_the_canonical_full_ref() {
    let yml = read_repo_file(WORKFLOW);
    for (name, source) in [
        ("plan", "steps.destination"),
        ("snapshot", "needs.plan"),
        ("union", "needs.plan"),
    ] {
        let job = job(&yml, name);
        let guard = step(job, "Check canonical acquisition outputs");
        assert!(guard.contains("if: ${{ inputs.commit }}"));
        assert_text_order(
            job,
            "name: Check canonical acquisition outputs",
            "uses: actions/checkout@v5",
            WORKFLOW,
        );
        assert_text_order(
            job,
            "uses: actions/checkout@v5",
            "name: Verify acquisition checkout",
            WORKFLOW,
        );
        assert_text_order(
            job,
            "name: Verify acquisition checkout",
            "uses: dtolnay/rust-toolchain@stable",
            WORKFLOW,
        );
        assert!(job.contains(&format!("ref: ${{{{ inputs.commit && {source}.outputs.delivery_ref || (!inputs.commit && inputs.ref) }}}}")));
        assert!(guard.contains(&format!(
            "BRANCH_NAME: ${{{{ {source}.outputs.branch_name }}}}"
        )));
        assert!(guard.contains(&format!(
            "DELIVERY_REF: ${{{{ {source}.outputs.delivery_ref }}}}"
        )));
        let dir = tempfile::tempdir().unwrap();
        for (branch, reference, admitted) in [
            ("custom", "refs/heads/custom", true),
            ("", "", false),
            ("custom", "", false),
            ("", "refs/heads/custom", false),
            ("custom", "refs/heads/staging", false),
            ("staging", "refs/heads/staging", false),
            ("main", "refs/heads/main", false),
            ("custom", "refs/tags/custom", false),
        ] {
            let output = run(
                &shell(guard),
                dir.path(),
                &[("BRANCH_NAME", branch), ("DELIVERY_REF", reference)],
            );
            assert_eq!(
                output.status.success(),
                admitted,
                "{name}: {branch}/{reference}: {output:?}"
            );
        }
    }
    let plan = job(&yml, "plan");
    assert!(plan
        .trim_start()
        .contains("branch_name: ${{ steps.destination.outputs.branch_name }}"));
    assert!(plan.contains("delivery_ref: ${{ steps.destination.outputs.delivery_ref }}"));
}

pub(super) fn git(cwd: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap();
    assert!(output.status.success(), "git {args:?}: {output:?}");
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

pub(super) fn repository() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-b", "custom"]);
    git(dir.path(), &["config", "user.name", "Acquisition test"]);
    git(
        dir.path(),
        &["config", "user.email", "acquisition@example.invalid"],
    );
    fs::write(dir.path().join("fixture"), "original\n").unwrap();
    git(dir.path(), &["add", "fixture"]);
    git(dir.path(), &["commit", "-m", "fixture"]);
    git(
        dir.path(),
        &["update-ref", "refs/remotes/origin/custom", "HEAD"],
    );
    dir
}

#[test]
fn checkout_verification_rejects_detached_wrong_branch_and_missing_remote_head() {
    let yml = read_repo_file(WORKFLOW);
    for name in ["plan", "snapshot", "union"] {
        let step = step(job(&yml, name), "Verify acquisition checkout");
        assert!(step.contains("if: ${{ inputs.commit }}"));
        let script = shell(step);
        let dir = repository();
        let check = || {
            run(
                &script,
                dir.path(),
                &[
                    ("BRANCH_NAME", "custom"),
                    ("DELIVERY_REF", "refs/heads/custom"),
                ],
            )
        };
        assert!(check().status.success());
        git(dir.path(), &["checkout", "--detach"]);
        assert!(!check().status.success());
        git(dir.path(), &["checkout", "-B", "other"]);
        assert!(!check().status.success());
        git(dir.path(), &["checkout", "custom"]);
        fs::write(dir.path().join("fixture"), "different checkout\n").unwrap();
        git(dir.path(), &["add", "fixture"]);
        git(dir.path(), &["commit", "-m", "different checkout"]);
        assert!(
            !check().status.success(),
            "checkout must match the fetched remote head"
        );
        git(
            dir.path(),
            &["update-ref", "refs/remotes/origin/custom", "HEAD"],
        );
        assert!(check().status.success());
        git(
            dir.path(),
            &["update-ref", "-d", "refs/remotes/origin/custom"],
        );
        git(dir.path(), &["tag", "custom"]);
        assert!(
            !check().status.success(),
            "a same-named tag cannot replace the remote branch"
        );
    }
}

#[test]
fn both_packet_ref_spellings_reach_the_same_request_refreeze() {
    let yml = read_repo_file(WORKFLOW);
    let step = step(
        job(&yml, "union"),
        "Re-freeze the maintenance request against the acquired artifacts",
    );
    assert!(step.contains("REF: ${{ needs.plan.outputs.branch_name }}"));
    for reference in [
        "automation/codex-maintenance-1.2.3",
        "refs/heads/automation/codex-maintenance-1.2.3",
    ] {
        let (output, outputs) = destination("true", reference);
        assert!(output.status.success());
        let branch = outputs
            .lines()
            .find_map(|line| line.strip_prefix("branch_name="))
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let outputs_file = dir.path().join("outputs");
        let result = run(
            &shell(step),
            dir.path(),
            &[
                ("AGENT_ID", "codex"),
                ("VERSION", "1.2.3"),
                ("REF", branch),
                ("GITHUB_OUTPUT", outputs_file.to_str().unwrap()),
            ],
        );
        assert!(result.status.success(), "{result:?}");
        let log = String::from_utf8(result.stdout).unwrap();
        assert!(
            log.contains("Maintenance request missing"),
            "must reach request lookup: {log}"
        );
        assert!(!log.contains("Re-freeze skipped"));
    }
}

#[test]
fn commit_shell_pushes_only_the_canonical_head_and_rechecks_its_destination() {
    let yml = read_repo_file(WORKFLOW);
    let step = step(
        job(&yml, "union"),
        "Commit the acquired artifacts onto the packet branch",
    );
    assert!(step.contains("BRANCH: ${{ needs.plan.outputs.branch_name }}"));
    assert!(step.contains("DELIVERY_REF: ${{ needs.plan.outputs.delivery_ref }}"));
    assert!(!step.contains("inputs.ref"));
    let script = shell(step);
    let dir = repository();
    for path in [
        "manifest/snapshots/1.2.3/item.json",
        "manifest/reports/1.2.3/item.json",
        "manifest/versions/1.2.3.json",
        "manifest/wrapper_coverage.json",
        "manifest/artifacts.lock.json",
        "cli_manifests/support_matrix/current.json",
        "docs/specs/unified-agent-api/support-matrix.md",
    ] {
        let path = dir.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "original\n").unwrap();
    }
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-m", "artifacts"]);
    let remote = tempfile::tempdir().unwrap();
    git(remote.path(), &["init", "--bare"]);
    git(
        dir.path(),
        &["remote", "add", "origin", remote.path().to_str().unwrap()],
    );
    git(dir.path(), &["push", "origin", "HEAD:refs/heads/custom"]);
    git(dir.path(), &["tag", "custom"]);
    git(dir.path(), &["push", "origin", "refs/tags/custom"]);
    let before = git(remote.path(), &["rev-parse", "refs/heads/custom"]);
    let output_file = dir.path().join("outputs");
    let transport = tempfile::tempdir().unwrap();
    let helper = dir.path().join("scripts/maintenance_transport.py");
    fs::create_dir_all(helper.parent().unwrap()).unwrap();
    fs::copy(
        repo_root().join("scripts/maintenance_transport.py"),
        &helper,
    )
    .unwrap();
    let report = transport.path().join("prepared.json");
    let state = transport.path().join("delivery.json");
    let deliver = |branch, reference| {
        run(
            &script,
            dir.path(),
            &[
                ("ROOT", "manifest"),
                ("VERSION", "1.2.3"),
                ("AGENT_ID", "codex"),
                ("BRANCH", branch),
                ("DELIVERY_REF", reference),
                ("GITHUB_OUTPUT", output_file.to_str().unwrap()),
                ("TRANSPORT_DIR", transport.path().to_str().unwrap()),
            ],
        )
    };
    fs::write(
        dir.path().join("manifest/artifacts.lock.json"),
        "acquired\n",
    )
    .unwrap();
    fs::write(&report, serde_json::to_vec(&serde_json::json!({
        "applied": true,
        "files": [{"path": "manifest/artifacts.lock.json", "sha256": hex::encode(Sha256::digest(b"acquired\n"))}]
    })).unwrap()).unwrap();
    let stage = Command::new("python3")
        .arg(&helper)
        .arg("stage-delivery")
        .arg("--preparation")
        .arg(&report)
        .arg("--state")
        .arg(&state)
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(stage.status.success(), "{stage:?}");
    for (branch, reference) in [
        ("", ""),
        ("custom", ""),
        ("custom", "refs/heads/staging"),
        ("staging", "refs/heads/staging"),
        ("main", "refs/heads/main"),
        ("other", "refs/heads/other"),
    ] {
        assert!(!deliver(branch, reference).status.success());
        assert_eq!(
            git(remote.path(), &["rev-parse", "refs/heads/custom"]),
            before
        );
    }
    let output = deliver("custom", "refs/heads/custom");
    assert!(output.status.success(), "{output:?}");
    assert_ne!(
        git(remote.path(), &["rev-parse", "refs/heads/custom"]),
        before
    );
    assert_eq!(
        git(
            remote.path(),
            &["show", "refs/heads/custom:manifest/artifacts.lock.json"]
        ),
        "acquired"
    );
    assert_eq!(
        git(
            remote.path(),
            &["for-each-ref", "--format=%(refname)", "refs/heads/"]
        ),
        "refs/heads/custom"
    );
    assert!(fs::read_to_string(output_file)
        .unwrap()
        .contains("committed=true"));
    assert_eq!(
        git(remote.path(), &["rev-parse", "refs/tags/custom"]),
        before
    );
}
