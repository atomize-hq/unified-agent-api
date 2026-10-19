use xtask::depth_record::part_identity;

use super::*;

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

/// Validate, returning whether it passed and what it reported.
fn validate(codex_dir: &Path) -> (bool, String) {
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

    // A root that names the record elsewhere is a RULES.json error.
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
    assert_fails_with(&codex_dir, "DEPTH_RECORD_NOT_THE_REQUESTS");
    write_request(&temp, COMMIT, Some(&"0".repeat(64)));
    assert_fails_with(&codex_dir, "DEPTH_RECORD_NOT_THE_REQUESTS");
    write_request(&temp, COMMIT, None);
    assert_fails_with(&codex_dir, "DEPTH_RECORD_NOT_THE_REQUESTS");

    // Check 1: the request froze P and the version has no record.
    fs::remove_file(codex_dir.join(format!("reports/{VERSION}/depth-record.json"))).unwrap();
    write_request(&temp, COMMIT, Some(&policy));
    assert_fails_with(&codex_dir, "DEPTH_RECORD_MISSING");
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

    // A packet that is not the one the lifecycle record names.
    write_text(
        &temp.join(GOVERNANCE).join("publication-ready.json"),
        "{}\n",
    );
    assert_fails_with(&codex_dir, "DEPTH_LIFECYCLE_DANGLING");
}
