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
        "5087538cbbc8c99460d218fc6256b7cbc98d196e681fbc9798f36bf479b2c1da"
    );
    assert_eq!(
        record.generation.as_ref().unwrap().identity().unwrap(),
        "141491450471d7ce945131a830e01fff76967b19d8034d7765d0088b946804f4"
    );
    assert_eq!(
        record.inputs_identity().unwrap(),
        "0b11d7f709932f9c7711c2504efa47ba2abb8208ebd16857b49779d26ef0d7da"
    );
}
