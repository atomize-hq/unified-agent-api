use super::*;
#[test]
fn pending_and_unauthorized_native_records_do_not_become_success() {
    let mut f = NativeFixture::new();
    f.install(30, CaseOutcome::Passed);
    f.select(0);
    assert_eq!(f.evaluate().unwrap().result, DepthResult::Verified);
    let completion = f.attempts[1].completion.clone();
    f.attempts[1].completion =
        json!({"id":40,"body":"ordinary comment","user":{"id":7,"login":"maintainer"}});
    f.comments();
    assert!(f.evaluate().is_err());
    f.attempts[1].completion = completion;
    f.comments();
    let path = format!(
        "{}/issues/8/comments?per_page=100&page=1",
        f.p.repository.name
    );
    let rows = f.api.values.get_mut(&path).unwrap().as_array_mut().unwrap();
    let mut rogue = rows[0].clone();
    rogue["id"] = json!(50);
    rogue["user"] = json!({"id":9,"login":"outsider"});
    rows.push(rogue);
    assert_eq!(f.evaluate().unwrap().result, DepthResult::Verified);
}
#[test]
fn unknown_context_or_tampered_current_proof_refuses_native_admission() {
    let mut f = NativeFixture::new();
    f.p.context_sha256 = "0".repeat(64);
    assert_ne!(
        f.evaluate().map(|v| v.result).ok(),
        Some(DepthResult::Verified)
    );
    let f = NativeFixture::new();
    fs::write(
        f.f.root
            .path()
            .join(&f.attempts[0].receipt.cases.raw_output.path),
        "changed native output",
    )
    .unwrap();
    assert!(f.evaluate().is_err());
}
#[test]
fn terminal_failures_and_nonterminal_native_outcomes_are_preserved() {
    for (outcome, expected) in [
        (CaseOutcome::Failed, DepthResult::Failed),
        (CaseOutcome::Aborted, DepthResult::Unverified),
        (CaseOutcome::Skipped, DepthResult::Unverified),
    ] {
        let f = NativeFixture::new_outcome(outcome);
        assert_eq!(f.evaluate().unwrap().result, expected);
    }
}
#[test]
fn native_host_identity_permission_tree_and_packet_guards_refuse() {
    for attack in 0..8 {
        let mut f = NativeFixture::new();
        let repo = &f.p.repository;
        match attack {
            0 => f.api.values.get_mut(&format!("{}/", repo.name)).unwrap()["id"] = json!(99),
            1 => {
                f.api
                    .values
                    .get_mut(&format!("{}/branches/staging", repo.name))
                    .unwrap()["protected"] = json!(false)
            }
            2 => {
                f.api
                    .values
                    .get_mut(&format!("{}/pulls/8", repo.name))
                    .unwrap()["head"]["repo"]["id"] = json!(99)
            }
            3 | 4 => {
                let value = f
                    .api
                    .values
                    .get_mut(&format!(
                        "{}/collaborators/maintainer/permission",
                        f.p.canonical.name
                    ))
                    .unwrap();
                if attack == 3 {
                    value["user"]["id"] = json!(99);
                } else {
                    value["permission"] = json!("read");
                }
            }
            5 => {
                let oid = git(
                    f.f.root.path(),
                    &["rev-parse", &format!("{}^{{tree}}", f.attempts[0].proof)],
                );
                f.api
                    .values
                    .get_mut(&format!("{}/git/trees/{oid}?recursive=1", repo.name))
                    .unwrap()["truncated"] = json!(true);
            }
            6 => {
                f.api.values.remove(&format!(
                    "{}/git/commits/{}",
                    f.p.canonical.name, f.approved
                ));
            }
            7 => {
                let key = format!("{}/issues/8/comments?per_page=100&page=1", repo.name);
                let value = f.api.values.get_mut(&key).unwrap().as_array_mut().unwrap();
                value.push(value[0].clone());
            }
            _ => unreachable!(),
        }
        assert!(f.evaluate().is_err(), "attack {attack}");
    }
}
#[test]
fn closing_population_movement_is_not_a_successful_snapshot() {
    let f = NativeFixture::new();
    let key = format!(
        "{}/issues/8/comments?per_page=100&page=1",
        f.p.repository.name
    );
    let original = f.api.values[&key].clone();
    let mut changed = original.clone();
    changed
        .as_array_mut()
        .unwrap()
        .push(json!({"id":50,"body":"new host record"}));
    f.api
        .sequences
        .borrow_mut()
        .insert(key, [original, changed].into());
    assert_eq!(
        f.evaluate().unwrap_err(),
        "native host population/source/authority changed"
    );
}
#[test]
fn native_profile_and_current_source_cannot_weaken_the_due_set_or_target_pair() {
    for attack in 0..5 {
        let mut f = NativeFixture::new();
        match attack {
            0 => {
                f.p.specs.pop_first();
            }
            1 => {
                f.f.current.record.obligations.as_mut().unwrap().operations[0]
                    .obligations
                    .pop();
                seal(&mut f.f.current.record);
            }
            2 => f.p.environment.target = f.p.native.target.clone(),
            3 => {
                write(
                    f.f.root.path(),
                    "crates/codex/src/lib.rs",
                    "changed production source",
                );
                git(f.f.root.path(), &["add", "."]);
            }
            4 => f.p.context_intent_sha256 = "0".repeat(64),
            _ => unreachable!(),
        }
        assert_ne!(
            f.evaluate().map(|v| v.result).ok(),
            Some(DepthResult::Verified),
            "attack {attack}"
        );
    }
}
#[test]
fn native_completion_attestations_are_required_individually() {
    for field in [
        "execution_attested",
        "outcomes_attested",
        "registration_order_attested",
        "completeness_attested",
    ] {
        let mut f = NativeFixture::new();
        let body = f.attempts[0].completion["body"].as_str().unwrap();
        let raw = body
            .split_once("```depth-native-finish-v1\n")
            .unwrap()
            .1
            .split_once("\n```")
            .unwrap()
            .0;
        let mut v: serde_json::Value = serde_json::from_str(raw).unwrap();
        v[field] = json!(false);
        f.attempts[0].completion["body"] = json!(body.replace(raw, &v.to_string()));
        f.comments();
        assert_eq!(
            f.evaluate().unwrap_err(),
            "native execution/completeness attestation absent",
            "{field}"
        );
    }
}
#[test]
fn immutable_aggregate_claims_cannot_filter_obligations_cases_or_native_facts() {
    for attack in 0..8 {
        let mut f = NativeFixture::new();
        f.install_with(30, CaseOutcome::Passed, |plan, claims| match attack {
            0 => {
                claims.pop();
            }
            1 => {
                plan.requested[0].name = "unapproved-case".into();
            }
            2 => {
                plan.native.executable_sha256 = "0".repeat(64);
            }
            3 => {
                claims
                    .last_mut()
                    .unwrap()
                    .provenance
                    .retain(|p| p.kind != ProvenanceKind::Binary);
            }
            4 => {
                plan.build.host_target = plan.native.target.clone();
                for c in claims {
                    c.build = plan.build.clone();
                }
            }
            5 => {
                claims.last_mut().unwrap().cases.executed.clear();
            }
            6 => {
                claims.last_mut().unwrap().mapping.assertions[0].counterfactual =
                    "unapproved assertion meaning".into();
            }
            7 => {
                plan.requested[0].executable_sha256 = "0".repeat(64);
                for c in claims {
                    c.cases.requested[0] = plan.requested[0].clone();
                    c.cases.executed[0].case = plan.requested[0].clone();
                    c.mapping.assertions[0].case = plan.requested[0].clone();
                }
            }
            _ => unreachable!(),
        });
        // The unfavorable bundle stays in discovery even when E selects the older good result.
        f.select(0);
        assert_eq!(
            f.evaluate().unwrap().result,
            DepthResult::Unverified,
            "unselected {attack}"
        );
        f.select(1);
        assert!(f.evaluate().is_err(), "selected {attack}");
    }
}
