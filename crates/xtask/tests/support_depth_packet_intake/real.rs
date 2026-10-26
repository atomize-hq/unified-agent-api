use super::*;
use std::collections::BTreeSet;

#[test]
fn actual_codex_packet_retains_54_units_with_30_approved_and_24_unresolved() {
    let root = fixture::workspace("codex");
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for directory in ["snapshots/0.156.1", "reports/0.156.1"] {
        let directory = format!("cli_manifests/codex/{directory}");
        for entry in fs::read_dir(repo.join(&directory)).unwrap() {
            let entry = entry.unwrap();
            fixture::write(
                root.path(),
                &format!("{directory}/{}", entry.file_name().to_str().unwrap()),
                fs::read(entry.path()).unwrap(),
            );
        }
    }
    for name in ["wrapper_coverage.json", "SCHEMA.json"] {
        let path = format!("cli_manifests/codex/{name}");
        fixture::write(root.path(), &path, fs::read(repo.join(&path)).unwrap());
    }
    let request_path = xtask::agent_lifecycle::maintenance_request_path("codex");
    fixture::write(
        root.path(),
        &request_path,
        fs::read(repo.join(&request_path)).unwrap(),
    );
    policy(root.path(), |d| {
        d["version"] = json!("0.156.1");
        d["modes"][0]["command_path"] = json!("codex exec");
        d["modes"][0]["predicates"] = json!([{"flag":"--json"}]);
    });
    let mut req = request();
    req.version = "0.156.1".into();
    let unclassified = plan_freeze(root.path(), &req).unwrap().unwrap();
    assert_eq!(
        unclassified.record.obligations.as_ref().unwrap().operations[0]
            .surfaces
            .len(),
        54
    );
    assert_eq!(
        unclassified
            .record
            .obligations
            .as_ref()
            .unwrap()
            .acquired_inputs
            .len(),
        14
    );
    // Independent exact expected identities from the approved appendix; no name blanket/default.
    let local = [
        "--add-dir",
        "--approve-for-me",
        "--cd",
        "--color",
        "--dangerously-bypass-approvals-and-sandbox",
        "--ephemeral",
        "--ignore-rules",
        "--ignore-user-config",
        "--image",
        "--json",
        "--model",
        "--output-last-message",
        "--output-schema",
        "--sandbox",
        "--skip-git-repo-check",
        "--strict-config",
        "--worktree",
    ];
    let global = [
        "--add-dir",
        "--approve-for-me",
        "--cd",
        "--dangerously-bypass-approvals-and-sandbox",
        "--image",
        "--model",
        "--remote",
        "--remote-auth-token-env",
        "--sandbox",
        "--strict-config",
        "--worktree",
    ];
    let make = |kind: &str, path: &str, id: &str| json!({"surface_kind":kind,"command_path":path,"surface_id":id});
    let expected: Vec<_> = std::iter::once(make("commands", "codex exec", "exec"))
        .chain(local.iter().map(|id| make("flags", "codex exec", id)))
        .chain(global.iter().map(|id| make("global_flags", "codex", id)))
        .chain(std::iter::once(make(
            "positional_args",
            "codex exec",
            "PROMPT",
        )))
        .collect();
    assert_eq!(expected.len(), 30);
    // Keep the approved effect rationale, including denied/no-effect remote intent.
    let appendix = include_str!(
        "../../../../docs/agents/lifecycle/support-depth-packet-intake-classifications.md"
    );
    let rows: Vec<_> = expected
        .iter()
        .map(|surface| {
            let line = appendix
                .lines()
                .find(|line| {
                    let cells: Vec<_> = line.split('|').map(str::trim).collect();
                    cells.len() > 8
                        && cells[2] == format!("`{}`", surface["surface_kind"].as_str().unwrap())
                        && cells[3] == format!("`{}`", surface["command_path"].as_str().unwrap())
                        && cells[4] == format!("`{}`", surface["surface_id"].as_str().unwrap())
                })
                .unwrap();
            let cells: Vec<_> = line.split('|').map(str::trim).collect();
            let templates = if cells[7] == "**CORE+ADMIN**" {
                json!(["CORE", "ADMIN"])
            } else {
                assert!(matches!(cells[7], "**CORE**" | "**CORE-denial**"));
                json!(["CORE"])
            };
            json!({"surface":surface,"templates":templates,"rationale":cells[8]})
        })
        .collect();
    policy(root.path(), |d| {
        d["operations"][0]["classification_basis"] =
            json!(unclassified.classification_bases[0].sha256);
        d["operations"][0]["classifications"] = json!(rows);
    });
    let before = git(root.path(), &["status", "--porcelain=v1"]);
    let plan = plan_freeze(root.path(), &req).unwrap().unwrap();
    let o = plan.record.obligations.as_ref().unwrap();
    assert_eq!(o.operations[0].surfaces.len(), 54);
    assert_eq!(plan.required_surfaces.len(), 56);
    assert_eq!(o.acceptance_work.len(), 56);
    assert_eq!(o.classification_required.len(), 24);
    let approved: BTreeSet<_> = expected.iter().map(Value::to_string).collect();
    let resolved: BTreeSet<_> = o.operations[0]
        .surfaces
        .iter()
        .filter(|surface| {
            !o.classification_required
                .iter()
                .any(|r| r.surface == **surface)
        })
        .map(|s| serde_json::to_value(s).unwrap().to_string())
        .collect();
    assert_eq!(resolved, approved);
    let unknown_local = [
        "--config",
        "--dangerously-bypass-hook-trust",
        "--disable",
        "--enable",
        "--help",
        "--local-provider",
        "--oss",
        "--profile",
        "--thread-source",
        "--version",
    ];
    let unknown_global = [
        "--ask-for-approval",
        "--config",
        "--dangerously-bypass-hook-trust",
        "--disable",
        "--enable",
        "--full-auto",
        "--help",
        "--local-provider",
        "--no-alt-screen",
        "--no-daemon",
        "--oss",
        "--profile",
        "--search",
        "--version",
    ];
    let unknown: BTreeSet<_> = unknown_local
        .iter()
        .map(|id| make("flags", "codex exec", id).to_string())
        .chain(
            unknown_global
                .iter()
                .map(|id| make("global_flags", "codex", id).to_string()),
        )
        .collect();
    assert_eq!(unknown.len(), 24);
    assert_eq!(
        o.classification_required
            .iter()
            .map(|r| serde_json::to_value(&r.surface).unwrap().to_string())
            .collect::<BTreeSet<_>>(),
        unknown
    );
    for name in [
        "--full-auto",
        "--no-alt-screen",
        "--help",
        "--version",
        "--config",
        "--enable",
        "--disable",
    ] {
        assert!(o
            .classification_required
            .iter()
            .any(|s| s.surface.surface_id == name));
    }
    assert_eq!(
        plan.record.policy.selection.enrollment.operations[0]
            .classifications
            .as_ref()
            .unwrap()
            .iter()
            .filter(|r| r.templates.len() == 2)
            .count(),
        3
    );
    assert!(plan.record.execution.is_none());
    assert!(plan.record.acceptance_entries.is_empty());
    assert_eq!(git(root.path(), &["status", "--porcelain=v1"]), before);
}
