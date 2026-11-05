//! The lifecycle state's depth fields: the `support_depth` object of decision OF-P2 and the
//! `written_paths_sha256` of OF-P9 check (i), in
//! `docs/agents/lifecycle/support-depth-onboarding-freeze-decisions.md`.
//!
//! Only the onboarding depth freeze may write or replace the object, and only a passing
//! depth-enrolled write run may record the digest. Neither is installed, so the lifecycle writer
//! refuses any write that changes either field's bytes, and every lifecycle writer preserves both.
//! Because every writer re-serializes the state, the fields are byte-preserved only when they are
//! held in the one form a writer writes; a reader therefore refuses any other form.

use serde_json::Value;

use super::{validation::validate_sha256, LifecycleError, LifecycleStage, LifecycleState};

const OBJECT: &str = "support_depth";
const WRITTEN_PATHS: &str = "written_paths_sha256";
const FIELDS: [&str; 2] = [OBJECT, WRITTEN_PATHS];

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

/// The bytes a lifecycle writer writes for `state`.
pub(super) fn written_bytes(state: &LifecycleState) -> Result<Vec<u8>, LifecycleError> {
    let mut bytes = serde_json::to_vec_pretty(state)
        .map_err(|err| LifecycleError::Internal(format!("serialize lifecycle state: {err}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// Refuses a state whose depth fields are not written byte for byte as a lifecycle writer writes
/// them, since a writer would then change their bytes.
pub(super) fn require_written_form(
    bytes: &[u8],
    state: &LifecycleState,
) -> Result<(), LifecycleError> {
    let written = written_bytes(state)?;
    for field in FIELDS {
        if top_level_value(bytes, field) != top_level_value(&written, field) {
            return Err(LifecycleError::Validation(format!(
                "lifecycle state: `{field}` must be written exactly as the lifecycle writer writes \
                 it (two-space pretty JSON, members in declared order), so that every writer \
                 preserves it byte for byte"
            )));
        }
    }
    Ok(())
}

/// Refuses a lifecycle write that adds, changes or removes a depth field's bytes, compared with
/// the state already at the destination. `previous` is the destination's bytes, `None` when there
/// is no file.
pub(super) fn require_preserved(
    previous: Option<&[u8]>,
    next: &LifecycleState,
) -> Result<(), LifecycleError> {
    let next = written_bytes(next)?;
    // A destination that is not JSON states no depth field this write could preserve. The write
    // may replace it only while it adds none.
    let previous = previous.filter(|bytes| serde_json::from_slice::<Value>(bytes).is_ok());
    for field in FIELDS {
        let before = previous.and_then(|bytes| top_level_value(bytes, field));
        if before != top_level_value(&next, field) {
            return Err(LifecycleError::Validation(format!(
                "lifecycle writer: `{field}` must be preserved byte for byte; only the onboarding \
                 depth freeze writes the depth object and only a depth-enrolled write run records \
                 the written-paths digest, and neither is installed"
            )));
        }
    }
    Ok(())
}

/// The exact bytes of the value of the root member `key` of a JSON object, `None` when there is no
/// such member or the bytes are not an object. A member name written with an escape is not
/// matched, which reads as a missing member and so is refused wherever the member is expected.
pub(super) fn top_level_value<'a>(bytes: &'a [u8], key: &str) -> Option<&'a [u8]> {
    let mut at = skip_ws(bytes, 0);
    if bytes.get(at) != Some(&b'{') {
        return None;
    }
    at += 1;
    loop {
        at = skip_ws(bytes, at);
        match bytes.get(at)? {
            b'}' => return None,
            b',' => at = skip_ws(bytes, at + 1),
            _ => {}
        }
        let name_end = skip_string(bytes, at)?;
        let name = &bytes[at + 1..name_end - 1];
        at = skip_ws(bytes, name_end);
        if bytes.get(at) != Some(&b':') {
            return None;
        }
        let start = skip_ws(bytes, at + 1);
        let end = skip_value(bytes, start)?;
        if name == key.as_bytes() {
            return Some(&bytes[start..end]);
        }
        at = end;
    }
}

fn skip_ws(bytes: &[u8], mut at: usize) -> usize {
    while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
        at += 1;
    }
    at
}

/// The index just past the string that starts at `at`.
fn skip_string(bytes: &[u8], at: usize) -> Option<usize> {
    if bytes.get(at) != Some(&b'"') {
        return None;
    }
    let mut at = at + 1;
    loop {
        match bytes.get(at)? {
            b'\\' => at += 2,
            b'"' => return Some(at + 1),
            _ => at += 1,
        }
    }
}

/// The index just past the value that starts at `at`.
fn skip_value(bytes: &[u8], at: usize) -> Option<usize> {
    match bytes.get(at)? {
        b'"' => skip_string(bytes, at),
        b'{' | b'[' => {
            let mut depth = 0usize;
            let mut at = at;
            loop {
                match bytes.get(at)? {
                    b'"' => {
                        at = skip_string(bytes, at)?;
                        continue;
                    }
                    b'{' | b'[' => depth += 1,
                    b'}' | b']' => {
                        depth -= 1;
                        if depth == 0 {
                            return Some(at + 1);
                        }
                    }
                    _ => {}
                }
                at += 1;
            }
        }
        _ => {
            let mut at = at;
            while bytes
                .get(at)
                .is_some_and(|b| !matches!(b, b',' | b'}' | b']') && !b.is_ascii_whitespace())
            {
                at += 1;
            }
            Some(at)
        }
    }
}
