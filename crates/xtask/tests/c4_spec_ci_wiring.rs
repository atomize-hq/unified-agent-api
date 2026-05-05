use std::fs;
use std::path::PathBuf;

use regex::Regex;

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
    let packet_only_pr = read_repo_file(".github/workflows/agent-maintenance-open-pr.yml");

    assert!(
        shared_watch.contains("cargo run -p xtask -- maintenance-watch --emit-json _ci_tmp/maintenance-watch.json"),
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
    ] {
        assert!(
            packet_only_pr.contains(required),
            "packet-only PR workflow must retain {required}"
        );
    }
    for forbidden in [
        "actions/download-artifact@v7",
        "codex-snapshot",
        "claude-snapshot",
        "prepare-publication",
        "refresh-publication",
        "artifacts.lock.json",
    ] {
        assert!(
            !packet_only_pr.contains(forbidden),
            "packet-only PR workflow must not perform acquisition/generation work: {forbidden}"
        );
    }
}

#[test]
fn c4_spec_update_snapshot_workflow_runs_full_pipeline_and_uploads_artifacts() {
    let yml = read_repo_file(".github/workflows/codex-cli-update-snapshot.yml");

    // C4-spec: acquire pinned upstream binaries using artifacts.lock + RULES.json expected targets.
    assert!(
        yml.contains("cli_manifests/codex/artifacts.lock.json"),
        "workflow must reference cli_manifests/codex/artifacts.lock.json to acquire pinned binaries"
    );
    assert!(
        yml.contains("cli_manifests/codex/RULES.json"),
        "workflow must reference cli_manifests/codex/RULES.json (for union.expected_targets contract)"
    );
    assert!(
        yml.contains("expected_targets"),
        "workflow must reference RULES.json union.expected_targets (expected_targets)"
    );

    // C4-spec: per-target snapshots should run on Linux/macOS/Windows.
    assert!(
        yml.contains("ubuntu-"),
        "workflow must include at least one ubuntu runs-on job (Linux snapshots + union stage)"
    );
    assert!(
        yml.contains("macos-"),
        "workflow must include at least one macos runs-on job (macOS snapshots)"
    );
    assert!(
        yml.contains("windows-"),
        "workflow must include at least one windows runs-on job (Windows snapshots)"
    );

    // C4-spec: generate per-target snapshots + raw help captures and upload raw help as CI artifacts.
    assert!(
        yml.contains("codex-snapshot"),
        "workflow must run xtask codex-snapshot"
    );
    assert!(
        yml.contains("cli_manifests/codex/raw_help/"),
        "workflow must capture/upload raw help under cli_manifests/codex/raw_help/<version>/<target_triple>/"
    );
    let upload_artifact_invocation =
        Regex::new(r"actions/upload-artifact@v[0-9]+").expect("valid regex");
    assert!(
        upload_artifact_invocation.is_match(&yml),
        "workflow must upload raw help and artifact bundles via actions/upload-artifact"
    );

    // C4-spec: on Linux, run union → wrapper-coverage → report → version-metadata → validate.
    for required in [
        "codex-union",
        "codex-wrapper-coverage",
        "codex-report",
        "codex-version-metadata",
        "codex-validate",
    ] {
        assert!(
            yml.contains(required),
            "workflow must run xtask {required} as part of the end-to-end pipeline"
        );
    }

    // C4-spec: upload artifact bundle containing snapshots/reports/versions + wrapper coverage.
    for required_path in [
        "cli_manifests/codex/snapshots/",
        "cli_manifests/codex/reports/",
        "cli_manifests/codex/versions/",
        "cli_manifests/codex/wrapper_coverage.json",
    ] {
        assert!(
            yml.contains(required_path),
            "workflow must upload committed-artifact bundle including {required_path}"
        );
    }
}

#[test]
fn c4_spec_ci_workflow_has_conditional_codex_validate_gate() {
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
        "ci.yml must gate codex-validate behind either hashFiles('cli_manifests/codex/versions/*.json') != '' or a Detect Codex committed artifacts step gate"
    );

    // Ensure the job actually runs codex-validate (not just mentions it).
    let validate_invocation =
        Regex::new(r"cargo\s+run\s+-p\s+xtask\s+--[\s\\]*\n?[\s\\]*codex-validate")
            .expect("valid regex");
    assert!(
        validate_invocation.is_match(&yml),
        "ci.yml must invoke: cargo run -p xtask -- codex-validate"
    );
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
