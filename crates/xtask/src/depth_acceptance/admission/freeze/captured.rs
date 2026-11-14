//! Invocation-local freeze authority. Closing reads verify availability without replacing facts.
use super::{acquisition, authority, Host};
use crate::depth_acceptance::admission::observations;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Captured {
    pub observation: Option<authority::Observation>,
    pub acquired: Option<acquisition::Retained>,
}
impl Captured {
    pub(crate) fn read(host: &dyn Host, candidate: &str) -> Result<Self, String> {
        let observation = host.freeze_inputs(candidate)?;
        let acquired = acquisition::read(host, candidate)?;
        Ok(Self {
            observation,
            acquired,
        })
    }

    pub(crate) fn pin(&self) {
        observations::fact("in1/freeze", "writer-inputs", &self.observation);
        observations::fact(
            "in1/freeze",
            "acquisition",
            &self
                .acquired
                .as_ref()
                .map(|retained| (&retained.observation, &retained.approved)),
        );
    }

    pub(crate) fn recheck(&self, host: &dyn Host, candidate: &str) -> Result<(), String> {
        if Self::read(host, candidate)? != *self {
            return Err("IN1 freeze input authority changed after capture".into());
        }
        Ok(())
    }
}
