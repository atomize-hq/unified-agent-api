//! The other MA2 read routes under the isolated context (B.3 part 2b): acquisition plan,
//! stand-down check, manifest validation, support matrix and audit status.
use super::prepare::{acquire, workspace};
use super::*;
use crate::agent_lifecycle::maintenance_request_path;
use crate::agent_maintenance::{
    audit_status,
    prepare::{apply_prepare_plan, build_prepare_plan},
    stand_down::{self, StandDownOutcome},
};

fn stand_down(root: &Path) -> Result<StandDownOutcome, String> {
    stand_down::run(stand_down::Args {
        agent: "depth_fixture".into(),
        target_version: "0.2.0".into(),
        from_ref: None,
        workspace_root: Some(root.to_path_buf()),
    })
    .map_err(|e| e.to_string())
}

fn validate(root: &Path) -> i32 {
    crate::manifest_validate::run(validate_args(root))
}

fn validate_args(root: &Path) -> crate::manifest_validate::Args {
    crate::manifest_validate::Args {
        root: Some(root.join("cli_manifests/depth_fixture")),
        rules: None,
        schema: None,
        version_schema: None,
        mode: crate::manifest_validate::Mode::Check,
        json: false,
        tip: None,
    }
}

fn audit(root: &Path) -> Result<audit_status::AuditStatusOutcome, String> {
    audit_status::run_in_workspace(
        root,
        audit_status::Args {
            request: root.join(maintenance_request_path("depth_fixture")),
            expect_target_version: Some("0.2.0".into()),
            emit_json: None,
            workspace_root: Some(root.to_path_buf()),
        },
        &mut Vec::new(),
    )
    .map_err(|e| e.to_string())
}

#[test]
fn ma2_read_routes_load_the_scoped_registry_only_under_an_installed_route() {
    let (f, args) = workspace();
    let root = f.root.path();
    // The ordinary route keeps every production depth refusal.
    let refused = |error: String| assert!(error.contains("support_depth"), "{error}");
    refused(
        crate::manifest_acquisition::plan_for_agent(root, "depth_fixture", "0.2.0")
            .unwrap_err()
            .to_string(),
    );
    refused(stand_down(root).unwrap_err());
    refused(crate::support_matrix::generate_publication_bundle(root).unwrap_err());

    let _installed = f.install();
    let plan = crate::manifest_acquisition::plan_for_agent(root, "depth_fixture", "0.2.0").unwrap();
    assert_eq!(plan.manifest_root, "cli_manifests/depth_fixture");
    assert_eq!(plan.version, "0.2.0");
    // MA1, then MA2's acquisition and re-freeze, through the public prepare route.
    apply_prepare_plan(root, &build_prepare_plan(root, &args).unwrap()).unwrap();
    f.commit();
    acquire(root, "0.2.0");
    f.commit();
    // Before publication the support matrix is stale for the acquired version, so validation
    // fails on support-matrix checks alone. This is why parity-acquire.yml validates only after
    // it publishes the support outputs.
    let codes = crate::manifest_validate::violation_codes(validate_args(root)).unwrap();
    assert!(
        !codes.is_empty() && codes.iter().all(|code| code.starts_with("SUPPORT_MATRIX_")),
        "{codes:?}"
    );
    apply_prepare_plan(root, &build_prepare_plan(root, &args).unwrap()).unwrap();
    f.commit();
    assert_eq!(stand_down(root).unwrap(), StandDownOutcome::Authorized);
    assert_eq!(validate(root), 0);
    let bundle = crate::support_matrix::generate_publication_bundle(root).unwrap();
    assert!(bundle.json.contains("depth_fixture"));
    // Audit-status is a support-surface audit and, by the contract, not a depth-gated output: it
    // stays clean while O is unclassified. Depth readiness is gated at MA5 closeout.
    assert_eq!(
        audit(root).unwrap(),
        audit_status::AuditStatusOutcome::Clean
    );
    assert!(crate::isolated_maintenance::registry(root)
        .unwrap()
        .find("depth_fixture")
        .unwrap()
        .support_depth
        .is_some());
}

#[test]
fn a_failing_installed_route_never_falls_back_to_the_ordinary_registry() {
    let (f, _) = workspace();
    let root = f.root.path();
    let _installed = f.install();
    // The isolated repository is recreated under another identity.
    f.api.put(
        "repos/fixture/isolated",
        json!({"id":11,"owner":{"id":20},"full_name":"fixture/isolated"}),
    );
    assert!(crate::isolated_maintenance::registry(root).is_err());
    assert!(crate::manifest_acquisition::plan_for_agent(root, "depth_fixture", "0.2.0").is_err());
    assert!(stand_down(root).is_err());
    // Validation reports the failing route instead of skipping its depth checks.
    let codes = crate::manifest_validate::violation_codes(validate_args(root)).unwrap();
    assert!(codes.contains(&"DEPTH_ROUTE_REFUSED"), "{codes:?}");
    assert!(crate::support_matrix::generate_publication_bundle(root).is_err());
}
