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
    let normalized: Record = serde_json::from_slice(inputs).unwrap();
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
        record.inputs_identity().unwrap(),
        "79493c986ea198827b223508785188dc8b9a36204f911acc35086ff0ff73b5ec"
    );
}
