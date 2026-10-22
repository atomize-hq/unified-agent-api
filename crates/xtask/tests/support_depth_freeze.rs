use std::{fs, path::Path, process::Command};
use xtask::{
    depth_record::Event,
    support_depth::{plan_freeze, FreezeRequest},
};

#[path = "support_depth_freeze/fixture.rs"]
mod fixture;

fn git(root: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn event() -> Event {
    Event::Maintenance {
        request_commit: "abc".into(),
        request_recorded_at: "2026-10-22T00:00:00Z".into(),
        trigger_kind: "manual_reopen".into(),
        opened_from: "manual".into(),
    }
}

fn request() -> FreezeRequest {
    FreezeRequest {
        agent: "codex".into(),
        version: "1.2.3".into(),
        event: event(),
    }
}

#[test]
fn first_freeze_without_acquisition_is_p_only_and_read_only() {
    let root = fixture::workspace("codex");
    let before = git(root.path(), &["status", "--porcelain=v1"]);
    let index = fs::read(root.path().join(".git/index")).unwrap();
    let plan = plan_freeze(root.path(), &request()).unwrap().unwrap();
    assert!(plan.record.obligations.is_none());
    assert!(plan.record.execution.is_none());
    assert_eq!(plan.record.event, event());
    assert!(plan.request.obligations_identity.is_none());
    assert!(plan.record.request_mismatch(&plan.request).is_none());
    assert_eq!(git(root.path(), &["status", "--porcelain=v1"]), before);
    assert_eq!(fs::read(root.path().join(".git/index")).unwrap(), index);
    assert!(!root
        .path()
        .join("cli_manifests/codex/reports/1.2.3/depth-record.json")
        .exists());
}

#[test]
fn adopted_roots_reconcile_acquisition_and_include_all_coverage_levels() {
    for agent in ["codex", "claude_code", "opencode"] {
        let root = fixture::workspace(agent);
        fixture::acquire(root.path(), agent);
        let mut req = request();
        req.agent = agent.into();
        let plan = plan_freeze(root.path(), &req).unwrap().unwrap();
        let o = plan.record.obligations.as_ref().unwrap();
        let surfaces = &o.operations[0].surfaces;
        for name in ["run", "--output", "--claimed", "PROMPT"] {
            assert!(
                surfaces.iter().any(|s| s.surface_id == name),
                "{agent}: {name}"
            );
        }
        assert_eq!(o.classification_required.len(), surfaces.len());
        assert!(plan.record.execution.is_none());
        assert!(plan.record.request_mismatch(&plan.request).is_none());
        let encoded = serde_json::to_string(&plan.record).unwrap();
        assert_eq!(
            xtask::depth_record::DepthRecord::parse(&encoded, agent, "1.2.3").unwrap(),
            plan.record
        );
        assert!(!o
            .acquired_inputs
            .iter()
            .any(|f| f.path.contains("depth-record") || f.path.contains("maintenance-request")));
        fixture::install(root.path(), agent, &plan.record);
        let repeated = plan_freeze(root.path(), &req).unwrap().unwrap();
        assert_eq!(repeated.record, plan.record);
    }
}

#[test]
fn changed_o_clears_execution_while_unchanged_o_preserves_it_and_entries() {
    use xtask::depth_record::{part_identity, AcceptanceEntry, ExecutionPart};
    let root = fixture::workspace("codex");
    fixture::acquire(root.path(), "codex");
    let mut record = plan_freeze(root.path(), &request())
        .unwrap()
        .unwrap()
        .record;
    let mut e = ExecutionPart {
        identity: String::new(),
        policy_identity: record.policy.identity.clone(),
        obligations_identity: record.obligations.as_ref().unwrap().identity.clone(),
        results: vec![],
        reuse_bindings: vec![],
        debt_grants: vec![],
        debt_retired: vec![],
    };
    e.identity = part_identity(&e).unwrap();
    record.execution = Some(e);
    record.acceptance_entries.push(AcceptanceEntry::Promotion {
        record_sha256: "b".repeat(64),
        targets: vec!["x86_64-unknown-linux-musl".into()],
    });
    fixture::install(root.path(), "codex", &record);
    assert_eq!(
        plan_freeze(root.path(), &request())
            .unwrap()
            .unwrap()
            .record,
        record
    );
    fixture::change_json(
        root.path(),
        "cli_manifests/codex/snapshots/1.2.3/x86_64-unknown-linux-musl.json",
        |v| {
            v["commands"][1]["usage"] =
                serde_json::json!("run [--output FILE=changed-default] PROMPT")
        },
    );
    fixture::regenerate(root.path(), "codex");
    let changed = plan_freeze(root.path(), &request())
        .unwrap()
        .unwrap()
        .record;
    assert_eq!(changed.policy, record.policy);
    assert_ne!(changed.obligations, record.obligations);
    assert!(changed.execution.is_none());
    assert_eq!(changed.acceptance_entries, record.acceptance_entries);
}

#[test]
fn malformed_partial_stale_or_untracked_artifacts_refuse_instead_of_p_only() {
    for case in [
        "shard-only",
        "missing-report",
        "stale-report",
        "bad-schema",
        "dirty",
        "untracked",
    ] {
        let root = fixture::workspace("codex");
        fixture::acquire(root.path(), "codex");
        let dir = "cli_manifests/codex/";
        match case {
            "shard-only" => {
                git(
                    root.path(),
                    &["rm", "-f", &format!("{dir}snapshots/1.2.3/union.json")],
                );
            }
            "missing-report" => {
                git(
                    root.path(),
                    &[
                        "rm",
                        "-f",
                        &format!("{dir}reports/1.2.3/coverage.x86_64-unknown-linux-musl.json"),
                    ],
                );
            }
            "stale-report" => {
                fixture::change_json(
                    root.path(),
                    &format!("{dir}reports/1.2.3/coverage.any.json"),
                    |v| v["deltas"]["missing_flags"] = serde_json::json!([]),
                );
                git(root.path(), &["add", "."]);
            }
            "bad-schema" => {
                fixture::change_json(
                    root.path(),
                    &format!("{dir}snapshots/1.2.3/union.json"),
                    |v| v["snapshot_schema_version"] = serde_json::json!(99),
                );
                git(root.path(), &["add", "."]);
            }
            "dirty" => {
                let path = root.path().join(format!("{dir}wrapper_coverage.json"));
                let mut bytes = fs::read(&path).unwrap();
                bytes.push(b'\n');
                fs::write(path, bytes).unwrap();
            }
            "untracked" => {
                git(
                    root.path(),
                    &[
                        "rm",
                        "-f",
                        "--cached",
                        &format!("{dir}reports/1.2.3/coverage.any.json"),
                    ],
                );
            }
            _ => unreachable!(),
        }
        assert!(plan_freeze(root.path(), &request()).is_err(), "{case}");
    }
}

#[test]
fn policy_drift_and_bad_standing_bindings_refuse_re_freeze() {
    for case in [
        "policy",
        "request-p",
        "request-o",
        "unknown-table-key",
        "missing-request",
    ] {
        let root = fixture::workspace("codex");
        fixture::acquire(root.path(), "codex");
        let record = plan_freeze(root.path(), &request())
            .unwrap()
            .unwrap()
            .record;
        fixture::install(root.path(), "codex", &record);
        let request_path = xtask::agent_lifecycle::maintenance_request_path("codex");
        match case {
            "policy" => {
                let p = root.path().join("crates/xtask/data/agent_registry.toml");
                let text = fs::read_to_string(&p)
                    .unwrap()
                    .replace("x1 = \"x\"", "x1 = \"changed\"");
                fs::write(p, text).unwrap();
            }
            "missing-request" => {
                git(root.path(), &["rm", "-f", &request_path]);
            }
            _ => {
                let p = root.path().join(&request_path);
                let mut text = fs::read_to_string(&p).unwrap();
                if case == "request-p" {
                    text = text.replace(&record.policy.identity, &"f".repeat(64));
                }
                if case == "request-o" {
                    text = text.replace(
                        &record.obligations.as_ref().unwrap().identity,
                        &"f".repeat(64),
                    );
                }
                if case == "unknown-table-key" {
                    text = text.replace("[support_depth]", "[support_depth]\nextra = true");
                }
                fs::write(p, text).unwrap();
            }
        }
        git(root.path(), &["add", "."]);
        assert!(plan_freeze(root.path(), &request()).is_err(), "{case}");
    }
}

#[test]
fn retained_due_work_survives_a_new_event_and_disappearance_from_help() {
    use serde_json::json;
    use xtask::depth_record::{part_identity, AcceptanceWork};
    let root = fixture::workspace("codex");
    fixture::acquire(root.path(), "codex");
    let mut record = plan_freeze(root.path(), &request())
        .unwrap()
        .unwrap()
        .record;
    let vanished = xtask::agent_maintenance::support_audit::SurfaceIdentity::new(
        "flags".into(),
        "codex vanished".into(),
        "--old".into(),
    );
    record.obligations.as_mut().unwrap().acceptance_work = vec![AcceptanceWork {
        surface: vanished.clone(),
        targets: vec!["outside-selected-target".into()],
    }];
    let o = record.obligations.as_mut().unwrap();
    o.identity = part_identity(o).unwrap();
    fixture::install(root.path(), "codex", &record);
    let path = xtask::agent_lifecycle::maintenance_request_path("codex");
    let mut doc: toml_edit::DocumentMut = fs::read_to_string(root.path().join(&path))
        .unwrap()
        .parse()
        .unwrap();
    let audit = json!({"required_uplifts_this_run":[{"surface_kind":"flags","command_path":"codex run","surface_id":"--output"}]});
    doc["support_surface_audit"] = toml_edit::Item::Table(
        toml_edit::ser::to_document(&audit)
            .unwrap()
            .as_table()
            .clone(),
    );
    fixture::write(root.path(), &path, doc.to_string());
    git(root.path(), &["add", "."]);
    let mut req = request();
    if let Event::Maintenance {
        request_recorded_at,
        ..
    } = &mut req.event
    {
        *request_recorded_at = "2026-10-23T00:00:00Z".into();
    }
    let plan = plan_freeze(root.path(), &req).unwrap().unwrap();
    let work = &plan.record.obligations.as_ref().unwrap().acceptance_work;
    assert!(work
        .iter()
        .any(|w| w.surface == vanished && w.targets == ["outside-selected-target"]));
    assert!(work
        .iter()
        .any(|w| w.surface.surface_id == "--output" && w.targets == ["x86_64-unknown-linux-musl"]));
    assert!(plan.record.execution.is_none());
    git(
        root.path(),
        &["rm", "-rf", "cli_manifests/codex/snapshots/1.2.3"],
    );
    git(
        root.path(),
        &[
            "rm",
            "-f",
            "cli_manifests/codex/reports/1.2.3/coverage.any.json",
            "cli_manifests/codex/reports/1.2.3/coverage.x86_64-unknown-linux-musl.json",
        ],
    );
    let p_only = plan_freeze(root.path(), &req).unwrap().unwrap();
    assert!(p_only.record.obligations.is_none());
    assert_eq!(
        p_only.carried_work,
        record.obligations.unwrap().acceptance_work
    );
    assert!(p_only.required_surfaces.contains(&vanished));
    assert!(p_only
        .required_surfaces
        .iter()
        .any(|s| s.surface_id == "--output"));
}

#[test]
fn same_event_another_version_and_stale_execution_binding_refuse() {
    use xtask::depth_record::{part_identity, ExecutionPart};
    let root = fixture::workspace("codex");
    fixture::acquire(root.path(), "codex");
    let mut record = plan_freeze(root.path(), &request())
        .unwrap()
        .unwrap()
        .record;
    fixture::install(root.path(), "codex", &record);
    let path = xtask::agent_lifecycle::maintenance_request_path("codex");
    let mut req = fs::read_to_string(root.path().join(&path)).unwrap();
    req = req.replace("1.2.3", "1.2.4");
    fixture::write(root.path(), &path, req);
    git(root.path(), &["add", "."]);
    assert!(plan_freeze(root.path(), &request())
        .unwrap_err()
        .contains("same Event"));
    let mut e = ExecutionPart {
        identity: String::new(),
        policy_identity: record.policy.identity.clone(),
        obligations_identity: "f".repeat(64),
        results: vec![],
        reuse_bindings: vec![],
        debt_grants: vec![],
        debt_retired: vec![],
    };
    e.identity = part_identity(&e).unwrap();
    record.execution = Some(e);
    fixture::install(root.path(), "codex", &record);
    assert!(plan_freeze(root.path(), &request())
        .unwrap_err()
        .contains("prior E"));
}

#[test]
fn unrelated_dirty_files_do_not_change_a_freeze() {
    let root = fixture::workspace("codex");
    fixture::acquire(root.path(), "codex");
    let first = plan_freeze(root.path(), &request()).unwrap().unwrap();
    fixture::write(root.path(), "unrelated.txt", "local work");
    let second = plan_freeze(root.path(), &request()).unwrap().unwrap();
    assert_eq!(first.record, second.record);
    assert_eq!(
        fs::read_to_string(root.path().join("unrelated.txt")).unwrap(),
        "local work"
    );
}

#[test]
fn contradictory_union_aliases_and_wrapper_scopes_refuse() {
    use serde_json::json;
    for case in [
        "union-identity",
        "union-surface",
        "shard-identity",
        "duplicate-command",
        "alias",
        "unknown-scope",
        "overlap",
    ] {
        let root = fixture::workspace("codex");
        fixture::acquire(root.path(), "codex");
        let shard = "cli_manifests/codex/snapshots/1.2.3/x86_64-unknown-linux-musl.json";
        match case {
            "union-identity" => fixture::change_json(
                root.path(),
                "cli_manifests/codex/snapshots/1.2.3/union.json",
                |v| v["tool"] = json!("other-cli"),
            ),
            "union-surface" => fixture::change_json(
                root.path(),
                "cli_manifests/codex/snapshots/1.2.3/union.json",
                |v| {
                    v["commands"].as_array_mut().unwrap().pop();
                },
            ),
            "shard-identity" => fixture::change_json(root.path(), shard, |v| {
                v["binary"]["semantic_version"] = json!("1.2.4")
            }),
            "duplicate-command" => {
                fixture::change_json(root.path(), shard, |v| {
                    let c = v["commands"][1].clone();
                    v["commands"].as_array_mut().unwrap().push(c);
                });
                fixture::regenerate(root.path(), "codex");
            }
            "alias" => {
                fixture::change_json(root.path(), shard, |v| {
                    v["commands"][1]["flags"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!({"long":"--other","short":"-o","takes_value":false}))
                });
                fixture::regenerate(root.path(), "codex");
            }
            "unknown-scope" => fixture::change_json(
                root.path(),
                "cli_manifests/codex/wrapper_coverage.json",
                |v| v["coverage"][0]["scope"] = json!({"target_triples":["not-authorized"]}),
            ),
            "overlap" => fixture::change_json(
                root.path(),
                "cli_manifests/codex/wrapper_coverage.json",
                |v| {
                    let c = v["coverage"][0].clone();
                    v["coverage"].as_array_mut().unwrap().push(c);
                },
            ),
            _ => unreachable!(),
        }
        git(root.path(), &["add", "."]);
        assert!(plan_freeze(root.path(), &request()).is_err(), "{case}");
    }
}

#[cfg(unix)]
#[test]
fn symlink_files_and_parent_directories_refuse() {
    use std::os::unix::fs::symlink;
    for parent in [false, true] {
        let root = fixture::workspace("codex");
        fixture::acquire(root.path(), "codex");
        if parent {
            fs::rename(
                root.path().join("cli_manifests/codex"),
                root.path().join("actual"),
            )
            .unwrap();
            symlink("../actual", root.path().join("cli_manifests/codex")).unwrap();
        } else {
            let p = root
                .path()
                .join("cli_manifests/codex/wrapper_coverage.json");
            fs::remove_file(&p).unwrap();
            symlink("SCHEMA.json", &p).unwrap();
            git(root.path(), &["add", "."]);
        }
        assert!(plan_freeze(root.path(), &request())
            .unwrap_err()
            .contains("symlink"));
    }
}

#[test]
fn acquisition_extends_p_only_without_changing_event_or_policy() {
    let root = fixture::workspace("codex");
    let first = plan_freeze(root.path(), &request()).unwrap().unwrap();
    fixture::install(root.path(), "codex", &first.record);
    fixture::acquire(root.path(), "codex");
    let acquired = plan_freeze(root.path(), &request()).unwrap().unwrap();
    assert_eq!(acquired.record.policy, first.record.policy);
    assert_eq!(acquired.record.event, first.record.event);
    assert!(acquired.record.obligations.is_some());
    assert!(acquired.record.execution.is_none());
}

#[test]
fn selected_target_without_observation_and_external_schema_inputs_refuse() {
    for case in ["target", "schema", "widened-union"] {
        let root = fixture::workspace("opencode");
        fixture::acquire(root.path(), "opencode");
        match case {
            "target" => {
                let path = root.path().join("crates/xtask/data/agent_registry.toml");
                let raw = fs::read_to_string(&path)
                    .unwrap()
                    .replace("targets = [\"linux-x64\"]", "targets = [\"win32-x64\"]");
                fs::write(path, raw).unwrap();
            }
            "schema" => {
                fixture::change_json(root.path(), "cli_manifests/opencode/SCHEMA.json", |v| {
                    v["$defs"]["UpstreamSnapshotV1"]["$ref"] =
                        serde_json::json!("https://example.invalid/policy.json")
                })
            }
            "widened-union" => fixture::change_json(
                root.path(),
                "cli_manifests/opencode/snapshots/1.2.3/union.json",
                |v| {
                    v["expected_targets"]
                        .as_array_mut()
                        .unwrap()
                        .push(serde_json::json!("current-only-target"))
                },
            ),
            _ => unreachable!(),
        }
        git(root.path(), &["add", "."]);
        assert!(
            plan_freeze(root.path(), &request_for("opencode")).is_err(),
            "{case}"
        );
    }
}

fn request_for(agent: &str) -> FreezeRequest {
    let mut req = request();
    req.agent = agent.into();
    req
}

#[path = "support_depth_freeze/semantics.rs"]
mod semantics;
