//! `uaa-0039`: closeout must enforce the support-audit baseline.
//!
//! Every artifact here is hand-authored rather than produced by a generator. The manual closeout
//! path is the one this item exists to protect, and a generator test would only prove that the
//! generator and the validator agree with each other.

use std::path::Path;

use serde_json::{json, Value};

use crate::{
    closeout_harness::{automated_maintenance_request_toml, valid_closeout},
    harness::{fixture_root, sha256_hex, write_text},
    load_linked_closeout, seed_opencode_basis,
};

const REQUEST_PATH: &str =
    "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
const CLOSEOUT_PATH: &str =
    "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json";
const BASIS_REF: &str = "docs/integrations/opencode/governance/seam-2-closeout.md";
const BASELINE_REF: &str = "cli_manifests/opencode/reports/1.14.47/coverage.any.json";

/// A wrapper-only flag row on `opencode status`, which resolves to the audit identity
/// `surface_kind=flags command_path=opencode status surface_id=--json`.
fn status_json_flag() -> Value {
    json!({ "path": ["status"], "key": "--json", "wrapper_level": "explicit" })
}

fn disposition(category: &str, extra: Value) -> Value {
    let mut row = json!({
        "surface_kind": "flags",
        "command_path": "opencode status",
        "surface_id": "--json",
        "category": category,
        "evidence_ref": BASIS_REF,
        "note": "Upstream hides this flag from help; the wrapper still forwards it.",
    });
    if let (Some(row), Some(extra)) = (row.as_object_mut(), extra.as_object()) {
        for (key, value) in extra {
            row.insert(key.clone(), value.clone());
        }
    }
    row
}

/// Rewrites the selected report so its wrapper-only lists hold `rows`. An empty slice leaves the
/// keys off entirely, which is what the report writer does and what claude_code's report looks like.
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

/// Seeds a packet whose request names a detected release, then writes the closeout that `mutate`
/// shapes. Returns the validator's verdict.
fn close(
    name: &str,
    wrapper_only: &[Value],
    mutate: impl FnOnce(&mut Value),
) -> Result<(), String> {
    let fixture = fixture_root(name);
    seed_opencode_basis(&fixture);
    write_text(
        &fixture.join(".github/workflows/agent-maintenance-open-pr.yml"),
        "name: Packet PR worker\n",
    );
    set_wrapper_only_flags(&fixture, wrapper_only);

    let request_absolute = fixture.join(REQUEST_PATH);
    write_text(
        &request_absolute,
        &automated_maintenance_request_toml("opencode", BASIS_REF),
    );

    let mut closeout = valid_closeout(REQUEST_PATH, &sha256_hex(&request_absolute));
    mutate(&mut closeout);
    write_text(
        &fixture.join(CLOSEOUT_PATH),
        &serde_json::to_string_pretty(&closeout).expect("serialize closeout"),
    );

    load_linked_closeout(&fixture, Path::new(REQUEST_PATH), Path::new(CLOSEOUT_PATH))
        .map(|_| ())
        .map_err(|err| err.to_string())
}

fn adjudicated(dispositions: Value) -> impl FnOnce(&mut Value) {
    move |closeout: &mut Value| {
        closeout["wrapper_only_baseline_ref"] = json!(BASELINE_REF);
        closeout["wrapper_only_dispositions"] = dispositions;
    }
}

#[test]
fn an_adjudicated_wrapper_only_row_closes() {
    close(
        "uaa-0039-clean",
        &[status_json_flag()],
        adjudicated(json!([disposition("hidden_upstream_supported", json!({}))])),
    )
    .expect("a fully adjudicated packet closes");
}

#[test]
fn a_packet_with_no_wrapper_only_row_closes_without_a_baseline() {
    // The emptiness is re-derived here, so there is nothing for the artifact to point at.
    close("uaa-0039-empty", &[], |_| {}).expect("an empty baseline needs no disposition");
}

#[test]
fn an_uncategorised_wrapper_only_row_blocks_closeout() {
    let err = close(
        "uaa-0039-uncategorised",
        &[status_json_flag()],
        adjudicated(json!([])),
    )
    .expect_err("an uncategorised row must not close");
    assert!(
        err.contains("has no entry in `wrapper_only_dispositions`")
            && err.contains("surface_id=--json"),
        "{err}"
    );
}

#[test]
fn a_row_sorted_obsolete_that_is_still_claimed_blocks_closeout() {
    // The category owes a contraction in the same run. A row still in the live report proves the
    // wrapper claim was never withdrawn, whatever the note says.
    let err = close(
        "uaa-0039-obsolete-uncontracted",
        &[status_json_flag()],
        adjudicated(json!([disposition("obsolete", json!({}))])),
    )
    .expect_err("an uncontracted obsolete row must not close");
    assert!(
        err.contains("sorted `obsolete` but still appears in the live wrapper-only report"),
        "{err}"
    );
}

#[test]
fn an_obsolete_row_that_was_contracted_closes() {
    // The `uaa-0039` step 1b case: the claim was withdrawn and the report regenerated, so the row
    // is gone. Judged against the final set alone this entry would look extraneous.
    close(
        "uaa-0039-obsolete-contracted",
        &[],
        adjudicated(json!([disposition("obsolete", json!({}))])),
    )
    .expect("a contracted obsolete row keeps its disposition");
}

#[test]
fn a_disposition_for_a_surface_that_was_never_claimed_blocks_closeout() {
    let err = close(
        "uaa-0039-extraneous",
        &[],
        adjudicated(json!([disposition("hidden_upstream_supported", json!({}))])),
    )
    .expect_err("only `obsolete` explains an absent row");
    assert!(err.contains("is not a live wrapper-only row"), "{err}");
}

#[test]
fn duplicate_dispositions_for_one_surface_block_closeout() {
    let err = close(
        "uaa-0039-duplicate",
        &[status_json_flag()],
        adjudicated(json!([
            disposition("hidden_upstream_supported", json!({})),
            disposition(
                "discovery_bug",
                json!({ "follow_on": "TODOS.md#opencode-json" })
            ),
        ])),
    )
    .expect_err("one row gets one disposition");
    assert!(err.contains("more than one entry for"), "{err}");
}

#[test]
fn an_unknown_category_blocks_closeout() {
    let err = close(
        "uaa-0039-bad-category",
        &[status_json_flag()],
        adjudicated(json!([disposition("target_partial", json!({}))])),
    )
    .expect_err("the taxonomy is closed");
    assert!(
        err.contains("is not one of hidden_upstream_supported"),
        "{err}"
    );
}

#[test]
fn older_upstream_only_without_a_last_supported_version_blocks_closeout() {
    let err = close(
        "uaa-0039-missing-version",
        &[status_json_flag()],
        adjudicated(json!([disposition("older_upstream_only", json!({}))])),
    )
    .expect_err("the version is the whole claim");
    assert!(
        err.contains("last_supported_version` is required for category `older_upstream_only`"),
        "{err}"
    );
}

#[test]
fn a_discovery_bug_without_a_follow_on_blocks_closeout() {
    let err = close(
        "uaa-0039-untracked-bug",
        &[status_json_flag()],
        adjudicated(json!([disposition("discovery_bug", json!({}))])),
    )
    .expect_err("a discovery bug is work owed");
    assert!(
        err.contains("follow_on` is required for category `discovery_bug`"),
        "{err}"
    );
}

#[test]
fn a_follow_on_on_another_category_blocks_closeout() {
    // Otherwise an obsolete row can carry a tracking ref it never earned.
    let err = close(
        "uaa-0039-stray-follow-on",
        &[status_json_flag()],
        adjudicated(json!([disposition(
            "hidden_upstream_supported",
            json!({ "follow_on": "TODOS.md#opencode-json" })
        )])),
    )
    .expect_err("evidence belongs to one category");
    assert!(
        err.contains("follow_on` is only allowed for category `discovery_bug`"),
        "{err}"
    );
}

#[test]
fn a_dangling_evidence_ref_blocks_closeout() {
    let mut row = disposition("hidden_upstream_supported", json!({}));
    row["evidence_ref"] = json!("docs/specs/unified-agent-api/does-not-exist.md");
    let err = close(
        "uaa-0039-dangling-evidence",
        &[status_json_flag()],
        adjudicated(json!([row])),
    )
    .expect_err("evidence that does not resolve is not evidence");
    assert!(
        err.contains("does not resolve to a file in the repository"),
        "{err}"
    );
}

#[test]
fn a_baseline_that_is_not_the_selected_report_blocks_closeout() {
    // Adjudicating `coverage.all.json` while `coverage.any.json` exists judges a narrower set than
    // the one that governs.
    let err = close(
        "uaa-0039-wrong-baseline",
        &[status_json_flag()],
        |closeout| {
            closeout["wrapper_only_baseline_ref"] =
                json!("cli_manifests/opencode/reports/1.14.47/coverage.all.json");
            closeout["wrapper_only_dispositions"] =
                json!([disposition("hidden_upstream_supported", json!({}))]);
        },
    )
    .expect_err("the baseline is not the packet's to choose");
    assert!(
        err.contains("but the support audit selects") && err.contains("coverage.any.json"),
        "{err}"
    );
}

#[test]
fn dispositions_without_a_detected_release_block_closeout() {
    // No target version means no bound report, so a disposition asserts an adjudication that
    // nothing can verify.
    let fixture = fixture_root("uaa-0039-unbound");
    seed_opencode_basis(&fixture);
    let request_absolute = fixture.join(REQUEST_PATH);
    write_text(
        &request_absolute,
        &crate::closeout_harness::maintenance_request_toml("opencode", BASIS_REF),
    );
    let mut closeout = valid_closeout(REQUEST_PATH, &sha256_hex(&request_absolute));
    closeout["wrapper_only_dispositions"] =
        json!([disposition("hidden_upstream_supported", json!({}))]);
    write_text(
        &fixture.join(CLOSEOUT_PATH),
        &serde_json::to_string_pretty(&closeout).expect("serialize closeout"),
    );

    let err = load_linked_closeout(&fixture, Path::new(REQUEST_PATH), Path::new(CLOSEOUT_PATH))
        .expect_err("an unbound disposition must not close")
        .to_string();
    assert!(
        err.contains("must be absent because the request declares no detected release"),
        "{err}"
    );
}

/// Writes one fully-formed debt row for `opencode status`. The seeded report lists no missing
/// command, so the row matches nothing in the gap set and the live audit reports it unmatched.
fn seed_unmatched_debt_row(fixture: &Path) {
    write_text(
        &fixture.join("docs/specs/unified-agent-api/non-tui-support-debt.md"),
        concat!(
            "# Non-TUI Support Debt Inventory\n\n",
            "### `support-debt-authorization-contract-target-version-v1`\n\n",
            "## Inventory\n\n",
            "### `opencode-status-command`\n\n",
            "- `agent_id`: `opencode`\n",
            "- `surface_kind`: `commands`\n",
            "- `command_path`: `opencode status`\n",
            "- `surface_id`: `status`\n",
            "- `current_reason`: `The status command remains deferred.`\n",
            "- `blocker_class`: `requires_new_infra`\n",
            "- `owner`: `wrappers team`\n",
            "- `milestone`: `post packet-pr convergence follow-on`\n",
            "- `follow_on`: `TODOS.md#close-opencode-status-gap`\n",
            "- `evidence_ref`: `cli_manifests/opencode/reports/1.14.47/coverage.any.json`\n",
            "- `scope_target_triples`: `linux-x64, darwin-arm64, win32-x64`\n",
            "- `authorized_at_version`: `1.14.47`\n",
            "- `authorization_evidence_ref`: `cli_manifests/opencode/reports/1.14.47/coverage.authorization.json`\n"
        ),
    );
    write_text(
        &fixture.join("cli_manifests/opencode/reports/1.14.47/coverage.authorization.json"),
        &json!({
            "inputs": {"upstream": {
                "semantic_version": "1.14.47",
                "targets": ["linux-x64", "darwin-arm64", "win32-x64"]
            }},
            "deltas": {
                "missing_commands": [{
                    "path": ["status"],
                    "upstream_available_on": ["linux-x64", "darwin-arm64", "win32-x64"]
                }],
                "missing_flags": [],
                "missing_args": [],
                "intentionally_unsupported": []
            }
        })
        .to_string(),
    );
    // `classify_unmatched_debt_rows` reads the union to decide whether an unmatched row is a stale
    // claim or a surface upstream hides. An empty command list makes `status` `not_observed`, which
    // is still unmatched.
    write_text(
        &fixture.join("cli_manifests/opencode/snapshots/1.14.47/union.json"),
        &json!({
            "inputs": [
                {"target_triple": "linux-x64"},
                {"target_triple": "darwin-arm64"},
                {"target_triple": "win32-x64"}
            ],
            "commands": []
        })
        .to_string(),
    );
}

#[test]
fn an_unmatched_debt_row_blocks_closeout() {
    // Field invariant 6. Derived live rather than read from the request, because a frozen request
    // that records the same unmatched rows reconciles `exact` and would otherwise close.
    let fixture = fixture_root("uaa-0039-unmatched-debt");
    seed_opencode_basis(&fixture);
    write_text(
        &fixture.join(".github/workflows/agent-maintenance-open-pr.yml"),
        "name: Packet PR worker\n",
    );
    seed_unmatched_debt_row(&fixture);

    // The request records the unmatched row it found, so the frozen block reconciles `exact` and
    // the request loads cleanly. That is the hole: reconciliation asks whether the artifact still
    // describes the repository, never whether what it describes is allowed to close.
    let request_absolute = fixture.join(REQUEST_PATH);
    write_text(
        &request_absolute,
        &(automated_maintenance_request_toml("opencode", BASIS_REF)
            .replace("pre_run_debt_count = 0", "pre_run_debt_count = 1")
            + concat!(
                "\n",
                "[[support_surface_audit.unmatched_debt_surface]]\n",
                "surface_kind = \"commands\"\n",
                "command_path = \"opencode status\"\n",
                "surface_id = \"status\"\n",
                "debt_ref = \"docs/specs/unified-agent-api/non-tui-support-debt.md#opencode-status-command\"\n",
                "observation = \"not_observed\"\n"
            )),
    );
    write_text(
        &fixture.join(CLOSEOUT_PATH),
        &serde_json::to_string_pretty(&valid_closeout(
            REQUEST_PATH,
            &sha256_hex(&request_absolute),
        ))
        .expect("serialize closeout"),
    );

    let err = load_linked_closeout(&fixture, Path::new(REQUEST_PATH), Path::new(CLOSEOUT_PATH))
        .expect_err("an unmatched debt row must not close")
        .to_string();
    assert!(
        err.contains("live `unmatched_debt_surface` must be empty before a packet closes")
            && err.contains("opencode status"),
        "{err}"
    );
}

// --- the admission gate (`uaa-0039`) -------------------------------------------------------

#[test]
fn closeout_refuses_when_the_stand_down_cannot_be_confirmed() {
    // The fixture is not a git repository, so reading the marker directory out of
    // `origin/staging` fails. A predicate error is never evidence of a declaration: the gate
    // refuses rather than treating an unreadable ref as an absent freeze that may be ignored.
    let fixture = fixture_root("uaa-0039-gate-unconfirmed");
    seed_opencode_basis(&fixture);
    write_text(
        &fixture.join(".github/workflows/agent-maintenance-open-pr.yml"),
        "name: Packet PR worker\n",
    );
    let request_absolute = fixture.join(REQUEST_PATH);
    write_text(
        &request_absolute,
        &automated_maintenance_request_toml("opencode", BASIS_REF),
    );
    write_text(
        &fixture.join(CLOSEOUT_PATH),
        &serde_json::to_string_pretty(&valid_closeout(
            REQUEST_PATH,
            &sha256_hex(&request_absolute),
        ))
        .expect("serialize closeout"),
    );

    let mut stdout = Vec::new();
    let err = crate::closeout::run_in_workspace(
        &fixture,
        crate::closeout::Args {
            request: std::path::PathBuf::from(REQUEST_PATH),
            closeout: std::path::PathBuf::from(CLOSEOUT_PATH),
        },
        &mut stdout,
    )
    .expect_err("an unconfirmable stand-down must not close")
    .to_string();
    assert!(
        err.contains("cannot confirm the automation stand-down for `opencode` 1.14.47"),
        "{err}"
    );
}

/// Seed a manual (ungated) closeout fixture and return the repository head the closeout records.
fn manual_closeout_fixture(prefix: &str) -> (std::path::PathBuf, String) {
    let fixture = fixture_root(prefix);
    seed_opencode_basis(&fixture);
    let head = crate::closeout_harness::init_git_fixture(&fixture);
    let request_absolute = fixture.join(REQUEST_PATH);
    write_text(
        &request_absolute,
        &crate::closeout_harness::maintenance_request_toml("opencode", BASIS_REF),
    );
    (fixture, head)
}

fn write_closeout_at_commit(fixture: &Path, commit: &str) {
    let request_absolute = fixture.join(REQUEST_PATH);
    let mut closeout = valid_closeout(REQUEST_PATH, &sha256_hex(&request_absolute));
    closeout["commit"] = json!(commit);
    write_text(
        &fixture.join(CLOSEOUT_PATH),
        &serde_json::to_string_pretty(&closeout).expect("serialize closeout"),
    );
}

fn run_closeout(fixture: &Path) -> Result<Vec<u8>, String> {
    let mut stdout = Vec::new();
    crate::closeout::run_in_workspace(
        fixture,
        crate::closeout::Args {
            request: std::path::PathBuf::from(REQUEST_PATH),
            closeout: std::path::PathBuf::from(CLOSEOUT_PATH),
        },
        &mut stdout,
    )
    .map(|()| stdout)
    .map_err(|err| err.to_string())
}

#[test]
fn a_request_without_a_detected_release_is_not_gated() {
    // No detected release means no packet generation for a marker to name and no cron job
    // regenerating it, so there is nothing to freeze against.
    let (fixture, head) = manual_closeout_fixture("uaa-0039-gate-manual");
    write_closeout_at_commit(&fixture, &head);

    let stdout = run_closeout(&fixture).expect("a manual closeout passes the gate");
    assert!(String::from_utf8_lossy(&stdout).contains("close-agent-maintenance write complete"));
}

/// T4's companion hardening. `validate_commit_shape` accepts any 7-40 lowercase hex string and
/// compares it to nothing, so a shaped-but-fabricated commit reads as verified. Without this, a T4
/// resolving evidence against the wrong revision emits an artifact that validates clean.
#[test]
fn a_commit_this_repository_has_never_contained_blocks_closeout() {
    let (fixture, _head) = manual_closeout_fixture("t4-binding-absent");
    write_closeout_at_commit(&fixture, "4adefdf");

    let err = run_closeout(&fixture).expect_err("a fabricated commit must not close");
    assert!(err.contains("is not a commit in this repository"), "{err}");
    assert!(err.contains("It is shaped like one"), "{err}");
}

/// Reachability, not equality: committing the closeout moves the head, so an equality check would
/// be unsatisfiable by construction. A commit on an unrelated branch is still refused.
#[test]
fn a_commit_that_is_not_reachable_from_head_blocks_closeout() {
    let (fixture, head) = manual_closeout_fixture("t4-binding-unreachable");
    let git = |args: &[&str]| {
        let out = std::process::Command::new("git")
            .current_dir(&fixture)
            .args(args)
            .output()
            .expect("git");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout)
            .expect("utf-8")
            .trim()
            .to_string()
    };
    git(&["switch", "--quiet", "-c", "elsewhere"]);
    git(&["commit", "--quiet", "--allow-empty", "-m", "elsewhere"]);
    let unreachable = git(&["rev-parse", "HEAD"]);
    git(&["switch", "--quiet", "main"]);

    write_closeout_at_commit(&fixture, &unreachable);
    let err = run_closeout(&fixture).expect_err("an unreachable commit must not close");
    assert!(err.contains("is not reachable from `HEAD`"), "{err}");

    // The same fixture closes once it records a reachable commit, so the refusal is the binding
    // and not the fixture.
    write_closeout_at_commit(&fixture, &head);
    run_closeout(&fixture).expect("a reachable commit closes");
}
