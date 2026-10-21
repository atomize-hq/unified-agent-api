use super::*;
#[test]
fn record_only_versions_and_reporting_metadata_removal_need_no_entry() {
    let (temp, root, _) = baseline(Some(&record()));
    // Keep existing runtime pointers valid while the retained record precedes its metadata.
    let mut retained = record();
    retained["policy"]["selection"]["enrollment"]["version"] = json!("9.9.9");
    seal(&mut retained);
    write_json(&root.join("reports/9.9.9/depth-record.json"), &retained);
    republish(&root);
    let tip = commit_all(temp.path());
    assert!(requirements(temp.path(), &tip).is_empty());
}
#[test]
fn expected_target_addition_exposes_stationary_embedded_pointer_and_needs_entry() {
    let (temp, root, _) = baseline(Some(&record()));
    let path = root.join("current.json");
    let original: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let mut current = original.clone();
    current["expected_targets"]
        .as_array_mut()
        .unwrap()
        .retain(|v| v != REQUIRED_TARGET);
    write_json(&path, &current);
    republish(&root);
    republish(&root);
    let tip = commit_all(temp.path());
    write_json(&path, &original);
    republish(&root);
    let missing = requirements(temp.path(), &tip);
    assert!(missing[0]
        .effects
        .iter()
        .any(|e| e.kind == "embedded runtime support"));
    assert!(!missing[0]
        .effects
        .iter()
        .any(|e| e.kind == "latest_validated_by_target"));
    let mut rec = record();
    matching(&mut rec);
    write_record(&root, &rec);
    republish(&root);
    assert!(requirements(temp.path(), &tip).is_empty());
}
#[test]
fn backsteps_into_recorded_versions_count_but_move_off_and_outside_selection_are_reporting() {
    let (temp, root, _) = baseline(Some(&record()));
    let path = root.join(format!("pointers/latest_supported/{REQUIRED_TARGET}.txt"));
    write_text(&path, "9.9.9\n");
    republish(&root);
    let tip = commit_all(temp.path());
    write_text(&path, &format!("{VERSION}\n"));
    republish(&root);
    assert_eq!(requirements(temp.path(), &tip).len(), 1);
    write_text(&path, "none\n");
    republish(&root);
    assert!(requirements(temp.path(), &tip).is_empty());
    let other = root.join("pointers/latest_supported/aarch64-apple-darwin.txt");
    write_text(&other, "none\n");
    republish(&root);
    let tip = commit_all(temp.path());
    write_text(&other, &format!("{VERSION}\n"));
    republish(&root);
    assert!(requirements(temp.path(), &tip).is_empty());
}
