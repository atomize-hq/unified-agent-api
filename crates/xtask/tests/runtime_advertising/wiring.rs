use super::*;

#[test]
fn preflight_requires_runtime_source_conformance_alongside_matrix_freshness() {
    let makefile = std::fs::read_to_string(workspace().join("Makefile")).unwrap();
    let guard = makefile
        .split("capability-matrix-guard:\n")
        .nth(1)
        .unwrap()
        .split(".PHONY:")
        .next()
        .unwrap();
    assert!(guard.contains("cargo run -p xtask -- runtime-advertising-model --check"));
    let flight = makefile
        .split("flightcheck:\n")
        .nth(1)
        .unwrap()
        .split(".PHONY:")
        .next()
        .unwrap();
    assert!(flight.contains("$(MAKE) capability-matrix-guard"));
}
