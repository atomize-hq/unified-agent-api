use super::*;
use crate::{agent_lifecycle::LifecycleStage, depth_record::DepthRecord};

const STATE: &str =
    "docs/agents/lifecycle/depth-fixture-onboarding/governance/lifecycle-state.json";
const DEPTH: &str = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";

struct Lineage {
    root: tempfile::TempDir,
    publication: Publication,
    host: Arc<MemoryHost>,
    depth: DepthRecord,
    enrolled: Record,
}

#[test]
fn generation_association_follows_unadmitted_descendants_and_stops_at_a_complete_new_generation() {
    let mut f = Lineage::new();
    let old = Generation::from_record(&f.depth);
    let removed = f.commit(LifecycleStage::Enrolled, "unadmitted descendant");
    let history = f
        .publication
        .resolve(f.root.path(), &[f.enrolled.candidate.commit.clone()])
        .unwrap();
    assert!(history
        .associates_generation(f.root.path(), &removed, &old)
        .unwrap());
    if let crate::depth_record::Event::Maintenance { request_commit, .. } = &mut f.depth.event {
        *request_commit = "a".repeat(40);
    }
    std::fs::write(
        f.root.path().join(DEPTH),
        serde_json::to_vec_pretty(&f.depth).unwrap(),
    )
    .unwrap();
    let next = f.commit(
        LifecycleStage::Enrolled,
        "independently admitted new generation",
    );
    f.admit(&next, &f.enrolled);
    f.host.root(&next);
    let descendant = f.commit(LifecycleStage::Enrolled, "new generation descendant");
    let history = f.publication.resolve(f.root.path(), &[next]).unwrap();
    assert!(!history
        .associates_generation(f.root.path(), &descendant, &old)
        .unwrap());
    assert!(history
        .associates_generation(
            f.root.path(),
            &descendant,
            &Generation::from_record(&f.depth)
        )
        .unwrap());
}
impl Lineage {
    fn new() -> Self {
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
        let depth_bytes = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/depth-synthetic/record.json"
        ));
        let depth = DepthRecord::parse(depth_bytes, "depth_fixture", "0.2.0").unwrap();
        std::fs::create_dir_all(root.path().join(DEPTH).parent().unwrap()).unwrap();
        std::fs::write(root.path().join(DEPTH), depth_bytes).unwrap();
        let host = Arc::new(fixture().host.fork());
        let publication = Publication {
            authority: fixture().publication.authority.clone(),
            host: host.clone(),
        };
        let mut lineage = Self {
            root,
            publication,
            host,
            depth,
            enrolled: retained(),
        };
        let tip = lineage.commit(LifecycleStage::Enrolled, "enrolled T");
        lineage.enrolled = lineage.admit(&tip, &retained());
        lineage.host.root(&tip);
        lineage
    }
    fn commit(&self, stage: LifecycleStage, message: &str) -> String {
        let mut state: Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json"
        )))
        .unwrap();
        state["agent_id"] = json!("depth_fixture");
        state["lifecycle_stage"] = json!(stage);
        std::fs::create_dir_all(self.root.path().join(STATE).parent().unwrap()).unwrap();
        std::fs::write(
            self.root.path().join(STATE),
            serde_json::to_vec(&state).unwrap(),
        )
        .unwrap();
        git(self.root.path(), &["add", "."]);
        git(
            self.root.path(),
            &["commit", "--allow-empty", "-qm", message],
        );
        git(self.root.path(), &["rev-parse", "HEAD"])
    }
    fn admit(&self, commit: &str, prior: &Record) -> Record {
        let mut record = prior.clone();
        let mut objects = objects::Objects::new(self.root.path());
        record.parent = prior.candidate.clone();
        record.protected_tip = record.parent.clone();
        record.candidate = objects.commit(commit).unwrap().object;
        record.checkout = record.candidate.clone();
        record.old_state = objects.state(&record.parent, STATE).unwrap();
        record.new_state = objects.state(&record.candidate, STATE).unwrap();
        record.generation = Some(Generation::from_record(&self.depth));
        record.generation_sha256 = record
            .generation
            .as_ref()
            .map(Generation::identity)
            .transpose()
            .unwrap();
        record.predecessors = vec![prior.identity().unwrap()];
        record.checkpoint = Checkpoint::Unclassified;
        publish(&self.host, record)
    }
    fn check(&self, candidate: &str) -> Result<(), String> {
        self.publication
            .resolve(
                self.root.path(),
                &[self.enrolled.candidate.commit.clone(), candidate.into()],
            )?
            .check_candidate(self.root.path(), candidate, &self.depth, STATE)
    }
}

#[test]
fn unmerged_ancestor_missing_admission_refuses_even_when_candidate_restores_protected_tree() {
    let f = Lineage::new();
    let higher = f.commit(
        LifecycleStage::RuntimeIntegrated,
        "unmerged H with no producer",
    );
    let admitted = f.admit(&higher, &f.enrolled);
    let lower = f.commit(LifecycleStage::Enrolled, "L restores enrolled T tree");
    assert_eq!(
        git(f.root.path(), &["rev-parse", &format!("{lower}^{{tree}}")]),
        f.enrolled.candidate.tree
    );
    assert!(f
        .check(&lower)
        .unwrap_err()
        .contains("lifecycle progression"));
    // Drop H from complete discovery. No surviving record references its admission.
    f.host
        .change(&format!("{}?per_page=100&page=1", transport::RUNS), |v| {
            v["workflow_runs"]
                .as_array_mut()
                .unwrap()
                .retain(|r| r["id"] != admitted.claims.run);
            v["total_count"] = json!(v["workflow_runs"].as_array().unwrap().len());
        });
    assert!(f.check(&lower).unwrap_err().contains("history gap"));
}

#[test]
fn ordinary_unadmitted_ancestors_and_pending_candidate_transition_pass() {
    let f = Lineage::new();
    f.commit(LifecycleStage::Enrolled, "ordinary unadmitted ancestor");
    let candidate = f.commit(
        LifecycleStage::RuntimeIntegrated,
        "pending transition needs no own admission",
    );
    f.check(&candidate).unwrap();
}

#[test]
fn missing_refusal_predecessors_are_quarantined_and_same_key_still_conflicts() {
    let f = Lineage::new();
    let candidate = f.commit(LifecycleStage::Enrolled, "refused child");
    let mut refusal = f.admit(&candidate, &f.enrolled);
    refusal.decision = Decision::Refused;
    refusal.reasons = vec![Reason::OwningPredicateRefused];
    refusal.predecessors.clear();
    let refusal = publish(&f.host, refusal);
    f.check(&candidate).unwrap();
    let history = f
        .publication
        .resolve(
            f.root.path(),
            &[f.enrolled.candidate.commit.clone(), candidate.clone()],
        )
        .unwrap();
    assert!(history.incomplete.contains(&refusal.identity().unwrap()));
    let mut contradiction = refusal;
    contradiction.decision = Decision::Admitted;
    contradiction.reasons = vec![Reason::OwningPredicatesSatisfied];
    publish(&f.host, contradiction);
    assert!(f
        .check(&candidate)
        .unwrap_err()
        .contains("conflicting decisions over identical inputs"));
}

#[test]
fn missing_refusal_bootstrap_is_incomplete_and_still_conflicts() {
    let host = fixture().host.fork();
    host.page(transport::RUNS, "workflow_runs", vec![]);
    let mut refusal = retained();
    refusal.decision = Decision::Refused;
    refusal.reasons = vec![
        Reason::OwningPredicateRefused,
        Reason::ObservationIncomplete,
    ];
    refusal.checkpoint = Checkpoint::Unclassified;
    let refusal = publish(&host, refusal);
    let history = resolve(host.fork()).unwrap();
    assert!(history.incomplete.contains(&refusal.identity().unwrap()));
    assert!(!history.eligible(&refusal.identity().unwrap()).unwrap());
    for incomplete_admission in [false, true] {
        let conflicting = host.fork();
        let mut admission = retained();
        if incomplete_admission {
            admission.reasons.push(Reason::ObservationIncomplete);
        }
        assert_ne!(
            admission.inputs_identity(&[]).unwrap(),
            refusal.inputs_identity(&[]).unwrap()
        );
        publish(&conflicting, admission.clone());
        resolve(conflicting.fork()).unwrap();
        // The malformed checkpoint remains pinned. A reached opposite decision with the
        // same classification must conflict even when both records are quarantined.
        admission.checkpoint = refusal.checkpoint;
        publish(&conflicting, admission);
        conflicting.change(
            &format!("{}?per_page=100&page=1", transport::RUNS),
            |value| {
                value["workflow_runs"].as_array_mut().unwrap().reverse();
            },
        );
        assert!(resolve(conflicting)
            .err()
            .unwrap()
            .contains("conflicting decisions over identical inputs"));
    }
}

#[test]
fn later_parent_admission_preserves_a_refusals_original_inputs() {
    let f = Lineage::new();
    let parent = f.commit(LifecycleStage::Enrolled, "pending parent");
    let child = f.commit(LifecycleStage::Enrolled, "refused child");
    let mut refusal = f.enrolled.clone();
    refusal.parent = f.enrolled.candidate.clone();
    refusal.protected_tip = refusal.parent.clone();
    refusal.candidate = emitter::object(f.root.path(), &child).unwrap();
    refusal.checkout = refusal.candidate.clone();
    refusal.old_state = f.enrolled.new_state.clone();
    refusal.new_state = objects::Objects::new(f.root.path())
        .state(&refusal.candidate, STATE)
        .unwrap();
    refusal.predecessors = vec![f.enrolled.identity().unwrap()];
    refusal.decision = Decision::Refused;
    refusal.reasons = vec![Reason::OwningPredicateRefused];
    let refusal = publish(&f.host, refusal);
    let original = refusal.canonical().unwrap();
    f.admit(&parent, &f.enrolled);
    let history = f
        .publication
        .resolve(f.root.path(), &[f.enrolled.candidate.commit.clone(), child])
        .unwrap();
    assert!(history.incomplete.contains(&refusal.identity().unwrap()));
    assert!(!history.eligible(&refusal.identity().unwrap()).unwrap());
    assert_eq!(
        f.host
            .records()
            .into_iter()
            .find(|r| r.claims == refusal.claims)
            .unwrap()
            .canonical()
            .unwrap(),
        original
    );
}

#[test]
fn legacy_inputs_over_64k_remain_readable_in_ancestor_history() {
    let f = Lineage::new();
    for path in [DEPTH, STATE] {
        let mut bytes = std::fs::read(f.root.path().join(path)).unwrap();
        bytes.extend(vec![b' '; 65_537]);
        std::fs::write(f.root.path().join(path), bytes).unwrap();
    }
    git(f.root.path(), &["add", "."]);
    git(f.root.path(), &["commit", "-qm", "legacy input formatting"]);
    let formatted = git(f.root.path(), &["rev-parse", "HEAD"]);
    let admitted = f.admit(&formatted, &f.enrolled);
    assert!(admitted.canonical().unwrap().len() < RECORD_LIMIT);
    let candidate = f.commit(LifecycleStage::RuntimeIntegrated, "pending transition");
    f.check(&candidate).unwrap();
}

#[test]
fn missing_admission_on_second_parent_refuses() {
    let f = Lineage::new();
    let higher = f.commit(
        LifecycleStage::RuntimeIntegrated,
        "unadmitted second parent transition",
    );
    git(
        f.root.path(),
        &["reset", "--hard", &f.enrolled.candidate.commit],
    );
    let ordinary = f.commit(LifecycleStage::Enrolled, "ordinary first parent");
    let merge = git(
        f.root.path(),
        &[
            "commit-tree",
            &f.enrolled.candidate.tree,
            "-p",
            &ordinary,
            "-p",
            &higher,
            "-m",
            "pending merge",
        ],
    );
    assert!(f.check(&merge).unwrap_err().contains("history gap"));
}

#[test]
fn predecessor_retry_reordering_cannot_bypass_a_reached_refusal() {
    let f = Lineage::new();
    let parent = f.commit(LifecycleStage::Enrolled, "parent with incomplete first run");
    let mut incomplete = f.enrolled.clone();
    incomplete.candidate = emitter::object(f.root.path(), &parent).unwrap();
    incomplete.checkout = incomplete.candidate.clone();
    incomplete.parent = f.enrolled.candidate.clone();
    incomplete.protected_tip = incomplete.parent.clone();
    incomplete.old_state = f.enrolled.new_state.clone();
    incomplete.new_state = incomplete.old_state.clone();
    incomplete.predecessors = vec![f.enrolled.identity().unwrap()];
    incomplete.decision = Decision::Refused;
    incomplete.reasons = vec![Reason::ObservationIncomplete];
    let incomplete = publish(&f.host, incomplete);
    let successful = f.admit(&parent, &f.enrolled);
    let child = f.commit(LifecycleStage::Enrolled, "refused child");
    let mut refusal = successful.clone();
    refusal.candidate = emitter::object(f.root.path(), &child).unwrap();
    refusal.checkout = refusal.candidate.clone();
    refusal.parent = successful.candidate.clone();
    refusal.protected_tip = refusal.parent.clone();
    refusal.predecessors = vec![successful.identity().unwrap()];
    refusal.decision = Decision::Refused;
    refusal.reasons = vec![Reason::OwningPredicateRefused];
    let refusal = publish(&f.host, refusal);
    let mut retry = incomplete.clone();
    retry.decision = Decision::Admitted;
    retry.reasons = vec![Reason::OwningPredicatesSatisfied];
    f.host.retry_on_next_begin(&incomplete);
    let retry = publish(&f.host, retry);
    let history = f
        .publication
        .resolve(
            f.root.path(),
            &[f.enrolled.candidate.commit.clone(), child.clone()],
        )
        .unwrap();
    assert_eq!(
        history.candidate_predecessors(&child).unwrap(),
        vec![retry.identity().unwrap()]
    );
    let mut contradiction = refusal;
    contradiction.predecessors = history.candidate_predecessors(&child).unwrap();
    assert_eq!(
        contradiction.inputs_identity(&[&retry]).unwrap(),
        contradiction.inputs_identity(&[&successful]).unwrap()
    );
    contradiction.decision = Decision::Admitted;
    contradiction.reasons = vec![Reason::OwningPredicatesSatisfied];
    publish(&f.host, contradiction);
    assert!(f
        .publication
        .resolve(f.root.path(), &[f.enrolled.candidate.commit.clone(), child])
        .err()
        .unwrap()
        .contains("conflicting decisions over identical inputs"));
}

#[test]
fn incomplete_admission_with_superseded_frontier_allows_recovery_and_other_candidates() {
    let f = Lineage::new();
    let parent = f.commit(LifecycleStage::Enrolled, "concurrently admitted parent");
    let child = f.commit(LifecycleStage::Enrolled, "captured child");
    // Capture C before P has an admission, then publish P before retaining C's reached result.
    let mut captured = f.enrolled.clone();
    captured.parent = f.enrolled.candidate.clone();
    captured.protected_tip = captured.parent.clone();
    captured.candidate = emitter::object(f.root.path(), &child).unwrap();
    captured.checkout = captured.candidate.clone();
    captured.old_state = f.enrolled.new_state.clone();
    captured.new_state = captured.old_state.clone();
    captured.predecessors = vec![f.enrolled.identity().unwrap()];
    let admitted_parent = f.admit(&parent, &f.enrolled);
    captured.reasons.push(Reason::ObservationIncomplete);
    let incomplete = publish(&f.host, captured.clone());
    let history = f
        .publication
        .resolve(
            f.root.path(),
            &[f.enrolled.candidate.commit.clone(), child.clone()],
        )
        .unwrap();
    assert!(!history.eligible(&incomplete.identity().unwrap()).unwrap());
    assert_eq!(
        history.candidate_predecessors(&child).unwrap(),
        vec![admitted_parent.identity().unwrap()]
    );
    let other = f.commit(LifecycleStage::Enrolled, "unrelated later candidate");
    f.check(&other).unwrap();
    let mut retry = captured;
    retry.predecessors = history.candidate_predecessors(&child).unwrap();
    retry.reasons = vec![Reason::OwningPredicatesSatisfied];
    let retry = publish(&f.host, retry);
    let history = f
        .publication
        .resolve(
            f.root.path(),
            &[f.enrolled.candidate.commit.clone(), other.clone()],
        )
        .unwrap();
    assert!(history.eligible(&retry.identity().unwrap()).unwrap());
    f.check(&other).unwrap();
    let mut refusal = retry;
    refusal.decision = Decision::Refused;
    refusal.reasons = vec![Reason::OwningPredicateRefused];
    publish(&f.host, refusal);
    assert!(f
        .publication
        .resolve(f.root.path(), &[f.enrolled.candidate.commit.clone(), other])
        .err()
        .unwrap()
        .contains("conflicting decisions over identical inputs"));
}

#[test]
fn history_deficient_admissions_cannot_grant_frontier_progression_or_transition_history() {
    let f = Lineage::new();
    let higher = f.commit(LifecycleStage::RuntimeIntegrated, "quarantined transition");
    let mut deficient = f.enrolled.clone();
    deficient.parent = f.enrolled.candidate.clone();
    deficient.protected_tip = deficient.parent.clone();
    deficient.candidate = emitter::object(f.root.path(), &higher).unwrap();
    deficient.checkout = deficient.candidate.clone();
    deficient.old_state = f.enrolled.new_state.clone();
    deficient.new_state = objects::Objects::new(f.root.path())
        .state(&deficient.candidate, STATE)
        .unwrap();
    deficient.predecessors = vec!["f".repeat(64)];
    let deficient = publish(&f.host, deficient);
    let later = f.commit(LifecycleStage::Enrolled, "later lower candidate");
    let history = f
        .publication
        .resolve(
            f.root.path(),
            &[f.enrolled.candidate.commit.clone(), later.clone()],
        )
        .unwrap();
    assert!(history.incomplete.contains(&deficient.identity().unwrap()));
    assert!(!history.eligible(&deficient.identity().unwrap()).unwrap());
    assert_eq!(
        history.predecessors(&higher).unwrap(),
        vec![f.enrolled.identity().unwrap()]
    );
    let generation = Generation::from_record(&f.depth);
    // The quarantined higher-stage decision supplies no association authority of its own.
    // Its complete enrolled predecessor still associates this descendant with the generation.
    assert!(history
        .associates_generation(f.root.path(), &higher, &generation)
        .unwrap());
    assert!(history
        .associates_generation(f.root.path(), &f.enrolled.candidate.commit, &generation)
        .unwrap());
    history
        .check_progression(&later, &generation, LifecycleStage::Enrolled)
        .unwrap();
    assert!(history
        .check_candidate(f.root.path(), &later, &f.depth, STATE)
        .unwrap_err()
        .contains("generation history gap"));
    // A record citing the quarantined admission also loses authority without poisoning history.
    let mut descendant = deficient.clone();
    descendant.candidate = emitter::object(f.root.path(), &later).unwrap();
    descendant.checkout = descendant.candidate.clone();
    descendant.parent = deficient.candidate.clone();
    descendant.protected_tip = descendant.parent.clone();
    descendant.old_state = deficient.new_state.clone();
    descendant.new_state = f.enrolled.new_state.clone();
    descendant.predecessors = vec![deficient.identity().unwrap()];
    let descendant = publish(&f.host, descendant);
    let history = f
        .publication
        .resolve(f.root.path(), &[f.enrolled.candidate.commit.clone(), later])
        .unwrap();
    assert!(history.incomplete.contains(&descendant.identity().unwrap()));
    assert!(!history.eligible(&descendant.identity().unwrap()).unwrap());
    // Object substitution remains an integrity error even for a quarantined record.
    let mut forged = deficient;
    forged.reasons.push(Reason::ObservationIncomplete);
    forged.candidate.tree = "f".repeat(40);
    forged.checkout = forged.candidate.clone();
    publish(&f.host, forged);
    assert!(f
        .publication
        .resolve(
            f.root.path(),
            std::slice::from_ref(&f.enrolled.candidate.commit)
        )
        .is_err());
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
    let bounded = history::resolve(
        root.path(),
        &fixture().publication.authority,
        &[],
        std::slice::from_ref(&at_limit),
    )
    .unwrap();
    let depth = crate::depth_ci::profile::Profile::Synthetic
        .record(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/depth-synthetic/record.json"
        )))
        .unwrap();
    let generation = Generation::from_record(&depth);
    assert!(!bounded
        .associates_generation(root.path(), &at_limit, &generation)
        .unwrap());
    assert!(bounded
        .associates_generation(root.path(), &tip, &generation)
        .unwrap_err()
        .contains("unique commit traversal bound"));
    assert!(
        history::resolve(root.path(), &fixture().publication.authority, &[], &[tip])
            .err()
            .unwrap()
            .contains("unique commit traversal bound")
    );
}
