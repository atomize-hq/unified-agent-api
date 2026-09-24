#![allow(dead_code, unused_imports, clippy::enum_variant_names)]

use std::{fs, os::unix::fs::symlink, path::Path};

use serde_json::json;

mod agent_registry {
    pub use xtask::agent_registry::*;
}

// `closeout.rs` is pulled in by path, so its `super::stand_down` needs a sibling here.
mod stand_down {
    pub use xtask::agent_maintenance::stand_down::*;
}

// Likewise `evidence.rs`, whose default fetcher reuses the watcher's hardened `fetch_text`.
mod watch {
    pub use xtask::agent_maintenance::watch::*;
}

mod agent_lifecycle {
    pub use xtask::agent_lifecycle::*;
}

mod prepare_publication {
    pub use xtask::prepare_publication::*;
}

mod capability_publication {
    pub use xtask::capability_publication::*;
}

#[path = "../src/agent_maintenance/finding_signature.rs"]
mod finding_signature;

mod workspace_mutation {
    pub use xtask::workspace_mutation::*;
}

mod approval_artifact {
    pub use xtask::approval_artifact::*;
}
#[path = "../src/capability_projection.rs"]
mod capability_projection;
#[path = "../src/agent_maintenance/closeout.rs"]
mod closeout;
#[path = "../src/agent_maintenance/contract_policy.rs"]
mod contract_policy;
#[path = "../src/agent_maintenance/drift/mod.rs"]
mod drift;
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

#[path = "support/agent_maintenance_closeout_harness.rs"]
mod closeout_harness;
#[path = "support/onboard_agent_harness.rs"]
mod harness;
#[path = "support/agent_maintenance_harness.rs"]
mod maintenance_harness;

#[path = "agent_maintenance_closeout/evidence.rs"]
mod evidence;
#[path = "agent_maintenance_closeout/findings.rs"]
mod findings;
#[path = "agent_maintenance_closeout/live_drift_validation.rs"]
mod live_drift_validation;
#[path = "agent_maintenance_closeout/request_and_schema.rs"]
mod request_and_schema;
#[path = "agent_maintenance_closeout/support_audit_baseline.rs"]
mod support_audit_baseline;
#[path = "agent_maintenance_closeout/write_outputs.rs"]
mod write_outputs;

use closeout::{
    load_linked_closeout, validate_live_drift_report, validate_live_drift_truth,
    write_closeout_outputs,
};
use closeout_harness::{
    automated_maintenance_request_toml, automated_maintenance_request_with_execution_contract_toml,
    closeout_with_deferred, finding_json, maintenance_request_toml,
    maintenance_request_toml_with_refs, valid_closeout_json, valid_closeout_struct,
};
use harness::{fixture_root, sha256_hex, write_text};

fn seed_opencode_basis(root: &Path) {
    maintenance_harness::seed_opencode_basis(root);
    let registry = agent_registry::AgentRegistry::load(root).expect("load registry");
    let entry = registry.find("opencode").expect("opencode registry entry");
    write_text(
        &root.join("cli_manifests/opencode/RULES.json"),
        &serde_json::json!({"union": {"expected_targets": entry.canonical_targets}}).to_string(),
    );
    write_opencode_coverage_reports(root, "1.14.47", false);
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
