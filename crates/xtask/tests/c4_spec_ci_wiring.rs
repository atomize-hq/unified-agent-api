use std::fs;
use std::path::PathBuf;

use regex::Regex;
use xtask::agent_maintenance::audit_status::{EXIT_INCOMPLETE_ACQUISITION, EXIT_UPLIFTS_REQUIRED};
use xtask::agent_maintenance::stand_down::{BASE_BRANCH, EXIT_STOOD_DOWN};

#[path = "c4_spec_ci_wiring/maintenance_audit_gate.rs"]
mod maintenance_audit_gate;

const GENERATED_PR_SUMMARY_SUFFIX: &str = "governance/pr-summary.md";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("CARGO_MANIFEST_DIR has crates/<crate> parent structure")
        .to_path_buf()
}

fn read_repo_file(relative_path: &str) -> String {
    let path = repo_root().join(relative_path);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

#[test]
fn c4_spec_agent_maintenance_workflows_share_the_release_watch_and_packet_only_pr_contract() {
    let shared_watch = read_repo_file(".github/workflows/agent-maintenance-release-watch.yml");
    let packet_pr = read_repo_file(".github/workflows/agent-maintenance-open-pr.yml");

    assert!(
        shared_watch.contains(
            "cargo run -p xtask -- maintenance-watch --emit-json _ci_tmp/maintenance-watch.json"
        ),
        "shared watcher must delegate stale detection to xtask"
    );
    assert!(
        shared_watch.contains(".stale_agents[]"),
        "shared watcher must fan out from stale_agents queue data"
    );
    assert!(
        !shared_watch.contains("listReleases"),
        "workflow yaml must not reimplement stale detection"
    );
    for required in [
        "concurrency:",
        "group: agent-maintenance-release-watch",
        "cancel-in-progress: false",
    ] {
        assert!(
            shared_watch.contains(required),
            "shared watcher must retain workflow concurrency guard {required}"
        );
    }
    for legacy in [
        ".github/workflows/codex-cli-release-watch.yml",
        ".github/workflows/claude-code-release-watch.yml",
    ] {
        assert!(
            !repo_root().join(legacy).exists(),
            "legacy watcher must be deleted: {legacy}"
        );
    }

    for required in [
        "prepare-agent-maintenance",
        "--current-version",
        "--latest-stable",
        "--target-version",
        "--opened-from",
        "--detected-by",
        "--dispatch-kind",
        "--branch-name",
        "base: staging",
        "add-paths: ${{ inputs.add_paths }}",
        "body-path: docs/agents/lifecycle/${{ inputs.agent_id }}-maintenance/governance/pr-summary.md",
        "concurrency:",
        "group: agent-maintenance-${{ inputs.branch_name }}",
        "cancel-in-progress: false",
        "continue-on-error: true",
        "steps.create_pr.outcome == 'failure'",
        "If PR creation fails after packet generation, rerun packet regeneration from the frozen request and reopen the PR from the generated pr-summary path.",
        "cargo run -p xtask -- refresh-agent --request \"${REQUEST_PATH}\" --write",
        "gh pr create --base staging --head \"${{ inputs.branch_name }}\"",
    ] {
        assert!(
            packet_pr.contains(required),
            "packet PR workflow must retain {required}"
        );
    }
    assert!(
        !packet_pr.contains("\n          body:"),
        "packet PR workflow must not keep an inline body block"
    );
    assert_prepare_step_precedes(
        &packet_pr,
        "prepare-agent-maintenance",
        "body-path: docs/agents/lifecycle/${{ inputs.agent_id }}-maintenance/governance/pr-summary.md",
        ".github/workflows/agent-maintenance-open-pr.yml",
    );

    // The packet-opening job itself still performs no acquisition work: acquisition happens in a
    // separate job that delegates to the reusable lane.
    for forbidden in [
        "actions/download-artifact@v7",
        "codex-snapshot",
        "claude-snapshot",
        "prepare-publication",
        "refresh-publication",
        "artifacts.lock.json",
        "_ci_tmp/codex_cli_pr_body.md",
    ] {
        assert!(
            !packet_pr.contains(forbidden),
            "packet PR workflow must not inline acquisition/generation work: {forbidden}"
        );
    }
}

#[test]
fn c4_spec_packet_pr_delegates_a_complete_union_to_the_reusable_acquisition_lane() {
    let packet_pr = read_repo_file(".github/workflows/agent-maintenance-open-pr.yml");

    for required in [
        // The gate is committed truth (registry enrollment + an `acquisition` block), enforced by
        // the planner, rather than a second per-agent enablement field.
        "manifest-acquisition-plan",
        "acquire=true",
        "acquire=false",
        // A build failure and a planner rejection must stay distinguishable: collapsing both into
        // `acquire=false` would silently downgrade every agent to the docs-only lane.
        "cargo build -p xtask",
        "gate_status=$?",
        "uses: ./.github/workflows/parity-acquire.yml",
        "ref: ${{ inputs.branch_name }}",
        "commit: true",
    ] {
        assert!(
            packet_pr.contains(required),
            "packet PR workflow must route acquisition through the reusable lane: {required}"
        );
    }

    assert!(
        packet_pr.contains("needs.open-pr.outputs.acquire == 'true'")
            && packet_pr.contains("needs.open-pr.outputs.branch_created == 'true'"),
        "acquisition must be gated on both the descriptor gate and a branch that actually exists"
    );
}

#[test]
fn c4_spec_reusable_acquisition_is_agent_parameterized_with_no_per_agent_branching() {
    let yml = read_repo_file(".github/workflows/parity-acquire.yml");

    for required in [
        "workflow_call:",
        "workflow_dispatch:",
        "agent_id:",
        "target_version:",
        "cargo run -p xtask -- manifest-acquisition-plan",
        "--emit-json _ci_tmp/acquisition-plan.json",
        "fromJSON(needs.plan.outputs.snapshot_matrix)",
        "runs-on: ${{ matrix.runs_on }}",
        // Engine invocations are read from the plan, not hardcoded per agent.
        "UNION_COMMAND=",
        "SNAPSHOT_COMMAND=",
        "manifest-report --root",
        "manifest-version-metadata --root",
        "--status reported",
        "manifest-validate --root",
    ] {
        assert!(
            yml.contains(required),
            "reusable acquisition workflow must retain {required}"
        );
    }

    // Both acquisition source kinds and all three archive shapes must be handled generically.
    for required in ["github_releases)", "npm)", "none)", "tar_gz)", "npm_tgz)"] {
        assert!(
            yml.contains(required),
            "reusable acquisition workflow must handle {required}"
        );
    }

    // Integrity: pins are re-verified on the target runner before the binary is executed.
    assert!(
        yml.contains("sha256 mismatch") && yml.contains("size mismatch"),
        "reusable acquisition must re-verify the download pin on the snapshotting runner"
    );

    // No agent may be named in a branching construct: the whole point is one lane for all agents.
    for forbidden in [
        "cli_manifests/codex",
        "cli_manifests/claude_code",
        "cli_manifests/opencode",
        "openai/codex",
        "storage.googleapis",
        "codex-union",
        "claude-union",
        "codex-report",
        "codex-validate",
        "codex-version-metadata",
    ] {
        assert!(
            !yml.contains(forbidden),
            "reusable acquisition workflow must stay agent-agnostic: {forbidden}"
        );
    }
}

#[test]
fn c4_spec_reusable_acquisition_retries_snapshot_capture_once_in_place() {
    let workflow = ".github/workflows/parity-acquire.yml";
    let yml = read_repo_file(workflow);
    let snapshot_section = section_between(
        &yml,
        "- name: Generate the per-target snapshot",
        "- name: Upload the per-target snapshot",
        workflow,
    );

    for required in [
        "ATTEMPT=1",
        "while [ \"$ATTEMPT\" -le 2 ]; do",
        "cargo run -p xtask -- \"${ARGS[@]}\"",
        "SNAPSHOT_STATUS=$?",
        "if [ \"$ATTEMPT\" -ge 2 ]; then",
        "exit \"$SNAPSHOT_STATUS\"",
        "::warning title=Snapshot capture retry::${TARGET} snapshot capture failed on attempt ${ATTEMPT}; retrying once.",
        "ATTEMPT=$((ATTEMPT + 1))",
    ] {
        assert!(
            snapshot_section.contains(required),
            "snapshot capture must implement a single in-place retry: {required}"
        );
    }

    assert!(
        !snapshot_section.contains("[ \"$ATTEMPT\" -le 3 ]")
            && !snapshot_section.contains("[ \"$ATTEMPT\" -lt 3 ]"),
        "snapshot capture must not retry more than once"
    );
}

#[test]
fn c4_spec_reusable_promotion_validates_every_promoted_target_before_advancing_pointers() {
    let yml = read_repo_file(".github/workflows/parity-promote.yml");

    for required in [
        "workflow_call:",
        "workflow_dispatch:",
        "validation_matrix",
        "fromJSON(needs.plan.outputs.validation_matrix)",
        "runs-on: ${{ matrix.runs_on }}",
        "needs: [plan, validate-target]",
        // Only targets present in the committed union may be validated and promoted.
        "jq -r '.inputs[].target_triple'",
        "VALIDATION_ARGS+=(--passed-target \"$target\")",
        "--status validated",
        "pointers/latest_validated/${REQUIRED_TARGET}.txt",
        "manifest-validate --root",
        // Validation commands and env come from the descriptor, not from this file.
        "jq -r '.validation_commands[]'",
        "{scratch_dir}",
    ] {
        assert!(
            yml.contains(required),
            "reusable promotion workflow must retain {required}"
        );
    }

    assert!(
        yml.contains("default: true") && yml.contains("if: ${{ !inputs.dry_run }}"),
        "promotion must default to dry-run and only open a PR when explicitly asked"
    );

    for forbidden in [
        "cli_manifests/codex",
        "cli_manifests/claude_code",
        "cargo test -p unified-agent-api-codex",
        "cargo test -p unified-agent-api-claude-code",
        "x86_64-unknown-linux-musl",
    ] {
        assert!(
            !yml.contains(forbidden),
            "reusable promotion workflow must stay agent-agnostic: {forbidden}"
        );
    }
}

#[test]
fn c4_spec_legacy_per_agent_parity_workflows_are_retired() {
    for legacy in [
        ".github/workflows/codex-cli-update-snapshot.yml",
        ".github/workflows/claude-code-update-snapshot.yml",
        ".github/workflows/codex-cli-promote.yml",
        ".github/workflows/claude-code-promote.yml",
    ] {
        assert!(
            !repo_root().join(legacy).exists(),
            "legacy per-agent parity workflow must be deleted: {legacy}"
        );
    }

    let workflows = fs::read_dir(repo_root().join(".github/workflows"))
        .expect("read .github/workflows")
        .map(|entry| {
            entry
                .expect("dir entry")
                .file_name()
                .to_string_lossy()
                .to_string()
        })
        .collect::<Vec<_>>();

    for name in &workflows {
        assert!(
            !(name.contains("update-snapshot") || name.contains("promote"))
                || name.starts_with("parity-"),
            "acquisition/promotion must live only in the reusable parity-* lanes, found {name}"
        );
    }
}

#[test]
fn c4_spec_ci_workflow_has_conditional_manifest_validate_gate() {
    let yml = read_repo_file(".github/workflows/ci.yml");

    // C4-spec (normative): gate runs only when committed artifacts regime is active.
    //
    // Two supported implementations:
    // - job-level hashFiles gate
    // - a first step that detects committed versions and gates subsequent steps via outputs
    let has_hashfiles_gate = yml.contains("hashFiles('cli_manifests/codex/versions/*.json') != ''");
    let has_step_gate = yml.contains("Detect Codex committed artifacts")
        && yml.contains("has_versions")
        && yml.contains("steps.codex-artifacts.outputs.has_versions");
    assert!(
        has_hashfiles_gate || has_step_gate,
        "ci.yml must gate manifest validation behind either hashFiles('cli_manifests/codex/versions/*.json') != '' or a Detect Codex committed artifacts step gate"
    );

    // Ensure the job actually runs the validator (not just mentions it). Both the neutral name and
    // the retained back-compat alias are acceptable.
    let validate_invocation = Regex::new(
        r"cargo\s+run\s+-p\s+xtask\s+--[\s\\]*\n?[\s\\]*(manifest-validate|codex-validate)",
    )
    .expect("valid regex");
    assert!(
        validate_invocation.is_match(&yml),
        "ci.yml must invoke: cargo run -p xtask -- manifest-validate"
    );
}

#[test]
fn c4_spec_acquisition_commits_the_paths_support_matrix_actually_publishes() {
    let yml = read_repo_file(".github/workflows/parity-acquire.yml");

    // `git add` fails the entire commit on a pathspec that matches nothing, so a wrong path here
    // does not degrade gracefully: it throws away a full acquisition run — every download,
    // snapshot, union and report — at the very last step. Bind the workflow to the constants the
    // publisher actually writes so renaming one without updating the other fails here instead.
    for published in [
        xtask::support_matrix::JSON_OUTPUT_PATH,
        xtask::support_matrix::MARKDOWN_OUTPUT_PATH,
    ] {
        assert!(
            yml.contains(published),
            "parity-acquire must commit the support publication path `{published}`"
        );
    }

    assert!(
        !yml.contains("docs/support\n") && !yml.contains("docs/support "),
        "`docs/support` is not a path this repository has; it fails `git add` and discards the run"
    );
}

#[test]
fn c4_spec_process_substitution_loops_strip_cr_for_the_windows_runners() {
    // Git Bash backs process substitution with a temp file subject to text-mode translation, so
    // `while read ... done < <(...)` yields a trailing CR on the Windows runners while `$(...)`
    // captures do not. A CR is never part of a legitimate CLI flag, target triple, env var name
    // or shell command, so every such loop in a workflow with a Windows matrix must strip it.
    //
    // This is not hypothetical: it took the whole win32-x64 leg of parity-acquire down with
    // `unexpected argument '--help-timeout-ms\r'`, and parity-promote fed the same shape into
    // `eval`.
    for workflow in [
        ".github/workflows/parity-acquire.yml",
        ".github/workflows/parity-promote.yml",
        ".github/workflows/ci.yml",
    ] {
        let yml = read_repo_file(workflow);
        for (index, line) in yml.lines().enumerate() {
            if !line.contains("done < <(") {
                continue;
            }
            assert!(
                line.contains(r"tr -d '\r'"),
                "{workflow}:{} reads through process substitution without stripping CR, which \
                 corrupts every value on the Windows runners: {}",
                index + 1,
                line.trim()
            );
        }
    }
}

#[test]
fn c4_spec_ci_pins_the_latest_validated_binary_from_the_lockfile_row_not_the_descriptor() {
    let yml = read_repo_file(".github/workflows/ci.yml");

    // The descriptor answers "how would a new version be acquired"; the lockfile row answers
    // "what was actually pinned for this one". Those diverge across a distribution migration:
    // claude_code 2.1.29 is pinned as a bare binary from the old bucket while the descriptor
    // resolves an npm platform tarball. Selecting the row by the descriptor's asset name matches
    // nothing and fails the job, so the row must be selected by (version, target) alone.
    let selects_row_by_version_and_target =
        yml.contains(r#"select(.[$key]==$v and .target_triple==$t)"#);
    assert!(
        selects_row_by_version_and_target,
        "ci.yml must select the claude_code lockfile row by (version, target) only"
    );

    assert!(
        !yml.contains(r#"select(.[$key]==$v and .target_triple==$t and .asset_name==$a)"#),
        "ci.yml must not constrain the lockfile row on the descriptor's asset name: a version \
         pinned before the acquisition descriptor existed has no row under that name"
    );

    // The pinned asset names its own shape. Inferring it from the descriptor's `archive` would
    // try to untar a bare binary (or execute a tarball) depending on migration direction.
    assert!(
        yml.contains("*.tgz|*.tar.gz)"),
        "ci.yml must infer archive shape from the pinned asset name"
    );

    // Whatever the shape, the binary must never be executed before the autoupdater is disabled:
    // a self-update between the digest check and the run would make the verified pin a lie.
    let acquire_step = yml
        .split("- name: Acquire Claude Code CLI (required target)")
        .nth(1)
        .expect("ci.yml must retain the claude_code acquisition step");
    let env_export = acquire_step
        .find(".snapshot.env[]?")
        .expect("acquisition step must apply the descriptor's snapshot env");
    let version_probe = acquire_step
        .find(r#""$BIN" --version"#)
        .expect("acquisition step must smoke the acquired binary");
    assert!(
        env_export < version_probe,
        "ci.yml must export the descriptor's snapshot env before executing the binary"
    );
}

fn assert_prepare_step_precedes(
    workflow_text: &str,
    prepare_needle: &str,
    body_path_needle: &str,
    workflow: &str,
) {
    let prepare_index = workflow_text
        .find(prepare_needle)
        .unwrap_or_else(|| panic!("{workflow} must contain {prepare_needle}"));
    let body_path_index = workflow_text
        .find(body_path_needle)
        .unwrap_or_else(|| panic!("{workflow} must contain {body_path_needle}"));
    assert!(
        prepare_index < body_path_index,
        "{workflow} must render the maintenance packet before referencing {GENERATED_PR_SUMMARY_SUFFIX}"
    );
}

fn assert_text_order(workflow_text: &str, first: &str, second: &str, workflow: &str) {
    let first_index = workflow_text
        .find(first)
        .unwrap_or_else(|| panic!("{workflow} must contain {first}"));
    let second_index = workflow_text
        .find(second)
        .unwrap_or_else(|| panic!("{workflow} must contain {second}"));
    assert!(
        first_index < second_index,
        "{workflow} must place `{first}` before `{second}`"
    );
}

fn section_between<'a>(workflow_text: &'a str, start: &str, end: &str, workflow: &str) -> &'a str {
    let start_index = workflow_text
        .find(start)
        .unwrap_or_else(|| panic!("{workflow} must contain {start}"));
    let section = &workflow_text[start_index..];
    let end_index = section
        .find(end)
        .unwrap_or_else(|| panic!("{workflow} must contain {end} after {start}"));
    &section[..end_index]
}

fn section_from<'a>(workflow_text: &'a str, start: &str, workflow: &str) -> &'a str {
    let start_index = workflow_text
        .find(start)
        .unwrap_or_else(|| panic!("{workflow} must contain {start}"));
    &workflow_text[start_index..]
}

fn nonfatal_shell_branch_violations(branch: &str) -> Vec<String> {
    branch
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let trimmed = line.trim_start();
            if shell_statement_starts_with(trimmed, "exit")
                || shell_statement_starts_with(trimmed, "false")
            {
                Some(format!("line {}: {}", index + 1, trimmed))
            } else {
                None
            }
        })
        .collect()
}

fn shell_statement_starts_with(line: &str, keyword: &str) -> bool {
    if line.is_empty() || line.starts_with('#') {
        return false;
    }
    let Some(rest) = line.strip_prefix(keyword) else {
        return false;
    };
    rest.is_empty()
        || matches!(
            rest.chars().next(),
            Some(' ' | '\t' | ';' | '#' | '\n' | '\r')
        )
}

#[test]
fn backend_type_leak_guard_is_centralized_in_ci_and_smoke_workflows() {
    let guard_invocation = Regex::new(
        r"cargo\s+run\s+-p\s+xtask\s+--[\s\\]*\n?[\s\\]*agent-api-backend-type-leak-guard",
    )
    .expect("valid regex");
    for workflow in [
        ".github/workflows/ci.yml",
        ".github/workflows/unified-agent-api-smoke.yml",
        ".github/workflows/agent-api-codex-stream-exec-smoke.yml",
        ".github/workflows/claude-code-live-stream-json-smoke.yml",
    ] {
        let yml = read_repo_file(workflow);
        assert!(
            guard_invocation.is_match(&yml),
            "{workflow} must invoke cargo run -p xtask -- agent-api-backend-type-leak-guard"
        );
        assert!(
            !yml.contains("(?:codex|claude_code)::"),
            "{workflow} must not keep the stale inline backend regex guard"
        );
    }
}

/// Strip comment lines before asserting on command order.
///
/// These workflow steps carry their rationale inline, so a rule like "the fetch precedes the read"
/// is otherwise an assertion about where the prose mentioning it sits — which breaks on an edit
/// that changes nothing, and then gets deleted rather than fixed.
fn without_comments(section: &str) -> String {
    section
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

/// `uaa-0048`: one early check is a semantic goal, not enforcement. An invocation already queued
/// passes an earlier check and then overwrites later maintainer work, and `workflow_dispatch` and
/// re-runs bypass the queue entirely — so the question is re-asked immediately before every step
/// that can destroy a packet generation, and each destructive step is gated on its own answer.
#[test]
fn c4_spec_every_destructive_packet_step_is_gated_on_a_fresh_stand_down_check() {
    let packet_pr = read_repo_file(".github/workflows/agent-maintenance-open-pr.yml");

    for (boundary, guard_id, mutation, gate) in [
        (
            "request regeneration",
            "id: stand_down\n",
            "name: Prepare maintenance packet",
            "steps.stand_down.outputs.stand_down != 'true'",
        ),
        (
            "branch and packet-body replacement",
            "id: stand_down_recheck\n",
            "name: Create PR",
            "steps.stand_down_recheck.outputs.stand_down != 'true'",
        ),
    ] {
        assert!(
            packet_pr.contains(guard_id),
            "{boundary} needs its own stand-down check step ({guard_id})"
        );
        assert!(
            packet_pr.contains(gate),
            "{boundary} must be gated on that check: {gate}"
        );
        assert_text_order(
            &packet_pr,
            guard_id,
            mutation,
            "agent-maintenance-open-pr.yml",
        );
    }

    let regenerate = section_between(
        &packet_pr,
        "- name: Check packet stand-down before regenerating the request",
        "- name: Prepare maintenance packet",
        "agent-maintenance-open-pr.yml",
    );
    let recheck = section_between(
        &packet_pr,
        "- name: Check packet stand-down before replacing the branch",
        "- name: Create PR",
        "agent-maintenance-open-pr.yml",
    );

    // The re-check must not itself be conditioned on the first one. A skipped step has an empty
    // output, and `!= 'true'` on an empty output reads as permission — which would turn the first
    // stand-down into an authorization for the force-push it exists to block.
    // A line-level check, not a substring one: prose in this step mentions `if:`, and an assertion
    // that a comment can break is an assertion that gets deleted rather than fixed.
    assert!(
        !recheck
            .lines()
            .any(|line| line.trim_start().starts_with("if:")),
        "the pre-replacement re-check must run unconditionally, or a skipped step's empty output \
         reads as permission"
    );

    // Supersession is the one boundary that acts on a packet other than this run's: a newer
    // version opens a different branch, so its run is in a different concurrency group and can
    // close an older packet mid-closeout. It therefore asks about the candidate, not the run —
    // and has to act on the answer, which is a separate thing to assert.
    let supersede = section_from(
        &packet_pr,
        "- name: Close superseded same-agent PRs",
        "agent-maintenance-open-pr.yml",
    );
    assert!(
        supersede.contains("maintenance-stand-down-check"),
        "supersession must ask before closing an older packet PR"
    );
    assert!(
        supersede.contains("--target-version \"$candidate_version\""),
        "supersession must ask about the candidate's version, not this run's"
    );
    assert!(
        supersede.contains(r#"candidate_status" -ne 0"#),
        "supersession must branch on the guard's exit status, not merely invoke it"
    );
    let refusal = section_between(
        supersede,
        r#"candidate_status" -ne 0"#,
        "superseded_prs+=",
        "agent-maintenance-open-pr.yml",
    );
    assert!(
        without_comments(refusal).contains("continue"),
        "supersession must skip a frozen candidate, not merely report it: the loop already has \
         unrelated `continue`s, so this has to be the guard's own branch"
    );

    // `uaa-0050`: every boundary reads a freshly fetched base, not the job-start checkout.
    //
    // A checkout is a snapshot of when the job began, and all three boundaries act later than
    // that: the first one because a re-run replays the original event's `GITHUB_SHA`, the second
    // because `create-pull-request` resets the branch to *current* base, and the third because it
    // runs after the PR has been pushed. The window each guard claims to close is the job's whole
    // duration unless it fetches, so reading the working tree is the mistake this rules out.
    for (boundary, section) in [
        ("request regeneration", regenerate),
        ("branch and packet-body replacement", recheck),
        ("supersession", supersede),
    ] {
        let commands = without_comments(section);
        assert!(
            commands.contains("git fetch") && commands.contains("--from-ref"),
            "{boundary} must read a freshly fetched base, not the job-start checkout"
        );
        assert_text_order(
            &commands,
            "git fetch",
            "--from-ref",
            "agent-maintenance-open-pr.yml",
        );
    }
}

/// The marker only protects anything if it is committed where the guard looks, so the branch named
/// in the instruction and the branch the packet PR targets are one fact, not two.
///
/// A rename that moved the workflow without the constant would send every declared freeze to a
/// branch nothing reads — a false authorization wearing the shape of a declaration, which is the
/// one direction this guard may never fail in.
#[test]
fn c4_spec_the_stand_down_base_branch_is_the_branch_the_packet_pr_targets() {
    let packet_pr = read_repo_file(".github/workflows/agent-maintenance-open-pr.yml");
    let create_pr = section_from(
        &packet_pr,
        "- name: Create PR",
        "agent-maintenance-open-pr.yml",
    );
    assert!(
        create_pr.contains(&format!("base: {BASE_BRANCH}")),
        "the packet PR must target the branch markers are committed to ({BASE_BRANCH})"
    );
}

/// `uaa-0050`: a marker is retired by the promotion of its own version, and by nothing else.
///
/// Promotion is the one moment that means the packet is finished, it is maintainer-merged, and it
/// names exactly one version. Tying cleanup to a *new version* instead would delete the marker of
/// the packet still being worked on, because upstream cadence says nothing about whether the
/// previous closeout completed.
#[test]
fn c4_spec_a_stand_down_marker_retires_with_its_own_promotion() {
    let promote = read_repo_file(".github/workflows/parity-promote.yml");
    let advance = section_between(
        &promote,
        "- name: Advance pointers and version metadata",
        "- name: Refresh support publication",
        "parity-promote.yml",
    );
    let commands = without_comments(advance);

    assert!(
        commands.contains("automation-stand-down/${VERSION}.toml"),
        "the pointer-advance step must retire the marker for the version being promoted"
    );
    assert!(
        commands.contains("rm -f"),
        "retirement has to remove the marker, not merely name it"
    );
    // Same commit as the `reported -> validated` flip: a failed cleanup then leaves an extra
    // marker, and a failed promotion never leaves a missing one.
    assert!(
        commands.contains("--status validated"),
        "retirement must ride with the status flip, which is what makes it one commit"
    );

    // The removal reaches the PR only because this step has no `add-paths` to exclude it.
    let open_pr = section_from(
        &promote,
        "- name: Open the promotion PR",
        "parity-promote.yml",
    );
    assert!(
        !open_pr.contains("add-paths"),
        "an `add-paths` list on the promotion PR would silently drop the marker deletion"
    );
    assert!(
        section_from(&promote, "jobs:", "parity-promote.yml").contains("contents: write"),
        "the promote job needs write access to commit the retirement"
    );
}

/// The guard's one forbidden outcome is a false authorization, so every way of not getting a clean
/// answer has to land on "stand down" — and, because a stand-down skips everything downstream, a
/// guard that could not answer has to be distinguishable from one that answered "frozen".
#[test]
fn c4_spec_a_stand_down_check_that_cannot_answer_does_not_authorize() {
    let packet_pr = read_repo_file(".github/workflows/agent-maintenance-open-pr.yml");

    assert_eq!(
        EXIT_STOOD_DOWN, 3,
        "the workflow routes the declared stand-down by literal exit code"
    );
    assert!(
        packet_pr.contains("cargo build -p xtask"),
        "the guard builds xtask separately so a compile failure cannot read as `no marker`"
    );

    for guard_id in ["id: stand_down\n", "id: stand_down_recheck\n"] {
        let guard = section_from(&packet_pr, guard_id, "agent-maintenance-open-pr.yml");
        let routing = guard
            .split_once("- name: ")
            .map(|(head, _)| head)
            .unwrap_or(guard);

        // Counting occurrences is not enough: swapping the two branch bodies leaves every count
        // identical and inverts the guard, so bind the authorization to the exit-0 arm itself.
        let authorized_arm = section_between(
            routing,
            r#"$status" -eq 0"#,
            r#"$status" -eq 3"#,
            "agent-maintenance-open-pr.yml",
        );
        assert!(
            authorized_arm.contains("stand_down=false")
                && !authorized_arm.contains("stand_down=true"),
            "{guard_id} must authorize on exit 0 and only on exit 0"
        );
        assert_eq!(
            routing.matches("stand_down=false").count(),
            1,
            "{guard_id} must have exactly one way to authorize"
        );
        assert!(
            routing.matches("stand_down=true").count() >= 2,
            "{guard_id} must stand down both for the declared marker and for a guard that could \
             not answer, so an unanswerable guard never authorizes"
        );

        // `::error` annotates but does not fail a step. Without the explicit exit, a guard that
        // cannot answer leaves the job green with no PR, and a total outage looks like a quiet
        // night.
        let unanswerable_arm = routing
            .rsplit_once("else")
            .map(|(_, tail)| tail)
            .unwrap_or_default();
        assert!(
            unanswerable_arm.contains("exit 1"),
            "{guard_id} must fail the job when the guard could not answer, so the outage is visible"
        );
    }
}
