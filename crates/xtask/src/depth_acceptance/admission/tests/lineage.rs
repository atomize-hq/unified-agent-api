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
