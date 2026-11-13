//! Faithful source/artifact/adoption construction for the fake authenticated host.
use super::*;

/// Keep a file the workspace already holds, or write the fixture bytes; either way cover it.
fn keep_or_write(root: &Path, path: &str, bytes: impl FnOnce() -> Vec<u8>) -> FileRef {
    match fs::read(root.join(path)) {
        Ok(existing) => depth_ci::reference(path, &existing),
        Err(_) => write(root, path, bytes()),
    }
}
/// Write the profile's producer closure into `path` (keeping files it already holds), commit,
/// and return the faithful witness of that commit. A workspace may supply its own frozen record.
pub(super) fn build(path: &Path, p: Profile) -> (Witness, String) {
    {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let record_raw = fs::read_to_string(path.join(p.fixture()))
            .unwrap_or_else(|_| fs::read_to_string(repo.join(p.fixture())).unwrap());
        let record = record_raw.as_str();
        let mut inputs = vec![];
        for (file, raw) in [
            (depth_ci::FIXTURE, record),
            (
                depth_ci::CONTRACT,
                include_str!("../../../../../tests/fixtures/depth-ci/contract.md"),
            ),
            (
                "crates/xtask/tests/fixtures/depth-ci/build-profile.json",
                include_str!("../../../../../tests/fixtures/depth-ci/build-profile.json"),
            ),
            ("Cargo.toml", "[workspace]"),
            ("Cargo.lock", "locked"),
            ("crates/wrapper_events/Cargo.toml", "[package]"),
            (depth_ci::PRODUCTION, "fn validate() {}"),
            (depth_ci::ASSERTION, "fn test_grammar() {}"),
        ] {
            inputs.push(keep_or_write(path, file, || raw.as_bytes().to_vec()));
        }
        if p != Profile::Channel {
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
            if let Some(scope) = p.requirements() {
                files.extend(
                    crate::runtime_advertising::source::Source::tip(&repo, "HEAD")
                        .unwrap()
                        .committed_files(scope)
                        .unwrap(),
                );
            }
            for path_name in files {
                inputs.push(keep_or_write(path, &path_name, || {
                    fs::read(repo.join(&path_name)).unwrap()
                }));
            }
        }
        keep_or_write(path, profile::WORKFLOW, || {
            b"approved exact workflow".to_vec()
        });
        keep_or_write(path, depth_ci::COLLECTOR, || {
            b"approved collector closure".to_vec()
        });
        keep_or_write(path, depth_ci::ARCHIVE, || {
            include_bytes!("../../../../../../../scripts/depth_ci_archive.py").to_vec()
        });
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
                inputs.push(keep_or_write(path, &name, || b"[package]".to_vec()));
            }
        }
        if p == Profile::SyntheticOnboarding {
            inputs.push(depth_ci::reference(
                depth_ci::onboarding::APPROVAL,
                &fs::read(path.join(depth_ci::onboarding::APPROVAL)).unwrap(),
            ));
            // ON2 installs the whole real wrapper, including files outside named assertions.
            let source = crate::runtime_advertising::source::Source::workspace(path).unwrap();
            for d in &local_dirs {
                for file in source.paths(d).unwrap() {
                    if !path.join(&file).is_file() {
                        continue;
                    }
                    if !inputs.iter().any(|i| i.path == file) {
                        inputs.push(depth_ci::reference(
                            &file,
                            &fs::read(path.join(&file)).unwrap(),
                        ));
                    }
                }
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
                match p {
                    Profile::Channel => "wrapper_events",
                    Profile::Core => "agent_api",
                    Profile::Synthetic | Profile::SyntheticOnboarding => "depth_fixture",
                },
                p.package()
            )],
            packages: serde_json::from_value(profile["packages"].clone()).unwrap(),
            inputs: inputs.clone(),
            scopes: std::iter::once(p.directory().to_owned())
                .chain(local_dirs.iter().map(|d| (*d).to_owned()))
                .chain(p.requirements().map(str::to_owned))
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
                toolchain: if p == Profile::SyntheticOnboarding {
                    depth_ci::reference(
                        format!(
                            "{}/toolchains/{}.json",
                            p.evidence(),
                            depth_ci::reference("", toolchain.as_bytes()).sha256
                        ),
                        toolchain.as_bytes(),
                    )
                } else {
                    reference("toolchain.json", toolchain.as_bytes())
                },
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
        (w, commit)
    }
}
/// The fake host serving one run with `attempts` faithful attempts of `w`, and its adoption.
pub(super) fn endpoints(root: &Path, p: Profile, w: &Witness, commit: &str, attempts: u64) -> Fake {
    endpoints_for_run(root, p, w, commit, 1, attempts)
}
pub(super) fn endpoints_for_run(
    root: &Path,
    p: Profile,
    w: &Witness,
    commit: &str,
    id: u64,
    attempts: u64,
) -> Fake {
    let mut fake = Fake::default();
    {
        let this = &mut fake;
        if p.synthetic() {
            this.1 = Some(ISOLATED);
        }
        let repository = this.repository().to_owned();
        let identity = this.identity_endpoint();
        this.0.insert(
            identity,
            serde_json::to_vec(&json!({"id":10,"owner":{"id":20},"full_name":repository})).unwrap(),
        );
        this.put("git/ref/heads/staging", json!({"object":{"sha":commit}}));
        let parsed = p
            .record(&fs::read_to_string(root.join(p.fixture())).unwrap())
            .unwrap();
        let (request_commit, request_recorded_at) = match parsed.event {
            Event::Maintenance {
                request_commit,
                request_recorded_at,
                ..
            } => (request_commit, request_recorded_at),
            Event::Onboarding {
                approval_commit,
                approval_recorded_at,
            } => (approval_commit, approval_recorded_at),
        };
        this.page(
            &format!("commits/{request_commit}/pulls"),
            None,
            vec![
                json!({"id":9,"number":9,"base":{"ref":"staging","repo":{"full_name":repository}}}),
            ],
        );
        let run = json!({"id":id,"head_sha":commit,"head_repository":{"full_name":repository},"path":profile::WORKFLOW,"event":"workflow_dispatch","run_attempt":attempts,"status":"completed"});
        this.page(
            &format!(
                "actions/workflows/ci.yml/runs?created=%3E%3D{}",
                &request_recorded_at[..10]
            ),
            Some("workflow_runs"),
            vec![run.clone()],
        );
        let mut artifacts = vec![];
        for attempt in 1..=attempts {
            let mut w = w.clone();
            w.run_id = id;
            w.run_attempt = attempt;
            for reference in [
                &mut w.build.dependency_graph,
                &mut w.build.environment,
                &mut w.cases.raw_output,
            ] {
                let leaf = reference.path.rsplit('/').next().unwrap();
                reference.path = format!("{}/{id}-{attempt}/{leaf}", p.evidence());
            }
            let artifact_id = (id - 1) * 1000 + 100 + attempt;
            let job_id = (id - 1) * 1000 + 200 + attempt;
            let name = format!("{}-{id}-{attempt}", p.artifact());
            let file = root.join("witness.json");
            fs::write(&file, serde_json::to_vec(&w).unwrap()).unwrap();
            let zip=depth_ci::output(Command::new("python3").args(["-c","import sys,zipfile,io; b=io.BytesIO(); z=zipfile.ZipFile(b,'w'); z.write(sys.argv[1],'witness.json'); z.close(); sys.stdout.buffer.write(b.getvalue())"]).arg(&file)).unwrap();
            fs::remove_file(file).unwrap();
            let digest = depth_ci::reference("", &zip).sha256;
            artifacts.push(json!({"id":artifact_id,"name":name,"expired":false,"size_in_bytes":zip.len(),"digest":format!("sha256:{digest}"),"workflow_run":{"id":id,"head_sha":commit}}));
            this.0.insert(
                this.endpoint(&format!("actions/artifacts/{artifact_id}/zip")),
                zip,
            );
            let mut attempt_run = run.clone();
            attempt_run["run_attempt"] = json!(attempt);
            this.put(
                &format!("actions/runs/{id}/attempts/{attempt}"),
                attempt_run,
            );
            this.page(&format!("actions/runs/{id}/attempts/{attempt}/jobs"),Some("jobs"),vec![json!({"id":job_id,"name":"Rust tests (workspace)","run_id":id,"run_attempt":attempt,"status":"completed","conclusion":"success","steps":([format!("Collect {} witness",p.step_label()),format!("Upload {} witness",p.step_label()),format!("Bind {} artifact",p.step_label())].map(|s|json!({"name":s,"status":"completed","conclusion":"success","started_at":"2026-10-23T15:00:00Z","completed_at":"2026-10-23T15:00:01Z"})))})]);
            this.0.insert(
                this.endpoint(&format!("actions/jobs/{job_id}/logs")),
                format!(
                    "2026-10-23T15:00:00.123456Z DEPTH_ARTIFACT_BINDING={}\n",
                    json!({"id":artifact_id,"digest":digest,"name":name})
                )
                .into_bytes(),
            );
        }
        this.page(
            &format!("actions/runs/{id}/artifacts"),
            Some("artifacts"),
            artifacts,
        );
        let proposal =
            adoption::proposal_for(root, &repository, w, "independent-review-invocation", p)
                .unwrap();
        this.page("issues/9/comments",None,vec![json!({"id":33,"user":{"login":"maintainer"},"body":format!("```depth-adequacy-v1\n{}\n```",proposal)})]);
        this.page("pulls/9/reviews", None, vec![]);
        this.put(
            "collaborators/maintainer/permission",
            json!({"permission":"write"}),
        );
    }
    fake
}

#[cfg(unix)]
pub(super) fn onboarding_workspace() -> tempfile::TempDir {
    // One immutable construction, then private clones. Every fake-host installation is scoped
    // inside construction and dropped before publication through OnceLock. Call order cannot
    // affect either the template or any caller's mutable repository/host.
    static TEMPLATE: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    let template = TEMPLATE.get_or_init(crate::isolated_maintenance::onboarding_producer_workspace);
    let root = tempfile::tempdir().unwrap();
    git(
        template.path(),
        &[
            "clone",
            "--quiet",
            "--no-hardlinks",
            ".",
            root.path().to_str().unwrap(),
        ],
    );
    root
}
