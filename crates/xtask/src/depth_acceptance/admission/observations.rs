//! Invocation-local normalized authority commitments, computed by approved readers.
use super::record::{canonical, digest};
use serde::Serialize;
use std::{cell::RefCell, collections::BTreeSet, marker::PhantomData};

const DOMAIN: &[u8] = b"support-depth-in1-observed-authority-v1\0";
#[derive(Default)]
struct State {
    facts: BTreeSet<String>,
    incomplete: bool,
    error: Option<String>,
}
thread_local! {
    static ACTIVE: RefCell<Option<State>> = const { RefCell::new(None) };
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Authority {
    pub identity: String,
    pub incomplete: bool,
}

/// Ends before admission-history reconstruction/publication. Independent read order and
/// repeated facts preserve identity. Any unavailable read remains sticky for this invocation.
pub(crate) struct Capture(PhantomData<*const ()>);
impl Capture {
    pub(crate) fn start() -> Result<Self, String> {
        ACTIVE.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_some() {
                return Err("IN1 host observation capture already active".into());
            }
            *slot = Some(State::default());
            Ok(Self(PhantomData))
        })
    }
    pub(crate) fn finish(self) -> Result<Authority, String> {
        ACTIVE.with(|slot| {
            let state = slot.borrow_mut().take().ok_or("IN1 capture missing")?;
            if let Some(error) = state.error {
                return Err(error);
            }
            Ok(Authority {
                identity: digest(DOMAIN, &canonical(&state.facts)?),
                incomplete: state.incomplete,
            })
        })
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        ACTIVE.with(|slot| *slot.borrow_mut() = None);
    }
}

/// Call only with the fields used by the predicate, after authority filtering.
pub(crate) fn fact(endpoint: &str, kind: &str, value: &impl Serialize) {
    ACTIVE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(state) = slot.as_mut() else { return };
        let observed = canonical(value).and_then(|bytes| {
            canonical(&(endpoint, kind, digest(b"", &bytes)))
                .map(|bytes| digest(b"support-depth-in1-host-read-v1\0", &bytes))
        });
        match observed {
            Ok(identity) => {
                state.facts.insert(identity);
            }
            Err(error) => state.error = Some(error),
        }
    });
}

/// Successful transport payloads acquire meaning only at the owning predicate. Arbitrary
/// response/error text cannot supply authority. Availability is recorded before parsing.
pub(crate) fn unavailable<T>(endpoint: &str, response: &Result<T, String>) {
    if response.is_err() {
        ACTIVE.with(|slot| {
            if let Some(state) = slot.borrow_mut().as_mut() {
                state.incomplete = true;
            }
        });
        fact(endpoint, "availability", &["unavailable"]);
    }
}

/// Exhaustion is independent of the number of ignored comments or unrelated artifacts.
pub(crate) fn discovery(endpoint: &str, complete: bool) {
    fact(endpoint, "discovery", &complete);
}

pub(crate) fn population(endpoint: &str, kind: &str, mut rows: Vec<serde_json::Value>) {
    rows.sort_by_cached_key(serde_json::Value::to_string);
    fact(endpoint, kind, &rows);
}

#[cfg(test)]
mod tests;
