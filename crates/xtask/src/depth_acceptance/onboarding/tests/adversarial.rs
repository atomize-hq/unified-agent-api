use super::*;

#[test]
fn proving_prepared_wrong_approval_path_and_changed_artifacts_refuse() {
    for (key, value, expected) in [
        (
            "state",
            json!("prepared"),
            "onboarding proving requires exact closed closeout with passed preflight",
        ),
        (
            "preflight_passed",
            json!(false),
            "onboarding proving requires exact closed closeout with passed preflight",
        ),
        (
            "approval_ref",
            json!("docs/foreign-approved-agent.toml"),
            "onboarding proving closeout is not bound to its exact approval",
        ),
        (
            "approval_sha256",
            json!("b".repeat(64)),
            "onboarding proving closeout is not bound to its exact approval",
        ),
        (
            "maintenance_settlement",
            Value::Null,
            "onboarding proving maintenance settlement differs from exact approval",
        ),
    ] {
        let mut f = Fixture::new(Kind::Proving);
        let mut closeout: Value = serde_json::from_str(&f.after[CLOSEOUT]).unwrap();
        closeout[key] = value;
        f.after.insert(CLOSEOUT.into(), encode(&closeout));
        f.refuse(expected);
    }
    let f = Fixture::new(Kind::Proving);
    let original = f.accept().unwrap();
    for key in ["recorded_at", "explicit_none_reason"] {
        let mut f = Fixture::new(Kind::Proving);
        let mut closeout: Value = serde_json::from_str(&f.after[CLOSEOUT]).unwrap();
        closeout[key] = if key == "recorded_at" {
            json!("2026-11-06T00:00:00Z")
        } else {
            json!("Changed exact artifact")
        };
        f.after.insert(CLOSEOUT.into(), encode(&closeout));
        assert_ne!(f.accept().unwrap(), original);
        f.record.acceptance_entries.push(original.clone());
        f.save();
        assert_eq!(
            effects::check(
                &source(&f.before),
                &source(&f.after),
                &f.view(),
                &f.record,
                f.kind
            )
            .unwrap_err(),
            "onboarding batch has an entry outside its owning path"
        );
    }
}

#[test]
fn published_and_closed_assertions_cannot_be_retargeted_to_foreign_paths() {
    for kind in [Kind::Publication, Kind::Proving] {
        let mut f = Fixture::new(kind);
        let mut state: LifecycleState = serde_json::from_str(&f.after[STATE]).unwrap();
        state.publication_packet_path =
            Some("docs/agents/lifecycle/foreign/governance/publication-ready.json".into());
        f.after.insert(STATE.into(), encode(&state));
        f.refuse("onboarding publication is not bound to its exact packet");
    }
    let mut f = Fixture::new(Kind::Proving);
    let mut state: LifecycleState = serde_json::from_str(&f.after[STATE]).unwrap();
    state.closeout_baseline_path =
        Some("docs/agents/lifecycle/foreign/governance/proving-run-closeout.json".into());
    f.after.insert(STATE.into(), encode(&state));
    f.refuse("onboarding proving requires its exact closeout path");
}

#[test]
fn drift_clearance_and_frozen_object_byte_changes_refuse() {
    let mut f = Fixture::new(Kind::Proving);
    let mut state: LifecycleState = serde_json::from_str(&f.before[STATE]).unwrap();
    state.side_states = vec![SideState::Drifted];
    f.before.insert(STATE.into(), encode(&state));
    f.refuse("onboarding cannot clear maintenance drift");
    let mut f = Fixture::new(Kind::Publication);
    let raw = f.after[STATE].clone();
    f.after.insert(
        STATE.into(),
        raw.replace("\"support_depth\": {", "\"support_depth\": { "),
    );
    f.refuse("onboarding lifecycle is not the exact approval/P/O binding");
}

#[test]
fn public_in1_retains_its_onboarding_effect_refusal() {
    // The public route guards remain the authorization boundary in this slice.
    let f = Fixture::new(Kind::Publication);
    let old = View::with_registry(source(&f.before), &f.view().registry, AGENT).unwrap();
    let new = f.view();
    struct Unreachable;
    impl super::super::super::manual::Host for Unreachable {
        fn bindings(
            &self,
            _: &Path,
            _: &DepthRecord,
        ) -> Result<(PolicyPart, ObligationsPart), String> {
            panic!("onboarding reached maintenance bindings")
        }
        fn evidence(&self, _: &Path, _: &DepthRecord) -> Result<EvidenceVerdicts, String> {
            panic!("onboarding reached maintenance evidence")
        }
    }
    assert_eq!(
        super::super::super::integration::admit(
            &repo(),
            &old,
            &new,
            &source(&f.before),
            &Unreachable,
            &Unreachable
        )
        .unwrap_err(),
        "unsupported onboarding lifecycle authority at integration"
    );
}

fn debt_row() -> Value {
    json!({"row_id":"legacy", "agent_id":AGENT, "surface_kind":"flags", "command_path":"depth_fixture run", "surface_id":"--legacy", "current_reason":"Existing seam blocker", "blocker_class":"requires_new_architectural_seam", "owner":"wrappers", "milestone":"bounded follow-on", "follow_on":"TODOS.md#legacy", "evidence_ref":"historical.md", "scope_target_triples":["x86_64-unknown-linux-gnu"], "authorized_at_version":VERSION, "authorization_evidence_ref":"cli_manifests/depth_fixture/reports/0.2.0/coverage.any.json"})
}

#[test]
fn grants_renewals_baseline_debt_and_current_inventory_are_all_refused() {
    for kind in [Kind::Publication, Kind::Proving] {
        let mut f = Fixture::new(kind);
        f.record
            .execution
            .as_mut()
            .unwrap()
            .debt_grants
            .push(serde_json::from_value(debt_row()).unwrap());
        reseal(&mut f.record);
        f.save();
        f.refuse("onboarding debt grants, retirements and baseline debt are unsupported");
        let mut f = Fixture::new(kind);
        f.record
            .policy
            .debt_baseline
            .push(serde_json::from_value(debt_row()).unwrap());
        f.replace_frozen();
        f.refuse("onboarding debt grants, retirements and baseline debt are unsupported");
        let mut renewal = Fixture::new(kind);
        let mut baseline = debt_row();
        baseline["authorized_at_version"] = json!("0.1.0");
        baseline["authorization_evidence_ref"] =
            json!("cli_manifests/depth_fixture/reports/0.1.0/coverage.any.json");
        renewal
            .record
            .policy
            .debt_baseline
            .push(serde_json::from_value(baseline).unwrap());
        renewal
            .record
            .execution
            .as_mut()
            .unwrap()
            .debt_grants
            .push(serde_json::from_value(debt_row()).unwrap());
        renewal.replace_frozen();
        renewal.refuse("onboarding debt grants, retirements and baseline debt are unsupported");
        let mut f = Fixture::new(kind);
        let mut inventory =
            "### `support-debt-authorization-contract-target-version-v1`\n\n### `legacy`\n"
                .to_owned();
        for (key, value) in debt_row().as_object().unwrap() {
            if key == "row_id" {
                continue;
            }
            let text = if let Some(values) = value.as_array() {
                values
                    .iter()
                    .map(|v| v.as_str().unwrap())
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                value.as_str().unwrap().into()
            };
            inventory += &format!("- `{key}`: `{text}`\n");
        }
        let path = crate::agent_maintenance::support_audit::NON_TUI_SUPPORT_DEBT_PATH;
        f.before.insert(path.into(), inventory.clone());
        f.after.insert(path.into(), inventory);
        f.refuse("onboarding requires a debt-free baseline and inventory");
    }
}

#[test]
fn missing_execution_and_inconsistent_execution_bindings_refuse() {
    for kind in [Kind::Publication, Kind::Proving] {
        let mut f = Fixture::new(kind);
        f.record.execution = None;
        f.save();
        f.refuse("onboarding requires current E");
        let mut f = Fixture::new(kind);
        let execution = f.record.execution.as_mut().unwrap();
        execution.obligations_identity = "b".repeat(64);
        execution.identity = part_identity(execution).unwrap();
        f.save();
        f.refuse("onboarding P/O/E binding is stale");
    }
}

#[test]
fn metadata_without_a_record_and_min_supported_cannot_escape_the_batch() {
    for kind in [Kind::Publication, Kind::Proving] {
        let mut f = Fixture::new(kind);
        f.after.insert(format!("{ROOT}/versions/0.3.0.json"),encode(&json!({"schema_version":1,"semantic_version":"0.3.0","status":"supported","validation":{"passed_targets":[]}})));
        assert_eq!(
            effects::check(
                &source(&f.before),
                &source(&f.after),
                &f.view(),
                &f.record,
                f.kind
            )
            .unwrap_err(),
            "onboarding batch has acceptance outside its owned generation"
        );
        let mut f = Fixture::new(kind);
        f.after
            .insert(format!("{ROOT}/min_supported.txt"), "0.2.0\n".into());
        assert_eq!(
            effects::check(
                &source(&f.before),
                &source(&f.after),
                &f.view(),
                &f.record,
                f.kind
            )
            .unwrap_err(),
            "onboarding cannot change pointer file cli_manifests/depth_fixture/min_supported.txt"
        );
    }
}
