use xtask::depth_record::part_identity;

use super::*;

#[path = "depth_records/merge.rs"]
mod merge;

const COMMIT: &str = "1111111111111111111111111111111111111111";
const RECORDED_AT: &str = "2026-10-10T00:00:00Z";
const OPENED_FROM: &str = ".github/workflows/agent-maintenance-open-pr.yml";
const GOVERNANCE: &str = "docs/agents/lifecycle/codex-cli-onboarding/governance";
const MAINTENANCE: &str = "docs/agents/lifecycle/codex-maintenance/governance";

/// A record with only its Event and P, for codex at `VERSION`, its P identity computed.
fn record() -> Value {
    let mut record = json!({
        "schema_version": 1,
        "event": { "lifecycle_path": "maintenance", "request_commit": COMMIT,
                   "request_recorded_at": RECORDED_AT, "opened_from": OPENED_FROM,
                   "trigger_kind": "upstream_release_detected" },
        "policy": {
            "identity": "",
            "selection": { "agent_id": "codex", "enrollment": {
                "lifecycle_path": "maintenance",
                "version": VERSION,
                "targets": [REQUIRED_TARGET],
                "modes": [{ "selector": "exec-json", "command_path": "codex exec",
                            "mode": "headless",
                            "predicates": [{ "flag": "--output-format", "value": "json" }] }],
                "operations": [{ "id": "exec", "serves": ["exec-json"], "templates": ["CORE"],
                                 "shared_mapping": "wrapper_only", "forwarding": false,
                                 "not_applicable_permitted": [],
                                 "promise": { "r1": "r", "v1": "v", "g1": "g", "e1": "e",
                                              "a1": "a", "x1": "x", "x2": "x", "x3": "x",
                                              "x4": "x" } }],
                "capabilities": []
            } },
            "rules_revision": "1",
            "base": ["A1", "E1", "G1", "R1", "V1", "X1", "X2", "X3", "X4"],
            "templates": [{ "template": "CORE", "obligations": ["LC2", "OUT3"] }],
            "path_enabled": false,
            "debt_baseline": []
        },
        "obligations": null,
        "execution": null,
        "acceptance_entries": []
    });
    record["policy"]["identity"] = json!(part_identity(&record["policy"]).unwrap());
    record
}

fn write_record(codex_dir: &Path, record: &Value) {
    let path = codex_dir.join(format!("reports/{VERSION}/depth-record.json"));
    write_text(&path, &record.to_string());
}

/// The standing request for `VERSION`, with the `[support_depth]` table when `policy` is given.
fn write_request(workspace: &Path, request_commit: &str, policy: Option<&str>) {
    let mut text = format!(
        "artifact_version = \"2\"\nagent_id = \"codex\"\ntrigger_kind = \"upstream_release_detected\"\n\
         opened_from = \"{OPENED_FROM}\"\nrequest_recorded_at = \"{RECORDED_AT}\"\n\
         request_commit = \"{request_commit}\"\n\n[detected_release]\ntarget_version = \"{VERSION}\"\n"
    );
    if let Some(policy) = policy {
        text.push_str(&format!(
            "\n[support_depth]\npolicy_identity = \"{policy}\"\n"
        ));
    }
    write_text(
        &workspace.join(MAINTENANCE).join("maintenance-request.toml"),
        &text,
    );
}

/// Validate as a change has to be validated: with the support publication regenerated for the
/// tree, as the change that writes a record regenerates it. Returns whether it passed and what it
/// reported.
fn validate(codex_dir: &Path) -> (bool, String) {
    republish(codex_dir);
    validate_as_published(codex_dir)
}

/// Rewrite the support matrix artifact from the generator's rows, where the tree derives them. A
/// tree that does not derive keeps its publication, which then fails as well.
fn republish(codex_dir: &Path) {
    let Some(workspace) = fs::canonicalize(codex_dir)
        .ok()
        .and_then(|dir| Some(dir.parent()?.parent()?.to_path_buf()))
    else {
        return;
    };
    if !workspace.join("crates/agent_api/Cargo.toml").exists() {
        copy_dir_recursive(
            &workspace_root().join("crates/agent_api/src"),
            &workspace.join("crates/agent_api/src"),
        );
        fs::copy(
            workspace_root().join("crates/agent_api/Cargo.toml"),
            workspace.join("crates/agent_api/Cargo.toml"),
        )
        .unwrap();
    }
    if let Ok(rows) = xtask::support_matrix::derive_rows(&workspace) {
        let bundle = xtask::support_matrix::render_publication_bundle(&rows, String::new())
            .expect("render publication");
        write_text(
            &workspace.join("cli_manifests/support_matrix/current.json"),
            &bundle.json,
        );
        write_text(&workspace.join("docs/specs/unified-agent-api/support-matrix.md"), &format!("<!-- support-matrix-published:start -->\n{}<!-- support-matrix-published:end -->\n",bundle.markdown));
        if let Ok(source) = xtask::support_matrix::render_agent_api_runtime_support_data(&workspace)
        {
            write_text(
                &workspace.join(xtask::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH),
                &source,
            );
        }
        if let Ok(source) = xtask::capability_matrix::generate_markdown_for_workspace(&workspace) {
            write_text(
                &workspace.join(xtask::capability_matrix::DEFAULT_OUT_PATH),
                &source,
            );
        }
    }
}

/// Validate with the publication as it stands.
fn validate_as_published(codex_dir: &Path) -> (bool, String) {
    let output = run_xtask_validate(codex_dir);
    let report = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.success(), report)
}

fn assert_fails_with(codex_dir: &Path, code: &str) {
    let (passed, report) = validate(codex_dir);
    assert!(
        !passed && report.contains(code),
        "expected {code}:\n{report}"
    );
}

/// As `assert_fails_with`, and the violation is reported at `path`, relative to the workspace.
fn assert_fails_at(codex_dir: &Path, code: &str, path: &str) {
    let (passed, report) = validate(codex_dir);
    let at = format!("{code} error {path} ");
    assert!(
        !passed && report.lines().any(|line| line.starts_with(&at)),
        "expected {code} at {path}:\n{report}"
    );
}

fn record_path() -> String {
    format!("cli_manifests/codex/reports/{VERSION}/depth-record.json")
}

fn request_path() -> String {
    format!("{MAINTENANCE}/maintenance-request.toml")
}

/// A record that also states O, whose identity a request can freeze.
fn record_with_obligations() -> Value {
    let mut record = record();
    let policy = record["policy"]["identity"].clone();
    record["obligations"] = json!({
        "identity": "", "policy_identity": policy, "targets": [REQUIRED_TARGET],
        "acquired_inputs": [],
        "operations": [{ "id": "exec", "surfaces": [],
                         "obligations": ["A1", "E1", "G1", "LC2", "OUT3", "R1", "V1", "X1", "X2",
                                         "X3", "X4"] }],
        "classification_required": [],
        "acceptance_work": []
    });
    record["obligations"]["identity"] = json!(part_identity(&record["obligations"]).unwrap());
    record
}

/// The standing request, with `edit` applied to the text `write_request` writes.
fn edit_request(workspace: &Path, edit: impl Fn(String) -> String) {
    let path = workspace.join(request_path());
    write_text(&path, &edit(fs::read_to_string(&path).unwrap()));
}

/// The lifecycle record, approval and publication packet codex holds, as the lifecycle test copies.
fn copy_lifecycle(workspace: &Path) {
    for file in [
        "lifecycle-state.json",
        "approved-agent.toml",
        "publication-ready.json",
    ] {
        let src = workspace_root().join(GOVERNANCE).join(file);
        write_text(
            &workspace.join(GOVERNANCE).join(file),
            &fs::read_to_string(src).unwrap(),
        );
    }
}

#[test]
fn c0_validate_reads_a_depth_record_and_refuses_one_that_does_not_parse() {
    let temp = make_temp_dir("ccm-c0-depth-parse");
    let codex_dir = materialize_minimal_valid_workspace(&temp);
    write_record(&codex_dir, &record());
    let (passed, report) = validate(&codex_dir);
    assert!(passed, "a record with Event and P only passes:\n{report}");

    // Check 13: a part that changes while its identity does not.
    let mut stale = record();
    stale["policy"]["path_enabled"] = json!(true);
    write_record(&codex_dir, &stale);
    assert_fails_at(&codex_dir, "DEPTH_RECORD_INVALID", &record_path());

    // A record that cannot be read is not taken for no record.
    let path = codex_dir.join(format!("reports/{VERSION}/depth-record.json"));
    fs::remove_file(&path).unwrap();
    std::os::unix::fs::symlink("absent.json", &path).unwrap();
    assert_fails_with(&codex_dir, "DEPTH_RECORD_INVALID");
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    assert_fails_with(&codex_dir, "DEPTH_RECORD_INVALID");
}

#[test]
fn c0_validate_refuses_a_depth_record_in_a_root_that_does_not_adopt_annex_b() {
    let temp = make_temp_dir("ccm-c0-depth-unadopted");
    let codex_dir = materialize_minimal_valid_workspace(&temp);
    write_record(&codex_dir, &record());
    let rules_path = codex_dir.join("RULES.json");
    let mut rules: Value = serde_json::from_str(&fs::read_to_string(&rules_path).unwrap()).unwrap();
    rules["storage"]["committed"]["reports"]
        .as_object_mut()
        .unwrap()
        .remove("depth_record");
    write_json(&rules_path, &rules);
    assert_fails_with(&codex_dir, "DEPTH_RECORD_UNADOPTED");

    // The record is not read any further: the root has not adopted the reader that would.
    let mut stale = record();
    stale["policy"]["path_enabled"] = json!(true);
    write_record(&codex_dir, &stale);
    let (_, report) = validate(&codex_dir);
    assert!(
        report.contains("DEPTH_RECORD_UNADOPTED") && !report.contains("DEPTH_RECORD_INVALID"),
        "{report}"
    );

    // A root that names the record elsewhere, another root's place included, is misconfigured.
    rules["storage"]["committed"]["reports"]["depth_record"] =
        json!("cli_manifests/opencode/reports/<version>/depth-record.json");
    write_json(&rules_path, &rules);
    assert_fails_at(
        &codex_dir,
        "DEPTH_RECORD_PATH_INVALID",
        "cli_manifests/codex/RULES.json",
    );
    rules["storage"]["committed"]["reports"]["depth_record"] = json!("record.json");
    write_json(&rules_path, &rules);
    let (passed, report) = validate(&codex_dir);
    assert!(
        !passed && report.contains("storage.committed.reports.depth_record"),
        "{report}"
    );
}

#[test]
fn c0_validate_requires_the_record_to_state_what_the_standing_request_froze() {
    let temp = make_temp_dir("ccm-c0-depth-request");
    let codex_dir = materialize_minimal_valid_workspace(&temp);
    let record = record();
    let policy = record["policy"]["identity"].as_str().unwrap().to_string();
    write_record(&codex_dir, &record);

    write_request(&temp, COMMIT, Some(&policy));
    let (passed, report) = validate(&codex_dir);
    assert!(
        passed,
        "a record that states the request's Event and P passes:\n{report}"
    );

    // Check 3: another Event, another P, and a request of the version that froze no P.
    write_request(
        &temp,
        "2222222222222222222222222222222222222222",
        Some(&policy),
    );
    assert_fails_at(&codex_dir, "DEPTH_RECORD_NOT_THE_REQUESTS", &record_path());
    // Every member of the Event is compared, not only the commit.
    for (stated, other) in [
        (RECORDED_AT, "2026-10-11T00:00:00Z"),
        ("upstream_release_detected", "manual"),
        (OPENED_FROM, ".github/workflows/other.yml"),
    ] {
        write_request(&temp, COMMIT, Some(&policy));
        edit_request(&temp, |text| text.replace(stated, other));
        assert_fails_with(&codex_dir, "DEPTH_RECORD_NOT_THE_REQUESTS");
    }
    write_request(&temp, COMMIT, Some(&"0".repeat(64)));
    assert_fails_with(&codex_dir, "DEPTH_RECORD_NOT_THE_REQUESTS");
    write_request(&temp, COMMIT, None);
    assert_fails_with(&codex_dir, "DEPTH_RECORD_NOT_THE_REQUESTS");

    // Check 1: the request froze P and the version has no record.
    fs::remove_file(codex_dir.join(format!("reports/{VERSION}/depth-record.json"))).unwrap();
    write_request(&temp, COMMIT, Some(&policy));
    assert_fails_at(&codex_dir, "DEPTH_RECORD_MISSING", &request_path());
    write_request(&temp, COMMIT, None);
    let (passed, report) = validate(&codex_dir);
    assert!(
        passed,
        "a request without the table needs no record:\n{report}"
    );
}

#[test]
fn c0_validate_requires_the_lifecycle_record_of_an_agent_with_a_depth_record_to_report_held_files()
{
    let temp = make_temp_dir("ccm-c0-depth-lifecycle");
    let codex_dir = materialize_minimal_valid_workspace(&temp);
    write_record(&codex_dir, &record());
    for file in [
        "lifecycle-state.json",
        "approved-agent.toml",
        "publication-ready.json",
    ] {
        let src = workspace_root().join(GOVERNANCE).join(file);
        write_text(
            &temp.join(GOVERNANCE).join(file),
            &fs::read_to_string(src).unwrap(),
        );
    }

    // Check 10: codex's lifecycle record states `maintenance_closeout_written`.
    assert_fails_with(&codex_dir, "DEPTH_LIFECYCLE_DANGLING");
    let closeout = fs::read_to_string(
        workspace_root()
            .join(MAINTENANCE)
            .join("maintenance-closeout.json"),
    );
    write_text(
        &temp.join(MAINTENANCE).join("maintenance-closeout.json"),
        &closeout.unwrap(),
    );
    let (passed, report) = validate(&codex_dir);
    assert!(passed, "every reported file is held:\n{report}");

    // A file at a closeout's path that is not a closeout of its kind is not held.
    let maintenance_closeout = temp.join(MAINTENANCE).join("maintenance-closeout.json");
    let held = fs::read_to_string(&maintenance_closeout).unwrap();
    write_text(&maintenance_closeout, "{}\n");
    assert_fails_with(&codex_dir, "DEPTH_LIFECYCLE_DANGLING");
    write_text(&maintenance_closeout, &held);
    let lifecycle_path = temp.join(GOVERNANCE).join("lifecycle-state.json");
    let mut lifecycle: Value =
        serde_json::from_str(&fs::read_to_string(&lifecycle_path).unwrap()).unwrap();
    lifecycle["closeout_baseline_path"] = json!("cli_manifests/codex/RULES.json");
    write_json(&lifecycle_path, &lifecycle);
    assert_fails_with(&codex_dir, "is not a proving-run closeout");
    lifecycle["closeout_baseline_path"] = Value::Null;
    write_json(&lifecycle_path, &lifecycle);

    // A packet that is not the one the lifecycle record names.
    write_text(
        &temp.join(GOVERNANCE).join("publication-ready.json"),
        "{}\n",
    );
    assert_fails_at(
        &codex_dir,
        "DEPTH_LIFECYCLE_DANGLING",
        &format!("{GOVERNANCE}/lifecycle-state.json"),
    );
}

#[test]
fn c0_validate_reads_only_the_closeout_evidence_a_lifecycle_record_states() {
    let temp = make_temp_dir("ccm-c0-depth-evidence");
    let codex_dir = materialize_minimal_valid_workspace(&temp);
    copy_lifecycle(&temp);

    // Check 10 is the check of an agent that has a depth record. Codex's lifecycle record states
    // `maintenance_closeout_written` and this revision holds no maintenance closeout.
    let (passed, report) = validate(&codex_dir);
    assert!(
        passed,
        "an agent with no record is not held to it:\n{report}"
    );

    // Evidence the lifecycle record requires and has not satisfied reports nothing yet.
    write_record(&codex_dir, &record());
    let path = temp.join(GOVERNANCE).join("lifecycle-state.json");
    let stated: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let mut state = stated.clone();
    let satisfied = state["satisfied_evidence"].as_array_mut().unwrap();
    satisfied.retain(|id| id != "maintenance_closeout_written");
    write_json(&path, &state);
    let (passed, report) = validate(&codex_dir);
    assert!(
        passed,
        "evidence that is required alone is not reported:\n{report}"
    );

    // `proving_run_closeout_written` names a proving-run closeout that is held, the maintenance
    // closeout held as well. The closeout is claude_code's, which is in form.
    write_text(
        &temp.join(MAINTENANCE).join("maintenance-closeout.json"),
        &fs::read_to_string(
            workspace_root()
                .join(MAINTENANCE)
                .join("maintenance-closeout.json"),
        )
        .unwrap(),
    );
    let mut state = stated.clone();
    for key in ["required_evidence", "satisfied_evidence"] {
        let ids = state[key].as_array_mut().unwrap();
        ids.push(json!("proving_run_closeout_written"));
    }
    write_json(&path, &state);
    assert_fails_with(&codex_dir, "DEPTH_LIFECYCLE_DANGLING");
    let closeout = format!("{GOVERNANCE}/proving-run-closeout.json");
    let source =
        "docs/agents/lifecycle/claude-code-cli-onboarding/governance/proving-run-closeout.json";
    write_text(
        &temp.join(&closeout),
        &fs::read_to_string(workspace_root().join(source)).unwrap(),
    );
    state["closeout_baseline_path"] = json!(closeout);
    write_json(&path, &state);
    let (passed, report) = validate(&codex_dir);
    assert!(
        passed,
        "a proving-run closeout that is named and in form is held:\n{report}"
    );

    // A lifecycle record that is there and cannot be read fails, as a record that cannot be read
    // does. One that is absent is no lifecycle record.
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    assert_fails_at(
        &codex_dir,
        "DEPTH_LIFECYCLE_DANGLING",
        &format!("{GOVERNANCE}/lifecycle-state.json"),
    );
    fs::remove_dir(&path).unwrap();
    let (passed, report) = validate(&codex_dir);
    assert!(
        passed,
        "an absent lifecycle record has nothing to report:\n{report}"
    );
}

#[test]
fn c0_validate_requires_the_record_to_state_the_obligations_the_request_froze() {
    let temp = make_temp_dir("ccm-c0-depth-obligations");
    let codex_dir = materialize_minimal_valid_workspace(&temp);
    let early = record();
    let policy = early["policy"]["identity"].as_str().unwrap().to_string();
    let full = record_with_obligations();
    let obligations = full["obligations"]["identity"]
        .as_str()
        .unwrap()
        .to_string();
    let freezing = |obligations: &str| {
        write_request(&temp, COMMIT, Some(&policy));
        edit_request(&temp, |text| {
            format!("{text}obligations_identity = \"{obligations}\"\n")
        });
    };

    // The request froze P and O, and the record states both.
    write_record(&codex_dir, &full);
    freezing(&obligations);
    let (passed, report) = validate(&codex_dir);
    assert!(
        passed,
        "a record that states the request's O passes:\n{report}"
    );

    // Another O, a request that froze none, and a request that froze an O the record lacks.
    freezing(&"0".repeat(64));
    assert_fails_with(&codex_dir, "DEPTH_RECORD_NOT_THE_REQUESTS");
    write_request(&temp, COMMIT, Some(&policy));
    assert_fails_with(&codex_dir, "DEPTH_RECORD_NOT_THE_REQUESTS");
    write_record(&codex_dir, &early);
    freezing(&obligations);
    assert_fails_with(&codex_dir, "DEPTH_RECORD_NOT_THE_REQUESTS");
    write_request(&temp, COMMIT, Some(&policy));
    let (passed, report) = validate(&codex_dir);
    assert!(
        passed,
        "O is not yet produced while the request froze none:\n{report}"
    );
}

#[test]
fn c0_validate_refuses_a_depth_record_it_cannot_attribute_to_an_agent() {
    let temp = make_temp_dir("ccm-c0-depth-unanchored");
    let codex_dir = materialize_minimal_valid_workspace(&temp);

    // No registry entry has the manifest root. The copy of codex's root names its own records.
    let unregistered = temp.join("cli_manifests/unregistered");
    copy_dir_recursive(&codex_dir, &unregistered);
    let rules_path = unregistered.join("RULES.json");
    let mut rules: Value = serde_json::from_str(&fs::read_to_string(&rules_path).unwrap()).unwrap();
    rules["storage"]["committed"]["reports"]["depth_record"] =
        json!("cli_manifests/unregistered/reports/<version>/depth-record.json");
    write_json(&rules_path, &rules);
    let (passed, report) = validate(&unregistered);
    assert!(
        passed,
        "a root with no record needs no registry entry:\n{report}"
    );
    write_record(&unregistered, &record());
    assert_fails_at(
        &unregistered,
        "DEPTH_RECORD_UNANCHORED",
        &format!("cli_manifests/unregistered/reports/{VERSION}/depth-record.json"),
    );

    // Outside a workspace there is no registry to ask.
    let outside = make_temp_dir("ccm-c0-depth-outside").join("codex");
    copy_dir_recursive(&codex_dir, &outside);
    let (passed, report) = validate(&outside);
    assert!(
        passed,
        "a root with no record needs no workspace:\n{report}"
    );
    write_record(&outside, &record());
    assert_fails_at(
        &outside,
        "DEPTH_RECORD_UNANCHORED",
        &format!("reports/{VERSION}/depth-record.json"),
    );
}

#[test]
fn c0_validate_names_the_root_it_is_given_as_the_registry_does() {
    let temp = make_temp_dir("ccm-c0-depth-root-forms");
    let codex_dir = materialize_minimal_valid_workspace(&temp);
    let mut stale = record();
    stale["policy"]["path_enabled"] = json!(true);
    write_record(&codex_dir, &stale);

    // The root is the same root whether it is given through `..` or through a symlink.
    let dotted = temp.join("cli_manifests/../cli_manifests/codex");
    assert_fails_at(&dotted, "DEPTH_RECORD_INVALID", &record_path());
    let linked = temp.join("linked");
    std::os::unix::fs::symlink(&codex_dir, &linked).unwrap();
    assert_fails_at(&linked, "DEPTH_RECORD_INVALID", &record_path());

    // The rules file a violation names is where it is in the workspace, not where the root was given.
    let rules_path = codex_dir.join("RULES.json");
    let mut rules: Value = serde_json::from_str(&fs::read_to_string(&rules_path).unwrap()).unwrap();
    rules["storage"]["committed"]["reports"]["depth_record"] = json!("record.json");
    write_json(&rules_path, &rules);
    for root in [&dotted, &linked] {
        assert_fails_at(
            root,
            "DEPTH_RECORD_PATH_INVALID",
            "cli_manifests/codex/RULES.json",
        );
    }
}

#[test]
fn c0_validate_refuses_a_standing_request_that_is_not_the_toml_it_reads() {
    let temp = make_temp_dir("ccm-c0-depth-unreadable");
    let codex_dir = materialize_minimal_valid_workspace(&temp);
    // The request is read whether or not the root holds a record.
    for text in ["this is [not toml\n", "request_commit = 5\n"] {
        write_text(&temp.join(request_path()), text);
        assert_fails_at(&codex_dir, "DEPTH_REQUEST_UNREADABLE", &request_path());
    }

    // A request that cannot be read as text is not TOML either, and neither is a directory.
    let request = temp.join(request_path());
    fs::write(&request, [0xff_u8, 0xfe, b'=']).unwrap();
    assert_fails_at(&codex_dir, "DEPTH_REQUEST_UNREADABLE", &request_path());
    fs::remove_file(&request).unwrap();
    fs::create_dir(&request).unwrap();
    assert_fails_at(&codex_dir, "DEPTH_REQUEST_UNREADABLE", &request_path());
    fs::remove_dir(&request).unwrap();
    std::os::unix::fs::symlink(temp.join("missing.toml"), &request).unwrap();
    assert_fails_at(&codex_dir, "DEPTH_REQUEST_UNREADABLE", &request_path());

    // An absent request is no request.
    fs::remove_file(&request).unwrap();
    let (passed, report) = validate(&codex_dir);
    assert!(passed, "a request that is not there is not read:\n{report}");
}

#[test]
fn c0_validate_requires_a_record_of_a_frozen_p_whether_or_not_the_root_adopts_annex_b() {
    let temp = make_temp_dir("ccm-c0-depth-frozen-unadopted");
    let codex_dir = materialize_minimal_valid_workspace(&temp);
    let rules_path = codex_dir.join("RULES.json");
    let mut rules: Value = serde_json::from_str(&fs::read_to_string(&rules_path).unwrap()).unwrap();
    rules["storage"]["committed"]["reports"]
        .as_object_mut()
        .unwrap()
        .remove("depth_record");
    write_json(&rules_path, &rules);
    let policy = record()["policy"]["identity"].as_str().unwrap().to_string();
    write_request(&temp, COMMIT, Some(&policy));
    assert_fails_at(&codex_dir, "DEPTH_RECORD_MISSING", &request_path());
}

#[test]
fn c0_validate_requires_the_published_depth_facts_to_be_the_ones_the_record_decides() {
    let temp = make_temp_dir("ccm-c0-depth-publication");
    let codex_dir = materialize_minimal_valid_workspace(&temp);
    write_record(&codex_dir, &record());
    let (passed, report) = validate(&codex_dir);
    assert!(passed, "the regenerated publication passes:\n{report}");

    let path = temp.join("cli_manifests/support_matrix/current.json");
    let published: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(published["schema_version"], json!(2));
    let at = published["rows"]
        .as_array()
        .unwrap()
        .iter()
        .position(|row| row.get("depth").is_some())
        .expect("the record's row is depth-enrolled");
    let edit = |edit: &dyn Fn(&mut Value)| {
        let mut artifact = published.clone();
        edit(&mut artifact);
        write_json(&path, &artifact);
        validate_as_published(&codex_dir)
    };
    let fails_with = |(passed, report): (bool, String), code: &str| {
        assert!(
            !passed && report.contains(code),
            "expected {code}:\n{report}"
        );
    };

    // Check 6: a tuple the record selects published as not depth-enrolled, in the revision that
    // has no depth facts at all.
    fails_with(
        edit(&|artifact| {
            artifact["rows"][at]
                .as_object_mut()
                .unwrap()
                .remove("depth");
            artifact["schema_version"] = json!(1);
        }),
        "SUPPORT_MATRIX_DEPTH_MISMATCH",
    );
    // Check 5: obligations published as frozen that no frozen O states.
    fails_with(
        edit(&|artifact| artifact["rows"][at]["depth"]["obligations_frozen"] = json!(true)),
        "SUPPORT_MATRIX_DEPTH_MISMATCH",
    );
    // A row with depth facts in an artifact that says it has none.
    fails_with(
        edit(&|artifact| artifact["schema_version"] = json!(1)),
        "SUPPORT_MATRIX_SCHEMA_INVALID",
    );
    // And an artifact that says it has them where no row does.
    fails_with(
        edit(&|artifact| {
            for row in artifact["rows"].as_array_mut().unwrap() {
                row.as_object_mut().unwrap().remove("depth");
            }
        }),
        "SUPPORT_MATRIX_SCHEMA_INVALID",
    );
    // A member the revision does not define is refused, not ignored, in `depth`, in a row and in
    // the artifact.
    let edits: [&dyn Fn(&mut Value); 3] = [
        &|artifact| artifact["rows"][at]["depth"]["qualified"] = json!(true),
        &|artifact| artifact["rows"][at]["qualified"] = json!(true),
        &|artifact| artifact["qualified"] = json!(true),
    ];
    for unknown in edits {
        fails_with(edit(unknown), "SUPPORT_MATRIX_INVALID_JSON");
    }
}

#[test]
fn c0_validate_accepts_the_row_a_record_implies_for_a_version_without_metadata() {
    let temp = make_temp_dir("ccm-c0-depth-record-only");
    let codex_dir = materialize_minimal_valid_workspace(&temp);
    // A record for a version that no `versions/<version>.json` describes yet.
    let mut record = record();
    record["policy"]["selection"]["enrollment"]["version"] = json!("0.62.0");
    record["policy"]["identity"] = json!(part_identity(&record["policy"]).unwrap());
    write_text(
        &codex_dir.join("reports/0.62.0/depth-record.json"),
        &record.to_string(),
    );
    let (passed, report) = validate(&codex_dir);
    assert!(passed, "{report}");

    let published: Value = serde_json::from_str(
        &fs::read_to_string(temp.join("cli_manifests/support_matrix/current.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(published["schema_version"], json!(2));
    let row = published["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["agent"] == json!("codex") && row["version"] == json!("0.62.0"))
        .expect("the record implies a row");
    assert_eq!(row["target"], json!(REQUIRED_TARGET));
    assert_eq!(row["manifest_support"], json!("unsupported"));
    assert!(row.get("depth").is_some());
}
