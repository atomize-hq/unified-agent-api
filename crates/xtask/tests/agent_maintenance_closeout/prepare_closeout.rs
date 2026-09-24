//! T6 — `prepare-agent-closeout`.
//!
//! The acceptance condition is a single sentence: a generated artifact passes
//! `close-agent-maintenance` unmodified. Every test here drives the real command through the real
//! validator on a real file, because a generator checked against its own in-memory model proves
//! only that it agrees with itself.
//!
//! Evidence is injected through the `_with_fetcher` seam rather than reaching GitHub, so the
//! refusal paths — red CI, an unreachable commit, an unadjudicated wrapper-only row — are
//! exercisable without a network or a packet.

use std::path::Path;

use serde_json::{json, Value};

use crate::{
    closeout_harness::{automated_maintenance_request_toml, init_git_fixture},
    harness::{fixture_root, write_text},
    load_linked_closeout, seed_opencode_basis,
};

const REQUEST_PATH: &str =
    "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
const CLOSEOUT_PATH: &str =
    "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json";
const BASIS_REF: &str = "docs/integrations/opencode/governance/seam-2-closeout.md";
const BASELINE_REF: &str = "cli_manifests/opencode/reports/1.14.47/coverage.any.json";
const RECORDED_AT: &str = "2026-09-24T12:00:00Z";

/// One page of workflow runs whose `head_sha` is `commit`, all agreeing on `conclusion`.
fn runs_page(commit: &str, conclusion: &str) -> String {
    json!({
        "workflow_runs": [{
            "id": 42_u64,
            "run_attempt": 1_u64,
            "head_sha": commit,
            "status": "completed",
            "conclusion": conclusion,
            "path": ".github/workflows/ci.yml",
        }]
    })
    .to_string()
}

fn seed_packet(name: &str, wrapper_only: &[Value]) -> (std::path::PathBuf, String) {
    let fixture = fixture_root(name);
    seed_opencode_basis(&fixture);
    write_text(
        &fixture.join(".github/workflows/agent-maintenance-open-pr.yml"),
        "name: Packet PR worker\n",
    );
    set_wrapper_only_flags(&fixture, wrapper_only);
    write_text(
        &fixture.join(REQUEST_PATH),
        &automated_maintenance_request_toml("opencode", BASIS_REF),
    );
    let head = init_git_fixture(&fixture);
    (fixture, head)
}

fn set_wrapper_only_flags(fixture: &Path, rows: &[Value]) {
    let path = fixture.join(BASELINE_REF);
    let mut report: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("read seeded report"))
            .expect("parse seeded report");
    if rows.is_empty() {
        report["deltas"]
            .as_object_mut()
            .expect("deltas object")
            .remove("wrapper_only_flags");
    } else {
        report["deltas"]["wrapper_only_flags"] = json!(rows);
    }
    write_text(&path, &report.to_string());
}

fn status_json_flag() -> Value {
    json!({ "path": ["status"], "key": "--json", "wrapper_level": "explicit" })
}

fn prepare(fixture: &Path, commit: &str, conclusion: &str, write: bool) -> Result<String, String> {
    let args = crate::prepare_closeout::Args {
        request: Path::new(REQUEST_PATH).to_path_buf(),
        commit: commit.to_string(),
        recorded_at: RECORDED_AT.to_string(),
        write,
    };
    let mut out = Vec::new();
    let page = runs_page(commit, conclusion);
    crate::prepare_closeout::run_in_workspace_with_fetcher(fixture, args, &mut out, |_url| {
        Ok(page.clone())
    })
    .map(|()| String::from_utf8(out).expect("stdout is utf8"))
    .map_err(|err| err.to_string())
}

/// The acceptance condition. Nothing else in this file matters if this fails.
#[test]
fn a_generated_artifact_passes_the_real_validator_unmodified() {
    let (fixture, head) = seed_packet("t6-accept", &[]);

    prepare(&fixture, &head, "success", true).expect("generation succeeds");

    load_linked_closeout(&fixture, Path::new(REQUEST_PATH), Path::new(CLOSEOUT_PATH))
        .expect("the generated artifact validates with no hand editing");
}

/// The bytes on disk are the bytes the closeout writer would produce, so the artifact a maintainer
/// reviews is the artifact that lands.
#[test]
fn the_written_bytes_are_what_close_agent_maintenance_would_rewrite() {
    let (fixture, head) = seed_packet("t6-bytes", &[]);
    prepare(&fixture, &head, "success", true).expect("generation succeeds");

    let written = std::fs::read(fixture.join(CLOSEOUT_PATH)).expect("read generated artifact");
    let linked = load_linked_closeout(&fixture, Path::new(REQUEST_PATH), Path::new(CLOSEOUT_PATH))
        .expect("generated artifact validates");
    let rewritten =
        crate::closeout::serialize_closeout_json(&linked.closeout).expect("re-serialize");

    assert_eq!(
        written, rewritten,
        "close-agent-maintenance would rewrite the file this command just wrote"
    );
}

/// Preview is the default, and it must not touch the output path.
#[test]
fn without_write_nothing_is_written() {
    let (fixture, head) = seed_packet("t6-preview", &[]);

    let stdout = prepare(&fixture, &head, "success", false).expect("preview succeeds");

    assert!(stdout.contains("preview"), "{stdout}");
    assert!(stdout.contains("Nothing written"), "{stdout}");
    assert!(
        !fixture.join(CLOSEOUT_PATH).exists(),
        "preview wrote the closeout"
    );
}

/// Spec §3 and §6: the generator never records a failure. T4 resolves one; T6 refuses on it.
#[test]
fn a_red_preflight_refuses_instead_of_recording_false() {
    let (fixture, head) = seed_packet("t6-red", &[]);

    let err = prepare(&fixture, &head, "failure", true).expect_err("a red preflight refuses");

    assert!(err.contains("resolved to a failure"), "{err}");
    assert!(err.contains("preflight_passed: false"), "{err}");
    assert!(
        err.contains("42"),
        "the refusal must name the runs it consulted: {err}"
    );
    assert!(
        !fixture.join(CLOSEOUT_PATH).exists(),
        "a refused generation wrote the closeout anyway"
    );
}

/// A wrapper-only row nobody has adjudicated is a missing human judgement, and T6 says which rows.
#[test]
fn an_unadjudicated_wrapper_only_row_refuses_and_names_it() {
    let (fixture, head) = seed_packet("t6-unadjudicated", &[status_json_flag()]);

    let err = prepare(&fixture, &head, "success", true)
        .expect_err("an unadjudicated row cannot be generated past");

    assert!(err.contains("no recorded disposition"), "{err}");
    assert!(
        err.contains("--json"),
        "the refusal must name the row: {err}"
    );
    assert!(
        !fixture.join(CLOSEOUT_PATH).exists(),
        "a refused generation wrote the closeout anyway"
    );
}

/// The carry-forward: an adjudication recorded once survives into the next generation, so the same
/// judgement is not asked for twice.
#[test]
fn a_recorded_disposition_is_carried_forward_into_the_new_artifact() {
    let (fixture, head) = seed_packet("t6-carry", &[status_json_flag()]);
    write_text(
        &fixture.join(CLOSEOUT_PATH),
        &json!({
            "wrapper_only_dispositions": [{
                "surface_kind": "flags",
                "command_path": "opencode status",
                "surface_id": "--json",
                "category": "hidden_upstream_supported",
                "evidence_ref": BASIS_REF,
                "note": "Upstream hides this flag from help; the wrapper still forwards it.",
            }]
        })
        .to_string(),
    );

    prepare(&fixture, &head, "success", true).expect("a carried disposition generates");

    let linked = load_linked_closeout(&fixture, Path::new(REQUEST_PATH), Path::new(CLOSEOUT_PATH))
        .expect("the carried artifact validates");
    assert_eq!(linked.closeout.wrapper_only_dispositions.len(), 1);
    assert_eq!(
        linked.closeout.wrapper_only_baseline_ref.as_deref(),
        Some(BASELINE_REF)
    );
}

/// A disposition whose evidence no longer resolves is not carried on trust — the same bar the
/// validator applies is applied at carry time, so the refusal names the field rather than
/// surfacing later as a rejected artifact.
#[test]
fn a_carried_disposition_with_dangling_evidence_refuses() {
    let (fixture, head) = seed_packet("t6-dangling", &[status_json_flag()]);
    write_text(
        &fixture.join(CLOSEOUT_PATH),
        &json!({
            "wrapper_only_dispositions": [{
                "surface_kind": "flags",
                "command_path": "opencode status",
                "surface_id": "--json",
                "category": "hidden_upstream_supported",
                "evidence_ref": "docs/evidence/this-file-does-not-exist.md",
                "note": "Upstream hides this flag from help.",
            }]
        })
        .to_string(),
    );

    let err = prepare(&fixture, &head, "success", true)
        .expect_err("dangling evidence must not be carried forward");

    assert!(err.contains("evidence_ref"), "{err}");
}

/// The commit binding is T6's to call: `closeout.rs` says so explicitly, because the validator does
/// not cover it.
#[test]
fn a_commit_that_is_not_reachable_from_head_refuses() {
    let (fixture, _head) = seed_packet("t6-unreachable", &[]);

    let err = prepare(
        &fixture,
        "4adefdf4adefdf4adefdf4adefdf4adefdf4adef",
        "success",
        true,
    )
    .expect_err("a commit this repository never contained refuses");

    assert!(err.contains("not a commit in this repository"), "{err}");
    assert!(
        !fixture.join(CLOSEOUT_PATH).exists(),
        "a refused generation wrote the closeout anyway"
    );
}

/// A refused generation restores the closeout that was there, rather than leaving a rejected
/// artifact where a valid one used to be.
#[test]
fn a_refused_generation_leaves_a_prior_closeout_untouched() {
    let (fixture, head) = seed_packet("t6-restore", &[status_json_flag()]);
    let prior = json!({
        "wrapper_only_dispositions": [{
            "surface_kind": "flags",
            "command_path": "opencode status",
            "surface_id": "--json",
            "category": "hidden_upstream_supported",
            "evidence_ref": BASIS_REF,
            "note": "Upstream hides this flag from help; the wrapper still forwards it.",
        }]
    })
    .to_string();
    write_text(&fixture.join(CLOSEOUT_PATH), &prior);

    // Red CI refuses after the prior file has been read but before anything is written.
    prepare(&fixture, &head, "failure", true).expect_err("a red preflight refuses");

    assert_eq!(
        std::fs::read_to_string(fixture.join(CLOSEOUT_PATH)).expect("read prior closeout"),
        prior,
        "the prior closeout was modified by a refused generation"
    );
}

/// A request with no detected release has no version to scope T5's derivation to, and says so
/// rather than deriving against an empty version.
#[test]
fn a_request_without_a_detected_release_refuses_with_the_reason() {
    let (fixture, head) = seed_packet("t6-no-release", &[]);
    write_text(
        &fixture.join(REQUEST_PATH),
        &crate::closeout_harness::maintenance_request_toml("opencode", BASIS_REF),
    );

    let err = prepare(&fixture, &head, "success", true).expect_err("no release, no closeout");

    assert!(err.contains("no detected release"), "{err}");
}
