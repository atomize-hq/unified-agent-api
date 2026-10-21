//! Private generated advertising data and fixed runtime inputs; no repository access at runtime.
pub(crate) mod data;
pub(crate) mod evaluator;

pub(crate) fn host() -> evaluator::HostFacts<'static> {
    evaluator::HostFacts {
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
    }
}
