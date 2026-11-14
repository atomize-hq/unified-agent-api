//! Drift occurs after the standalone consumer's final authenticated registry observation.
use super::*;

fn closing_observations(wrapper: &str) {
    let (f, host) = setup();
    let root = f.root.path();
    let _route = f.install();
    let _reader = host.install();
    let before = super::super::ma5_closeout::snapshot(root);
    for case in ["host", "protected_parent"] {
        let observed = std::rc::Rc::new(std::cell::Cell::new(false));
        let reached = observed.clone();
        let api = f.api.clone();
        let baseline = f.baseline.clone();
        let _observer = super::super::fixture::after_registry_read(Box::new(move || {
            reached.set(true);
            match case {
                "host" => api.put(
                    "repos/fixture/isolated",
                    json!({"id":11,"owner":{"id":20},"full_name":"fixture/isolated"}),
                ),
                "protected_parent" => api.put(
                    "repos/fixture/isolated/branches/staging",
                    json!({"name":"staging","protected":true,"commit":{"sha":baseline}}),
                ),
                _ => unreachable!(),
            }
        }));
        let result = match wrapper {
            "readonly_route" => gates::with_readonly_route(root, || {
                crate::capability_matrix::check_publication(root)
            }),
            "capability" => crate::capability_matrix::check_in_workspace(root),
            _ => unreachable!(),
        };
        assert!(observed.get(), "consumer never completed its registry read");
        let expected = match case {
            "host" => "isolated repository id, owner id or full name differs",
            "protected_parent" => "isolated host identity, source or protected tip changed",
            _ => unreachable!(),
        };
        assert_eq!(result.unwrap_err(), expected, "{wrapper}: {case}");
        assert_eq!(super::super::ma5_closeout::snapshot(root), before);
        f.host();
    }
}

#[test]
fn readonly_gates_registry_route_closes_after_final_read() {
    closing_observations("readonly_route");
}

#[test]
fn readonly_gates_registry_capability_closes_after_final_read() {
    closing_observations("capability");
}
