//! FR2b owns construction of authenticated non-empty acquisition deltas.
#[derive(Debug, Clone)]
pub(crate) struct ValidatedAcquisitionDelta {
    _private: (),
}
impl ValidatedAcquisitionDelta {
    pub(crate) fn empty() -> Self {
        Self { _private: () }
    }
}

/// Only this authority module may construct the planner input. FR2a grants no non-empty
/// authority, even if acquisition schemas, checksums or self-asserted O references agree.
pub(crate) fn validate(
    predecessor: &super::Tree,
    candidate: &super::Tree,
    output_paths: &[String],
) -> Result<ValidatedAcquisitionDelta, String> {
    if output_paths
        .iter()
        .any(|path| predecessor.get(path) != candidate.get(path))
    {
        return Err("acquisition authority unavailable".into());
    }
    Ok(ValidatedAcquisitionDelta::empty())
}
