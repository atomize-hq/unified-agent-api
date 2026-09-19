use super::*;

const FROZEN_DISCOVERY_ROW: &str = concat!(
    "\n",
    "[[support_surface_audit.unbaselined_gap_surface]]\n",
    "surface_kind = \"commands\"\n",
    "command_path = \"opencode status\"\n",
    "surface_id = \"status\"\n",
    "evidence_ref = \"cli_manifests/opencode/reports/1.14.47/coverage.any.json\"\n",
    "\n",
    "[[support_surface_audit.required_uplifts_this_run]]\n",
    "surface_kind = \"commands\"\n",
    "command_path = \"opencode status\"\n",
    "surface_id = \"status\"\n",
    "reason = \"unbaselined_gap\"\n",
    "required_writes = [\"wrapper\", \"backend\", \"manifest\", \"publication\", \"packet_docs\"]\n"
);

const FROZEN_DEFERRED_ROW: &str = concat!(
    "\n",
    "[[support_surface_audit.preexisting_unsupported_surface]]\n",
    "surface_kind = \"commands\"\n",
    "command_path = \"opencode status\"\n",
    "surface_id = \"status\"\n",
    "debt_ref = \"docs/specs/unified-agent-api/non-tui-support-debt.md#opencode-status-command\"\n",
    "\n",
    "[[support_surface_audit.missing_wrapper_support]]\n",
    "surface_kind = \"commands\"\n",
    "command_path = \"opencode status\"\n",
    "surface_id = \"status\"\n",
    "\n",
    "[[support_surface_audit.missing_backend_support]]\n",
    "surface_kind = \"commands\"\n",
    "command_path = \"opencode status\"\n",
    "surface_id = \"status\"\n",
    "\n",
    "[[support_surface_audit.deferred_preexisting_gaps]]\n",
    "surface_kind = \"commands\"\n",
    "command_path = \"opencode status\"\n",
    "surface_id = \"status\"\n",
    "defer_reason = \"requires_new_infra\"\n",
    "blocking_follow_on = \"TODOS.md#close-opencode-status-gap\"\n",
    "\n",
    "[[support_surface_audit.publication_impacts]]\n",
    "surface_kind = \"commands\"\n",
    "command_path = \"opencode status\"\n",
    "surface_id = \"status\"\n",
    "surface_doc = \"docs/specs/unified-agent-api/support-matrix.md\"\n"
);

const FROZEN_UNMATCHED_DEBT_ROW: &str = concat!(
    "\n",
    "[[support_surface_audit.unmatched_debt_surface]]\n",
    "surface_kind = \"commands\"\n",
    "command_path = \"opencode serve\"\n",
    "surface_id = \"serve\"\n",
    "debt_ref = \"docs/specs/unified-agent-api/non-tui-support-debt.md#opencode-serve-command\"\n",
    "observation = \"not_observed\"\n"
);

const FROZEN_ELIGIBLE_ROW: &str = concat!(
    "\n",
    "[[support_surface_audit.eligible_preexisting_surface]]\n",
    "surface_kind = \"commands\"\n",
    "command_path = \"opencode status\"\n",
    "surface_id = \"status\"\n",
    "eligibility_reason = \"bounded_write_envelope\"\n"
);

const SUPPORT_AUDIT_REQUEST: &str =
    "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";

fn seed_live_new_discovery(fixture: &std::path::Path) {
    write_opencode_coverage_reports(fixture, "1.14.47", true);
}

fn seed_live_clean_report(fixture: &std::path::Path) {
    write_opencode_coverage_reports(fixture, "1.14.47", false);
}

fn seed_live_deferred_row(fixture: &std::path::Path, blocker_class: &str) {
    write_opencode_coverage_reports(fixture, "1.14.47", true);
    write_text(
        &fixture.join("docs/specs/unified-agent-api/non-tui-support-debt.md"),
        &format!(
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
                "- `blocker_class`: `{blocker_class}`\n",
                "- `owner`: `wrappers team`\n",
                "- `milestone`: `post packet-pr convergence follow-on`\n",
                "- `follow_on`: `TODOS.md#close-opencode-status-gap`\n",
                "- `evidence_ref`: `cli_manifests/opencode/reports/1.14.47/coverage.any.json`\n",
                "- `scope_target_triples`: `linux-x64, darwin-arm64, win32-x64`\n",
                "- `authorized_at_version`: `1.14.47`\n",
                "- `authorization_evidence_ref`: `cli_manifests/opencode/reports/1.14.47/coverage.authorization.json`\n"
            ),
            blocker_class = blocker_class
        ),
    );
    write_text(
        &fixture.join("cli_manifests/opencode/reports/1.14.47/coverage.authorization.json"),
        &serde_json::json!({
            "inputs": {
                "upstream": {
                    "semantic_version": "1.14.47",
                    "targets": ["linux-x64", "darwin-arm64", "win32-x64"]
                }
            },
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
}

#[test]
fn close_agent_maintenance_requires_request_linkage() {
    let fixture = fixture_root("close-agent-maintenance-request-linkage");
    seed_opencode_basis(&fixture);
    let request_path =
        Path::new("docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml");
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &maintenance_request_toml(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ),
    );

    let closeout_path = Path::new(
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json",
    );
    write_text(
        &fixture.join(closeout_path),
        &serde_json::to_string_pretty(&json!({
            "request_ref": "docs/agents/lifecycle/opencode-maintenance/governance/not-the-request.toml",
            "request_sha256": sha256_hex(&request_absolute),
            "resolved_findings": [finding_json(
                "governance_doc_drift",
                "SEAM-2 closeout now matches the landed capability advertisement boundary.",
                &[
                    "docs/integrations/opencode/governance/seam-2-closeout.md",
                    "docs/agents/lifecycle/opencode-maintenance/HANDOFF.md"
                ],
            )],
            "explicit_none_reason": "No deferred maintenance findings remain after packet refresh.",
            "preflight_passed": true,
            "recorded_at": "2026-04-22T01:45:00Z",
            "commit": "4adefdf"
        }))
        .expect("serialize closeout"),
    );

    let err = load_linked_closeout(&fixture, request_path, closeout_path)
        .expect_err("request linkage mismatch should fail");
    assert!(err
        .to_string()
        .contains("`request_ref` must equal `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`"));
}

#[test]
fn close_agent_maintenance_requires_resolved_and_deferred_truth() {
    let fixture = fixture_root("close-agent-maintenance-truth");
    seed_opencode_basis(&fixture);
    let request_path =
        Path::new("docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml");
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &maintenance_request_toml(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ),
    );

    let closeout_path = Path::new(
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json",
    );
    write_text(
        &fixture.join(closeout_path),
        &serde_json::to_string_pretty(&json!({
            "request_ref": request_path.display().to_string(),
            "request_sha256": sha256_hex(&request_absolute),
            "resolved_findings": [],
            "explicit_none_reason": "No deferred maintenance findings remain after packet refresh.",
            "preflight_passed": true,
            "recorded_at": "2026-04-22T01:45:00Z",
            "commit": "4adefdf"
        }))
        .expect("serialize closeout"),
    );

    let err = load_linked_closeout(&fixture, request_path, closeout_path)
        .expect_err("empty resolved findings should fail");
    assert!(err
        .to_string()
        .contains("`resolved_findings` must not be empty"));

    write_text(
        &fixture.join(closeout_path),
        &serde_json::to_string_pretty(&json!({
            "request_ref": request_path.display().to_string(),
            "request_sha256": sha256_hex(&request_absolute),
            "resolved_findings": [finding_json(
                "governance_doc_drift",
                "SEAM-2 closeout now matches the landed capability advertisement boundary.",
                &[
                    "docs/integrations/opencode/governance/seam-2-closeout.md",
                ],
            )],
            "deferred_findings": [finding_json(
                "support_publication_drift",
                "Support publication still needs follow-up.",
                &[
                    "docs/specs/unified-agent-api/support-matrix.md",
                ],
            )],
            "explicit_none_reason": "No deferred maintenance findings remain after packet refresh.",
            "preflight_passed": true,
            "recorded_at": "2026-04-22T01:45:00Z",
            "commit": "4adefdf"
        }))
        .expect("serialize closeout"),
    );

    let err = load_linked_closeout(&fixture, request_path, closeout_path)
        .expect_err("deferred findings xor explicit-none is required");
    assert!(err
        .to_string()
        .contains("exactly one of `deferred_findings` or `explicit_none_reason` is required"));
}

#[test]
fn close_agent_maintenance_rejects_symlinked_output() {
    let fixture = fixture_root("close-agent-maintenance-symlink-output");
    seed_opencode_basis(&fixture);
    let request_path =
        Path::new("docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml");
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &maintenance_request_toml(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ),
    );

    let closeout_path = Path::new(
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json",
    );
    write_text(
        &fixture.join(closeout_path),
        &serde_json::to_string_pretty(&json!({
            "request_ref": request_path.display().to_string(),
            "request_sha256": sha256_hex(&request_absolute),
            "resolved_findings": [finding_json(
                "governance_doc_drift",
                "SEAM-2 closeout still matches live governance drift.",
                &[
                    "docs/integrations/opencode/governance/seam-2-closeout.md",
                    "docs/specs/unified-agent-api/capability-matrix.md"
                ],
            )],
            "explicit_none_reason": "No deferred maintenance findings remain after publication and packet refresh.",
            "preflight_passed": true,
            "recorded_at": "2026-04-22T01:45:00Z",
            "commit": "4adefdf"
        }))
        .expect("serialize closeout"),
    );

    let handoff_path = fixture.join("docs/agents/lifecycle/opencode-maintenance/HANDOFF.md");
    let outside = fixture_root("close-agent-maintenance-symlink-target");
    let outside_target = outside.join("handoff.md");
    write_text(&outside_target, "outside handoff\n");
    if let Some(parent) = handoff_path.parent() {
        fs::create_dir_all(parent).expect("create handoff parent");
    }
    symlink(&outside_target, &handoff_path).expect("create handoff symlink");

    let err = write_closeout_outputs(&fixture, request_path, closeout_path)
        .expect_err("symlinked output should fail");
    let message = err.to_string();
    assert!(message.contains("HANDOFF.md"));
    assert!(message.contains("symlink"));
}

#[test]
fn close_agent_maintenance_rejects_missing_request_evidence_refs() {
    let fixture = fixture_root("close-agent-maintenance-missing-request-evidence");
    seed_opencode_basis(&fixture);
    let request_path =
        Path::new("docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml");
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &maintenance_request_toml_with_refs(
            "opencode",
            "docs/agents/lifecycle/opencode-maintenance/governance/missing-basis.md",
            "docs/agents/lifecycle/opencode-maintenance/governance/missing-opened-from.md",
        ),
    );

    let closeout_path = Path::new(
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json",
    );
    write_text(
        &fixture.join(closeout_path),
        &valid_closeout_json(&request_absolute, request_path),
    );

    let err = load_linked_closeout(&fixture, request_path, closeout_path)
        .expect_err("missing request evidence refs should fail");
    let message = err.to_string();
    assert!(message.contains("unable to load linked request"));
    assert!(message.contains("field `basis_ref`"));
    assert!(message.contains("must point to an existing file"));
}

#[test]
fn close_agent_maintenance_accepts_satisfied_support_surface_audit_request() {
    let fixture = fixture_root("close-agent-maintenance-support-audit-satisfied");
    seed_opencode_basis(&fixture);
    write_text(
        &fixture.join(".github/workflows/agent-maintenance-open-pr.yml"),
        "name: Packet PR worker\n",
    );
    seed_live_clean_report(&fixture);
    let request_path =
        Path::new("docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml");
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &(automated_maintenance_request_toml(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ) + FROZEN_DISCOVERY_ROW),
    );

    let closeout_path = Path::new(
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json",
    );
    write_text(
        &fixture.join(closeout_path),
        &valid_closeout_json(&request_absolute, request_path),
    );

    let linked = load_linked_closeout(&fixture, request_path, closeout_path)
        .expect("satisfied request should remain closeable");
    assert_eq!(
        linked.request.trigger_kind.as_str(),
        "upstream_release_detected"
    );
}

#[test]
fn close_agent_maintenance_rejects_missing_live_report_in_linked_request() {
    let fixture = fixture_root("close-agent-maintenance-support-audit-missing-report");
    seed_opencode_basis(&fixture);
    write_text(
        &fixture.join(".github/workflows/agent-maintenance-open-pr.yml"),
        "name: Packet PR worker\n",
    );
    seed_live_clean_report(&fixture);
    fs::remove_dir_all(fixture.join("cli_manifests/opencode/reports/1.14.47"))
        .expect("remove live reports");
    let request_path =
        Path::new("docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml");
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &(automated_maintenance_request_toml(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ) + FROZEN_DISCOVERY_ROW),
    );

    let closeout_path = Path::new(
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json",
    );
    write_text(
        &fixture.join(closeout_path),
        &valid_closeout_json(&request_absolute, request_path),
    );

    let err = load_linked_closeout(&fixture, request_path, closeout_path)
        .expect_err("missing live report should invalidate closeout");
    let message = err.to_string();
    assert!(message.contains("cannot confirm reconciliation"));
    assert!(message.contains("target version `1.14.47`"));
    assert!(message.contains("cli_manifests/opencode/reports/1.14.47"));
}

#[test]
fn close_agent_maintenance_rejects_new_live_discovery_in_linked_request() {
    let fixture = fixture_root("close-agent-maintenance-support-audit-new-discovery");
    seed_opencode_basis(&fixture);
    write_text(
        &fixture.join(".github/workflows/agent-maintenance-open-pr.yml"),
        "name: Packet PR worker\n",
    );
    seed_live_new_discovery(&fixture);
    let request_path =
        Path::new("docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml");
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &automated_maintenance_request_toml(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ),
    );

    let closeout_path = Path::new(
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json",
    );
    write_text(
        &fixture.join(closeout_path),
        &valid_closeout_json(&request_absolute, request_path),
    );

    let err = load_linked_closeout(&fixture, request_path, closeout_path)
        .expect_err("new live discovery should invalidate closeout");
    let message = err.to_string();
    assert!(message.contains("support_surface_audit.unbaselined_gap_surface added"));
    assert!(
        message.contains("surface_kind=commands command_path=opencode status surface_id=status")
    );
}

#[test]
fn close_agent_maintenance_rejects_deferred_reason_mismatch_in_linked_request() {
    let fixture = fixture_root("close-agent-maintenance-support-audit-deferred-mismatch");
    seed_opencode_basis(&fixture);
    write_text(
        &fixture.join(".github/workflows/agent-maintenance-open-pr.yml"),
        "name: Packet PR worker\n",
    );
    seed_live_deferred_row(&fixture, "requires_new_architectural_seam");
    let request_path =
        Path::new("docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml");
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &(automated_maintenance_request_toml(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        )
        .replace("pre_run_debt_count = 0", "pre_run_debt_count = 1")
        .replace(
            "expected_post_run_debt_count = 0",
            "expected_post_run_debt_count = 1",
        ) + FROZEN_DEFERRED_ROW),
    );

    let closeout_path = Path::new(
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json",
    );
    write_text(
        &fixture.join(closeout_path),
        &valid_closeout_json(&request_absolute, request_path),
    );

    let err = load_linked_closeout(&fixture, request_path, closeout_path)
        .expect_err("deferred mismatch should invalidate closeout");
    let message = err.to_string();
    assert!(message.contains("support_surface_audit.deferred_preexisting_gaps changed"));
    assert!(message.contains("requires_new_infra"));
    assert!(message.contains("requires_new_architectural_seam"));
}

/// Each case breaks one row value the contract enumerates: the field the error must name, the
/// frozen rows with contract values, and the same rows with that value replaced by `invalid`.
fn invalid_row_value_cases() -> [(&'static str, String, String); 4] {
    let eligible = format!("{FROZEN_DISCOVERY_ROW}{FROZEN_ELIGIBLE_ROW}");
    let deferred = format!("{FROZEN_DISCOVERY_ROW}{FROZEN_DEFERRED_ROW}");
    let unmatched = format!("{FROZEN_DISCOVERY_ROW}{FROZEN_UNMATCHED_DEBT_ROW}");
    [
        (
            "unmatched_debt_surface[0].observation",
            unmatched.clone(),
            unmatched.replace("not_observed", "invalid"),
        ),
        (
            "required_uplifts_this_run[0].required_writes",
            FROZEN_DISCOVERY_ROW.to_string(),
            FROZEN_DISCOVERY_ROW.replace("\"packet_docs\"]", "\"packet_docs\", \"invalid\"]"),
        ),
        (
            "eligible_preexisting_surface[0].eligibility_reason",
            eligible.clone(),
            eligible.replace("bounded_write_envelope", "invalid"),
        ),
        (
            "deferred_preexisting_gaps[0].defer_reason",
            deferred.clone(),
            deferred.replace("requires_new_infra", "invalid"),
        ),
    ]
}

fn support_audit_row_fixture(prefix: &str) -> std::path::PathBuf {
    let fixture = fixture_root(prefix);
    seed_opencode_basis(&fixture);
    write_text(
        &fixture.join(".github/workflows/agent-maintenance-open-pr.yml"),
        "name: Packet PR worker\n",
    );
    fixture
}

fn load_with_frozen_rows(
    fixture: &Path,
    frozen_rows: &str,
    policy: request::AuditDriftPolicy,
) -> Result<(), String> {
    write_text(
        &fixture.join(SUPPORT_AUDIT_REQUEST),
        &(automated_maintenance_request_toml(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ) + frozen_rows),
    );
    request::load_request_envelope_validated_with_policy(
        fixture,
        Path::new(SUPPORT_AUDIT_REQUEST),
        policy,
    )
    .map(|_| ())
    .map_err(|err| err.to_string())
}

#[test]
fn request_load_accepts_unbaselined_gap_key_and_rejects_legacy_key() {
    let fixture = support_audit_row_fixture("support-audit-list-key");
    seed_live_clean_report(&fixture);
    // Assemble the retired key so repository searches only find historical documentation.
    let legacy_key = ["discovered", "upstream", "surface"].join("_");
    let legacy_rows = FROZEN_DISCOVERY_ROW.replace("unbaselined_gap_surface", &legacy_key);
    let legacy_list = legacy_rows
        .split("[[support_surface_audit.required_uplifts_this_run]]")
        .next()
        .unwrap();
    let both_rows = format!("{FROZEN_DISCOVERY_ROW}{legacy_list}");
    for policy in [
        request::AuditDriftPolicy::Reject,
        request::AuditDriftPolicy::Tolerate,
    ] {
        load_with_frozen_rows(&fixture, FROZEN_DISCOVERY_ROW, policy)
            .expect("the new list key must load");
        for rows in [&legacy_rows, &both_rows] {
            let err = load_with_frozen_rows(&fixture, rows, policy)
                .expect_err("the legacy list key must be rejected even alongside the new key");
            assert!(
                err.contains(&format!("unknown field `{legacy_key}`")),
                "{err}"
            );
        }
    }
}

#[test]
fn drift_tolerant_request_load_rejects_invalid_row_values_instead_of_reporting_drift() {
    // The live report still lists `opencode status`, as on a gate run whose uplifts remain.
    let fixture = support_audit_row_fixture("support-audit-row-values-tolerate");
    seed_live_new_discovery(&fixture);
    for (field, valid_rows, invalid_rows) in invalid_row_value_cases() {
        load_with_frozen_rows(&fixture, &valid_rows, request::AuditDriftPolicy::Tolerate)
            .unwrap_or_else(|err| panic!("contract values must load as drift for {field}: {err}"));
        let err =
            load_with_frozen_rows(&fixture, &invalid_rows, request::AuditDriftPolicy::Tolerate)
                .expect_err("an invalid row value must not load as drift");
        assert!(
            err.contains(&format!("support_surface_audit.{field}")) && err.contains("`invalid`"),
            "unexpected error for {field}: {err}"
        );
    }
}

#[test]
fn strict_request_load_rejects_invalid_row_values_even_when_satisfied() {
    // With the live report clean, the uplift, eligible and unmatched-debt cases reconcile as
    // satisfied, which never reads those rows; the deferred case drifts, so its error must name the
    // field instead.
    let fixture = support_audit_row_fixture("support-audit-row-values-satisfied");
    seed_live_clean_report(&fixture);
    for (field, _, invalid_rows) in invalid_row_value_cases() {
        let err = load_with_frozen_rows(&fixture, &invalid_rows, request::AuditDriftPolicy::Reject)
            .expect_err("an invalid row value must not reconcile as satisfied");
        assert!(
            err.contains(&format!("support_surface_audit.{field}")),
            "unexpected error for {field}: {err}"
        );
    }
}
