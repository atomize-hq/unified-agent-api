use super::*;

fn git(root: &Path, args: &[&str]) -> String {
    String::from_utf8(candidate_git::run(root, args).unwrap()).unwrap()
}
fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    fs::write(dir.path().join("tracked"), b"tracked").unwrap();
    git(dir.path(), &["add", "tracked"]);
    dir
}
fn bound(path: &str, bytes: &[u8]) -> Bound {
    Bound {
        path: path.into(),
        mode: "100644",
        bytes: bytes.to_vec(),
    }
}

#[test]
fn publish_indexes_the_validated_bytes_never_the_live_file() {
    let root = repo();
    let path = "evidence/receipt.json";
    fs::create_dir_all(root.path().join("evidence")).unwrap();
    // The live file was swapped after validation; only the validated bytes gain an identity.
    fs::write(root.path().join(path), b"swapped").unwrap();
    publish(root.path(), &[bound(path, b"validated")]).unwrap();
    assert_eq!(
        git(root.path(), &["show", &format!(":{path}")]),
        "validated"
    );
    // A path the index already holds is never overwritten.
    let error = publish(root.path(), &[bound(path, b"other")]).unwrap_err();
    assert!(error.to_string().contains("already holds"), "{error}");
    assert_eq!(
        git(root.path(), &["show", &format!(":{path}")]),
        "validated"
    );
}

#[test]
fn a_locked_index_fails_publication_and_leaves_the_index_unchanged() {
    let root = repo();
    let index = fs::read(root.path().join(".git/index")).unwrap();
    fs::write(root.path().join(".git/index.lock"), b"").unwrap();
    assert!(publish(root.path(), &[bound("evidence/r.json", b"r")]).is_err());
    fs::remove_file(root.path().join(".git/index.lock")).unwrap();
    assert_eq!(fs::read(root.path().join(".git/index")).unwrap(), index);
}

#[test]
fn unpublish_removes_only_owned_entries_and_reports_conflicts() {
    let root = repo();
    let before = git(root.path(), &["ls-files", "--stage"]);
    let published = publish(
        root.path(),
        &[
            bound("evidence/a.json", b"a"),
            bound("evidence/b.json", b"b"),
        ],
    )
    .unwrap();
    unpublish(root.path(), &published).unwrap();
    assert_eq!(git(root.path(), &["ls-files", "--stage"]), before);
    // A concurrent caller change to one owned entry is preserved and reported.
    let published = publish(
        root.path(),
        &[
            bound("evidence/a.json", b"a"),
            bound("evidence/b.json", b"b"),
        ],
    )
    .unwrap();
    let oid = hash_object(root.path(), b"concurrent").unwrap();
    git(
        root.path(),
        &[
            "update-index",
            "--cacheinfo",
            &format!("100644,{oid},evidence/b.json"),
        ],
    );
    let error = unpublish(root.path(), &published).unwrap_err();
    assert!(error.to_string().contains("evidence/b.json"), "{error}");
    let entries = git(root.path(), &["ls-files", "--stage"]);
    assert!(!entries.contains("evidence/a.json"), "{entries}");
    assert!(
        entries.contains(&format!("{oid} 0\tevidence/b.json")),
        "{entries}"
    );
}

/// Another Git process tries to stage `path` as `bytes` in the caller; returns whether it could.
fn concurrent_stage(root: &Path, path: &str, bytes: &[u8]) -> bool {
    let oid = hash_object(root, bytes).unwrap();
    candidate_git::command(root)
        .args([
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("100644,{oid},{path}"),
        ])
        .output()
        .unwrap()
        .status
        .success()
}
struct Hooked;
impl Drop for Hooked {
    fn drop(&mut self) {
        super::super::index_lock::between::clear();
    }
}
fn between(hook: impl Fn(&Path) + 'static) -> Hooked {
    super::super::index_lock::between::set(hook);
    Hooked
}

#[test]
fn a_receipt_staged_between_publish_read_and_update_is_never_overwritten() {
    let root = repo();
    let staged = std::rc::Rc::new(std::cell::Cell::new(None));
    let seen = staged.clone();
    // Between the expected-before comparison and the update, another process stages a receipt at
    // the same path and another path: the held lock refuses both, so nothing it did is lost.
    let hooked = between(move |root| {
        seen.set(Some((
            concurrent_stage(root, "evidence/r.json", b"theirs"),
            concurrent_stage(root, "other.json", b"other"),
        )))
    });
    publish(root.path(), &[bound("evidence/r.json", b"ours")]).unwrap();
    drop(hooked);
    assert_eq!(staged.get(), Some((false, false)));
    assert_eq!(git(root.path(), &["show", ":evidence/r.json"]), "ours");
    let entries = git(root.path(), &["ls-files"]);
    assert_eq!(entries, "evidence/r.json\ntracked\n");
    // Once released, the index is usable again and the lock is gone.
    assert!(!root.path().join(".git/index.lock").exists());
    assert!(concurrent_stage(root.path(), "other.json", b"other"));
}

#[test]
fn a_change_between_rollback_read_and_removal_is_never_erased() {
    let root = repo();
    let published = publish(
        root.path(),
        &[
            bound("evidence/a.json", b"a"),
            bound("evidence/b.json", b"b"),
        ],
    )
    .unwrap();
    let staged = std::rc::Rc::new(std::cell::Cell::new(None));
    let seen = staged.clone();
    let hooked =
        between(move |root| seen.set(Some(concurrent_stage(root, "evidence/b.json", b"x"))));
    unpublish(root.path(), &published).unwrap();
    drop(hooked);
    // The concurrent change was refused while the rollback held the lock; exactly the owned
    // entries were removed.
    assert_eq!(staged.get(), Some(false));
    assert_eq!(git(root.path(), &["ls-files"]), "tracked\n");
    // A change that lands before rollback takes the lock is reported, not erased.
    let published = publish(root.path(), &[bound("evidence/b.json", b"b")]).unwrap();
    assert!(concurrent_stage(root.path(), "evidence/b.json", b"theirs"));
    let error = unpublish(root.path(), &published).unwrap_err();
    assert!(
        error.to_string().contains("restoration conflict"),
        "{error}"
    );
    assert_eq!(git(root.path(), &["show", ":evidence/b.json"]), "theirs");
}

#[test]
fn an_existing_index_lock_refuses_publication_and_rollback_unchanged() {
    let root = repo();
    let published = publish(root.path(), &[bound("evidence/a.json", b"a")]).unwrap();
    let index = fs::read(root.path().join(".git/index")).unwrap();
    let lock = root.path().join(".git/index.lock");
    fs::write(&lock, b"held").unwrap();
    for error in [
        publish(root.path(), &[bound("evidence/b.json", b"b")]).unwrap_err(),
        unpublish(root.path(), &published).unwrap_err(),
    ] {
        assert!(
            error.to_string().contains("locked by another Git process"),
            "{error}"
        );
    }
    // Another process's lock is never removed, and the index is untouched.
    assert_eq!(fs::read(&lock).unwrap(), b"held");
    assert_eq!(fs::read(root.path().join(".git/index")).unwrap(), index);
}

/// Production stages nothing, so no evidence-closure file is ever admitted; under a route only
/// the exact host-staged bytes of the whole closure (receipts and the payloads they reference,
/// transitively) are, and any other byte refuses the delivery outright.
#[test]
fn only_host_staged_bytes_of_the_evidence_closure_are_admitted() {
    let record = "cli/reports/1/depth-record.json";
    let receipt = "cli/reports/1/evidence/r.json";
    let payload = "cli/reports/1/evidence/raw.txt";
    let receipt_bytes = serde_json::to_vec(&serde_json::json!({
        "cases": {"raw_output": {"path": payload, "sha256": sha256(b"raw")}}
    }))
    .unwrap();
    let tree = |payload_bytes: &[u8]| {
        let mut after = Tree::new();
        after.insert(
            record.into(),
            Node::File(
                serde_json::to_vec(&serde_json::json!({
                    "execution": {"results": [{"evidence": [{"path": receipt}]}]}
                }))
                .unwrap(),
                0o644,
            ),
        );
        after.insert(receipt.into(), Node::File(receipt_bytes.clone(), 0o644));
        after.insert(payload.into(), Node::File(payload_bytes.to_vec(), 0o644));
        after
    };
    let host: Staged = [(receipt, receipt_bytes.as_slice()), (payload, b"raw")]
        .iter()
        .map(|(p, b)| (p.to_string(), sha256(b)))
        .collect();
    let admitted = |after: &Tree, staged: Option<&Staged>| {
        bound_new_evidence(&Tree::new(), after, "runs/x", staged)
            .into_iter()
            .map(|b| b.path)
            .collect::<Vec<_>>()
    };
    let genuine = tree(b"raw");
    assert!(
        admitted(&genuine, None).is_empty(),
        "production admitted evidence"
    );
    // Under a route the whole closure is admitted, payload included.
    assert_eq!(admitted(&genuine, Some(&host)), [receipt, payload]);
    check_staged_closure(&Tree::new(), &genuine, "runs/x", Some(&host)).unwrap();
    // A substituted payload behind an unchanged receipt refuses the whole delivery.
    let substituted = tree(b"substituted");
    let error = check_staged_closure(&Tree::new(), &substituted, "runs/x", Some(&host))
        .unwrap_err()
        .to_string();
    assert!(error.contains(payload), "{error}");
    // So does a file the host never staged, even with an empty staged set.
    let error = check_staged_closure(&Tree::new(), &genuine, "runs/x", Some(&Staged::new()))
        .unwrap_err()
        .to_string();
    assert!(error.contains("not the bytes the host staged"), "{error}");
    // By path class, whether or not any record changed: with genuine E and evidence already
    // committed, a substituted, removed or added evidence file refuses, under a route or not.
    let mut removed = genuine.clone();
    removed.remove(payload);
    let mut added = genuine.clone();
    added.insert(
        "cli/reports/1/evidence/unreferenced.json".into(),
        Node::File(b"{}".to_vec(), 0o644),
    );
    for after in [&substituted, &removed, &added] {
        for staged in [None, Some(&host)] {
            assert!(check_staged_closure(&genuine, after, "runs/x", staged).is_err());
        }
    }
    // Unchanged committed evidence adds no condition, in production too.
    check_staged_closure(&genuine, &genuine, "runs/x", None).unwrap();
    // A changed evidence file in production always refuses.
    assert!(check_staged_closure(&Tree::new(), &genuine, "runs/x", None).is_err());
}

/// A payload outside any evidence destination, referenced by an unchanged committed receipt, is
/// still evidence: substituting it, or a receipt newly referencing another external file,
/// refuses in production and under a route. A receipt's source inputs are not evidence.
#[test]
fn payloads_outside_evidence_destinations_are_bound_through_their_receipts() {
    let receipt = "cli/reports/1/evidence/run/receipt.json";
    let external = "cli/reports/1/raw.txt";
    let source = "crates/cli/src/lib.rs";
    let receipt_for = |payloads: &[&str]| {
        let mut value = serde_json::json!({
            "build": {"covered_inputs": [{"path": source, "sha256": sha256(b"src")}]},
            "cases": {"raw_output": {"path": payloads[0], "sha256": sha256(b"raw")}}
        });
        if let Some(extra) = payloads.get(1) {
            value["build"]["toolchain"] = serde_json::json!({"path": extra, "sha256": "0"});
        }
        serde_json::to_vec(&value).unwrap()
    };
    let mut committed = Tree::new();
    committed.insert(receipt.into(), Node::File(receipt_for(&[external]), 0o644));
    committed.insert(external.into(), Node::File(b"raw".to_vec(), 0o644));
    committed.insert(source.into(), Node::File(b"src".to_vec(), 0o644));
    let host: Staged = Staged::new();
    // (a) substitute the external payload behind the unchanged receipt
    let mut substituted = committed.clone();
    substituted.insert(external.into(), Node::File(b"substituted".to_vec(), 0o644));
    // (b) the receipt newly references another external file
    let mut extended = committed.clone();
    extended.insert(
        receipt.into(),
        Node::File(receipt_for(&[external, "cli/reports/1/extra.txt"]), 0o644),
    );
    extended.insert(
        "cli/reports/1/extra.txt".into(),
        Node::File(b"x".to_vec(), 0o644),
    );
    for after in [&substituted, &extended] {
        for staged in [None, Some(&host)] {
            let error = check_staged_closure(&committed, after, "runs/x", staged)
                .unwrap_err()
                .to_string();
            assert!(error.contains("not the bytes the host staged"), "{error}");
        }
    }
    // A product source file the receipt names as an input may change freely.
    let mut edited = committed.clone();
    edited.insert(source.into(), Node::File(b"edited".to_vec(), 0o644));
    check_staged_closure(&committed, &edited, "runs/x", None).unwrap();
}
