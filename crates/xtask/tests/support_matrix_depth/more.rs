//! Further tests of the depth facts (child module of `support_matrix_depth.rs`).

use super::*;

fn promise_op(id: &str, serves: Value, templates: Value) -> Value {
    json!({ "id": id, "serves": serves, "templates": templates, "shared_mapping": "mapped", "forwarding": false,
            "not_applicable_permitted": [],
            "promise": { "r1": "r", "v1": "v", "g1": "g", "e1": "e", "a1": "a",
                         "x1": "x", "x2": "x", "x3": "x", "x4": "x" } })
}

fn exclusion_op(id: &str, mode: &str) -> Value {
    json!({ "id": id, "serves": [mode], "templates": ["EXCLUDED"],
            "not_applicable_permitted": [], "rationale": "interactive only" })
}

fn verified_results(op: &str, ids: &[&str], capabilities: &[&str], target: &str) -> Vec<Value> {
    let mut results: Vec<Value> = ids
        .iter()
        .map(|i| {
            result(
                json!({ "operation": op, "id": i, "target": target }),
                "verified",
            )
        })
        .collect();
    for capability in capabilities {
        results.push(result(
            json!({ "operation": op, "capability": capability, "target": target }),
            "verified",
        ));
    }
    results
}

const BASE_IDS: [&str; 11] = [
    "A1", "E1", "G1", "R1", "V1", "X1", "X2", "X3", "X4", "OUT3", "LC2",
];

/// Two promises (`zeta` templated SESSION then CORE, `alpha`), two exclusions and two capabilities,
/// each declared out of ascending order, all verified on linux.
fn many() -> Value {
    let mut r = record();
    let e = &mut r["policy"]["selection"]["enrollment"];
    e["modes"] = json!([
        { "selector": "run-json", "command_path": "aider run", "mode": "headless", "predicates": [] },
        { "selector": "tui-b", "command_path": "aider b", "mode": "interactive", "predicates": [] },
        { "selector": "tui-a", "command_path": "aider a", "mode": "interactive", "predicates": [] }
    ]);
    e["operations"] = json!([
        promise_op("zeta", json!(["run-json"]), json!(["SESSION", "CORE"])),
        promise_op("alpha", json!(["run-json"]), json!(["CORE"])),
        exclusion_op("ex-b", "tui-b"),
        exclusion_op("ex-a", "tui-a")
    ]);
    e["capabilities"] = json!([
        { "id": "b.cap", "operations": ["zeta"] },
        { "id": "a.cap", "operations": ["alpha", "zeta"] }
    ]);
    r["policy"]["templates"] = json!([
        { "template": "CORE", "obligations": ["LC2", "OUT3"] },
        { "template": "SESSION", "obligations": ["S1"] },
        { "template": "EXCLUDED", "obligations": ["EX1"] }
    ]);
    let mut zeta = BASE_IDS.to_vec();
    zeta.push("S1");
    r["obligations"]["operations"] = json!([
        { "id": "zeta", "surfaces": [surface()], "obligations": zeta },
        { "id": "alpha", "surfaces": [], "obligations": BASE_IDS },
        { "id": "ex-a", "surfaces": [], "obligations": ["EX1"] },
        { "id": "ex-b", "surfaces": [], "obligations": ["EX1"] }
    ]);
    let mut results = verified_results("zeta", &zeta, &["b.cap", "a.cap"], LINUX);
    results.extend(verified_results("alpha", &BASE_IDS, &["a.cap"], LINUX));
    results.extend(verified_results("ex-b", &["EX1"], &[], LINUX));
    results.extend(verified_results("ex-a", &["EX1"], &[], LINUX));
    r["execution"]["results"] = json!(results);
    r
}

fn name_of(o: &xtask::support_matrix::depth::PublishedObligation) -> String {
    o.id.map(|id| {
        serde_json::to_value(id)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string()
    })
    .or_else(|| o.capability.clone().map(|c| format!("M {c}")))
    .unwrap()
}

#[test]
fn lists_are_written_in_the_order_the_spec_states() {
    let linux = facts(&sealed(many(), true), LINUX);
    let names = |f: &DepthFacts| {
        f.promises
            .iter()
            .map(|p| p.operation.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(names(&linux), ["alpha", "zeta"]);
    assert!(linux.promises.iter().all(|p| p.qualified));
    let exclusions: Vec<_> = linux
        .mode_exclusions
        .iter()
        .map(|m| (&m.mode[..], &m.operation[..]))
        .collect();
    assert_eq!(exclusions, [("tui-a", "ex-a"), ("tui-b", "ex-b")]);
    let capabilities: Vec<_> = linux
        .capabilities
        .iter()
        .map(|c| (&c.id[..], c.depth_qualified))
        .collect();
    assert_eq!(capabilities, [("a.cap", true), ("b.cap", true)]);
    let zeta = linux
        .promises
        .iter()
        .find(|p| p.operation == "zeta")
        .unwrap();
    assert_eq!(zeta.templates, ["SESSION", "CORE"]);
    let order: Vec<String> = zeta.obligations.iter().map(name_of).collect();
    assert_eq!(
        order,
        [
            "R1", "V1", "G1", "E1", "A1", "X1", "X2", "X3", "X4", "OUT3", "LC2", "S1", "M a.cap",
            "M b.cap"
        ]
    );
}

#[test]
fn a_capability_is_depth_qualified_only_when_every_operation_of_its_mapping_is_qualified() {
    // alpha loses one verified result; zeta stays qualified. `a.cap` maps both, `b.cap` only zeta.
    let mut r = many();
    let results = r["execution"]["results"].as_array_mut().unwrap();
    results.retain(|x| {
        x["obligation"] != json!({ "operation": "alpha", "id": "V1", "target": LINUX })
    });
    let linux = facts(&sealed(r, true), LINUX);
    let state = |id: &str| {
        linux
            .capabilities
            .iter()
            .find(|c| c.id == id)
            .unwrap()
            .depth_qualified
    };
    assert!(!state("a.cap"), "alpha is not qualified");
    assert!(state("b.cap"), "zeta is");
    let by = |op: &str| {
        linux
            .promises
            .iter()
            .find(|p| p.operation == op)
            .unwrap()
            .qualified
    };
    assert!(!by("alpha") && by("zeta"));
}

#[test]
fn a_supported_row_is_lowered_unless_every_claimed_capability_is_depth_qualified() {
    use xtask::support_matrix::depth::CapabilityFacts;
    let with = |states: &[bool]| DepthFacts {
        obligations_frozen: true,
        promises: Vec::new(),
        mode_exclusions: Vec::new(),
        capabilities: states
            .iter()
            .enumerate()
            .map(|(i, q)| CapabilityFacts {
                id: format!("c{i}"),
                depth_qualified: *q,
            })
            .collect(),
    };
    let lowered = |f: &DepthFacts| apply_depth(UaaSupportState::Supported, Some(f));
    assert_eq!(lowered(&with(&[true, false])), UaaSupportState::Partial);
    assert_eq!(lowered(&with(&[false, true])), UaaSupportState::Partial);
    assert_eq!(lowered(&with(&[true, true])), UaaSupportState::Supported);
    assert_eq!(lowered(&with(&[])), UaaSupportState::Supported);
}

#[test]
fn an_operation_is_on_a_target_where_any_mode_it_serves_is_selected() {
    let mut r = record();
    let e = &mut r["policy"]["selection"]["enrollment"];
    e["modes"].as_array_mut().unwrap().push(json!({
        "selector": "q-json", "command_path": "aider q", "mode": "headless",
        "predicates": [], "targets": [LINUX]
    }));
    e["operations"][0]["serves"] = json!(["run-json", "q-json"]);
    let record = sealed(r, true);
    // darwin selects run-json only, and `run` serves it.
    assert_eq!(facts(&record, DARWIN).promises.len(), 1);
    assert_eq!(facts(&record, LINUX).promises.len(), 1);
}

#[test]
fn any_acceptance_entry_made_for_the_present_state_qualifies() {
    let accepted = sealed(record(), true);
    let mut v = serde_json::to_value(&accepted).unwrap();
    let present = v["acceptance_entries"][0].clone();
    let stale = json!({ "backs": "promotion", "targets": [LINUX], "record_sha256": sha('a') });
    for entries in [json!([stale, present]), json!([present, stale])] {
        v["acceptance_entries"] = entries;
        let record = DepthRecord::parse(&v.to_string(), "aider", "1.2.3").unwrap();
        assert!(facts(&record, LINUX).promises[0].qualified);
    }
    v["acceptance_entries"] = json!([stale]);
    let record = DepthRecord::parse(&v.to_string(), "aider", "1.2.3").unwrap();
    assert!(!facts(&record, LINUX).promises[0].qualified);
}

/// Recomputes E's identity and lists an entry made for the entry-less record, so that a record
/// whose E was edited after sealing still reads as accepted.
fn finished(mut v: Value) -> DepthRecord {
    v["acceptance_entries"] = json!([]);
    let mut e = v["execution"].clone();
    e.as_object_mut().unwrap().remove("identity");
    v["execution"]["identity"] = json!(part_identity(&e).unwrap());
    let parsed = DepthRecord::parse(&v.to_string(), "aider", "1.2.3").unwrap();
    v["acceptance_entries"] = json!([{ "backs": "promotion", "targets": [LINUX],
                                       "record_sha256": parsed.stated_sha256().unwrap() }]);
    DepthRecord::parse(&v.to_string(), "aider", "1.2.3").unwrap()
}

fn grant(version: &str, target: &str, surface_id: &str) -> Value {
    json!({
        "row_id": "row-1", "agent_id": "aider", "surface_kind": "flags",
        "command_path": "aider run", "surface_id": surface_id, "current_reason": "r",
        "blocker_class": "b", "owner": "o", "milestone": "m", "follow_on": "f",
        "evidence_ref": "e", "scope_target_triples": [target],
        "authorized_at_version": version, "authorization_evidence_ref": "a"
    })
}

#[test]
fn authorized_debt_needs_a_current_e_and_a_grant_for_this_version_target_and_surface() {
    let outcomes = |mut r: Value, grants: Vec<Value>| {
        r["obligations"]["operations"][0]["surfaces"] = json!([surface(), {
            "surface_kind": "flags", "command_path": "aider run", "surface_id": "--other" }]);
        for (i, g) in grants.iter().enumerate() {
            r["execution"]["debt_grants"]
                .as_array_mut()
                .unwrap()
                .push(g.clone());
            r["execution"]["debt_grants"][i]["row_id"] = json!(format!("row-{i}"));
        }
        facts(&sealed(r, true), LINUX).promises[0].outcomes.clone()
    };
    let authorized = [Outcome::AuthorizedDebt];
    assert_eq!(
        outcomes(record(), vec![grant("1.2.3", LINUX, "--output-format")]),
        authorized
    );
    // a grant of another version, for another target, or for a surface the operation does not own
    assert!(outcomes(record(), vec![grant("1.2.2", LINUX, "--output-format")]).is_empty());
    assert!(outcomes(record(), vec![grant("1.2.3", DARWIN, "--output-format")]).is_empty());
    assert!(outcomes(record(), vec![grant("1.2.3", LINUX, "--not-owned")]).is_empty());
    // the match may be the second grant, or the second surface of the operation
    assert_eq!(
        outcomes(
            record(),
            vec![
                grant("1.2.3", DARWIN, "--output-format"),
                grant("1.2.3", LINUX, "--output-format")
            ]
        ),
        authorized
    );
    assert_eq!(
        outcomes(record(), vec![grant("1.2.3", LINUX, "--other")]),
        authorized
    );

    // E references another O: the grant counts for nothing.
    let mut r = record();
    r["execution"]["debt_grants"] = json!([grant("1.2.3", LINUX, "--output-format")]);
    let mut v = serde_json::to_value(sealed(r, true)).unwrap();
    v["execution"]["obligations_identity"] = json!(sha('b'));
    let linux = facts(&finished(v), LINUX);
    assert_eq!(linux.promises[0].outcomes, [Outcome::InsufficientDepth]);
    assert_eq!(unverified_count(&linux), 12);
}

#[test]
fn every_file_a_verified_result_binds_must_be_in_the_revision_with_its_identity() {
    let mut r = record();
    let first = r["execution"]["results"][0]["obligation"].clone();
    r["execution"]["results"][0]["evidence"] = json!([{ "path": EVIDENCE, "sha256": sha('e') }, { "path": "crates/aider/tests/other.rs", "sha256": sha('e') }]);
    let record = sealed(r, true);
    let state = |files: &BTreeMap<&str, String>| {
        let linux = facts_with(&record, LINUX, files).unwrap();
        let first = first["id"].as_str().unwrap().to_string();
        results(&linux)
            .into_iter()
            .find(|(id, _)| *id == first)
            .unwrap()
            .1
    };
    let both = BTreeMap::from([
        (EVIDENCE, sha('e')),
        ("crates/aider/tests/other.rs", sha('e')),
    ]);
    assert_eq!(state(&both), "verified");
    let one = BTreeMap::from([(EVIDENCE, sha('e'))]);
    assert_eq!(state(&one), "unverified");
    let other = BTreeMap::from([("crates/aider/tests/other.rs", sha('e'))]);
    assert_eq!(state(&other), "unverified");
}

#[test]
fn classification_required_names_the_operation_of_the_unit() {
    let mut r = many();
    r["obligations"]["classification_required"] = json!([
        { "operation": "zeta", "surface": surface() },
        { "operation": "elsewhere", "surface": surface() }
    ]);
    let linux = facts(&sealed(r, true), LINUX);
    let outcomes = |op: &str| {
        linux
            .promises
            .iter()
            .find(|p| p.operation == op)
            .unwrap()
            .outcomes
            .clone()
    };
    assert_eq!(outcomes("zeta"), [Outcome::ClassificationRequired]);
    assert!(outcomes("alpha").is_empty());
}

#[test]
fn markdown_for_rows_of_several_agents_and_rows_without_depth() {
    let row = |agent: &str, target: &str, depth: Option<DepthFacts>| SupportRow {
        agent: agent.to_string(),
        version: "1.2.3".to_string(),
        target: target.to_string(),
        manifest_support: ManifestSupportState::Supported,
        backend_support: BackendSupportState::Supported,
        uaa_support: UaaSupportState::Supported,
        pointer_promotion: PointerPromotionState::None,
        evidence_notes: Vec::new(),
        depth,
    };
    let full = facts(&sealed(record(), true), LINUX);
    let none = |_: &str| None;
    let not_frozen = derive_depth_facts(None, true, DARWIN, &none)
        .unwrap()
        .unwrap();
    let rows = vec![
        row("aider", DARWIN, Some(not_frozen)),
        row("aider", LINUX, Some(full.clone())),
        row("aider", "win32-x64", None),
        row("codex", LINUX, None),
        row("opencode", LINUX, Some(full)),
    ];
    let markdown = render_publication_bundle(&rows, String::new())
        .unwrap()
        .markdown;
    let table = |agent: &str, targets: &[&str]| {
        let mut out = format!("### `{agent}`\n\n| agent | version | target | manifest_support | backend_support | uaa_support | pointer_promotion | evidence_notes |\n|---|---|---|---|---|---|---|---|\n");
        for t in targets {
            out.push_str(&format!("| `{agent}` | `1.2.3` | `{t}` | `supported` | `supported` | `supported` | `none` | — |\n"));
        }
        out
    };
    let facts_block = "\n#### Depth facts: `AGENT` `1.2.3` `linux-x64`\n\nObligations frozen: yes\n\n\
        | promise | templates | qualified | outcomes | obligations |\n|---|---|---|---|---|\n\
        | `run` | `CORE` | yes | — | `R1` verified; `V1` verified; `G1` verified; `E1` verified; \
        `A1` verified; `X1` verified; `X2` verified; `X3` verified; `X4` verified; `OUT3` verified; \
        `LC2` verified; `M agent_api.run` verified |\n\
        \n| mode exclusion | operation | `EX1` |\n|---|---|---|\n| `tui` | `tui-excluded` | verified |\n\
        \n| capability | depth_qualified |\n|---|---|\n| `agent_api.run` | yes |\n";
    let expected = format!(
        "{}\n#### Depth facts: `aider` `1.2.3` `darwin-arm64`\n\nObligations frozen: no\n{}\n{}\n{}{}",
        table("aider", &[DARWIN, LINUX, "win32-x64"]),
        facts_block.replace("AGENT", "aider"),
        table("codex", &[LINUX]),
        table("opencode", &[LINUX]),
        facts_block.replace("AGENT", "opencode"),
    );
    assert_eq!(markdown, expected);
}

#[test]
fn outcomes_are_listed_in_the_order_the_spec_states() {
    let mut r = record();
    r["execution"]["debt_grants"] = json!([grant("1.2.3", LINUX, "--output-format")]);
    // a verified result that binds nothing is published as unverified
    r["execution"]["results"][0]["evidence"] = json!([]);
    r["obligations"]["classification_required"] =
        json!([{ "operation": "run", "surface": surface() }]);
    let linux = facts(&sealed(r, true), LINUX);
    assert_eq!(
        linux.promises[0].outcomes,
        [
            Outcome::InsufficientDepth,
            Outcome::AuthorizedDebt,
            Outcome::ClassificationRequired
        ]
    );
}

#[test]
fn markdown_lists_a_promises_templates_in_declaration_order() {
    let facts = facts(&sealed(many(), true), LINUX);
    let row = SupportRow {
        agent: "aider".to_string(),
        version: "1.2.3".to_string(),
        target: LINUX.to_string(),
        manifest_support: ManifestSupportState::Supported,
        backend_support: BackendSupportState::Supported,
        uaa_support: UaaSupportState::Supported,
        pointer_promotion: PointerPromotionState::None,
        evidence_notes: Vec::new(),
        depth: Some(facts),
    };
    let markdown = render_publication_bundle(&[row], String::new())
        .unwrap()
        .markdown;
    assert!(
        markdown.contains("| `alpha` | `CORE` | yes |"),
        "{markdown}"
    );
    assert!(
        markdown.contains("| `zeta` | `SESSION`, `CORE` | yes |"),
        "{markdown}"
    );
    assert!(
        markdown.contains("| `tui-a` | `ex-a` | verified |\n| `tui-b` | `ex-b` | verified |"),
        "{markdown}"
    );
    assert!(
        markdown.contains("| `a.cap` | yes |\n| `b.cap` | yes |"),
        "{markdown}"
    );
}

/// Result rule 1: a result counts only where the obligations part is frozen for the row's target.
/// O is frozen for linux only; E states a verified `EX1` on darwin too, bound to evidence that is in
/// the revision.
#[test]
fn a_result_stated_for_a_target_o_is_not_frozen_for_is_unverified() {
    use xtask::depth_record::DepthResult;
    let mut r = record();
    r["policy"]["selection"]["enrollment"]["modes"][1]
        .as_object_mut()
        .unwrap()
        .remove("targets");
    r["obligations"]["targets"] = json!([LINUX]);
    r["execution"]["results"]
        .as_array_mut()
        .unwrap()
        .push(result(
            json!({ "operation": "tui-excluded", "id": "EX1", "target": DARWIN }),
            "verified",
        ));
    let record = sealed(r, true);
    assert_eq!(
        facts(&record, LINUX).mode_exclusions[0].result,
        DepthResult::Verified
    );
    let darwin = facts(&record, DARWIN);
    assert!(!darwin.obligations_frozen);
    assert_eq!(darwin.mode_exclusions[0].result, DepthResult::Unverified);
}

/// The artifact is revision 2 when any row carries depth facts, wherever that row stands.
#[test]
fn the_artifact_is_revision_2_when_any_row_carries_depth_facts() {
    let linux = facts(&sealed(record(), true), LINUX);
    let row = |agent: &str, depth: Option<DepthFacts>| SupportRow {
        agent: agent.to_string(),
        version: "1.2.3".to_string(),
        target: LINUX.to_string(),
        manifest_support: ManifestSupportState::Supported,
        backend_support: BackendSupportState::Supported,
        uaa_support: UaaSupportState::Supported,
        pointer_promotion: PointerPromotionState::None,
        evidence_notes: Vec::new(),
        depth,
    };
    let schema = |rows: &[SupportRow]| {
        let json = render_publication_bundle(rows, String::new()).unwrap().json;
        serde_json::from_str::<Value>(&json).unwrap()["schema_version"].clone()
    };
    assert_eq!(
        schema(&[row("aider", Some(linux.clone())), row("codex", None)]),
        json!(2)
    );
    assert_eq!(
        schema(&[row("aider", None), row("codex", Some(linux))]),
        json!(2)
    );
    assert_eq!(schema(&[row("aider", None), row("codex", None)]), json!(1));
}
