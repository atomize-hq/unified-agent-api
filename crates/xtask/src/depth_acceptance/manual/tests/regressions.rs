use super::*;
fn row() -> Value {
    json!({"row_id":"legacy", "agent_id":"codex", "surface_kind":"flags", "command_path":"codex exec", "surface_id":"--legacy", "current_reason":"Existing seam blocker", "blocker_class":"requires_new_architectural_seam", "owner":"wrappers", "milestone":"bounded follow-on", "follow_on":"TODOS.md#legacy", "evidence_ref":"historical.md", "scope_target_triples":[LINUX], "authorized_at_version":"0.9.0", "authorization_evidence_ref":"cli_manifests/codex/reports/0.9.0/coverage.any.json"})
}
fn inventory(rows: &[Value]) -> String {
    let mut raw = "### `support-debt-authorization-contract-target-version-v1`\n".to_owned();
    for row in rows {
        raw += &format!("\n### `{}`\n", row["row_id"].as_str().unwrap());
        for (key, value) in row.as_object().unwrap() {
            if key == "row_id" {
                continue;
            }
            let text = if let Some(a) = value.as_array() {
                a.iter()
                    .map(|v| v.as_str().unwrap())
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                value.as_str().unwrap().to_owned()
            };
            raw += &format!("- `{key}`: `{text}`\n");
        }
    }
    raw
}
fn coverage(root: &Path, gap: bool) {
    for target in [LINUX, "any"] {
        write(root, &format!("cli_manifests/codex/reports/1.2.3/coverage.{target}.json"), json!({"inputs":{"upstream":{"semantic_version":"1.2.3", "targets":[LINUX]}}, "platform_filter":{"mode":if target=="any" {"any"} else {"exact_target"}, "target_triple":if target=="any" {Value::Null} else {json!(LINUX)}}, "deltas":{"missing_commands":[], "missing_flags":if gap {json!([{"path":["exec"], "key":"--legacy", "upstream_available_on":[LINUX]}])} else {json!([])}, "missing_args":[]}}).to_string());
    }
}
#[test]
fn debt_inventory_cannot_add_or_alter_a_grant_outside_e() {
    let f = Fixture::new();
    write(
        f.root.path(),
        NON_TUI_SUPPORT_DEBT_PATH,
        inventory(&[row()]),
    );
    f.refuse("inventory differs");
}
#[test]
fn structurally_valid_renewal_or_retirement_is_not_semantic_authority() {
    for renewal in [true, false] {
        let mut f = Fixture::new();
        let old = row();
        let mut value = serde_json::to_value(&f.record).unwrap();
        value["policy"]["debt_baseline"] = json!([old]);
        if renewal {
            let mut grant = row();
            grant["authorized_at_version"] = json!("1.2.3");
            grant["authorization_evidence_ref"] =
                json!("cli_manifests/codex/reports/1.2.3/coverage.any.json");
            value["execution"]["debt_grants"] = json!([grant]);
            write(
                f.root.path(),
                NON_TUI_SUPPORT_DEBT_PATH,
                inventory(&[grant]),
            );
        } else {
            value["execution"]["debt_retired"] = json!(["legacy"]);
        }
        coverage(f.root.path(), renewal);
        f.record = seal(value);
        f.producer.current = f.record.clone();
        f.save();
        f.refuse("debt transitions lack independent semantic corroboration");
    }
}
#[test]
fn omitted_selected_target_cannot_be_hidden_by_e_population() {
    let mut f = Fixture::new();
    let mut value = serde_json::to_value(&f.record).unwrap();
    value["policy"]["selection"]["enrollment"]["targets"] = json!([LINUX, "aarch64-apple-darwin"]);
    value["obligations"]["targets"] = json!([LINUX, "aarch64-apple-darwin"]);
    f.record = seal(value);
    f.producer.current = f.record.clone();
    f.save();
    f.refuse("missing a due depth obligation");
}
#[test]
fn not_applicable_requires_current_explicit_policy_permission_and_never_waives_m() {
    for id in ["X1", "R1", "M"] {
        let mut f = Fixture::new();
        let mut value = serde_json::to_value(&f.record).unwrap();
        let result = value["execution"]["results"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|r| {
                if id == "M" {
                    r["obligation"]["capability"].is_string()
                } else {
                    r["obligation"]["id"] == id
                }
            })
            .unwrap();
        result["result"] = json!("not_applicable");
        result["rationale"] = json!("Current policy rationale");
        result["evidence"] = json!([]);
        f.record = seal(value);
        f.producer.current = f.record.clone();
        f.save();
        if id == "X1" {
            f.plan().unwrap();
        } else {
            f.refuse("not independently satisfied");
        }
    }
}
#[test]
fn unchanged_closeout_in_unrelated_depth_agent_is_not_reaccepted() {
    let f = Fixture::new();
    write(f.root.path(), CLOSEOUT, &f.closeout);
    // A different agent's ordinary HANDOFF update must not reaccept Codex's existing closeout.
    let proposal = Proposal::capture(f.root.path()).unwrap();
    let plans = proposal
        .plan(
            f.root.path(),
            vec![(
                "docs/agents/lifecycle/opencode-maintenance/HANDOFF.md".into(),
                b"ordinary closeout docs".to_vec(),
            )],
        )
        .unwrap();
    assert_eq!(plans.len(), 1);
}
#[test]
fn durable_old_closeout_is_attributed_when_new_ordinary_request_replaces_its_binding() {
    let mut f = Fixture::new();
    write(f.root.path(), CLOSEOUT, &f.closeout);
    f.record
        .acceptance_entries
        .push(AcceptanceEntry::MaintenanceCloseout {
            closeout: FileRef {
                path: CLOSEOUT.into(),
                sha256: artifacts::digest(std::str::from_utf8(&f.closeout).unwrap()),
            },
            record_sha256: f.record.stated_sha256().unwrap(),
        });
    f.save();
    let request = format!("agent_id = \"codex\"\nrequest_commit = \"abc\"\nrequest_recorded_at = \"{STAMP}\"\ntrigger_kind = \"upstream_release_detected\"\nopened_from = \"docs/basis.md\"\n[detected_release]\ntarget_version = \"1.2.4\"\n");
    write(f.root.path(), REQUEST, &request);
    let mut next: Value = serde_json::from_slice(&f.closeout).unwrap();
    next["request_sha256"] = json!(artifacts::digest(&request));
    let proposal = Proposal::capture(f.root.path()).unwrap();
    let plans = proposal
        .plan(
            f.root.path(),
            vec![(CLOSEOUT.into(), serde_json::to_vec(&next).unwrap())],
        )
        .unwrap();
    assert_eq!(
        plans.len(),
        1,
        "ordinary replacement must preserve the retained record/entries"
    );
    crate::workspace_mutation::apply_mutations(f.root.path(), &plans).unwrap();
    assert_eq!(
        fs::read(f.root.path().join(RECORD)).unwrap(),
        serde_json::to_vec_pretty(&f.record).unwrap()
    );
}
#[test]
fn replaced_request_without_durable_old_ownership_refuses() {
    let f = Fixture::new();
    write(f.root.path(), CLOSEOUT, &f.closeout);
    write(f.root.path(), REQUEST, "agent_id = \"codex\"\n");
    let error = f.plan().unwrap_err();
    assert!(error.contains("generation scope unresolved"), "{error}");
    assert_eq!(fs::read(f.root.path().join(CLOSEOUT)).unwrap(), f.closeout);
}
#[test]
fn tracked_producer_input_outside_standard_scopes_is_rechecked() {
    let f = Fixture::new();
    write(f.root.path(), "scripts/producer.sh", "original");
    git(f.root.path(), &["add", "scripts/producer.sh"]);
    let proposal = Proposal::capture(f.root.path()).unwrap();
    write(
        f.root.path(),
        "scripts/producer.sh",
        "concurrent producer edit",
    );
    let error = proposal
        .plan_with(
            f.root.path(),
            vec![(CLOSEOUT.into(), f.closeout.clone())],
            |root, before, after, paths| plan_with(root, before, after, paths, &f.producer),
        )
        .unwrap_err();
    assert!(error.contains("inputs changed"), "{error}");
    assert!(!f.root.path().join(CLOSEOUT).exists());
}
#[cfg(unix)]
#[test]
fn existing_artifact_mode_and_bytes_survive_refusal_and_success_preserves_mode() {
    use std::os::unix::fs::PermissionsExt;
    let mut f = Fixture::new();
    write(f.root.path(), CLOSEOUT, &f.closeout);
    fs::set_permissions(
        f.root.path().join(CLOSEOUT),
        fs::Permissions::from_mode(0o640),
    )
    .unwrap();
    let previous = f.closeout.clone();
    let mut replacement: Value = serde_json::from_slice(&f.closeout).unwrap();
    replacement["recorded_at"] = json!("2026-10-23T13:00:00Z");
    f.closeout = serde_json::to_vec_pretty(&replacement).unwrap();
    f.producer.proved = false;
    let error = f.plan().unwrap_err();
    assert!(error.contains("not independently satisfied"));
    assert_eq!(fs::read(f.root.path().join(CLOSEOUT)).unwrap(), previous);
    assert_eq!(
        fs::metadata(f.root.path().join(CLOSEOUT))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o640
    );
    f.producer.proved = true;
    let plans = f.plan().unwrap();
    crate::workspace_mutation::apply_mutations(f.root.path(), &plans).unwrap();
    assert_eq!(fs::read(f.root.path().join(CLOSEOUT)).unwrap(), f.closeout);
    assert_eq!(
        fs::metadata(f.root.path().join(CLOSEOUT))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o640
    );
}

#[test]
fn public_proposal_cannot_replace_a_record_or_accept_without_production_binding_proof() {
    let f = Fixture::new();
    let proposal = Proposal::capture(f.root.path()).unwrap();
    let error = proposal
        .plan(
            f.root.path(),
            vec![(RECORD.into(), b"forged record".to_vec())],
        )
        .unwrap_err();
    assert!(error.contains("unowned output"), "{error}");
    let error = proposal
        .plan(f.root.path(), vec![(CLOSEOUT.into(), f.closeout.clone())])
        .unwrap_err();
    assert!(
        error.contains("missing policy for retained depth record"),
        "{error}"
    );
    assert!(!f.root.path().join(CLOSEOUT).exists());
}

#[test]
fn integration_tip_movement_during_admission_refuses_without_delivering() {
    struct Moving<'a>(&'a Producer);
    impl Host for Moving<'_> {
        fn bindings(
            &self,
            root: &Path,
            record: &DepthRecord,
        ) -> Result<(PolicyPart, ObligationsPart), String> {
            git(root, &["update-ref", "refs/remotes/origin/staging", "HEAD"]);
            self.0.bindings(root, record)
        }
        fn evidence(&self, root: &Path, record: &DepthRecord) -> Result<EvidenceVerdicts, String> {
            self.0.evidence(root, record)
        }
    }
    let f = Fixture::new();
    git(
        f.root.path(),
        &[
            "commit",
            "--allow-empty",
            "-qm",
            "new integration candidate",
        ],
    );
    let proposal = Proposal::capture(f.root.path()).unwrap();
    let error = proposal
        .plan_with(
            f.root.path(),
            vec![(CLOSEOUT.into(), f.closeout.clone())],
            |root, before, after, paths| {
                plan_with(root, before, after, paths, &Moving(&f.producer))
            },
        )
        .unwrap_err();
    assert!(error.contains("integration tip moved"), "{error}");
    assert!(!f.root.path().join(CLOSEOUT).exists());
    assert_eq!(
        fs::read(f.root.path().join(RECORD)).unwrap(),
        serde_json::to_vec_pretty(&f.record).unwrap()
    );
}
