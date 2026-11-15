//! Exact inventory checks make the unverified-result control nonvacuous at both levels.
use crate::{
    agent_registry::support_depth::ObligationId,
    depth_record::DepthResult,
    support_matrix::depth::{DepthFacts, PromiseFacts},
};

pub(super) fn assert_unverified(depth: &DepthFacts) {
    assert_eq!(
        depth
            .promises
            .iter()
            .map(|p| (p.operation.as_str(), p.templates.as_slice()))
            .collect::<Vec<_>>(),
        [("run", ["CORE".to_owned()].as_slice())],
        "depth promise inventory"
    );
    let promise = &depth.promises[0];
    let mut expected = ObligationId::BASE.to_vec();
    expected.extend([ObligationId::Lc2, ObligationId::Out3]);
    expected.sort();
    assert_eq!(
        promise
            .obligations
            .iter()
            .map(|o| (o.id, o.capability.as_deref()))
            .collect::<Vec<_>>(),
        expected
            .into_iter()
            .map(|id| (Some(id), None))
            .collect::<Vec<_>>(),
        "depth obligation inventory"
    );
    assert!(depth.promises.iter().all(|p| !p.qualified
        && p.obligations
            .iter()
            .all(|o| o.result == DepthResult::Unverified)));
}

fn empty() -> DepthFacts {
    DepthFacts {
        obligations_frozen: true,
        promises: Vec::new(),
        mode_exclusions: Vec::new(),
        capabilities: Vec::new(),
    }
}

#[test]
#[should_panic(expected = "depth promise inventory")]
fn readonly_gates_unverified_control_refuses_empty_promises() {
    assert_unverified(&empty());
}

#[test]
#[should_panic(expected = "depth obligation inventory")]
fn readonly_gates_unverified_control_refuses_empty_obligations() {
    let mut depth = empty();
    depth.promises.push(PromiseFacts {
        operation: "run".into(),
        templates: vec!["CORE".into()],
        qualified: false,
        outcomes: Vec::new(),
        obligations: Vec::new(),
    });
    assert_unverified(&depth);
}
