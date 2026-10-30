use super::*;
use native::producer::{self, execution, registration::StartApi};
struct Host {
    inner: Fake,
    posts: Cell<usize>,
    body: RefCell<Option<String>>,
    ambiguous: bool,
    refuse: bool,
}
impl host::Api for Host {
    fn json(&self, repo: &form::Repository, path: &str) -> Result<serde_json::Value, String> {
        if let Some(body) = self.body.borrow().as_ref() {
            let row = json!({"id":90,"body":body,"user":{"id":7,"login":"maintainer"}});
            if path == "issues/8/comments?per_page=100&page=1" {
                return Ok(json!([row]));
            }
            if path == "issues/comments/90" {
                return Ok(row);
            }
        }
        self.inner.json(repo, path)
    }
}
impl StartApi for Host {
    fn post_start(&self, _: &form::Repository, _: u64, body: &str) -> Result<u64, String> {
        self.posts.set(self.posts.get() + 1);
        self.body.replace(Some(body.into()));
        if self.ambiguous {
            return Err("ambiguous POST".into());
        }
        if self.refuse {
            self.body.replace(Some("changed body".into()));
        }
        Ok(90)
    }
}
fn setup() -> (
    NativeFixture,
    crate::depth_evidence::native::CollectArgs,
    Host,
) {
    let mut f = NativeFixture::new();
    let root_path = f.f.root.path().to_path_buf();
    let root = root_path.as_path();
    let sha = "e".repeat(64);
    let requested = execution::requested(&sha);
    let template = f.claims.mapping.assertions[0].clone();
    let assertions: Vec<_> = requested
        .into_iter()
        .map(|case| AssertionBinding {
            case,
            ..template.clone()
        })
        .collect();
    f.p.specs.values_mut().for_each(|v| *v = assertions.clone());
    f.p.context = form::context::synthetic(&execution::requested(&sha), &f.p.graph.inputs[0]).1;
    let mappings: Vec<_> =
        f.p.specs
            .keys()
            .map(|obligation| AdequacyMapping {
                obligation: obligation.clone(),
                reviewer_identity: "reviewer".into(),
                executor_identity: "github:maintainer:7".into(),
                assertions: assertions.clone(),
            })
            .collect();
    let intent = write(
        root,
        "crates/xtask/data/depth-native-context-intent-v1.json",
        b"approved pre-run intent",
    );
    let mappings = canonical(
        root,
        "crates/xtask/data/depth-native-mappings-v1.json",
        &mappings,
    );
    f.p.graph.inputs.extend([intent, mappings]);
    f.p.graph.inputs.sort();
    let request = &f.f.current.record.event;
    let Event::Maintenance {
        request_commit,
        request_recorded_at,
        trigger_kind,
        opened_from,
    } = request
    else {
        panic!()
    };
    let request_commit = request_commit.clone();
    let raw=toml_edit::ser::to_string(&json!({"request_commit":request_commit,"request_recorded_at":request_recorded_at,"trigger_kind":trigger_kind,"opened_from":opened_from,"detected_release":{"target_version":"0.156.1"},"support_depth":{"policy_identity":f.f.current.record.policy.identity,"obligations_identity":f.f.current.record.obligations.as_ref().unwrap().identity}})).unwrap();
    write(
        root,
        &crate::agent_lifecycle::maintenance_request_path("codex"),
        raw,
    );
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "reviewed native producer inputs"]);
    let head = git(root, &["rev-parse", "HEAD"]);
    f.approved = head.clone();
    f.built = head.clone();
    f.host_setup();
    f.api.ancestor(&f.p.repository, &f.p.baseline, &head);
    f.api.ancestor(&f.p.repository, &request_commit, &head);
    f.api.put(&f.p.repository,&format!("commits/{request_commit}/pulls?per_page=100&page=1"),json!([{"id":8,"number":8,"base":{"ref":"staging","repo":{"id":1,"full_name":"fixture/isolated"}}}]));
    f.api.put(&f.p.repository,"pulls/8",json!({"number":8,"base":{"ref":"staging","repo":{"id":1}},"head":{"repo":{"id":1},"sha":head}}));
    let binary = root.join("native-test-binary");
    fs::write(&binary, b"not executed").unwrap();
    f.p.native.executable_sha256 = crate::depth_ci::reference("", b"not executed").sha256;
    let args = form::CollectArgs {
        live: true,
        register: true,
        binary,
        working_dir: root.join("unused"),
        codex_home: None,
        attempt: "new-attempt".into(),
    };
    let api = Host {
        inner: std::mem::take(&mut f.api),
        posts: Cell::new(0),
        body: RefCell::new(None),
        ambiguous: false,
        refuse: false,
    };
    (f, args, api)
}
fn built(root: &Path, prefix: &str, f: &NativeFixture) -> crate::depth_ci::native::Built {
    let graph = serde_json::to_vec(&f.p.graph).unwrap();
    let env = serde_json::to_vec(&f.p.environment).unwrap();
    let graphref = crate::depth_ci::reference(format!("{prefix}/graph.json"), &graph);
    let envref = crate::depth_ci::reference(format!("{prefix}/environment.json"), &env);
    let toolchain = fs::read(root.join(&f.p.toolchain.path)).unwrap();
    crate::depth_ci::native::Built {
        _target: tempfile::tempdir().unwrap(),
        executable: root.join("not-executed-collector"),
        executable_sha256: "e".repeat(64),
        build: Build {
            source_commit: git(root, &["rev-parse", "HEAD"]),
            source_tree: git(root, &["rev-parse", "HEAD^{tree}"]),
            host_target: f.p.environment.target.clone(),
            toolchain: f.p.toolchain.clone(),
            features: f.p.environment.features.clone(),
            environment: envref.clone(),
            dependency_graph: graphref.clone(),
            covered_inputs: f.p.graph.inputs.clone(),
            absent_inputs: [
                ".cargo/config",
                ".cargo/config.toml",
                "rust-toolchain",
                "rust-toolchain.toml",
                "crates/xtask/build.rs",
            ]
            .map(str::to_owned)
            .to_vec(),
        },
        payloads: vec![
            (graphref, graph),
            (envref, env),
            (f.p.toolchain.clone(), toolchain),
        ],
    }
}
#[test]
fn no_native_execution_after_failed_ambiguous_or_changed_registration() {
    for ambiguous in [true, false] {
        let (f, args, mut api) = setup();
        api.ambiguous = ambiguous;
        api.refuse = !ambiguous;
        let result = producer::collect_with(
            f.f.root.path(),
            args,
            &f.p,
            &api,
            |root, prefix| Ok(built(root, prefix, &f)),
            |_, _, _, _, _| panic!("native call before confirmed registration"),
        );
        assert!(result.is_err(), "unexpected result {result:?}");
        assert_eq!(api.posts.get(), 1, "{result:?}");
        let path =
            f.f.root.path().join(
                "cli_manifests/codex/reports/0.156.1/evidence/native-v1/new-attempt/plan.json",
            );
        assert!(
            path.is_file(),
            "plan must survive ambiguous registration: {result:?}"
        );
    }
}
#[test]
fn registered_failed_run_keeps_all_due_claims_full_population_and_v2_bundle() {
    let (f, args, api) = setup();
    let called = Cell::new(false);
    let result = producer::collect_with(
        f.f.root.path(),
        args,
        &f.p,
        &api,
        |root, prefix| Ok(built(root, prefix, &f)),
        |_, _, _, _, plan| {
            assert_eq!(api.posts.get(), 1);
            called.set(true);
            Ok(execution::Execution {
                cases: Cases {
                    requested: plan.requested.clone(),
                    executed: vec![CaseResult {
                        case: plan.requested[0].clone(),
                        outcome: CaseOutcome::Failed,
                    }],
                    raw_output: plan.context.clone(),
                },
                context: form::context::Observation::incomplete(),
                observations: json!({"status":"failed","unrun":&plan.requested[1..]}),
                complete: false,
            })
        },
    );
    assert!(result.is_err(), "{result:?}");
    assert!(called.get(), "{result:?}");
    assert_eq!(api.posts.get(), 1);
    let root = f.f.root.path();
    let prefix = "cli_manifests/codex/reports/0.156.1/evidence/native-v1/new-attempt";
    let bundle =
        form::Bundle::parse(&fs::read_to_string(root.join(prefix).join("bundle.json")).unwrap())
            .unwrap();
    assert_eq!(bundle.claims.len(), f.p.specs.len());
    assert!(!bundle
        .files
        .iter()
        .any(|f| f.path.ends_with("registration.json")));
    let plan =
        form::Plan::parse(&fs::read_to_string(root.join(bundle.plan.path)).unwrap()).unwrap();
    assert_eq!(plan.requested.len(), 12);
    for file in &bundle.claims {
        let bytes = fs::read(root.join(&file.path)).unwrap();
        assert_eq!(crate::depth_ci::reference(&file.path, &bytes), *file);
        let claim = form::Claim::parse(std::str::from_utf8(&bytes).unwrap()).unwrap();
        assert_eq!(claim.cases.requested.len(), 12);
        assert_eq!(claim.cases.executed[0].outcome, CaseOutcome::Failed);
    }
}
#[test]
fn committed_producer_bundle_prepares_only_unposted_false_attestation_template() {
    let (mut f, args, mut api) = setup();
    let cases = execution::requested(&"e".repeat(64));
    let (context, expected) = form::context::synthetic(&cases, &f.p.graph.inputs[0]);
    f.p.context = expected;
    producer::collect_with(
        f.f.root.path(),
        args,
        &f.p,
        &api,
        |root, prefix| Ok(built(root, prefix, &f)),
        |_, _, _, _, plan| {
            Ok(execution::Execution {
                cases: Cases {
                    requested: plan.requested.clone(),
                    executed: plan
                        .requested
                        .iter()
                        .map(|c| CaseResult {
                            case: c.clone(),
                            outcome: CaseOutcome::Passed,
                        })
                        .collect(),
                    raw_output: plan.context.clone(),
                },
                context,
                observations: json!({"fixture_only":true}),
                complete: true,
            })
        },
    )
    .unwrap();
    let root = f.f.root.path();
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "immutable produced bundle"]);
    let proof = git(root, &["rev-parse", "HEAD"]);
    api.inner.tree(root, &f.p.repository, &proof);
    for a in [&f.built, &f.p.baseline, &proof] {
        api.inner.ancestor(&f.p.repository, a, &proof);
    }
    if let Event::Maintenance { request_commit, .. } = &f.f.current.record.event {
        api.inner.ancestor(&f.p.repository, request_commit, &proof);
    }
    api.inner.put(&f.p.repository,"pulls/8",json!({"number":8,"base":{"ref":"staging","repo":{"id":1}},"head":{"repo":{"id":1},"sha":proof}}));
    let body = producer::prepare_finish(
        root,
        form::PrepareFinishArgs {
            attempt: "new-attempt".into(),
            evidence_commit: proof,
        },
        &f.p,
        &api,
    )
    .unwrap();
    let raw = body
        .split_once("```depth-native-finish-v1\n")
        .unwrap()
        .1
        .split_once("\n```")
        .unwrap()
        .0;
    let finish = form::Finish::parse(raw).unwrap();
    assert!(
        !finish.execution_attested
            && !finish.outcomes_attested
            && !finish.registration_order_attested
            && !finish.completeness_attested
    );
    assert_eq!(finish.registration.id, 90);
    assert_eq!(finish.registrations, vec![90]);
    assert_eq!(finish.executor_identity, "github:maintainer:7");
    assert_eq!(api.posts.get(), 1, "template preparation must never POST");
}
