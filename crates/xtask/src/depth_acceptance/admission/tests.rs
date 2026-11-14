use super::*;
use fake::MemoryHost;
use serde_json::{json, Value};
use std::{
    path::Path,
    sync::{Arc, OnceLock},
};

struct Fixture {
    root: tempfile::TempDir,
    publication: Publication,
    host: Arc<MemoryHost>,
}
fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let (root, publication, host) = crate::isolated_maintenance::admission_fixture();
        Fixture {
            root,
            publication,
            host,
        }
    })
}
fn retained() -> Record {
    fixture().host.records().pop().unwrap()
}
fn publication(host: MemoryHost) -> Publication {
    Publication {
        authority: fixture().publication.authority.clone(),
        host: Arc::new(host),
    }
}
fn roots() -> Vec<String> {
    vec![retained().candidate.commit]
}
fn resolve(host: MemoryHost) -> Result<history::History, String> {
    publication(host).resolve(fixture().root.path(), &roots())
}
fn publish(host: &MemoryHost, mut record: Record) -> Record {
    let claims = host
        .begin(&record.candidate.commit, &record.policy)
        .unwrap();
    record.claims = claims.clone();
    let artifact = host.upload(&claims, &record.canonical().unwrap()).unwrap();
    host.commit(
        &claims,
        &transport::commitment(&record.identity().unwrap(), artifact),
    )
    .unwrap();
    record
}

#[test]
fn canonical_record_roundtrip_bound_versions_unknown_fields_and_domains() {
    let record = retained();
    let bytes = record.canonical().unwrap();
    assert_eq!(Record::parse(&bytes).unwrap(), record);
    assert_eq!(record.identity().unwrap(), digest(DOMAIN, &bytes));
    assert_ne!(
        digest(DOMAIN, &bytes),
        digest(b"reporting-context-v1\0", &bytes)
    );
    assert!(Record::parse(&vec![b' '; RECORD_LIMIT + 1])
        .unwrap_err()
        .contains("64 KiB"));
    let mut whitespace = bytes.clone();
    whitespace.push(b'\n');
    assert!(Record::parse(&whitespace).is_err());
    let mut value: Value = serde_json::from_slice(&bytes).unwrap();
    value["schema_version"] = json!(2);
    assert!(Record::parse(&canonical(&value).unwrap()).is_err());
    value["schema_version"] = json!(1);
    value["unknown"] = json!(true);
    assert!(Record::parse(&canonical(&value).unwrap()).is_err());
    let mut large = record;
    large.accepted_origin = "x".repeat(RECORD_LIMIT);
    assert!(large.canonical().unwrap_err().contains("64 KiB"));
}

#[test]
fn authenticates_actual_in1_and_first_record_enrollment_anchor() {
    let f = fixture();
    let record = retained();
    assert_eq!(record.decision, Decision::Admitted);
    assert_eq!(record.checkpoint, Checkpoint::Bootstrap);
    assert!(record.predecessors.is_empty());
    assert_eq!(record.enrollment_anchor, f.publication.authority.anchor);
    assert!(record.producer_context.is_none());
    let history = resolve(f.host.fork()).unwrap();
    assert!(history.eligible(&record.identity().unwrap()).unwrap());
    let mut authority = f.publication.authority.clone();
    authority.anchor.tree = "f".repeat(40);
    assert!(Publication {
        authority,
        host: f.host.clone()
    }
    .resolve(f.root.path(), &roots())
    .is_err());
}

#[test]
fn refuses_candidate_json_same_name_wrong_job_attempt_and_missing_publication() {
    let record = retained();
    let jobs = format!(
        "actions/runs/{}/attempts/1/jobs?per_page=100&page=1",
        record.claims.run
    );
    for mutation in 0..9 {
        let host = fixture().host.fork();
        match mutation {
            0 => {
                host.page(transport::RUNS, "workflow_runs", vec![]);
            }
            1 => host.change(&jobs, |v| {
                v["jobs"][0]["workflow_job_key"] = json!("candidate-job");
            }),
            2 => host.change(&jobs, |v| {
                v["jobs"][0]["run_attempt"] = json!(2);
            }),
            3 => host.change(&jobs, |v| {
                v["jobs"][0]["id"] = json!(999);
            }),
            4 => {
                host.facts
                    .lock()
                    .unwrap()
                    .remove(&format!("actions/jobs/{}/logs", record.claims.job));
            }
            5 => host.change(&jobs, |v| {
                v["jobs"][0]["check_run_id"] = json!(999);
            }),
            6 => host.change(
                &format!(
                    "actions/runs/{}/artifacts?per_page=100&page=1",
                    record.claims.run
                ),
                |v| {
                    v["artifacts"][0]["expired"] = json!(true);
                },
            ),
            7 => host.change(
                &format!("actions/runs/{}/attempts/1", record.claims.run),
                |v| {
                    v["policy"]["implementation"] = json!("a".repeat(64));
                },
            ),
            _ => {
                let mut facts = host.facts.lock().unwrap();
                let path = format!("actions/jobs/{}/logs", record.claims.job);
                let log = facts[&path].clone();
                facts.get_mut(&path).unwrap().extend(log);
            }
        }
        // Empty discovery cannot recover authority from a payload left in the checkout/artifact
        // store. Every other forged association fails before any record can become eligible.
        if mutation == 0 {
            let history = resolve(host).unwrap();
            assert!(!history.eligible(&record.identity().unwrap()).unwrap());
        } else {
            assert!(resolve(host).is_err(), "mutation {mutation}");
        }
    }
}

#[test]
fn retries_are_distinct_conflicts_refuse_and_roots_distinguish_inputs() {
    let first = retained();
    let host = fixture().host.fork();
    let retry = publish(&host, first.clone());
    assert_ne!(retry.identity().unwrap(), first.identity().unwrap());
    assert_eq!(
        retry.inputs_identity().unwrap(),
        first.inputs_identity().unwrap()
    );
    let history = resolve(host.fork()).unwrap();
    assert!(history.eligible(&retry.identity().unwrap()).unwrap());
    let mut refusal = first.clone();
    refusal.decision = Decision::Refused;
    refusal.reasons = vec![Reason::OwningPredicateRefused];
    publish(&host, refusal);
    assert!(resolve(host)
        .err()
        .unwrap()
        .contains("conflicting decisions"));
    let mut changed = first.clone();
    changed.protected_tip = first.candidate.clone();
    assert_ne!(
        changed.inputs_identity().unwrap(),
        first.inputs_identity().unwrap()
    );
    let host = fixture().host.fork();
    changed.decision = Decision::Refused;
    changed.reasons = vec![Reason::OwningPredicateRefused];
    let other_snapshot = publish(&host, changed);
    let history = resolve(host).unwrap();
    assert!(history.eligible(&first.identity().unwrap()).unwrap());
    assert!(!history
        .eligible(&other_snapshot.identity().unwrap())
        .unwrap());
}

#[test]
fn observed_authority_distinguishes_decisions_but_unchanged_facts_still_conflict() {
    let first = retained();
    let host = fixture().host.fork();
    let mut refusal = first.clone();
    refusal.decision = Decision::Refused;
    refusal.reasons = vec![Reason::OwningPredicateRefused];
    assert_eq!(
        refusal.inputs_identity().unwrap(),
        first.inputs_identity().unwrap()
    );
    refusal.observed_authority = digest(b"test missing authority\0", b"[]");
    assert_ne!(
        refusal.inputs_identity().unwrap(),
        first.inputs_identity().unwrap()
    );
    let refused = publish(&host, refusal.clone());
    let history = resolve(host.fork()).unwrap();
    assert!(history.eligible(&first.identity().unwrap()).unwrap());
    assert!(!history.eligible(&refused.identity().unwrap()).unwrap());
    refusal.decision = Decision::Admitted;
    refusal.reasons = vec![Reason::OwningPredicatesSatisfied];
    publish(&host, refusal);
    assert!(resolve(host)
        .err()
        .unwrap()
        .contains("conflicting decisions over identical inputs"));
    let mut malformed = first;
    malformed.observed_authority = "candidate-selected".into();
    assert!(malformed.canonical().is_err());
}

#[test]
fn revocation_preserves_immutable_history_and_changes_present_eligibility() {
    let record = retained();
    let host = fixture().host.fork();
    host.put(
        "in1/revocations",
        json!({"records":[record.identity().unwrap()]}),
    );
    let history = resolve(host).unwrap();
    assert!(!history.eligible(&record.identity().unwrap()).unwrap());
    assert_eq!(
        history.predecessors(&record.candidate.commit).unwrap(),
        vec![record.identity().unwrap()]
    );
}

#[test]
fn missing_predecessors_object_mismatches_and_history_gaps_refuse() {
    let first = retained();
    for mutation in 0..4 {
        let mut record = first.clone();
        match mutation {
            0 => record.predecessors = vec!["a".repeat(64)],
            1 => record.candidate.tree = "a".repeat(40),
            2 => record
                .inputs
                .insert("missing/control.json".into(), "a".repeat(40))
                .map(|_| ())
                .unwrap_or(()),
            _ => record.checkpoint = Checkpoint::Unclassified,
        }
        // Keep host authentication valid so these refusals exercise independent reconstruction.
        record.checkout = record.candidate.clone();
        assert!(
            history::resolve(
                fixture().root.path(),
                &fixture().publication.authority,
                &[record],
                &roots()
            )
            .is_err(),
            "mutation {mutation}"
        );
    }
}

struct Changes {
    host: MemoryHost,
    reads: std::sync::atomic::AtomicUsize,
    path: String,
}
impl Host for Changes {
    fn json(&self, path: &str) -> Result<Value, String> {
        let mut value = self.host.json(path)?;
        if path == self.path && self.reads.fetch_add(1, std::sync::atomic::Ordering::SeqCst) > 0 {
            value["changed"] = json!(true);
        }
        Ok(value)
    }
    fn bytes(&self, path: &str, limit: usize) -> Result<Vec<u8>, String> {
        self.host.bytes(path, limit)
    }
    fn event(&self) -> Result<String, String> {
        self.host.event()
    }
    fn begin(&self, _: &str, _: &Policy) -> Result<Claims, String> {
        unreachable!()
    }
    fn upload(&self, _: &Claims, _: &[u8]) -> Result<u64, String> {
        unreachable!()
    }
    fn commit(&self, _: &Claims, _: &str) -> Result<(), String> {
        unreachable!()
    }
}
#[test]
fn roots_and_populations_are_rechecked_after_history_and_at_consumption() {
    for path in [
        "git/ref/heads/staging".to_owned(),
        format!("{}?per_page=100&page=1", transport::RUNS),
    ] {
        let publication = Publication {
            authority: fixture().publication.authority.clone(),
            host: Arc::new(Changes {
                host: fixture().host.fork(),
                reads: 0.into(),
                path,
            }),
        };
        assert!(publication
            .resolve(fixture().root.path(), &roots())
            .err()
            .unwrap()
            .contains("changed"));
    }
    let host = Arc::new(fixture().host.fork());
    let publication = Publication {
        authority: fixture().publication.authority.clone(),
        host: host.clone(),
    };
    let history = publication
        .resolve(fixture().root.path(), &roots())
        .unwrap();
    host.root(&fixture().publication.authority.anchor.commit);
    assert!(history.recheck().unwrap_err().contains("changed"));
}

#[test]
fn each_host_stream_and_control_bound_overflow_refuses() {
    let first = retained();
    for (path, field, value) in [
        (
            format!("{}?per_page=100&page=1", transport::RUNS),
            "total_count",
            json!(1001),
        ),
        (
            format!("{}?per_page=100&page=1", transport::RUNS),
            "attempts",
            json!(101),
        ),
        (
            format!(
                "actions/runs/{}/attempts/1/jobs?per_page=100&page=1",
                first.claims.run
            ),
            "total_count",
            json!(1001),
        ),
        (
            format!(
                "actions/runs/{}/artifacts?per_page=100&page=1",
                first.claims.run
            ),
            "total_count",
            json!(1001),
        ),
    ] {
        let host = fixture().host.fork();
        host.change(&path, |v| {
            if field == "attempts" {
                v["workflow_runs"][0]["run_attempt"] = value;
            } else {
                v[field] = value;
            }
        });
        assert!(resolve(host).is_err());
    }
    assert!(Record::parse(&vec![b' '; 65_537]).is_err());
}

#[test]
fn fake_tip_advance_has_no_admission_issuance_side_effect() {
    let host = fixture().host.fork();
    let original = host.records();
    host.root(&fixture().publication.authority.anchor.commit);
    assert_eq!(host.records(), original);
}

fn git(root: &Path, args: &[&str]) -> String {
    let output = crate::git_inputs::command(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}

#[test]
fn every_retry_is_authenticated_and_wrong_attempt_commitment_refuses() {
    let first = retained();
    let host = fixture().host.fork();
    let claims = host.retry(&first);
    let mut retry = first.clone();
    retry.claims = claims.clone();
    let artifact = host.upload(&claims, &retry.canonical().unwrap()).unwrap();
    host.commit(
        &claims,
        &transport::commitment(&retry.identity().unwrap(), artifact),
    )
    .unwrap();
    let history = resolve(host.fork()).unwrap();
    assert!(history.eligible(&first.identity().unwrap()).unwrap());
    assert!(history.eligible(&retry.identity().unwrap()).unwrap());
    // Copy attempt 1's exact commitment into attempt 2's independently enumerated job log.
    let old = host
        .bytes(
            &format!("actions/jobs/{}/logs", first.claims.job),
            16_777_216,
        )
        .unwrap();
    host.facts
        .lock()
        .unwrap()
        .insert(format!("actions/jobs/{}/logs", claims.job), old);
    assert!(resolve(host).is_err());
}

#[test]
fn unique_commit_traversal_accepts_1000_and_refuses_1001_without_truncation() {
    use std::{io::Write, process::Stdio};
    let root = tempfile::tempdir().unwrap();
    git(
        root.path(),
        &[
            "clone",
            "--quiet",
            "--no-hardlinks",
            fixture().root.path().to_str().unwrap(),
            ".",
        ],
    );
    let anchor = &fixture().publication.authority.anchor.commit;
    let mut stream = Vec::new();
    for index in 1..=1000 {
        let parent = if index == 1 {
            anchor.clone()
        } else {
            format!(":{}", index - 1)
        };
        let message = format!("bounded history {index}\n");
        stream.extend_from_slice(format!("commit refs/heads/bound\nmark :{index}\ncommitter Fixture <fixture@example.invalid> {index} +0000\ndata {}\n{message}from {parent}\n\n",message.len()).as_bytes());
    }
    let mut import = crate::git_inputs::command(root.path())
        .args(["fast-import", "--quiet"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    import.stdin.take().unwrap().write_all(&stream).unwrap();
    assert!(import.wait().unwrap().success());
    let tip = git(root.path(), &["rev-parse", "refs/heads/bound"]);
    let at_limit = git(root.path(), &["rev-parse", "refs/heads/bound^"]);
    history::resolve(
        root.path(),
        &fixture().publication.authority,
        &[],
        &[at_limit],
    )
    .unwrap();
    assert!(
        history::resolve(root.path(), &fixture().publication.authority, &[], &[tip])
            .err()
            .unwrap()
            .contains("unique commit traversal bound")
    );
}

#[test]
fn verifies_raw_object_hashes_and_traverses_second_parent_to_the_anchor() {
    let root = tempfile::tempdir().unwrap();
    git(
        root.path(),
        &[
            "clone",
            "--quiet",
            "--no-hardlinks",
            fixture().root.path().to_str().unwrap(),
            ".",
        ],
    );
    git(root.path(), &["config", "user.name", "History"]);
    git(
        root.path(),
        &["config", "user.email", "history@example.invalid"],
    );
    let main = git(root.path(), &["rev-parse", "HEAD"]);
    // A validly hashed unrelated second parent cannot be hidden by first-parent traversal.
    git(root.path(), &["checkout", "--orphan", "unrelated"]);
    git(root.path(), &["commit", "-qm", "unrelated root"]);
    let other = git(root.path(), &["rev-parse", "HEAD"]);
    git(root.path(), &["checkout", "--detach", &main]);
    git(
        root.path(),
        &[
            "merge",
            "--allow-unrelated-histories",
            "--no-ff",
            "-qm",
            "join",
            &other,
        ],
    );
    let merge = git(root.path(), &["rev-parse", "HEAD"]);
    assert!(
        history::resolve(root.path(), &fixture().publication.authority, &[], &[merge])
            .err()
            .unwrap()
            .contains("history gap")
    );
    // Replace a local loose blob's contents under its old filename. Hash verification detects
    // the inconsistency independently of the claimed tree and record identities.
    let record = retained();
    let blob = record.registry_blob;
    let path = root
        .path()
        .join(".git/objects")
        .join(&blob[..2])
        .join(&blob[2..]);
    let output = std::process::Command::new("python3").args(["-c", "import sys,zlib,pathlib; p=pathlib.Path(sys.argv[1]); b=zlib.decompress(p.read_bytes()); p.write_bytes(zlib.compress(b.replace(b'codex',b'cOdex',1)))"]).arg(path).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut objects = super::objects::Objects::new(root.path());
    assert!(objects
        .read("blob", &blob)
        .unwrap_err()
        .contains("hash mismatch"));
}

#[test]
fn host_pagination_accepts_the_run_bound_and_refuses_gaps_duplicates_and_page_overflow() {
    let host = fixture().host.fork();
    for page in 1..=10 {
        let rows: Vec<_> = (1..=100)
            .map(|offset| json!({"id":(page-1)*100+offset}))
            .collect();
        host.put(
            &format!("bound?per_page=100&page={page}"),
            json!({"total_count":1000,"rows":rows}),
        );
    }
    assert_eq!(
        transport::list(&host, "bound", "rows").unwrap().0.len(),
        1000
    );
    for mutation in 0..3 {
        let host = host.fork();
        host.change("bound?per_page=100&page=1", |v| match mutation {
            0 => {
                v["rows"].as_array_mut().unwrap().pop();
            }
            1 => v["rows"][1]["id"] = v["rows"][0]["id"].clone(),
            _ => {
                let row = v["rows"][0].clone();
                v["rows"].as_array_mut().unwrap().push(row);
            }
        });
        assert!(transport::list(&host, "bound", "rows").is_err());
    }
}

#[test]
fn oversized_archives_logs_and_objects_refuse_without_truncation() {
    let first = retained();
    let host = fixture().host.fork();
    host.facts.lock().unwrap().insert(
        format!("actions/jobs/{}/logs", first.claims.job),
        vec![b' '; 16_777_217],
    );
    assert!(resolve(host).is_err());
    let host = fixture().host.fork();
    let artifacts = host
        .json(&format!(
            "actions/runs/{}/artifacts?per_page=100&page=1",
            first.claims.run
        ))
        .unwrap();
    let id = artifacts["artifacts"][0]["id"].as_u64().unwrap();
    host.facts
        .lock()
        .unwrap()
        .insert(format!("actions/artifacts/{id}/zip"), vec![0; 2_097_153]);
    assert!(resolve(host).is_err());
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    let path = root.path().join("large");
    std::fs::write(&path, vec![b'x'; 16_777_217]).unwrap();
    let blob = git(root.path(), &["hash-object", "-w", path.to_str().unwrap()]);
    let mut objects = super::objects::Objects::new(root.path());
    assert!(objects
        .read("blob", &blob)
        .unwrap_err()
        .contains("object bound"));
    git(root.path(), &["config", "user.name", "Object fixture"]);
    git(
        root.path(),
        &["config", "user.email", "object@example.invalid"],
    );
    git(
        root.path(),
        &[
            "tag",
            "-a",
            "large-tag",
            &blob,
            "-m",
            "small tag to huge blob",
        ],
    );
    let tag = git(root.path(), &["rev-parse", "large-tag"]);
    assert!(
        git(root.path(), &["cat-file", "-s", &tag])
            .parse::<usize>()
            .unwrap()
            < 1024
    );
    // A malformed regular-file tree entry points at the small tag. cat-file blob would
    // dereference it and buffer the huge target despite measuring only the tag's size.
    let mut tree = b"100644 admission.json\0".to_vec();
    tree.extend(hex::decode(&tag).unwrap());
    let tree_file = root.path().join("malformed-tree");
    std::fs::write(&tree_file, tree).unwrap();
    let tree = git(
        root.path(),
        &[
            "hash-object",
            "--literally",
            "-t",
            "tree",
            "-w",
            tree_file.to_str().unwrap(),
        ],
    );
    let commit = git(
        root.path(),
        &["commit-tree", &tree, "-m", "malformed tag entry"],
    );
    let object = objects.commit(&commit).unwrap().object;
    assert!(objects
        .blob(&object, "admission.json")
        .unwrap_err()
        .contains("object type mismatch"));
}

#[test]
fn concurrent_host_attempt_registration_preserves_the_complete_population() {
    let first = retained();
    let host = Arc::new(fixture().host.fork());
    let barrier = Arc::new(std::sync::Barrier::new(16));
    std::thread::scope(|scope| {
        for _ in 0..16 {
            let host = host.clone();
            let barrier = barrier.clone();
            let record = &first;
            scope.spawn(move || {
                barrier.wait();
                host.begin(&record.candidate.commit, &record.policy)
                    .unwrap();
            });
        }
    });
    let rows = host
        .json(&format!("{}?per_page=100&page=1", transport::RUNS))
        .unwrap();
    assert_eq!(rows["total_count"], 17);
    assert_eq!(rows["workflow_runs"].as_array().unwrap().len(), 17);
}

#[test]
fn policy_identity_covers_git_policy_and_dependency_closure() {
    let policy = &fixture().publication.authority.policy;
    for path in [
        "crates",
        "Cargo.toml",
        "Cargo.lock",
        ".github/workflows/ci.yml",
        "scripts/depth_ci_archive.py",
    ] {
        assert!(
            policy.closure.contains_key(path),
            "missing policy closure {path}"
        );
    }
}

mod lineage;
mod policy;
mod vectors;
