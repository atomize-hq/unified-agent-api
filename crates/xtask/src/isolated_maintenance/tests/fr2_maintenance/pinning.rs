use super::*;
use crate::depth_acceptance::admission::Decision;
use std::sync::atomic::Ordering;

#[test]
fn fr2_classification_is_pinned_for_refusal_and_late_freeze_flake() {
    let (f, args) = prepare::workspace();
    let root = f.root.path();
    let _installed = f.install();
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
    let candidate = super::super::in1_integration::candidate(&f, &parent);
    let context = f.resolve().unwrap();
    let publication = context.admission_publication().unwrap();
    let depth = DepthRecord::parse(
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
        predecessor: admission::emitter::object(root, &parent).unwrap(),
        generation: Generation::from_record(&depth),
        policy: publication.authority.policy.clone(),
        writer: Writer::Maintenance,
    };
    let path = format!("freeze-inputs/{candidate}");
    f.api
        .3
        .put(&path, serde_json::to_value(&observation).unwrap());
    // A failed initial host read must abort capture before any owning policy is evaluated.
    let healthy_facts = f.api.3.facts.lock().unwrap().clone();
    f.api
        .3
        .facts
        .lock()
        .unwrap()
        .insert(path.clone(), b"unavailable freeze observation".to_vec());
    let evaluated = std::cell::Cell::new(false);
    let error = Route(f.resolve().unwrap())
        .with_admitted_publication(root, &parent, || {
            evaluated.set(true);
            Ok(())
        })
        .unwrap_err();
    assert!(error.contains("evaluated inputs unavailable"), "{error}");
    assert!(!evaluated.get());
    assert!(f.api.3.records().is_empty());
    *f.api.3.facts.lock().unwrap() = healthy_facts;
    // Both reached refusals must carry the same exact-freeze classification as an admission.
    for disturbance in 0..4 {
        f.api
            .3
            .put(&path, serde_json::to_value(&observation).unwrap());
        let error = Route(f.resolve().unwrap())
            .with_admitted_publication(root, &parent, || {
                f.api
                    .3
                    .fail_freeze_after_capture
                    .store(disturbance == 1, Ordering::SeqCst);
                f.api
                    .3
                    .fail_freeze_after_upload
                    .store(disturbance == 3, Ordering::SeqCst);
                if disturbance == 2 {
                    f.api.3.facts.lock().unwrap().remove(&path);
                }
                Err::<(), _>("deliberate owning refusal".into())
            })
            .unwrap_err();
        assert!(error.contains("deliberate owning refusal"), "{error}");
        let retained = f.api.3.records().pop().unwrap();
        assert_eq!(retained.decision, Decision::Refused);
        assert_eq!(retained.checkpoint, Checkpoint::RequestFreeze);
        assert!(retained.bootstrap);
        assert!(retained.transition.is_some());
        assert!(retained.reasons.contains(&Reason::ExactFreezeTree));
        assert_eq!(
            retained.reasons.contains(&Reason::ObservationIncomplete),
            disturbance != 0
        );
        assert!(!f.api.3.fail_freeze_after_capture.load(Ordering::SeqCst));
    }
    f.api
        .3
        .put(&path, serde_json::to_value(&observation).unwrap());
    let refusals = f.api.3.records();
    Route(f.resolve().unwrap())
        .with_admitted_publication(root, &parent, || Ok(()))
        .unwrap();
    let admitted = f.api.3.records().pop().unwrap();
    assert_eq!(admitted.decision, Decision::Admitted);
    for refused in refusals {
        assert_eq!(refused.checkpoint, admitted.checkpoint);
        assert_eq!(refused.transition, admitted.transition);
        assert_eq!(refused.observed_authority, admitted.observed_authority);
        assert_eq!(
            f.api.3.inputs_identity(&refused),
            f.api.3.inputs_identity(&admitted)
        );
    }
    let error = publication
        .resolve(root, &[parent, candidate])
        .err()
        .unwrap();
    assert!(
        error.contains("conflicting decisions over identical inputs"),
        "{error}"
    );
}
