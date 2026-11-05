//! Slice 5: only the installed synthetic onboarding route can freeze a dry-run packet.
use super::*;
use crate::{agent_lifecycle as life, runtime_follow_on as runtime};
use sha2::{Digest, Sha256};

#[path = "runtime_follow_on/authority.rs"]
mod authority;

#[path = "../../prepare_publication/acquired_inputs_tests.rs"]
mod acquired_inputs_tests;

const PACK: &str = "depth-fixture-cli-onboarding";
const APPROVAL: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/approved-agent.toml";
const STATE: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/lifecycle-state.json";
const STATEMENT: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/depth-acquisition/0.2.0.toml";
const ROOT: &str = "cli_manifests/depth_fixture";
const RECORD: &str = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";
const RUN: &str = "docs/agents/.uaa-temp/runtime-follow-on/runs/freeze";
const TEMPLATE: &str = include_str!(
    "../../../tests/fixtures/depth-synthetic/onboarding/governance/depth-acquisition/0.2.0.toml"
);

fn setup(acquired: bool) -> Fixture {
    let (f, _) = super::super::prepare::workspace();
    let root = f.root.path();
    write(
        root,
        REGISTRY_RELATIVE_PATH,
        format!(
            "{}\n{}",
            include_str!("../../../data/agent_registry.toml"),
            include_str!(
                "../../../tests/fixtures/depth-synthetic/onboarding/registry-entry-deferred.toml"
            )
        ),
    );
    let approval = include_str!(
        "../../../tests/fixtures/depth-synthetic/onboarding/governance/approved-agent.toml"
    )
    .replace("{{APPROVAL_COMMIT}}", "abc1234")
    .replace("{{APPROVAL_RECORDED_AT}}", "2026-11-05T00:00:00Z");
    write(root, APPROVAL, &approval);
    let evidence = life::required_evidence_for_stage(life::LifecycleStage::Enrolled);
    let state: life::LifecycleState = serde_json::from_value(json!({
        "schema_version":life::LIFECYCLE_SCHEMA_VERSION,
        "agent_id":"depth_fixture", "onboarding_pack_prefix":PACK,
        "approval_artifact_path":APPROVAL,
        "approval_artifact_sha256":hex::encode(Sha256::digest(approval.as_bytes())),
        "lifecycle_stage":"enrolled", "support_tier":"bootstrap", "side_states":[],
        "current_owner_command":"onboard-agent", "expected_next_command":"runtime-follow-on",
        "last_transition_at":"2026-11-05T00:00:00Z", "last_transition_by":"fixture",
        "required_evidence":evidence, "satisfied_evidence":evidence,
        "blocking_issues":[], "retryable_failures":[],
    }))
    .unwrap();
    state.validate().unwrap();
    write(root, STATE, serde_json::to_vec_pretty(&state).unwrap());
    write(root, ".gitignore", "docs/agents/.uaa-temp/\n");
    write(
        root,
        &format!("{ROOT}/versions/0.2.0.json"),
        r#"{"schema_version":1,"semantic_version":"0.2.0"}"#,
    );
    if acquired {
        super::super::prepare::acquire(root, "0.2.0");
        statement(&f);
    }
    f.commit();
    f
}
fn statement(f: &Fixture) {
    let mut doc: toml_edit::DocumentMut = TEMPLATE.parse().unwrap();
    for input in doc["inputs"].as_array_of_tables_mut().unwrap().iter_mut() {
        let path = input["path"].as_str().unwrap();
        input["sha256"] = toml_edit::value(hex::encode(Sha256::digest(
            fs::read(f.root.path().join(path)).unwrap(),
        )));
    }
    write(f.root.path(), STATEMENT, doc.to_string());
}
fn args() -> runtime::Args {
    runtime::Args {
        approval: APPROVAL.into(),
        dry_run: true,
        write: false,
        requested_tier: runtime::RequestedTier::Default,
        minimal_justification_file: None,
        allow_rich_surface: vec![],
        run_id: Some("freeze".into()),
        codex_binary: Some("/must-never-execute".into()),
    }
}
fn freeze(f: &Fixture) -> Result<String, String> {
    let mut output = Vec::new();
    runtime::isolated_depth_dry_run(f.root.path(), args(), &mut output)
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8(output).unwrap())
}
fn json_file(f: &Fixture, path: &str) -> Value {
    serde_json::from_slice(&fs::read(f.root.path().join(path)).unwrap()).unwrap()
}
fn refuses_unchanged(f: &Fixture, expected: &str) {
    let before = git(f.root.path(), &["status", "--porcelain=v1", "--ignored"]);
    let state = fs::read(f.root.path().join(STATE)).unwrap();
    let record = fs::read(f.root.path().join(RECORD)).ok();
    let error = freeze(f).unwrap_err();
    assert!(error.contains(expected), "expected {expected}: {error}");
    assert_eq!(fs::read(f.root.path().join(STATE)).unwrap(), state);
    assert_eq!(fs::read(f.root.path().join(RECORD)).ok(), record);
    assert_eq!(
        git(f.root.path(), &["status", "--porcelain=v1", "--ignored"]),
        before
    );
}
#[test]
fn runtime_follow_on_freezes_p_record_object_and_contract_then_o() {
    let f = setup(false);
    let _installed = f.install();
    let output = freeze(&f).unwrap();
    assert!(output.contains("O not yet produced"), "{output}");
    assert!(output.contains("closes"), "{output}");
    let p = json_file(&f, RECORD);
    assert!(p["obligations"].is_null());
    let object = json_file(&f, STATE)["support_depth"].clone();
    assert_eq!(object["policy_identity"], p["policy"]["identity"]);
    let contract = json_file(&f, &format!("{RUN}/input-contract.json"));
    assert_eq!(contract["support_depth"]["version"], "0.2.0");
    assert_eq!(
        contract["support_depth"]["policy_identity"],
        object["policy_identity"]
    );
    assert!(contract["support_depth"]
        .get("obligations_identity")
        .is_none());
    f.commit();
    super::super::prepare::acquire(f.root.path(), "0.2.0");
    statement(&f);
    f.commit();
    freeze(&f).unwrap();
    let o = json_file(&f, RECORD);
    assert_eq!(p["policy"], o["policy"]);
    assert_eq!(
        json_file(&f, STATE)["support_depth"]["obligations_identity"],
        o["obligations"]["identity"]
    );
    assert_eq!(
        json_file(&f, &format!("{RUN}/input-contract.json"))["support_depth"]
            ["obligations_identity"],
        o["obligations"]["identity"]
    );
}
#[test]
fn runtime_follow_on_no_route_and_public_entry_refuse_without_writes() {
    let f = setup(true);
    refuses_unchanged(&f, "isolated route");
    let _installed = f.install();
    assert!(runtime::run_in_workspace(f.root.path(), args(), &mut Vec::new()).is_err());
    assert!(!f.root.path().join(RECORD).exists());
    assert!(!f.root.path().join(RUN).exists());
}

fn edit_json(f: &Fixture, path: &str, edit: impl FnOnce(&mut Value)) {
    let mut value = json_file(f, path);
    edit(&mut value);
    write(
        f.root.path(),
        path,
        serde_json::to_vec_pretty(&value).unwrap(),
    );
}
fn edit_approval(f: &Fixture, edit: impl FnOnce(&mut toml_edit::DocumentMut)) {
    let mut doc: toml_edit::DocumentMut = fs::read_to_string(f.root.path().join(APPROVAL))
        .unwrap()
        .parse()
        .unwrap();
    edit(&mut doc);
    let raw = doc.to_string();
    write(f.root.path(), APPROVAL, &raw);
    edit_json(f, STATE, |s| {
        s["approval_artifact_sha256"] = json!(hex::encode(Sha256::digest(raw.as_bytes())))
    });
}
/// Commit to the onboarding branch while leaving the protected integration tip at `protected`.
fn branch_commit(f: &Fixture, protected: &str) {
    f.commit();
    let head = git(f.root.path(), &["rev-parse", "HEAD"]);
    f.api.put(
        "repos/fixture/isolated/branches/staging",
        json!({"name":"staging","protected":true,"commit":{"sha":protected}}),
    );
    f.api.put(
        &format!("repos/fixture/isolated/compare/{protected}...{head}"),
        json!({"status":"ahead","merge_base_commit":{"sha":protected}}),
    );
}
#[test]
fn runtime_follow_on_refuses_missing_malformed_mismatched_and_extra_statement_inputs() {
    for case in 0..6 {
        let f = setup(true);
        match case {
            0 => fs::remove_file(f.root.path().join(STATEMENT)).unwrap(),
            1 => write(
                f.root.path(),
                STATEMENT,
                TEMPLATE.replace("{{SHA256}}", &"0".repeat(64)),
            ),
            2 => {
                let raw = fs::read_to_string(f.root.path().join(STATEMENT)).unwrap();
                write(
                    f.root.path(),
                    STATEMENT,
                    raw.replace("agent_id = \"depth_fixture\"", "agent_id = \"other\""),
                );
            }
            3 => {
                let raw = fs::read_to_string(f.root.path().join(STATEMENT)).unwrap();
                write(f.root.path(), STATEMENT, format!("unknown = true\n{raw}"));
            }
            4 => {
                let mut doc: toml_edit::DocumentMut =
                    fs::read_to_string(f.root.path().join(STATEMENT))
                        .unwrap()
                        .parse()
                        .unwrap();
                doc["inputs"].as_array_of_tables_mut().unwrap().remove(0);
                write(f.root.path(), STATEMENT, doc.to_string());
            }
            _ => {
                let raw = fs::read_to_string(f.root.path().join(STATEMENT)).unwrap();
                write(f.root.path(), STATEMENT, format!("{raw}\n[[inputs]]\npath = \"unbound.json\"\nsha256 = \"{}\"\ncommand = \"cp unbound.json unbound.json\"\n", "a".repeat(64)));
            }
        }
        f.commit();
        let _installed = f.install();
        refuses_unchanged(&f, "acquisition statement");
    }
}
#[test]
fn runtime_follow_on_branch_local_statement_is_ineffective_until_integration() {
    let f = setup(true);
    let statement_bytes = fs::read(f.root.path().join(STATEMENT)).unwrap();
    fs::remove_file(f.root.path().join(STATEMENT)).unwrap();
    f.commit();
    let protected = git(f.root.path(), &["rev-parse", "HEAD"]);
    write(f.root.path(), STATEMENT, statement_bytes);
    branch_commit(&f, &protected);
    let _installed = f.install();
    // The acquired surface is already integrated; committing the statement on this branch gives
    // no provenance. Commit timing (including pre-declaration production) is never authority.
    refuses_unchanged(&f, "acquisition statement missing at integration ref");
    f.host();
    freeze(&f).unwrap();
}
#[test]
fn runtime_follow_on_requires_inputs_to_equal_both_statement_and_protected_tip() {
    for equal_statement in [false, true] {
        let f = setup(true);
        let path = format!("{ROOT}/wrapper_coverage.json");
        let original = fs::read(f.root.path().join(&path)).unwrap();
        let changed = [original.as_slice(), b"\n"].concat();
        if equal_statement {
            // The protected statement deliberately describes the next branch's bytes, while the
            // protected input still has old bytes. Matching the statement alone is insufficient.
            write(f.root.path(), &path, &changed);
            statement(&f);
            write(f.root.path(), &path, &original);
            f.commit();
        }
        let protected = git(f.root.path(), &["rev-parse", "HEAD"]);
        write(f.root.path(), &path, &changed);
        branch_commit(&f, &protected);
        let _installed = f.install();
        refuses_unchanged(
            &f,
            if equal_statement {
                "acquired inputs differ from integration ref"
            } else {
                "acquisition statement does not state exactly"
            },
        );
        assert!(freeze(&f).unwrap_err().contains(&path));
    }
}
#[test]
fn runtime_follow_on_incomplete_union_names_missing_targets_without_a_freeze() {
    let f = setup(true);
    edit_json(&f, &format!("{ROOT}/snapshots/0.2.0/union.json"), |u| {
        u["complete"] = json!(false);
        u["expected_targets"] = json!(["x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"]);
        u["missing_targets"] = json!(["aarch64-unknown-linux-gnu"]);
    });
    f.commit();
    let _installed = f.install();
    refuses_unchanged(&f, "OF-P7");
    assert!(freeze(&f)
        .unwrap_err()
        .contains("aarch64-unknown-linux-gnu"));
}
#[test]
fn runtime_follow_on_rule_three_and_four_refusals_leave_no_packet_or_freeze() {
    for case in 0..6 {
        let f = setup(false);
        match case {
            0 => edit_approval(&f, |d| {
                d["support_depth"]["version"] = toml_edit::value("0.3.0")
            }),
            1 => {
                fs::remove_file(f.root.path().join(format!("{ROOT}/versions/0.2.0.json"))).unwrap();
            }
            2 => edit_json(&f, &format!("{ROOT}/versions/0.2.0.json"), |m| {
                m["semantic_version"] = json!("0.0.0")
            }),
            3 => write(
                f.root.path(),
                &format!("{ROOT}/VALIDATOR_SPEC.md"),
                "# Not adopted\n",
            ),
            4 => edit_json(&f, &format!("{ROOT}/RULES.json"), |r| {
                r["storage"]["committed"]["reports"]
                    .as_object_mut()
                    .unwrap()
                    .remove("depth_record")
                    .map(|_| ())
                    .unwrap()
            }),
            _ => edit_json(&f, &format!("{ROOT}/RULES.json"), |r| {
                r["union"]["expected_targets"] = json!(["other-target"]);
            }),
        }
        f.commit();
        let _installed = f.install();
        refuses_unchanged(
            &f,
            if case == 0 {
                "rule 3 unresolved"
            } else if case == 1 {
                "missing bound input"
            } else {
                "rule 4"
            },
        );
    }
}
#[test]
fn runtime_follow_on_narrow_approval_reader_refuses_unknown_backend_and_misattribution() {
    for case in 0..4 {
        let f = setup(false);
        edit_approval(&f, |d| match case {
            0 => d["descriptor"]["backend_module"] = toml_edit::value("fake"),
            1 => d["descriptor"]["unknown"] = toml_edit::value(true),
            2 => d["descriptor"]["maintenance"]["deferral"]["unknown"] = toml_edit::value(true),
            _ => d["descriptor"]["manifest_root"] = toml_edit::value("cli_manifests/other"),
        });
        f.commit();
        let _installed = f.install();
        refuses_unchanged(
            &f,
            if case == 3 {
                "manifest_root"
            } else {
                "private approval reader"
            },
        );
    }
}
#[test]
fn runtime_follow_on_changed_p_and_post_integration_dry_runs_refuse() {
    let f = setup(false);
    let _installed = f.install();
    freeze(&f).unwrap();
    f.commit();
    f.registry(|d| {
        let entry = d["agents"]
            .as_array_of_tables_mut()
            .unwrap()
            .iter_mut()
            .find(|e| e["agent_id"].as_str() == Some("depth_fixture"))
            .unwrap();
        entry["support_depth"]["enrollments"][0]["operations"][0]["promise"]["r1"] =
            toml_edit::value("changed promise");
    });
    f.commit();
    refuses_unchanged(&f, "selection differs");
    drop(_installed);
    let f = setup(true);
    let _installed = f.install();
    let original = json_file(&f, STATE);
    edit_json(&f, STATE, |s| {
        s["lifecycle_stage"] = json!("runtime_integrated");
        s["support_tier"] = json!("baseline_runtime");
        s["active_runtime_evidence_run_id"] = json!("passed-write");
        s["implementation_summary"] = json!({
            "requested_runtime_profile":"default", "achieved_runtime_profile":"default",
            "primary_template":"opencode", "template_lineage":["opencode"],
            "landed_surfaces":[], "deferred_surfaces":[],
        });
        s["required_evidence"] = json!(life::required_evidence_for_stage(
            life::LifecycleStage::RuntimeIntegrated
        ));
        s["satisfied_evidence"] = s["required_evidence"].clone();
    });
    assert_ne!(original, json_file(&f, STATE));
    f.commit();
    refuses_unchanged(&f, "after runtime_integrated a new approval is required");
}
#[test]
fn runtime_follow_on_failed_write_retry_preserves_p_and_requires_provenance() {
    let f = setup(true);
    let _installed = f.install();
    freeze(&f).unwrap();
    let frozen = json_file(&f, RECORD);
    edit_json(&f, STATE, |s| {
        s["side_states"] = json!(["failed_retryable"]);
        s["retryable_failures"] = json!(["failed candidate delivered no executor changes"]);
    });
    f.commit();
    freeze(&f).unwrap();
    assert_eq!(json_file(&f, RECORD), frozen);
    // Even a byte-identical re-acquisition after failure still needs the effective statement.
    fs::remove_file(f.root.path().join(STATEMENT)).unwrap();
    f.commit();
    refuses_unchanged(&f, "acquisition statement missing");
}

#[test]
fn runtime_follow_on_release_watch_is_optional_and_preview_remains_gated() {
    let f = setup(true);
    let _installed = f.install();
    let preview =
        crate::manifest_acquisition::plan_for_agent(f.root.path(), "depth_fixture", "0.2.0")
            .unwrap_err();
    assert!(matches!(
        preview,
        crate::manifest_acquisition::AcquisitionError::NotEnrolled(_)
    ));
    // The same installed input-only writers produced a complete surface without release-watch.
    freeze(&f).unwrap();
    f.commit();
    let governing: toml_edit::DocumentMut =
        include_str!("../../../tests/fixtures/depth-synthetic/governing/registry-entry.toml")
            .replace("{{ISOLATED_OWNER}}", "fixture")
            .replace("{{ISOLATED_REPO}}", "isolated")
            .parse()
            .unwrap();
    let release = toml_edit::Item::Value(toml_edit::Value::InlineTable(
        governing["agents"][0]["maintenance"]["release_watch"]
            .as_table()
            .unwrap()
            .clone()
            .into_inline_table(),
    ));
    f.registry(|d| {
        let entry = d["agents"]
            .as_array_of_tables_mut()
            .unwrap()
            .iter_mut()
            .find(|e| e["agent_id"].as_str() == Some("depth_fixture"))
            .unwrap();
        let mut maintenance = toml_edit::Table::new();
        maintenance["release_watch"] = release.clone();
        entry.insert("maintenance", toml_edit::Item::Table(maintenance));
    });
    edit_approval(&f, |d| {
        let mut maintenance = toml_edit::Table::new();
        maintenance["mode"] = toml_edit::value("release_watch_enrolled");
        maintenance["release_watch"] = release;
        d["descriptor"]["maintenance"] = toml_edit::Item::Table(maintenance);
    });
    f.commit();
    freeze(&f).unwrap();
}
#[test]
fn runtime_follow_on_packet_destination_failure_never_partially_freezes() {
    let f = setup(true);
    let _installed = f.install();
    fs::create_dir_all(f.root.path().join(format!("{RUN}/input-contract.json"))).unwrap();
    refuses_unchanged(&f, "directory");
}
#[test]
fn runtime_follow_on_private_path_refuses_write_mode_and_runtime_pilot() {
    let f = setup(false);
    let _installed = f.install();
    let mut write_args = args();
    write_args.write = true;
    write_args.dry_run = false;
    assert!(
        runtime::isolated_depth_dry_run(f.root.path(), write_args, &mut Vec::new())
            .unwrap_err()
            .to_string()
            .contains("only --dry-run")
    );
    drop(_installed);
    let f = Fixture::new();
    let _installed = f.install();
    let error = runtime::isolated_depth_dry_run(f.root.path(), args(), &mut Vec::new())
        .unwrap_err()
        .to_string();
    assert!(error.contains("only the synthetic subject"), "{error}");
}
