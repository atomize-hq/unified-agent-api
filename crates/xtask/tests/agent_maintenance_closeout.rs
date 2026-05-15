#![allow(dead_code, unused_imports, clippy::enum_variant_names)]

use std::{fs, os::unix::fs::symlink, path::Path};

use serde_json::json;

mod agent_registry {
    pub use xtask::agent_registry::*;
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

#[path = "agent_maintenance_closeout/live_drift_validation.rs"]
mod live_drift_validation;
#[path = "agent_maintenance_closeout/request_and_schema.rs"]
mod request_and_schema;
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
