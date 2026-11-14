//! Slice 10a consumes the actual ON3 checkpoint and coherent proposed Publication entry.
use super::*;
use crate::agent_maintenance::closeout_proposal::{IsolatedView, Proposal};
use crate::isolated_maintenance::CapturedRegistry;

pub(super) fn check_connected(
    caller: &Path,
    proposal: &Proposal,
    captured: &CapturedRegistry,
    view: &IsolatedView<'_>,
    scratch: &Path,
    record: &DepthRecord,
) {
    let before = crate::candidate_execution::tree::capture(caller, &Default::default()).unwrap();
    crate::readonly_gates::check_proposed(caller, proposal, captured, view, record).unwrap();

    let mut stale = record.clone();
    stale.acceptance_entries.clear();
    assert_eq!(
        crate::readonly_gates::check_proposed(caller, proposal, captured, view, &stale)
            .unwrap_err(),
        "read-only gates require the exact proposed record"
    );
    let path = crate::capability_matrix::DEFAULT_OUT_PATH;
    let original = std::fs::read(scratch.join(path)).unwrap();
    let mut changed = original.clone();
    changed.push(b'\n');
    std::fs::write(scratch.join(path), changed).unwrap();
    let error = crate::readonly_gates::check_proposed(caller, proposal, captured, view, record)
        .unwrap_err();
    assert!(
        error.ends_with("is stale; regenerate with `cargo run -p xtask -- capability-matrix`"),
        "{error}"
    );
    std::fs::write(scratch.join(path), original).unwrap();

    let path = crate::runtime_advertising::DATA;
    let original = std::fs::read_to_string(scratch.join(path)).unwrap();
    std::fs::write(
        scratch.join(path),
        format!("{original}\nconst S10_CORRUPT: bool = true;\n"),
    )
    .unwrap();
    assert_eq!(
        crate::readonly_gates::check_proposed(caller, proposal, captured, view, record)
            .unwrap_err(),
        "working bytes disagree with index: crates/agent_api/src/runtime_advertising/data.rs"
    );
    std::fs::write(scratch.join(path), original).unwrap();
    crate::readonly_gates::check_proposed(caller, proposal, captured, view, record).unwrap();
    assert_eq!(
        crate::candidate_execution::tree::capture(caller, &Default::default()).unwrap(),
        before
    );
}
