#![forbid(unsafe_code)]

//! Feature-gated OpenCode backend registration scaffold.
//!
//! S00 only establishes the module boundary so later slices can add the actual backend
//! implementation without reopening the registration contract.
