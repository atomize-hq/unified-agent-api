use std::{fs, path::Path};

use serde_json::{json, Value};

use super::{
    audit_status::is_bad_support_audit_evidence_message, request::DetectedRelease, support_audit::*,
};
use crate::agent_registry::AgentRegistry;

// Verbatim from cli_manifests/opencode/reports/1.18.29/coverage.any.json (`deltas.missing_commands[0]`)
// on the 2026-09-14 packet branch automation/opencode-maintenance-1.18.29 (`57b7a7f2`).
const OPENCODE_1_18_29_ROOT_COMMAND_ROW: &str =
    r#"{"path": [], "upstream_available_on": ["linux-x64", "darwin-arm64", "win32-x64"]}"#;

fn identity(kind: &str, command_path: &str, surface_id: &str) -> SurfaceIdentity {
    SurfaceIdentity::new(kind.into(), command_path.into(), surface_id.into())
}

fn deltas(missing_commands: Value, missing_flags: Value, missing_args: Value) -> Value {
    json!({
        "missing_commands": missing_commands,
        "missing_flags": missing_flags,
        "missing_args": missing_args,
        "intentionally_unsupported": [],
    })
}

fn surfaces(agent_id: &str, deltas: &Value) -> Result<Vec<SurfaceIdentity>, String> {
    surfaces_from_report_deltas(
        agent_id,
        Path::new("coverage.any.json"),
        deltas.as_object().expect("deltas object"),
    )
}

fn root_row() -> Value {
    serde_json::from_str(OPENCODE_1_18_29_ROOT_COMMAND_ROW).expect("root row")
}

#[test]
fn a_root_command_row_is_identified_as_the_agent_command() {
    for agent_id in ["opencode", "codex"] {
        let found = surfaces(agent_id, &deltas(json!([root_row()]), json!([]), json!([])))
            .expect("a root command row is valid evidence");

        assert_eq!(found, vec![identity("commands", agent_id, agent_id)]);
    }
}

#[test]
fn root_flags_and_root_args_keep_their_existing_identities() {
    let found = surfaces(
        "opencode",
        &deltas(
            json!([root_row()]),
            json!([{"path": [], "key": "--help"}, {"path": ["run"], "key": "--help"}]),
            json!([{"path": [], "name": "project"}]),
        ),
    )
    .expect("valid rows");

    assert_eq!(
        found,
        vec![
            identity("commands", "opencode", "opencode"),
            identity("flags", "opencode run", "--help"),
            identity("global_flags", "opencode", "--help"),
            identity("positional_args", "opencode", "project"),
        ]
    );
}

#[test]
fn the_root_command_stays_distinct_from_commands_that_share_its_name() {
    let found = surfaces(
        "opencode",
        &deltas(
            json!([root_row(), {"path": ["opencode"]}, {"path": ["tools", "opencode"]}]),
            json!([]),
            json!([]),
        ),
    )
    .expect("valid rows");

    assert_eq!(
        found,
        vec![
            identity("commands", "opencode", "opencode"),
            identity("commands", "opencode opencode", "opencode"),
            identity("subcommands", "opencode tools opencode", "opencode"),
        ]
    );
}

#[test]
fn a_root_command_listed_twice_is_one_surface() {
    let mut report = deltas(json!([root_row()]), json!([]), json!([]));
    report["intentionally_unsupported"] = json!([root_row()]);

    let found = surfaces("opencode", &report).expect("valid rows");

    assert_eq!(found, vec![identity("commands", "opencode", "opencode")]);
}

#[test]
fn malformed_rows_are_rejected_as_bad_report_evidence() {
    let cases = [
        deltas(json!([{"upstream_available_on": []}]), json!([]), json!([])),
        deltas(json!([{"path": null}]), json!([]), json!([])),
        deltas(json!([{"path": "run"}]), json!([]), json!([])),
        deltas(json!([{"path": ["run", 7]}]), json!([]), json!([])),
        deltas(json!(["not an object"]), json!([]), json!([])),
        // A flag row whose key is not a string must not fall back to a root command.
        deltas(json!([]), json!([{"path": [], "key": 17}]), json!([])),
        deltas(json!([]), json!([{"path": [], "key": null}]), json!([])),
        deltas(json!([]), json!([]), json!([{"path": [], "name": false}])),
        deltas(
            json!([]),
            json!([{"path": [], "key": "--x", "name": "x"}]),
            json!([]),
        ),
        // A row whose shape does not match the list it came from.
        deltas(json!([]), json!([{"path": []}]), json!([])),
        deltas(json!([]), json!([]), json!([{"path": [], "key": "--x"}])),
        deltas(
            json!([{"path": [], "name": "project"}]),
            json!([]),
            json!([]),
        ),
    ];

    for report in cases {
        let error = surfaces("opencode", &report).expect_err(&format!("reject {report}"));
        // The gate must classify it as bad evidence (exit 2), not an internal fault.
        assert!(
            is_bad_support_audit_evidence_message(&error),
            "unclassified error `{error}` for {report}"
        );
    }
}

#[test]
fn only_the_intentionally_unsupported_list_may_be_omitted() {
    let mut report = deltas(json!([root_row()]), json!([]), json!([]));
    report
        .as_object_mut()
        .expect("deltas object")
        .remove("intentionally_unsupported");
    assert_eq!(
        surfaces("opencode", &report).expect("the report writer omits an empty list"),
        vec![identity("commands", "opencode", "opencode")]
    );

    for (key, value) in [
        ("missing_commands", None),
        ("missing_flags", None),
        ("missing_args", None),
        ("missing_commands", Some(json!({}))),
        ("intentionally_unsupported", Some(json!(null))),
        ("intentionally_unsupported", Some(json!({}))),
    ] {
        let mut report = deltas(json!([]), json!([]), json!([]));
        let object = report.as_object_mut().expect("deltas object");
        match value {
            Some(value) => object.insert(key.to_string(), value),
            None => object.remove(key),
        };
        let error = surfaces("opencode", &report).expect_err(&format!("reject {report}"));
        assert!(
            error.contains(&format!("`deltas.{key}`"))
                && is_bad_support_audit_evidence_message(&error),
            "unexpected error `{error}` for {report}"
        );
    }
}

#[test]
fn intentionally_unsupported_accepts_every_row_shape() {
    let mut report = deltas(json!([]), json!([]), json!([]));
    report["intentionally_unsupported"] = json!([
        root_row(),
        {"path": ["run"], "key": "--port"},
        {"path": [], "name": "project"},
    ]);

    let found = surfaces("opencode", &report).expect("valid rows");

    assert_eq!(
        found,
        vec![
            identity("commands", "opencode", "opencode"),
            identity("flags", "opencode run", "--port"),
            identity("positional_args", "opencode", "project"),
        ]
    );
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

/// Derives the audit for `agent_id` from the committed registry and debt inventory plus `report`.
fn derive_with_report(agent_id: &str, version: &str, report: &Value) -> SupportSurfaceAudit {
    let registry = AgentRegistry::load(repo_root()).expect("load registry");
    let entry = registry.find(agent_id).expect("registry entry");
    let workspace = tempfile::TempDir::new().expect("workspace");
    let debt = workspace.path().join(NON_TUI_SUPPORT_DEBT_PATH);
    fs::create_dir_all(debt.parent().expect("debt parent")).expect("create debt parent");
    fs::copy(repo_root().join(NON_TUI_SUPPORT_DEBT_PATH), &debt).expect("copy debt inventory");
    let report_dir = workspace
        .path()
        .join(&entry.manifest_root)
        .join("reports")
        .join(version);
    fs::create_dir_all(&report_dir).expect("create report dir");
    fs::write(report_dir.join("coverage.any.json"), report.to_string()).expect("write report");
    let release = DetectedRelease {
        detected_by: String::new(),
        current_validated: String::new(),
        target_version: version.into(),
        latest_stable: String::new(),
        version_policy: String::new(),
        source_kind: String::new(),
        source_ref: String::new(),
        dispatch_kind: String::new(),
        dispatch_workflow: String::new(),
        branch_name: String::new(),
    };
    derive_support_surface_audit(workspace.path(), entry, &release).expect("derive audit")
}

#[test]
fn a_missing_root_command_remains_a_required_uplift() {
    let audit = derive_with_report(
        "opencode",
        "1.18.29",
        &json!({"deltas": deltas(json!([root_row()]), json!([]), json!([]))}),
    );

    let root = identity("commands", "opencode", "opencode");
    let required = audit
        .required_uplifts_this_run
        .iter()
        .map(RequiredUplift::identity)
        .collect::<Vec<_>>();
    assert_eq!(required, vec![root.clone()]);
    assert_eq!(audit.missing_wrapper_support, vec![root.clone()]);
    assert!(audit
        .preexisting_unsupported_surface
        .iter()
        .all(|surface| surface.identity() != root));
}

#[test]
fn every_debt_row_is_rooted_at_its_agent_id() {
    let registry = AgentRegistry::load(repo_root()).expect("load registry");
    for row in load_debt_inventory(repo_root()).expect("load debt inventory") {
        assert!(
            registry.find(&row.agent_id).is_some(),
            "debt row `{}` names unknown agent `{}`",
            row.row_id,
            row.agent_id
        );
        let mut tokens = row.command_path.split(' ');
        assert!(
            tokens.next() == Some(row.agent_id.as_str()) && tokens.all(|token| !token.is_empty()),
            "debt row `{}` command_path `{}` is not `<agent_id>` then single-spaced tokens (agent id `{}`)",
            row.row_id,
            row.command_path,
            row.agent_id
        );
    }
}

#[test]
fn claude_code_install_debt_matches_its_report_surfaces() {
    // Row shapes as the claude_code 2.1.236 report lists them under intentionally_unsupported.
    let mut report = deltas(json!([]), json!([]), json!([]));
    report["intentionally_unsupported"] = json!([
        {"path": ["install"], "upstream_available_on": ["win32-x64"]},
        {"path": ["install"], "key": "--force", "upstream_available_on": ["win32-x64"]},
    ]);

    let audit = derive_with_report("claude_code", "2.1.236", &json!({"deltas": report}));

    let install = vec![
        identity("commands", "claude_code install", "install"),
        identity("flags", "claude_code install", "--force"),
    ];
    let preexisting = audit
        .preexisting_unsupported_surface
        .iter()
        .map(DebtBackedSurface::identity)
        .collect::<Vec<_>>();
    assert_eq!(preexisting, install);
    assert!(audit.required_uplifts_this_run.is_empty());
    assert!(audit.removed_upstream_surface.is_empty());
}
