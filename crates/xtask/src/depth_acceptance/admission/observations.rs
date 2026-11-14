//! Invocation-local host-read commitments. Only approved transport adapters contribute facts.
use super::record::{canonical, digest};
use std::{cell::RefCell, collections::BTreeSet, marker::PhantomData};

const DOMAIN: &[u8] = b"support-depth-in1-observed-authority-v1\0";
thread_local! {
    static ACTIVE: RefCell<Option<Result<BTreeSet<String>, String>>> = const { RefCell::new(None) };
}

/// Scoped to the owning evaluation, ending before admission-history reconstruction/publication.
/// Repeated reads and independent read order cannot change the observed authority identity.
pub(crate) struct Capture(PhantomData<*const ()>);
impl Capture {
    pub(crate) fn start() -> Result<Self, String> {
        ACTIVE.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_some() {
                return Err("IN1 host observation capture already active".into());
            }
            *slot = Some(Ok(BTreeSet::new()));
            Ok(Self(PhantomData))
        })
    }
    pub(crate) fn finish(self) -> Result<String, String> {
        ACTIVE.with(|slot| {
            let facts = slot.borrow_mut().take().ok_or("IN1 capture missing")??;
            Ok(digest(DOMAIN, &canonical(&facts)?))
        })
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        ACTIVE.with(|slot| *slot.borrow_mut() = None);
    }
}

fn observe(endpoint: &str, kind: &str, response: impl FnOnce() -> Result<Vec<u8>, String>) {
    ACTIVE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(Ok(facts)) = slot.as_mut() else {
            return;
        };
        let observed = response().and_then(|bytes| {
            canonical(&(endpoint, kind, digest(b"", &bytes)))
                .map(|bytes| digest(b"support-depth-in1-host-read-v1\0", &bytes))
        });
        match observed {
            Ok(identity) => {
                facts.insert(identity);
            }
            Err(error) => *slot = Some(Err(error)),
        }
    });
}

pub(crate) fn json(endpoint: &str, response: &Result<serde_json::Value, String>) {
    // Transport failure has one bounded state. Error messages can contain local paths or
    // arbitrary remote text and never supply authority. Successful empty lists preserve absence.
    observe(endpoint, "json", || match response {
        Ok(value) => canonical(&("json", value)),
        Err(_) => canonical(&["unavailable"]),
    });
}
pub(crate) fn bytes(endpoint: &str, response: &Result<Vec<u8>, String>) {
    observe(endpoint, "bytes", || match response {
        Ok(bytes) => canonical(&("bytes", digest(b"", bytes))),
        Err(_) => canonical(&["unavailable"]),
    });
}

#[cfg(test)]
mod tests;
