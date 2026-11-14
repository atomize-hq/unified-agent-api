//! MA2 freeze-only proof. Acquisition is already present on the authenticated predecessor.
use super::*;
use crate::{
    agent_maintenance::prepare::{apply_prepare_plan, build_prepare_plan},
    depth_acceptance::{
        admission::{
            self,
            freeze::{
                self,
                authority::{Observation, Writer},
            },
            Checkpoint, Generation, Reason,
        },
        check_integration,
    },
    depth_record::DepthRecord,
};

#[test]
fn fr2_ma2_freeze_only_complete_tree_canonical_packet_and_real_in1() {
    let (f, args) = prepare::workspace();
    let root = f.root.path();
    let _installed = f.install();
    // MA1's original request is committed before already-acquired MA2 inputs.
    apply_prepare_plan(root, &build_prepare_plan(root, &args).unwrap()).unwrap();
    f.commit();
    prepare::acquire(root, "0.2.0");
    f.commit();
    let parent = git(root, &["rev-parse", "HEAD"]);
    f.host();
    git(
        root,
        &["update-ref", "refs/remotes/origin/staging", &parent],
    );
    let plan = build_prepare_plan(root, &args).unwrap();
    #[cfg(unix)]
    intervening_handoff_mode_change(root, &plan);
    apply_prepare_plan(root, &plan).unwrap();
    let freeze_commit = super::in1_integration::candidate(&f, &parent);
    let publication = f.resolve().unwrap().admission_publication().unwrap();
    let parent_object = admission::emitter::object(root, &parent).unwrap();
    let candidate_object = admission::emitter::object(root, &freeze_commit).unwrap();
    let record = DepthRecord::parse(
        &fs::read_to_string(
            root.join("cli_manifests/depth_fixture/reports/0.2.0/depth-record.json"),
        )
        .unwrap(),
        "depth_fixture",
        "0.2.0",
    )
    .unwrap();
    let observation = Observation {
        repository: publication.authority.repository.clone(),
        predecessor: parent_object.clone(),
        generation: Generation::from_record(&record),
        policy: publication.authority.policy.clone(),
        writer: Writer::Maintenance,
    };
    f.api.3.put(
        &format!("freeze-inputs/{freeze_commit}"),
        serde_json::to_value(&observation).unwrap(),
    );
    let (expected, candidate) = freeze::expected(
        root,
        &parent_object,
        &candidate_object,
        &record,
        &observation,
    )
    .unwrap();
    expected.require_equal(&candidate).unwrap();
    super::fr2_canonical::check("ma2", &candidate, None, &Default::default());
    assert!(record.obligations.is_some());
    assert!(record.execution.is_none());
    assert!(record.acceptance_entries.is_empty());
    let source = crate::runtime_advertising::source::Source::tip(root, &parent).unwrap();
    assert_eq!(
        candidate[crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH].bytes,
        source
            .read(crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH)
            .unwrap()
            .as_bytes()
    );
    let support: serde_json::Value =
        serde_json::from_slice(&candidate[crate::support_matrix::JSON_OUTPUT_PATH].bytes).unwrap();
    let row = support["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["agent"] == "depth_fixture" && row["version"] == "0.2.0")
        .unwrap();
    assert_eq!(row["pointer_promotion"], "none");
    assert_eq!(row["uaa_support"], "unsupported");
    // Independent canonical packet assertions and exact inventory, beyond planner equality.
    let maintenance = "docs/agents/lifecycle/depth_fixture-maintenance";
    let files: Vec<_> = plan
        .files
        .iter()
        .filter(|file| file.relative_path.starts_with(maintenance))
        .collect();
    let mut names: Vec<_> = files
        .iter()
        .map(|file| {
            file.relative_path
                .strip_prefix(&format!("{maintenance}/"))
                .unwrap()
        })
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "CI_WORKFLOWS_PLAN.md",
            "HANDOFF.md",
            "OPS_PLAYBOOK.md",
            "README.md",
            "governance/execute-agent-maintenance-prompt.md",
            "governance/maintenance-request.toml",
            "governance/pr-summary.md",
            "governance/remediation-log.md",
            "review_surfaces.md",
            "scope_brief.md",
            "seam_map.md",
            "threading.md"
        ]
    );
    let handoff = String::from_utf8(
        candidate[&format!("{maintenance}/HANDOFF.md")]
            .bytes
            .clone(),
    )
    .unwrap();
    assert!(handoff.contains("# Handoff"));
    assert!(handoff.contains("prepare-agent-maintenance"));
    assert!(handoff.contains("close-agent-maintenance"));
    assert!(handoff.contains("0.2.0"));
    let request: serde_json::Value = toml_edit::de::from_str(
        &String::from_utf8(
            candidate[&format!("{maintenance}/governance/maintenance-request.toml")]
                .bytes
                .clone(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(request["request_recorded_at"], "2026-11-03T00:00:00Z");
    assert_eq!(
        request["support_depth"]["policy_identity"],
        record.policy.identity
    );
    assert_eq!(
        request["support_depth"]["obligations_identity"],
        record.obligations.as_ref().unwrap().identity
    );
    check_integration(root, &parent).unwrap();
    let retained = f
        .api
        .3
        .records()
        .into_iter()
        .find(|r| r.candidate.commit == freeze_commit)
        .unwrap();
    assert_eq!(retained.checkpoint, Checkpoint::RequestFreeze);
    assert!(retained.reasons.contains(&Reason::ExactFreezeTree));
    // Frozen typed rendering inputs must never reopen live coverage reports.
    let registry = AgentRegistry::parse_unenforced(
        &fs::read_to_string(root.join(REGISTRY_RELATIVE_PATH)).unwrap(),
    )
    .unwrap();
    let inputs = crate::agent_maintenance::prepare::derive_inputs(root, &args, &registry).unwrap();
    let frozen = crate::freeze_transition::WriterInputs::render(&inputs)
        .unwrap()
        .files;
    let coverage_path = root
        .join("cli_manifests/depth_fixture/reports/0.2.0/coverage.x86_64-unknown-linux-gnu.json");
    let coverage = match fs::read(&coverage_path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => panic!("coverage fixture read failed: {error}"),
    };
    fs::write(&coverage_path, b"substituted malformed coverage").unwrap();
    assert_eq!(
        crate::freeze_transition::WriterInputs::render(&inputs)
            .unwrap()
            .files,
        frozen
    );
    match coverage {
        Some(bytes) => fs::write(coverage_path, bytes).unwrap(),
        None => fs::remove_file(coverage_path).unwrap(),
    }
    // Every acquisition-owned output, including metadata and lockfile absence, is empty-only.
    let (_, outputs) = crate::manifest_acquisition::plan::delivery_paths(
        "cli_manifests/depth_fixture",
        "0.2.0",
        &["x86_64-unknown-linux-gnu".into()],
    );
    for path in outputs {
        let mut changed = candidate.clone();
        changed.insert(
            path.clone(),
            crate::freeze_transition::Entry::file("100644", b"new acquisition bytes".to_vec()),
        );
        assert_eq!(
            crate::freeze_transition::acquisition::validate(
                &candidate,
                &changed,
                std::slice::from_ref(&path)
            )
            .unwrap_err(),
            "acquisition authority unavailable"
        );
        git(root, &["reset", "--hard", &freeze_commit]);
        write(root, &path, &changed[&path].bytes);
        let commit = super::in1_integration::candidate(&f, &parent);
        let object = admission::emitter::object(root, &commit).unwrap();
        let refused = freeze::classify(
            root,
            &publication,
            &parent_object,
            &object,
            &record,
            Some(&observation),
        );
        assert_eq!(refused.checkpoint, Checkpoint::ExecutionRequired, "{path}");
        assert_eq!(
            refused.reason,
            Reason::AcquisitionAuthorityUnavailable,
            "{path}"
        );
    }
    git(root, &["reset", "--hard", &freeze_commit]);
    // An authenticated original request supplies generation variables. Candidate-selected
    // request timestamps cannot acquire authority by generating matching-looking packets.
    let mut changed = candidate.clone();
    changed
        .get_mut(&format!(
            "{maintenance}/governance/maintenance-request.toml"
        ))
        .unwrap()
        .bytes
        .extend(b"\n# substituted request\n");
    assert!(expected.require_equal(&changed).is_err());
    let request_path = format!("{maintenance}/governance/maintenance-request.toml");
    write(root, &request_path, &changed[&request_path].bytes);
    let commit = super::in1_integration::candidate(&f, &parent);
    let object = admission::emitter::object(root, &commit).unwrap();
    let refused = freeze::classify(
        root,
        &publication,
        &parent_object,
        &object,
        &record,
        Some(&observation),
    );
    assert_eq!(refused.checkpoint, Checkpoint::ExecutionRequired);
    assert_eq!(refused.reason, Reason::FreezeTreeMismatch);
    git(root, &["reset", "--hard", &freeze_commit]);
    // A same-commit acquisition output receives the distinct bounded refusal at real IN1.
    write(
        root,
        "cli_manifests/depth_fixture/artifacts.lock.json",
        "{}\n",
    );
    let acquired_commit = super::in1_integration::candidate(&f, &parent);
    f.api.3.put(
        &format!("freeze-inputs/{acquired_commit}"),
        serde_json::to_value(&observation).unwrap(),
    );
    check_integration(root, &parent).unwrap();
    let refused_freeze = f
        .api
        .3
        .records()
        .into_iter()
        .find(|r| r.candidate.commit == acquired_commit)
        .unwrap();
    assert_eq!(refused_freeze.checkpoint, Checkpoint::ExecutionRequired);
    assert!(refused_freeze
        .reasons
        .contains(&Reason::AcquisitionAuthorityUnavailable));
    git(root, &["reset", "--hard", &freeze_commit]);
    // Only now advance fake protected history and observe through the same real predicate.
    f.host();
    git(
        root,
        &["update-ref", "refs/remotes/origin/staging", &freeze_commit],
    );
    check_integration(root, &parent).unwrap();
}

#[cfg(unix)]
fn intervening_handoff_mode_change(
    root: &Path,
    plan: &crate::agent_maintenance::prepare::PreparePlan,
) {
    use crate::agent_maintenance::prepare::BEFORE_APPLY;
    use std::os::unix::fs::PermissionsExt;
    const HANDOFF: &str = "docs/agents/lifecycle/depth_fixture-maintenance/HANDOFF.md";
    struct ResetHook;
    impl Drop for ResetHook {
        fn drop(&mut self) {
            BEFORE_APPLY.with(|hook| hook.set(None));
        }
    }
    let before = plan
        .files
        .iter()
        .map(|file| {
            (
                file.relative_path.clone(),
                fs::read(root.join(&file.relative_path)).ok(),
            )
        })
        .collect::<Vec<_>>();
    let index = fs::read(root.join(".git/index")).unwrap();
    let mode = fs::metadata(root.join(HANDOFF))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o100, 0);
    BEFORE_APPLY.with(|hook| {
        hook.set(Some(|root| {
            fs::set_permissions(root.join(HANDOFF), fs::Permissions::from_mode(0o755)).unwrap();
        }))
    });
    let reset = ResetHook;
    let error = apply_prepare_plan(root, plan).unwrap_err();
    drop(reset);
    assert!(error.to_string().contains(HANDOFF), "{error}");
    assert!(error.to_string().contains("divergent"), "{error}");
    assert_eq!(
        fs::metadata(root.join(HANDOFF))
            .unwrap()
            .permissions()
            .mode()
            & 0o100,
        0o100
    );
    for (path, bytes) in before {
        assert_eq!(fs::read(root.join(&path)).ok(), bytes, "{path}");
    }
    assert_eq!(fs::read(root.join(".git/index")).unwrap(), index);
    fs::set_permissions(root.join(HANDOFF), fs::Permissions::from_mode(mode)).unwrap();
}
