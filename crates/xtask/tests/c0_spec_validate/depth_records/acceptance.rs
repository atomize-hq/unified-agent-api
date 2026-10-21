use super::*;
use sha2::{Digest, Sha256};

fn sha(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}
fn commit_all(temp: &Path) -> String {
    git(temp, &["add", "."]);
    git(temp, &["commit", "-qm", "fixture transition"]);
    git(temp, &["rev-parse", "HEAD"])
}
fn matching(record: &mut Value) {
    let parsed =
        xtask::depth_record::DepthRecord::parse(&record.to_string(), "codex", VERSION).unwrap();
    record["acceptance_entries"].as_array_mut().unwrap().push(json!({"backs":"promotion","targets":["another-target"],"record_sha256":parsed.stated_sha256().unwrap()}));
}
fn requirements(temp: &Path, tip: &str) -> Vec<xtask::depth_acceptance::MissingEntry> {
    let report = xtask::depth_acceptance::compare(temp, tip, "codex").unwrap();
    assert!(report.input_issues.is_empty(), "{:?}", report.input_issues);
    report.missing_entries
}
fn maintenance_closeout(temp: &Path) -> Value {
    json!({"request_ref": request_path(),"request_sha256":sha(&fs::read_to_string(temp.join(request_path())).unwrap()),"resolved_findings":[],"explicit_none_reason":"none","preflight_passed":true,"recorded_at":RECORDED_AT,"commit":COMMIT})
}
fn closeout_path(temp: &Path) -> PathBuf {
    temp.join(MAINTENANCE).join("maintenance-closeout.json")
}

#[test]
fn closeout_creation_needs_current_entry_and_exact_durable_reference_survives_displacement() {
    let (temp, root, tip) = baseline(Some(&record()));
    let closeout = maintenance_closeout(temp.path());
    write_json(&closeout_path(temp.path()), &closeout);
    let missing = requirements(temp.path(), &tip);
    assert_eq!(missing.len(), 1);
    assert!(missing[0]
        .effects
        .iter()
        .any(|e| e.kind == "closeout semantic change"));
    let mut rec = record();
    matching(&mut rec);
    write_record(&root, &rec);
    assert!(requirements(temp.path(), &tip).is_empty());
    // A reference locates a generation even when it describes an older record statement.
    let raw = fs::read_to_string(closeout_path(temp.path())).unwrap();
    rec["acceptance_entries"].as_array_mut().unwrap().push(json!({"backs":"maintenance_closeout","closeout":{"path":format!("{MAINTENANCE}/maintenance-closeout.json"),"sha256":sha(&raw)},"record_sha256":"a".repeat(64)}));
    write_record(&root, &rec);
    write_text(
        &temp.path().join(request_path()),
        "this is a later binding, not the bound bytes",
    );
    assert!(requirements(temp.path(), &tip).is_empty());
    let tip = commit_all(temp.path());
    let mut changed = closeout.clone();
    changed["recorded_at"] = json!("2026-10-21T00:00:00Z");
    write_json(&closeout_path(temp.path()), &changed);
    assert!(
        xtask::depth_acceptance::compare(temp.path(), &tip, "codex")
            .unwrap_err()
            .contains("scope unresolved"),
        "an old FileRef cannot locate changed bytes"
    );
}

#[test]
fn marker_retirement_requires_entry_but_content_only_change_is_reporting() {
    let (temp, root, _) = baseline(Some(&record()));
    let marker = temp
        .path()
        .join(MAINTENANCE)
        .join(format!("automation-stand-down/{VERSION}.toml"));
    write_text(&marker, "reason = \"frozen\"\n");
    let tip = commit_all(temp.path());
    write_text(&marker, "reason = \"edited\"\n");
    assert!(requirements(temp.path(), &tip).is_empty());
    fs::remove_file(&marker).unwrap();
    let missing = requirements(temp.path(), &tip);
    assert!(missing[0]
        .effects
        .iter()
        .any(|e| e.kind == "marker retirement"));
    let mut rec = record();
    matching(&mut rec);
    write_record(&root, &rec);
    assert!(requirements(temp.path(), &tip).is_empty());
}

#[test]
fn status_and_passed_targets_compare_values_not_timestamps() {
    let (temp, root, tip) = baseline(Some(&record()));
    let path = root.join(format!("versions/{VERSION}.json"));
    let original: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let mut metadata = original.clone();
    metadata["updated_at"] = json!("2026-10-21T00:00:00Z");
    write_json(&path, &metadata);
    assert!(requirements(temp.path(), &tip).is_empty());
    metadata["status"] = json!("supported");
    write_json(&path, &metadata);
    assert!(requirements(temp.path(), &tip)[0]
        .effects
        .iter()
        .any(|e| e.kind == "status"));
    write_json(&path, &original);
    let mut older = original.clone();
    older["validation"]["passed_targets"] = json!([]);
    write_json(&path, &older);
    let tip = commit_all(temp.path());
    write_json(&path, &original);
    assert!(requirements(temp.path(), &tip)[0]
        .effects
        .iter()
        .any(|e| e.kind == "validation.passed_targets"));
}
