use super::*;
use crate::agent_registry::support_depth::{CapabilityMapping, SharedMapping};

#[cfg(unix)]
#[test]
fn executable_permission_drift_invalidates_even_without_byte_changes() {
    use std::os::unix::fs::PermissionsExt;
    for (input, mode) in [
        ("crates/codex/tests/fixtures/synthetic.sh", 0o644),
        ("crates/codex/src/lib.rs", 0o755),
    ] {
        for staged in [false, true] {
            let f = Fixture::new();
            assert_eq!(f.result(), DepthResult::Verified);
            fs::set_permissions(f.root.path().join(input), fs::Permissions::from_mode(mode))
                .unwrap();
            if staged {
                git(f.root.path(), &["add", "."]);
            }
            assert_eq!(
                f.result(),
                DepthResult::Unverified,
                "{input} staged={staged}"
            );
        }
    }
    let f = Fixture::new();
    let file = write(f.root.path(), "docs/unrelated.md", "unrelated");
    fs::set_permissions(
        f.root.path().join(file.path),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    git(f.root.path(), &["add", "."]);
    assert_eq!(f.result(), DepthResult::Verified);
}

fn rebind(f: &mut Fixture) {
    reseal(&mut f.current.record);
    f.current.current.policy_identity = f.current.record.policy.identity.clone();
    f.current.current.obligations_identity = f
        .current
        .record
        .obligations
        .as_ref()
        .unwrap()
        .identity
        .clone();
    f.receipt.scope.policy_identity = f.current.current.policy_identity.clone();
    f.receipt.scope.obligations_identity = f.current.current.obligations_identity.clone();
    f.update();
}
#[test]
fn capability_obligations_require_the_exact_policy_mapping() {
    let mut f = Fixture::new();
    f.receipt.scope.obligation.id = None;
    f.receipt.scope.obligation.capability = Some("agent_api.run".into());
    f.receipt.adequacy.mapping.obligation = f.receipt.scope.obligation.clone();
    f.current.current.obligation = f.receipt.scope.obligation.clone();
    f.current.record.execution.as_mut().unwrap().results[0].obligation =
        f.receipt.scope.obligation.clone();
    f.update();
    assert_eq!(
        f.result(),
        DepthResult::Unverified,
        "M is not selected by P"
    );
    let enrollment = &mut f.current.record.policy.selection.enrollment;
    enrollment.operations[0].shared_mapping = Some(SharedMapping::Mapped);
    enrollment.capabilities.push(CapabilityMapping {
        id: "agent_api.run".into(),
        operations: vec!["run".into()],
    });
    rebind(&mut f);
    assert_eq!(f.result(), DepthResult::Verified);
    f.receipt.scope.obligation.capability = Some("ghost".into());
    f.receipt.adequacy.mapping.obligation = f.receipt.scope.obligation.clone();
    f.current.current.obligation = f.receipt.scope.obligation.clone();
    f.current.record.execution.as_mut().unwrap().results[0].obligation =
        f.receipt.scope.obligation.clone();
    f.update();
    assert_eq!(f.result(), DepthResult::Unverified, "different capability");
    f.receipt.scope.obligation.capability = Some("agent_api.run".into());
    f.receipt.adequacy.mapping.obligation = f.receipt.scope.obligation.clone();
    f.current.current.obligation = f.receipt.scope.obligation.clone();
    f.current.record.execution.as_mut().unwrap().results[0].obligation =
        f.receipt.scope.obligation.clone();
    let enrollment = &mut f.current.record.policy.selection.enrollment;
    let mut other = enrollment.operations[0].clone();
    other.id = "z-other".into();
    enrollment.operations[0].shared_mapping = Some(SharedMapping::WrapperOnly);
    enrollment.operations.push(other);
    enrollment.capabilities[0].operations = vec!["z-other".into()];
    let o = f.current.record.obligations.as_mut().unwrap();
    let mut other = o.operations[0].clone();
    other.id = "z-other".into();
    o.operations.push(other);
    rebind(&mut f);
    assert_eq!(
        f.result(),
        DepthResult::Unverified,
        "capability maps another operation"
    );
}
