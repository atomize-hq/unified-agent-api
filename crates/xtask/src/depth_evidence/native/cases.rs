//! Fixed native request population shared by registration, execution and journal recovery.
use super::CaseIdentity;
pub const REQUEST_TIMEOUT_SECONDS: u64 = 30;
pub const ATTEMPT_TIMEOUT_SECONDS: u64 =
    (VARIANTS.len() * STEPS.len()) as u64 * REQUEST_TIMEOUT_SECONDS + 60;
pub const VARIANTS: [&str; 2] = ["unattended", "interactive"];
pub const STEPS: [&str; 14] = [
    "initial",
    "resume-last",
    "resume-id",
    "fork-last",
    "fork-id",
    "cancel-resume-last",
    "cancel-resume-id",
    "cancel-fork-last",
    "cancel-fork-id",
    "cancel",
    "no-match-resume-last",
    "no-match-resume-id",
    "no-match-fork-last",
    "no-match-fork-id",
];
pub fn operation(step: &str) -> &str {
    if step == "cancel" {
        "initial"
    } else {
        step.strip_prefix("cancel-")
            .or_else(|| step.strip_prefix("no-match-"))
            .unwrap_or(step)
    }
}
pub fn requested(executable: &str) -> Vec<CaseIdentity> {
    VARIANTS
        .iter()
        .flat_map(|variant| {
            STEPS.map(|step| CaseIdentity {
                name: format!("codex-native/{variant}/{step}"),
                executable_sha256: executable.into(),
            })
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_population_covers_each_selector_cancellation_and_retains_distinct_identity() {
        let cases = requested(&"a".repeat(64));
        assert_eq!(cases.len(), 28);
        assert_eq!(
            cases
                .iter()
                .map(|c| &c.name)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            28
        );
        for variant in VARIANTS {
            for step in [
                "initial",
                "resume-last",
                "resume-id",
                "fork-last",
                "fork-id",
            ] {
                let cancel = if step == "initial" {
                    "cancel".to_owned()
                } else {
                    format!("cancel-{step}")
                };
                assert_eq!(operation(&cancel), step);
                assert!(cases
                    .iter()
                    .any(|c| c.name == format!("codex-native/{variant}/{cancel}")));
            }
        }
    }
}
