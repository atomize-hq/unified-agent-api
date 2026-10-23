use serde_json::{json, Value};
use std::collections::BTreeMap;
use xtask::{
    depth_record::{part_identity, DepthRecord},
    support_depth::debt::{plan_debt_transition, DebtRequirement},
};

fn row(id: &str, targets: &[&str]) -> Value {
    json!({"row_id":id,"agent_id":"codex","surface_kind":"flags","command_path":"codex run","surface_id":"--legacy","current_reason":"Existing seam blocker","blocker_class":"requires_new_architectural_seam","owner":"wrappers","milestone":"bounded follow-on","follow_on":"TODOS.md#legacy","evidence_ref":"historical.md","scope_target_triples":targets,"authorized_at_version":"0.9.0","authorization_evidence_ref":"cli_manifests/codex/reports/0.9.0/coverage.any.json"})
}
fn record(rows: Vec<Value>) -> DepthRecord {
    let mut v: Value = serde_json::from_str(include_str!("fixtures/depth-ci/record.json")).unwrap();
    v["policy"]["debt_baseline"] = json!(rows);
    v["policy"]["identity"] = json!(part_identity(v["policy"].as_object().unwrap()).unwrap());
    v["obligations"]["policy_identity"] = v["policy"]["identity"].clone();
    v["obligations"]["identity"] =
        json!(part_identity(v["obligations"].as_object().unwrap()).unwrap());
    v["execution"] = json!({"identity":"","policy_identity":v["policy"]["identity"],"obligations_identity":v["obligations"]["identity"],"results":[],"reuse_bindings":[],"debt_grants":[],"debt_retired":[]});
    v["execution"]["identity"] = json!(part_identity(v["execution"].as_object().unwrap()).unwrap());
    DepthRecord::parse(&v.to_string(), "codex", "0.0.0").unwrap()
}
fn seal(r: &mut DepthRecord) {
    let e = r.execution.as_mut().unwrap();
    e.identity = part_identity(e).unwrap();
}
fn inventory(rows: &[Value]) -> String {
    let mut s = "### `support-debt-authorization-contract-target-version-v1`\n".to_string();
    for row in rows {
        s += &format!("\n### `{}`\n", row["row_id"].as_str().unwrap());
        for (k, v) in row.as_object().unwrap() {
            if k == "row_id" {
                continue;
            }
            let text = if let Some(a) = v.as_array() {
                a.iter()
                    .map(|t| t.as_str().unwrap())
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                v.as_str().unwrap().to_string()
            };
            s += &format!("- `{k}`: `{text}`\n");
        }
    }
    s
}
fn renewal(r: &mut DepthRecord, id: &str, targets: &[&str]) -> Value {
    let mut g = serde_json::to_value(
        r.policy
            .debt_baseline
            .iter()
            .find(|g| g.row_id == id)
            .unwrap(),
    )
    .unwrap();
    g["authorized_at_version"] = json!("0.0.0");
    g["scope_target_triples"] = json!(targets);
    g["authorization_evidence_ref"] = json!("cli_manifests/codex/reports/0.0.0/coverage.any.json");
    r.execution
        .as_mut()
        .unwrap()
        .debt_grants
        .push(serde_json::from_value(g.clone()).unwrap());
    seal(r);
    g
}
fn reports(targets: &[&str], gap: bool) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    for t in targets.iter().copied().chain(std::iter::once("any")) {
        let scope = if t == "any" {
            targets.to_vec()
        } else {
            vec![t]
        };
        let v = json!({"inputs":{"upstream":{"semantic_version":"0.0.0","targets":scope}},"platform_filter":{"mode":if t=="any"{"any"}else{"exact_target"},"target_triple":if t=="any"{Value::Null}else{json!(t)}},"deltas":{"missing_commands":[],"missing_flags":if gap{json!([{"path":["run"],"key":"--legacy","upstream_available_on":scope}])}else{json!([])},"missing_args":[]}});
        m.insert(
            format!("cli_manifests/codex/reports/0.0.0/coverage.{t}.json"),
            v.to_string(),
        );
    }
    m
}
#[test]
fn renewal_preserves_frozen_parts_and_requires_blocker_corroboration() {
    let mut r = record(vec![row("legacy", &["linux", "mac"])]);
    let p = r.policy.clone();
    let o = r.obligations.clone();
    let g = renewal(&mut r, "legacy", &["linux"]);
    let plan = plan_debt_transition(&r, &inventory(&[g]), &reports(&["linux"], true)).unwrap();
    assert_eq!(r.policy, p);
    assert_eq!(r.obligations, o);
    assert_eq!(plan.rows.len(), 1);
    assert!(
        matches!(&plan.requirements[0],DebtRequirement::Renewal{row_id,..} if row_id=="legacy")
    );
    assert!(plan.requires_corroboration());
}
#[test]
fn retirement_is_whole_row_and_absence_is_never_removal_proof() {
    let mut r = record(vec![row("legacy", &["linux", "mac"])]);
    r.execution
        .as_mut()
        .unwrap()
        .debt_retired
        .push("legacy".into());
    seal(&mut r);
    let plan =
        plan_debt_transition(&r, &inventory(&[]), &reports(&["linux", "mac"], false)).unwrap();
    assert!(plan.rows.is_empty());
    assert_eq!(plan.requirements.len(), 2);
    assert!(plan.requires_corroboration());
    assert!(plan
        .requirements
        .iter()
        .all(|p| matches!(p, DebtRequirement::Retirement { .. })));
    assert!(plan_debt_transition(&r, &inventory(&[]), &reports(&["linux"], false)).is_err());
    assert!(plan_debt_transition(&r, &inventory(&[]), &reports(&["linux", "mac"], true)).is_err());
}
#[test]
fn hidden_changes_outside_selection_and_new_ids_refuse() {
    let r = record(vec![
        row("legacy", &["linux"]),
        row("other-version", &["mac"]),
    ]);
    for rows in [
        vec![row("legacy", &["linux"])],
        vec![row("legacy", &["linux"]), row("new", &["mac"])],
    ] {
        assert!(plan_debt_transition(&r, &inventory(&rows), &BTreeMap::new()).is_err());
    }
    let mut rows: Vec<Value> = r
        .policy
        .debt_baseline
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    rows[1]["owner"] = json!("someone else");
    assert!(plan_debt_transition(&r, &inventory(&rows), &BTreeMap::new()).is_err());
}

#[test]
fn every_immutable_field_and_expanded_scope_refuse_even_with_same_row_id() {
    for (key, value) in [
        ("row_id", json!("new-id")),
        ("agent_id", json!("opencode")),
        ("surface_kind", json!("commands")),
        ("command_path", json!("codex other")),
        ("surface_id", json!("--new")),
        ("current_reason", json!("new reason")),
        ("blocker_class", json!("requires_new_infra")),
        ("owner", json!("new owner")),
        ("milestone", json!("later")),
        ("follow_on", json!("TODOS.md#other")),
        ("evidence_ref", json!("replacement.md")),
        ("scope_target_triples", json!(["linux", "mac"])),
    ] {
        let mut r = record(vec![row("legacy", &["linux"])]);
        let mut g = renewal(&mut r, "legacy", &["linux"]);
        g[key] = value;
        r.execution.as_mut().unwrap().debt_grants =
            vec![serde_json::from_value(g.clone()).unwrap()];
        seal(&mut r);
        assert!(
            plan_debt_transition(&r, &inventory(&[g]), &reports(&["linux", "mac"], true)).is_err(),
            "allowed {key}"
        );
    }
}
#[test]
fn disjoint_grants_are_kept_and_overlap_is_rejected_globally() {
    let mut r = record(vec![row("linux-row", &["linux"]), row("mac-row", &["mac"])]);
    let a = renewal(&mut r, "linux-row", &["linux"]);
    let b = renewal(&mut r, "mac-row", &["mac"]);
    let plan = plan_debt_transition(
        &r,
        &inventory(&[b.clone(), a.clone()]),
        &reports(&["linux", "mac"], true),
    )
    .unwrap();
    assert_eq!(plan.rows.len(), 2);
    assert_eq!(plan.requirements.len(), 2);
    let mut overlapping = b;
    overlapping["scope_target_triples"] = json!(["linux"]);
    assert!(plan_debt_transition(
        &r,
        &inventory(&[a.clone(), overlapping]),
        &reports(&["linux"], true)
    )
    .is_err());
    assert!(plan_debt_transition(&r, &inventory(&[a.clone(), a]), &BTreeMap::new()).is_err());
}
#[test]
fn stale_missing_wrong_surface_or_target_reports_refuse() {
    for case in [
        "missing",
        "stale",
        "wrong-surface",
        "wrong-target",
        "wrong-filter",
        "partial",
        "malformed",
        "bad-path",
    ] {
        let mut r = record(vec![row("legacy", &["linux"])]);
        let mut g = renewal(&mut r, "legacy", &["linux"]);
        let mut rs = reports(&["linux"], true);
        let path = "cli_manifests/codex/reports/0.0.0/coverage.any.json";
        let mut v: Value = serde_json::from_str(&rs[path]).unwrap();
        match case {
            "missing" => {
                rs.remove(path);
            }
            "stale" => {
                v["inputs"]["upstream"]["semantic_version"] = json!("0.9.0");
                rs.insert(path.into(), v.to_string());
            }
            "wrong-surface" => {
                v["deltas"]["missing_flags"][0]["key"] = json!("--other");
                rs.insert(path.into(), v.to_string());
            }
            "wrong-target" => {
                v["inputs"]["upstream"]["targets"] = json!(["mac"]);
                rs.insert(path.into(), v.to_string());
            }
            "wrong-filter" => {
                let p = "cli_manifests/codex/reports/0.0.0/coverage.linux.json";
                v = serde_json::from_str(&rs[p]).unwrap();
                v["platform_filter"]["mode"] = json!("any");
                rs.insert(p.into(), v.to_string());
            }
            "partial" => {
                rs.remove("cli_manifests/codex/reports/0.0.0/coverage.linux.json");
            }
            "malformed" => {
                rs.insert(path.into(), "{}".into());
            }
            "bad-path" => {
                g["authorization_evidence_ref"] = json!("../outside.json");
                r.execution.as_mut().unwrap().debt_grants =
                    vec![serde_json::from_value(g.clone()).unwrap()];
                seal(&mut r);
            }
            _ => unreachable!(),
        }
        assert!(
            plan_debt_transition(&r, &inventory(&[g]), &rs).is_err(),
            "allowed {case}"
        );
    }
}
#[test]
fn no_transition_is_not_an_inventory_edit_permission_and_debt_never_waives_depth() {
    let mut r = record(vec![row("legacy", &["linux"])]);
    let before = r.clone();
    let inv = inventory(&[row("legacy", &["linux"])]);
    let p = plan_debt_transition(&r, &inv, &BTreeMap::new()).unwrap();
    assert!(!p.requires_corroboration());
    assert_eq!(r, before);
    let digest = r.stated_sha256().unwrap();
    let g = renewal(&mut r, "legacy", &["linux"]);
    plan_debt_transition(&r, &inventory(&[g]), &reports(&["linux"], true)).unwrap();
    assert_eq!(r.policy, before.policy);
    assert_eq!(r.obligations, before.obligations);
    assert_eq!(r.event, before.event);
    assert_eq!(r.acceptance_entries, before.acceptance_entries);
    assert_eq!(
        r.execution.as_ref().unwrap().results,
        before.execution.as_ref().unwrap().results
    );
    assert_ne!(r.stated_sha256().unwrap(), digest);
    let facts = xtask::support_matrix::depth::derive_depth_facts(
        Some(&r),
        true,
        "x86_64-unknown-linux-gnu",
        &|_| None,
    )
    .unwrap()
    .unwrap();
    assert!(facts.promises.iter().all(|p| !p.qualified));
    assert!(facts
        .promises
        .iter()
        .flat_map(|p| &p.obligations)
        .all(|o| o.result == xtask::depth_record::DepthResult::Unverified));
}
#[test]
fn retired_unknown_row_missing_e_and_duplicate_ids_do_not_hide_inventory_changes() {
    let mut r = record(vec![row("legacy", &["linux"])]);
    assert!(plan_debt_transition(&r, &inventory(&[]), &reports(&["linux"], false)).is_err());
    r.execution = None;
    assert!(plan_debt_transition(&r, &inventory(&[]), &BTreeMap::new()).is_err());
    let mut r = record(vec![row("legacy", &["linux"])]);
    r.execution.as_mut().unwrap().debt_retired = vec!["unknown".into()];
    seal(&mut r);
    assert!(plan_debt_transition(
        &r,
        &inventory(&[row("legacy", &["linux"])]),
        &reports(&["linux"], false)
    )
    .is_err());
    let r = record(vec![row("legacy", &["linux"])]);
    assert!(plan_debt_transition(
        &r,
        &inventory(&[row("legacy", &["linux"]), row("legacy", &["mac"])]),
        &BTreeMap::new()
    )
    .is_err());
}
#[test]
fn workspace_evaluation_is_read_only_and_rejects_dirty_untracked_or_symlink_inputs() {
    use std::{fs, process::Command};
    use xtask::support_depth::debt::evaluate_debt_workspace;
    fn git(root: &std::path::Path, args: &[&str]) -> String {
        let o = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        String::from_utf8(o.stdout).unwrap()
    }
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    git(root, &["init", "-q"]);
    let mut r = record(vec![row("legacy", &["linux"])]);
    let g = renewal(&mut r, "legacy", &["linux"]);
    let record_path = "cli_manifests/codex/reports/0.0.0/depth-record.json";
    let inv_path = "docs/specs/unified-agent-api/non-tui-support-debt.md";
    let mut files = reports(&["linux"], true);
    files.insert(record_path.into(), serde_json::to_string(&r).unwrap());
    files.insert(inv_path.into(), inventory(&[g]));
    files.insert(
        "crates/xtask/data/agent_registry.toml".into(),
        include_str!("../data/agent_registry.toml").into(),
    );
    for (p, v) in &files {
        fs::create_dir_all(root.join(p).parent().unwrap()).unwrap();
        fs::write(root.join(p), v).unwrap();
    }
    git(root, &["add", "."]);
    let before = git(root, &["status", "--porcelain=v1"]);
    let index = fs::read(root.join(".git/index")).unwrap();
    assert!(evaluate_debt_workspace(root, "codex", "0.0.0")
        .unwrap()
        .requires_corroboration());
    assert_eq!(git(root, &["status", "--porcelain=v1"]), before);
    assert_eq!(fs::read(root.join(".git/index")).unwrap(), index);
    fs::write(root.join(inv_path), format!("{}\nchanged", files[inv_path])).unwrap();
    assert!(evaluate_debt_workspace(root, "codex", "0.0.0").is_err());
    fs::write(root.join(inv_path), &files[inv_path]).unwrap();
    git(root, &["rm", "--cached", inv_path]);
    assert!(evaluate_debt_workspace(root, "codex", "0.0.0").is_err());
    git(root, &["add", inv_path]);
    #[cfg(unix)]
    {
        let saved = root.join("saved");
        fs::rename(root.join(inv_path), &saved).unwrap();
        std::os::unix::fs::symlink(&saved, root.join(inv_path)).unwrap();
        assert!(evaluate_debt_workspace(root, "codex", "0.0.0").is_err());
    }
}

#[test]
fn all_adopted_roots_use_the_same_debt_transition_rules() {
    for agent in ["codex", "claude_code", "opencode"] {
        let r = record(vec![row("legacy", &["linux"])]);
        let raw = serde_json::to_string(&r).unwrap().replace("codex", agent);
        let mut r: DepthRecord = serde_json::from_str(&raw).unwrap();
        r.policy.identity = part_identity(&r.policy).unwrap();
        let o = r.obligations.as_mut().unwrap();
        o.policy_identity = r.policy.identity.clone();
        o.identity = part_identity(o).unwrap();
        let e = r.execution.as_mut().unwrap();
        e.policy_identity = r.policy.identity.clone();
        e.obligations_identity = r.obligations.as_ref().unwrap().identity.clone();
        seal(&mut r);
        let mut g = serde_json::to_value(&r.policy.debt_baseline[0]).unwrap();
        g["authorized_at_version"] = json!("0.0.0");
        g["authorization_evidence_ref"] = json!(format!(
            "cli_manifests/{agent}/reports/0.0.0/coverage.any.json"
        ));
        r.execution.as_mut().unwrap().debt_grants =
            vec![serde_json::from_value(g.clone()).unwrap()];
        seal(&mut r);
        let reports = reports(&["linux"], true)
            .into_iter()
            .map(|(p, v)| (p.replace("codex", agent), v))
            .collect();
        let plan = plan_debt_transition(&r, &inventory(&[g]), &reports).unwrap();
        assert_eq!(plan.rows[0].agent_id, agent);
        assert_eq!(plan.record_sha256, r.stated_sha256().unwrap());
        match &plan.requirements[0] {
            DebtRequirement::Renewal { target_reports, .. } => assert_eq!(target_reports.len(), 1),
            _ => panic!("expected renewal"),
        }
    }
}
