//! Real MA2 combined acquisition/freeze boundary; one fixture, shared immutable mutation inputs.
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
mod host;
mod refusals;

fn transport(root: &Path, args: &[&str]) {
    let script =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/maintenance_transport.py");
    let out = std::process::Command::new("python3")
        .arg(script)
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
fn metadata(root: &Path) {
    let (path, bytes) = crate::manifest_version_metadata::plan_at(
        crate::manifest_version_metadata::Args {
            root: Some(root.join("cli_manifests/depth_fixture")),
            rules: None,
            version: "0.2.0".into(),
            status: crate::manifest_version_metadata::Status::Reported,
            passed_targets: vec![],
            failed_targets: vec![],
            skipped_targets: vec![],
        },
        None,
        "2026-11-14T00:00:00Z",
    )
    .unwrap();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

#[test]
fn fr2_ma2_combined_acquisition_and_freeze_real_in1() {
    let (f, args) = prepare::workspace();
    let root = f.root.path();
    let _installed = f.install();
    // A genuine predecessor lockfile, with content the acquisition writer must preserve.
    let lock_path = "cli_manifests/depth_fixture/artifacts.lock.json";
    let old_lock = json!({"schema_version":1,"notes":"retained top-level bytes in producer JSON", "artifacts":[
        {"semantic_version":"0.1.0","target_triple":"x86_64-unknown-linux-gnu","sha256":"b".repeat(64),"size_bytes":7,"notes":"retained row"}
    ]});
    write(
        root,
        lock_path,
        serde_json::to_vec_pretty(&old_lock).unwrap(),
    );
    f.commit();
    apply_prepare_plan(root, &build_prepare_plan(root, &args).unwrap()).unwrap();
    f.commit();
    let parent = git(root, &["rev-parse", "HEAD"]);
    f.host();
    git(
        root,
        &["update-ref", "refs/remotes/origin/staging", &parent],
    );
    let registry = AgentRegistry::parse_unenforced(
        &fs::read_to_string(root.join(REGISTRY_RELATIVE_PATH)).unwrap(),
    )
    .unwrap();
    let acquired_plan =
        crate::manifest_acquisition::plan_for_agent(root, "depth_fixture", "0.2.0").unwrap();
    let outside = tempfile::tempdir().unwrap();
    let plan_file = outside.path().join("plan.json");
    fs::write(&plan_file, serde_json::to_vec(&acquired_plan).unwrap()).unwrap();
    prepare::acquire(root, "0.2.0");
    metadata(root);
    let mut lock = old_lock.clone();
    for target in &acquired_plan.include {
        lock["artifacts"].as_array_mut().unwrap().push(json!({
            "semantic_version":"0.2.0","target_triple":target.target_triple,"asset_name":target.asset_name,
            "download_url":target.download_url,"sha256":"a".repeat(64),"size_bytes":1
        }));
    }
    write(root, lock_path, serde_json::to_vec_pretty(&lock).unwrap());
    let inputs = outside.path().join("inputs.json");
    let delivery = outside.path().join("delivery.json");
    let preparation = outside.path().join("preparation.json");
    let bundle = outside.path().join("bundle");
    transport(
        root,
        &[
            "stage-inputs",
            "--plan",
            plan_file.to_str().unwrap(),
            "--state",
            inputs.to_str().unwrap(),
        ],
    );
    // The real guarded core is used for the staged MA2 boundary. The public installed profile's
    // clean-checkout entry policy is unchanged; its existing test-only apply shim rebuilds/checks.
    let prepared =
        crate::agent_maintenance::prepare::build_with_registry(root, &args, &registry).unwrap();
    crate::agent_maintenance::prepare::isolated_apply(root, &prepared, &registry).unwrap();
    fs::write(
        &preparation,
        serde_json::to_vec(
            &json!({"applied":true,"files":prepared.files.iter().map(|f| json!({
        "path":f.relative_path,"sha256":admission::digest(b"",&f.contents)
    })).collect::<Vec<_>>()}),
        )
        .unwrap(),
    )
    .unwrap();
    transport(
        root,
        &[
            "stage-delivery",
            "--plan",
            plan_file.to_str().unwrap(),
            "--preparation",
            preparation.to_str().unwrap(),
            "--input-state",
            inputs.to_str().unwrap(),
            "--state",
            delivery.to_str().unwrap(),
            "--bundle-dir",
            bundle.to_str().unwrap(),
        ],
    );
    transport(root, &["verify", "--state", delivery.to_str().unwrap()]);
    let modes = git(root, &["ls-files", "--stage"])
        .lines()
        .map(|line| {
            let (meta, path) = line.split_once('\t').unwrap();
            (path.to_owned(), meta.split(' ').next().unwrap().to_owned())
        })
        .collect();
    let commit = super::in1_integration::candidate(&f, &parent);
    assert_eq!(
        git(
            root,
            &["rev-list", "--count", &format!("{parent}..{commit}")]
        ),
        "1"
    );
    let publication = f.resolve().unwrap().admission_publication().unwrap();
    let predecessor = admission::emitter::object(root, &parent).unwrap();
    let candidate = admission::emitter::object(root, &commit).unwrap();
    let record_path = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";
    let record = DepthRecord::parse(
        &fs::read_to_string(root.join(record_path)).unwrap(),
        "depth_fixture",
        "0.2.0",
    )
    .unwrap();
    let observation = Observation {
        repository: publication.authority.repository.clone(),
        predecessor: predecessor.clone(),
        generation: Generation::from_record(&record),
        policy: publication.authority.policy.clone(),
        writer: Writer::Maintenance,
    };
    f.api.3.put(
        &format!("freeze-inputs/{commit}"),
        serde_json::to_value(&observation).unwrap(),
    );
    assert_eq!(
        freeze::expected(root, &predecessor, &candidate, &record, &observation).unwrap_err(),
        "acquisition authority unavailable"
    );
    assert_eq!(
        freeze::classify(
            root,
            &publication,
            &predecessor,
            &candidate,
            &record,
            Some(&observation)
        )
        .reason,
        Reason::AcquisitionAuthorityUnavailable
    );
    let acquisition = host::capture(
        &publication,
        &predecessor,
        &candidate,
        &record,
        &acquired_plan,
        &args.branch_name,
        &bundle,
        &modes,
    );
    host::observe(&f.api.3, &acquisition);
    let retained = freeze::acquisition::read(f.api.3.as_ref(), &commit)
        .unwrap()
        .unwrap();
    let (plan, proposed) = freeze::expected_authenticated(
        root,
        &predecessor,
        &candidate,
        &record,
        &observation,
        Some(&publication),
        Some(&retained),
    )
    .unwrap();
    plan.require_equal(&proposed).unwrap();
    assert_eq!(
        proposed[record_path].bytes,
        format!("{}\n", serde_json::to_string_pretty(&record).unwrap()).as_bytes()
    );
    assert!(record.execution.is_none());
    assert!(record.acceptance_entries.is_empty());
    assert!(record.obligations.is_some());
    assert_eq!(
        record.policy,
        DepthRecord::parse(
            &git(root, &["show", &format!("{parent}:{record_path}")]),
            "depth_fixture",
            "0.2.0"
        )
        .unwrap()
        .policy
    );
    let support: serde_json::Value =
        serde_json::from_slice(&proposed[crate::support_matrix::JSON_OUTPUT_PATH].bytes).unwrap();
    let row = support["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["agent"] == "depth_fixture" && r["version"] == "0.2.0")
        .unwrap();
    assert_eq!(row["pointer_promotion"], "none");
    // This wrapper-only fixture has no agent_api capability claims. Reported coverage can set
    // its existing name-level label; acquisition grants no qualified depth promise or capability.
    assert!(row["depth"]["promises"]
        .as_array()
        .unwrap()
        .iter()
        .all(|p| p["qualified"] == false));
    assert!(row["depth"]["capabilities"].as_array().unwrap().is_empty());
    assert_eq!(
        proposed[crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH].bytes,
        crate::runtime_advertising::source::Source::tip(root, &parent)
            .unwrap()
            .read(crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH)
            .unwrap()
            .as_bytes()
    );
    assert_eq!(
        crate::manifest_validate::run(crate::manifest_validate::Args {
            root: Some(root.join("cli_manifests/depth_fixture")),
            rules: None,
            schema: None,
            version_schema: None,
            mode: crate::manifest_validate::Mode::Check,
            json: false,
            tip: None,
        }),
        0
    );
    check_integration(root, &parent).unwrap();
    let decision = f
        .api
        .3
        .records()
        .into_iter()
        .find(|r| r.candidate.commit == commit)
        .unwrap();
    assert_eq!(decision.checkpoint, Checkpoint::RequestFreeze);
    assert!(decision.reasons.contains(&Reason::ExactFreezeTree));
    refusals::inventory(&plan, &proposed, &acquisition);
    refusals::host(
        &f,
        &publication,
        &retained,
        &acquired_plan,
        &args.branch_name,
    );
    // Metadata and the complete lockfile stay semantically valid, but authenticated bytes differ.
    for path in ["cli_manifests/depth_fixture/versions/0.2.0.json", lock_path] {
        let mut bytes = fs::read(root.join(path)).unwrap();
        bytes.extend(b" \n");
        write(root, path, bytes);
    }
    let substituted = super::in1_integration::candidate(&f, &parent);
    assert_eq!(
        crate::manifest_validate::run(crate::manifest_validate::Args {
            root: Some(root.join("cli_manifests/depth_fixture")),
            rules: None,
            schema: None,
            version_schema: None,
            mode: crate::manifest_validate::Mode::Check,
            json: false,
            tip: None,
        }),
        0
    );
    let mut substituted_observation = acquisition.clone();
    substituted_observation.candidate = admission::emitter::object(root, &substituted).unwrap();
    substituted_observation.reference = freeze::acquisition::Reference {
        run: 720,
        attempt: 1,
        job: 721,
        artifact: 722,
    };
    host::observe(&f.api.3, &substituted_observation);
    f.api.3.put(
        &format!("freeze-inputs/{substituted}"),
        serde_json::to_value(&observation).unwrap(),
    );
    check_integration(root, &parent).unwrap();
    let decision = f
        .api
        .3
        .records()
        .into_iter()
        .find(|r| r.candidate.commit == substituted)
        .unwrap();
    assert_eq!(decision.checkpoint, Checkpoint::ExecutionRequired);
    assert!(decision
        .reasons
        .contains(&Reason::AcquisitionAuthorityUnavailable));
    git(root, &["reset", "--hard", &commit]);
    // Genuine combined candidate plus an unrelated edit, independently associated with its own
    // acquisition job: the remaining complete-tree equality must refuse through actual IN1.
    write(root, "docs/unrelated.txt", "extra implementation bytes\n");
    let extra = super::in1_integration::candidate(&f, &parent);
    let mut extra_observation = acquisition.clone();
    extra_observation.candidate = admission::emitter::object(root, &extra).unwrap();
    extra_observation.reference = freeze::acquisition::Reference {
        run: 710,
        attempt: 1,
        job: 711,
        artifact: 712,
    };
    host::observe(&f.api.3, &extra_observation);
    f.api.3.put(
        &format!("freeze-inputs/{extra}"),
        serde_json::to_value(&observation).unwrap(),
    );
    check_integration(root, &parent).unwrap();
    let decision = f
        .api
        .3
        .records()
        .into_iter()
        .find(|r| r.candidate.commit == extra)
        .unwrap();
    assert_eq!(decision.checkpoint, Checkpoint::ExecutionRequired);
    assert!(decision.reasons.contains(&Reason::FreezeTreeMismatch));
    git(root, &["reset", "--hard", &commit]);
    f.host();
    git(
        root,
        &["update-ref", "refs/remotes/origin/staging", &commit],
    );
    check_integration(root, &parent).unwrap();
}
