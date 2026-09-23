//! T5 — closeout finding derivation.
//!
//! The acceptance condition is not "the derivation looks right", it is "the frozen validator
//! accepts what the derivation produced". Both halves are driven from one drift report here, so a
//! derivation that drifts from the validator's contract fails rather than agreeing with itself.

use super::*;

use crate::closeout::{derive_findings, derive_findings_from_report, DeferredFindingsTruth};

const AGENT: &str = "opencode";
const VERSION: &str = "1.14.47";
const CLOSEOUT_PATH: &str =
    "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json";

fn clean_report() -> drift::AgentDriftReport {
    drift::AgentDriftReport {
        agent_id: AGENT.to_string(),
        findings: vec![],
    }
}

fn report_with(category: drift::DriftCategory, surfaces: &[&str]) -> drift::AgentDriftReport {
    drift::AgentDriftReport {
        agent_id: AGENT.to_string(),
        findings: vec![drift::DriftFinding {
            category,
            summary: "Live drift is present.".to_string(),
            surfaces: surfaces.iter().map(|s| (*s).to_string()).collect(),
        }],
    }
}

/// Place a derivation into an otherwise valid closeout, so the validator sees exactly what T5 made.
fn closeout_from(
    derived: crate::closeout::DerivedFindings,
) -> crate::closeout::MaintenanceCloseout {
    let mut closeout = valid_closeout_struct(
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    closeout.resolved_findings = derived.resolved_findings;
    closeout.deferred_findings = derived.deferred_findings;
    closeout
}

/// The acceptance condition, clean case.
#[test]
fn a_clean_report_derives_explicit_none_that_the_validator_accepts() {
    let fixture = fixture_root("t5-clean");
    seed_opencode_basis(&fixture);
    let live = clean_report();

    let derived = derive_findings_from_report(&fixture, AGENT, VERSION, &live)
        .expect("a clean report derives");
    match &derived.deferred_findings {
        DeferredFindingsTruth::ExplicitNone(reason) => {
            assert!(reason.contains("status: clean"), "{reason}");
            assert!(
                reason.contains("not carried forward from the request"),
                "{reason}"
            );
        }
        other => panic!("a clean report must derive explicit-none, got {other:?}"),
    }

    validate_live_drift_report(
        Path::new(CLOSEOUT_PATH),
        AGENT,
        &closeout_from(derived),
        Ok(live),
    )
    .expect("the validator accepts what T5 derived");
}

/// The acceptance condition, drift case. The validator requires the deferred set to equal the live
/// set in both directions, so transcription is the only derivation that can pass.
#[test]
fn a_drifting_report_derives_deferred_findings_the_validator_accepts() {
    let fixture = fixture_root("t5-drift");
    seed_opencode_basis(&fixture);
    let live = report_with(
        drift::DriftCategory::GovernanceDoc,
        &[
            "docs/integrations/opencode/governance/seam-2-closeout.md",
            "docs/agents/lifecycle/opencode-maintenance/HANDOFF.md",
        ],
    );

    let derived =
        derive_findings_from_report(&fixture, AGENT, VERSION, &live).expect("drift derives");
    match &derived.deferred_findings {
        DeferredFindingsTruth::Findings(findings) => {
            assert_eq!(findings.len(), 1);
            assert_eq!(
                findings[0].category_id,
                crate::closeout::MaintenanceDriftCategory::GovernanceDoc
            );
            assert_eq!(findings[0].surfaces.len(), 2);
        }
        other => panic!("live drift must derive deferred findings, got {other:?}"),
    }

    validate_live_drift_report(
        Path::new(CLOSEOUT_PATH),
        AGENT,
        &closeout_from(derived),
        Ok(live),
    )
    .expect("the validator accepts what T5 derived");
}

/// `runtime_evidence_drift` exists in `DriftCategory` and not in `MaintenanceDriftCategory`. The
/// validator demands every live finding appear in `deferred_findings` and rejects that category id,
/// so the packet has no valid closeout. Emitting one anyway would be a generator writing a file it
/// knows the validator rejects.
#[test]
fn a_live_category_the_schema_cannot_express_refuses_rather_than_emitting_it() {
    let fixture = fixture_root("t5-inexpressible");
    seed_opencode_basis(&fixture);
    let live = report_with(
        drift::DriftCategory::RuntimeEvidence,
        &["docs/agents/lifecycle/opencode-cli-onboarding/governance/lifecycle-state.json"],
    );

    let err = derive_findings_from_report(&fixture, AGENT, VERSION, &live)
        .expect_err("an inexpressible category must refuse")
        .to_string();
    assert!(err.contains("runtime_evidence_drift"), "{err}");
    assert!(err.contains("cannot express"), "{err}");
    assert!(err.contains("do not close around it"), "{err}");
}

/// The validator compares whole signatures, and the live surfaces are whatever drifted while the
/// resolved ones are the write envelope — so it would not catch this. T5 refuses on the category.
#[test]
fn claiming_a_category_resolved_while_live_drift_reports_it_refuses() {
    let fixture = fixture_root("t5-contradiction");
    seed_opencode_basis(&fixture);
    let live = report_with(
        drift::DriftCategory::RegistryManifest,
        &["cli_manifests/opencode/artifacts.lock.json"],
    );

    let err = derive_findings_from_report(&fixture, AGENT, VERSION, &live)
        .expect_err("a category cannot be both fixed and outstanding")
        .to_string();
    assert!(err.contains("registry_manifest_drift"), "{err}");
    assert!(err.contains("not finished"), "{err}");
}

/// Every claimed surface must exist. A finding over surfaces that were never written satisfies the
/// validator's "absent from live drift" check vacuously.
#[test]
fn only_surfaces_that_exist_are_claimed() {
    let fixture = fixture_root("t5-surfaces-exist");
    seed_opencode_basis(&fixture);

    let derived =
        derive_findings_from_report(&fixture, AGENT, VERSION, &clean_report()).expect("derives");
    assert!(!derived.resolved_findings.is_empty());
    for finding in &derived.resolved_findings {
        assert!(!finding.surfaces.is_empty(), "{finding:?}");
        for surface in &finding.surfaces {
            assert!(
                fixture.join(surface).is_file(),
                "claimed a surface that does not exist: {surface}"
            );
        }
    }
    let registry = derived
        .resolved_findings
        .iter()
        .find(|f| f.category_id == crate::closeout::MaintenanceDriftCategory::RegistryManifest)
        .expect("registry manifest finding");
    assert!(
        registry
            .surfaces
            .iter()
            .any(|s| s.starts_with(&format!("cli_manifests/opencode/reports/{VERSION}/"))),
        "{:?}",
        registry.surfaces
    );
}

/// Two of three live packets have no report on `staging` for their target version, so this is the
/// message a maintainer on the wrong branch actually reaches.
#[test]
fn no_version_scoped_artifact_refuses_and_names_both_causes() {
    let fixture = fixture_root("t5-no-artifacts");
    seed_opencode_basis(&fixture);

    let err = derive_findings_from_report(&fixture, AGENT, "9.9.9", &clean_report())
        .expect_err("a version with no artifacts must refuse")
        .to_string();
    assert!(err.contains("no version-scoped manifest artifact"), "{err}");
    assert!(err.contains("wrong checkout"), "{err}");
    assert!(err.contains("never completed"), "{err}");
}

#[test]
fn a_live_finding_with_no_surfaces_refuses() {
    let fixture = fixture_root("t5-no-surfaces");
    seed_opencode_basis(&fixture);
    let live = report_with(drift::DriftCategory::GovernanceDoc, &[]);

    let err = derive_findings_from_report(&fixture, AGENT, VERSION, &live)
        .expect_err("a surfaceless finding must refuse")
        .to_string();
    assert!(err.contains("must not be empty"), "{err}");
}

/// An error is not a clean report. Falling through to `explicit_none_reason` would record "nothing
/// is outstanding" on the strength of not having looked.
#[test]
fn a_drift_check_that_cannot_run_refuses_instead_of_recording_none() {
    let fixture = fixture_root("t5-unrunnable");
    let err = derive_findings(&fixture, AGENT, VERSION)
        .expect_err("an unrunnable drift check must refuse")
        .to_string();
    assert!(!err.contains("status: clean"), "{err}");
}

#[test]
fn an_unknown_agent_refuses_rather_than_deriving_an_empty_envelope() {
    let fixture = fixture_root("t5-unknown-agent");
    seed_opencode_basis(&fixture);
    let live = drift::AgentDriftReport {
        agent_id: "nosuchagent".to_string(),
        findings: vec![],
    };

    let err = derive_findings_from_report(&fixture, "nosuchagent", VERSION, &live)
        .expect_err("an unknown agent must refuse")
        .to_string();
    assert!(err.contains("not in the agent registry"), "{err}");
}

/// The write envelope comes from the registry entry, not from a path list that happens to match
/// opencode. Each agent's surfaces must sit under its own `manifest_root` — the failure this
/// catches is a derivation that is correct for the agent it was written against and silently wrong
/// for the other two.
#[test]
fn each_agent_derives_surfaces_under_its_own_manifest_root() {
    let fixture = fixture_root("t5-per-agent-envelope");
    seed_opencode_basis(&fixture);

    for (agent, version, manifest_root) in [
        ("codex", "1.0.0", "cli_manifests/codex"),
        ("claude_code", "1.0.0", "cli_manifests/claude_code"),
        (AGENT, VERSION, "cli_manifests/opencode"),
    ] {
        let live = drift::AgentDriftReport {
            agent_id: agent.to_string(),
            findings: vec![],
        };
        let derived = derive_findings_from_report(&fixture, agent, version, &live)
            .unwrap_or_else(|err| panic!("{agent} must derive: {err}"));

        let registry = derived
            .resolved_findings
            .iter()
            .find(|f| f.category_id == crate::closeout::MaintenanceDriftCategory::RegistryManifest)
            .unwrap_or_else(|| panic!("{agent}: no registry manifest finding"));
        for surface in &registry.surfaces {
            assert!(
                surface.starts_with(&format!("{manifest_root}/")),
                "{agent} claimed `{surface}` outside `{manifest_root}`"
            );
            assert!(fixture.join(surface).is_file(), "{agent}: {surface}");
        }
        assert!(
            registry
                .surfaces
                .iter()
                .any(|s| s == &format!("{manifest_root}/versions/{version}.json")),
            "{agent}: {:?}",
            registry.surfaces
        );
    }
}
