use super::*;

#[test]
fn complete_tree_equality_includes_bytes_modes_kinds_presence_and_preserved_paths() {
    let before = Tree::from([(
        "Cargo.toml".into(),
        Entry::file("100644", b"manifest".to_vec()),
    )]);
    let rendered = vec![("packet/report.json".into(), Some(b"report".to_vec()))];
    let plan = operations(
        &before,
        &ValidatedAcquisitionDelta::empty(),
        rendered.clone(),
    )
    .unwrap();
    assert_eq!(
        plan,
        super::operations(&before, &ValidatedAcquisitionDelta::empty(), rendered).unwrap()
    );
    plan.require_equal(&plan.expected).unwrap();
    for path in [
        "Cargo.toml",
        "Cargo.lock",
        "build.rs",
        ".github/workflows/ci.yml",
        "unrelated",
        "packet/report.json",
        "expected-output/report.json",
    ] {
        let mut changed = plan.expected.clone();
        changed.insert(path.into(), Entry::file("100644", b"extra".to_vec()));
        assert!(plan.require_equal(&changed).is_err(), "{path}");
    }
    for path in plan.expected.keys() {
        let mut changed = plan.expected.clone();
        changed.remove(path);
        assert!(plan.require_equal(&changed).is_err());
        for (mode, kind) in [("100755", "blob"), ("120000", "blob"), ("160000", "commit")] {
            let mut changed = plan.expected.clone();
            let entry = changed.get_mut(path).unwrap();
            entry.mode = mode.into();
            entry.kind = kind.into();
            assert!(plan.require_equal(&changed).is_err());
        }
    }
}

#[test]
fn planner_preserves_symlinks_gitlinks_and_binary_blobs() {
    let mut link = Entry::file("120000", b"../target".to_vec());
    let before = Tree::from([
        ("link".into(), link.clone()),
        ("binary".into(), Entry::file("100755", vec![0, 255, 1])),
        (
            "submodule".into(),
            Entry {
                mode: "160000".into(),
                kind: "commit".into(),
                bytes: b"1234567890123456789012345678901234567890".to_vec(),
            },
        ),
    ]);
    let plan = operations(&before, &ValidatedAcquisitionDelta::empty(), []).unwrap();
    assert_eq!(plan.expected, before);
    link.mode = "100644".into();
    let mut candidate = before;
    candidate.insert("link".into(), link);
    assert!(plan.require_equal(&candidate).is_err());
}

#[test]
fn acquisition_authority_is_empty_only_including_presence_modes_and_kinds() {
    let before = Tree::from([(
        "input.json".into(),
        Entry::file("100644", b"acquired".to_vec()),
    )]);
    let outputs = vec!["input.json".into(), "artifacts.lock.json".into()];
    acquisition::validate(&before, &before, &outputs).unwrap();
    for mutation in 0..5 {
        let mut candidate = before.clone();
        match mutation {
            0 => {
                candidate.remove("input.json");
            }
            1 => {
                candidate.insert(
                    "artifacts.lock.json".into(),
                    Entry::file("100644", b"{}".to_vec()),
                );
            }
            2 => candidate.get_mut("input.json").unwrap().mode = "100755".into(),
            3 => candidate.get_mut("input.json").unwrap().mode = "120000".into(),
            _ => candidate.get_mut("input.json").unwrap().bytes.push(b' '),
        }
        assert_eq!(
            acquisition::validate(&before, &candidate, &outputs).unwrap_err(),
            "acquisition authority unavailable"
        );
    }
}

#[test]
fn deletion_and_output_collisions_are_explicit() {
    let before = Tree::from([("stale".into(), Entry::file("100644", b"old".to_vec()))]);
    let plan = operations(
        &before,
        &ValidatedAcquisitionDelta::empty(),
        [("stale".into(), None)],
    )
    .unwrap();
    assert!(plan.expected.is_empty());
    assert!(matches!(plan.operations["stale"], Operation::Delete(_)));
    assert!(operations(
        &before,
        &ValidatedAcquisitionDelta::empty(),
        [("stale/child".into(), Some(vec![]))]
    )
    .is_err());
    assert!(operations(
        &before,
        &ValidatedAcquisitionDelta::empty(),
        [
            ("stale".into(), Some(vec![])),
            ("stale".into(), Some(vec![]))
        ]
    )
    .is_err());
}

#[test]
fn authenticated_acquisition_is_applied_before_freeze_and_cannot_be_overwritten_or_rebased() {
    let before = Tree::from([
        (
            "retained".into(),
            Entry::file("100755", b"retained".to_vec()),
        ),
        ("replace".into(), Entry::file("100644", b"old".to_vec())),
        ("remove".into(), Entry::file("100644", b"old".to_vec())),
    ]);
    let mut candidate = before.clone();
    candidate.insert("replace".into(), Entry::file("100644", b"new".to_vec()));
    candidate.insert("create".into(), Entry::file("100644", b"created".to_vec()));
    candidate.remove("remove");
    let outputs: Vec<String> = vec![
        "replace".into(),
        "create".into(),
        "remove".into(),
        "absent".into(),
    ];
    let inventory = outputs
        .iter()
        .map(|p| (p.clone(), candidate.get(p).cloned()))
        .collect();
    let acquired = acquisition::authenticated(&before, &candidate, &outputs, &inventory).unwrap();
    let plan = operations(
        &before,
        &acquired,
        [("freeze".into(), Some(b"frozen".to_vec()))],
    )
    .unwrap();
    candidate.insert("freeze".into(), Entry::file("100644", b"frozen".to_vec()));
    plan.require_equal(&candidate).unwrap();
    assert!(matches!(
        plan.operations["replace"],
        Operation::Replace { .. }
    ));
    assert!(matches!(plan.operations["create"], Operation::Create(_)));
    assert!(matches!(plan.operations["remove"], Operation::Delete(_)));
    assert!(matches!(
        plan.operations["retained"],
        Operation::Preserve(_)
    ));
    assert!(operations(&before, &acquired, [("replace".into(), Some(vec![]))]).is_err());
    assert!(operations(&before, &acquired, [("absent".into(), Some(vec![]))]).is_err());
    let mut another_parent = before;
    another_parent.get_mut("replace").unwrap().bytes.push(b' ');
    assert!(acquired.apply(&another_parent).is_err());
}
