use super::*;
use crate::depth_record::part_identity;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

fn git(root: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().into()
}
fn write(root: &Path, path: &str, bytes: impl AsRef<[u8]>) -> FileRef {
    let full = root.join(path);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(&full, bytes.as_ref()).unwrap();
    FileRef {
        path: path.into(),
        sha256: hex::encode(Sha256::digest(bytes.as_ref())),
    }
}
fn save<T: serde::Serialize>(root: &Path, path: &str, value: &T) -> FileRef {
    write(root, path, serde_json::to_vec(value).unwrap())
}
fn record(scope: &mut Scope, evidence: &FileRef) -> DepthRecord {
    let target = &scope.obligation.target;
    let mut p = json!({
        "identity": "0".repeat(64), "selection": {"agent_id":"codex", "enrollment": {
            "lifecycle_path":"maintenance", "version":"1.2.3", "targets":[target],
            "modes":[{"selector":"run", "command_path":"codex exec", "mode":"headless", "predicates":[]}],
            "operations":[{"id":"run", "serves":["run"], "templates":["CORE"], "shared_mapping":"wrapper_only",
                "forwarding":false, "not_applicable_permitted":[],
                "promise":{"r1":"r", "v1":"v", "g1":"g", "e1":"e", "a1":"a", "x1":"x", "x2":"x", "x3":"x", "x4":"x"}}],
            "capabilities":[]}},
        "rules_revision":"1", "base":["R1"], "templates":[{"template":"CORE", "obligations":["LC2"]}],
        "path_enabled":true, "debt_baseline":[]
    });
    p["identity"] = json!(part_identity(&p).unwrap());
    let mut o = json!({"identity":"0".repeat(64), "policy_identity":p["identity"], "targets":[target],
        "acquired_inputs":[], "operations":[{"id":"run", "surfaces":[], "obligations":["R1"]}],
        "classification_required":[], "acceptance_work":[]});
    o["identity"] = json!(part_identity(&o).unwrap());
    scope.policy_identity = p["identity"].as_str().unwrap().into();
    scope.obligations_identity = o["identity"].as_str().unwrap().into();
    let mut e = json!({"identity":"0".repeat(64), "policy_identity":p["identity"], "obligations_identity":o["identity"],
        // The claimed label is deliberately verified; tests prove it confers nothing.
        "results":[{"obligation":scope.obligation, "result":"verified", "evidence":[evidence]}],
        "reuse_bindings":[], "debt_grants":[], "debt_retired":[]});
    e["identity"] = json!(part_identity(&e).unwrap());
    DepthRecord::parse(
        &json!({"schema_version":1, "event":scope.event, "policy":p, "obligations":o,
        "execution":e, "acceptance_entries":[]})
        .to_string(),
        "codex",
        "1.2.3",
    )
    .unwrap()
}
fn reseal(record: &mut DepthRecord) {
    record.policy.identity = part_identity(&record.policy).unwrap();
    let o = record.obligations.as_mut().unwrap();
    o.policy_identity = record.policy.identity.clone();
    o.identity = part_identity(o).unwrap();
    let e = record.execution.as_mut().unwrap();
    e.policy_identity = record.policy.identity.clone();
    e.obligations_identity = o.identity.clone();
    e.identity = part_identity(e).unwrap();
}
struct Fixture {
    root: tempfile::TempDir,
    receipt: EvidenceReceipt,
    graph: DependencyGraph,
    current: Evaluation,
    selected: FileRef,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let path = root.path();
        git(path, &["init", "-q"]);
        git(path, &["config", "user.name", "Fixture"]);
        git(path, &["config", "user.email", "fixture@example.invalid"]);
        let mut receipt = EvidenceReceipt::parse(include_str!(
            "../../../tests/fixtures/depth-evidence-v1.json"
        ))
        .unwrap();
        let mut inputs = vec![];
        for (file, bytes) in [
            ("Cargo.toml", "[workspace]"),
            ("Cargo.lock", "locked external packages"),
            ("crates/codex/Cargo.toml", "[package]"),
            ("crates/codex/src/lib.rs", "fn production() {}"),
            ("crates/codex/tests/request.rs", "assert_production();"),
            ("crates/codex/tests/fixtures/synthetic.sh", "fixture"),
            ("docs/requirements/request.md", "forward the request"),
        ] {
            inputs.push(write(path, file, bytes));
        }
        inputs.sort();
        // Binary fixture proves immutable reads do not reinterpret bytes as text.
        inputs.push(write(
            path,
            "crates/codex/tests/fixtures/raw.bin",
            [0xff, 0, 0xfe],
        ));
        inputs.sort();
        let graph = DependencyGraph {
            schema_version: 1,
            roots: vec!["codex@1.0.0".into()],
            packages: vec![
                Package {
                    id: "codex@1.0.0".into(),
                    origin: PackageOrigin::Local {
                        directory: "crates/codex".into(),
                    },
                    dependencies: vec!["registry+example@1.0.0".into()],
                },
                Package {
                    id: "registry+example@1.0.0".into(),
                    origin: PackageOrigin::Registry {
                        source: "registry+https://example.invalid/index".into(),
                        checksum: "a".repeat(64),
                    },
                    dependencies: vec![],
                },
            ],
            inputs,
            scopes: vec!["crates/codex".into(), "docs/requirements".into()],
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                path.join("crates/codex/tests/fixtures/synthetic.sh"),
                fs::Permissions::from_mode(0o755),
            )
            .unwrap();
        }
        git(path, &["add", "."]);
        git(path, &["commit", "-qm", "built inputs"]);
        receipt.build.source_commit = git(path, &["rev-parse", "HEAD"]);
        receipt.build.source_tree = git(path, &["rev-parse", "HEAD^{tree}"]);
        receipt.build.covered_inputs = graph.inputs.clone();
        receipt.build.absent_inputs = [
            ".cargo/config",
            ".cargo/config.toml",
            "rust-toolchain",
            "rust-toolchain.toml",
            "crates/codex/build.rs",
        ]
        .map(str::to_owned)
        .to_vec();
        receipt.build.toolchain = write(path, "reports/toolchain.txt", "rustc 1.78.0 exact build");
        receipt.build.dependency_graph = save(path, "reports/dependencies.json", &graph);
        let environment = Environment {
            schema_version: 1,
            target: receipt.build.host_target.clone(),
            features: receipt.build.features.clone(),
            variables: vec![Variable {
                name: "RUSTFLAGS".into(),
                value: String::new(),
            }],
        };
        receipt.build.environment = save(path, "reports/environment.json", &environment);
        receipt.cases.raw_output = write(path, "reports/results.txt", "observed case passed");
        receipt.producer.collector =
            write(path, "scripts/depth_collector.py", "approved collector");
        receipt.provenance[0].file = graph
            .inputs
            .iter()
            .find(|f| f.path.ends_with("synthetic.sh"))
            .unwrap()
            .clone();
        let assertion = &mut receipt.adequacy.mapping.assertions[0];
        assertion.assertion = graph
            .inputs
            .iter()
            .find(|f| f.path.ends_with("request.rs"))
            .unwrap()
            .clone();
        assertion.production_inputs = vec![graph
            .inputs
            .iter()
            .find(|f| f.path.ends_with("lib.rs"))
            .unwrap()
            .clone()];
        receipt.adequacy.endorsement.source_commit = receipt.build.source_commit.clone();
        receipt.adequacy.endorsement.mapping_sha256 = receipt.adequacy.mapping.sha256().unwrap();
        let placeholder = FileRef {
            path: "reports/receipt.json".into(),
            sha256: "0".repeat(64),
        };
        let mut rec = record(&mut receipt.scope, &placeholder);
        let selected = save(path, "reports/receipt.json", &receipt);
        rec.execution.as_mut().unwrap().results[0].evidence = vec![selected.clone()];
        reseal(&mut rec);
        let current = Evaluation {
            current: receipt.scope.clone(),
            definition_sha256: "d".repeat(64),
            record: rec,
            environment,
            toolchain: receipt.build.toolchain.clone(),
        };
        git(path, &["add", "."]);
        Self {
            root,
            receipt,
            graph,
            current,
            selected,
        }
    }
    fn update(&mut self) {
        self.receipt.adequacy.endorsement.mapping_sha256 =
            self.receipt.adequacy.mapping.sha256().unwrap();
        self.selected = save(self.root.path(), "reports/receipt.json", &self.receipt);
        self.current.record.execution.as_mut().unwrap().results[0].evidence =
            vec![self.selected.clone()];
        reseal(&mut self.current.record);
        git(self.root.path(), &["add", "."]);
    }
    fn run(&self) -> Run {
        Run {
            evidence: self.selected.clone(),
            scope: self.receipt.scope.clone(),
            definition_sha256: self.current.definition_sha256.clone(),
            producer: self.receipt.producer.clone(),
            build: self.receipt.build.clone(),
            graph: self.graph.clone(),
            environment: self.current.environment.clone(),
            provenance: self.receipt.provenance.clone(),
            cases: self.receipt.cases.clone(),
            adoption: Some(Adoption {
                endorsement: self.receipt.adequacy.endorsement.clone(),
                reviewer: self.receipt.adequacy.mapping.reviewer_identity.clone(),
                executor: self.receipt.adequacy.mapping.executor_identity.clone(),
            }),
        }
    }
    fn population(&self) -> Population {
        Population {
            current: self.current.current.clone(),
            definition_sha256: self.current.definition_sha256.clone(),
            environment: self.current.environment.clone(),
            toolchain: self.current.toolchain.clone(),
            runs: vec![self.run()],
        }
    }
    fn evaluate(&self, population: Population) -> Result<Verdict, String> {
        struct FixtureHost(std::cell::RefCell<Option<Population>>);
        impl Host for FixtureHost {
            fn population(&self, _: &Evaluation) -> Result<Option<Population>, String> {
                Ok(self.0.borrow_mut().take())
            }
        }
        evaluate(
            self.root.path(),
            &self.selected,
            &self.current,
            &FixtureHost(std::cell::RefCell::new(Some(population))),
        )
    }
    fn result(&self) -> DepthResult {
        self.evaluate(self.population()).unwrap().result
    }
}

#[test]
fn isolated_full_proof_verifies_but_public_claims_and_missing_population_never_do() {
    let f = Fixture::new();
    assert_eq!(f.result(), DepthResult::Verified);
    assert_eq!(
        evaluate_workspace(f.root.path(), &f.selected, &f.current)
            .unwrap()
            .result,
        DepthResult::Unverified
    );
    let mut population = f.population();
    population.runs.clear();
    assert_eq!(
        f.evaluate(population).unwrap().result,
        DepthResult::Unverified
    );
}

#[test]
fn exact_cases_and_outcomes_drive_verdicts_not_the_e_label() {
    for (outcome, expected) in [
        (CaseOutcome::Failed, DepthResult::Failed),
        (CaseOutcome::Skipped, DepthResult::Unverified),
        (CaseOutcome::Ignored, DepthResult::Unverified),
        (CaseOutcome::Aborted, DepthResult::Unverified),
    ] {
        let mut f = Fixture::new();
        f.receipt.cases.executed[0].outcome = outcome;
        f.update();
        assert_eq!(f.result(), expected);
    }
    for mode in ["zero", "missing", "filtered", "extra"] {
        let mut f = Fixture::new();
        match mode {
            "zero" => {
                f.receipt.cases.requested.clear();
                f.receipt.cases.executed.clear();
            }
            "missing" => f.receipt.cases.executed.clear(),
            "filtered" => f.receipt.cases.executed[0].case.name = "different".into(),
            _ => {
                let mut extra = f.receipt.cases.executed[0].clone();
                extra.case.name = "extra".into();
                f.receipt.cases.executed.push(extra);
            }
        }
        f.update();
        assert_eq!(f.result(), DepthResult::Unverified, "{mode}");
    }
}

#[test]
fn nested_drift_new_dependencies_and_absent_config_invalidate_but_unrelated_edits_do_not() {
    for path in [
        "crates/codex/src/lib.rs",
        "crates/codex/tests/request.rs",
        "crates/codex/tests/fixtures/raw.bin",
        "docs/requirements/request.md",
        "Cargo.lock",
        "crates/codex/src/new.rs",
        ".cargo/config.toml",
    ] {
        let f = Fixture::new();
        write(f.root.path(), path, "changed");
        git(f.root.path(), &["add", "."]);
        assert_eq!(f.result(), DepthResult::Unverified, "{path}");
    }
    let f = Fixture::new();
    write(f.root.path(), "docs/unrelated.md", "unrelated");
    git(f.root.path(), &["add", "."]);
    assert_eq!(f.result(), DepthResult::Verified);
}

#[test]
fn incomplete_and_forged_graphs_do_not_verify() {
    for mode in ["empty", "external", "dependency", "scope", "nested"] {
        let mut f = Fixture::new();
        match mode {
            "empty" => f.graph.roots.clear(),
            "external" => f.graph.packages.pop().map(|_| ()).unwrap(),
            "dependency" => f.graph.packages[0].dependencies.push("missing".into()),
            "scope" => f.graph.scopes.clear(),
            _ => {
                f.graph.inputs.retain(|p| !p.path.ends_with("raw.bin"));
            }
        }
        f.receipt.build.dependency_graph =
            save(f.root.path(), "reports/dependencies.json", &f.graph);
        f.receipt.build.covered_inputs = f.graph.inputs.clone();
        f.update();
        assert_eq!(f.result(), DepthResult::Unverified, "{mode}");
    }
    let mut f = Fixture::new();
    let original = f.population();
    f.graph.packages[1].origin = PackageOrigin::Git {
        source: "another".into(),
        commit: "c".repeat(40),
    };
    f.receipt.build.dependency_graph = save(f.root.path(), "reports/dependencies.json", &f.graph);
    f.update();
    let mut original = original;
    original.runs[0].evidence = f.selected.clone();
    assert_eq!(
        f.evaluate(original).unwrap().result,
        DepthResult::Unverified
    );
}

#[test]
fn adoption_needs_exact_mapping_source_and_distinct_invocations() {
    for mode in [
        "none",
        "digest",
        "reviewer",
        "source",
        "same",
        "empty-production",
        "uncovered",
    ] {
        let mut f = Fixture::new();
        if mode == "same" {
            f.receipt.adequacy.mapping.reviewer_identity =
                f.receipt.adequacy.mapping.executor_identity.clone();
            f.update();
        }
        if mode == "empty-production" {
            f.receipt.adequacy.mapping.assertions[0]
                .production_inputs
                .clear();
            f.update();
        }
        if mode == "uncovered" {
            f.receipt.adequacy.mapping.assertions[0].assertion.path = "docs/other.md".into();
            f.update();
        }
        let mut p = f.population();
        let adoption = p.runs[0].adoption.as_mut().unwrap();
        match mode {
            "none" => p.runs[0].adoption = None,
            "digest" => adoption.endorsement.mapping_sha256 = "f".repeat(64),
            "reviewer" => adoption.reviewer = "forged".into(),
            "source" => adoption.endorsement.source_commit = "c".repeat(40),
            _ => {}
        }
        assert_eq!(
            f.evaluate(p).unwrap().result,
            DepthResult::Unverified,
            "{mode}"
        );
    }
    let mut f = Fixture::new();
    let mut original = f.population();
    f.receipt.adequacy.mapping.assertions[0].counterfactual = "new unadopted assertion".into();
    f.update();
    original.runs[0].evidence = f.selected.clone();
    assert_eq!(
        f.evaluate(original).unwrap().result,
        DepthResult::Unverified
    );
}

#[test]
fn incomplete_optional_input_provenance_is_not_an_empty_set() {
    let mut f = Fixture::new();
    f.receipt.build.absent_inputs.clear();
    f.update();
    assert_eq!(f.result(), DepthResult::Unverified);
}

#[test]
fn reuse_requires_explicit_original_generation_and_unchanged_definition_and_rules() {
    let mut f = Fixture::new();
    if let Event::Maintenance { request_commit, .. } = &mut f.current.current.event {
        *request_commit = "new-event".into();
    }
    f.current.record.event = f.current.current.event.clone();
    assert_eq!(f.result(), DepthResult::Unverified);
    f.current
        .record
        .execution
        .as_mut()
        .unwrap()
        .reuse_bindings
        .push(crate::depth_record::ReuseBinding {
            obligation: f.receipt.scope.obligation.clone(),
            policy_identity: f.receipt.scope.policy_identity.clone(),
            obligations_identity: f.receipt.scope.obligations_identity.clone(),
        });
    reseal(&mut f.current.record);
    assert_eq!(f.result(), DepthResult::Verified);
    let mut p = f.population();
    p.runs[0].definition_sha256 = "b".repeat(64);
    assert_eq!(f.evaluate(p).unwrap().result, DepthResult::Unverified);
    let mut p = f.population();
    p.runs[0].scope.trust_revision = "different".into();
    assert_eq!(f.evaluate(p).unwrap().result, DepthResult::Unverified);
    f.current.current.enrollment_origin = EnrollmentOrigin::Production;
    assert_eq!(f.result(), DepthResult::Unverified);
}

#[test]
fn scope_original_binding_and_build_claims_are_not_self_authenticating() {
    for mode in [
        "agent",
        "version",
        "target",
        "po",
        "event",
        "review",
        "origin",
        "toolchain",
        "features",
        "tree",
        "environment",
        "cases",
        "raw-output",
    ] {
        let mut f = Fixture::new();
        let mut p = f.population();
        match mode {
            "agent" => p.runs[0].scope.agent_id = "another".into(),
            "version" => p.runs[0].scope.version = "1.2.4".into(),
            "target" => p.runs[0].scope.obligation.target = "another-target".into(),
            "po" => p.runs[0].scope.policy_identity = "b".repeat(64),
            "event" => {
                p.runs[0].scope.event = Event::Onboarding {
                    approval_commit: "other".into(),
                    approval_recorded_at: "now".into(),
                }
            }
            "review" => p.runs[0].scope.review_revision = "other".into(),
            "origin" => p.runs[0].scope.enrollment_origin = EnrollmentOrigin::Production,
            "toolchain" => p.runs[0].build.toolchain.sha256 = "b".repeat(64),
            "features" => p.runs[0].build.features.clear(),
            "tree" => {
                f.receipt.build.source_tree = "b".repeat(40);
                f.update();
                p = f.population();
            }
            "environment" => {
                p.environment.variables[0].value = "different".into();
            }
            "cases" => p.runs[0].cases.executed.clear(),
            _ => p.runs[0].cases.raw_output.sha256 = "b".repeat(64),
        }
        assert_eq!(
            f.evaluate(p).unwrap().result,
            DepthResult::Unverified,
            "{mode}"
        );
    }
}

#[test]
fn complete_population_prevents_selecting_only_the_green_attempt() {
    let mut f = Fixture::new();
    let green = f.run();
    f.receipt.producer.run_attempt += 1;
    f.receipt.cases.executed[0].outcome = CaseOutcome::Failed;
    f.update();
    let mut red = f.run();
    red.evidence = save(f.root.path(), "reports/red.json", &f.receipt);
    // Restore original green selected receipt, while host population still includes red.
    f.receipt.producer.run_attempt -= 1;
    f.receipt.cases.executed[0].outcome = CaseOutcome::Passed;
    f.update();
    git(f.root.path(), &["add", "."]);
    let mut p = f.population();
    p.runs = vec![green, red];
    assert_eq!(f.evaluate(p).unwrap().result, DepthResult::Unverified);
    let mut p = f.population();
    p.runs[0].cases.executed.clear();
    assert_eq!(f.evaluate(p).unwrap().result, DepthResult::Unverified);
    let mut p = f.population();
    p.runs.push(f.run());
    assert_eq!(f.evaluate(p).unwrap().result, DepthResult::Unverified);
}

#[test]
fn immutable_readers_ignore_blob_commit_and_tree_replacements() {
    let f = Fixture::new();
    let path = f.root.path();
    let original = f.receipt.build.source_commit.clone();
    write(
        path,
        "crates/codex/src/lib.rs",
        "replacement implementation",
    );
    git(path, &["add", "."]);
    git(path, &["commit", "-qm", "replacement snapshot"]);
    let replacement = git(path, &["rev-parse", "HEAD"]);
    let original_blob = git(
        path,
        &["rev-parse", &format!("{original}:crates/codex/src/lib.rs")],
    );
    let new_blob = git(path, &["rev-parse", "HEAD:crates/codex/src/lib.rs"]);
    let original_tree = f.receipt.build.source_tree.clone();
    let new_tree = git(path, &["rev-parse", "HEAD^{tree}"]);
    // Restore matching workspace/index before applying replacements to immutable source objects.
    write(path, "crates/codex/src/lib.rs", "fn production() {}");
    git(path, &["add", "."]);
    for (old, new) in [
        (&original_blob, &new_blob),
        (&original, &replacement),
        (&original_tree, &new_tree),
    ] {
        git(path, &["replace", old, new]);
        assert_eq!(f.result(), DepthResult::Verified);
        git(path, &["replace", "-d", old]);
    }
}

#[test]
fn payload_form_errors_differ_from_missing_or_unverified_evidence() {
    let mut f = Fixture::new();
    f.receipt.build.dependency_graph = write(f.root.path(), "reports/dependencies.json", "{}");
    f.update();
    assert!(f.evaluate(f.population()).is_err());
    let f = Fixture::new();
    fs::remove_file(f.root.path().join("reports/dependencies.json")).unwrap();
    git(f.root.path(), &["add", "."]);
    assert_eq!(f.result(), DepthResult::Unverified);
}

#[test]
fn a_new_po_can_reuse_only_exact_scope_and_current_inputs() {
    let mut f = Fixture::new();
    // Unrelated due work changes O's identity, not this obligation's definition.
    let o = f.current.record.obligations.as_mut().unwrap();
    o.acquired_inputs.push(FileRef {
        path: "reports/additional-input.json".into(),
        sha256: "c".repeat(64),
    });
    reseal(&mut f.current.record);
    f.current.current.obligations_identity = f
        .current
        .record
        .obligations
        .as_ref()
        .unwrap()
        .identity
        .clone();
    assert_eq!(f.result(), DepthResult::Unverified);
    f.current
        .record
        .execution
        .as_mut()
        .unwrap()
        .reuse_bindings
        .push(crate::depth_record::ReuseBinding {
            obligation: f.receipt.scope.obligation.clone(),
            policy_identity: f.receipt.scope.policy_identity.clone(),
            obligations_identity: f.receipt.scope.obligations_identity.clone(),
        });
    reseal(&mut f.current.record);
    assert_eq!(f.result(), DepthResult::Verified);
    for mode in ["version", "target", "agent", "review", "trust"] {
        let mut receipt = f.receipt.clone();
        match mode {
            "version" => f.receipt.scope.version = "1.2.4".into(),
            "target" => f.receipt.scope.obligation.target = "aarch64-apple-darwin".into(),
            "agent" => f.receipt.scope.agent_id = "other".into(),
            "review" => f.receipt.scope.review_revision = "2".into(),
            _ => f.receipt.scope.trust_revision = "2".into(),
        }
        f.update();
        assert_eq!(f.result(), DepthResult::Unverified, "{mode}");
        std::mem::swap(&mut receipt, &mut f.receipt);
        f.update();
    }
}

mod regressions;
