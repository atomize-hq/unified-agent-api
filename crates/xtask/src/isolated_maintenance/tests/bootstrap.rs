//! Slice 1: credential-free scope/IN1 checkpoints; no public ON1 route or host queue proof.
use super::*;
use crate::{agent_lifecycle as life, depth_acceptance::check_integration};

use sha2::{Digest, Sha256};

mod declaration;
mod producer;
mod regressions;
mod renderer;
mod sweep;

const ROOT: &str = "cli_manifests/depth_fixture";
const PACK: &str = "depth-fixture-cli-onboarding";
const APPROVAL: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/approved-agent.toml";
const STATE: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/lifecycle-state.json";
const RECORD: &str = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";
const SCAFFOLD: &str =
    include_str!("../../../tests/fixtures/depth-synthetic/onboarding/registry-entry-scaffold.toml");
const DECLARED: &str =
    include_str!("../../../tests/fixtures/depth-synthetic/onboarding/registry-entry-deferred.toml");
const APPROVAL_TEMPLATE: &str = include_str!(
    "../../../tests/fixtures/depth-synthetic/onboarding/governance/approved-agent.toml"
);

fn setup(scaffold: bool) -> Fixture {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    write(
        f.root.path(),
        REGISTRY_RELATIVE_PATH,
        include_str!("../../../data/agent_registry.toml"),
    );
    write(
        f.root.path(),
        APPROVAL,
        APPROVAL_TEMPLATE
            .replace("{{APPROVAL_COMMIT}}", "abc1234")
            .replace("{{APPROVAL_RECORDED_AT}}", "2026-11-05T00:00:00Z"),
    );
    write(f.root.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
    write(f.root.path(), "docs/crates-io-release.md", "# Releases\n");
    if scaffold {
        create(&f);
    }
    f.commit();
    local(&f, &head(&f));
    f
}
fn create(f: &Fixture) {
    renderer::create(f.root.path());
}

fn head(f: &Fixture) -> String {
    git(f.root.path(), &["rev-parse", "HEAD"])
}
fn local(f: &Fixture, tip: &str) {
    git(
        f.root.path(),
        &["update-ref", "refs/remotes/origin/staging", tip],
    );
}
fn candidate(f: &Fixture, tip: &str) -> String {
    f.stage();
    git(
        f.root.path(),
        &["commit", "--allow-empty", "-m", "bootstrap candidate"],
    );
    let head = head(f);
    for base in [&f.baseline, &tip.to_owned()] {
        f.api.put(
            &format!("repos/fixture/isolated/compare/{base}...{head}"),
            json!({"status":"ahead","merge_base_commit":{"sha":base}}),
        );
    }
    f.api.put(
        &format!("repos/fixture/isolated/commits/{head}"),
        json!({"sha":head}),
    );
    head
}
fn integrate(f: &Fixture, parent: &str) {
    let next = candidate(f, parent);
    check_integration(f.root.path(), parent).unwrap();
    f.host();
    local(f, &next);
    check_integration(f.root.path(), parent).unwrap();
}
fn adopt(f: &Fixture) {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/depth-synthetic/governing/manifest-root");
    fn copy(src: &Path, dst: &Path) {
        if src.is_dir() {
            for entry in fs::read_dir(src).unwrap() {
                let entry = entry.unwrap();
                copy(&entry.path(), &dst.join(entry.file_name()));
            }
        } else {
            fs::create_dir_all(dst.parent().unwrap()).unwrap();
            fs::copy(src, dst).unwrap();
        }
    }
    copy(&source, &f.root.path().join(ROOT));
    let rules = fs::read_to_string(f.root.path().join(format!("{ROOT}/RULES.json"))).unwrap();
    write(
        f.root.path(),
        &format!("{ROOT}/RULES.json"),
        rules
            .replace("{{ISOLATED_OWNER}}", "fixture")
            .replace("{{ISOLATED_REPO}}", "isolated"),
    );
}
fn metadata(f: &Fixture) {
    crate::manifest_version_metadata::run(crate::manifest_version_metadata::Args {
        root: Some(f.root.path().join(ROOT)),
        rules: None,
        version: "0.2.0".into(),
        status: crate::manifest_version_metadata::Status::Snapshotted,
        passed_targets: vec![],
        failed_targets: vec![],
        skipped_targets: vec![],
    })
    .unwrap();
}

#[test]
fn bootstrap_absent_and_unselected_subject_require_exact_approval_and_scaffold() {
    for scaffold in [false, true] {
        let f = setup(scaffold);
        let _installed = f.install();
        let context = f.resolve().unwrap();
        assert_eq!(
            context
                .registry(f.root.path())
                .unwrap()
                .find("depth_fixture")
                .is_some(),
            scaffold
        );
        for edit in ["backend_module = \"fake\"\n", "unknown = true\n"] {
            let raw = fs::read_to_string(f.root.path().join(APPROVAL)).unwrap();
            write(f.root.path(), APPROVAL, format!("{raw}{edit}"));
            f.commit();
            assert!(registry(f.root.path()).is_err());
            write(f.root.path(), APPROVAL, raw);
            f.commit();
        }
    }
}
#[test]
fn bootstrap_scope_refuses_records_foreign_descriptor_and_unknown_scope_without_fallback() {
    for edit in 0..7 {
        let f = setup(true);
        match edit {
            0 => write(f.root.path(), RECORD, "{}"),
            1 => write(f.root.path(), "reports/orphan/depth-record.json", "{}"),
            2 => {
                let raw = fs::read_to_string(f.root.path().join(APPROVAL)).unwrap();
                write(
                    f.root.path(),
                    APPROVAL,
                    raw.replace("Depth Fixture", "Foreign"),
                );
            }
            3 => {
                let raw = fs::read_to_string(f.root.path().join(APPROVAL)).unwrap();
                write(f.root.path(), APPROVAL, raw.replace("0.2.0", "0.3.0"));
            }
            4 => f.registry(|d| {
                let a = d["agents"]
                    .as_array_of_tables_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|a| a["agent_id"].as_str() == Some("depth_fixture"))
                    .unwrap();
                a["display_name"] = toml_edit::value("Foreign");
            }),
            5 => {
                let raw = fs::read_to_string(f.root.path().join(APPROVAL)).unwrap();
                write(
                    f.root.path(),
                    APPROVAL,
                    format!("{raw}\n[foreign]\npermission = true\n"),
                );
            }
            _ => {
                fs::remove_file(f.root.path().join(APPROVAL)).unwrap();
            }
        }
        f.commit();
        let _installed = f.install();
        assert!(
            matches!(registry(f.root.path()), Err(RegistryError::Isolated(_))),
            "edit {edit}"
        );
    }
    let f = setup(true);
    let mut f = f;
    f.subject = &scope::CODEX;
    assert!(f.resolve().is_err(), "Codex cannot enter bootstrap");
    let f = setup(false);
    write(
        f.root.path(),
        &format!("{ROOT}/unexpected"),
        "preexisting root",
    );
    f.commit();
    assert!(f.resolve().is_err());
}
#[test]
fn bootstrap_creation_is_admitted_before_tip_and_observed_after_tip() {
    let f = setup(false);
    let _installed = f.install();
    let parent = head(&f);
    create(&f);
    let next = candidate(&f, &parent);
    // Candidate registry is not yet captured as protected governing data.
    assert!(route(f.root.path())
        .unwrap()
        .unwrap()
        .registry(f.root.path())
        .unwrap()
        .find("depth_fixture")
        .is_none());
    check_integration(f.root.path(), &parent).unwrap();
    f.host();
    local(&f, &next);
    check_integration(f.root.path(), &parent).unwrap();
    assert!(route(f.root.path())
        .unwrap()
        .unwrap()
        .registry(f.root.path())
        .unwrap()
        .find("depth_fixture")
        .unwrap()
        .support_depth
        .is_none());
    assert!(!f.root.path().join(RECORD).exists());
}
#[test]
fn preparation_checkpoints_admit_governing_acquisition_and_ne3_then_observe_each_tip() {
    let f = setup(true);
    let _installed = f.install();
    let parent = head(&f);
    adopt(&f);
    integrate(&f, &parent);
    let parent = head(&f);
    producer::acquire(f.root.path());
    integrate(&f, &parent);
    let parent = head(&f);
    metadata(&f);
    integrate(&f, &parent);
    assert!(!f.root.path().join(RECORD).exists());
}
#[test]
fn preparation_refuses_every_binding_and_acceptance_effect() {
    for edit in 0..15 {
        let f = setup(true);
        adopt(&f);
        producer::acquire(f.root.path());
        metadata(&f);
        f.commit();
        let parent = head(&f);
        local(&f, &parent);
        match edit {
            0 => write(f.root.path(), RECORD, "{}"),
            1 => write(f.root.path(), &format!("{ROOT}/latest_validated.txt"), "0.2.0\n"),
            2 | 3 => {
                let path = format!("{ROOT}/versions/0.2.0.json");
                let mut v: Value = serde_json::from_slice(&fs::read(f.root.path().join(&path)).unwrap()).unwrap();
                if edit == 2 { v["status"] = json!("validated"); } else { v["validation"] = json!({"passed_targets":["x86_64-unknown-linux-gnu"]}); }
                write(f.root.path(), &path, serde_json::to_vec(&v).unwrap());
            },
            4 => { let raw=fs::read_to_string(f.root.path().join(APPROVAL)).unwrap(); write(f.root.path(), APPROVAL, raw.replace("abc1234","abc5678")); },
            5 => f.registry(|d| { d["agents"].as_array_of_tables_mut().unwrap().iter_mut().find(|e| e["agent_id"].as_str()==Some("depth_fixture")).unwrap()["display_name"]=toml_edit::value("Changed"); }),
            6 => write(f.root.path(), REGISTRY_RELATIVE_PATH, format!("{}\n{DECLARED}", include_str!("../../../data/agent_registry.toml"))),
            7 => { let mut v:Value = serde_json::from_slice(&fs::read(f.root.path().join(STATE)).unwrap()).unwrap(); v["support_depth"]=json!({"policy_identity":"a".repeat(64)}); write(f.root.path(),STATE,serde_json::to_vec(&v).unwrap()); },
            8 => write(f.root.path(), "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/publication-ready.json", "{}"),
            9 => write(f.root.path(), "cli_manifests/codex/versions/0.156.1.json", r#"{"status":"validated"}"#),
            10 => write(f.root.path(), "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/proving-run-closeout.json", r#"{"state":"closed"}"#),
            12 => {
                let raw = fs::read_to_string(f.root.path().join(APPROVAL)).unwrap();
                write(f.root.path(), APPROVAL, format!("{raw}\n# changed approval bytes\n"));
            },
            13 => {
                let raw = fs::read_to_string(f.root.path().join(REGISTRY_RELATIVE_PATH)).unwrap();
                write(f.root.path(), REGISTRY_RELATIVE_PATH, format!("{raw}\n# changed registry bytes\n"));
            },
            14 => write(f.root.path(), &format!("{ROOT}/versions/0.3.0.json"), r#"{"schema_version":1,"semantic_version":"0.3.0","status":"snapshotted"}"#),
            _ => { let path=format!("{ROOT}/RULES.json"); let raw=fs::read_to_string(f.root.path().join(&path)).unwrap(); write(f.root.path(), &path,raw.replace("depth-fixture-cli-parity","unreviewed")); },
        }
        candidate(&f, &parent);
        let _installed = f.install();
        assert!(
            check_integration(f.root.path(), &parent).is_err(),
            "edit {edit}"
        );
    }
}

#[test]
fn finite_approval_projection_rejects_unknown_misplaced_backend_and_foreign_fields() {
    let entry = AgentRegistry::parse_unenforced(SCAFFOLD)
        .unwrap()
        .agents
        .remove(0);
    let raw = APPROVAL_TEMPLATE
        .replace("{{APPROVAL_COMMIT}}", "abc1234")
        .replace("{{APPROVAL_RECORDED_AT}}", "2026-11-05T00:00:00Z");
    let read = |raw: &str| onboarding::approval::read(raw.as_bytes(), APPROVAL, &entry);
    read(&raw).unwrap();
    for edit in 0..9 {
        let mut doc: toml_edit::DocumentMut = raw.parse().unwrap();
        match edit {
            0 => doc["unknown"] = toml_edit::value(true),
            1 => doc["descriptor"]["unknown"] = toml_edit::value(true),
            2 => doc["descriptor"]["backend_module"] = toml_edit::value("fake"),
            3 => doc["descriptor"]["maintenance"]["deferral"]["unknown"] = toml_edit::value(true),
            4 => doc["support_depth"]["misplaced"] = toml_edit::value(true),
            5 => doc["descriptor"]["canonical_targets"][0] = toml_edit::value("foreign-target"),
            6 => doc["descriptor"]["agent_id"] = toml_edit::value("codex"),
            7 => {
                doc["descriptor"]["maintenance"]["deferral"]["approved_scope"] =
                    toml_edit::value("all_acceptance")
            }
            _ => {
                doc["descriptor"]["always_on_capabilities"] =
                    toml_edit::value(["agent_api.run"].into_iter().collect::<toml_edit::Array>())
            }
        }
        assert!(read(&doc.to_string()).is_err(), "approval edit {edit}");
    }
}
#[test]
fn bootstrap_route_absence_keeps_production_depth_refusal() {
    let f = setup(true);
    write(
        f.root.path(),
        REGISTRY_RELATIVE_PATH,
        format!(
            "{}\n{DECLARED}",
            include_str!("../../../data/agent_registry.toml")
        ),
    );
    f.commit();
    assert!(route(f.root.path()).unwrap().is_none());
    assert!(matches!(
        registry(f.root.path()),
        Err(RegistryError::Production(_))
    ));
    assert!(crate::approval_artifact::load_approval_artifact(f.root.path(), APPROVAL).is_err());
}
#[test]
fn bootstrap_scratch_view_keeps_exact_registry_bytes() {
    let f = setup(true);
    let captured = Route(f.resolve().unwrap()).capture(f.root.path()).unwrap();
    let proposal = captured.proposal(f.root.path()).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal
        .isolated_view(f.root.path(), scratch.path())
        .unwrap();
    let binding = captured.bind_view(&view).unwrap();
    drop(binding);
    write(
        scratch.path(),
        REGISTRY_RELATIVE_PATH,
        format!(
            "{}\n{DECLARED}",
            include_str!("../../../data/agent_registry.toml")
        ),
    );
    assert!(captured.bind_view(&view).is_err());
}

#[test]
fn bootstrap_creation_refuses_depth_tables_invented_metadata_and_non_scaffold_files() {
    for edit in 0..6 {
        let f = setup(false);
        let parent = head(&f);
        create(&f);
        match edit {
            0 => write(
                f.root.path(),
                REGISTRY_RELATIVE_PATH,
                format!(
                    "{}\n{DECLARED}",
                    include_str!("../../../data/agent_registry.toml")
                ),
            ),
            1 => write(
                f.root.path(),
                &format!("{ROOT}/versions/0.2.0.json"),
                r#"{"schema_version":1,"semantic_version":"0.2.0","status":"snapshotted"}"#,
            ),
            2 => write(
                f.root.path(),
                &format!("{ROOT}/RULES.json"),
                r#"{"storage":{"pointers":{"compat":{"latest_validated":"cli_manifests/codex/latest_validated.txt"}}}}"#,
            ),
            3 => write(
                f.root.path(),
                &format!("{ROOT}/wrapper_coverage.json"),
                r#"{"schema_version":1,"coverage":[{"path":["run"]}]}"#,
            ),
            5 => {
                let mut state: Value =
                    serde_json::from_slice(&fs::read(f.root.path().join(STATE)).unwrap()).unwrap();
                let future = serde_json::to_value(life::required_evidence_for_stage(
                    life::LifecycleStage::ClosedBaseline,
                ))
                .unwrap();
                state["required_evidence"] = future.clone();
                state["satisfied_evidence"] = future;
                write(f.root.path(), STATE, serde_json::to_vec(&state).unwrap());
            }
            _ => write(f.root.path(), "reports/orphan/depth-record.json", "{}"),
        }
        candidate(&f, &parent);
        let _installed = f.install();
        assert!(
            check_integration(f.root.path(), &parent).is_err(),
            "creation edit {edit}"
        );
    }
}

#[test]
fn preparation_rename_cannot_hide_an_unrelated_file_deletion() {
    let f = setup(true);
    let reviewed =
        include_str!("../../../tests/fixtures/depth-synthetic/governing/manifest-root/RULES.json")
            .replace("{{ISOLATED_OWNER}}", "fixture")
            .replace("{{ISOLATED_REPO}}", "isolated");
    write(f.root.path(), "docs/unrelated-rules.json", reviewed);
    f.commit();
    let parent = head(&f);
    local(&f, &parent);
    fs::create_dir_all(f.root.path().join(ROOT)).unwrap();
    fs::rename(
        f.root.path().join("docs/unrelated-rules.json"),
        f.root.path().join(format!("{ROOT}/RULES.json")),
    )
    .unwrap();
    candidate(&f, &parent);
    let _installed = f.install();
    let error = check_integration(f.root.path(), &parent).unwrap_err();
    assert!(
        error.contains("unauthorized effect: docs/unrelated-rules.json"),
        "{error}"
    );
}

#[test]
fn bootstrap_checkpoint_cannot_be_reused_for_changed_candidate_or_worktree() {
    let f = setup(true);
    let parent = head(&f);
    adopt(&f);
    candidate(&f, &parent);
    let context = f.resolve().unwrap();
    let checkpoint = context
        .bootstrap_checkpoint(f.root.path(), &parent)
        .unwrap()
        .unwrap();
    checkpoint.with_registry(f.root.path(), || Ok(())).unwrap();
    let foreign = tempfile::tempdir().unwrap();
    assert!(checkpoint.with_registry(foreign.path(), || Ok(())).is_err());
    write(f.root.path(), "docs/concurrent.txt", "changed worktree");
    assert!(checkpoint.with_registry(f.root.path(), || Ok(())).is_err());
    fs::remove_file(f.root.path().join("docs/concurrent.txt")).unwrap();
    checkpoint.with_registry(f.root.path(), || Ok(())).unwrap();
    candidate(&f, &parent);
    assert!(checkpoint.with_registry(f.root.path(), || Ok(())).is_err());
}
