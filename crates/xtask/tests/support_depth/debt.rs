use super::*;

fn row(id: &str, agent: &str, target: &str, version: &str) -> String {
    format!("### `{id}`\n- `agent_id`: `{agent}`\n- `surface_kind`: `flags`\n- `command_path`: `{agent} unrelated`\n- `surface_id`: `--outside`\n- `current_reason`: `r`\n- `blocker_class`: `requires_new_infra`\n- `owner`: `o`\n- `milestone`: `m`\n- `follow_on`: `f`\n- `evidence_ref`: `e`\n- `scope_target_triples`: `{target}`\n- `authorized_at_version`: `{version}`\n- `authorization_evidence_ref`: `a`\n")
}
fn policy(debt: &str) -> Result<xtask::depth_record::PolicyPart, String> {
    resolve_policy(
        &registry("codex", declaration()),
        "codex",
        "1.2.3",
        &rules("codex"),
        None,
        debt,
    )
    .map(Option::unwrap)
}

#[test]
fn baseline_keeps_every_agent_row_outside_selected_operation_target_and_version() {
    let a = row("z-outside", "codex", "other-target", "9.9.9");
    let b = row("a-old", "codex", "linux", "0.1.0");
    let foreign = row("foreign", "claude_code", "darwin", "2.0.0");
    let p = policy(&format!("{DEBT}{a}{b}{foreign}")).unwrap();
    assert_eq!(p.debt_baseline.len(), 2);
    assert_eq!(p.debt_baseline[0].authorized_at_version, "0.1.0");
    assert!(p
        .debt_baseline
        .iter()
        .any(|r| r.scope_target_triples.contains("other-target")));
    assert_eq!(p, policy(&format!("{DEBT}{foreign}{b}{a}")).unwrap());
    xtask::depth_record::DepthRecord::parse(&prior(&p), "codex", "1.2.3").unwrap();
}

#[test]
fn debt_change_invalidates_policy_without_dispositioning_a_transition() {
    let row = row("old", "codex", "linux", "0.1.0");
    let p = policy(&format!("{DEBT}{row}")).unwrap();
    let changed = policy(&format!("{DEBT}{}", row.replace("0.1.0", "1.2.3"))).unwrap();
    assert!(xtask::support_depth::check_current_policy(&p, &changed).is_err());
    assert!(xtask::support_depth::check_current_policy(&p, &p).is_ok());
    let mut forged = p.clone();
    forged.path_enabled = false;
    assert!(xtask::support_depth::check_current_policy(&forged, &forged)
        .unwrap_err()
        .contains("identity"));
}

#[test]
fn missing_marker_malformed_and_duplicate_rows_refuse() {
    let row = row("duplicate", "codex", "linux", "1.2.3");
    for text in [
        row.clone(),
        format!("{DEBT}{row}{row}"),
        format!("{DEBT}### `broken`\n"),
    ] {
        assert!(policy(&text).is_err());
    }
}
