//! Retained protected-IN1 decisions. Candidate data has no constructor for host authority.
pub(crate) mod emitter;
pub(crate) mod freeze;
pub(crate) mod history;
mod objects;
pub(crate) mod observations;
pub(crate) mod policy;
mod record;
pub(crate) mod transport;
pub(crate) use record::*;
pub(crate) use transport::{Authority, Host, Publication};

#[cfg(test)]
pub(crate) mod fake;
#[cfg(test)]
mod tests;
