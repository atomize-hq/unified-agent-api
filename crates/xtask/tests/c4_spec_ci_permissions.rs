//! Contract: no job that runs a downloaded upstream binary may hold a credential that can write
//! to this repository.
//!
//! `actions/checkout` defaults `persist-credentials` to true, which writes the job token into
//! `.git/config` as an auth header and clears it in post-job cleanup — after the binary has run.
//! `parity-acquire`'s `snapshot` job and `parity-promote`'s `validate-target` job both execute the
//! pinned upstream CLI, so both must check out without a credential and hold `contents: read`.
//!
//! These workflows carry only `workflow_call` and `workflow_dispatch` triggers, so editing them
//! runs nothing (plan doc §17.2). This file is the only guard on the permission shape.

use std::fs;
use std::path::PathBuf;

const ACQUIRE: &str = ".github/workflows/parity-acquire.yml";
const PROMOTE: &str = ".github/workflows/parity-promote.yml";

fn read_repo_file(relative_path: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("CARGO_MANIFEST_DIR has crates/<crate> parent structure")
        .join(relative_path);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

/// Job names, in file order. A job header is the only two-space-indented key under `jobs:`.
fn job_names(workflow: &str) -> Vec<String> {
    let body = workflow
        .split_once("\njobs:\n")
        .unwrap_or_else(|| panic!("workflow has no `jobs:` block"))
        .1;
    body.lines().filter_map(job_header).collect()
}

/// `Some(name)` when the line is a two-space-indented job header under `jobs:`.
fn job_header(line: &str) -> Option<String> {
    let rest = line.trim_end().strip_prefix("  ")?;
    if rest.starts_with(char::is_whitespace) || rest.starts_with('#') {
        return None;
    }
    rest.strip_suffix(':').map(str::to_string)
}

/// The text of one job, from its header to the next job header or the end of the file.
fn job_block(workflow: &str, job: &str) -> String {
    let header = format!("\n  {job}:\n");
    let start = workflow
        .find(&header)
        .unwrap_or_else(|| panic!("workflow has no job `{job}`"));
    let mut out = String::new();
    for line in workflow[start + 1..].lines() {
        if !out.is_empty() && job_header(line).is_some() {
            break;
        }
        out.push('\n');
        out.push_str(line);
    }
    out
}

fn assert_contains(block: &str, needle: &str, job: &str, why: &str) {
    assert!(
        block.contains(needle),
        "job `{job}` must contain `{}`: {why}",
        needle.trim()
    );
}

#[test]
fn c4_spec_parity_workflows_default_to_read_only() {
    for workflow in [ACQUIRE, PROMOTE] {
        let yml = read_repo_file(workflow);
        assert!(
            yml.contains("\npermissions:\n  contents: read\n"),
            "{workflow} must default to `contents: read` so a job that omits its own \
             `permissions` block cannot inherit write"
        );
        assert!(
            !yml.contains("\npermissions:\n  contents: write"),
            "{workflow} must not grant write at workflow level"
        );
    }
}

#[test]
fn c4_spec_every_parity_job_declares_its_own_permissions() {
    // A new job must make an explicit decision rather than inherit the file default silently.
    for workflow in [ACQUIRE, PROMOTE] {
        let yml = read_repo_file(workflow);
        for job in job_names(&yml) {
            let block = job_block(&yml, &job);
            assert!(
                block.contains("\n    permissions:\n"),
                "{workflow} job `{job}` must declare its own `permissions:` block"
            );
        }
    }
}

#[test]
fn c4_spec_jobs_that_run_upstream_binaries_hold_no_write_credential() {
    // (workflow, job, why it runs upstream code)
    let untrusted = [
        (
            ACQUIRE,
            "plan",
            "downloads and hashes upstream release assets",
        ),
        (
            ACQUIRE,
            "snapshot",
            "executes the pinned upstream CLI for the help crawl",
        ),
        (PROMOTE, "plan", "reads committed union inputs only"),
        (
            PROMOTE,
            "validate-target",
            "executes the pinned upstream CLI and this agent's declared validation commands",
        ),
    ];

    for (workflow, job, why) in untrusted {
        let block = job_block(&read_repo_file(workflow), job);
        assert_contains(
            &block,
            "\n    permissions:\n      contents: read\n",
            job,
            why,
        );
        assert!(
            !block.contains("contents: write"),
            "job `{job}` must not hold `contents: write`: {why}"
        );
        assert!(
            !block.contains("pull-requests: write"),
            "job `{job}` must not hold `pull-requests: write`: {why}"
        );
        assert_contains(&block, "\n          persist-credentials: false\n", job, why);
        assert!(
            !block.contains("\n          token: "),
            "job `{job}` must not check out with an explicit token: {why}"
        );
    }
}

#[test]
fn c4_spec_only_the_publishing_jobs_hold_write() {
    let acquire = read_repo_file(ACQUIRE);
    let union = job_block(&acquire, "union");
    assert!(
        union.contains("\n    permissions:\n      contents: write\n"),
        "parity-acquire's `union` job commits the acquired artifacts, so it needs \
         `contents: write` at job level once the workflow default is read-only"
    );

    let promote_yml = read_repo_file(PROMOTE);
    let promote = job_block(&promote_yml, "promote");
    assert!(
        promote.contains("\n    permissions:\n      contents: write\n      pull-requests: write\n"),
        "parity-promote's `promote` job opens the promotion PR, so it needs both write scopes \
         at job level once the workflow default is read-only"
    );

    // Exactly one writer per workflow.
    for (workflow, writer) in [(ACQUIRE, "union"), (PROMOTE, "promote")] {
        let yml = read_repo_file(workflow);
        let writers: Vec<String> = job_names(&yml)
            .into_iter()
            .filter(|job| job_block(&yml, job).contains("      contents: write"))
            .collect();
        assert_eq!(
            writers,
            vec![writer.to_string()],
            "{workflow} must grant `contents: write` to `{writer}` alone"
        );
    }
}
