//! OF-P8 checks exercise the same helpers used before execution and during observation.
use super::*;

fn check(root: &Path, context: &RuntimeContext, contract: &InputContract) -> Result<(), Error> {
    contract.check_support_depth(root, &context.approval, &context.lifecycle_state)
}

fn add_obligations(root: &Path, context: &mut RuntimeContext) {
    let mut record: crate::depth_record::DepthRecord =
        serde_json::from_str(&read_string(&root.join(RECORD)).unwrap()).unwrap();
    let original: crate::depth_record::DepthRecord =
        serde_json::from_str(include_str!("../../tests/fixtures/depth-core/record.json")).unwrap();
    let mut obligations = original.obligations.unwrap();
    obligations.policy_identity = record.policy.identity.clone();
    obligations.identity = crate::depth_record::part_identity(&obligations).unwrap();
    record.obligations = Some(obligations);
    context.lifecycle_state.support_depth = Some(
        life::Preserved::new(life::LifecycleDepth {
            policy_identity: record.policy.identity.clone(),
            obligations_identity: record.obligations.as_ref().map(|o| o.identity.clone()),
        })
        .unwrap(),
    );
    context.input_contract.support_depth =
        SupportDepthContract::copy_from(&context.approval, &context.lifecycle_state).unwrap();
    write(root, RECORD, bytes(&record).unwrap());
    write(root, STATE, bytes(&context.lifecycle_state).unwrap());
}

#[test]
fn create_candidate_input_identity_round_trips_with_and_without_obligations() {
    let (root, mut context) = setup(true);
    for obligations in [false, true] {
        if obligations {
            add_obligations(root.path(), &mut context);
        }
        let encoded = serde_json::to_value(&context.input_contract).unwrap();
        let object = context.lifecycle_state.support_depth.as_ref().unwrap();
        let mut expected = serde_json::json!({
            "version": context.approval.support_depth_version,
            "policy_identity": object.policy_identity,
        });
        if let Some(identity) = &object.obligations_identity {
            expected["obligations_identity"] = identity.clone().into();
        }
        assert_eq!(encoded["support_depth"], expected);
        let decoded: InputContract = serde_json::from_value(encoded).unwrap();
        check(root.path(), &context, &decoded).unwrap();
    }
}

#[test]
fn create_candidate_input_identity_refuses_missing_version_policy_and_obligations_bindings() {
    for obligations in [false, true] {
        let (root, mut context) = setup(true);
        if obligations {
            add_obligations(root.path(), &mut context);
        }
        for field in [
            "support_depth",
            "version",
            "policy_identity",
            "obligations_identity",
        ] {
            let mut value = serde_json::to_value(&context.input_contract).unwrap();
            if field == "support_depth" {
                value.as_object_mut().unwrap().remove(field);
            } else {
                value["support_depth"][field] = if field == "version" {
                    "9.9.9".into()
                } else {
                    "f".repeat(64).into()
                };
            }
            let changed: InputContract = serde_json::from_value(value).unwrap();
            let error = check(root.path(), &context, &changed).unwrap_err();
            assert!(error.to_string().contains("support_depth"), "{error}");
        }
        if obligations {
            let mut missing_o = context.input_contract.clone();
            missing_o
                .support_depth
                .as_mut()
                .unwrap()
                .obligations_identity = None;
            assert!(check(root.path(), &context, &missing_o).is_err());
        }
    }
}

#[test]
fn create_candidate_input_identity_requires_current_workflow_for_depth_only() {
    for depth in [false, true] {
        let (root, context) = setup(depth);
        let mut value = serde_json::to_value(&context.input_contract).unwrap();
        if !depth {
            value.as_object_mut().unwrap().remove("support_depth");
        }
        value["workflow_version"] = "runtime_follow_on_v1".into();
        let mut historical: InputContract = serde_json::from_value(value).unwrap();
        assert_eq!(check(root.path(), &context, &historical).is_ok(), !depth);
        historical.workflow_version = "runtime_follow_on_v999".into();
        assert!(check(root.path(), &context, &historical).is_err());
    }
}

#[test]
fn create_candidate_input_identity_refuses_record_selection_version_and_event_mismatches() {
    for field in [
        "version",
        "event",
        "policy",
        "missing_record",
        "missing_object",
    ] {
        let (root, mut context) = setup(true);
        let mut record: crate::depth_record::DepthRecord =
            serde_json::from_str(&read_string(&root.path().join(RECORD)).unwrap()).unwrap();
        match field {
            "version" => {
                record.policy.selection.enrollment.version = "9.9.9".into();
                record.policy.identity =
                    crate::depth_record::part_identity(&record.policy).unwrap();
            }
            "event" => {
                record.event = crate::depth_record::Event::Onboarding {
                    approval_commit: "a".repeat(40),
                    approval_recorded_at: "2024-01-01T00:00:00Z".into(),
                };
            }
            "policy" => {
                record.policy.rules_revision.push_str("-changed");
                record.policy.identity =
                    crate::depth_record::part_identity(&record.policy).unwrap();
            }
            "missing_object" => context.lifecycle_state.support_depth = None,
            _ => {}
        }
        write(root.path(), RECORD, bytes(&record).unwrap());
        if field == "missing_record" {
            fs::remove_file(root.path().join(RECORD)).unwrap();
        }
        assert!(
            check(root.path(), &context, &context.input_contract).is_err(),
            "{field}"
        );
    }
}

#[test]
fn create_candidate_observation_rechecks_changed_lifecycle_record_and_both_together() {
    for changed in ["state", "record", "both", "obligations"] {
        let (root, mut context) = setup(true);
        if changed == "obligations" {
            add_obligations(root.path(), &mut context);
        }
        check(root.path(), &context, &context.input_contract).unwrap();
        let candidate = Candidate::prepare(
            root.path(),
            &format!("{RUNTIME_RUNS_ROOT}/{RUN_ID}"),
            vec![], // Isolate the identity check from generic protected-file checks.
        )
        .unwrap();
        let _holder = candidate_execution::TestHolder::new(&candidate.root);
        let (_, observation) = candidate.execute_observed(|| {
            let mut state = context.lifecycle_state.clone();
            let mut record: crate::depth_record::DepthRecord =
                serde_json::from_str(&read_string(&candidate.root.join(RECORD)).unwrap()).unwrap();
            record.policy.rules_revision.push_str("-changed");
            record.policy.identity = crate::depth_record::part_identity(&record.policy).unwrap();
            if changed == "obligations" {
                // Even an unchanged P must refuse losing the frozen O.
                record = serde_json::from_str(&read_string(&candidate.root.join(RECORD)).unwrap())
                    .unwrap();
                record.obligations = None;
            }
            if changed != "state" {
                write(&candidate.root, RECORD, bytes(&record).unwrap());
            }
            if changed == "state" || changed == "both" {
                state.support_depth = Some(
                    life::Preserved::new(life::LifecycleDepth {
                        policy_identity: record.policy.identity,
                        obligations_identity: None,
                    })
                    .unwrap(),
                );
                write(&candidate.root, STATE, bytes(&state).unwrap());
            }
            Ok::<_, Error>(())
        });
        let observed = observation.unwrap();
        let error =
            check_observed_depth_identity(&candidate.root, &context, &context.input_contract)
                .unwrap_err();
        assert!(
            error.to_string().contains("support_depth"),
            "{changed}: {error}"
        );
        assert_eq!(
            tree::capture(root.path(), &Default::default()).unwrap(),
            candidate.before
        );
        // Refusal still finalizes a diagnostic-only tree; changed identities are never delivered.
        let finalized = candidate_execution::finalized_tree(
            &candidate,
            root.path(),
            Some(&observed),
            false,
            Tree::new(),
        )
        .unwrap();
        assert_eq!(finalized, candidate.before);
    }
}

#[test]
fn create_candidate_observation_accepts_unchanged_identity() {
    let (root, context) = setup(true);
    check_observed_depth_identity(root.path(), &context, &context.input_contract).unwrap();
}
