use super::*;
use crate::depth_evidence::payload::{DependencyGraph, Environment, Variable};
use crate::depth_evidence::{CaseIdentity, CaseOutcome, CaseResult, ProvenanceKind};
use std::{collections::BTreeMap, fs, process::Command};

#[derive(Clone, Default)]
struct Fake(BTreeMap<String, Vec<u8>>);
impl Api for Fake {
    fn bytes(&self, path: &str, limit: usize) -> Result<Vec<u8>, String> {
        let bytes = self
            .0
            .get(path)
            .cloned()
            .ok_or_else(|| format!("missing fake endpoint: {path}"))?;
        if bytes.len() > limit {
            return Err("bounded fake response exceeded".into());
        }
        Ok(bytes)
    }
}
impl Fake {
    fn put(&mut self, path: &str, value: Value) {
        self.0
            .insert(endpoint(path), serde_json::to_vec(&value).unwrap());
    }
    fn page(&mut self, path: &str, member: Option<&str>, rows: Vec<Value>) {
        let value = if let Some(member) = member {
            json!({member:rows,"total_count":rows.len()})
        } else {
            json!(rows)
        };
        let join = if path.contains('?') { '&' } else { '?' };
        self.put(&format!("{path}{join}per_page=100&page=1"), value);
    }
    fn change(&mut self, path: &str, edit: impl FnOnce(&mut Value)) {
        let key = endpoint(path);
        let mut value: Value = serde_json::from_slice(&self.0[&key]).unwrap();
        edit(&mut value);
        self.put(path, value);
    }
}
fn git(root: &Path, args: &[&str]) -> String {
    String::from_utf8(
        depth_ci::output(
            Command::new("git")
                .arg("--no-replace-objects")
                .arg("-C")
                .arg(root)
                .args(args),
        )
        .unwrap(),
    )
    .unwrap()
    .trim()
    .into()
}
fn write(root: &Path, path: &str, bytes: impl AsRef<[u8]>) -> FileRef {
    let file = root.join(path);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, bytes.as_ref()).unwrap();
    depth_ci::reference(path, bytes.as_ref())
}
struct Fixture {
    root: tempfile::TempDir,
    api: Fake,
    w: Witness,
    commit: String,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let path = root.path();
        git(path, &["init", "-q"]);
        git(path, &["config", "user.name", "Fixture"]);
        git(path, &["config", "user.email", "fixture@example.invalid"]);
        let record = include_str!("../../../../tests/fixtures/depth-ci/record.json");
        let mut inputs = vec![];
        for (file, raw) in [
            (depth_ci::FIXTURE, record),
            (
                depth_ci::CONTRACT,
                include_str!("../../../../tests/fixtures/depth-ci/contract.md"),
            ),
            (
                "crates/xtask/tests/fixtures/depth-ci/build-profile.json",
                include_str!("../../../../tests/fixtures/depth-ci/build-profile.json"),
            ),
            ("Cargo.toml", "[workspace]"),
            ("Cargo.lock", "locked"),
            ("crates/wrapper_events/Cargo.toml", "[package]"),
            (depth_ci::PRODUCTION, "fn validate() {}"),
            (depth_ci::ASSERTION, "fn test_grammar() {}"),
        ] {
            inputs.push(write(path, file, raw));
        }
        write(path, profile::WORKFLOW, "approved exact workflow");
        write(path, depth_ci::COLLECTOR, "approved collector closure");
        write(
            path,
            depth_ci::ARCHIVE,
            include_str!("../../../../../../scripts/depth_ci_archive.py"),
        );
        git(path, &["add", "."]);
        git(path, &["commit", "-qm", "fixture"]);
        let commit = git(path, &["rev-parse", "HEAD"]);
        inputs.sort_by(|a, b| a.path.cmp(&b.path));
        let profile: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/depth-ci/build-profile.json"
        ))
        .unwrap();
        let graph = DependencyGraph {
            schema_version: 1,
            roots: vec![
                "workspace:crates/wrapper_events#unified-agent-api-wrapper-events@0.3.7".into(),
            ],
            packages: serde_json::from_value(profile["packages"].clone()).unwrap(),
            inputs: inputs.clone(),
            scopes: vec![
                "crates/wrapper_events".into(),
                "crates/xtask/tests/fixtures/depth-ci".into(),
            ],
        };
        let environment = Environment {
            schema_version: 1,
            target: depth_ci::TARGET.into(),
            features: vec![],
            variables: vec![Variable {
                name: "policy".into(),
                value: "fixed".into(),
            }],
        };
        let toolchain = "pinned actual toolchain".to_string();
        let raw = "eight case observations".to_string();
        let reference = |name: &str, b: &[u8]| {
            depth_ci::reference(
                format!("cli_manifests/codex/reports/0.0.0/evidence/channel-v1/1-1/{name}"),
                b,
            )
        };
        let cases = Cases {
            requested: depth_ci::CASES
                .iter()
                .map(|name| CaseIdentity {
                    name: (*name).into(),
                    executable_sha256: "a".repeat(64),
                })
                .collect(),
            executed: depth_ci::CASES
                .iter()
                .map(|name| CaseResult {
                    case: CaseIdentity {
                        name: (*name).into(),
                        executable_sha256: "a".repeat(64),
                    },
                    outcome: CaseOutcome::Passed,
                })
                .collect(),
            raw_output: reference("raw.txt", raw.as_bytes()),
        };
        let w = Witness {
            schema_version: 1,
            run_id: 1,
            run_attempt: 1,
            workflow_commit: commit.clone(),
            build: Build {
                source_commit: commit.clone(),
                source_tree: git(path, &["rev-parse", "HEAD^{tree}"]),
                host_target: depth_ci::TARGET.into(),
                toolchain: reference("toolchain.json", toolchain.as_bytes()),
                features: vec![],
                environment: reference(
                    "environment.json",
                    &serde_json::to_vec(&environment).unwrap(),
                ),
                dependency_graph: reference("graph.json", &serde_json::to_vec(&graph).unwrap()),
                covered_inputs: inputs,
                absent_inputs: [
                    ".cargo/config",
                    ".cargo/config.toml",
                    "rust-toolchain",
                    "rust-toolchain.toml",
                    "crates/wrapper_events/build.rs",
                ]
                .map(str::to_owned)
                .to_vec(),
            },
            graph,
            environment,
            provenance: vec![Provenance {
                kind: ProvenanceKind::Source,
                origin: "isolated source contract".into(),
                file: depth_ci::reference(
                    depth_ci::CONTRACT,
                    include_str!("../../../../tests/fixtures/depth-ci/contract.md").as_bytes(),
                ),
            }],
            cases,
            toolchain_report: toolchain,
            raw_output: raw,
        };
        let mut fixture = Self {
            root,
            api: Fake::default(),
            w,
            commit,
        };
        fixture.install(1);
        fixture
    }
    fn install(&mut self, attempts: u64) {
        self.api = Fake::default();
        let commit = &self.commit;
        self.api
            .put("git/ref/heads/staging", json!({"object":{"sha":commit}}));
        let parsed = depth_ci::record(include_str!(
            "../../../../tests/fixtures/depth-ci/record.json"
        ))
        .unwrap();
        let Event::Maintenance { request_commit, .. } = parsed.event else {
            panic!()
        };
        self.api.page(&format!("commits/{request_commit}/pulls"),None,vec![json!({"id":9,"number":9,"base":{"ref":"staging","repo":{"full_name":depth_ci::REPOSITORY}}})]);
        let run = json!({"id":1,"head_sha":commit,"head_repository":{"full_name":depth_ci::REPOSITORY},"path":profile::WORKFLOW,"event":"workflow_dispatch","run_attempt":attempts,"status":"completed"});
        self.api.page(
            "actions/workflows/ci.yml/runs?created=%3E%3D2026-10-23",
            Some("workflow_runs"),
            vec![run.clone()],
        );
        let mut artifacts = vec![];
        for attempt in 1..=attempts {
            let mut w = self.w.clone();
            w.run_attempt = attempt;
            let artifact_id = 100 + attempt;
            let job_id = 200 + attempt;
            let name = format!("depth-channel-v1-1-{attempt}");
            let file = self.root.path().join("witness.json");
            fs::write(&file, serde_json::to_vec(&w).unwrap()).unwrap();
            let zip=depth_ci::output(Command::new("python3").args(["-c","import sys,zipfile,io; b=io.BytesIO(); z=zipfile.ZipFile(b,'w'); z.write(sys.argv[1],'witness.json'); z.close(); sys.stdout.buffer.write(b.getvalue())"]).arg(&file)).unwrap();
            fs::remove_file(file).unwrap();
            let digest = depth_ci::reference("", &zip).sha256;
            artifacts.push(json!({"id":artifact_id,"name":name,"expired":false,"size_in_bytes":zip.len(),"digest":format!("sha256:{digest}"),"workflow_run":{"id":1,"head_sha":commit}}));
            self.api.0.insert(
                endpoint(&format!("actions/artifacts/{artifact_id}/zip")),
                zip,
            );
            let mut attempt_run = run.clone();
            attempt_run["run_attempt"] = json!(attempt);
            self.api
                .put(&format!("actions/runs/1/attempts/{attempt}"), attempt_run);
            self.api.page(&format!("actions/runs/1/attempts/{attempt}/jobs"),Some("jobs"),vec![json!({"id":job_id,"name":"Rust tests (workspace)","run_id":1,"run_attempt":attempt,"status":"completed","steps":(["Collect isolated depth witness","Upload isolated depth witness","Bind isolated depth artifact"].map(|s|json!({"name":s,"status":"completed","conclusion":"success","started_at":"2026-10-23T15:00:00Z","completed_at":"2026-10-23T15:00:01Z"})))})]);
            self.api.0.insert(
                endpoint(&format!("actions/jobs/{job_id}/logs")),
                format!(
                    "2026-10-23T15:00:00.123456Z DEPTH_ARTIFACT_BINDING={}\n",
                    json!({"id":artifact_id,"digest":digest,"name":name})
                )
                .into_bytes(),
            );
        }
        self.api
            .page("actions/runs/1/artifacts", Some("artifacts"), artifacts);
        let proposal = adoption::proposal(&self.w, "independent-review-invocation").unwrap();
        self.api.page("issues/9/comments",None,vec![json!({"id":33,"user":{"login":"maintainer"},"body":format!("```depth-adequacy-v1\n{}\n```",proposal)})]);
        self.api.page("pulls/9/reviews", None, vec![]);
        self.api.put(
            "collaborators/maintainer/permission",
            json!({"permission":"write"}),
        );
    }
}
#[test]
fn authenticated_artifact_reaches_real_evaluator_and_publisher_without_qualifying_whole_promise() {
    let f = Fixture::new();
    let proof: Value = serde_json::from_str(&prove_with(f.root.path(), &f.api).unwrap()).unwrap();
    assert_eq!(proof["result"], "verified", "{proof}");
    let promise = &proof["publisher"]["promises"][0];
    assert_eq!(promise["qualified"], false);
    assert!(promise["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["id"] == "V1" && r["result"] == "verified"));
    assert!(promise["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["id"] != "V1" && r["result"] == "unverified"));
    assert_eq!(proof["production_enrollment"], false);
    assert_eq!(git(f.root.path(), &["status", "--porcelain"]), "");
}
#[test]
fn missing_attempt_or_wrong_job_or_artifact_digest_refuses() {
    let mut f = Fixture::new();
    f.install(2);
    assert_eq!(resolve(&f.api, f.root.path()).unwrap().runs.len(), 2);
    for mutate in 0..5 {
        let mut api = f.api.clone();
        match mutate {
            0 => {
                api.0.remove(&endpoint("actions/runs/1/attempts/2"));
            }
            1 => api.change("actions/runs/1/attempts/2/jobs?per_page=100&page=1", |v| {
                v["jobs"][0]["run_attempt"] = json!(1)
            }),
            2 => api.change("actions/runs/1/artifacts?per_page=100&page=1", |v| {
                v["artifacts"][1]["digest"] = json!(format!("sha256:{}", "0".repeat(64)))
            }),
            3 => {
                api.0.insert(
                    endpoint("actions/jobs/202/logs"),
                    b"2026-10-23T14:00:00Z DEPTH_ARTIFACT_BINDING={}\n".to_vec(),
                );
            }
            _ => api.change("actions/runs/1/attempts/2/jobs?per_page=100&page=1", |v| {
                v["jobs"][0]["steps"][1]["conclusion"] = json!("failure")
            }),
        }
        assert!(
            resolve(&api, f.root.path()).is_err(),
            "mutation {mutate} accepted"
        );
    }
}
#[test]
fn endorsement_permission_digest_and_source_are_authenticated() {
    let f = Fixture::new();
    let mut api = f.api.clone();
    api.put(
        "collaborators/maintainer/permission",
        json!({"permission":"read"}),
    );
    assert!(resolve(&api, f.root.path())
        .unwrap_err()
        .contains("missing exact authenticated"));
    let mut api = f.api.clone();
    api.page(
        "issues/9/comments",
        None,
        vec![json!({"id":33,"user":{"login":"maintainer"},"body":"Approved. All tests pass."})],
    );
    assert!(resolve(&api, f.root.path())
        .unwrap_err()
        .contains("missing exact authenticated"));
    let mut api = f.api.clone();
    api.change("issues/9/comments?per_page=100&page=1", |v| {
        v[0]["body"] = json!(v[0]["body"]
            .as_str()
            .unwrap()
            .replace("independent-review-invocation", profile::EXECUTOR))
    });
    assert!(resolve(&api, f.root.path()).is_err());
}
#[test]
fn changed_producer_inventory_or_contents_cannot_approve_itself() {
    let f = Fixture::new();
    write(
        f.root.path(),
        depth_ci::COLLECTOR,
        "candidate replaced producer",
    );
    git(f.root.path(), &["add", "."]);
    assert!(resolve(&f.api, f.root.path())
        .unwrap_err()
        .contains("reviewed staging"));
    write(
        f.root.path(),
        "crates/xtask/src/new_helper.rs",
        "fn tamper() {}",
    );
    git(f.root.path(), &["add", "."]);
    assert!(resolve(&f.api, f.root.path()).is_err());
}
#[test]
fn pagination_detects_truncation_duplicates_and_saturation() {
    let mut api = Fake::default();
    api.put(
        "x?per_page=100&page=1",
        json!({"total_count":2,"rows":[{"id":1}]}),
    );
    assert!(list(&api, "x", Some("rows")).is_err());
    api.put(
        "x?per_page=100&page=1",
        json!({"total_count":2,"rows":[{"id":1},{"id":1}]}),
    );
    assert!(list(&api, "x", Some("rows")).is_err());
    api.put(
        "x?per_page=100&page=1",
        json!({"total_count":1000,"rows":[]}),
    );
    assert!(list(&api, "x", Some("rows")).is_err());
    api.put(
        "x?per_page=100&page=1",
        json!({"total_count":1,"rows":[{"id":1}]}),
    );
    assert_eq!(list(&api, "x", Some("rows")).unwrap().len(), 1);
}

impl Fixture {
    fn replace_attempt(&mut self, attempt: u64, edit: impl FnOnce(&mut Witness)) {
        let mut w = self.w.clone();
        w.run_attempt = attempt;
        edit(&mut w);
        let file = self.root.path().join("witness.json");
        fs::write(&file, serde_json::to_vec(&w).unwrap()).unwrap();
        let zip=depth_ci::output(Command::new("python3").args(["-c","import sys,zipfile,io; b=io.BytesIO(); z=zipfile.ZipFile(b,'w'); z.write(sys.argv[1],'witness.json'); z.close(); sys.stdout.buffer.write(b.getvalue())"]).arg(&file)).unwrap();
        fs::remove_file(file).unwrap();
        let artifact_id = 100 + attempt;
        let digest = depth_ci::reference("", &zip).sha256;
        self.api
            .change("actions/runs/1/artifacts?per_page=100&page=1", |v| {
                let a = v["artifacts"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|a| a["id"] == artifact_id)
                    .unwrap();
                a["digest"] = json!(format!("sha256:{digest}"));
                a["size_in_bytes"] = json!(zip.len());
            });
        self.api.0.insert(
            endpoint(&format!("actions/artifacts/{artifact_id}/zip")),
            zip,
        );
        self.api.0.insert(endpoint(&format!("actions/jobs/{}/logs",200+attempt)),format!("2026-10-23T15:00:00.123456Z DEPTH_ARTIFACT_BINDING={}\n",json!({"id":artifact_id,"digest":digest,"name":format!("depth-channel-v1-1-{attempt}")})).into_bytes());
    }
}
#[test]
fn all_attempts_drive_verdict_even_when_e_selected_receipt_passes() {
    let mut f = Fixture::new();
    f.install(2);
    f.replace_attempt(2, |w| w.cases.executed[0].outcome = CaseOutcome::Failed);
    let proof: Value = serde_json::from_str(&prove_with(f.root.path(), &f.api).unwrap()).unwrap();
    assert_eq!(proof["result"], "unverified", "{proof}");
    assert!(proof["reason"].as_str().unwrap().contains("contradictory"));
    f.replace_attempt(1, |w| w.cases.executed[0].outcome = CaseOutcome::Failed);
    let proof: Value = serde_json::from_str(&prove_with(f.root.path(), &f.api).unwrap()).unwrap();
    assert_eq!(proof["result"], "failed", "{proof}");
}
#[test]
fn tampered_payload_or_workflow_or_case_population_is_rejected() {
    for mutation in 0..4 {
        let mut f = Fixture::new();
        f.replace_attempt(1, |w| match mutation {
            0 => w.toolchain_report.push_str("tamper"),
            1 => w.workflow_commit = "0".repeat(40),
            2 => {
                w.cases.requested.pop();
            }
            _ => w.build.source_tree = "0".repeat(40),
        });
        assert!(
            resolve(&f.api, f.root.path()).is_err(),
            "mutation {mutation}"
        );
    }
}

#[test]
fn unapproved_head_decoder_and_workflow_never_execute_or_replay_approved_merge() {
    let mut f = Fixture::new();
    let marker = f.root.path().join("unapproved-executed");
    write(
        f.root.path(),
        depth_ci::ARCHIVE,
        format!(
            "from pathlib import Path\nPath({:?}).write_text('executed')\n",
            marker.to_str().unwrap()
        ),
    );
    write(
        f.root.path(),
        profile::WORKFLOW,
        "unapproved replay workflow",
    );
    git(f.root.path(), &["add", "."]);
    git(f.root.path(), &["commit", "-qm", "unapproved head"]);
    let unapproved = git(f.root.path(), &["rev-parse", "HEAD"]);
    // The local evaluator itself is the approved anchor, not the unapproved discovered head.
    git(f.root.path(), &["checkout", "-q", "--detach", &f.commit]);
    // Leave witness/artifact bytes naming the approved source. Only authenticated execution
    // metadata names the unapproved head, reproducing the synthetic-source substitution boundary.
    f.api.change(
        "actions/workflows/ci.yml/runs?created=%3E%3D2026-10-23&per_page=100&page=1",
        |v| v["workflow_runs"][0]["head_sha"] = json!(unapproved),
    );
    f.api.change("actions/runs/1/attempts/1", |v| {
        v["head_sha"] = json!(unapproved)
    });
    f.api
        .change("actions/runs/1/artifacts?per_page=100&page=1", |v| {
            v["artifacts"][0]["workflow_run"]["head_sha"] = json!(unapproved)
        });
    assert!(resolve(&f.api, f.root.path())
        .unwrap_err()
        .contains("not approved"));
    assert!(!marker.exists(), "unapproved head helper was executed");
    // Safe decoding also has no dependency on the discovered head helper at all.
    let helper = source(f.root.path(), &f.commit)
        .unwrap()
        .read(depth_ci::ARCHIVE)
        .unwrap();
    let run = f.api.json(&endpoint("actions/runs/1/attempts/1")).unwrap();
    assert!(artifact::witness(&f.api, &helper, &run, 1).is_ok());
    assert!(!marker.exists());
}
#[test]
fn root_cargo_and_toolchain_configuration_is_inside_producer_approval() {
    for path in profile::OPTIONAL {
        let f = Fixture::new();
        write(f.root.path(), path, "candidate compilation configuration");
        git(f.root.path(), &["add", "."]);
        git(f.root.path(), &["commit", "-qm", "config"]);
        let candidate = git(f.root.path(), &["rev-parse", "HEAD"]);
        assert!(profile::approved(f.root.path(), &candidate, &candidate, &f.commit).is_err());
        assert!(resolve(&f.api, f.root.path()).is_err());
    }
}
