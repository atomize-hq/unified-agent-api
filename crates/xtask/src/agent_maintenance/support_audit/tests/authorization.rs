use super::*;

fn gap_report(path: &[&str], key: Option<&str>, targets: &[&str]) -> Value {
    let mut row = json!({"path": path, "upstream_available_on": targets});
    if let Some(key) = key {
        row["key"] = json!(key);
    }
    let report = if key.is_some() {
        deltas(json!([]), json!([row]), json!([]))
    } else {
        deltas(json!([row]), json!([]), json!([]))
    };
    any_report_for_targets(targets, report)
}

#[test]
fn gap_targets_come_from_exact_reports_not_the_any_row_availability_mask() {
    let workspace = tempfile::TempDir::new().expect("workspace");
    let version_dir = workspace.path().join("reports/1.18.30");
    fs::create_dir_all(&version_dir).expect("report dir");
    for (target, rows) in [
        ("linux-x64", json!([])),
        (
            "darwin-arm64",
            json!([{"path": ["acp"], "upstream_available_on": TEST_TARGETS}]),
        ),
    ] {
        fs::write(
            version_dir.join(format!("coverage.{target}.json")),
            json!({
                "inputs": {"upstream": {
                    "semantic_version": "1.18.30",
                    "targets": [target],
                }},
                "platform_filter": {"mode": "exact_target", "target_triple": target},
                "deltas": deltas(rows, json!([]), json!([])),
            })
            .to_string(),
        )
        .expect("write report");
    }
    let targets = TEST_TARGETS.iter().map(ToString::to_string).collect();
    let gaps = load_targeted_gaps(&version_dir, "opencode", "1.18.30", &targets)
        .expect("load exact-target gaps");
    assert_eq!(gaps.len(), 1);
    assert_eq!(
        gaps[0].targets,
        ["darwin-arm64".to_string()].into_iter().collect()
    );
}

#[test]
fn authorization_requires_the_exact_identity_version_and_every_gap_target() {
    let evidence = "cli_manifests/opencode/reports/1.18.30/coverage.authorization.json";
    let exact = debt_inventory(&scoped_debt_row(
        "grant",
        "commands",
        "opencode acp",
        "acp",
        "linux-x64, darwin-arm64",
        "1.18.30",
        evidence,
    ));
    let report = gap_report(&["acp"], None, &TEST_TARGETS);
    let audit = derive_audit("opencode", "1.18.30", Some(&exact), &report, None);
    assert!(audit.required_uplifts_this_run.is_empty());
    assert_eq!(audit.deferred_preexisting_gaps.len(), 1);

    let wrong_identity = debt_inventory(&scoped_debt_row(
        "grant",
        "flags",
        "opencode other",
        "acp",
        "linux-x64, darwin-arm64",
        "1.18.30",
        evidence,
    ));
    let audit = derive_audit(
        "opencode",
        "1.18.30",
        Some(&wrong_identity),
        &report,
        Some(&union(json!([]))),
    );
    assert_eq!(audit.required_uplifts_this_run.len(), 1);

    let old_version = debt_inventory(&scoped_debt_row(
        "grant",
        "commands",
        "opencode acp",
        "acp",
        "linux-x64, darwin-arm64",
        "1.18.29",
        "cli_manifests/opencode/reports/1.18.29/coverage.authorization.json",
    ));
    let audit = derive_audit("opencode", "1.18.30", Some(&old_version), &report, None);
    assert_eq!(audit.required_uplifts_this_run.len(), 1);
}

#[test]
fn a_single_target_grant_does_not_authorize_a_new_target() {
    let debt = debt_inventory(&scoped_debt_row(
        "linux-only",
        "commands",
        "opencode acp",
        "acp",
        "linux-x64",
        "1.18.30",
        "cli_manifests/opencode/reports/1.18.30/coverage.authorization.json",
    ));
    let audit = derive_audit(
        "opencode",
        "1.18.30",
        Some(&debt),
        &gap_report(&["acp"], None, &TEST_TARGETS),
        None,
    );
    assert_eq!(audit.required_uplifts_this_run.len(), 1);
    assert_eq!(audit.deferred_preexisting_gaps.len(), 1);
}

#[test]
fn disjoint_grants_contribute_independently_and_order_does_not_change_the_verdict() {
    let linux = scoped_debt_row(
        "linux",
        "commands",
        "opencode acp",
        "acp",
        "linux-x64",
        "1.18.30",
        "cli_manifests/opencode/reports/1.18.30/coverage.authorization.json",
    );
    let darwin = scoped_debt_row(
        "darwin",
        "commands",
        "opencode acp",
        "acp",
        "darwin-arm64",
        "1.18.30",
        "cli_manifests/opencode/reports/1.18.30/coverage.authorization.json",
    );
    let report = gap_report(&["acp"], None, &TEST_TARGETS);
    for rows in [format!("{linux}{darwin}"), format!("{darwin}{linux}")] {
        let audit = derive_audit(
            "opencode",
            "1.18.30",
            Some(&debt_inventory(&rows)),
            &report,
            None,
        );
        assert!(audit.required_uplifts_this_run.is_empty());
        assert_eq!(audit.pre_run_debt_count, 1);
        assert_eq!(audit.deferred_preexisting_gaps.len(), 1);
        assert!(audit.preexisting_unsupported_surface[0]
            .debt_ref
            .ends_with("#darwin"));
    }
}

#[test]
fn overlapping_grants_are_rejected() {
    let first = scoped_debt_row(
        "first",
        "commands",
        "opencode acp",
        "acp",
        "linux-x64",
        "1.18.30",
        "cli_manifests/opencode/reports/1.18.30/coverage.authorization.json",
    );
    let second = scoped_debt_row(
        "second",
        "commands",
        "opencode acp",
        "acp",
        "linux-x64, darwin-arm64",
        "1.18.30",
        "cli_manifests/opencode/reports/1.18.30/coverage.authorization.json",
    );
    let error = try_derive_audit(
        "opencode",
        "1.18.30",
        Some(&debt_inventory(&format!("{first}{second}"))),
        &gap_report(&["acp"], None, &TEST_TARGETS),
        None,
    )
    .expect_err("overlap must fail validation");
    assert!(error.contains("overlap") && error.contains("target=linux-x64"));
}

#[test]
fn authorization_scope_cannot_exceed_its_evidence_observations() {
    let debt = debt_inventory(&scoped_debt_row(
        "too-wide",
        "commands",
        "opencode acp",
        "acp",
        "linux-x64, darwin-arm64",
        "1.18.30",
        "cli_manifests/opencode/reports/1.18.30/coverage.any.json",
    ));
    let report = gap_report(&["acp"], None, &["linux-x64"]);
    let error = try_derive_audit("opencode", "1.18.30", Some(&debt), &report, None)
        .expect_err("scope wider than evidence must fail");
    assert!(error.contains("scope exceeds surface observations"));
}

#[test]
fn debt_schema_rejects_missing_empty_or_malformed_authorization_fields() {
    let valid = debt_inventory(&debt_row("row", "commands", "opencode acp", "acp"));
    let cases = [
        valid.replace("- `scope_target_triples`: `linux-x64, darwin-arm64`\n", ""),
        valid.replace(
            "- `scope_target_triples`: `linux-x64, darwin-arm64`",
            "- `scope_target_triples`: ``",
        ),
        valid.replace("linux-x64, darwin-arm64", "linux-x64,"),
        valid.replace("- `authorized_at_version`: `1.18.30`\n", ""),
        valid.replace(
            "authorized_at_version`: `1.18.30",
            "authorized_at_version`: `current",
        ),
    ];
    for text in cases {
        let workspace = tempfile::TempDir::new().expect("workspace");
        let path = workspace.path().join(NON_TUI_SUPPORT_DEBT_PATH);
        fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
        fs::write(path, text).expect("write debt");
        assert!(load_debt_inventory(workspace.path()).is_err());
    }
}

#[test]
fn debt_schema_requires_the_contract_marker_and_rejects_unknown_keys() {
    let valid = debt_inventory(&debt_row("row", "commands", "opencode acp", "acp"));
    let cases = [
        valid.replace(
            "### `support-debt-authorization-contract-target-version-v1`\n\n",
            "",
        ),
        valid.replace(
            "- `owner`: `test`",
            "- `owner`: `test`\n- `scope_target_tripples`: `linux-x64`",
        ),
    ];
    for text in cases {
        let workspace = tempfile::TempDir::new().expect("workspace");
        let path = workspace.path().join(NON_TUI_SUPPORT_DEBT_PATH);
        fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
        fs::write(path, text).expect("write debt");
        assert!(load_debt_inventory(workspace.path()).is_err());
    }
}
