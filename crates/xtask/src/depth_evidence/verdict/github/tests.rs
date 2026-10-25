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
    profile: Profile,
    root: tempfile::TempDir,
    api: Fake,
    w: Witness,
    commit: String,
}
impl Fixture {
    fn new() -> Self {
        Self::new_for(Profile::Channel)
    }
    fn new_for(p: Profile) -> Self {
        let root = tempfile::tempdir().unwrap();
        let path = root.path();
        git(path, &["init", "-q"]);
        git(path, &["config", "user.name", "Fixture"]);
        git(path, &["config", "user.email", "fixture@example.invalid"]);
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let record_raw = fs::read_to_string(repo.join(p.fixture())).unwrap();
        let record = record_raw.as_str();
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
        if p == Profile::Core {
            inputs.retain(|f| !f.path.starts_with("crates/xtask/tests/fixtures/depth-ci/"));
            fs::remove_dir_all(path.join("crates/xtask/tests/fixtures/depth-ci")).unwrap();
            let original = p.record(record).unwrap();
            let mut files: std::collections::BTreeSet<String> = original
                .obligations
                .unwrap()
                .acquired_inputs
                .into_iter()
                .map(|f| f.path)
                .collect();
            for name in [
                "record.json",
                "cases.json",
                "build-profile.json",
                "contract.md",
            ] {
                files.insert(format!("{}/{name}", p.directory()));
            }
            let specs: std::collections::BTreeMap<String, Vec<crate::depth_ci::profile::CaseSpec>> =
                serde_json::from_str(
                    &fs::read_to_string(repo.join(p.directory()).join("cases.json")).unwrap(),
                )
                .unwrap();
            for spec in specs.values().flatten() {
                files.insert(spec.assertion.clone());
                files.extend(spec.production_inputs.clone());
            }
            for path_name in files {
                inputs.push(write(
                    path,
                    &path_name,
                    fs::read(repo.join(&path_name)).unwrap(),
                ));
            }
        }
        write(path, profile::WORKFLOW, "approved exact workflow");
        write(path, depth_ci::COLLECTOR, "approved collector closure");
        write(
            path,
            depth_ci::ARCHIVE,
            include_str!("../../../../../../scripts/depth_ci_archive.py"),
        );
        inputs.sort_by(|a, b| a.path.cmp(&b.path));
        let profile: Value =
            serde_json::from_slice(&fs::read(repo.join(p.build_profile())).unwrap()).unwrap();
        // Fixture closure need not build; production collector corroborates real Cargo closure.
        let local_dirs: Vec<_> = profile["packages"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|p| p["origin"]["directory"].as_str())
            .collect();
        for d in &local_dirs {
            let name = format!("{d}/Cargo.toml");
            if !inputs.iter().any(|f| f.path == name) {
                inputs.push(write(path, &name, "[package]"));
            }
        }
        inputs.sort();
        git(path, &["add", "."]);
        git(path, &["commit", "-qm", "fixture"]);
        let commit = git(path, &["rev-parse", "HEAD"]);
        let graph = DependencyGraph {
            schema_version: 1,
            roots: vec![format!(
                "workspace:crates/{}#{}@0.3.7",
                if p == Profile::Channel {
                    "wrapper_events"
                } else {
                    "agent_api"
                },
                p.package()
            )],
            packages: serde_json::from_value(profile["packages"].clone()).unwrap(),
            inputs: inputs.clone(),
            scopes: std::iter::once(p.directory().to_owned())
                .chain(local_dirs.iter().map(|d| (*d).to_owned()))
                .collect(),
        };
        let environment = Environment {
            schema_version: 1,
            target: depth_ci::TARGET.into(),
            features: p.features(),
            variables: vec![Variable {
                name: "policy".into(),
                value: "fixed".into(),
            }],
        };
        let toolchain = "pinned actual toolchain".to_string();
        let raw = "eight case observations".to_string();
        let reference =
            |name: &str, b: &[u8]| depth_ci::reference(format!("{}/1-1/{name}", p.evidence()), b);
        let raw_specs = fs::read_to_string(repo.join(p.directory()).join("cases.json")).ok();
        let names = p.case_names(raw_specs.as_deref()).unwrap();
        let cases = Cases {
            requested: names
                .iter()
                .map(|name| CaseIdentity {
                    name: name.clone(),
                    executable_sha256: "a".repeat(64),
                })
                .collect(),
            executed: names
                .iter()
                .map(|name| CaseResult {
                    case: CaseIdentity {
                        name: name.clone(),
                        executable_sha256: "a".repeat(64),
                    },
                    outcome: CaseOutcome::Passed,
                })
                .collect(),
            raw_output: reference("raw.txt", raw.as_bytes()),
        };
        let mut absent = profile["packages"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|p| {
                p["origin"]["directory"]
                    .as_str()
                    .map(|d| format!("{d}/build.rs"))
            })
            .collect::<Vec<_>>();
        absent.extend(profile::OPTIONAL.map(str::to_owned));
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
                features: p.features(),
                environment: reference(
                    "environment.json",
                    &serde_json::to_vec(&environment).unwrap(),
                ),
                dependency_graph: reference("graph.json", &serde_json::to_vec(&graph).unwrap()),
                covered_inputs: inputs,
                absent_inputs: absent,
            },
            graph,
            environment,
            provenance: p
                .record(record)
                .unwrap()
                .obligations
                .unwrap()
                .acquired_inputs
                .into_iter()
                .map(|file| Provenance {
                    kind: ProvenanceKind::Fixture,
                    origin: "isolated fixture".into(),
                    file,
                })
                .collect(),
            cases,
            toolchain_report: toolchain,
            raw_output: raw,
        };
        let mut fixture = Self {
            profile: p,
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
        let p = self.profile;
        let parsed = p
            .record(&fs::read_to_string(self.root.path().join(p.fixture())).unwrap())
            .unwrap();
        let Event::Maintenance { request_commit, .. } = parsed.event else {
            panic!()
        };
        self.api.page(&format!("commits/{request_commit}/pulls"),None,vec![json!({"id":9,"number":9,"base":{"ref":"staging","repo":{"full_name":depth_ci::REPOSITORY}}})]);
        let run = json!({"id":1,"head_sha":commit,"head_repository":{"full_name":depth_ci::REPOSITORY},"path":profile::WORKFLOW,"event":"workflow_dispatch","run_attempt":attempts,"status":"completed"});
        self.api.page(
            &format!(
                "actions/workflows/ci.yml/runs?created=%3E%3D{}",
                if p == Profile::Channel {
                    "2026-10-23"
                } else {
                    "2026-10-25"
                }
            ),
            Some("workflow_runs"),
            vec![run.clone()],
        );
        let mut artifacts = vec![];
        for attempt in 1..=attempts {
            let mut w = self.w.clone();
            w.run_attempt = attempt;
            let artifact_id = 100 + attempt;
            let job_id = 200 + attempt;
            let name = format!("{}-1-{attempt}", p.artifact());
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
            self.api.page(&format!("actions/runs/1/attempts/{attempt}/jobs"),Some("jobs"),vec![json!({"id":job_id,"name":"Rust tests (workspace)","run_id":1,"run_attempt":attempt,"status":"completed","steps":([format!("Collect {} witness",if p==Profile::Channel {"isolated depth"} else {"isolated CORE depth"}),format!("Upload {} witness",if p==Profile::Channel {"isolated depth"} else {"isolated CORE depth"}),format!("Bind {} artifact",if p==Profile::Channel {"isolated depth"} else {"isolated CORE depth"})].map(|s|json!({"name":s,"status":"completed","conclusion":"success","started_at":"2026-10-23T15:00:00Z","completed_at":"2026-10-23T15:00:01Z"})))})]);
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
        let proposal = adoption::proposal_for(
            self.root.path(),
            &self.w,
            "independent-review-invocation",
            p,
        )
        .unwrap();
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
    assert!(artifact::witness(
        &f.api,
        &helper,
        &run,
        1,
        crate::depth_ci::profile::Profile::Channel
    )
    .is_ok());
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

#[test]
fn core_authenticated_bridge_evaluates_all_due_and_preserves_production_refusal() {
    let f = Fixture::new_for(Profile::Core);
    let raw = prove_profile(f.root.path(), &f.api, Profile::Core).unwrap();
    let result: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(result["obligations"].as_array().unwrap().len(), 12);
    assert!(
        result["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|o| o["result"] == "verified"),
        "{result}"
    );
    assert_eq!(result["production_enrollment"], false);
    let record = Profile::Core
        .record(&fs::read_to_string(f.root.path().join(Profile::Core.fixture())).unwrap())
        .unwrap();
    let production = crate::depth_evidence::production_results(f.root.path(), &record).unwrap();
    for obligation in Profile::Core.obligations(&record) {
        assert_eq!(
            production.result(&record, &obligation),
            DepthResult::Unverified
        );
    }
}
#[test]
fn core_unadopted_mapping_and_wrong_profile_cannot_verify() {
    let mut f = Fixture::new_for(Profile::Core);
    f.api.page("issues/9/comments", None, vec![]);
    assert!(prove_profile(f.root.path(), &f.api, Profile::Core)
        .unwrap_err()
        .contains("missing exact authenticated adequacy adoption"));
    assert!(resolve_profile(&f.api, f.root.path(), Profile::Channel).is_err());
}

#[test]
fn core_failed_and_missing_cases_are_not_hidden_by_other_green_obligations() {
    for missing in [false, true] {
        let mut f = Fixture::new_for(Profile::Core);
        let name = "adapter::parse_errors_do_not_leak_raw_lines_and_stream_continues";
        if missing {
            f.w.cases.executed.retain(|r| r.case.name != name);
        } else {
            f.w.cases
                .executed
                .iter_mut()
                .find(|r| r.case.name == name)
                .unwrap()
                .outcome = CaseOutcome::Failed;
        }
        f.install(1);
        let raw = prove_profile(f.root.path(), &f.api, Profile::Core).unwrap();
        let result: Value = serde_json::from_str(&raw).unwrap();
        let rows = result["obligations"].as_array().unwrap();
        let row = |id| rows.iter().find(|r| r["obligation"]["id"] == id).unwrap();
        assert_eq!(
            row("E1")["result"],
            if missing { "unverified" } else { "failed" },
            "{result}"
        );
        assert_eq!(row("A1")["result"], "verified", "{result}");
    }
}
#[test]
fn core_requires_every_exact_mapping_adoption() {
    let mut f = Fixture::new_for(Profile::Core);
    let mut proposal = adoption::proposal_for(
        f.root.path(),
        &f.w,
        "independent-review-invocation",
        Profile::Core,
    )
    .unwrap();
    proposal["mappings"].as_array_mut().unwrap().pop();
    f.api.page("issues/9/comments",None,vec![json!({"id":33,"user":{"login":"maintainer"},"body":format!("```depth-adequacy-v1\n{}\n```",proposal)})]);
    assert!(resolve_profile(&f.api, f.root.path(), Profile::Core)
        .unwrap_err()
        .contains("missing exact authenticated adequacy adoption"));
}
