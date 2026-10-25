use super::*;

fn git(workspace: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
        ])
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
        ])
        .args(args)
        .current_dir(workspace)
        .output()
        .unwrap();
    assert!(output.status.success(), "git {args:?}: {output:?}");
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn seal(record: &mut Value) {
    for part in ["policy", "obligations", "execution"] {
        if !record[part].is_null() {
            record[part]["identity"] = json!(part_identity(&record[part]).unwrap());
        }
    }
}

fn freeze(workspace: &Path, record: &Value) {
    write_request(
        workspace,
        record["event"]["request_commit"].as_str().unwrap(),
        record["policy"]["identity"].as_str(),
    );
    edit_request(workspace, |text| {
        let mut text = text.replace(
            RECORDED_AT,
            record["event"]["request_recorded_at"].as_str().unwrap(),
        );
        if let Some(identity) = record["obligations"]["identity"].as_str() {
            text.push_str(&format!("obligations_identity = \"{identity}\"\n"));
        }
        text
    });
}

fn baseline(record: Option<&Value>) -> (tempfile::TempDir, PathBuf, String) {
    let temp = tempfile::tempdir().unwrap();
    let root = materialize_minimal_valid_workspace(temp.path());
    if let Some(record) = record {
        write_record(&root, record);
        freeze(temp.path(), record);
    }
    republish(&root);
    git(temp.path(), &["init", "-q"]);
    git(temp.path(), &["add", "."]);
    git(temp.path(), &["commit", "-qm", "baseline"]);
    let tip = git(temp.path(), &["rev-parse", "HEAD"]);
    (temp, root, tip)
}

fn compare(root: &Path, tip: &str) -> (i32, String) {
    compare_args(root, tip, "manifest-validate", &[])
}

fn compare_args(root: &Path, tip: &str, command: &str, extra: &[&str]) -> (i32, String) {
    republish(root);
    compare_as_published(root, tip, command, extra)
}

fn compare_as_published(root: &Path, tip: &str, command: &str, extra: &[&str]) -> (i32, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg(command)
        .arg("--root")
        .arg(root)
        .args(["--tip", tip, "--json"])
        .args(extra)
        .current_dir(root.parent().unwrap().parent().unwrap())
        .output()
        .unwrap();
    (
        output.status.code().unwrap(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

fn fails(root: &Path, tip: &str, code: &str) {
    let (status, report) = compare(root, tip);
    assert_eq!(status, 2, "{report}");
    assert!(report.contains(code), "expected {code}: {report}");
}

#[path = "merge_cases.rs"]
mod cases;

#[test]
fn merge_preserves_records_even_when_the_candidate_has_none() {
    let (temp, root, tip) = baseline(Some(&record()));
    assert_eq!(compare(&root, &tip).0, 0);
    fs::remove_dir_all(root.join(format!("reports/{VERSION}"))).unwrap();
    fs::remove_file(temp.path().join(request_path())).unwrap();
    fails(&root, &tip, "DEPTH_RECORD_REMOVED");
}

#[test]
fn merge_checks_statement_changes_and_first_creation_against_a_frozen_request() {
    let (temp, root, tip) = baseline(None);
    let mut rec = record();
    write_record(&root, &rec);
    fails(&root, &tip, "DEPTH_RECORD_WITHOUT_FREEZE");
    freeze(temp.path(), &rec);
    let (status, report) = compare(&root, &tip);
    assert_eq!(status, 0, "{report}");
    git(temp.path(), &["add", "."]);
    git(temp.path(), &["commit", "-qm", "freeze"]);
    let tip = git(temp.path(), &["rev-parse", "HEAD"]);
    rec["event"]["request_recorded_at"] = json!("2026-10-20T00:00:00Z");
    write_record(&root, &rec);
    freeze(temp.path(), &rec);
    assert_eq!(compare(&root, &tip).0, 0, "nightly redispatch");
    fs::remove_file(temp.path().join(request_path())).unwrap();
    fails(&root, &tip, "DEPTH_RECORD_WITHOUT_FREEZE");
}

#[test]
fn merge_entries_are_append_only_and_do_not_require_surviving_working_files() {
    let mut rec = record();
    rec["acceptance_entries"] = json!([{ "backs": "promotion", "targets": [REQUIRED_TARGET], "record_sha256": "a".repeat(64) }]);
    let (temp, root, tip) = baseline(Some(&rec));
    fs::remove_file(temp.path().join(request_path())).unwrap();
    assert_eq!(
        compare(&root, &tip).0,
        0,
        "historical entries need no working request"
    );
    let entry = json!({ "backs": "promotion", "targets": [REQUIRED_TARGET], "record_sha256": "b".repeat(64) });
    rec["acceptance_entries"]
        .as_array_mut()
        .unwrap()
        .push(entry.clone());
    write_record(&root, &rec);
    let (status, report) = compare(&root, &tip);
    assert_eq!(status, 2, "{report}");
    assert!(report.contains("DEPTH_ADMISSION_REFUSED"), "{report}");
    for entries in [
        json!([]),
        json!([entry.clone()]),
        json!([entry, rec["acceptance_entries"][0].clone()]),
    ] {
        let mut changed = rec.clone();
        changed["acceptance_entries"] = entries;
        write_record(&root, &changed);
        fails(&root, &tip, "DEPTH_ACCEPTANCE_ENTRIES_CHANGED");
    }
}

#[test]
fn merge_rejects_a_selector_that_keeps_its_name_but_changes_its_meaning() {
    let (temp, root, tip) = baseline(Some(&record()));
    let mut rec = record();
    rec["policy"]["selection"]["enrollment"]["modes"][0]["predicates"][0]["value"] = json!("text");
    seal(&mut rec);
    write_record(&root, &rec);
    freeze(temp.path(), &rec);
    fails(&root, &tip, "DEPTH_SELECTION_SHRUNK");
}

#[test]
fn merge_rejects_invalid_tips_and_fix_mode_without_writing() {
    let (_temp, root, tip) = baseline(None);
    for bad in [
        "no-such-ref",
        "HEAD:Cargo.toml",
        "--help",
        "0000000000000000000000000000000000000000",
    ] {
        let (status, report) = compare(&root, bad);
        assert_ne!(status, 0, "{report}");
    }
    let pointer = root.join(format!("pointers/latest_validated/{REQUIRED_TARGET}.txt"));
    fs::remove_file(&pointer).unwrap();
    let (status, report) = compare_args(&root, &tip, "codex-validate", &["--mode", "fix"]);
    assert_eq!(status, 3, "{report}");
    assert!(!pointer.exists(), "merge checking must not repair files");
}

#[test]
fn merge_acceptance_pointer_requires_the_candidate_statement_entry() {
    let (temp, root, _) = baseline(Some(&record()));
    let pointer = root.join(format!("pointers/latest_supported/{REQUIRED_TARGET}.txt"));
    write_text(&pointer, "none\n");
    republish(&root);
    git(temp.path(), &["add", "."]);
    git(temp.path(), &["commit", "-qm", "unpromoted"]);
    let tip = git(temp.path(), &["rev-parse", "HEAD"]);
    write_text(&pointer, &format!("{VERSION}\n"));
    fails(&root, &tip, "DEPTH_ACCEPTANCE_ENTRY_MISSING");
    let mut rec = record();
    let parsed =
        xtask::depth_record::DepthRecord::parse(&rec.to_string(), "codex", VERSION).unwrap();
    rec["acceptance_entries"] = json!([{ "backs": "promotion", "targets": [REQUIRED_TARGET], "record_sha256": parsed.stated_sha256().unwrap() }]);
    write_record(&root, &rec);
    let (status, report) = compare(&root, &tip);
    assert_eq!(status, 2, "{report}");
    assert!(report.contains("DEPTH_ADMISSION_REFUSED"), "{report}");
}

#[test]
fn merge_acceptance_runtime_source_cannot_diverge_from_its_revision_inputs() {
    let (temp, root, tip) = baseline(Some(&record()));
    write_text(
        &temp
            .path()
            .join("crates/agent_api/src/runtime_support_data.rs"),
        "const OTHER: () = ();\n",
    );
    let (status, report) = compare_as_published(&root, &tip, "manifest-validate", &[]);
    assert_eq!(status, 2, "{report}");
    assert!(report.contains("runtime_support_data.rs"), "{report}");
}

#[path = "acceptance.rs"]
mod acceptance;
