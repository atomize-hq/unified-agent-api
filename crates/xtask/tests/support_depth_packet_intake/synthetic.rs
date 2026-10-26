use super::*;
const SHARD: &str = "cli_manifests/codex/snapshots/1.2.3/x86_64-unknown-linux-musl.json";

#[test]
fn matching_rows_resolve_only_owned_units_and_add_parent_obligations() {
    let root = acquisition();
    let before = plan_freeze(root.path(), &request()).unwrap().unwrap();
    let plan = classify(root.path(), &request(), &["--output", "--json"]);
    let o = plan.record.obligations.as_ref().unwrap();
    assert_eq!(
        o.operations[0].surfaces,
        before.record.obligations.unwrap().operations[0].surfaces
    );
    assert_eq!(
        o.classification_required.len(),
        o.operations[0].surfaces.len() - 2
    );
    assert!(o
        .classification_required
        .iter()
        .all(|s| s.surface.surface_id != "--json" && s.surface.surface_id != "--output"));
    let obligations = serde_json::to_value(&o.operations[0].obligations).unwrap();
    for id in ["R1", "OUT3", "LC2", "OUT1", "LC1", "FX1"] {
        assert!(obligations.as_array().unwrap().contains(&json!(id)));
    }
    let p = plan.record.policy;
    policy(root.path(), |d| {
        d["operations"][0]["classifications"]
            .as_array_mut()
            .unwrap()
            .reverse()
    });
    let reordered = plan_freeze(root.path(), &request()).unwrap().unwrap();
    assert_ne!(p.identity, reordered.record.policy.identity);
    assert_eq!(p.templates, reordered.record.policy.templates);
    let rows = &reordered.record.policy.selection.enrollment.operations[0]
        .classifications
        .as_ref()
        .unwrap();
    assert_eq!(rows[0].surface.surface_id, "--json");
}

#[test]
fn invalid_classification_policies_refuse_with_and_without_acquisition() {
    for acquired in [false, true] {
        for case in [
            "unpaired-basis",
            "unpaired-rows",
            "digest",
            "duplicate",
            "foreign",
            "unowned",
            "excluded",
            "unsupported",
            "rationale",
            "forwarding",
            "extra-mode",
            "extra-target",
        ] {
            let root = if acquired {
                acquisition()
            } else {
                fixture::workspace("codex")
            };
            if acquired {
                classify(root.path(), &request(), &["--output"]);
            } else {
                policy(root.path(), |d| {
                    d["operations"][0]["classification_basis"] = json!("a".repeat(64));
                    d["operations"][0]["classifications"] = json!([{"surface":{"surface_kind":"flags","command_path":"codex run","surface_id":"--output"},"templates":["ADMIN"],"rationale":"File mutation"}]);
                });
            }
            policy(root.path(), |d| {
                let op = &mut d["operations"][0];
                match case {
                    "unpaired-basis" => {
                        op.as_object_mut().unwrap().remove("classifications");
                    }
                    "unpaired-rows" => {
                        op.as_object_mut().unwrap().remove("classification_basis");
                    }
                    "digest" => op["classification_basis"] = json!("bad"),
                    "duplicate" => {
                        let row = op["classifications"][0].clone();
                        op["classifications"].as_array_mut().unwrap().push(row);
                    }
                    "foreign" => {
                        op["classifications"][0]["surface"]["command_path"] = json!("other run")
                    }
                    "unowned" => {
                        op["classifications"][0]["surface"]["command_path"] =
                            json!("codex run nested")
                    }
                    "excluded" => op["classifications"][0]["templates"] = json!(["EXCLUDED"]),
                    "unsupported" => op["classifications"][0]["templates"] = json!(["SOMETHING"]),
                    "rationale" => op["classifications"][0]["rationale"] = json!("  "),
                    "forwarding" => {
                        op["templates"] = json!(["DIAG"]);
                        op["forwarding"] = json!(true);
                        op["classifications"][0]["templates"] = json!(["CORE"]);
                    }
                    "extra-mode" => {
                        let mut mode = d["modes"][0].clone();
                        mode["selector"] = json!("other");
                        mode["command_path"] = json!("codex other");
                        d["modes"].as_array_mut().unwrap().push(mode);
                        d["operations"][0]["serves"] = json!(["run", "other"]);
                    }
                    "extra-target" => d["targets"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!("aarch64-apple-darwin")),
                    _ => unreachable!(),
                }
            });
            assert!(
                plan_freeze(root.path(), &request()).is_err(),
                "{case}, acquired={acquired}"
            );
        }
    }
    let root = acquisition();
    classify(root.path(), &request(), &["--output"]);
    policy(root.path(), |d| {
        d["operations"][0]["classifications"][0]["surface"]["surface_id"] = json!("--not-acquired")
    });
    assert!(plan_freeze(root.path(), &request())
        .unwrap_err()
        .contains("unowned"));
}

#[test]
fn only_exact_presence_and_coherent_acquired_aliases_are_supported() {
    let root = acquisition();
    policy(root.path(), |d| {
        d["modes"][0]["predicates"][0]["flag"] = json!("-j")
    });
    let plan = plan_freeze(root.path(), &request()).unwrap().unwrap();
    assert!(plan.record.obligations.unwrap().operations[0]
        .surfaces
        .iter()
        .any(|s| s.surface_id == "--json"));
    for case in [
        "hidden",
        "false",
        "other",
        "nested",
        "prefix",
        "overlap",
        "value",
        "missing",
        "alias-collision",
    ] {
        let root = acquisition();
        match case {
            "hidden" => policy(root.path(), |d| {
                d["modes"][0]["predicates"][0]["flag"] = json!("--experimental-json")
            }),
            "false" => policy(root.path(), |d| {
                d["modes"][0]["predicates"][0]["value"] = json!("false")
            }),
            "other" => policy(root.path(), |d| {
                d["modes"][0]["predicates"][0]["flag"] = json!("--global")
            }),
            "nested" => policy(root.path(), |d| {
                d["modes"][0]["command_path"] = json!("codex run resume")
            }),
            "prefix" => {
                fixture::change_json(root.path(), SHARD, |v| {
                    v["commands"][1]["path"] = json!(["run", "nested"])
                });
                fixture::regenerate(root.path(), "codex");
            }
            "overlap" => policy(root.path(), |d| {
                let mut mode = d["modes"][0].clone();
                mode["selector"] = json!("other");
                mode["mode"] = json!("other");
                d["modes"].as_array_mut().unwrap().push(mode);
                d["operations"][0]["serves"] = json!(["run", "other"]);
            }),
            "value" => {
                fixture::change_json(root.path(), SHARD, |v| {
                    v["commands"][1]["flags"][1]["takes_value"] = json!(true);
                    v["commands"][1]["flags"][1]["value_name"] = json!("BOOL");
                });
                fixture::regenerate(root.path(), "codex");
            }
            "missing" => {
                fixture::change_json(root.path(), SHARD, |v| {
                    v["commands"][1]["flags"].as_array_mut().unwrap().pop();
                });
                fixture::regenerate(root.path(), "codex");
            }
            "alias-collision" => {
                fixture::change_json(root.path(), SHARD, |v| {
                    v["commands"][1]["flags"][0]["short"] = json!("-j")
                });
                fixture::regenerate(root.path(), "codex");
            }
            _ => unreachable!(),
        }
        assert!(plan_freeze(root.path(), &request()).is_err(), "{case}");
    }
}

#[test]
fn every_bound_semantic_regime_invalidates_reviewed_rows() {
    for case in [
        "binary",
        "features",
        "omissions",
        "usage",
        "alias",
        "arity",
        "repeatable",
        "original-global",
        "claim",
        "rules",
        "schema",
        "new-unit",
    ] {
        let root = acquisition();
        classify(root.path(), &request(), &["--output"]);
        match case {
            "claim" => fixture::change_json(
                root.path(),
                "cli_manifests/codex/wrapper_coverage.json",
                |v| v["coverage"][0]["flags"][1]["level"] = json!("explicit"),
            ),
            "rules" => fixture::change_json(root.path(), "cli_manifests/codex/RULES.json", |v| {
                v["description"] = json!("reviewed rules changed")
            }),
            "schema" => fixture::change_json(root.path(), "cli_manifests/codex/SCHEMA.json", |v| {
                v["description"] = json!("reviewed schema changed")
            }),
            _ => fixture::change_json(root.path(), SHARD, |v| match case {
                "binary" => v["binary"]["sha256"] = json!("b".repeat(64)),
                "features" => v["features"] = json!({"enabled_for_snapshot":["new-effect"]}),
                "omissions" => v["known_omissions"] = json!(["unobserved effect"]),
                "usage" => v["commands"][1]["usage"] = json!("run CHANGED_DEFAULT"),
                "alias" => v["commands"][1]["flags"][0]["short"] = json!("-q"),
                "arity" => {
                    v["commands"][1]["flags"][0]["takes_value"] = json!(false);
                    v["commands"][1]["flags"][0]
                        .as_object_mut()
                        .unwrap()
                        .remove("value_name");
                }
                "repeatable" => v["commands"][1]["flags"][0]["repeatable"] = json!(true),
                "original-global" => v["commands"][0]["flags"][0]["repeatable"] = json!(true),
                "new-unit" => v["commands"][1]["flags"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"long":"--new-effect","takes_value":false})),
                _ => unreachable!(),
            }),
        }
        fixture::regenerate(root.path(), "codex");
        let error = plan_freeze(root.path(), &request()).unwrap_err();
        assert!(
            error.contains("stale classification_basis"),
            "{case}: {error}"
        );
    }
}

#[test]
fn timestamps_preserve_basis_but_change_o_and_clear_e() {
    let root = acquisition();
    let mut plan = classify(root.path(), &request(), &["--output"]);
    let mut e = xtask::depth_record::ExecutionPart {
        identity: String::new(),
        policy_identity: plan.record.policy.identity.clone(),
        obligations_identity: plan.record.obligations.as_ref().unwrap().identity.clone(),
        results: vec![],
        reuse_bindings: vec![],
        debt_grants: vec![],
        debt_retired: vec![],
    };
    e.identity = xtask::depth_record::part_identity(&e).unwrap();
    plan.record.execution = Some(e);
    fixture::install(root.path(), "codex", &plan.record);
    assert_eq!(
        plan_freeze(root.path(), &request())
            .unwrap()
            .unwrap()
            .record,
        plan.record
    );
    fixture::change_json(root.path(), SHARD, |v| {
        v["collected_at"] = json!("2026-10-25T00:00:00Z")
    });
    fixture::change_json(
        root.path(),
        "cli_manifests/codex/wrapper_coverage.json",
        |v| {
            v["generated_at"] = json!("2026-10-25T00:00:00Z");
            v["wrapper_version"] = json!("regenerated");
        },
    );
    fixture::regenerate(root.path(), "codex");
    let next = plan_freeze(root.path(), &request()).unwrap().unwrap();
    assert_eq!(
        next.classification_bases[0].sha256,
        plan.classification_bases[0].sha256
    );
    assert_eq!(next.record.policy, plan.record.policy);
    assert_ne!(next.record.obligations, plan.record.obligations);
    assert!(next.record.execution.is_none());
}

#[test]
fn selected_json_refuses_union_arity_conflicts_and_root_alias_collisions() {
    for case in ["union-arity", "root-alias", "root-arity"] {
        let root = acquisition();
        if case == "union-arity" {
            let mut shard: Value =
                serde_json::from_slice(&fs::read(root.path().join(SHARD)).unwrap()).unwrap();
            shard["binary"]["target_triple"] = json!("aarch64-apple-darwin");
            shard["binary"]["platform"] = json!({"os":"macos","arch":"aarch64"});
            shard["commands"][1]["flags"][1]["takes_value"] = json!(true);
            shard["commands"][1]["flags"][1]["value_name"] = json!("BOOL");
            fixture::write(
                root.path(),
                "cli_manifests/codex/snapshots/1.2.3/aarch64-apple-darwin.json",
                shard.to_string(),
            );
        } else {
            fixture::change_json(root.path(), SHARD, |v| {
                let flag = if case == "root-alias" {
                    json!({"long":"--other","short":"-j","takes_value":false})
                } else {
                    json!({"long":"--json","short":"-j","takes_value":true,"value_name":"BOOL"})
                };
                v["commands"][0]["flags"].as_array_mut().unwrap().push(flag);
            });
        }
        fixture::regenerate(root.path(), "codex");
        assert!(plan_freeze(root.path(), &request()).is_err(), "{case}");
    }
}

#[test]
fn omitted_and_present_empty_omissions_are_distinct_and_unknown_fields_refuse() {
    let root = acquisition();
    let absent = plan_freeze(root.path(), &request())
        .unwrap()
        .unwrap()
        .classification_bases[0]
        .sha256
        .clone();
    fixture::change_json(root.path(), SHARD, |v| v["known_omissions"] = json!([]));
    fixture::regenerate(root.path(), "codex");
    let present = plan_freeze(root.path(), &request())
        .unwrap()
        .unwrap()
        .classification_bases[0]
        .sha256
        .clone();
    assert_ne!(absent, present);
    fixture::change_json(root.path(), SHARD, |v| {
        v.as_object_mut().unwrap().remove("known_omissions");
    });
    fixture::regenerate(root.path(), "codex");
    assert_eq!(
        absent,
        plan_freeze(root.path(), &request())
            .unwrap()
            .unwrap()
            .classification_bases[0]
            .sha256
    );
    // Even a widened input schema cannot cause the projection to silently drop a new row field.
    fixture::change_json(root.path(), "cli_manifests/codex/SCHEMA.json", |v| {
        v["$defs"]["FlagSnapshotV1"]["additionalProperties"] = json!(true)
    });
    fixture::change_json(root.path(), SHARD, |v| {
        v["commands"][1]["flags"][0]["unknown_effect"] = json!("mutation")
    });
    fixture::regenerate(root.path(), "codex");
    assert!(plan_freeze(root.path(), &request())
        .unwrap_err()
        .contains("unprojectable"));
}

#[test]
fn numeric_semantic_feature_fields_refuse_instead_of_lossy_projection() {
    let root = acquisition();
    fixture::change_json(root.path(), SHARD, |v| {
        v["features"] = json!({"future_numeric_regime":12})
    });
    fixture::regenerate(root.path(), "codex");
    assert!(plan_freeze(root.path(), &request())
        .unwrap_err()
        .contains("number"));
}

#[test]
fn new_intake_cannot_treat_initial_only_mapping_as_complete_or_p_only_fallback() {
    for acquired in [false, true] {
        for classified in [false, true] {
            let root = if acquired {
                acquisition()
            } else {
                fixture::workspace("codex")
            };
            policy(root.path(), |d| {
                d["operations"][0]["shared_mapping"] = json!("mapped");
                d["capabilities"] = json!([{"id":"agent_api.run","operations":["run"]}]);
                d["modes"][0]["predicates"] = json!([]);
            });
            // Legacy structural/unqualified fixtures keep their existing behavior.
            assert!(plan_freeze(root.path(), &request()).is_ok());
            policy(root.path(), |d| {
                if classified {
                    d["operations"][0]["classification_basis"] = json!("a".repeat(64));
                    d["operations"][0]["classifications"] = json!([]);
                } else {
                    d["modes"][0]["predicates"] = json!([{"flag":"--json"}]);
                }
            });
            let error = plan_freeze(root.path(), &request()).unwrap_err();
            assert!(
                error.contains("complete mapped flow policy"),
                "acquired={acquired},classified={classified}: {error}"
            );
        }
    }
}
