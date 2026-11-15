//! Named writer-variable provenance, independently observed by the installed protected host.
use super::super::{canonical, digest, Generation, Object, Policy, Repository};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Observation {
    pub repository: Repository,
    pub predecessor: Object,
    pub generation: Generation,
    pub policy: Policy,
    pub writer: Writer,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Writer {
    /// Event, release-watch invocation and timestamp are read from the predecessor's original
    /// standing request. A candidate request is never used to choose these inputs.
    Maintenance,
    /// Both historical clock reads are named separately to preserve existing writer bytes.
    Onboarding {
        run_id: String,
        contract_generated_at: String,
        packet_generated_at: String,
        requested_tier: crate::runtime_follow_on::RequestedTier,
        minimal_justification_file: Option<String>,
        allow_rich_surface: Vec<String>,
    },
}
impl Observation {
    pub(crate) fn identity(&self) -> Result<String, String> {
        Ok(digest(
            b"support-depth-freeze-inputs-v1\0",
            &canonical(self)?,
        ))
    }
}

impl Writer {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if let Self::Onboarding {
            run_id,
            contract_generated_at,
            packet_generated_at,
            minimal_justification_file,
            ..
        } = self
        {
            crate::runtime_evidence_run::validate_run_id(run_id)?;
            for stamp in [contract_generated_at, packet_generated_at] {
                time::OffsetDateTime::parse(stamp, &time::format_description::well_known::Rfc3339)
                    .map_err(|_| "invalid authenticated freeze timestamp")?;
            }
            if let Some(path) = minimal_justification_file {
                crate::freeze_transition::tree::validate_path(path)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn writer_variables_require_valid_named_clocks_run_and_predecessor_relative_inputs() {
        let variables = Writer::Onboarding {
            run_id: "named-run".into(),
            contract_generated_at: "2026-11-14T00:00:00Z".into(),
            packet_generated_at: "2026-11-14T00:00:01Z".into(),
            requested_tier: crate::runtime_follow_on::RequestedTier::Default,
            minimal_justification_file: Some("docs/justification.md".into()),
            allow_rich_surface: vec![],
        };
        variables.validate().unwrap();
        for path in [
            "/tmp/justification.md",
            "../justification.md",
            "docs/../justification.md",
            ".git/config",
        ] {
            let mut changed = variables.clone();
            if let Writer::Onboarding {
                minimal_justification_file,
                ..
            } = &mut changed
            {
                *minimal_justification_file = Some(path.into());
            }
            assert!(changed.validate().is_err());
        }
        for field in 0..3 {
            let mut changed = variables.clone();
            if let Writer::Onboarding {
                run_id,
                contract_generated_at,
                packet_generated_at,
                ..
            } = &mut changed
            {
                match field {
                    0 => *run_id = "../run".into(),
                    1 => *contract_generated_at = "candidate clock".into(),
                    _ => *packet_generated_at = "candidate clock".into(),
                }
            }
            assert!(changed.validate().is_err());
        }
    }
}
