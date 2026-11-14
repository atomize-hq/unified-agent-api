//! Public ON1 starts the connected, credential-free onboarding chain.
use super::*;
use clap::Parser;

pub(super) fn approval_args() -> crate::onboard_agent::Args {
    crate::onboard_agent::Args::parse_from(["onboard-agent", "--write", "--approval", APPROVAL])
}

#[test]
fn public_on1_empty_root_scaffold_and_committed_replay() {
    let f = setup(false);
    let _installed = f.install();
    let parent = head(&f);
    let mut args = approval_args();
    args.write = false;
    args.dry_run = true;
    let mut preview = Vec::new();
    crate::onboard_agent::run_in_workspace(f.root.path(), args, &mut preview).unwrap();
    assert!(!f.root.path().join(ROOT).exists());
    assert!(!f.root.path().join(STATE).exists());
    let mut written = Vec::new();
    crate::onboard_agent::run_in_workspace(f.root.path(), approval_args(), &mut written).unwrap();
    let plan = |output: Vec<u8>| {
        String::from_utf8(output)
            .unwrap()
            .split_once("== INPUT SUMMARY ==")
            .unwrap()
            .1
            .split_once("== RESULT ==")
            .unwrap()
            .0
            .to_owned()
    };
    assert_eq!(plan(preview), plan(written));
    assert!(!f.root.path().join(RECORD).exists());
    assert!(!f
        .root
        .path()
        .join(format!("{ROOT}/versions/0.2.0.json"))
        .exists());
    let registry = AgentRegistry::load(f.root.path()).unwrap();
    let entry = registry.find("depth_fixture").unwrap();
    assert!(entry.backend_module.is_none());
    assert!(entry.support_depth.is_none());
    integrate(&f, &parent);
    let mut output = Vec::new();
    crate::onboard_agent::run_in_workspace(f.root.path(), approval_args(), &mut output).unwrap();
    assert!(String::from_utf8(output).unwrap().contains("0 written"));
    assert!(git(f.root.path(), &["status", "--porcelain=v1"]).is_empty());
}

fn raw_args() -> crate::onboard_agent::Args {
    let mut args = renderer::args();
    args.backend_module = None;
    args
}

#[test]
fn public_on1_raw_descriptor_uses_the_same_approval_bound_plan() {
    let f = setup(false);
    let _installed = f.install();
    let parent = head(&f);
    crate::onboard_agent::run_in_workspace(f.root.path(), raw_args(), &mut Vec::new()).unwrap();
    integrate(&f, &parent);
    let mut output = Vec::new();
    crate::onboard_agent::run_in_workspace(f.root.path(), approval_args(), &mut output).unwrap();
    assert!(String::from_utf8(output).unwrap().contains("0 written"));
    let state = life::load_lifecycle_state(f.root.path(), STATE).unwrap();
    assert_eq!(
        state.approval_artifact_sha256,
        hex::encode(Sha256::digest(
            fs::read(f.root.path().join(APPROVAL)).unwrap()
        ))
    );
    assert!(state.support_depth.is_none());
}

pub(super) fn canonical_context(f: &Fixture) {
    canonical_context_for(f, false);
}

fn canonical_context_for(f: &Fixture, producer: bool) {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let raw = fs::read_to_string(repo.join("Cargo.toml")).unwrap();
    let manifest: toml_edit::DocumentMut = raw.parse().unwrap();
    for member in manifest["workspace"]["members"].as_array().unwrap() {
        let path = format!("{}/Cargo.toml", member.as_str().unwrap());
        declaration::copy(&repo.join(&path), &f.root.path().join(path));
    }
    for path in [
        "Cargo.toml",
        "crates/agent_api/src",
        "cli_manifests",
        "docs/specs/unified-agent-api",
    ] {
        if producer && path == "crates/agent_api/src" {
            // Keep the reporting selectors used before ON2 restores full conformance source.
            for name in ["runtime_support.rs", "runtime_support_data.rs"] {
                let file = format!("{path}/{name}");
                declaration::copy(&repo.join(&file), &f.root.path().join(file));
            }
            continue;
        }
        declaration::copy(&repo.join(path), &f.root.path().join(path));
    }
    f.commit();
    local(f, &head(f));
}

fn validate(f: &Fixture) {
    assert_eq!(
        crate::manifest_validate::run(crate::manifest_validate::Args::parse_from([
            "manifest-validate",
            "--root",
            f.root.path().join(ROOT).to_str().unwrap(),
        ])),
        0
    );
}

pub(super) fn declared_chain() -> Fixture {
    declared_chain_for(false)
}

/// Producer fixtures need actual writer outputs; the ordinary connected tests above/below
/// separately prove IN1 admission/observation. Do not replay that unrelated proof during setup.
pub(super) fn declared_chain_for(producer: bool) -> Fixture {
    declared_chain_from(setup(false), producer)
}

/// Establish a full host commit identity before ON1 and the first retained IN1 decision.
pub(super) fn declared_chain_with_host_approval() -> Fixture {
    let f = setup(false);
    let raw = fs::read_to_string(f.root.path().join(APPROVAL))
        .unwrap()
        .replace("abc1234", &f.baseline);
    write(f.root.path(), APPROVAL, raw);
    f.commit();
    local(&f, &head(&f));
    declared_chain_from(f, false)
}

fn declared_chain_from(f: Fixture, producer: bool) -> Fixture {
    canonical_context_for(&f, producer);
    assert!(!f.root.path().join(ROOT).exists());
    let _installed = f.install();
    let checkpoint = if producer {
        producer_checkpoint
    } else {
        integrate
    };
    let approval = fs::read(f.root.path().join(APPROVAL)).unwrap();
    let parent = head(&f);
    crate::onboard_agent::run_in_workspace(f.root.path(), approval_args(), &mut Vec::new())
        .unwrap();
    checkpoint(&f, &parent);
    let parent = head(&f);
    adopt(&f);
    checkpoint(&f, &parent);
    let parent = head(&f);
    producer::acquire(f.root.path());
    checkpoint(&f, &parent);
    let parent = head(&f);
    metadata(&f);
    let next = declaration::publish(&f, &parent);
    if producer {
        f.host();
        local(&f, &next);
    } else {
        admit_and_observe(&f, &parent, &next);
    }
    validate(&f);
    assert!(AgentRegistry::load(f.root.path())
        .unwrap()
        .find("depth_fixture")
        .unwrap()
        .support_depth
        .is_none());
    let metadata: Value = serde_json::from_slice(
        &fs::read(f.root.path().join(format!("{ROOT}/versions/0.2.0.json"))).unwrap(),
    )
    .unwrap();
    assert_eq!(metadata["status"], "snapshotted");
    assert!(!f.root.path().join(RECORD).exists());
    let parent = head(&f);
    declaration::declare(&f);
    let next = declaration::publish(&f, &parent);
    if !producer {
        check_integration(f.root.path(), &parent).unwrap();
    }
    assert!(route(f.root.path())
        .unwrap()
        .unwrap()
        .registry(f.root.path())
        .unwrap()
        .find("depth_fixture")
        .unwrap()
        .support_depth
        .is_none());
    f.host();
    local(&f, &next);
    if !producer {
        check_integration(f.root.path(), &parent).unwrap();
    }
    let registry = crate::isolated_maintenance::registry(f.root.path()).unwrap();
    let entry = registry.find("depth_fixture").unwrap();
    assert!(entry.support_depth.is_some());
    assert_eq!(fs::read(f.root.path().join(APPROVAL)).unwrap(), approval);
    assert!(!f.root.path().join(RECORD).exists());
    assert!(life::load_lifecycle_state(f.root.path(), STATE)
        .unwrap()
        .support_depth
        .is_none());
    f
}

#[test]
fn public_on1_connected_preparation_metadata_and_first_declaration_chain() {
    let f = declared_chain();
    let _installed = f.install();
    let before = git(f.root.path(), &["status", "--porcelain=v1"]);
    for args in [approval_args(), raw_args()] {
        let error = crate::onboard_agent::run_in_workspace(f.root.path(), args, &mut Vec::new())
            .unwrap_err();
        assert!(error.to_string().contains("declared enrollment"), "{error}");
    }
    assert_eq!(git(f.root.path(), &["status", "--porcelain=v1"]), before);
}

fn admit_and_observe(f: &Fixture, parent: &str, next: &str) {
    check_integration(f.root.path(), parent).unwrap();
    f.host();
    local(f, next);
    check_integration(f.root.path(), parent).unwrap();
}

#[test]
fn public_on1_refuses_foreign_descriptors_and_mixed_approval_without_writes() {
    let f = setup(false);
    let _installed = f.install();
    let mut cases = Vec::new();
    for edit in 0..7 {
        let mut args = raw_args();
        match edit {
            0 => args.agent_id = Some("unknown".into()),
            1 => args.agent_id = Some("codex".into()),
            2 => args.backend_module = Some("crates/agent_api/src/backends/depth_fixture".into()),
            3 => args.canonical_targets = vec!["foreign-target".into()],
            4 => args.always_on_capabilities = vec!["agent_api.run".into()],
            5 => args.manifest_root = Some("cli_manifests/codex".into()),
            _ => args.approval = Some(APPROVAL.into()),
        }
        cases.push(args);
    }
    let mut args = approval_args();
    args.approval = Some("docs/agents/lifecycle/foreign/governance/approved-agent.toml".into());
    cases.push(args);
    for args in cases {
        let before = head(&f);
        assert!(
            crate::onboard_agent::run_in_workspace(f.root.path(), args, &mut Vec::new()).is_err()
        );
        assert_eq!(head(&f), before);
        assert!(git(f.root.path(), &["status", "--porcelain=v1"]).is_empty());
    }
}

#[test]
fn public_on1_refuses_divergent_scaffolds_and_retained_acceptance() {
    for edit in 0..5 {
        let f = setup(true);
        match edit {
            0 => write(
                f.root.path(),
                &format!("docs/agents/lifecycle/{PACK}/README.md"),
                "divergent scaffold\n",
            ),
            1 => write(f.root.path(), RECORD, "{}"),
            2 => write(
                f.root.path(),
                RECORD,
                r#"{"acceptance_entries":[{"kind":"Publication"}]}"#,
            ),
            3 => write(
                f.root.path(),
                &format!("{ROOT}/latest_validated.txt"),
                "0.2.0\n",
            ),
            _ => write(
                f.root.path(),
                REGISTRY_RELATIVE_PATH,
                format!(
                    "{}\n{DECLARED}",
                    include_str!("../../../../data/agent_registry.toml")
                ),
            ),
        }
        f.commit();
        local(&f, &head(&f));
        let _installed = f.install();
        let before = head(&f);
        assert!(
            crate::onboard_agent::run_in_workspace(f.root.path(), approval_args(), &mut Vec::new())
                .is_err(),
            "edit {edit}"
        );
        assert_eq!(head(&f), before);
        assert!(git(f.root.path(), &["status", "--porcelain=v1"]).is_empty());
    }
}

#[test]
fn public_on1_creation_admission_refuses_injected_table_record_and_entry() {
    for edit in 0..3 {
        let f = setup(false);
        let _installed = f.install();
        let parent = head(&f);
        crate::onboard_agent::run_in_workspace(f.root.path(), approval_args(), &mut Vec::new())
            .unwrap();
        match edit {
            0 => write(
                f.root.path(),
                REGISTRY_RELATIVE_PATH,
                format!(
                    "{}\n{DECLARED}",
                    include_str!("../../../../data/agent_registry.toml")
                ),
            ),
            1 => write(f.root.path(), RECORD, "{}"),
            _ => write(
                f.root.path(),
                RECORD,
                r#"{"acceptance_entries":[{"kind":"Publication"}]}"#,
            ),
        }
        candidate(&f, &parent);
        assert!(
            check_integration(f.root.path(), &parent).is_err(),
            "edit {edit}"
        );
        assert_eq!(
            git(f.root.path(), &["rev-parse", "refs/remotes/origin/staging"]),
            parent
        );
    }
}

#[test]
fn public_on1_absent_route_keeps_ordinary_success_and_depth_refusal() {
    let f = setup(false);
    assert!(route(f.root.path()).unwrap().is_none());
    assert!(crate::onboard_agent::run_in_workspace(
        f.root.path(),
        approval_args(),
        &mut Vec::new()
    )
    .is_err());
    assert!(!f.root.path().join(ROOT).exists());
    crate::onboard_agent::run_in_workspace(f.root.path(), renderer::args(), &mut Vec::new())
        .unwrap();
    let entry = AgentRegistry::load(f.root.path())
        .unwrap()
        .find("depth_fixture")
        .unwrap()
        .clone();
    assert_eq!(
        entry.backend_module.as_deref(),
        Some("crates/agent_api/src/backends/depth_fixture")
    );
    write(
        f.root.path(),
        REGISTRY_RELATIVE_PATH,
        format!(
            "{}\n{DECLARED}",
            include_str!("../../../../data/agent_registry.toml")
        ),
    );
    let before = fs::read(f.root.path().join(STATE)).unwrap();
    assert!(crate::onboard_agent::run_in_workspace(
        f.root.path(),
        renderer::args(),
        &mut Vec::new()
    )
    .is_err());
    assert_eq!(fs::read(f.root.path().join(STATE)).unwrap(), before);
}

#[test]
fn public_on1_maintenance_and_codex_controls_refuse_without_fallback() {
    for subject in [&scope::SYNTHETIC, &scope::CODEX] {
        let f = Fixture::new_for(subject);
        let _installed = f.install();
        for args in [approval_args(), renderer::args()] {
            assert!(
                crate::onboard_agent::run_in_workspace(f.root.path(), args, &mut Vec::new())
                    .is_err()
            );
            assert!(git(f.root.path(), &["status", "--porcelain=v1"]).is_empty());
        }
    }
}

struct DuringPreview<F>(Option<F>);
impl<F: FnOnce()> std::io::Write for DuringPreview<F> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if let Some(change) = self.0.take() {
            change();
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn public_on1_rechecks_host_and_preserves_concurrent_scaffold_before_delivery() {
    for host_change in [false, true] {
        let f = setup(false);
        let _installed = f.install();
        let mut writer = DuringPreview(Some(|| {
            if host_change {
                f.api.put(
                    "repos/fixture/isolated/branches/staging",
                    json!({
                        "name":"staging", "protected":true, "commit":{"sha":"f".repeat(40)}
                    }),
                );
            } else {
                write(
                    f.root.path(),
                    &format!("{ROOT}/current.json"),
                    "concurrent owner\n",
                );
            }
        }));
        assert!(crate::onboard_agent::run_in_workspace(
            f.root.path(),
            approval_args(),
            &mut writer
        )
        .is_err());
        assert!(!f.root.path().join(STATE).exists());
        assert!(AgentRegistry::load(f.root.path())
            .unwrap()
            .find("depth_fixture")
            .is_none());
        if !host_change {
            assert_eq!(
                fs::read_to_string(f.root.path().join(format!("{ROOT}/current.json"))).unwrap(),
                "concurrent owner\n"
            );
        }
    }
}

#[test]
fn public_on1_installed_approval_errors_never_fall_back_to_raw_descriptor() {
    for edit in 0..3 {
        let f = setup(false);
        let raw = fs::read_to_string(f.root.path().join(APPROVAL)).unwrap();
        let raw = match edit {
            0 => raw.replace("0.2.0", "0.3.0"),
            1 => raw.replace(
                "[descriptor]\n",
                "[descriptor]\nbackend_module = \"fake\"\n",
            ),
            _ => format!("{raw}\nunknown = true\n"),
        };
        write(f.root.path(), APPROVAL, raw);
        f.commit();
        let _installed = f.install();
        for args in [approval_args(), raw_args()] {
            assert!(
                crate::onboard_agent::run_in_workspace(f.root.path(), args, &mut Vec::new())
                    .is_err()
            );
            assert!(!f.root.path().join(ROOT).exists());
            assert!(git(f.root.path(), &["status", "--porcelain=v1"]).is_empty());
        }
    }
}

/// Pin actual public-writer output for producer fixture setup, without claiming IN1 admission.
pub(super) fn producer_checkpoint(f: &Fixture, parent: &str) {
    let next = candidate(f, parent);
    f.host();
    local(f, &next);
}
