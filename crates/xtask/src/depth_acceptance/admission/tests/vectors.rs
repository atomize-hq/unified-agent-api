use super::*;

// Fixed UTF-8 JSON and SHA-256 vectors computed independently with Python's json/hashlib.
// Each digest covers the literal domain, its terminating NUL, then the fixture bytes.
#[test]
fn fixed_canonical_bytes_and_domain_digests() {
    let admission = include_bytes!("vectors/admission.json");
    let generation = include_bytes!("vectors/generation.json");
    let inputs = include_bytes!("vectors/inputs.json");
    let record: Record = serde_json::from_slice(admission).unwrap();
    assert_eq!(record.canonical().unwrap(), admission);
    assert_eq!(
        canonical(record.generation.as_ref().unwrap()).unwrap(),
        generation
    );
    let normalized: Value = serde_json::from_slice(inputs).unwrap();
    assert_eq!(canonical(&normalized).unwrap(), inputs);
    assert_eq!(Record::parse(admission).unwrap(), record);
    assert_eq!(
        record.identity().unwrap(),
        "ab204698473c5cb9cba16156b4eafc8e9effca90a996d6478103f0c865b3d358"
    );
    assert_eq!(
        record.generation.as_ref().unwrap().identity().unwrap(),
        "141491450471d7ce945131a830e01fff76967b19d8034d7765d0088b946804f4"
    );
    assert_eq!(
        record
            .inputs_identity(&[&vector_predecessor(&record)])
            .unwrap(),
        "d4d8e94e1aae330ce60f2c13c2f2b1f5c3b3ff90f14d54b19479d6b81bd9a399"
    );
}

#[test]
fn fixed_reached_decisions_with_incomplete_closing_digests() {
    let first: Record = serde_json::from_slice(include_bytes!("vectors/admission.json")).unwrap();
    for (decision, reason, identity) in [
        (
            Decision::Admitted,
            Reason::OwningPredicatesSatisfied,
            "320064f43916c987960b73578ececb81e17fe5922905ba728fa71d58174c154d",
        ),
        (
            Decision::Refused,
            Reason::OwningPredicateRefused,
            "34e074e7ce7a02d2981ab2fc08b5e6404d3c723be041e2605031d00d0c517fea",
        ),
    ] {
        let mut record = first.clone();
        record.decision = decision;
        record.reasons = vec![reason, Reason::ObservationIncomplete];
        assert_eq!(record.identity().unwrap(), identity);
        assert_eq!(Record::parse(&record.canonical().unwrap()).unwrap(), record);
        assert_eq!(
            record
                .inputs_identity(&[&vector_predecessor(&record)])
                .unwrap(),
            first
                .inputs_identity(&[&vector_predecessor(&first)])
                .unwrap()
        );
        assert!(!record.successful());
    }
}

fn vector_predecessor(record: &Record) -> Record {
    let mut prior = record.clone();
    prior.candidate.commit = "7".repeat(40);
    prior.checkout = prior.candidate.clone();
    prior.old_state = None;
    prior.new_state = record.old_state.clone();
    prior
}

#[test]
fn substantive_predecessor_set_ignores_retries_order_and_multiplicity() {
    let record: Record = serde_json::from_slice(include_bytes!("vectors/admission.json")).unwrap();
    let prior = vector_predecessor(&record);
    assert_eq!(
        digest(
            b"support-depth-in1-transition-v1\0",
            &canonical(&(&prior.old_state, &prior.new_state)).unwrap()
        ),
        "98e4ae1846fa06900dc1f159fee09db39fe3ddcaa8d3c57a8ae03988e07f1ad9"
    );
    let mut retry = prior.clone();
    retry.claims.run += 1;
    retry.claims.attempt += 1;
    retry.claims.job += 1;
    retry.claims.check += 1;
    retry.claims.event = "workflow_dispatch".into();
    assert_ne!(retry.identity().unwrap(), prior.identity().unwrap());
    let expected = record.inputs_identity(&[&prior]).unwrap();
    assert_eq!(record.inputs_identity(&[&retry, &prior]).unwrap(), expected);
    assert_eq!(record.inputs_identity(&[&prior, &retry]).unwrap(), expected);
    let mut changed = record.clone();
    changed.predecessors = vec![retry.identity().unwrap()];
    changed.claims = retry.claims.clone();
    assert_eq!(changed.inputs_identity(&[&retry]).unwrap(), expected);
    retry.new_state.as_mut().unwrap().stage =
        crate::agent_lifecycle::LifecycleStage::RuntimeIntegrated;
    assert_ne!(record.inputs_identity(&[&retry]).unwrap(), expected);
    retry = prior.clone();
    retry.candidate.commit = "8".repeat(40);
    assert_ne!(record.inputs_identity(&[&retry]).unwrap(), expected);
    assert_eq!(
        record.inputs_identity(&[&prior, &retry]).unwrap(),
        record.inputs_identity(&[&retry, &prior]).unwrap()
    );
    changed = record.clone();
    changed.checkpoint = Checkpoint::Bootstrap;
    assert_ne!(changed.inputs_identity(&[&prior]).unwrap(), expected);
}
