use serde_json::{json, Value};
use xtask::{agent_registry::support_depth::SupportDepth, support_depth::resolve_policy};

const REGISTRY: &str = include_str!("../data/agent_registry.toml");
const DEBT: &str = "### `support-debt-authorization-contract-target-version-v1`\n";

fn declaration() -> Value {
    json!({"enabled_paths":["maintenance"],"enrollments":[{
        "lifecycle_path":"maintenance","version":"1.2.3","targets":["linux"],
        "modes":[{"selector":"run","command_path":"codex run","mode":"headless","predicates":[]}],
        "operations":[{"id":"run","serves":["run"],"templates":["CORE"],
            "shared_mapping":"wrapper_only","forwarding":false,
            "promise":{"r1":"r","v1":"v","g1":"g","e1":"e","a1":"a",
                "x1":"x","x2":"x","x3":"x","x4":"x"}}]}]})
}

fn registry(agent: &str, mut table: Value) -> String {
    let d = &mut table["enrollments"][0];
    for mode in d["modes"].as_array_mut().unwrap() {
        mode["command_path"] = json!(mode["command_path"]
            .as_str()
            .unwrap()
            .replacen("codex", agent, 1));
    }
    let table: SupportDepth = serde_json::from_value(table).unwrap();
    let mut doc: toml_edit::DocumentMut = REGISTRY.parse().unwrap();
    let item = doc["agents"]
        .as_array_of_tables_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["agent_id"].as_str() == Some(agent))
        .unwrap();
    item.insert(
        "support_depth",
        toml_edit::Item::Table(
            toml_edit::ser::to_document(&table)
                .unwrap()
                .as_table()
                .clone(),
        ),
    );
    doc.to_string()
}

fn rules(agent: &str) -> String {
    json!({"rules_schema_version":1,"union":{"expected_targets":["linux"]},
        "storage":{"committed":{"reports":{"depth_record":format!("cli_manifests/{agent}/reports/<version>/depth-record.json")}}}}).to_string()
}

#[test]
fn declared_policy_resolves_without_enabling_a_command() {
    let raw = registry("codex", declaration());
    let p = resolve_policy(&raw, "codex", "1.2.3", &rules("codex"), None, DEBT)
        .unwrap()
        .expect("declared scope must not become remainder");
    assert!(p.path_enabled);
    assert_eq!(p.selection.enrollment.version, "1.2.3");
    assert_eq!(p.rules_revision, "1");
    assert_eq!(p.identity, xtask::depth_record::part_identity(&p).unwrap());
    assert!(xtask::agent_registry::AgentRegistry::parse(&raw).is_err());
}

fn resolve(table: Value) -> xtask::depth_record::PolicyPart {
    resolve_policy(
        &registry("codex", table),
        "codex",
        "1.2.3",
        &rules("codex"),
        None,
        DEBT,
    )
    .unwrap()
    .unwrap()
}

fn prior(policy: &xtask::depth_record::PolicyPart) -> String {
    json!({"schema_version":1,"event":{"lifecycle_path":"maintenance","request_commit":"abc",
        "request_recorded_at":"2026-10-22T00:00:00Z","trigger_kind":"upstream_release_detected","opened_from":"w.yml"},
        "policy":policy,"obligations":null,"execution":null,"acceptance_entries":[]}).to_string()
}

#[test]
fn no_declaration_is_remainder_but_unknown_agent_and_bad_version_are_errors() {
    assert!(resolve_policy(REGISTRY, "codex", "1.2.3", "", None, "")
        .unwrap()
        .is_none());
    assert!(resolve_policy(REGISTRY, "unknown", "1.2.3", "", None, "").is_err());
    for version in ["01.2.3", "1.2.3-beta", "1.2.3+build", "1.2"] {
        assert!(resolve_policy(REGISTRY, "codex", version, "", None, "").is_err());
    }
}

#[test]
fn disabled_path_is_still_enrolled_and_changes_policy_identity() {
    let enabled = resolve(declaration());
    let mut d = declaration();
    d["enabled_paths"] = json!(["onboarding"]);
    let disabled = resolve(d);
    assert!(!disabled.path_enabled);
    assert_ne!(enabled.identity, disabled.identity);
    assert!(xtask::support_depth::check_current_policy(&enabled, &disabled).is_err());
}

#[test]
fn rules_authorize_targets_current_snapshot_cannot_widen_them() {
    let mut d = declaration();
    d["enrollments"][0]["targets"] = json!(["snapshot-only"]);
    let mut r: Value = serde_json::from_str(&rules("codex")).unwrap();
    r["current"] = json!({"expected_targets":["linux", "snapshot-only"]});
    let error = resolve_policy(
        &registry("codex", d),
        "codex",
        "1.2.3",
        &r.to_string(),
        None,
        DEBT,
    )
    .unwrap_err();
    assert!(
        error.contains("no RULES or retained-record authority"),
        "{error}"
    );
}

#[test]
fn three_adopted_roots_share_the_same_resolver() {
    for agent in ["codex", "claude_code", "opencode"] {
        let policy = resolve_policy(
            &registry(agent, declaration()),
            agent,
            "1.2.3",
            &rules(agent),
            None,
            DEBT,
        )
        .unwrap()
        .unwrap();
        assert_eq!(policy.selection.agent_id, agent);
    }
}

#[test]
fn rules_schema_adoption_and_duplicate_fields_fail_closed() {
    let raw = registry("codex", declaration());
    for r in [
        rules("codex").replace("\"rules_schema_version\":1", "\"rules_schema_version\":2"),
        rules("codex").replace("cli_manifests/codex/reports", "cli_manifests/other/reports"),
        rules("codex").replace(
            "\"expected_targets\":[\"linux\"]",
            "\"expected_targets\":[\"linux\"],\"expected_targets\":[\"extra\"]",
        ),
        rules("codex").replace("\"linux\"", "\"bad target\""),
        rules("codex").replace("[\"linux\"]", "[\"linux\",\"linux\"]"),
        "{}".into(),
    ] {
        assert!(
            resolve_policy(&raw, "codex", "1.2.3", &r, None, DEBT).is_err(),
            "{r}"
        );
    }
}

#[test]
fn retained_selection_cannot_be_lost_or_moved() {
    let p = resolve(declaration());
    let prior = prior(&p);
    let error = resolve_policy(
        REGISTRY,
        "codex",
        "1.2.3",
        &rules("codex"),
        Some(&prior),
        DEBT,
    )
    .unwrap_err();
    assert!(error.contains("missing policy"));
    let mut d = declaration();
    d["enrollments"][0]["modes"][0]["selector"] = json!("renamed");
    d["enrollments"][0]["operations"][0]["serves"] = json!(["renamed"]);
    let error = resolve_policy(
        &registry("codex", d),
        "codex",
        "1.2.3",
        &rules("codex"),
        Some(&prior),
        DEBT,
    )
    .unwrap_err();
    assert!(error.contains("loses retained coverage"), "{error}");
    let mut d = declaration();
    d["enrollments"][0]["lifecycle_path"] = json!("onboarding");
    assert!(resolve_policy(
        &registry("codex", d),
        "codex",
        "1.2.3",
        &rules("codex"),
        Some(&prior),
        DEBT
    )
    .unwrap_err()
    .contains("cannot move"));
}

#[test]
fn retained_record_can_authorize_a_target_removed_from_rules() {
    let p = resolve(declaration());
    let prior = prior(&p);
    let r = rules("codex").replace("[\"linux\"]", "[\"new-target\"]");
    assert_eq!(
        resolve_policy(
            &registry("codex", declaration()),
            "codex",
            "1.2.3",
            &r,
            Some(&prior),
            DEBT
        )
        .unwrap()
        .unwrap(),
        p
    );
}

#[test]
fn malformed_foreign_and_stale_identity_records_are_errors_even_without_a_declaration() {
    let p = resolve(declaration());
    for old in [
        "{}".into(),
        prior(&p).replace("\"agent_id\":\"codex\"", "\"agent_id\":\"other\""),
        prior(&p).replace("\"rules_revision\":\"1\"", "\"rules_revision\":\"2\""),
    ] {
        assert!(resolve_policy(
            REGISTRY,
            "codex",
            "1.2.3",
            &rules("codex"),
            Some(&old),
            DEBT
        )
        .is_err());
    }
}

#[test]
fn unimplemented_predicate_and_mode_exclusion_intake_refuses() {
    let mut d = declaration();
    d["enrollments"][0]["modes"][0]["predicates"] = json!([{"flag":"--json"}]);
    assert!(resolve_policy(
        &registry("codex", d),
        "codex",
        "1.2.3",
        &rules("codex"),
        None,
        DEBT
    )
    .unwrap_err()
    .contains("acquisition intake"));
    let mut d = declaration();
    d["enrollments"][0]["operations"] = json!([{"id":"excluded","serves":["run"],"templates":["EXCLUDED"],"rationale":"review required"}]);
    assert!(resolve_policy(
        &registry("codex", d),
        "codex",
        "1.2.3",
        &rules("codex"),
        None,
        DEBT
    )
    .unwrap_err()
    .contains("mode exclusion"));
}

#[test]
fn different_mode_labels_do_not_prove_disjoint_invocations() {
    let mut d = declaration();
    let mut mode = d["enrollments"][0]["modes"][0].clone();
    mode["selector"] = json!("another");
    mode["mode"] = json!("other-label");
    d["enrollments"][0]["modes"]
        .as_array_mut()
        .unwrap()
        .push(mode);
    d["enrollments"][0]["operations"][0]["serves"] = json!(["run", "another"]);
    assert!(resolve_policy(
        &registry("codex", d),
        "codex",
        "1.2.3",
        &rules("codex"),
        None,
        DEBT
    )
    .unwrap_err()
    .contains("overlapping"));
}

#[test]
fn templates_expand_canonically_and_capability_mapping_stays_explicit() {
    use xtask::agent_registry::support_depth::{ObligationId, ObligationTemplate};
    let mut d = declaration();
    let e = &mut d["enrollments"][0];
    e["operations"][0]["templates"] = json!(["CORE", "SESSION"]);
    e["operations"][0]["shared_mapping"] = json!("mapped");
    e["capabilities"] = json!([{"id":"agent_api.run","operations":["run"]}]);
    let p = resolve(d);
    assert_eq!(p.base.len(), 9);
    assert_eq!(
        p.templates
            .iter()
            .find(|t| t.template == ObligationTemplate::Session)
            .unwrap()
            .obligations,
        vec![ObligationId::S1]
    );
    assert_eq!(
        p.selection.enrollment.capabilities[0].operations,
        vec!["run"]
    );
    let old = prior(&p);
    xtask::depth_record::DepthRecord::parse(&old, "codex", "1.2.3").unwrap();
}

#[path = "support_depth/debt.rs"]
mod debt;

#[test]
fn target_and_capability_carryover_cannot_be_narrowed() {
    let mut d = declaration();
    d["enrollments"][0]["targets"] = json!(["linux", "darwin"]);
    d["enrollments"][0]["operations"][0]["shared_mapping"] = json!("mapped");
    d["enrollments"][0]["capabilities"] = json!([{"id":"agent_api.run","operations":["run"]}]);
    let r = rules("codex").replace("[\"linux\"]", "[\"linux\",\"darwin\"]");
    let old = resolve_policy(
        &registry("codex", d.clone()),
        "codex",
        "1.2.3",
        &r,
        None,
        DEBT,
    )
    .unwrap()
    .unwrap();
    let raw = prior(&old);
    let mut narrowed = d.clone();
    narrowed["enrollments"][0]["targets"] = json!(["linux"]);
    assert!(resolve_policy(
        &registry("codex", narrowed),
        "codex",
        "1.2.3",
        &r,
        Some(&raw),
        DEBT
    )
    .unwrap_err()
    .contains("darwin"));
    d["enrollments"][0]["operations"][0]["shared_mapping"] = json!("wrapper_only");
    d["enrollments"][0]["capabilities"] = json!([]);
    assert!(resolve_policy(
        &registry("codex", d),
        "codex",
        "1.2.3",
        &r,
        Some(&raw),
        DEBT
    )
    .unwrap_err()
    .contains("capability"));
}

#[test]
fn source_formatting_is_irrelevant_but_declaration_list_order_is_frozen_policy() {
    let mut d = declaration();
    d["enrollments"][0]["operations"][0]["templates"] = json!(["CORE", "SESSION"]);
    let before = resolve(d.clone());
    let raw = format!("\n{}\n", registry("codex", d.clone()));
    assert_eq!(
        before,
        resolve_policy(&raw, "codex", "1.2.3", &rules("codex"), None, DEBT)
            .unwrap()
            .unwrap()
    );
    d["enrollments"][0]["operations"][0]["templates"] = json!(["SESSION", "CORE"]);
    let after = resolve(d);
    assert_ne!(before.identity, after.identity);
    assert!(xtask::support_depth::check_current_policy(&before, &after).is_err());
}

#[test]
fn old_rules_revision_is_not_current_even_with_its_own_valid_digest() {
    let mut p = resolve(declaration());
    p.rules_revision = "older".into();
    p.identity = xtask::depth_record::part_identity(&p).unwrap();
    assert!(xtask::support_depth::check_current_policy(&p, &p)
        .unwrap_err()
        .contains("unsupported"));
}

#[test]
fn actual_adopted_rules_contracts_resolve_and_roundtrip_policies() {
    let cases = [
        (
            "codex",
            include_str!("../../../cli_manifests/codex/RULES.json"),
        ),
        (
            "claude_code",
            include_str!("../../../cli_manifests/claude_code/RULES.json"),
        ),
        (
            "opencode",
            include_str!("../../../cli_manifests/opencode/RULES.json"),
        ),
    ];
    for (agent, rules_text) in cases {
        let r: Value = serde_json::from_str(rules_text).unwrap();
        let mut d = declaration();
        d["enrollments"][0]["targets"] = json!([r["union"]["expected_targets"][0]]);
        let p = resolve_policy(&registry(agent, d), agent, "1.2.3", rules_text, None, DEBT)
            .unwrap()
            .unwrap();
        assert_eq!(
            p.selection.enrollment.targets[0],
            r["union"]["expected_targets"][0].as_str().unwrap()
        );
        xtask::depth_record::DepthRecord::parse(&prior(&p), agent, "1.2.3").unwrap();
    }
}
