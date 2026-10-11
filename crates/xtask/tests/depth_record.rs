use serde_json::{json, Value};
use xtask::depth_record::{part_identity, AcceptanceEntry, DepthRecord};

const SHA: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn debt_row(row_id: &str) -> Value {
    json!({
        "row_id": row_id, "agent_id": "aider", "surface_kind": "flags",
        "command_path": "aider run", "surface_id": "--watch", "current_reason": "r",
        "blocker_class": "b", "owner": "o", "milestone": "m", "follow_on": "f",
        "evidence_ref": "e", "scope_target_triples": ["darwin-arm64", "linux-x64"],
        "authorized_at_version": "1.2.2", "authorization_evidence_ref": "a"
    })
}

fn surface() -> Value {
    json!({ "surface_kind": "flags", "command_path": "aider run", "surface_id": "--output-format" })
}

fn obligation(id: &str, target: &str) -> Value {
    json!({ "operation": "run", "id": id, "target": target })
}

/// A record with every part produced. Identities are placeholders until `sealed` computes them.
fn record() -> Value {
    json!({
        "schema_version": 1,
        "event": {
            "lifecycle_path": "maintenance", "request_commit": "abc", "opened_from": "w.yml",
            "request_recorded_at": "2026-10-10T00:00:00Z",
            "trigger_kind": "upstream_release_detected"
        },
        "policy": {
            "identity": SHA,
            "selection": { "agent_id": "aider", "enrollment": {
                "lifecycle_path": "maintenance",
                "version": "1.2.3",
                "targets": ["linux-x64", "darwin-arm64"],
                "modes": [
                    { "selector": "run-json", "command_path": "aider run", "mode": "headless",
                      "predicates": [{ "flag": "--output-format", "value": "json" }] },
                    { "selector": "tui", "command_path": "aider", "mode": "interactive",
                      "predicates": [], "targets": ["linux-x64"] }
                ],
                "operations": [
                    { "id": "run", "serves": ["run-json"], "templates": ["CORE"],
                      "shared_mapping": "mapped", "not_applicable_permitted": ["X1"],
                      "promise": { "r1": "r", "v1": "v", "g1": "g", "e1": "e", "a1": "a",
                                   "x1": "x", "x2": "x", "x3": "x", "x4": "x" } },
                    { "id": "tui-excluded", "serves": ["tui"], "templates": ["EXCLUDED"],
                      "not_applicable_permitted": [], "rationale": "interactive only" }
                ],
                "capabilities": [{ "id": "agent_api.run", "operations": ["run"] }]
            } },
            "base": ["A1", "E1", "G1", "R1", "V1", "X1", "X2", "X3", "X4"],
            "templates": [
                { "template": "EXCLUDED", "obligations": ["EX1"] },
                { "template": "CORE", "obligations": ["LC2", "OUT3"] }
            ],
            "path_enabled": true,
            "debt_baseline": [debt_row("row-1")]
        },
        "obligations": {
            "identity": SHA,
            "policy_identity": SHA,
            "targets": ["darwin-arm64", "linux-x64"],
            "acquired_inputs": [{ "path": "cli_manifests/aider/snapshots/1.2.3/union.json",
                                  "sha256": SHA }],
            "operations": [{ "id": "run", "surfaces": [surface()],
                             "obligations": ["LC2", "OUT3", "R1"] }],
            "classification_required": [{ "operation": "run", "surface": surface() }],
            "acceptance_work": [{ "surface": surface(), "targets": ["linux-x64"] }]
        },
        "execution": {
            "identity": SHA,
            "policy_identity": SHA,
            "obligations_identity": SHA,
            "results": [
                { "obligation": { "operation": "run", "capability": "agent_api.run",
                                  "target": "linux-x64" },
                  "result": "unverified", "evidence": [] },
                { "obligation": obligation("X1", "linux-x64"), "result": "not_applicable",
                  "rationale": "no credential is read", "evidence": [] },
                { "obligation": obligation("R1", "linux-x64"), "result": "verified",
                  "evidence": [{ "path": "crates/aider/tests/run.rs", "sha256": SHA }] }
            ],
            "reuse_bindings": [{ "obligation": obligation("R1", "linux-x64"),
                                 "policy_identity": SHA, "obligations_identity": SHA }],
            "debt_grants": [debt_row("row-1")],
            "debt_retired": ["row-2"]
        },
        "acceptance_entries": [
            { "backs": "promotion", "targets": ["linux-x64"], "record_sha256": SHA },
            { "backs": "publication", "record_sha256": SHA,
              "packet": { "path": "docs/agents/lifecycle/aider/publication.json", "sha256": SHA } },
            { "backs": "maintenance_closeout", "record_sha256": SHA,
              "closeout": { "path": "docs/agents/lifecycle/aider/c.json", "sha256": SHA } }
        ]
    })
}

fn sealed(mut record: Value) -> String {
    for part in ["policy", "obligations", "execution"] {
        if record[part].is_object() {
            record[part]["identity"] = json!(part_identity(&record[part]).unwrap());
        }
    }
    record.to_string()
}

fn parse(raw: &str) -> Result<DepthRecord, String> {
    DepthRecord::parse(raw, "aider", "1.2.3")
}

#[test]
fn depth_record_digest_matches_the_contract_vector() {
    // Annex B.7's vector: member names sorted, no whitespace, `identity` left out.
    let part = json!({ "z": "a", "a": ["é\n", true, null], "identity": "left out" });
    assert_eq!(
        part_identity(&part).unwrap(),
        "1019655b4e277dabfe0cb08487ef861e99531f65f058a980e76a1927afe4c441"
    );
    assert!(
        part_identity(&json!({ "n": 1 })).is_err(),
        "a part holds no number"
    );
}

#[test]
fn depth_record_parses_and_states_what_an_acceptance_entry_is_made_over() {
    let record = parse(&sealed(record())).expect("parse depth record");
    assert!(record.obligations.is_some() && record.execution.is_some());

    // Listing an entry does not change what the record states; changing a part does.
    let stated = record.stated_sha256().unwrap();
    let mut listed = record.clone();
    listed.acceptance_entries.push(AcceptanceEntry::Promotion {
        targets: vec!["linux-x64".to_string()],
        record_sha256: stated.clone(),
    });
    assert_eq!(listed.stated_sha256().unwrap(), stated);
    assert_eq!(
        listed.acceptance_entries.last().unwrap().record_sha256(),
        stated
    );
    assert!(record
        .acceptance_entries
        .iter()
        .all(|e| e.record_sha256() != stated));
    let mut changed = record.clone();
    changed.execution = None;
    assert_ne!(changed.stated_sha256().unwrap(), stated);

    // A part not yet produced is stated as `null`.
    let mut early = self::record();
    early["obligations"] = Value::Null;
    early["execution"] = Value::Null;
    let early = parse(&sealed(early)).expect("parse record with P only");
    assert!(early.obligations.is_none() && early.execution.is_none());
}

#[test]
fn depth_record_rejects_what_the_annex_refuses() {
    type Edit = fn(&mut Value);
    // Each edit is applied to the full record, which is then sealed again.
    let sealed_cases: &[(&str, Edit)] = &[
        ("no other revision", |r| r["schema_version"] = json!(2)),
        ("no other revision", |r| r["schema_version"] = json!("1")),
        ("missing field `obligations`", |r| {
            r.as_object_mut().unwrap().remove("obligations");
        }),
        ("missing field `execution`", |r| {
            r.as_object_mut().unwrap().remove("execution");
        }),
        ("unknown field", |r| r["extra"] = json!(1)),
        ("unknown field", |r| {
            r["event"]["approval_commit"] = json!("x")
        }),
        ("unknown field", |r| {
            r["acceptance_entries"][0]["packet"] = json!({})
        }),
        ("unknown variant", |r| {
            r["acceptance_entries"][0]["backs"] = json!("merge")
        }),
        ("must be left out", |r| {
            r["execution"]["results"][2]["rationale"] = Value::Null
        }),
        ("must be written", |r| {
            let mode = &mut r["policy"]["selection"]["enrollment"]["modes"][1];
            mode.as_object_mut().unwrap().remove("predicates");
        }),
        ("must be left out", |r| {
            r["policy"]["debt_baseline"][0]["scope_target_triples"] = json!(["b", "a"]);
        }),
        ("must name agent `aider` and version `1.2.3`", |r| {
            r["policy"]["selection"]["agent_id"] = json!("codex");
        }),
        ("must name agent `aider` and version `1.2.3`", |r| {
            r["policy"]["selection"]["enrollment"]["version"] = json!("1.2.4");
        }),
        ("policy.selection.enrollment:", |r| {
            r["policy"]["selection"]["enrollment"]["modes"] = json!([]);
        }),
        ("event.lifecycle_path", |r| {
            r["event"] = json!({ "lifecycle_path": "onboarding", "approval_commit": "a",
                                 "approval_recorded_at": "b" });
        }),
        ("event must state every field", |r| {
            r["event"]["opened_from"] = json!(" ")
        }),
        ("policy.base", |r| r["policy"]["base"] = json!(["R1", "R1"])),
        ("policy.templates", |r| {
            r["policy"]["templates"].as_array_mut().unwrap().pop();
        }),
        ("policy.templates", |r| {
            r["policy"]["templates"][0] = json!({ "template": "QUERY", "obligations": [] });
        }),
        ("policy.templates", |r| {
            let rule = r["policy"]["templates"][1].clone();
            r["policy"]["templates"][0] = rule;
        }),
        ("policy.debt_baseline", |r| {
            let row = r["policy"]["debt_baseline"][0].clone();
            r["policy"]["debt_baseline"]
                .as_array_mut()
                .unwrap()
                .push(row);
        }),
        ("rows of agent `aider` only", |r| {
            r["policy"]["debt_baseline"][0]["agent_id"] = json!("codex");
        }),
        ("`targets` must list its entries in ascending order", |r| {
            r["obligations"]["targets"] = json!(["linux-x64", "darwin-arm64"]);
        }),
        ("`results` must list its entries in ascending order", |r| {
            r["execution"]["results"].as_array_mut().unwrap().reverse();
        }),
        ("granted or retired, never both", |r| {
            r["execution"]["debt_retired"] = json!(["row-1"]);
        }),
        ("obligations.policy_identity", |r| {
            r["obligations"]["policy_identity"] = json!("x")
        }),
        ("obligations.targets", |r| {
            r["obligations"]["targets"] = json!(["a", "a"])
        }),
        ("obligations.operations", |r| {
            let operation = r["obligations"]["operations"][0].clone();
            r["obligations"]["operations"]
                .as_array_mut()
                .unwrap()
                .push(operation);
        }),
        ("repository-relative path", |r| {
            r["obligations"]["acquired_inputs"][0]["path"] = json!("/etc/passwd");
        }),
        ("repository-relative path", |r| {
            r["obligations"]["acquired_inputs"][0]["path"] = json!("a/../b");
        }),
        ("execution.policy_identity", |r| {
            r["execution"]["policy_identity"] = json!(SHA.to_uppercase().replace('0', "A"));
        }),
        ("execution.obligations_identity", |r| {
            r["execution"]["obligations_identity"] = json!(&SHA[1..]);
        }),
        ("execution.results must not repeat", |r| {
            let result = r["execution"]["results"][2].clone();
            r["execution"]["results"]
                .as_array_mut()
                .unwrap()
                .push(result);
        }),
        ("exactly one of", |r| {
            r["execution"]["results"][2]["obligation"]["capability"] = json!("agent_api.run");
        }),
        ("exactly one of", |r| {
            r["execution"]["results"][0]["obligation"] =
                json!({ "operation": "run", "target": "linux-x64" });
        }),
        ("states a rationale", |r| {
            r["execution"]["results"][1]
                .as_object_mut()
                .unwrap()
                .remove("rationale");
        }),
        ("states a rationale", |r| {
            r["execution"]["results"][1]["rationale"] = json!(" ")
        }),
        ("states a rationale", |r| {
            r["execution"]["results"][2]["rationale"] = json!("why")
        }),
        ("only a `verified` result binds evidence", |r| {
            r["execution"]["results"][2]["result"] = json!("failed");
        }),
        ("sha256 of `crates/aider/tests/run.rs`", |r| {
            r["execution"]["results"][2]["evidence"][0]["sha256"] = json!("x");
        }),
        ("execution.reuse_bindings must not repeat", |r| {
            let binding = r["execution"]["reuse_bindings"][0].clone();
            r["execution"]["reuse_bindings"]
                .as_array_mut()
                .unwrap()
                .push(binding);
        }),
        ("exactly one of", |r| {
            r["execution"]["reuse_bindings"][0]["obligation"]["capability"] = json!("c");
        }),
        ("execution.reuse_bindings.policy_identity", |r| {
            r["execution"]["reuse_bindings"][0]["policy_identity"] = json!("x");
        }),
        ("execution.reuse_bindings.obligations_identity", |r| {
            r["execution"]["reuse_bindings"][0]["obligations_identity"] = json!("x");
        }),
        ("execution.debt_grants", |r| {
            let row = r["execution"]["debt_grants"][0].clone();
            r["execution"]["debt_grants"]
                .as_array_mut()
                .unwrap()
                .push(row);
        }),
        ("execution.debt_retired", |r| {
            r["execution"]["debt_retired"] = json!(["row-2", "row-2"]);
        }),
        ("acceptance_entries.record_sha256", |r| {
            r["acceptance_entries"][0]["record_sha256"] = json!("x");
        }),
        ("acceptance_entries.targets must name", |r| {
            r["acceptance_entries"][0]["targets"] = json!([]);
        }),
        ("acceptance_entries.targets must not repeat", |r| {
            r["acceptance_entries"][0]["targets"] = json!(["a", "a"]);
        }),
        ("acceptance_entries.packet", |r| {
            r["acceptance_entries"][1]["packet"]["path"] = json!("");
        }),
        ("acceptance_entries.closeout", |r| {
            r["acceptance_entries"][2]["closeout"]["sha256"] = json!("x");
        }),
    ];
    for (expected, edit) in sealed_cases {
        let mut record = record();
        edit(&mut record);
        let err = parse(&sealed(record)).expect_err(expected);
        assert!(err.contains(expected), "expected `{expected}`, got `{err}`");
    }

    // A part that changes while its identity does not is refused, for each part.
    let stale_cases: &[(&str, Edit)] = &[
        ("policy.identity", |r| {
            r["policy"]["path_enabled"] = json!(false)
        }),
        ("obligations.identity", |r| {
            r["obligations"]["targets"] = json!(["linux-x64"])
        }),
        ("execution.identity", |r| {
            r["execution"]["debt_retired"] = json!([])
        }),
    ];
    for (expected, edit) in stale_cases {
        let mut record: Value = serde_json::from_str(&sealed(record())).unwrap();
        edit(&mut record);
        let err = parse(&record.to_string()).expect_err(expected);
        assert!(err.contains(expected), "expected `{expected}`, got `{err}`");
    }

    assert!(parse("not json").unwrap_err().contains("parse JSON"));
    // A member stated twice is refused, although the last one would read as a valid record.
    let twice = sealed(record()).replacen('{', r#"{"schema_version":2,"#, 1);
    let err = parse(&twice).unwrap_err();
    assert!(err.contains("duplicate field `schema_version`"), "{err}");
    let err = DepthRecord::parse(&sealed(record()), "aider", "1.2.4").unwrap_err();
    assert!(err.contains("must name agent"), "{err}");
}

#[test]
fn depth_record_coverage_comparison_names_what_a_later_selection_lacks() {
    let earlier = parse(&sealed(record())).unwrap();
    let later = |edit: fn(&mut Value)| {
        let mut record = record();
        edit(&mut record["policy"]["selection"]["enrollment"]);
        parse(&sealed(record))
            .expect("later record")
            .lacking_from(&earlier)
    };

    assert!(later(|_| ()).is_empty());
    // The order a declaration writes its predicates in selects nothing.
    let two = |order: [usize; 2]| {
        let flags = [
            json!({ "flag": "--output-format", "value": "json" }),
            json!({ "flag": "-q" }),
        ];
        let mut record = record();
        record["policy"]["selection"]["enrollment"]["modes"][0]["predicates"] =
            json!([flags[order[0]], flags[order[1]]]);
        parse(&sealed(record)).unwrap()
    };
    assert!(two([1, 0]).lacking_from(&two([0, 1])).is_empty());
    // Renaming the operation that serves a mode, or covering more, lacks nothing.
    assert!(later(|e| {
        e["operations"][0]["id"] = json!("run-renamed");
        e["capabilities"][0]["operations"] = json!(["run-renamed"]);
        e["modes"][1].as_object_mut().unwrap().remove("targets");
    })
    .is_empty());

    assert_eq!(
        later(|e| e["targets"] = json!(["linux-x64"])),
        ["mode `run-json` on `darwin-arm64`"]
    );
    // A kept name that selects something else does not keep the mode.
    assert_eq!(
        later(|e| e["modes"][0]["predicates"] = json!([])),
        ["mode `run-json`"]
    );
    assert_eq!(
        later(|e| {
            e["modes"][0]["selector"] = json!("run");
            e["operations"][0]["serves"] = json!(["run"]);
        }),
        ["mode `run-json`"]
    );
    assert_eq!(
        later(|e| {
            e["capabilities"] = json!([]);
            e["operations"][0]["shared_mapping"] = json!("wrapper_only");
        }),
        ["capability `agent_api.run`"]
    );
}

fn sealed_with(edit: impl Fn(&mut Value)) -> Result<DepthRecord, String> {
    let mut r = record();
    edit(&mut r);
    parse(&sealed(r))
}

fn rec_with(edit: impl Fn(&mut Value)) -> DepthRecord {
    let mut r = record();
    edit(&mut r["policy"]["selection"]["enrollment"]);
    parse(&sealed(r)).expect("variant record must parse")
}

/// B.7 rule 3, built here from the written record and not through `stated_sha256`.
#[test]
fn acceptance_digest_is_taken_over_the_event_and_the_three_parts() {
    for early in [false, true] {
        let mut record = record();
        if early {
            record["obligations"] = Value::Null;
            record["execution"] = Value::Null;
        }
        let raw = sealed(record);
        let written: Value = serde_json::from_str(&raw).unwrap();
        let stated = json!({
            "event": written["event"], "policy": written["policy"],
            "obligations": written["obligations"], "execution": written["execution"]
        });
        assert_eq!(
            parse(&raw).unwrap().stated_sha256().unwrap(),
            part_identity(&stated).unwrap()
        );
    }
}

#[test]
fn digest_characters() {
    let real = "0123456789abcdef".repeat(4);
    let with = |s: String| {
        sealed_with(move |r| {
            r["execution"]["results"][2]["evidence"][0]["sha256"] = json!(s.clone())
        })
    };
    with(real.clone()).expect("a digest with hex letters is valid");
    for bad in [
        format!("{real}0"),
        format!("{}g", &real[1..]),
        real.to_uppercase(),
        format!("{}f", &real[1..]).replace('f', "F"),
    ] {
        let err = with(bad.clone()).expect_err(&bad);
        assert!(err.contains("64 lowercase hexadecimal"), "{bad}: {err}");
    }
}

#[test]
fn file_paths() {
    for bad in [
        "a//b", "a/", "./a", "a/./b", "a/../b", "../a", "/a", "a\\b", "",
    ] {
        let err = sealed_with(|r| r["obligations"]["acquired_inputs"][0]["path"] = json!(bad))
            .expect_err(bad);
        assert!(err.contains("repository-relative path"), "{bad:?}: {err}");
    }
    for ok in ["a", "a/b.json", ".hidden/x", "a/b..c", "docs/é.md"] {
        sealed_with(|r| r["obligations"]["acquired_inputs"][0]["path"] = json!(ok))
            .unwrap_or_else(|e| panic!("{ok}: {e}"));
    }
}

#[test]
fn event_fields_and_the_onboarding_path() {
    for field in [
        "request_commit",
        "request_recorded_at",
        "trigger_kind",
        "opened_from",
    ] {
        let err = sealed_with(|r| r["event"][field] = json!(" ")).expect_err(field);
        assert!(
            err.contains("event must state every field"),
            "{field}: {err}"
        );
    }
    let onboarding = |edit: &dyn Fn(&mut Value)| {
        let mut r = record();
        r["event"] = json!({"lifecycle_path": "onboarding", "approval_commit": "abc", "approval_recorded_at": "2026-10-10T00:00:00Z"});
        r["policy"]["selection"]["enrollment"]["lifecycle_path"] = json!("onboarding");
        edit(&mut r);
        parse(&sealed(r))
    };
    onboarding(&|_| ()).expect("an onboarding record parses");
    for field in ["approval_commit", "approval_recorded_at"] {
        let err = onboarding(&|r| r["event"][field] = json!("")).expect_err(field);
        assert!(
            err.contains("event must state every field"),
            "{field}: {err}"
        );
    }
    // an onboarding event under a maintenance selection, and the reverse
    let err = sealed_with(|r| r["event"] = json!({"lifecycle_path": "onboarding", "approval_commit": "a", "approval_recorded_at": "b"})).unwrap_err();
    assert!(err.contains("event.lifecycle_path"), "{err}");
    let err = onboarding(&|r| {
        r["event"] = json!({"lifecycle_path": "maintenance", "request_commit": "a", "request_recorded_at": "b", "trigger_kind": "c", "opened_from": "d"})
    })
    .unwrap_err();
    assert!(err.contains("event.lifecycle_path"), "{err}");
}

#[test]
fn a_repeated_template_when_every_selected_template_is_stated() {
    let err = sealed_with(|r| {
        let rule = r["policy"]["templates"][0].clone();
        r["policy"]["templates"].as_array_mut().unwrap().push(rule);
    })
    .unwrap_err();
    assert!(err.contains("policy.templates"), "{err}");
}

#[test]
fn lacking_from_compares_all_four_parts_of_a_mode_and_the_capability_id() {
    let earlier = parse(&sealed(record())).unwrap();
    assert_eq!(
        rec_with(|e| e["modes"][0]["command_path"] = json!("aider exec")).lacking_from(&earlier),
        ["mode `run-json`"]
    );
    assert_eq!(
        rec_with(|e| e["modes"][0]["mode"] = json!("batch")).lacking_from(&earlier),
        ["mode `run-json`"]
    );
    assert_eq!(
        rec_with(|e| e["capabilities"][0]["id"] = json!("agent_api.other")).lacking_from(&earlier),
        ["capability `agent_api.run`"]
    );
}

#[test]
fn proving_run_closeout_entry() {
    let entry = |sha: &str| json!({"backs": "proving_run_closeout", "record_sha256": SHA, "closeout": {"path": "docs/x.json", "sha256": sha}});
    sealed_with(|r| {
        r["acceptance_entries"]
            .as_array_mut()
            .unwrap()
            .push(entry(SHA))
    })
    .expect("valid entry");
    let err = sealed_with(|r| {
        r["acceptance_entries"]
            .as_array_mut()
            .unwrap()
            .push(entry("x"))
    })
    .unwrap_err();
    assert!(err.contains("acceptance_entries.closeout"), "{err}");
}

#[test]
fn a_predicate_that_states_only_a_flag() {
    // the registry contract's own example is `predicates = [{ flag = "--json" }]`
    sealed_with(|r| {
        r["policy"]["selection"]["enrollment"]["modes"][0]["predicates"] =
            json!([{"flag": "--json"}]);
    })
    .expect("a flag-only predicate is a valid record member");
}

#[test]
fn part_identity_refuses_non_ascii_member_names() {
    assert!(part_identity(&json!({"é": "x"}))
        .unwrap_err()
        .contains("not ASCII"));
    assert!(part_identity(&json!({"ok": [{"\u{1F600}": "x"}]})).is_err());
}
