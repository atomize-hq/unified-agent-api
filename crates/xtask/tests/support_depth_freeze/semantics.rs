use super::{fixture, git, plan_freeze, request};
use serde_json::json;

#[test]
fn manifest_invalid_passthrough_notes_refuse_even_with_fresh_reports() {
    for kind in ["command", "flag", "arg"] {
        let root = fixture::workspace("codex");
        fixture::acquire(root.path(), "codex");
        fixture::change_json(
            root.path(),
            "cli_manifests/codex/wrapper_coverage.json",
            |v| {
                let row = match kind {
                    "command" => &mut v["coverage"][0],
                    "flag" => &mut v["coverage"][0]["flags"][1],
                    _ => &mut v["coverage"][0]["args"][0],
                };
                row["level"] = json!("passthrough");
                row.as_object_mut().unwrap().remove("note");
            },
        );
        fixture::regenerate(root.path(), "codex");
        let error = plan_freeze(root.path(), &request()).unwrap_err();
        assert!(
            error.contains("PASSTHROUGH_NOTE_MISSING"),
            "{kind}: {error}"
        );
    }
}

#[test]
fn excluded_command_flag_and_argument_claims_refuse_even_with_fresh_reports() {
    for kind in ["command", "flag", "arg"] {
        let root = fixture::workspace("codex");
        fixture::acquire(root.path(), "codex");
        fixture::change_json(root.path(), "cli_manifests/codex/RULES.json", |v| {
            let mut exclusion = json!({"unit":kind,"path":["run"],"category":"tui_only","note":"isolated excluded mode fixture"});
            if kind == "flag" {
                exclusion["key"] = json!("--output");
            }
            if kind == "arg" {
                exclusion["name"] = json!("PROMPT");
            }
            v["parity_exclusions"]["units"]
                .as_array_mut()
                .unwrap()
                .push(exclusion);
        });
        fixture::regenerate(root.path(), "codex");
        git(root.path(), &["add", "."]);
        let error = plan_freeze(root.path(), &request()).unwrap_err();
        assert!(
            error.contains("WRAPPER_COVERAGE_INCLUDES_EXCLUDED"),
            "{kind}: {error}"
        );
    }
}
