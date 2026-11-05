//! The lifecycle state's depth fields: the `support_depth` object of decision OF-P2 and the
//! `written_paths_sha256` of OF-P9 check (i), in
//! `docs/agents/lifecycle/support-depth-onboarding-freeze-decisions.md`.
//!
//! Only the onboarding depth freeze may write or replace the object, and only a passing
//! depth-enrolled write run may record the digest. Neither is installed, so the lifecycle writer
//! refuses any write that changes either field, and every lifecycle writer preserves both.

use serde_json::Value;

use super::{validation::validate_sha256, LifecycleError, LifecycleStage, LifecycleState};

const OBJECT: &str = "support_depth";
const WRITTEN_PATHS: &str = "written_paths_sha256";

pub(super) fn validate_state_fields(state: &LifecycleState) -> Result<(), LifecycleError> {
    if let Some(object) = &state.support_depth {
        object.validate().map_err(LifecycleError::Validation)?;
    }
    if let Some(digest) = &state.written_paths_sha256 {
        validate_sha256(WRITTEN_PATHS, digest)?;
        if state.support_depth.is_none() {
            return Err(LifecycleError::Validation(format!(
                "{WRITTEN_PATHS} is recorded only for a depth-enrolled generation, whose lifecycle \
                 state states `{OBJECT}`"
            )));
        }
        if state.lifecycle_stage < LifecycleStage::RuntimeIntegrated {
            return Err(LifecycleError::Validation(format!(
                "{WRITTEN_PATHS} is recorded by a passing write run, so it is valid only at \
                 `runtime_integrated` and later stages"
            )));
        }
    }
    Ok(())
}

/// Refuses a lifecycle write that adds, changes or removes a depth field of the state already at
/// the destination. `previous` is the destination's bytes, `None` when there is no file.
pub(super) fn require_preserved(
    previous: Option<&[u8]>,
    next: &LifecycleState,
) -> Result<(), LifecycleError> {
    let next = serde_json::to_value(next)
        .map_err(|err| LifecycleError::Internal(format!("serialize lifecycle state: {err}")))?;
    let previous = match previous.map(serde_json::from_slice::<Value>) {
        None => None,
        Some(Ok(value)) => Some(value),
        // A destination that is not JSON states no depth field this write could preserve. The
        // write may replace it only while it adds none.
        Some(Err(_)) => None,
    };
    for field in [OBJECT, WRITTEN_PATHS] {
        let before = previous.as_ref().and_then(|value| value.get(field));
        if before != next.get(field) {
            return Err(LifecycleError::Validation(format!(
                "lifecycle writer: `{field}` must be preserved byte for byte; only the onboarding \
                 depth freeze writes the depth object and only a depth-enrolled write run records \
                 the written-paths digest, and neither is installed"
            )));
        }
    }
    Ok(())
}
