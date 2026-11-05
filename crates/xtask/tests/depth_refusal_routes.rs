#![cfg(unix)]
//! Executed depth-refusal tests for the support-depth contract's Annex A routes ON1–ON5, NE1–NE8
//! and LW1 (roadmap B.7). Each route's public entrypoint is driven against a copy of the
//! repository that carries the synthetic `depth_fixture` subject's depth (a declared table, or a
//! retained record whose declaration is gone) and must refuse before any write. The route → test
//! mapping is in docs/agents/lifecycle/support-depth-validation-test-list.md.

#[path = "depth_refusal_routes/support.rs"]
mod support;

#[path = "depth_refusal_routes/lower.rs"]
mod lower;
#[path = "depth_refusal_routes/neutral.rs"]
mod neutral;
#[path = "depth_refusal_routes/onboarding.rs"]
mod onboarding;

use support::Workspace;

/// The refusal is caused by the synthetic depth, not by the copied tree: without it the guard
/// admits the same workspace.
#[test]
fn copied_workspace_without_synthetic_depth_is_not_refused() {
    let ws = Workspace::plain();
    xtask::depth_routes::refuse_workspace(ws.root(), "control").unwrap();
    for depth in support::Depth::ALL {
        let ws = Workspace::with(depth);
        let error = xtask::depth_routes::refuse_workspace(ws.root(), "control").unwrap_err();
        assert_eq!(error, format!("control: {}", depth.guard_reason()));
    }
}
