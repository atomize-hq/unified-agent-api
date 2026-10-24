#![forbid(unsafe_code)]

extern crate self as xtask;

pub mod agent_lifecycle;
pub mod agent_maintenance;
pub mod agent_registry;
pub mod approval_artifact;
pub mod capability_matrix;
pub mod capability_projection;
pub mod capability_publication;
pub mod depth_evidence;
pub mod depth_record;
mod git_inputs;
pub mod manifest_acquisition;
pub mod manifest_report;
pub mod manifest_snapshot_schema;
pub mod manifest_union;
pub mod onboard_agent;
pub mod prepare_proving_run_closeout;
pub mod prepare_publication;
pub mod proving_run_closeout;
pub mod publication_refresh;
pub mod recommend_next_agent_research;
pub mod release_doc;
pub mod repair_runtime_evidence;
mod root_intake_layout;
pub mod runtime_advertising;
pub mod runtime_evidence_bundle;
pub mod runtime_evidence_run;
pub mod runtime_follow_on;
pub mod support_depth;
pub mod support_matrix;
pub mod workspace_mutation;
pub mod wrapper_scaffold;

pub mod depth_acceptance;

pub mod depth_ci;

pub mod depth_routes;
pub mod manifest_promotion;
pub mod manifest_version_metadata;

pub mod manifest_validate;
