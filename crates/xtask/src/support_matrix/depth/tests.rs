use std::collections::BTreeMap;

use crate::depth_record::{part_identity, DepthRecord};
use crate::support_matrix::depth::{apply_depth, DepthFacts, Outcome};
use crate::support_matrix::{
    render_publication_bundle, BackendSupportState, ManifestSupportState, PointerPromotionState,
    SupportRow, UaaSupportState,
};
use serde_json::{json, Value};

const LINUX: &str = "linux-x64";
const DARWIN: &str = "darwin-arm64";
const EVIDENCE: &str = "crates/aider/tests/run.rs";

fn sha(byte: char) -> String {
    byte.to_string().repeat(64)
}

fn surface() -> Value {
    json!({ "surface_kind": "flags", "command_path": "aider run", "surface_id": "--output-format" })
}

fn result(obligation: Value, result: &str) -> Value {
    let evidence = if result == "verified" {
        json!([{ "path": EVIDENCE, "sha256": sha('e') }])
    } else {
        json!([])
    };
    let mut value = json!({ "obligation": obligation, "result": result, "evidence": evidence });
    if result == "not_applicable" {
        value["rationale"] = json!("no credential is read");
    }
    value
}

fn id(id: &str, target: &str) -> Value {
    json!({ "operation": "run", "id": id, "target": target })
}

/// Every obligation of `run` and the exclusion's `EX1` verified on linux, nothing on darwin.
fn record() -> Value {
    let mut results: Vec<Value> = [
        "A1", "E1", "G1", "R1", "V1", "X1", "X2", "X3", "X4", "OUT3", "LC2",
    ]
    .iter()
    .map(|o| result(id(o, LINUX), "verified"))
    .collect();
    results.push(result(
        json!({ "operation": "run", "capability": "agent_api.run", "target": LINUX }),
        "verified",
    ));
    results.push(result(
        json!({ "operation": "tui-excluded", "id": "EX1", "target": LINUX }),
        "verified",
    ));
    json!({
        "schema_version": 1,
        "event": { "lifecycle_path": "maintenance", "request_commit": "abc", "opened_from": "w",
                   "request_recorded_at": "t", "trigger_kind": "upstream_release_detected" },
        "policy": {
            "identity": "",
            "selection": { "agent_id": "aider", "enrollment": {
                "lifecycle_path": "maintenance", "version": "1.2.3", "targets": [LINUX, DARWIN],
                "modes": [
                    { "selector": "run-json", "command_path": "aider run", "mode": "headless",
                      "predicates": [{ "flag": "--output-format", "value": "json" }] },
                    { "selector": "tui", "command_path": "aider", "mode": "interactive",
                      "predicates": [], "targets": [LINUX] }
                ],
                "operations": [
                    { "id": "run", "serves": ["run-json"], "templates": ["CORE"],
                      "shared_mapping": "mapped", "forwarding": false, "not_applicable_permitted": ["X1"],
                      "promise": { "r1": "r", "v1": "v", "g1": "g", "e1": "e", "a1": "a",
                                   "x1": "x", "x2": "x", "x3": "x", "x4": "x" } },
                    { "id": "tui-excluded", "serves": ["tui"], "templates": ["EXCLUDED"],
                      "not_applicable_permitted": [], "rationale": "interactive only" }
                ],
                "capabilities": [{ "id": "agent_api.run", "operations": ["run"] }]
            } },
            "rules_revision": "1",
            "base": ["A1", "E1", "G1", "R1", "V1", "X1", "X2", "X3", "X4"],
            "templates": [
                { "template": "CORE", "obligations": ["LC2", "OUT3"] },
                { "template": "EXCLUDED", "obligations": ["EX1"] }
            ],
            "path_enabled": true,
            "debt_baseline": []
        },
        "obligations": {
            "identity": "", "policy_identity": "",
            "targets": [LINUX, DARWIN],
            "acquired_inputs": [],
            "operations": [{ "id": "run", "surfaces": [surface()],
                             "obligations": ["A1", "E1", "G1", "R1", "V1", "X1", "X2", "X3", "X4",
                                             "OUT3", "LC2"] },
                           { "id": "tui-excluded", "surfaces": [], "obligations": ["EX1"] }],
            "classification_required": [],
            "acceptance_work": []
        },
        "execution": {
            "identity": "", "policy_identity": "", "obligations_identity": "",
            "results": results,
            "reuse_bindings": [], "debt_grants": [], "debt_retired": []
        },
        "acceptance_entries": []
    })
}

/// Every list the record derives is a set in ascending canonical order.
fn sorted(value: &mut Value, at: &str) {
    match value {
        Value::Array(items) => {
            items.iter_mut().for_each(|item| sorted(item, at));
            if at != "acceptance_entries" {
                items.sort_by_key(|item| item.to_string());
            }
        }
        Value::Object(members) => {
            for (name, member) in members.iter_mut().filter(|(name, _)| *name != "enrollment") {
                sorted(member, name);
            }
        }
        _ => {}
    }
}

/// Seals the identities, links E to P and O, and, when `accept`, lists an entry made for what
/// the sealed record states.
fn sealed(record: Value, accept: bool) -> DepthRecord {
    let record = sealed_json(record, accept);
    DepthRecord::parse(&record.to_string(), "aider", "1.2.3").expect("parse record")
}

/// `sealed`, as the JSON a manifest root holds.
fn sealed_json(record: Value, accept: bool) -> Value {
    sealed_json_for(record, accept, "1.2.3")
}

/// `sealed_json` for the record of another version of the aider root.
fn sealed_json_for(mut record: Value, accept: bool, version: &str) -> Value {
    sorted(&mut record, "");
    record["policy"]["identity"] = json!(part_identity(&record["policy"]).unwrap());
    let policy = record["policy"]["identity"].clone();
    if record["obligations"].is_object() {
        record["obligations"]["policy_identity"] = policy.clone();
        record["obligations"]["identity"] = json!(part_identity(&record["obligations"]).unwrap());
    }
    if record["execution"].is_object() {
        if record["execution"]["policy_identity"] == json!("") {
            record["execution"]["policy_identity"] = policy;
        }
        record["execution"]["obligations_identity"] = record["obligations"]["identity"].clone();
        record["execution"]["identity"] = json!(part_identity(&record["execution"]).unwrap());
    }
    let parsed = DepthRecord::parse(&record.to_string(), "aider", version).expect("parse record");
    if accept {
        record["acceptance_entries"] = json!([{ "backs": "promotion", "targets": [LINUX],
                                                "record_sha256": parsed.stated_sha256().unwrap() }]);
    }
    record
}

fn facts_with(
    record: &DepthRecord,
    target: &str,
    files: &BTreeMap<&str, String>,
) -> Option<DepthFacts> {
    let lookup = |path: &str| files.get(path).cloned();
    derive_depth_facts(Some(record), false, target, &lookup).expect("derive facts")
}

fn facts(record: &DepthRecord, target: &str) -> DepthFacts {
    facts_with(record, target, &BTreeMap::from([(EVIDENCE, sha('e'))])).expect("depth-enrolled")
}

fn results(facts: &DepthFacts) -> Vec<(String, String)> {
    facts.promises[0]
        .obligations
        .iter()
        .map(|o| {
            let id =
                o.id.map(|id| {
                    serde_json::to_value(id)
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string()
                })
                .or_else(|| o.capability.clone().map(|c| format!("M {c}")))
                .unwrap();
            let result = serde_json::to_value(o.result).unwrap();
            (id, result.as_str().unwrap().to_string())
        })
        .collect()
}

fn unverified_count(facts: &DepthFacts) -> usize {
    results(facts)
        .iter()
        .filter(|(_, r)| r == "unverified")
        .count()
}

#[test]
fn a_verified_and_accepted_record_qualifies_its_promise_and_capability() {
    let record = sealed(record(), true);
    let linux = facts(&record, LINUX);
    assert!(linux.obligations_frozen);
    let run = &linux.promises[0];
    assert_eq!(run.operation, "run");
    assert_eq!(run.templates, ["CORE"]);
    assert!(run.qualified, "{run:?}");
    assert!(run.outcomes.is_empty());
    assert_eq!(results(&linux).len(), 12);
    assert_eq!(unverified_count(&linux), 0);
    assert_eq!(results(&linux).last().unwrap().0, "M agent_api.run");
    assert_eq!(linux.mode_exclusions.len(), 1);
    assert_eq!(linux.mode_exclusions[0].mode, "tui");
    assert_eq!(linux.mode_exclusions[0].operation, "tui-excluded");
    assert_eq!(linux.capabilities.len(), 1);
    assert!(linux.capabilities[0].depth_qualified);
    assert_eq!(
        apply_depth(UaaSupportState::Supported, Some(&linux)),
        UaaSupportState::Supported
    );

    // Darwin has no results and no exclusion: insufficient, and the capability lowers the row.
    let darwin = facts(&record, DARWIN);
    assert_eq!(darwin.promises[0].outcomes, [Outcome::InsufficientDepth]);
    assert!(!darwin.promises[0].qualified);
    assert_eq!(unverified_count(&darwin), 12);
    assert!(darwin.mode_exclusions.is_empty());
    assert!(!darwin.capabilities[0].depth_qualified);
    assert_eq!(
        apply_depth(UaaSupportState::Supported, Some(&darwin)),
        UaaSupportState::Partial
    );
    for state in [UaaSupportState::Partial, UaaSupportState::Unsupported] {
        assert_eq!(apply_depth(state, Some(&darwin)), state);
    }
    assert_eq!(
        apply_depth(UaaSupportState::Supported, None),
        UaaSupportState::Supported
    );
}

#[test]
fn a_result_is_published_as_unverified_where_invariant_3_says() {
    let record = sealed(record(), true);
    // Bound evidence absent, or with another content identity.
    for files in [BTreeMap::new(), BTreeMap::from([(EVIDENCE, sha('f'))])] {
        let linux = facts_with(&record, LINUX, &files).unwrap();
        assert_eq!(unverified_count(&linux), 12);
        assert!(!linux.promises[0].qualified);
        assert!(!linux.capabilities[0].depth_qualified);
        assert_eq!(
            linux.mode_exclusions[0].result,
            crate::depth_record::DepthResult::Unverified
        );
    }

    // E references another P.
    let mut stale = self::record();
    stale["execution"]["policy_identity"] = json!(sha('a'));
    let linux = facts(&sealed(stale, true), LINUX);
    assert_eq!(unverified_count(&linux), 12);

    // A verified result that binds no evidence.
    let mut unbound = self::record();
    unbound["execution"]["results"][0]["evidence"] = json!([]);
    let linux = facts(&sealed(unbound, true), LINUX);
    assert_eq!(unverified_count(&linux), 1);
    assert!(!linux.promises[0].qualified);

    // `failed` stays `failed`.
    let mut failed = self::record();
    failed["execution"]["results"][0] = result(id("A1", LINUX), "failed");
    let linux = facts(&sealed(failed, true), LINUX);
    assert!(results(&linux).contains(&("A1".to_string(), "failed".to_string())));
    assert_eq!(linux.promises[0].outcomes, [Outcome::InsufficientDepth]);
}

#[test]
fn not_applicable_stands_only_where_the_promise_permits_it() {
    let edit = |obligation: Value| {
        let mut record = record();
        let results = record["execution"]["results"].as_array_mut().unwrap();
        let at = results
            .iter()
            .position(|r| r["obligation"] == obligation)
            .unwrap();
        results[at] = result(obligation, "not_applicable");
        facts(&sealed(record, true), LINUX)
    };

    let permitted = edit(id("X1", LINUX));
    assert!(results(&permitted).contains(&("X1".to_string(), "not_applicable".to_string())));
    assert!(permitted.promises[0].qualified);

    let refused = edit(id("V1", LINUX));
    assert!(results(&refused).contains(&("V1".to_string(), "unverified".to_string())));
    assert!(!refused.promises[0].qualified);

    let m = edit(json!({ "operation": "run", "capability": "agent_api.run", "target": LINUX }));
    assert!(results(&m).contains(&("M agent_api.run".to_string(), "unverified".to_string())));
    assert!(!m.capabilities[0].depth_qualified);
}

#[test]
fn qualification_needs_an_acceptance_entry_for_what_the_record_states_now() {
    let linux = facts(&sealed(record(), false), LINUX);
    assert_eq!(unverified_count(&linux), 0);
    assert!(linux.promises[0].outcomes.is_empty());
    assert!(!linux.promises[0].qualified);
    assert!(!linux.capabilities[0].depth_qualified);

    // An entry made before the record changed identifies nothing it states now.
    let accepted = sealed(record(), true);
    let mut changed = serde_json::to_value(&accepted).unwrap();
    changed["event"]["request_commit"] = json!("def");
    let changed = DepthRecord::parse(&changed.to_string(), "aider", "1.2.3").unwrap();
    assert!(!facts(&changed, LINUX).promises[0].qualified);
}

/// Makes the `run` promise an `OPAQUE` one, for which forwarding suffices.
fn opaque(record: &mut Value, forwarding: bool) {
    let run = &mut record["policy"]["selection"]["enrollment"]["operations"][0];
    run["templates"] = json!(["OPAQUE"]);
    run["forwarding"] = json!(forwarding);
    record["policy"]["templates"][0] =
        json!({ "template": "OPAQUE", "obligations": ["LC1", "OUT1"] });
    let ids = [
        "A1", "E1", "G1", "R1", "V1", "X1", "X2", "X3", "X4", "OUT1", "LC1",
    ];
    record["obligations"]["operations"][0]["obligations"] = json!(ids);
    let results = record["execution"]["results"].as_array_mut().unwrap();
    for (from, to) in [("OUT3", "OUT1"), ("LC2", "LC1")] {
        let at = results
            .iter()
            .position(|r| r["obligation"] == id(from, LINUX))
            .unwrap();
        results[at] = result(id(to, LINUX), "verified");
    }
}

#[test]
fn a_qualified_forwarding_promise_has_every_obligation_verified() {
    let outcomes = |forwarding: bool, edit: &dyn Fn(&mut Value), accept: bool| {
        let mut record = record();
        opaque(&mut record, forwarding);
        edit(&mut record);
        let promise = facts(&sealed(record, accept), LINUX).promises[0].clone();
        (promise.qualified, promise.outcomes)
    };
    assert_eq!(
        outcomes(true, &|_| (), true),
        (true, vec![Outcome::QualifiedForwarding])
    );
    assert_eq!(outcomes(false, &|_| (), true), (true, vec![]));
    assert_eq!(outcomes(true, &|_| (), false), (false, vec![]));
    // A permitted `not_applicable` qualifies the promise, but not as qualified forwarding.
    let not_applicable = |r: &mut Value| {
        let results = r["execution"]["results"].as_array_mut().unwrap();
        let at = results
            .iter()
            .position(|r| r["obligation"] == id("X1", LINUX))
            .unwrap();
        results[at] = result(id("X1", LINUX), "not_applicable");
    };
    assert_eq!(outcomes(true, &not_applicable, true), (true, vec![]));
}

#[test]
fn classification_required_and_authorized_debt_are_outcomes_of_the_promise() {
    let mut record = record();
    record["obligations"]["classification_required"] =
        json!([{ "operation": "run", "surface": surface() }]);
    record["execution"]["debt_grants"] = json!([{
        "row_id": "row-1", "agent_id": "aider", "surface_kind": "flags",
        "command_path": "aider run", "surface_id": "--output-format", "current_reason": "r",
        "blocker_class": "b", "owner": "o", "milestone": "m", "follow_on": "f",
        "evidence_ref": "e", "scope_target_triples": [LINUX],
        "authorized_at_version": "1.2.3", "authorization_evidence_ref": "a"
    }]);
    let record = sealed(record, true);
    let linux = facts(&record, LINUX);
    assert_eq!(
        linux.promises[0].outcomes,
        [Outcome::AuthorizedDebt, Outcome::ClassificationRequired]
    );
    assert!(!linux.promises[0].qualified);
    // The grant is for linux only.
    assert_eq!(
        facts(&record, DARWIN).promises[0].outcomes,
        [Outcome::InsufficientDepth, Outcome::ClassificationRequired]
    );
}

#[test]
fn a_row_whose_obligations_are_not_frozen_states_so() {
    let mut early = record();
    early["obligations"] = Value::Null;
    early["execution"] = Value::Null;
    let linux = facts(&sealed(early, false), LINUX);
    assert!(!linux.obligations_frozen);
    assert!(linux.promises[0].obligations.is_empty());
    assert_eq!(linux.promises[0].outcomes, [Outcome::InsufficientDepth]);
    assert!(!linux.capabilities[0].depth_qualified);

    // O frozen for linux only.
    let mut partial = record();
    partial["obligations"]["targets"] = json!([LINUX]);
    let record = sealed(partial, true);
    assert!(facts(&record, LINUX).obligations_frozen);
    assert!(!facts(&record, DARWIN).obligations_frozen);

    // No record, or one that covers no mode on the target: depth-enrolled only if declared.
    let none = |_: &str| None;
    assert_eq!(derive_depth_facts(None, false, LINUX, &none).unwrap(), None);
    let declared = derive_depth_facts(None, true, LINUX, &none)
        .unwrap()
        .unwrap();
    assert!(!declared.obligations_frozen && declared.promises.is_empty());
    assert!(declared.capabilities.is_empty());
    assert_eq!(
        derive_depth_facts(Some(&record), false, "win32-x64", &none).unwrap(),
        None
    );
    // A declared target the record does not cover still lists the record's capabilities, so
    // rule 6 lowers its row.
    let win = derive_depth_facts(Some(&record), true, "win32-x64", &none)
        .unwrap()
        .unwrap();
    assert!(win.promises.is_empty() && win.mode_exclusions.is_empty());
    // O frozen for a target the selection does not cover freezes nothing there.
    let mut wider = self::record();
    wider["obligations"]["targets"] = json!([LINUX, DARWIN, "win32-x64"]);
    let wider = derive_depth_facts(Some(&sealed(wider, true)), true, "win32-x64", &none)
        .unwrap()
        .unwrap();
    assert!(!wider.obligations_frozen);
    assert_eq!(win.capabilities.len(), 1);
    assert!(!win.capabilities[0].depth_qualified);
    assert_eq!(
        apply_depth(UaaSupportState::Supported, Some(&win)),
        UaaSupportState::Partial
    );
}

#[test]
fn an_obligation_set_other_than_the_one_policy_resolves_is_refused() {
    for (operation, set) in [
        (0, json!(["OUT3"])),
        (0, json!([])),
        (
            0,
            json!(["A1", "E1", "G1", "R1", "V1", "X1", "X2", "X3", "X4", "OUT3", "LC2", "S1"]),
        ),
        (1, json!([])),
        (1, json!(["EX1", "R1"])),
    ] {
        let mut record = record();
        record["obligations"]["operations"][operation]["obligations"] = set.clone();
        let record = sealed(record, true);
        let err = derive_depth_facts(Some(&record), false, LINUX, &|_| None).unwrap_err();
        assert!(
            err.contains("not the one its policy resolves"),
            "{set}: {err}"
        );
    }
    // The exclusion is checked only where its mode is selected.
    let mut record = record();
    record["obligations"]["operations"][1]["obligations"] = json!([]);
    let record = sealed(record, true);
    derive_depth_facts(Some(&record), false, DARWIN, &|_| None).expect("tui is linux-only");
}

#[test]
fn debt_in_force_is_the_baseline_as_e_grants_or_retires_it() {
    let row = |row_id: &str, targets: Value| {
        json!({
            "row_id": row_id, "agent_id": "aider", "surface_kind": "flags",
            "command_path": "aider run", "surface_id": "--output-format", "current_reason": "r",
            "blocker_class": "b", "owner": "o", "milestone": "m", "follow_on": "f",
            "evidence_ref": "e", "scope_target_triples": targets,
            "authorized_at_version": "1.2.3", "authorization_evidence_ref": "a"
        })
    };
    let debt = |edit: &dyn Fn(&mut Value)| {
        let mut record = record();
        record["policy"]["debt_baseline"] = json!([row("row-1", json!([LINUX]))]);
        edit(&mut record);
        facts(&sealed(record, true), LINUX).promises[0]
            .outcomes
            .contains(&Outcome::AuthorizedDebt)
    };
    assert!(debt(&|_| ()), "a baseline row in force");
    // A baseline row authorized for another version authorizes nothing for this one.
    assert!(!debt(
        &|r| r["policy"]["debt_baseline"][0]["authorized_at_version"] = json!("1.2.2")
    ));
    assert!(!debt(&|r| r["execution"]["debt_retired"] = json!(["row-1"])));
    assert!(!debt(
        &|r| r["execution"]["debt_grants"] = json!([row("row-1", json!([DARWIN]))])
    ));
    // A stale E neither retires nor replaces a baseline row.
    assert!(debt(&|r| {
        r["execution"]["debt_retired"] = json!(["row-1"]);
        r["execution"]["policy_identity"] = json!(sha('a'));
    }));
}

#[test]
fn a_frozen_obligations_part_without_the_operation_is_refused() {
    let mut record = record();
    record["obligations"]["operations"] = json!([]);
    let record = sealed(record, true);
    let err = derive_depth_facts(Some(&record), false, LINUX, &|_| None).unwrap_err();
    assert!(
        err.contains("no obligation set for operation `run`"),
        "{err}"
    );
}

#[test]
fn revision_2_rows_carry_depth_facts_and_other_rows_are_unchanged() {
    let row = |depth: Option<DepthFacts>| SupportRow {
        agent: "aider".to_string(),
        version: "1.2.3".to_string(),
        target: LINUX.to_string(),
        manifest_support: ManifestSupportState::Supported,
        backend_support: BackendSupportState::Supported,
        uaa_support: UaaSupportState::Supported,
        pointer_promotion: PointerPromotionState::None,
        evidence_notes: Vec::new(),
        depth,
    };
    let plain = serde_json::to_value(row(None)).unwrap();
    assert!(
        plain.get("depth").is_none(),
        "a row without depth facts carries no marker"
    );

    let linux = facts(&sealed(record(), true), LINUX);
    let enrolled = row(Some(linux.clone()));
    let value = serde_json::to_value(&enrolled).unwrap();
    assert_eq!(value["depth"]["promises"][0]["qualified"], json!(true));
    assert_eq!(
        serde_json::from_value::<SupportRow>(value).unwrap(),
        enrolled
    );

    let bundle = render_publication_bundle(&[enrolled], String::new()).unwrap();
    assert!(
        bundle.json.starts_with("{\n  \"schema_version\": 2,"),
        "{}",
        bundle.json
    );
    let plain_bundle = render_publication_bundle(&[row(None)], String::new()).unwrap();
    assert!(plain_bundle.json.starts_with("{\n  \"schema_version\": 1,"));
    let expected = "\n#### Depth facts: `aider` `1.2.3` `linux-x64`\n\nObligations frozen: yes\n\n\
        | promise | templates | qualified | outcomes | obligations |\n|---|---|---|---|---|\n\
        | `run` | `CORE` | yes | — | `R1` verified; `V1` verified; `G1` verified; `E1` verified; \
        `A1` verified; `X1` verified; `X2` verified; `X3` verified; `X4` verified; `OUT3` verified; \
        `LC2` verified; `M agent_api.run` verified |\n\
        \n| mode exclusion | operation | `EX1` |\n|---|---|---|\n| `tui` | `tui-excluded` | verified |\n\
        \n| capability | depth_qualified |\n|---|---|\n| `agent_api.run` | yes |\n";
    assert!(bundle.markdown.ends_with(expected), "{}", bundle.markdown);
}

#[path = "tests/generator.rs"]
mod generator;
#[path = "tests/more.rs"]
mod more;

#[test]
fn hash_only_success_and_acceptance_cannot_verify_a_published_obligation() {
    let r = sealed(record(), true);
    let published =
        crate::support_matrix::depth::derive_depth_facts(Some(&r), false, LINUX, &|_| {
            Some(sha('e'))
        })
        .unwrap()
        .unwrap();
    assert!(!published.promises[0].qualified);
    assert!(published.promises[0]
        .obligations
        .iter()
        .all(|o| o.result == crate::depth_record::DepthResult::Unverified));
}

// These are pure publication-model fixtures, not authenticated host facts. The public path
// rejection test above deliberately calls the real public hash-only API directly.
fn derive_depth_facts(
    record: Option<&DepthRecord>,
    declared: bool,
    target: &str,
    hashes: &dyn Fn(&str) -> Option<String>,
) -> Result<Option<DepthFacts>, String> {
    let results = crate::depth_evidence::test_results(record);
    super::derive_evaluated_depth_facts(record, declared, target, hashes, &results)
}
