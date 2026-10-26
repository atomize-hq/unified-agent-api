use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::Path, process::Command};
use xtask::support_depth::{
    codex_pilot::{decode_route, readiness, PilotTransport},
    plan_freeze, FreezeRequest,
};

#[allow(dead_code)]
#[path = "support_depth_freeze/fixture.rs"]
mod fixture;

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}
fn request() -> FreezeRequest {
    FreezeRequest {
        agent: "codex".into(),
        version: "0.156.1".into(),
        event: xtask::depth_record::Event::Maintenance {
            request_commit: "diagnostic".into(),
            request_recorded_at: "2026-10-26T00:00:00Z".into(),
            trigger_kind: "manual_reopen".into(),
            opened_from: "manual".into(),
        },
    }
}

fn declaration() -> Value {
    let modes: Vec<_> = [
        ("initial-json", "codex exec", "adapter-initial-json"),
        (
            "resume-last-json",
            "codex exec resume",
            "adapter-resume-last-json",
        ),
        (
            "resume-id-json",
            "codex exec resume",
            "adapter-resume-id-json",
        ),
        (
            "fork-last-stdio",
            "codex app-server",
            "adapter-fork-last-stdio",
        ),
        ("fork-id-stdio", "codex app-server", "adapter-fork-id-stdio"),
    ]
    .into_iter()
    .map(|(selector, path, mode)| json!({"selector":selector,"command_path":path,"mode":mode}))
    .collect();
    let operations: Vec<_> = [
        ("run-initial", vec!["initial-json"], vec!["CORE","ADMIN","MANAGED"]),
        ("run-resume", vec!["resume-last-json","resume-id-json"], vec!["CORE","SESSION","ADMIN","MANAGED"]),
        ("run-fork", vec!["fork-last-stdio","fork-id-stdio"], vec!["CORE","SESSION","MANAGED","ADMIN"]),
    ].into_iter().map(|(id,serves,templates)| json!({"id":id,"serves":serves,"templates":templates,
        "shared_mapping":"mapped","forwarding":false,"promise": {"r1":"Typed adapter request", "v1":"Native values and validation", "g1":"Exact route grammar", "e1":"Native error mapping", "a1":"Pinned target binary", "x1":"Private controlled roots", "x2":"Explicit defaults", "x3":"Owned lifecycle", "x4":"Native safety policy"}
        })).collect();
    json!({"lifecycle_path":"maintenance","version":"0.156.1","targets":["x86_64-unknown-linux-musl"],
        "modes":modes,"operations":operations,"capabilities":[{"id":"agent_api.run","operations":["run-initial","run-resume","run-fork"]}]})
}

fn policy(root: &Path, update: impl FnOnce(&mut Value)) {
    let path = root.join("crates/xtask/data/agent_registry.toml");
    let mut registry: Value = toml_edit::de::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let entry = registry["agents"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|a| a["agent_id"] == "codex")
        .unwrap();
    update(&mut entry["support_depth"]["enrollments"][0]);
    fs::write(path, toml_edit::ser::to_string_pretty(&registry).unwrap()).unwrap();
    git(root, &["add", "."]);
}
fn workspace() -> tempfile::TempDir {
    let root = fixture::workspace("codex");
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let files = git(&repo, &["ls-files"]);
    for path in files.lines().filter(|p| {
        p.starts_with("crates/codex/src/")
            || p.starts_with("crates/agent_api/src/")
            || p.starts_with("crates/wrapper_events/src/")
            || p.starts_with("crates/xtask/src/")
            || (p.starts_with("crates/xtask/src/") && !p["crates/xtask/src/".len()..].contains('/'))
            || p.starts_with("docs/specs/")
            || *p == "scripts/acquire_codex_pilot_protocol.py"
            || p.ends_with("Cargo.toml")
            || *p == "Cargo.lock"
            || p.starts_with("cli_manifests/codex/snapshots/0.156.1/")
            || p.starts_with("cli_manifests/codex/reports/0.156.1/")
            || matches!(
                *p,
                "cli_manifests/codex/wrapper_coverage.json" | "cli_manifests/codex/SCHEMA.json"
            )
    }) {
        fixture::write(root.path(), path, fs::read(repo.join(path)).unwrap());
    }
    let standing = xtask::agent_lifecycle::maintenance_request_path("codex");
    fixture::write(
        root.path(),
        &standing,
        fs::read(repo.join(&standing)).unwrap(),
    );
    policy(root.path(), |d| *d = declaration());
    root
}

#[test]
fn five_selectors_have_exact_transport_branches_and_preserve_id_bytes() {
    for (key, session, selector) in [
        (None, None, "initial-json"),
        (
            Some("agent_api.session.resume.v1"),
            Some(json!({"selector":"last"})),
            "resume-last-json",
        ),
        (
            Some("agent_api.session.resume.v1"),
            Some(json!({"selector":"id","id":"  original  "})),
            "resume-id-json",
        ),
        (
            Some("agent_api.session.fork.v1"),
            Some(json!({"selector":"last"})),
            "fork-last-stdio",
        ),
        (
            Some("agent_api.session.fork.v1"),
            Some(json!({"selector":"id","id":"  original  "})),
            "fork-id-stdio",
        ),
    ] {
        for (interactive, external) in [(false, false), (true, false), (false, true)] {
            let mut ext = BTreeMap::new();
            if let (Some(key), Some(session)) = (key, session.clone()) {
                ext.insert(key.into(), session);
            }
            ext.insert("agent_api.exec.non_interactive".into(), json!(!interactive));
            if external {
                ext.insert("agent_api.exec.external_sandbox.v1".into(), json!(true));
            }
            let route = decode_route(&ext, external).unwrap();
            assert_eq!(route.selector, selector);
            assert_eq!(
                route.transport,
                if interactive || selector.starts_with("fork") {
                    PilotTransport::AppServer
                } else {
                    PilotTransport::Exec
                }
            );
            assert_eq!(
                route.session_id.as_deref(),
                selector.contains("-id-").then_some("  original  ")
            );
        }
    }
}

#[test]
fn malformed_or_conflicting_context_never_falls_back_to_initial() {
    for value in [
        json!(false),
        json!(null),
        json!({}),
        json!({"selector":"last","id":null}),
        json!({"selector":"last","extra":true}),
        json!({"selector":"id","id":" "}),
        json!({"selector":"id","id":"ok","extra":true}),
    ] {
        let ext = BTreeMap::from([("agent_api.session.resume.v1".into(), value)]);
        assert!(decode_route(&ext, false).is_err());
    }
    for ext in [
        json!({"agent_api.session.resume.v1":{"selector":"last"},"agent_api.session.fork.v1":{"selector":"last"}}),
        json!({"agent_api.exec.non_interactive":null}),
        json!({"agent_api.exec.non_interactive":false,"agent_api.exec.external_sandbox.v1":true}),
        json!({"backend.codex.exec.approval_policy":"on-request"}),
        json!({"backend.codex.exec.sandbox_mode":"unknown"}),
        json!({"backend.codex.exec.unknown":true}),
    ] {
        assert!(decode_route(&serde_json::from_value(ext).unwrap(), true).is_err());
    }
}

#[test]
fn native_policy_and_sandbox_variants_keep_the_managed_route() {
    for policy in [
        json!("never"),
        json!("on-request"),
        json!("on-failure"),
        json!("untrusted"),
        json!({"granular":{"sandbox_approval":false,"rules":true,"mcp_elicitations":false}}),
        json!({"granular":{"sandbox_approval":true,"rules":true,"mcp_elicitations":true,
            "skill_approval":false,"request_permissions":true}}),
    ] {
        for sandbox in ["read-only", "workspace-write", "danger-full-access"] {
            let extensions = serde_json::from_value(json!({
                "agent_api.exec.non_interactive":false,
                "backend.codex.exec.approval_policy":policy,
                "backend.codex.exec.sandbox_mode":sandbox
            }))
            .unwrap();
            assert_eq!(
                decode_route(&extensions, false).unwrap().transport,
                PilotTransport::AppServer
            );
        }
    }
}

#[test]
fn real_acquisition_binds_all_routes_ancestry_protocol_source_and_carried_work() {
    let root = workspace();
    let before = git(root.path(), &["status", "--porcelain=v1"]);
    let index = fs::read(root.path().join(".git/index")).unwrap();
    let preview = readiness(root.path(), &request()).unwrap().unwrap();
    let o = preview.plan.record.obligations.as_ref().unwrap();
    assert_eq!(o.operations.len(), 3);
    for id in ["run-initial", "run-resume"] {
        assert!(o
            .operations
            .iter()
            .find(|op| op.id == id)
            .unwrap()
            .surfaces
            .iter()
            .any(|s| s.command_path == "codex app-server" && s.surface_id == "--listen"));
    }
    // M is supplied by the complete capability mapping, once per operation.
    assert_eq!(
        o.operations
            .iter()
            .map(|op| op.obligations.len())
            .sum::<usize>()
            + 3,
        53
    );
    assert_eq!(preview.plan.classification_bases.len(), 3);
    assert_eq!(o.acceptance_work.len(), 56);
    assert!(o
        .acquired_inputs
        .iter()
        .any(|r| r.path.ends_with("pilot-protocol-facts.json")));
    assert!(o.acquired_inputs.len() > 14);
    assert_eq!(
        o.classification_required.len(),
        o.operations
            .iter()
            .map(|op| op.surfaces.len())
            .sum::<usize>()
    );
    let resume = o
        .operations
        .iter()
        .find(|op| op.id == "run-resume")
        .unwrap();
    assert!(resume
        .surfaces
        .iter()
        .any(|s| s.command_path == "codex exec" && s.surface_id == "--output-last-message"));
    assert!(resume
        .surfaces
        .iter()
        .any(|s| s.command_path == "codex exec resume" && s.surface_id == "--last"));
    let fork = o.operations.iter().find(|op| op.id == "run-fork").unwrap();
    assert!(!fork.surfaces.iter().any(|s| s.command_path == "codex exec"));
    assert!(preview
        .remaining
        .iter()
        .any(|s| s.contains("runtime observation")));
    assert!(preview
        .plan
        .classification_bases
        .iter()
        .all(
            |b| b.facts["pilot"]["context_observation"] == "not_acquired"
                && b.facts["pilot"]["unbound_object_feature_defaults"] == json!(["tool_registry"])
        ));
    assert!(preview.plan.record.execution.is_none());
    assert!(preview.plan.record.acceptance_entries.is_empty());
    // The ordinary planner has no permission to use this finite diagnostic scope.
    assert!(plan_freeze(root.path(), &request()).is_err());
    assert_eq!(git(root.path(), &["status", "--porcelain=v1"]), before);
    assert_eq!(fs::read(root.path().join(".git/index")).unwrap(), index);
    let bases: Vec<_> = preview
        .plan
        .classification_bases
        .iter()
        .map(|b| b.sha256.clone())
        .collect();
    let source = "crates/agent_api/src/backends/codex/fork/native_config.rs";
    let path = root.path().join(source);
    let bytes = fs::read_to_string(&path).unwrap();
    fs::write(
        &path,
        format!("{bytes}\n// Changed effective configuration reader\n"),
    )
    .unwrap();
    git(root.path(), &["add", source]);
    let changed = readiness(root.path(), &request()).unwrap().unwrap();
    assert!(changed
        .plan
        .classification_bases
        .iter()
        .zip(&bases)
        .all(|(a, b)| a.sha256 != *b));
    // Installing the old diagnostic digest as a classification basis cannot survive source drift.
    policy(root.path(), |d| {
        d["operations"][0]["classification_basis"] = json!(bases[0]);
        d["operations"][0]["classifications"] = json!([]);
    });
    assert!(readiness(root.path(), &request())
        .unwrap_err()
        .contains("stale classification_basis"));
}

#[test]
fn protocol_gaps_and_incomplete_mapping_refuse() {
    let root = workspace();
    let path = "cli_manifests/codex/reports/0.156.1/pilot-protocol-facts.json";
    fixture::write(root.path(), path, "{}");
    git(root.path(), &["add", path]);
    assert!(readiness(root.path(), &request())
        .unwrap_err()
        .contains("protocol/config acquisition"));
    policy(root.path(), |d| {
        d["modes"].as_array_mut().unwrap().pop();
    });
    assert!(readiness(root.path(), &request()).is_err());
}

#[test]
fn supplemental_classification_rows_roundtrip_only_in_complete_pilot_scope() {
    let root = workspace();
    let first = readiness(root.path(), &request()).unwrap().unwrap();
    let o = first.plan.record.obligations.as_ref().unwrap();
    for (id, path, leaf) in [
        ("run-resume", "codex exec", "--output-last-message"),
        ("run-resume", "codex app-server", "--listen"),
        ("run-initial", "codex app-server", "--listen"),
    ] {
        let unit = o
            .operations
            .iter()
            .find(|op| op.id == id)
            .unwrap()
            .surfaces
            .iter()
            .find(|s| s.command_path == path && s.surface_id == leaf);
        // Construct exact acquired identity even on the broken diagnostic that drops app-server.
        let surface = unit
            .map(|s| serde_json::to_value(s).unwrap())
            .unwrap_or_else(
                || json!({"surface_kind":"flags","command_path":path,"surface_id":leaf}),
            );
        let basis = &first
            .plan
            .classification_bases
            .iter()
            .find(|b| b.operation == id)
            .unwrap()
            .sha256;
        policy(root.path(), |d| {
            let op = d["operations"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|op| op["id"] == id)
                .unwrap();
            op["classification_basis"] = json!(basis);
            let rows = op
                .as_object_mut()
                .unwrap()
                .entry("classifications")
                .or_insert(json!([]));
            rows.as_array_mut().unwrap().push(json!({"surface":surface,"templates":["CORE","ADMIN"],
                "rationale":"Complete bound pilot effect regime, subject to independent evidence."}));
        });
    }
    let next = readiness(root.path(), &request()).unwrap().unwrap();
    assert_eq!(
        next.plan
            .record
            .obligations
            .as_ref()
            .unwrap()
            .classification_required
            .len(),
        o.classification_required.len() - 3
    );
    assert!(next.plan.record.execution.is_none());
    policy(root.path(), |d| {
        d["version"] = json!("0.156.2");
    });
    assert!(readiness(root.path(), &request()).is_err());
}

#[test]
fn nested_acquisition_source_content_and_mode_drift_invalidate_every_basis() {
    let root = workspace();
    let before = readiness(root.path(), &request()).unwrap().unwrap();
    let path = "crates/xtask/src/manifest_union/intake.rs";
    let file = root.path().join(path);
    let bytes = fs::read_to_string(&file).unwrap();
    fs::write(
        &file,
        format!("{bytes}\n// New acquired alias interpretation\n"),
    )
    .unwrap();
    git(root.path(), &["add", path]);
    let content = readiness(root.path(), &request()).unwrap().unwrap();
    assert!(content
        .plan
        .classification_bases
        .iter()
        .zip(&before.plan.classification_bases)
        .all(|(a, b)| a.sha256 != b.sha256));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&file, fs::Permissions::from_mode(0o755)).unwrap();
        git(root.path(), &["add", path]);
        let mode = readiness(root.path(), &request()).unwrap().unwrap();
        assert!(mode
            .plan
            .classification_bases
            .iter()
            .zip(&content.plan.classification_bases)
            .all(|(a, b)| a.sha256 != b.sha256));
    }
}
