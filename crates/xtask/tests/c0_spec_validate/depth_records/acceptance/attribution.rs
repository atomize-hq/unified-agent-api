use super::*;

#[test]
fn ordinary_legacy_approval_is_positive_but_misplaced_or_invalid_depth_never_is() {
    let path = format!("{GOVERNANCE}/approved-agent.toml");
    let text = fs::read_to_string(workspace_root().join(&path)).unwrap();
    assert!(
        xtask::approval_artifact::parse_approval_attribution(&text, &path)
            .unwrap()
            .support_depth_version
            .is_none()
    );
    for suffix in [
        "\n[support_depth]\nversion=3\n",
        "\n[descriptor.support_depth]\nversion=\"0.61.0\"\n",
        "\n[support_dept]\nversion=\"0.61.0\"\n",
    ] {
        assert!(xtask::approval_artifact::parse_approval_attribution(
            &format!("{text}{suffix}"),
            &path
        )
        .is_err());
    }
}
#[test]
fn surviving_publication_retarget_counts_raw_replacement_and_rejects_stale_linkage() {
    let (temp, root, _) = baseline(Some(&record()));
    let mut state = lifecycle::setup(temp.path(), &root, true);
    let tip = commit_all(temp.path());
    assert!(requirements(temp.path(), &tip).is_empty());
    let path = temp.path().join(GOVERNANCE).join("publication-ready.json");
    let packet: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    // Semantically identical bytes still change the reported artifact's content identity (D2).
    fs::write(&path, packet.to_string()).unwrap();
    assert!(xtask::depth_acceptance::compare(temp.path(), &tip, "codex")
        .unwrap_err()
        .contains("hash disagrees"));
    state["publication_packet_sha256"] = json!(sha(&packet.to_string()));
    write_json(&lifecycle::state_path(temp.path()), &state);
    let missing = requirements(temp.path(), &tip);
    assert!(missing[0].effects.iter().any(|e| e.kind == "published"));
    let mut rec = record();
    matching(&mut rec);
    write_record(&root, &rec);
    republish(&root);
    assert!(requirements(temp.path(), &tip).is_empty());
}
#[test]
fn unchanged_absent_closeout_and_disappearance_do_not_create_a_d2_destination() {
    let (temp, root, _) = baseline(Some(&record()));
    lifecycle::setup(temp.path(), &root, false);
    let tip = commit_all(temp.path());
    assert!(requirements(temp.path(), &tip).is_empty());
    let closeout = maintenance_closeout(temp.path());
    write_json(&closeout_path(temp.path()), &closeout);
    let tip = commit_all(temp.path());
    fs::remove_file(closeout_path(temp.path())).unwrap();
    assert!(requirements(temp.path(), &tip).is_empty());
}
#[test]
fn durable_owner_conflicts_and_candidate_binding_loss_fail_instead_of_becoming_ordinary() {
    let (temp, root, tip) = baseline(Some(&record()));
    let closeout = maintenance_closeout(temp.path());
    write_json(&closeout_path(temp.path()), &closeout);
    fs::remove_file(temp.path().join(request_path())).unwrap();
    assert!(xtask::depth_acceptance::compare(temp.path(), &tip, "codex")
        .unwrap_err()
        .contains("scope unresolved"));
    let raw = fs::read_to_string(closeout_path(temp.path())).unwrap();
    let mut rec = record();
    rec["acceptance_entries"] = json!([{"backs":"maintenance_closeout","closeout":{"path":format!("{MAINTENANCE}/maintenance-closeout.json"),"sha256":sha(&raw)},"record_sha256":"a".repeat(64)}]);
    write_record(&root, &rec);
    assert_eq!(
        requirements(temp.path(), &tip).len(),
        1,
        "historical statement cannot back current effect"
    );
    let mut second = rec.clone();
    second["policy"]["selection"]["enrollment"]["version"] = json!("0.60.0");
    seal(&mut second);
    write_json(&root.join("reports/0.60.0/depth-record.json"), &second);
    let mut metadata: Value = serde_json::from_str(
        &fs::read_to_string(root.join(format!("versions/{VERSION}.json"))).unwrap(),
    )
    .unwrap();
    metadata["semantic_version"] = json!("0.60.0");
    write_json(&root.join("versions/0.60.0.json"), &metadata);
    assert!(xtask::depth_acceptance::compare(temp.path(), &tip, "codex")
        .unwrap_err()
        .contains("conflicting durable owners"));
}
