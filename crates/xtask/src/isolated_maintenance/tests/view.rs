//! CP-P2: scratch borrows the caller's registry and Git inputs, scoped to one thread and guard.
#![cfg(unix)]
use super::*;

fn captured(f: &Fixture) -> CapturedRegistry {
    Route(f.resolve().unwrap()).capture(f.root.path()).unwrap()
}

#[test]
fn binding_is_scoped_to_the_scratch_root_thread_and_guard() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let _installed = f.install();
    let captured = captured(&f);
    let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal
        .isolated_view(f.root.path(), scratch.path())
        .unwrap();
    assert!(route(scratch.path()).is_err());
    assert!(registry(scratch.path()).is_err());
    let guard = captured.bind_view(&view).unwrap();
    assert_eq!(registry(scratch.path()).unwrap(), *captured.registry());
    assert!(route(scratch.path()).is_err());
    assert!(evidence_host(scratch.path()).is_err());
    let mut inputs = GitInputs::for_bound_evidence(scratch.path()).unwrap();
    inputs.required(REGISTRY_RELATIVE_PATH).unwrap();
    inputs.verify().unwrap();
    let other = tempfile::tempdir().unwrap();
    assert!(registry(other.path()).is_err());
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                let _installed = f.install();
                assert!(registry(scratch.path()).is_err());
                assert!(GitInputs::for_bound_evidence(scratch.path()).is_err());
            })
            .join()
            .unwrap();
    });
    drop(guard);
    assert!(registry(scratch.path()).is_err());
    assert!(inputs.verify().is_err());
    assert!(GitInputs::for_bound_evidence(scratch.path()).is_err());
    // Dropping the guard frees the single shared candidate slot for another binding.
    let _again = captured.bind_view(&view).unwrap();
    assert_eq!(registry(scratch.path()).unwrap(), *captured.registry());
}

#[test]
fn binding_refuses_even_semantically_equal_registry_byte_changes() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let captured = captured(&f);
    let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal
        .isolated_view(f.root.path(), scratch.path())
        .unwrap();
    let raw = fs::read_to_string(scratch.path().join(REGISTRY_RELATIVE_PATH)).unwrap();
    let changed = format!("{raw}\n# scratch-only comment\n");
    assert_eq!(
        AgentRegistry::parse_unenforced(&raw).unwrap(),
        AgentRegistry::parse_unenforced(&changed).unwrap()
    );
    write(scratch.path(), REGISTRY_RELATIVE_PATH, changed);
    let err = captured.bind_view(&view).err().unwrap();
    assert!(err.contains("registry bytes"), "{err}");
    assert!(registry(scratch.path()).is_err());
}

#[test]
fn binding_rechecks_the_proposal_beyond_the_captured_route_inputs() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    // An ignored validation input belongs to Proposal's observation but is outside the route's
    // producer/registry inputs. A route recheck alone cannot detect this change.
    write(f.root.path(), ".git/info/exclude", "docs/local-input.txt\n");
    write(f.root.path(), "docs/local-input.txt", "original\n");
    let captured = captured(&f);
    let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal
        .isolated_view(f.root.path(), scratch.path())
        .unwrap();
    write(f.root.path(), "docs/local-input.txt", "concurrent edit\n");
    captured.recheck(f.root.path()).unwrap();
    let err = captured.bind_view(&view).err().unwrap();
    assert!(err.contains("closeout inputs changed"), "{err}");
    assert!(candidate_registry(scratch.path()).unwrap().is_none());
    assert_eq!(
        fs::read_to_string(f.root.path().join("docs/local-input.txt")).unwrap(),
        "concurrent edit\n"
    );
}

#[test]
fn binding_refuses_a_replaced_destination_even_with_identical_files() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let captured = captured(&f);
    let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal
        .isolated_view(f.root.path(), scratch.path())
        .unwrap();
    let saved = tempfile::tempdir().unwrap();
    fs::rename(scratch.path(), saved.path().join("original")).unwrap();
    fs::create_dir(scratch.path()).unwrap();
    proposal.materialize(scratch.path()).unwrap();
    let err = captured.bind_view(&view).err().unwrap();
    assert!(
        err.contains("scratch cannot acquire Git authority"),
        "{err}"
    );
    assert!(candidate_registry(scratch.path()).unwrap().is_none());
}

#[test]
fn binding_refuses_scratch_git_metadata_and_registry_symlinks() {
    for change in 0..3 {
        let f = Fixture::new_for(&scope::SYNTHETIC);
        let captured = captured(&f);
        let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let view = proposal
            .isolated_view(f.root.path(), scratch.path())
            .unwrap();
        match change {
            0 => fs::create_dir(scratch.path().join(".git")).unwrap(),
            1 => write(scratch.path(), ".git", "gitdir: /foreign\n"),
            _ => {
                fs::remove_file(scratch.path().join(REGISTRY_RELATIVE_PATH)).unwrap();
                std::os::unix::fs::symlink(
                    f.root.path().join(REGISTRY_RELATIVE_PATH),
                    scratch.path().join(REGISTRY_RELATIVE_PATH),
                )
                .unwrap();
            }
        }
        assert!(captured.bind_view(&view).is_err(), "accepted {change}");
        assert!(candidate_registry(scratch.path()).unwrap().is_none());
    }
}

#[test]
fn lookup_refuses_destination_replacement_after_binding() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let captured = captured(&f);
    let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal
        .isolated_view(f.root.path(), scratch.path())
        .unwrap();
    let _guard = captured.bind_view(&view).unwrap();
    assert_eq!(registry(scratch.path()).unwrap(), *captured.registry());
    let saved = tempfile::tempdir().unwrap();
    fs::rename(scratch.path(), saved.path().join("original")).unwrap();
    fs::create_dir(scratch.path()).unwrap();
    proposal.materialize(scratch.path()).unwrap();
    assert_isolated_lookup_error(scratch.path(), "scratch cannot acquire Git authority");
}

#[test]
fn lookup_refuses_registry_rewrite_after_binding() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let captured = captured(&f);
    let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal
        .isolated_view(f.root.path(), scratch.path())
        .unwrap();
    let _guard = captured.bind_view(&view).unwrap();
    assert_eq!(registry(scratch.path()).unwrap(), *captured.registry());
    let raw = fs::read_to_string(scratch.path().join(REGISTRY_RELATIVE_PATH)).unwrap();
    let changed = format!("{raw}\n# changed after binding\n");
    assert_eq!(
        AgentRegistry::parse_unenforced(&raw).unwrap(),
        AgentRegistry::parse_unenforced(&changed).unwrap()
    );
    write(scratch.path(), REGISTRY_RELATIVE_PATH, changed);
    assert_isolated_lookup_error(scratch.path(), "registry bytes");
    write(scratch.path(), REGISTRY_RELATIVE_PATH, raw);
    assert_eq!(registry(scratch.path()).unwrap(), *captured.registry());
}

#[test]
fn lookup_refuses_a_caller_commit_moved_after_binding() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let captured = captured(&f);
    let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal
        .isolated_view(f.root.path(), scratch.path())
        .unwrap();
    let _guard = captured.bind_view(&view).unwrap();
    assert_eq!(registry(scratch.path()).unwrap(), *captured.registry());
    git(
        f.root.path(),
        &["commit", "--allow-empty", "-m", "moved caller"],
    );
    assert_isolated_lookup_error(scratch.path(), "caller workspace");
}

#[test]
fn borrowed_git_inputs_keep_regular_file_byte_and_mode_checks() {
    use std::os::unix::fs::PermissionsExt;
    const EVIDENCE: &str = "docs/evidence.bin";
    let f = Fixture::new_for(&scope::SYNTHETIC);
    write(f.root.path(), EVIDENCE, b"committed evidence");
    f.commit();
    let captured = captured(&f);
    let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal
        .isolated_view(f.root.path(), scratch.path())
        .unwrap();
    assert!(GitInputs::for_bound_evidence(scratch.path()).is_err());
    let _guard = captured.bind_view(&view).unwrap();
    let mut inputs = GitInputs::for_bound_evidence(scratch.path()).unwrap();
    assert_eq!(inputs.required(EVIDENCE).unwrap(), b"committed evidence");
    inputs.verify().unwrap();
    let path = scratch.path().join(EVIDENCE);
    write(scratch.path(), EVIDENCE, b"tampered evidence");
    assert!(inputs
        .verify()
        .unwrap_err()
        .contains("working bytes disagree"));
    write(scratch.path(), EVIDENCE, b"committed evidence");
    let mode = fs::metadata(&path).unwrap().permissions().mode();
    fs::set_permissions(&path, fs::Permissions::from_mode(mode ^ 0o100)).unwrap();
    assert!(inputs.verify().unwrap_err().contains("executable mode"));
    fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
    fs::remove_file(&path).unwrap();
    std::os::unix::fs::symlink(f.root.path().join(EVIDENCE), &path).unwrap();
    assert!(inputs.verify().unwrap_err().contains("symlink input"));
    fs::remove_file(&path).unwrap();
    assert!(
        inputs.verify().is_err(),
        "missing committed evidence accepted"
    );
    write(scratch.path(), EVIDENCE, b"committed evidence");
    inputs.verify().unwrap();
    write(scratch.path(), "docs/untracked.bin", b"untracked evidence");
    assert!(inputs
        .optional("docs/untracked.bin")
        .unwrap_err()
        .contains("untracked bound input"));
}

#[test]
fn lookup_refuses_registry_symlink_after_binding_even_with_captured_bytes() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let captured = captured(&f);
    let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal
        .isolated_view(f.root.path(), scratch.path())
        .unwrap();
    let _guard = captured.bind_view(&view).unwrap();
    assert_eq!(registry(scratch.path()).unwrap(), *captured.registry());
    let path = scratch.path().join(REGISTRY_RELATIVE_PATH);
    let original = fs::read(&path).unwrap();
    fs::remove_file(&path).unwrap();
    std::os::unix::fs::symlink(f.root.path().join(REGISTRY_RELATIVE_PATH), &path).unwrap();
    assert_eq!(fs::read(&path).unwrap(), original);
    assert_isolated_lookup_error(scratch.path(), "symlink");
}

#[test]
fn lookup_refuses_git_insertion_after_binding() {
    for change in 0..3 {
        let f = Fixture::new_for(&scope::SYNTHETIC);
        let captured = captured(&f);
        let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let view = proposal
            .isolated_view(f.root.path(), scratch.path())
            .unwrap();
        let _guard = captured.bind_view(&view).unwrap();
        assert_eq!(registry(scratch.path()).unwrap(), *captured.registry());
        match change {
            0 => fs::create_dir(scratch.path().join(".git")).unwrap(),
            1 => write(scratch.path(), ".git", "gitdir: /foreign\n"),
            _ => std::os::unix::fs::symlink("missing", scratch.path().join(".git")).unwrap(),
        }
        assert_isolated_lookup_error(scratch.path(), "scratch cannot acquire Git authority");
    }
}

fn assert_isolated_lookup_error(root: &Path, expected: &str) {
    // Keep the guard alive across repeated lookups: an error must never fall back to production.
    for _ in 0..2 {
        match registry(root) {
            Err(RegistryError::Isolated(err)) => assert!(err.contains(expected), "{err}"),
            Err(err) => panic!("expected isolated {expected} error, got {err}"),
            Ok(_) => panic!("expected isolated {expected} error, got a registry"),
        }
    }
}

#[test]
fn binding_refuses_another_caller_and_captured_host_drift() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let captured = captured(&f);
    let other = Fixture::new_for(&scope::SYNTHETIC);
    let proposal = other
        .resolve()
        .unwrap()
        .proposal(other.root.path())
        .unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal
        .isolated_view(other.root.path(), scratch.path())
        .unwrap();
    let err = captured.bind_view(&view).err().unwrap();
    assert!(err.contains("another caller"), "{err}");
    assert!(candidate_registry(scratch.path()).unwrap().is_none());

    let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal
        .isolated_view(f.root.path(), scratch.path())
        .unwrap();
    f.api.put("repos/fixture/isolated", json!({"id":999}));
    assert!(captured.bind_view(&view).is_err());
    assert!(candidate_registry(scratch.path()).unwrap().is_none());
}

#[test]
fn view_and_ma4_candidate_share_one_exclusive_binding_slot() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let captured = captured(&f);
    let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal
        .isolated_view(f.root.path(), scratch.path())
        .unwrap();
    let candidate = tempfile::tempdir().unwrap();
    git(
        f.root.path(),
        &[
            "clone",
            "--no-local",
            f.root.path().to_str().unwrap(),
            candidate.path().to_str().unwrap(),
        ],
    );
    let guard = captured.bind_view(&view).unwrap();
    assert!(captured.bind_view(&view).is_err());
    assert!(captured.bind_candidate(candidate.path()).is_err());
    assert_eq!(registry(scratch.path()).unwrap(), *captured.registry());
    drop(guard);
    let _candidate = captured.bind_candidate(candidate.path()).unwrap();
    assert!(captured.bind_view(&view).is_err());
    assert_eq!(registry(candidate.path()).unwrap(), *captured.registry());
    assert!(candidate_registry(scratch.path()).unwrap().is_none());
}

fn validate_args(root: &Path) -> crate::manifest_validate::Args {
    crate::manifest_validate::Args {
        root: Some(root.join("cli_manifests/depth_fixture")),
        rules: None,
        schema: None,
        version_schema: None,
        mode: crate::manifest_validate::Mode::Check,
        json: false,
        tip: None,
    }
}

#[test]
#[cfg(target_os = "linux")]
fn scratch_render_and_manifest_validation_borrow_git_inputs_for_delivered_ma4_evidence() {
    use crate::{
        depth_ci::profile::Profile,
        depth_record::{DepthRecord, DepthResult},
        support_matrix::{self, SupportMatrixArtifact},
    };
    let (f, host) = delivered();
    let root = f.root.path();
    let _installed = f.install();
    let _reader = host.install();
    let record = DepthRecord::parse(
        &fs::read_to_string(root.join(RECORD)).unwrap(),
        "depth_fixture",
        "0.2.0",
    )
    .unwrap();
    let evidence: Vec<_> = record
        .execution
        .as_ref()
        .expect("delivered MA4 execution")
        .results
        .iter()
        .flat_map(|result| &result.evidence)
        .collect();
    assert!(!evidence.is_empty());
    let verdicts = crate::depth_evidence::evaluate_isolated_record(root, &record).unwrap();
    assert_eq!(
        Profile::Synthetic
            .obligations(&record)
            .iter()
            .map(|o| verdicts.result(&record, o))
            .collect::<Vec<_>>(),
        vec![DepthResult::Verified; 11]
    );

    let captured = captured(&f);
    let proposal = f.resolve().unwrap().proposal(root).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal.isolated_view(root, scratch.path()).unwrap();
    assert!(!scratch.path().join(".git").exists());
    assert!(fs::read_to_string(scratch.path().join("Cargo.toml"))
        .unwrap()
        .contains("[workspace]"));
    assert!(support_matrix::generate_publication_artifacts(scratch.path()).is_err());
    // Exercise the Git-backed dependency directly: no registry failure or empty-ref shortcut.
    assert!(crate::depth_record::bound_evidence(scratch.path(), [&record]).is_err());
    let guard = captured.bind_view(&view).unwrap();
    let identities = crate::depth_record::bound_evidence(scratch.path(), [&record]).unwrap();
    for reference in &evidence {
        assert_eq!(identities.get(&reference.path), Some(&reference.sha256));
    }
    let bundle = support_matrix::generate_publication_artifacts(scratch.path()).unwrap();
    let artifact: SupportMatrixArtifact = serde_json::from_str(&bundle.json).unwrap();
    let row = artifact
        .rows
        .iter()
        .find(|r| r.agent == "depth_fixture" && r.version == "0.2.0")
        .unwrap();
    let facts = row.depth.as_ref().unwrap();
    assert!(facts.obligations_frozen);
    assert!(!facts.promises.is_empty());
    for promise in &facts.promises {
        assert!(!promise.qualified);
        assert!(!promise.obligations.is_empty());
        assert!(promise
            .obligations
            .iter()
            .all(|o| o.result == DepthResult::Unverified));
    }
    for (path, contents) in [
        (support_matrix::JSON_OUTPUT_PATH, bundle.json),
        (support_matrix::MARKDOWN_OUTPUT_PATH, bundle.markdown),
        (
            support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
            bundle.runtime_support_data,
        ),
        (
            crate::capability_matrix::DEFAULT_OUT_PATH,
            crate::capability_matrix::generate_markdown_for_workspace(scratch.path()).unwrap(),
        ),
    ] {
        assert!(!contents.is_empty());
        write(scratch.path(), path, contents);
    }
    assert_eq!(
        crate::manifest_validate::run(validate_args(scratch.path())),
        0
    );

    // The MA6 check uses no --tip. Adding it still requires real Git history and must refuse.
    let mut with_tip = validate_args(scratch.path());
    with_tip.tip = Some(git(root, &["rev-parse", "HEAD"]));
    let codes = crate::manifest_validate::violation_codes(with_tip).unwrap();
    assert_eq!(codes, ["DEPTH_TIP_UNREADABLE"]);
    assert!(route(scratch.path()).is_err());
    assert!(evidence_host(scratch.path()).is_err());
    assert!(!scratch.path().join(".git").exists());

    // Borrowed objects cannot bless substituted working bytes, even with genuine E and receipts.
    let reference = evidence[0];
    let original = fs::read(scratch.path().join(&reference.path)).unwrap();
    write(scratch.path(), &reference.path, b"substituted evidence");
    let err = crate::depth_record::bound_evidence(scratch.path(), [&record]).unwrap_err();
    assert!(err.contains("working bytes disagree"), "{err}");
    assert!(support_matrix::generate_publication_artifacts(scratch.path()).is_err());
    assert_eq!(
        crate::manifest_validate::run(validate_args(scratch.path())),
        2
    );
    write(scratch.path(), &reference.path, original);

    // This control proves validation actually consulted the binding and did not skip depth.
    drop(guard);
    assert!(crate::depth_record::bound_evidence(scratch.path(), [&record]).is_err());
    let codes = crate::manifest_validate::violation_codes(validate_args(scratch.path())).unwrap();
    assert!(codes.contains(&"DEPTH_ROUTE_REFUSED"), "{codes:?}");
    assert_eq!(
        crate::manifest_validate::run(validate_args(scratch.path())),
        2
    );
    view.source().unwrap();
    captured.recheck(root).unwrap();

    // Moving the caller after binding invalidates both the registry and borrowed Git handle.
    let _guard = captured.bind_view(&view).unwrap();
    git(root, &["commit", "--allow-empty", "-m", "moved caller"]);
    let err = crate::depth_record::bound_evidence(scratch.path(), [&record]).unwrap_err();
    assert!(err.contains("caller workspace"), "{err}");
    assert!(support_matrix::generate_publication_artifacts(scratch.path()).is_err());
    assert_eq!(
        crate::manifest_validate::run(validate_args(scratch.path())),
        2
    );
}

#[cfg(target_os = "linux")]
const RECORD: &str = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";

/// The connected `ma4_evidence` chain: freeze MA1/MA2, run its real offline executor, deliver E.
#[cfg(target_os = "linux")]
fn delivered() -> (Fixture, crate::depth_evidence::SyntheticHost) {
    use crate::{
        agent_maintenance::{
            execute,
            prepare::{apply_prepare_plan, build_prepare_plan},
        },
        depth_ci::profile::Profile,
    };
    use std::os::unix::fs::PermissionsExt;
    let (f, mut args) = super::prepare::workspace();
    let root = f.root.path();
    args.request_commit = "0b1e1f0a478af0cd06d6ba6998ab594f09fbe592".into();
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.gitignore"),
        root.join(".gitignore"),
    )
    .unwrap();
    f.commit();
    {
        let _installed = f.install();
        apply_prepare_plan(root, &build_prepare_plan(root, &args).unwrap()).unwrap();
        f.commit();
        super::prepare::acquire(root, "0.2.0");
        f.commit();
        apply_prepare_plan(root, &build_prepare_plan(root, &args).unwrap()).unwrap();
        f.commit();
    }
    write(
        root,
        Profile::Synthetic.fixture(),
        fs::read(root.join(RECORD)).unwrap(),
    );
    let host = crate::depth_evidence::synthetic_host(root);
    f.host();
    let bin = tempfile::tempdir().unwrap();
    let exe = bin.path().join("assembler");
    fs::copy(std::env::current_exe().unwrap(), &exe).unwrap();
    let executor = bin.path().join("executor.sh");
    fs::write(&executor, format!(
        "#!/bin/sh\nprompt=$(cat)\ncase \"$prompt\" in\n*'Repository preflight'*) echo {} ;;\n*) \"{}\" --exact isolated_maintenance::tests::ma4_evidence::executor_entry --ignored --quiet || exit 1 ;;\nesac\n",
        execute::PREFLIGHT_SENTINEL, exe.display(),
    )).unwrap();
    fs::set_permissions(&executor, fs::Permissions::from_mode(0o755)).unwrap();
    {
        let _installed = f.install();
        let _reader = host.install();
        let _gates = execute::gate_stub::stub();
        super::ma4_route::run_as(root, executor.to_str().unwrap(), "scratch-view").unwrap();
    }
    f.commit();
    host.staging(&git(root, &["rev-parse", "HEAD"]));
    (f, host)
}
