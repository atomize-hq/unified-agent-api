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
    seed_publication_inputs,
};
use request::{load_request, load_request_envelope};
