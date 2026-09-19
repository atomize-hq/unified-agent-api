#![allow(dead_code, unused_imports, clippy::enum_variant_names)]

use std::{collections::BTreeSet, fs, path::Path};

#[path = "support/onboard_agent_harness.rs"]
mod harness;

mod agent_registry {
    pub use xtask::agent_registry::*;
}
mod capability_publication {
    pub use xtask::capability_publication::*;
}
mod publication_refresh {
    pub use xtask::publication_refresh::*;
}
#[path = "../src/capability_matrix.rs"]
mod capability_matrix;
#[path = "../src/capability_projection.rs"]
mod capability_projection;
#[path = "../src/agent_maintenance/contract_policy.rs"]
mod contract_policy;
#[path = "../src/agent_maintenance/docs.rs"]
mod docs;
#[path = "../src/agent_maintenance/refresh.rs"]
mod refresh;
#[path = "../src/release_doc.rs"]
mod release_doc;
#[path = "../src/agent_maintenance/request.rs"]
mod request;
#[path = "../src/root_intake_layout.rs"]
mod root_intake_layout;
#[path = "../src/agent_maintenance/support_audit.rs"]
mod support_audit;
#[path = "../src/support_matrix.rs"]
mod support_matrix;
#[path = "../src/workspace_mutation.rs"]
mod workspace_mutation;

#[path = "support/agent_maintenance_refresh_harness.rs"]
mod refresh_harness;

#[path = "agent_maintenance_refresh/automated_requests.rs"]
mod automated_requests;
#[path = "agent_maintenance_refresh/planning_and_apply.rs"]
mod planning_and_apply;
#[path = "agent_maintenance_refresh/request_validation.rs"]
mod request_validation;

use harness::{fixture_root, seed_release_touchpoints, snapshot_files, write_text};
use refresh::{apply_refresh_plan, build_refresh_plan};
use refresh_harness::{
    automated_request_toml, automated_request_with_execution_contract_toml, diff_paths,
    normalize_support_matrix_fixture, planned_utf8, request_toml, request_toml_with_refs,
};
use request::{load_request, load_request_envelope};

fn seed_publication_inputs(root: &Path) {
    refresh_harness::seed_publication_inputs(root);
    let registry = agent_registry::AgentRegistry::load(root).expect("load registry");
    let entry = registry.find("opencode").expect("opencode registry entry");
    write_text(
        &root.join("cli_manifests/opencode/RULES.json"),
        &serde_json::json!({"union": {"expected_targets": entry.canonical_targets}}).to_string(),
    );
    write_opencode_coverage_reports(root, "0.98.0", false);
    write_text(
        &root.join("docs/specs/unified-agent-api/non-tui-support-debt.md"),
        "# Non-TUI Support Debt Inventory\n\n### `support-debt-authorization-contract-target-version-v1`\n\n## Inventory\n",
    );
}

fn write_opencode_coverage_reports(root: &Path, version: &str, missing_status: bool) {
    let registry = agent_registry::AgentRegistry::load(root).expect("load registry");
    let targets = &registry
        .find("opencode")
        .expect("opencode registry entry")
        .canonical_targets;
    let missing_commands = if missing_status {
        serde_json::json!([{
            "path": ["status"],
            "upstream_available_on": targets
        }])
    } else {
        serde_json::json!([])
    };
    let reports = root.join("cli_manifests/opencode/reports").join(version);
    let mut report = serde_json::json!({
        "inputs": {"upstream": {"semantic_version": version, "targets": targets}},
        "platform_filter": {"mode": "any"},
        "deltas": {
            "missing_commands": missing_commands,
            "missing_flags": [],
            "missing_args": [],
            "intentionally_unsupported": []
        }
    });
    write_text(&reports.join("coverage.any.json"), &report.to_string());
    for target in targets {
        report["inputs"]["upstream"]["targets"] = serde_json::json!([target]);
        report["platform_filter"] =
            serde_json::json!({"mode": "exact_target", "target_triple": target});
        if missing_status {
            report["deltas"]["missing_commands"][0]["upstream_available_on"] =
                serde_json::json!([target]);
        }
        write_text(
            &reports.join(format!("coverage.{target}.json")),
            &report.to_string(),
        );
    }
}
