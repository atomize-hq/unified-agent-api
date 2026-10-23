use super::*;
fn relay(workspace: &Path, baseline: &str) -> Result<(), String> {
    xtask::depth_acceptance::check_reporting_relay(workspace, baseline)
}
#[test]
fn relay_allows_reporting_execution_part_and_preserves_p_o() {
    let rec = record_with_obligations();
    let (temp, root, tip) = baseline(Some(&rec));
    let mut next = rec.clone();
    next["execution"] = json!({"identity":"", "policy_identity": rec["policy"]["identity"],
        "obligations_identity": rec["obligations"]["identity"], "results":[], "reuse_bindings":[], "debt_grants":[], "debt_retired":[]});
    seal(&mut next);
    write_record(&root, &next);
    republish(&root);
    assert!(
        relay(temp.path(), &tip).is_ok(),
        "{:?}",
        relay(temp.path(), &tip)
    );
    git(temp.path(), &["add", "."]);
    assert!(relay(temp.path(), &tip).is_ok());
    next["policy"]["selection"]["revision"] = json!("forged");
    seal(&mut next);
    write_record(&root, &next);
    assert!(relay(temp.path(), &tip).is_err());
}
#[test]
fn relay_rejects_retained_acceptance_even_when_request_is_for_a_later_version() {
    let (temp, root, _) = baseline(Some(&record()));
    let pointer = root.join(format!("pointers/latest_supported/{REQUIRED_TARGET}.txt"));
    write_text(&pointer, "none\n");
    republish(&root);
    let tip = commit_all(temp.path());
    edit_request(temp.path(), |s| s.replace(VERSION, "9.9.9"));
    fs::write(pointer, format!("{VERSION}\n")).unwrap();
    republish(&root);
    let error = relay(temp.path(), &tip).unwrap_err();
    assert!(error.contains("acceptance effects forbidden"), "{error}");
}
#[test]
fn relay_refuses_forged_matching_entry_and_debt_projection_changes() {
    let (temp, root, tip) = baseline(Some(&record()));
    let mut next = record();
    matching(&mut next);
    write_record(&root, &next);
    assert!(relay(temp.path(), &tip)
        .unwrap_err()
        .contains("preserve Event/P/O"));
    write_record(&root, &record());
    let path = temp
        .path()
        .join("docs/specs/unified-agent-api/non-tui-support-debt.md");
    fs::write(path, "forged debt inventory").unwrap();
    assert!(relay(temp.path(), &tip)
        .unwrap_err()
        .contains("corroborated transition"));
}
