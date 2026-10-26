use serde_json::{json, Value};
use std::{fs, path::Path, process::Command};
use xtask::support_depth::{plan_freeze, FreezeRequest};

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
fn request() -> FreezeRequest {
    FreezeRequest {
        agent: "codex".into(),
        version: "1.2.3".into(),
        event: xtask::depth_record::Event::Maintenance {
            request_commit: "abc".into(),
            request_recorded_at: "2026-10-25T00:00:00Z".into(),
            trigger_kind: "manual_reopen".into(),
            opened_from: "manual".into(),
        },
    }
}
fn policy(root: &Path, f: impl FnOnce(&mut Value)) {
    let path = root.join("crates/xtask/data/agent_registry.toml");
    let mut registry: Value = toml_edit::de::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let codex = registry["agents"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["agent_id"] == "codex")
        .unwrap();
    f(&mut codex["support_depth"]["enrollments"][0]);
    fs::write(path, toml_edit::ser::to_string_pretty(&registry).unwrap()).unwrap();
    git(root, &["add", "."]);
}
fn acquisition() -> tempfile::TempDir {
    let root = fixture::workspace("codex");
    fixture::acquire(root.path(), "codex");
    fixture::change_json(
        root.path(),
        "cli_manifests/codex/snapshots/1.2.3/x86_64-unknown-linux-musl.json",
        |v| {
            v["commands"][1]["flags"]
                .as_array_mut()
                .unwrap()
                .push(json!({"long":"--json","short":"-j","takes_value":false}));
        },
    );
    fixture::regenerate(root.path(), "codex");
    policy(root.path(), |d| {
        d["modes"][0]["predicates"] = json!([{"flag":"--json"}])
    });
    root
}
#[test]
fn acquired_json_presence_builds_read_only_o_without_erasing_unknown_units() {
    let root = acquisition();
    let before = git(root.path(), &["status", "--porcelain=v1"]);
    let index = fs::read(root.path().join(".git/index")).unwrap();
    let plan = plan_freeze(root.path(), &request()).unwrap().unwrap();
    let o = plan.record.obligations.unwrap();
    assert!(o.operations[0]
        .surfaces
        .iter()
        .any(|s| s.surface_id == "--json"));
    assert_eq!(
        o.classification_required.len(),
        o.operations[0].surfaces.len()
    );
    assert!(plan.record.execution.is_none());
    assert_eq!(git(root.path(), &["status", "--porcelain=v1"]), before);
    assert_eq!(fs::read(root.path().join(".git/index")).unwrap(), index);
}

#[test]
fn classification_rows_are_paired_ordered_and_additive_without_acquisition() {
    let root = fixture::workspace("codex");
    let original = plan_freeze(root.path(), &request())
        .unwrap()
        .unwrap()
        .record
        .policy;
    let old_bytes = serde_json::to_vec(&original).unwrap();
    assert!(!String::from_utf8(old_bytes.clone())
        .unwrap()
        .contains("classification"));
    policy(root.path(), |d| {
        d["operations"][0]["classification_basis"] = json!("a".repeat(64));
        d["operations"][0]["classifications"] = json!([
            {"surface":{"surface_kind":"flags","command_path":"codex run","surface_id":"--output"},
             "templates":["CORE","ADMIN"],"rationale":"Output file mutation is part of this promise."}
        ]);
    });
    let plan = plan_freeze(root.path(), &request()).unwrap().unwrap();
    assert!(plan.record.obligations.is_none());
    let value = serde_json::to_value(&plan.record.policy).unwrap();
    assert_eq!(
        value["selection"]["enrollment"]["operations"][0]["classifications"][0]["surface"]
            ["surface_id"],
        "--output"
    );
    assert!(value["templates"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["template"] == "ADMIN"));
    policy(root.path(), |d| {
        d["operations"][0]
            .as_object_mut()
            .unwrap()
            .remove("classification_basis");
        d["operations"][0]
            .as_object_mut()
            .unwrap()
            .remove("classifications");
    });
    assert_eq!(
        serde_json::to_vec(
            &plan_freeze(root.path(), &request())
                .unwrap()
                .unwrap()
                .record
                .policy
        )
        .unwrap(),
        old_bytes
    );
}

#[path = "support_depth_packet_intake/real.rs"]
mod real;
#[path = "support_depth_packet_intake/synthetic.rs"]
mod synthetic;

fn classify(root: &Path, req: &FreezeRequest, names: &[&str]) -> xtask::support_depth::FreezePlan {
    let plan = plan_freeze(root, req).unwrap().unwrap();
    let basis = &plan.classification_bases[0].sha256;
    let surfaces = &plan.record.obligations.as_ref().unwrap().operations[0].surfaces;
    let rows: Vec<_> = names.iter().map(|name| {
        let surface = surfaces.iter().find(|s| s.surface_id == *name).unwrap();
        json!({"surface": surface, "templates": ["CORE", "ADMIN"], "rationale": "Bound output mutation and streaming boundary."})
    }).collect();
    policy(root, |d| {
        d["operations"][0]["classification_basis"] = json!(basis);
        d["operations"][0]["classifications"] = json!(rows);
    });
    plan_freeze(root, req).unwrap().unwrap()
}
