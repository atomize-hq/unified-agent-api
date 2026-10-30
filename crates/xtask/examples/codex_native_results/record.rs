use agent_api::AgentWrapperEventKind;
use serde_json::Value;
use std::collections::BTreeSet;
use xtask::depth_evidence::CaseOutcome;

pub use xtask::depth_evidence::native::cases::{operation, STEPS};
pub use xtask::depth_evidence::native::native_session_id as valid_id;
pub use xtask::depth_evidence::native::NativeObservation as Observation;
/// Native strings are not copied into the report unless they have the finite ID grammar.
pub fn bind_session(data: Option<&Value>, id: &mut Option<String>, consistent: &mut bool) {
    if let Some(d) = data.filter(|d| d["schema"] == "agent_api.session.handle.v1") {
        match d["session"]["id"].as_str().filter(|s| valid_id(s)) {
            Some(s) if id.as_deref().map_or(true, |old| old == s) => *id = Some(s.into()),
            _ => *consistent = false,
        }
    }
}
/// Managed adapters label all item-start snapshots ToolCall; inspect their actual kind.
pub fn unexpected_item(kind: &AgentWrapperEventKind, data: Option<&Value>) -> bool {
    matches!(
        kind,
        AgentWrapperEventKind::ToolCall | AgentWrapperEventKind::ToolResult
    ) && !matches!(
        data.and_then(|d| d["tool"]["kind"].as_str()),
        Some("userMessage" | "agentMessage" | "reasoning")
    )
}
#[derive(Default)]
pub struct State {
    next: usize,
    seed: Option<String>,
    seen: BTreeSet<String>,
    stopped: bool,
}
impl State {
    pub fn seed(&self) -> Option<&str> {
        self.seed.as_deref()
    }
    pub fn record(&mut self, step: &str, o: &Observation) -> CaseOutcome {
        if self.stopped {
            return CaseOutcome::Aborted;
        }
        if step.starts_with("no-match-") {
            if STEPS.get(self.next) == Some(&step)
                && o.session_not_found
                && o.consistent
                && o.session_id.is_none()
                && !o.completion_success
                && !o.reply_matches
                && !o.cancel_requested
                && !o.cancelled
                && !o.unexpected_item
            {
                self.next += 1;
                return CaseOutcome::Passed;
            }
            self.stopped = true;
            return CaseOutcome::Failed;
        }
        let id = o.session_id.as_deref().filter(|id| valid_id(id));
        let identity = match operation(step) {
            "initial" => id.is_some(),
            // Native "last" uses metadata ordering, including timestamp ties. Cancellation
            // proves an owned known-project session, not an inferred chronological ordering.
            "resume-last" if step.starts_with("cancel") => {
                id.is_some_and(|id| self.seen.contains(id))
            }
            "resume-last" => id.is_some() && id == self.seed.as_deref(),
            "resume-id" => id.is_some() && id == self.seed.as_deref(),
            "fork-last" | "fork-id" => id.is_some_and(|id| !self.seen.contains(id)),
            _ => false,
        };
        let completion = if step.starts_with("cancel") {
            o.cancel_requested && o.cancelled && !o.completion_success
        } else {
            o.completion_success && o.reply_matches && !o.cancelled
        };
        if STEPS.get(self.next) != Some(&step)
            || !o.consistent
            || o.unexpected_item
            || !identity
            || !completion
        {
            self.stopped = true;
            return CaseOutcome::Failed;
        }
        if step == "initial" {
            self.seed = o.session_id.clone();
        }
        self.seen.insert(id.unwrap().into());
        self.next += 1;
        CaseOutcome::Passed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn success(id: &str) -> Observation {
        Observation {
            session_id: Some(id.into()),
            consistent: true,
            completion_success: true,
            reply_matches: true,
            ..Default::default()
        }
    }
    const SEED: &str = "aaaaaaaa-1111-4111-8111-111111111111";
    const FORK: &str = "bbbbbbbb-1111-4111-8111-111111111111";
    const OTHER: &str = "cccccccc-1111-4111-8111-111111111111";
    #[test]
    fn report_never_retains_unvalidated_session_strings_or_generic_item_payloads() {
        let mut id = None;
        let mut consistent = true;
        bind_session(
            Some(
                &serde_json::json!({"schema":"agent_api.session.handle.v1","session":{"id":"sk-secret-native-value"}}),
            ),
            &mut id,
            &mut consistent,
        );
        assert!(!consistent);
        assert!(id.is_none());
        for kind in ["userMessage", "agentMessage", "reasoning"] {
            assert!(!unexpected_item(
                &AgentWrapperEventKind::ToolCall,
                Some(&serde_json::json!({"tool":{"kind":kind}}))
            ));
        }
        for kind in [
            "commandExecution",
            "fileChange",
            "mcpToolCall",
            "unknown",
            "secret-like-native-string",
        ] {
            assert!(unexpected_item(
                &AgentWrapperEventKind::ToolCall,
                Some(&serde_json::json!({"tool":{"kind":kind}}))
            ));
        }
        assert!(unexpected_item(&AgentWrapperEventKind::ToolResult, None));
    }
    #[test]
    fn failed_or_ambiguous_seed_stops_all_later_admission() {
        for mutate in 0..6 {
            let mut state = State::default();
            let mut o = success(SEED);
            match mutate {
                0 => o.completion_success = false,
                1 => o.reply_matches = false,
                2 => o.consistent = false,
                3 => o.session_id = None,
                4 => o.session_id = Some("credential-shaped-untrusted-native-string".into()),
                _ => o.unexpected_item = true,
            };
            assert_eq!(state.record("initial", &o), CaseOutcome::Failed);
            assert_eq!(
                state.record("resume-id", &success(SEED)),
                CaseOutcome::Aborted
            );
        }
    }
    #[test]
    fn real_continuity_requires_exact_order_seed_and_distinct_forks() {
        let mut s = State::default();
        for (step, id) in STEPS[..5].iter().zip([SEED, SEED, SEED, FORK, OTHER]) {
            assert_eq!(s.record(step, &success(id)), CaseOutcome::Passed);
        }
        for (step, id) in [
            ("cancel-resume-last", OTHER),
            ("cancel-resume-id", SEED),
            ("cancel-fork-last", "dddddddd-1111-4111-8111-111111111111"),
            ("cancel-fork-id", "eeeeeeee-1111-4111-8111-111111111111"),
        ] {
            let o = Observation {
                session_id: Some(id.into()),
                consistent: true,
                cancel_requested: true,
                cancelled: true,
                ..Default::default()
            };
            assert_eq!(s.record(step, &o), CaseOutcome::Passed);
        }
        let cancelled = Observation {
            session_id: Some(OTHER.into()),
            consistent: true,
            cancel_requested: true,
            cancelled: true,
            ..Default::default()
        };
        assert_eq!(s.record("cancel", &cancelled), CaseOutcome::Passed);
        for (step, id) in [("resume-id", SEED), ("resume-last", OTHER)] {
            let mut s = State::default();
            assert_eq!(s.record("initial", &success(SEED)), CaseOutcome::Passed);
            assert_eq!(s.record(step, &success(id)), CaseOutcome::Failed);
        }
        let mut s = State::default();
        for step in &STEPS[..3] {
            assert_eq!(s.record(step, &success(SEED)), CaseOutcome::Passed);
        }
        assert_eq!(s.record("fork-last", &success(SEED)), CaseOutcome::Failed);
    }
    #[test]
    fn cancellation_requires_owned_request_and_exact_cancelled_completion() {
        for requested in [false, true] {
            for cancelled in [false, true] {
                let mut s = State::default();
                for (step, id) in STEPS[..5].iter().zip([SEED, SEED, SEED, FORK, OTHER]) {
                    assert_eq!(s.record(step, &success(id)), CaseOutcome::Passed);
                }
                for (step, id) in [
                    ("cancel-resume-last", OTHER),
                    ("cancel-resume-id", SEED),
                    ("cancel-fork-last", "dddddddd-1111-4111-8111-111111111111"),
                    ("cancel-fork-id", "eeeeeeee-1111-4111-8111-111111111111"),
                ] {
                    let o = Observation {
                        session_id: Some(id.into()),
                        consistent: true,
                        cancel_requested: true,
                        cancelled: true,
                        ..Default::default()
                    };
                    assert_eq!(s.record(step, &o), CaseOutcome::Passed);
                }
                let o = Observation {
                    session_id: Some(OTHER.into()),
                    consistent: true,
                    cancel_requested: requested,
                    cancelled,
                    ..Default::default()
                };
                assert_eq!(
                    s.record("cancel", &o),
                    if requested && cancelled {
                        CaseOutcome::Passed
                    } else {
                        CaseOutcome::Failed
                    }
                );
            }
        }
    }
}

#[cfg(test)]
mod absence_tests {
    use super::*;
    #[test]
    fn missing_session_controls_require_exact_refusal_and_no_session_or_success() {
        for attack in 0..4 {
            let mut state = State {
                next: 10,
                ..Default::default()
            };
            let mut obs = Observation {
                consistent: true,
                session_not_found: true,
                ..Default::default()
            };
            match attack {
                0 => (),
                1 => obs.session_not_found = false,
                2 => obs.session_id = Some("aaaaaaaa-1111-4111-8111-111111111111".into()),
                _ => obs.completion_success = true,
            }
            assert_eq!(
                state.record("no-match-resume-last", &obs),
                if attack == 0 {
                    CaseOutcome::Passed
                } else {
                    CaseOutcome::Failed
                }
            );
        }
    }
}

#[cfg(test)]
mod last_cancellation_tests {
    use super::*;
    #[test]
    fn last_cancellation_does_not_infer_native_metadata_order_from_observation_order() {
        let known = [
            "aaaaaaaa-1111-4111-8111-111111111111",
            "bbbbbbbb-1111-4111-8111-111111111111",
        ];
        for id in known
            .into_iter()
            .chain(["cccccccc-1111-4111-8111-111111111111"])
        {
            let mut state = State {
                next: 5,
                seed: Some(known[0].into()),
                seen: known.into_iter().map(String::from).collect(),
                ..Default::default()
            };
            let obs = Observation {
                session_id: Some(id.into()),
                consistent: true,
                cancel_requested: true,
                cancelled: true,
                ..Default::default()
            };
            assert_eq!(
                state.record("cancel-resume-last", &obs),
                if known.contains(&id) {
                    CaseOutcome::Passed
                } else {
                    CaseOutcome::Failed
                }
            );
        }
    }
}
