//! Genuine MA2 with a complete original request commit for the population proof.
use super::*;

#[test]
fn fr3_reporting_context_real_ma2_and_later_execution_population() {
    let (f, mut args) = prepare::workspace();
    let root = f.root.path();
    let _installed = f.install();
    args.request_commit = f.baseline.clone();
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
    apply_prepare_plan(root, &build_prepare_plan(root, &args).unwrap()).unwrap();
    let freeze = super::super::in1_integration::candidate(&f, &parent);
    let publication = f.resolve().unwrap().admission_publication().unwrap();
    let record = DepthRecord::parse(
        &fs::read_to_string(root.join(crate::depth_ci::onboarding::RECORD)).unwrap(),
        "depth_fixture",
        "0.2.0",
    )
    .unwrap();
    let observation = Observation {
        repository: publication.authority.repository.clone(),
        predecessor: admission::emitter::object(root, &parent).unwrap(),
        generation: Generation::from_record(&record),
        policy: publication.authority.policy.clone(),
        writer: Writer::Maintenance,
    };
    f.api.3.put(
        &format!("freeze-inputs/{freeze}"),
        serde_json::to_value(observation).unwrap(),
    );
    let context = super::super::fr3_reporting::before(&f, &publication, &record, &freeze);
    check_integration(root, &parent).unwrap();
    f.host();
    git(
        root,
        &["update-ref", "refs/remotes/origin/staging", &freeze],
    );
    write(
        root,
        "docs/fr3-implementation.md",
        "execution-required implementation candidate",
    );
    let readiness = super::super::in1_integration::candidate(&f, &freeze);
    check_integration(root, &freeze).unwrap();
    let reached = f
        .api
        .3
        .records()
        .into_iter()
        .find(|record| record.candidate.commit == readiness)
        .unwrap();
    assert_eq!(reached.checkpoint, Checkpoint::ExecutionRequired);
    git(root, &["reset", "--hard", &freeze]);
    super::super::fr3_reporting::after(&f, &publication, &context);
    super::super::fr3_reporting::freeze_after_readiness(
        &f,
        &publication,
        &freeze,
        &readiness,
        &record,
    );
}
