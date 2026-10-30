//! Untrusted finite observations. Only private native admission can make a verdict.
use super::{CaseIdentity, CaseOutcome, CaseResult, FileRef};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[path = "../../../../codex/src/native_context/projection.rs"]
mod projection;
pub(crate) use projection::environment_known;
pub use projection::{Approval, Config, Launch, Route, Sandbox};
pub const TRACE_TARGET: &str = projection::TARGET;
pub(crate) const INPUTS: [&str; 2] = [
    "cli_manifests/codex/reports/0.156.1/pilot-runtime-context.json",
    "cli_manifests/codex/reports/0.156.1/pilot-protocol-facts.json",
];
pub mod guard;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fact {
    LayerInputs,
    AuthenticationResolver,
    TransitiveRuntime,
    EffectiveTurn,
    LoaderEffects,
    MutationScope,
}
impl Fact {
    pub const ALL: [Self; 6] = [
        Self::LayerInputs,
        Self::AuthenticationResolver,
        Self::TransitiveRuntime,
        Self::EffectiveTurn,
        Self::LoaderEffects,
        Self::MutationScope,
    ];
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub fact: Fact,
    pub source: FileRef,
    pub corroboration: CaseIdentity,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefaultId {
    ToolRegistryErrorOnCollisions,
    ToolRegistryTurnMetadata,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerivedDefault {
    pub id: DefaultId,
    pub value: bool,
    pub source: FileRef,
    pub upstream_commit: String,
    pub binary_sha256: String,
    pub overrides_absent: bool,
    pub precedence_established: bool,
    pub corroboration: CaseIdentity,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseContext {
    pub case: CaseIdentity,
    pub route: Route,
    pub local_before: bool,
    pub local_after: bool,
    pub launches: Vec<Launch>,
    #[serde(deserialize_with = "projection::required_nullable")]
    pub config: Option<Config>,
    pub bindings: Vec<Binding>,
    pub missing: Vec<Fact>,
    pub derived_defaults: Vec<DerivedDefault>,
    pub diagnostics_valid: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub schema_version: u64,
    pub complete: bool,
    pub cases: Vec<CaseContext>,
}
// Source-owned by private Profile, not constructed from the receipt or CLI intent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExpectedCase {
    pub case: CaseIdentity,
    pub route: Route,
    pub config: Option<Config>,
    pub bindings: Vec<Binding>,
    pub derived_defaults: Vec<DerivedDefault>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Expectations {
    pub cases: Vec<ExpectedCase>,
}
impl Observation {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > 1_048_576 {
            return Err("native context exceeds bound".into());
        }
        let v: Self = serde_json::from_slice(bytes).map_err(|_| "invalid finite native context")?;
        if v.schema_version != 1
            || v.cases.len() > 4096
            || v.cases.iter().any(|c| {
                c.launches.len() > 2
                    || c.bindings.len() > 6
                    || c.missing.len() > 6
                    || c.derived_defaults.len() > 2
            })
        {
            return Err("invalid finite native context".into());
        }
        let cases: BTreeSet<_> = v.cases.iter().map(|c| &c.case).collect();
        if cases.len() != v.cases.len() {
            return Err("duplicate native context case".into());
        }
        Ok(v)
    }
    pub fn incomplete() -> Self {
        Self {
            schema_version: 1,
            complete: false,
            cases: vec![],
        }
    }
    pub(crate) fn validate(
        &self,
        expected: &Expectations,
        requested: &[CaseIdentity],
        executed: &[CaseResult],
        approved: &[FileRef],
        binary: &str,
    ) -> Result<(), String> {
        let fail = || "native context is incomplete or differs from approved facts".to_owned();
        if Self::parse(&serde_json::to_vec(self).map_err(|_| fail())?)? != *self {
            return Err(fail());
        }
        let set: BTreeSet<_> = requested.iter().collect();
        if set.len() != requested.len()
            || set.is_empty()
            || expected.cases.len() != set.len()
            || self.cases.len() != set.len()
            || expected
                .cases
                .iter()
                .map(|c| &c.case)
                .collect::<BTreeSet<_>>()
                != set
            || self.cases.iter().map(|c| &c.case).collect::<BTreeSet<_>>() != set
        {
            return Err(fail());
        }
        for c in &self.cases {
            if !c.missing.is_empty()
                || c.config.as_ref().is_some_and(|p| {
                    !p.model_gpt5
                        || !p.provider_openai
                        || p.approval.is_none()
                        || p.sandbox.is_none()
                        || !p.origins_present
                        || !p.requirements_empty
                })
            {
                return Err(fail());
            }
            let e = expected
                .cases
                .iter()
                .find(|e| e.case == c.case)
                .ok_or_else(fail)?;
            if c.route != e.route
                || !c.local_before
                || !c.local_after
                || !c.diagnostics_valid
                || c.config != e.config
                || c.launches.len() != 2
                || c.launches[0].spawned
                || !c.launches[1].spawned
                || c.launches.iter().any(|l| {
                    l.route != c.route
                        || !l.binary_matches
                        || !l.cwd_matches
                        || !l.home_matches
                        || !l.environment_known
                })
                || c.bindings != e.bindings
                || c.derived_defaults != e.derived_defaults
            {
                return Err(fail());
            }
            let facts: BTreeSet<_> = c.bindings.iter().map(|b| b.fact).collect();
            if facts != Fact::ALL.into_iter().collect() || facts.len() != c.bindings.len() {
                return Err(fail());
            }
            let passed = |case: &CaseIdentity| {
                executed
                    .iter()
                    .any(|r| r.case == *case && r.outcome == CaseOutcome::Passed)
                    && !executed
                        .iter()
                        .any(|r| r.case == *case && r.outcome != CaseOutcome::Passed)
            };
            for b in &c.bindings {
                if !approved.contains(&b.source)
                    || !set.contains(&b.corroboration)
                    || !executed.iter().any(|r| r.case == b.corroboration)
                {
                    return Err(fail());
                }
            }
            let mut defaults = BTreeSet::new();
            for d in &c.derived_defaults {
                if !defaults.insert(format!("{:?}", d.id))
                    || d.value
                    || !d.overrides_absent
                    || !d.precedence_established
                    || d.upstream_commit != "b412ff32c417f855c2b2d1581b77058eed87c84b"
                    || d.binary_sha256 != binary
                    || !approved.contains(&d.source)
                    || !set.contains(&d.corroboration)
                    || !passed(&d.corroboration)
                {
                    return Err(fail());
                }
            }
        }
        // The claimed flag cannot substitute for any predicate above.
        if !self.complete {
            return Err(fail());
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) fn synthetic(cases: &[CaseIdentity], source: &FileRef) -> (Observation, Expectations) {
    let observations = cases
        .iter()
        .map(|case| {
            let route = if case.name.contains("interactive/") && !case.name.contains("unattended/")
                || case.name.contains("fork-")
            {
                Route::AppServer
            } else if case.name.contains("resume-") {
                Route::ResumeExec
            } else {
                Route::InitialExec
            };
            let launch = Launch {
                route,
                spawned: false,
                binary_matches: true,
                cwd_matches: true,
                home_matches: true,
                environment_known: true,
            };
            CaseContext {
                case: case.clone(),
                route,
                local_before: true,
                local_after: true,
                launches: vec![
                    launch.clone(),
                    Launch {
                        spawned: true,
                        ..launch
                    },
                ],
                config: None,
                bindings: Fact::ALL
                    .into_iter()
                    .map(|fact| Binding {
                        fact,
                        source: source.clone(),
                        corroboration: case.clone(),
                    })
                    .collect(),
                missing: vec![],
                derived_defaults: vec![],
                diagnostics_valid: true,
            }
        })
        .collect::<Vec<_>>();
    let expected = Expectations {
        cases: observations
            .iter()
            .map(|c| ExpectedCase {
                case: c.case.clone(),
                route: c.route,
                config: c.config.clone(),
                bindings: c.bindings.clone(),
                derived_defaults: vec![],
            })
            .collect(),
    };
    (
        Observation {
            schema_version: 1,
            complete: true,
            cases: observations,
        },
        expected,
    )
}
