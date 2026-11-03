//! The `synthetic-wrapper-v1` case population (SA-P3). One test target, so the isolated
//! collector runs and binds every mapped case from a single executable. These cases prove the
//! wrapper against the fixture CLI only; they qualify no real agent.
#[path = "depth_ci_synthetic/lifecycle.rs"]
mod lifecycle;
#[path = "depth_ci_synthetic/run.rs"]
mod run;
