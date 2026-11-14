//! Continue the genuine FR2 ON2 freeze and FR3 population through real ON2/ON3 writers.
#![cfg(target_os = "linux")]
use super::super::fixture::Installed;
use super::*;
use crate::{
    depth_acceptance::admission::{fake::MemoryHost, Host},
    depth_evidence::{EvidenceVerdicts, SyntheticHost},
};
use clap::Parser;

mod review;

fn install_route(
    f: &Fixture,
    root: &Path,
    host: &Arc<MemoryHost>,
) -> (Arc<super::super::fixture::Fake>, Installed) {
    let api = Arc::new(super::super::fixture::Fake(
        std::sync::Mutex::new(f.api.0.lock().unwrap().clone()),
        Default::default(),
        Default::default(),
        host.clone(),
        Default::default(),
    ));
    let tip = host.json("git/ref/heads/staging").unwrap()["object"]["sha"]
        .as_str()
        .unwrap()
        .to_owned();
    let head = git(root, &["rev-parse", "HEAD"]);
    api.put(
        "repos/fixture/isolated/branches/staging",
        json!({"name":"staging","protected":true,"commit":{"sha":tip}}),
    );
    let blob = git(
        root,
        &["rev-parse", &format!("{tip}:{REGISTRY_RELATIVE_PATH}")],
    );
    api.put(
        &format!("repos/fixture/isolated/contents/{REGISTRY_RELATIVE_PATH}?ref={tip}"),
        json!({"path":REGISTRY_RELATIVE_PATH,"type":"file","sha":blob}),
    );
    for base in [&tip, &f.baseline] {
        api.put(&format!("repos/fixture/isolated/compare/{base}...{head}"), json!({"status":if base == &head {"identical"} else {"ahead"},"merge_base_commit":{"sha":base}}));
    }
    api.put(
        &format!("repos/fixture/isolated/commits/{head}"),
        json!({"sha":head}),
    );
    git(
        root,
        &[
            "remote",
            "set-url",
            "origin",
            "https://github.com/fixture/isolated.git",
        ],
    );
    git(root, &["update-ref", "refs/remotes/origin/staging", &tip]);
    let baseline = f.baseline.clone();
    let subject = f.subject;
    let bound = api.clone();
    let guard = super::super::fixture::install(Box::new(move || {
        (
            super::super::fixture::profile_with(baseline.clone(), subject),
            bound.clone() as Arc<dyn Api>,
        )
    }));
    (api, guard)
}

pub(in crate::isolated_maintenance::tests) fn implement(
    f: &Fixture,
    root: &Path,
    host: &Arc<MemoryHost>,
) {
    eprintln!("ON3 connected proof enters real ON2 implementation");
    let (_api, _route) = install_route(f, root, host);
    let args = |write| {
        crate::runtime_follow_on::Args::parse_from([
            "runtime-follow-on",
            if write { "--write" } else { "--dry-run" },
            "--approval",
            APPROVAL,
            "--run-id",
            "fr2-freeze",
        ])
    };
    let executor = public_on2::executor("", 0);
    let mut args = args(true);
    args.codex_binary = Some(executor.path().join("executor.sh").to_string_lossy().into());
    let frozen = fs::read(root.join(RECORD)).unwrap();
    crate::runtime_follow_on::run_in_workspace(root, args, &mut Vec::new()).unwrap();
    assert_eq!(fs::read(root.join(RECORD)).unwrap(), frozen);
    assert_eq!(
        life::load_lifecycle_state(root, STATE)
            .unwrap()
            .lifecycle_stage,
        life::LifecycleStage::RuntimeIntegrated
    );
}

fn args(write: bool) -> crate::prepare_publication::Args {
    crate::prepare_publication::Args::parse_from([
        "prepare-publication",
        if write { "--write" } else { "--check" },
        "--approval",
        APPROVAL,
    ])
}
fn run(root: &Path, write: bool) -> Result<(), crate::prepare_publication::Error> {
    crate::prepare_publication::run_in_workspace(root, args(write), &mut Vec::new())
}
fn commit(root: &Path, api: &super::super::fixture::Fake, baseline: &str, parent: &str) -> String {
    git(root, &["add", "."]);
    git(
        root,
        &["commit", "--allow-empty", "-qm", "ON3 reporting candidate"],
    );
    let head = git(root, &["rev-parse", "HEAD"]);
    for base in [baseline, parent] {
        api.put(
            &format!("repos/fixture/isolated/compare/{base}...{head}"),
            json!({"status":"ahead","merge_base_commit":{"sha":base}}),
        );
    }
    api.put(
        &format!("repos/fixture/isolated/commits/{head}"),
        json!({"sha":head}),
    );
    head
}

pub(in crate::isolated_maintenance::tests) fn prove(
    f: &Fixture,
    root: &Path,
    host: &Arc<MemoryHost>,
    evidence: SyntheticHost,
    repaired: bool,
    owning_only: bool,
) {
    let review_refusals = !repaired && !owning_only;
    eprintln!("ON3 connected proof enters reporting and IN1 siblings");
    let implemented = git(root, &["rev-parse", "HEAD"]);
    let frozen = fs::read(root.join(RECORD)).unwrap();
    let host = Arc::new(host.fork());
    let _evidence = evidence.install();
    if repaired {
        repair(f, root, &host, &evidence);
    }
    // Refresh the route after any admitted repair changed protected state.
    let (api, _route) = install_route(f, root, &host);
    let parent = git(root, &["rev-parse", "HEAD"]);
    let state = life::load_lifecycle_state(root, STATE).unwrap();
    assert_eq!(fs::read(root.join(RECORD)).unwrap(), frozen);
    if review_refusals {
        refusals(f, root, &api, &evidence);
    }
    let _evidence = evidence.install();
    if review_refusals {
        review::runtime_selection(f, root, &api);
    }
    run(root, false).unwrap();
    run(root, true).unwrap();
    let error = crate::git_inputs::GitInputs::new(root)
        .unwrap()
        .required(RECORD)
        .unwrap_err();
    assert!(
        error.contains("working bytes disagree with index"),
        "{error}"
    );
    let next = life::load_lifecycle_state(root, STATE).unwrap();
    assert_eq!(next.lifecycle_stage, life::LifecycleStage::PublicationReady);
    assert_eq!(next.support_depth, state.support_depth);
    assert_eq!(next.written_paths_sha256, state.written_paths_sha256);
    assert!(next.active_runtime_evidence_run_id.is_none());
    let packet = life::load_publication_ready_packet(
        root,
        "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/publication-ready.json",
    )
    .unwrap();
    let selected = state.active_runtime_evidence_run_id.as_deref().unwrap();
    assert!(packet
        .runtime_evidence_paths
        .iter()
        .all(|p| p.contains(&format!("/runs/{selected}/"))));
    assert_eq!(repaired, selected != "fr2-freeze");
    let record = crate::depth_record::DepthRecord::parse(
        &fs::read_to_string(root.join(RECORD)).unwrap(),
        "depth_fixture",
        "0.2.0",
    )
    .unwrap();
    let prior: crate::depth_record::DepthRecord = serde_json::from_slice(&frozen).unwrap();
    assert_eq!(record.event, prior.event);
    assert_eq!(record.policy, prior.policy);
    assert_eq!(record.obligations, prior.obligations);
    assert!(record.acceptance_entries.is_empty());
    canonical(&next, &packet, &record);
    let execution = record.execution.as_ref().unwrap();
    assert_eq!(execution.results.len(), 11);
    for row in &execution.results {
        assert_eq!(row.result, crate::depth_record::DepthResult::Verified);
        for file in &row.evidence {
            let bytes = fs::read(root.join(&file.path)).unwrap();
            assert_eq!(file.sha256, hex::encode(Sha256::digest(&bytes)));
            let receipt =
                crate::depth_evidence::EvidenceReceipt::parse(std::str::from_utf8(&bytes).unwrap())
                    .unwrap();
            assert_eq!(receipt.build.source_commit, implemented);
            for payload in &receipt.provenance {
                assert_eq!(
                    payload.file.sha256,
                    hex::encode(Sha256::digest(
                        fs::read(root.join(&payload.file.path)).unwrap()
                    ))
                );
            }
        }
    }
    let head = commit(root, &api, &f.baseline, &parent);
    closure(root, &parent, &head);
    run(root, false).unwrap();
    if review_refusals {
        in1_refusals(f, root, &api, &parent, &head, &record);
    }
    if review_refusals {
        review::reporting_candidate(f, root, &api, &parent, &head);
    }
    check_integration(root, &parent).unwrap();
    assert!(host
        .records()
        .iter()
        .any(|r| r.candidate.commit == head && r.successful()));
    host.root(&head);
    evidence.staging(&head);
    drop(_route);
    let (_api, _route) = install_route(f, root, &host);
    check_integration(root, &parent).unwrap();
    run(root, false).unwrap();
    reporting_facts(root);
    if owning_only {
        crate::depth_acceptance::onboarding::tests::connected_publication(root);
    } else {
        review::publication_checkpoint(f, root, &_api, &head);
    }
}

fn repair(f: &Fixture, root: &Path, host: &Arc<MemoryHost>, evidence: &SyntheticHost) {
    let (api, route) = install_route(f, root, host);
    let parent = git(root, &["rev-parse", "HEAD"]);
    let state = life::load_lifecycle_state(root, STATE).unwrap();
    let run = state.active_runtime_evidence_run_id.as_deref().unwrap();
    fs::remove_file(root.join(format!(
        "docs/agents/.uaa-temp/runtime-follow-on/runs/{run}/handoff.json"
    )))
    .unwrap();
    let broken = commit(root, &api, &f.baseline, &parent);
    check_integration(root, &parent).unwrap();
    host.root(&broken);
    evidence.staging(&broken);
    drop(route);
    let (api, route) = install_route(f, root, host);
    check_integration(root, &parent).unwrap();
    let args = crate::repair_runtime_evidence::Args::parse_from([
        "repair-runtime-evidence",
        "--write",
        "--approval",
        APPROVAL,
    ]);
    crate::repair_runtime_evidence::run_in_workspace(root, args, &mut Vec::new()).unwrap();
    let repaired = commit(root, &api, &f.baseline, &broken);
    check_integration(root, &broken).unwrap();
    host.root(&repaired);
    evidence.staging(&repaired);
    drop(route);
    let (_api, _route) = install_route(f, root, host);
    check_integration(root, &broken).unwrap();
}

fn closure(root: &Path, before: &str, after: &str) {
    for path in [
        "crates",
        "Cargo.toml",
        "Cargo.lock",
        ".github/workflows/ci.yml",
        crate::depth_ci::ARCHIVE,
    ] {
        assert_eq!(
            git(root, &["rev-parse", &format!("{before}:{path}")]),
            git(root, &["rev-parse", &format!("{after}:{path}")]),
            "ON3 changed protected closure {path}"
        );
    }
}
fn reporting_facts(root: &Path) {
    let artifact: Value = serde_json::from_slice(
        &fs::read(root.join(crate::support_matrix::JSON_OUTPUT_PATH)).unwrap(),
    )
    .unwrap();
    let row = artifact["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["agent"] == "depth_fixture")
        .unwrap();
    assert_eq!(row["depth"]["promises"][0]["qualified"], false);
    let expected: Value = serde_json::from_str(include_str!("vectors/on3-reporting.json")).unwrap();
    assert_eq!(row["depth"]["promises"].as_array().unwrap().len(), 1);
    assert_eq!(
        row["depth"]["promises"][0]["obligations"],
        expected["published_obligations"]
    );
    for pointer in [
        "min_supported.txt",
        "latest_validated.txt",
        "pointers/latest_supported/x86_64-unknown-linux-gnu.txt",
        "pointers/latest_validated/x86_64-unknown-linux-gnu.txt",
    ] {
        assert_eq!(
            fs::read_to_string(root.join(ROOT).join(pointer)).unwrap(),
            "none\n"
        );
    }
    let metadata: Value =
        serde_json::from_slice(&fs::read(root.join(ROOT).join("versions/0.2.0.json")).unwrap())
            .unwrap();
    assert_eq!(metadata["status"], "snapshotted");
}
fn refusals(f: &Fixture, root: &Path, api: &super::super::fixture::Fake, evidence: &SyntheticHost) {
    let state = life::load_lifecycle_state(root, STATE).unwrap();
    let run_id = state.active_runtime_evidence_run_id.as_deref().unwrap();
    let parent = git(root, &["rev-parse", "HEAD"]);
    let tip = git(root, &["rev-parse", "refs/remotes/origin/staging"]);
    for (path, bytes, expected) in [
        (
            format!("docs/agents/.uaa-temp/runtime-follow-on/runs/{run_id}/written-paths.json"),
            Some(b"[]\n".to_vec()),
            "written-paths.json differs from lifecycle written_paths_sha256",
        ),
        (
            format!("docs/agents/.uaa-temp/runtime-follow-on/runs/{run_id}/run-status.json"),
            None,
            "is missing required runtime evidence files",
        ),
        (
            format!("{ROOT}/wrapper_coverage.json"),
            Some(b"{}\n".to_vec()),
            "changed or missing frozen acquired inputs",
        ),
    ] {
        if let Some(bytes) = bytes {
            write(root, &path, bytes);
        } else {
            fs::remove_file(root.join(&path)).unwrap();
        }
        commit(root, api, &f.baseline, &tip);
        let before = crate::candidate_execution::tree::capture(root, &Default::default()).unwrap();
        let error = run(root, true).unwrap_err().to_string();
        assert!(error.contains(expected), "{error}");
        assert_eq!(
            crate::candidate_execution::tree::capture(root, &Default::default()).unwrap(),
            before
        );
        git(root, &["reset", "--hard", &parent]);
    }
    let revoked = evidence.fork();
    revoked.revoke_adoption();
    let _reader = revoked.install();
    let before = crate::candidate_execution::tree::capture(root, &Default::default()).unwrap();
    let error = run(root, true).unwrap_err().to_string();
    assert_eq!(
        crate::candidate_execution::tree::capture(root, &Default::default()).unwrap(),
        before
    );
    assert!(
        error.starts_with("missing exact authenticated adequacy adoption on PR #9;"),
        "{error}"
    );
    drop(_reader);
    let _reader = evidence.install();
    let captured = crate::isolated_maintenance::route(root)
        .unwrap()
        .unwrap()
        .capture(root)
        .unwrap();
    let host = captured.evidence_host(root).unwrap();
    assert_eq!(evidence.onboarding_population(root, &host), (1, 11));
    let assembly = EvidenceVerdicts::assemble_reporting(root, &host).unwrap();
    assert!(matches!(
        assembly.original.event,
        crate::depth_record::Event::Onboarding { .. }
    ));
}

fn canonical(
    state: &life::LifecycleState,
    packet: &life::PublicationReadyPacket,
    record: &crate::depth_record::DepthRecord,
) {
    let expected: Value = serde_json::from_str(include_str!("vectors/on3-reporting.json")).unwrap();
    let value = serde_json::to_value(state).unwrap();
    for key in [
        "lifecycle_stage",
        "support_tier",
        "current_owner_command",
        "last_transition_by",
        "active_runtime_evidence_run_id",
    ] {
        assert_eq!(value[key], expected[key], "{key}");
    }
    assert_eq!(
        serde_json::to_value(&packet.required_commands).unwrap(),
        expected["required_commands"]
    );
    assert_eq!(
        serde_json::to_value(&record.acceptance_entries).unwrap(),
        expected["acceptance_entries"]
    );
    let names: Vec<_> = packet
        .runtime_evidence_paths
        .iter()
        .map(|p| p.rsplit('/').next().unwrap())
        .collect();
    assert_eq!(json!(names), expected["runtime_evidence_files"]);
    let results: Vec<_> = record.execution.as_ref().unwrap().results.iter().map(|r| json!({
        "operation":r.obligation.operation, "id":r.obligation.id, "target":r.obligation.target, "result":r.result,
    })).collect();
    assert_eq!(json!(results), expected["results"]);
}

#[test]
fn public_on3_no_route_and_maintenance_scope_refuse_without_writes() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let before =
        crate::candidate_execution::tree::capture(f.root.path(), &Default::default()).unwrap();
    let error = run(f.root.path(), true).unwrap_err().to_string();
    assert!(
        error.contains("unsupported depth write (declared enrollment)"),
        "{error}"
    );
    let _route = f.install();
    let error = run(f.root.path(), true).unwrap_err().to_string();
    assert_eq!(
        error,
        "prepare_publication: unsupported depth write (declared enrollment)"
    );
    assert_eq!(
        crate::candidate_execution::tree::capture(f.root.path(), &Default::default()).unwrap(),
        before
    );
}

fn in1_refusals(
    f: &Fixture,
    root: &Path,
    api: &super::super::fixture::Fake,
    parent: &str,
    head: &str,
    record: &crate::depth_record::DepthRecord,
) {
    let prior_state = crate::runtime_advertising::source::Source::tip(root, parent)
        .unwrap()
        .read(STATE)
        .unwrap();
    write(root, STATE, prior_state);
    git(root, &["reset", "--soft", parent]);
    commit(root, api, &f.baseline, parent);
    let error = check_integration(root, parent).unwrap_err();
    assert!(
        error.contains(
            "ON3 reporting E requires the preserved publication_ready lifecycle checkpoint"
        ),
        "{error}"
    );
    git(root, &["reset", "--hard", head]);
    let execution = record.execution.as_ref().unwrap();
    let receipt_ref = &execution.results[0].evidence[0];
    for remove in [false, true] {
        if remove {
            fs::remove_file(root.join(&receipt_ref.path)).unwrap();
        } else {
            write(root, &receipt_ref.path, b"{}\n");
        }
        git(root, &["reset", "--soft", parent]);
        commit(root, api, &f.baseline, parent);
        let error = check_integration(root, parent).unwrap_err();
        assert!(
            error.contains("ON3 reporting evidence is missing or changed"),
            "{error}"
        );
        git(root, &["reset", "--hard", head]);
    }
    // A maintenance receipt cannot supply this onboarding E, even after all candidate hashes
    // are made internally consistent. Independent host assembly still binds the approval Event.
    let raw = SyntheticHost::maintenance_receipt();
    let receipt =
        crate::depth_evidence::EvidenceReceipt::parse(std::str::from_utf8(&raw).unwrap()).unwrap();
    assert!(matches!(
        receipt.scope.event,
        crate::depth_record::Event::Maintenance { .. }
    ));
    write(root, &receipt_ref.path, &raw);
    let mut forged = record.clone();
    let execution = forged.execution.as_mut().unwrap();
    execution.results[0].evidence[0].sha256 = hex::encode(Sha256::digest(&raw));
    execution.identity = crate::depth_record::part_identity(execution).unwrap();
    write(root, RECORD, serde_json::to_vec_pretty(&forged).unwrap());
    git(root, &["reset", "--soft", parent]);
    commit(root, api, &f.baseline, parent);
    let error = check_integration(root, parent).unwrap_err();
    assert!(
        error.contains("ON3 E differs from authenticated onboarding reporting assembly"),
        "{error}"
    );
    git(root, &["reset", "--hard", head]);
    let mut forged = record.clone();
    forged.acceptance_entries.push(crate::depth_record::AcceptanceEntry::Publication {
        packet: crate::depth_record::FileRef {path: "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/publication-ready.json".into(), sha256: "a".repeat(64)},
        record_sha256: record.stated_sha256().unwrap(),
    });
    write(root, RECORD, serde_json::to_vec_pretty(&forged).unwrap());
    git(root, &["reset", "--soft", parent]);
    commit(root, api, &f.baseline, parent);
    let error = check_integration(root, parent).unwrap_err();
    assert!(
        error.contains("new acceptance entry has no supported owning authority"),
        "{error}"
    );
    git(root, &["reset", "--hard", head]);
}

// Publish only an immutable real-writer checkpoint, after scoped host guards have dropped.
fn template() -> &'static (Fixture, SyntheticHost) {
    static TEMPLATE: std::sync::OnceLock<(Fixture, SyntheticHost)> = std::sync::OnceLock::new();
    TEMPLATE.get_or_init(|| {
        use crate::depth_acceptance::admission::{
            self, freeze::authority::Observation, Generation,
        };
        let f = public_on1::declared_chain_with_host_approval();
        let _route = f.install();
        let parent = head(&f);
        fr2::run(&f);
        let frozen = candidate(&f, &parent);
        let publication = f.resolve().unwrap().admission_publication().unwrap();
        let record = crate::depth_record::DepthRecord::parse(
            &fs::read_to_string(f.root.path().join(RECORD)).unwrap(),
            "depth_fixture",
            "0.2.0",
        )
        .unwrap();
        let observation = Observation {
            repository: publication.authority.repository.clone(),
            predecessor: admission::emitter::object(f.root.path(), &parent).unwrap(),
            generation: Generation::from_record(&record),
            policy: publication.authority.policy.clone(),
            writer: fr2::variables(),
        };
        f.api.3.put(
            &format!("freeze-inputs/{frozen}"),
            serde_json::to_value(&observation).unwrap(),
        );
        let context = super::super::fr3_reporting::collect_checkpoint(&f, &publication, &frozen);
        check_integration(f.root.path(), &parent).unwrap();
        assert!(f
            .api
            .3
            .records()
            .iter()
            .any(|r| r.candidate.commit == frozen
                && r.successful()
                && r.checkpoint == admission::Checkpoint::RequestFreeze));
        f.host();
        local(&f, &frozen);
        check_integration(f.root.path(), &parent).unwrap();
        publication
            .resolve(f.root.path(), std::slice::from_ref(&frozen))
            .unwrap()
            .require_reporting(&context)
            .unwrap();
        implement(&f, f.root.path(), &f.api.3);
        let evidence = SyntheticHost::onboarding(f.root.path());
        let implemented = head(&f);
        evidence.staging(&frozen);
        evidence.include_reporting(&f.api.3);
        {
            let (_api, _route) = install_route(&f, f.root.path(), &f.api.3);
            let _reader = evidence.install();
            let captured = crate::isolated_maintenance::route(f.root.path())
                .unwrap()
                .unwrap()
                .capture(f.root.path())
                .unwrap();
            let host = captured.evidence_host(f.root.path()).unwrap();
            let error = EvidenceVerdicts::assemble_reporting(f.root.path(), &host)
                .err()
                .unwrap();
            assert_eq!(
                error,
                "current producer inventory differs from approved staging"
            );
        }
        // The genuine execution precedes source approval. The retained exact bytes become current
        // only after the actual implementation candidate passes real IN1 and protected readback.
        {
            let (_api, _route) = install_route(&f, f.root.path(), &f.api.3);
            check_integration(f.root.path(), &frozen).unwrap();
        }
        f.host();
        local(&f, &implemented);
        check_integration(f.root.path(), &frozen).unwrap();
        evidence.staging(&implemented);
        drop(_route);
        (f, evidence)
    })
}

fn connected(repaired: bool, owning_only: bool) {
    let (template, evidence) = template();
    let root = tempfile::tempdir().unwrap();
    git(
        template.root.path(),
        &[
            "clone",
            "--quiet",
            "--no-hardlinks",
            ".",
            root.path().to_str().unwrap(),
        ],
    );
    // The private clone must also commit without caller-global Git identity.
    git(root.path(), &["config", "user.name", "Binding Fixture"]);
    git(
        root.path(),
        &["config", "user.email", "binding@example.invalid"],
    );
    let api = Arc::new(super::super::fixture::Fake(
        std::sync::Mutex::new(template.api.0.lock().unwrap().clone()),
        Default::default(),
        Default::default(),
        Arc::new(template.api.3.fork()),
        Default::default(),
    ));
    let f = Fixture {
        root,
        api,
        baseline: template.baseline.clone(),
        subject: template.subject,
    };
    prove(
        &f,
        f.root.path(),
        &f.api.3,
        evidence.fork(),
        repaired,
        owning_only,
    );
}

#[test]
fn public_on3_connected_active_reporting_e_and_real_in1() {
    connected(false, false);
}

#[test]
fn public_on3_connected_repaired_runtime_selection_and_real_in1() {
    connected(true, false);
}

/// Slice 7 consumes the same actual ON3 checkpoint and independent admission/observation,
/// then exercises private owning predicates. The ON3 refusal matrices retain their own tests.
#[test]
fn private_onboarding_predicates_from_connected_on3() {
    connected(false, true);
}
