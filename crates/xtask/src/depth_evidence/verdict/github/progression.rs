//! Source ancestry, not API listing order or retry time, orders a frozen generation.
use super::*;
use crate::agent_lifecycle::{LifecycleStage, LifecycleState};
use crate::runtime_advertising::source::Source;
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct Onboarding {
    sources: BTreeMap<String, LifecycleStage>,
}

impl Onboarding {
    /// The caller establishes matching Event/P/O before observing a host head or checkout.
    /// Include both: a PR test merge must not restore lifecycle state behind its own head.
    pub(super) fn observe(
        &mut self,
        root: &Path,
        commit: &str,
        source: &Source,
        p: Profile,
        record: &DepthRecord,
    ) -> Result<(), String> {
        if p != Profile::SyntheticOnboarding || self.sources.contains_key(commit) {
            return Ok(());
        }
        depth_ci::onboarding::check(source, p, record)?;
        let state: LifecycleState =
            serde_json::from_str(&source.read(depth_ci::onboarding::STATE)?)
                .map_err(|e| e.to_string())?;
        for (other, &stage) in &self.sources {
            if state.lifecycle_stage == stage {
                continue;
            }
            // Only a higher-stage ancestor of a lower-stage descendant is a rollback.
            // Historical enrolled sources and their later retries remain legitimate.
            let (ancestor, prior, descendant, next) = if stage > state.lifecycle_stage {
                (other.as_str(), stage, commit, state.lifecycle_stage)
            } else {
                (commit, state.lifecycle_stage, other.as_str(), stage)
            };
            let status = crate::git_inputs::command(root)
                .args(["merge-base", "--is-ancestor", ancestor, descendant])
                .output()
                .map_err(|e| e.to_string())?;
            match status.status.code() {
                Some(0) => crate::runtime_follow_on::depth::check_progression(prior, next)?,
                Some(1) => {}
                _ => return Err("cannot establish onboarding source ancestry".into()),
            }
        }
        self.sources.insert(commit.into(), state.lifecycle_stage);
        Ok(())
    }
}
