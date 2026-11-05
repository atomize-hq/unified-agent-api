//! CP-P2: scratch receives only the caller's captured registry, scoped to one thread and guard.
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
    let other = tempfile::tempdir().unwrap();
    assert!(registry(other.path()).is_err());
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                let _installed = f.install();
                assert!(registry(scratch.path()).is_err());
            })
            .join()
            .unwrap();
    });
    drop(guard);
    assert!(registry(scratch.path()).is_err());
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
    assert!(candidate_registry(scratch.path()).is_none());
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
    assert!(candidate_registry(scratch.path()).is_none());
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
        assert!(candidate_registry(scratch.path()).is_none());
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
    assert!(candidate_registry(scratch.path()).is_none());

    let proposal = f.resolve().unwrap().proposal(f.root.path()).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal
        .isolated_view(f.root.path(), scratch.path())
        .unwrap();
    f.api.put("repos/fixture/isolated", json!({"id":999}));
    assert!(captured.bind_view(&view).is_err());
    assert!(candidate_registry(scratch.path()).is_none());
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
    assert!(candidate_registry(scratch.path()).is_none());
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
fn scratch_render_and_manifest_validation_need_no_additional_git_inputs() {
    use crate::{
        agent_maintenance::prepare::{apply_prepare_plan, build_prepare_plan},
        depth_record::DepthResult,
        support_matrix::{self, SupportMatrixArtifact},
    };
    let (f, args) = super::prepare::workspace();
    let root = f.root.path();
    let _installed = f.install();
    apply_prepare_plan(root, &build_prepare_plan(root, &args).unwrap()).unwrap();
    f.commit();
    super::prepare::acquire(root, "0.2.0");
    f.commit();
    apply_prepare_plan(root, &build_prepare_plan(root, &args).unwrap()).unwrap();
    f.commit();

    let captured = captured(&f);
    let proposal = f.resolve().unwrap().proposal(root).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal.isolated_view(root, scratch.path()).unwrap();
    assert!(!scratch.path().join(".git").exists());
    assert!(fs::read_to_string(scratch.path().join("Cargo.toml"))
        .unwrap()
        .contains("[workspace]"));
    assert!(support_matrix::generate_publication_artifacts(scratch.path()).is_err());
    let guard = captured.bind_view(&view).unwrap();
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

    // This control proves validation actually consulted the binding and did not skip depth.
    drop(guard);
    let codes = crate::manifest_validate::violation_codes(validate_args(scratch.path())).unwrap();
    assert!(codes.contains(&"DEPTH_ROUTE_REFUSED"), "{codes:?}");
    assert_eq!(
        crate::manifest_validate::run(validate_args(scratch.path())),
        2
    );
    view.source().unwrap();
    captured.recheck(root).unwrap();
}
