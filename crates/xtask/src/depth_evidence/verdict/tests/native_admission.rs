use super::super::native::{self, host};
use super::*;
use crate::depth_evidence::native as form;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
};
mod regressions;
#[derive(Default)]
struct Fake {
    values: BTreeMap<String, serde_json::Value>,
    calls: Cell<usize>,
    sequences: RefCell<BTreeMap<String, std::collections::VecDeque<serde_json::Value>>>,
}
impl host::Api for Fake {
    fn json(&self, repo: &form::Repository, path: &str) -> Result<serde_json::Value, String> {
        self.calls.set(self.calls.get() + 1);
        let key = format!("{}/{path}", repo.name);
        if let Some(value) = self
            .sequences
            .borrow_mut()
            .get_mut(&key)
            .and_then(|s| s.pop_front())
        {
            return Ok(value);
        }
        self.values
            .get(&key)
            .cloned()
            .ok_or_else(|| format!("missing fake endpoint {}/{path}", repo.name))
    }
}
impl Fake {
    fn put(&mut self, repo: &form::Repository, path: &str, v: serde_json::Value) {
        self.values.insert(format!("{}/{path}", repo.name), v);
    }
    fn tree(&mut self, root: &Path, repo: &form::Repository, commit: &str) {
        let oid = git(root, &["rev-parse", &format!("{commit}^{{tree}}")]);
        self.put(
            repo,
            &format!("git/commits/{commit}"),
            json!({"sha":commit,"tree":{"sha":oid}}),
        );
        let rows: Vec<_> = git(root, &["ls-tree", "-r", "-t", commit])
            .lines()
            .map(|l| {
                let (head, path) = l.split_once('\t').unwrap();
                let v: Vec<_> = head.split_whitespace().collect();
                json!({"mode":v[0],"type":v[1],"sha":v[2],"path":path})
            })
            .collect();
        self.put(
            repo,
            &format!("git/trees/{oid}?recursive=1"),
            json!({"sha":oid,"truncated":false,"tree":rows}),
        );
    }
    fn ancestor(&mut self, repo: &form::Repository, a: &str, b: &str) {
        self.put(
            repo,
            &format!("compare/{a}...{b}"),
            json!({"status":if a==b{"identical"}else{"ahead"},"merge_base_commit":{"sha":a}}),
        );
    }
}
fn canonical<T: serde::Serialize>(root: &Path, path: &str, value: &T) -> FileRef {
    write(
        root,
        path,
        serde_json::to_vec(&serde_json::to_value(value).unwrap()).unwrap(),
    )
}
fn seal(record: &mut DepthRecord) {
    record.policy.identity = part_identity(&record.policy).unwrap();
    let o = record.obligations.as_mut().unwrap();
    o.policy_identity = record.policy.identity.clone();
    o.identity = part_identity(o).unwrap();
    if let Some(e) = record.execution.as_mut() {
        e.policy_identity = record.policy.identity.clone();
        e.obligations_identity = o.identity.clone();
        e.identity = part_identity(e).unwrap();
    }
}
struct Attempt {
    receipt: form::NativeReceipt,
    selected: FileRef,
    registration: serde_json::Value,
    completion: serde_json::Value,
    proof: String,
}
struct NativeFixture {
    f: Fixture,
    p: native::Profile,
    api: Fake,
    approved: String,
    built: String,
    attempts: Vec<Attempt>,
    claims: form::Claim,
}
impl NativeFixture {
    fn new() -> Self {
        Self::new_outcome(CaseOutcome::Passed)
    }
    fn new_outcome(outcome: CaseOutcome) -> Self {
        use crate::agent_registry::{support_depth::SupportDepth, REGISTRY_RELATIVE_PATH};
        use crate::depth_evidence::payload::{Package, PackageOrigin};
        let mut f = Fixture::new();
        let root = f.root.path();
        let canonical_registry = include_str!("../../../../data/agent_registry.toml");
        write(root, REGISTRY_RELATIVE_PATH, canonical_registry);
        let acquisition = write(
            root,
            "docs/requirements/native.json",
            b"pinned native acquisition identity",
        );
        let manifest = write(root, "crates/xtask/Cargo.toml", "[package]");
        f.graph.inputs.extend([
            acquisition.clone(),
            manifest,
            f.receipt.producer.collector.clone(),
        ]);
        f.graph
            .scopes
            .extend(["crates/xtask".into(), "scripts".into()]);
        f.graph.packages.push(Package {
            id: "xtask@1.0.0".into(),
            origin: PackageOrigin::Local {
                directory: "crates/xtask".into(),
            },
            dependencies: f.graph.roots.clone(),
        });
        f.graph.roots = vec!["xtask@1.0.0".into()];
        git(root, &["add", "."]);
        git(root, &["commit", "-qm", "canonical approved inputs"]);
        let approved = git(root, &["rev-parse", "HEAD"]);
        let mut rec = f.current.record.clone();
        let execution = rec.execution.take().unwrap();
        rec.policy.selection.enrollment.version = "0.156.1".into();
        rec.policy.selection.enrollment.targets = vec!["x86_64-unknown-linux-musl".into()];
        rec.obligations.as_mut().unwrap().targets = vec!["x86_64-unknown-linux-musl".into()];
        if let Event::Maintenance { request_commit, .. } = &mut rec.event {
            *request_commit = approved.clone();
        }
        rec.obligations.as_mut().unwrap().operations[0].obligations =
            rec.policy.selection.enrollment.operations[0]
                .obligations()
                .into_iter()
                .collect();
        rec.obligations.as_mut().unwrap().operations[0]
            .obligations
            .sort_by_key(|v| serde_json::to_string(v).unwrap());
        rec.policy.base = crate::agent_registry::support_depth::ObligationId::BASE.to_vec();
        rec.policy
            .base
            .sort_by_key(|v| serde_json::to_string(v).unwrap());
        rec.policy.templates[0].obligations = rec.policy.selection.enrollment.operations[0]
            .templates[0]
            .obligations()
            .to_vec();
        rec.policy.templates[0]
            .obligations
            .sort_by_key(|v| serde_json::to_string(v).unwrap());
        seal(&mut rec);
        let selection = rec.policy.selection.enrollment.clone();
        let depth: SupportDepth = serde_json::from_value(
            json!({"enabled_paths":["maintenance"],"enrollments":[selection]}),
        )
        .unwrap();
        let mut document: toml_edit::DocumentMut = canonical_registry.parse().unwrap();
        let table = document["agents"]
            .as_array_of_tables_mut()
            .unwrap()
            .iter_mut()
            .find(|a| a["agent_id"].as_str() == Some("codex"))
            .unwrap();
        table.insert(
            "support_depth",
            toml_edit::Item::Table(
                toml_edit::ser::to_document(&depth)
                    .unwrap()
                    .as_table()
                    .clone(),
            ),
        );
        let registry = write(root, REGISTRY_RELATIVE_PATH, document.to_string());
        f.graph.inputs.push(registry.clone());
        f.graph.inputs.sort();
        let record_path = "cli_manifests/codex/reports/0.156.1/depth-record.json".to_owned();
        save(root, &record_path, &rec);
        git(root, &["add", "."]);
        git(root, &["commit", "-qm", "isolated frozen packet"]);
        let built = git(root, &["rev-parse", "HEAD"]);
        f.receipt.scope.version = "0.156.1".into();
        f.receipt.scope.obligation.target = "x86_64-unknown-linux-musl".into();
        f.receipt.scope.event = rec.event.clone();
        f.receipt.scope.policy_identity = rec.policy.identity.clone();
        f.receipt.scope.obligations_identity = rec.obligations.as_ref().unwrap().identity.clone();
        f.receipt.scope.trust_revision = "maintainer-native-v1".into();
        f.receipt.scope.review_revision = "maintainer-mapping-v1".into();
        let base = "cli_manifests/codex/reports/0.156.1/evidence/native-v1/shared";
        let toolchain = write(
            root,
            &format!("{base}/toolchain.txt"),
            b"rustc actual fixture toolchain",
        );
        f.receipt.build.source_commit = built.clone();
        f.receipt.build.source_tree = git(root, &["rev-parse", "HEAD^{tree}"]);
        f.receipt.build.covered_inputs = f.graph.inputs.clone();
        f.receipt
            .build
            .absent_inputs
            .push("crates/xtask/build.rs".into());
        f.receipt.build.toolchain = toolchain.clone();
        f.current.current = f.receipt.scope.clone();
        f.current.toolchain = toolchain.clone();
        rec.execution = Some(execution);
        rec.execution.as_mut().unwrap().results[0].obligation =
            f.current.current.obligation.clone();
        seal(&mut rec);
        f.current.record = rec;
        let context_sha256 = write(
            root,
            &format!("{base}/context.txt"),
            b"finite approved nonsecret context",
        )
        .sha256;
        f.receipt.provenance.push(Provenance {
            kind: ProvenanceKind::Binary,
            origin: "pinned native acquisition".into(),
            file: acquisition.clone(),
        });
        let claims = form::Claim {
            scope: f.current.current.clone(),
            build: f.receipt.build.clone(),
            provenance: f.receipt.provenance.clone(),
            cases: f.receipt.cases.clone(),
            mapping: f.receipt.adequacy.mapping.clone(),
        };
        let p = native::Profile {
            repository: form::Repository {
                name: "fixture/isolated".into(),
                id: 1,
                owner_id: 2,
            },
            canonical: form::Repository {
                name: "fixture/canonical".into(),
                id: 3,
                owner_id: 2,
            },
            baseline: approved.clone(),
            registry,
            selection,
            record_path,
            collector: f.receipt.producer.collector.clone(),
            native: form::Runtime {
                version: "0.156.1".into(),
                target: "x86_64-unknown-linux-musl".into(),
                executable_sha256: "f".repeat(64),
                acquisition,
            },
            graph: f.graph.clone(),
            environment: f.current.environment.clone(),
            toolchain,
            context_intent_sha256: crate::depth_ci::reference("", b"approved pre-run intent")
                .sha256,
            context_sha256,
            definition_sha256: f.current.definition_sha256.clone(),
            specs: crate::depth_record::due(&f.current.record)
                .unwrap()
                .into_iter()
                .map(|o| (o, claims.mapping.assertions.clone()))
                .collect(),
        };
        let mut this = Self {
            f,
            p,
            api: Fake::default(),
            approved,
            built,
            attempts: vec![],
            claims,
        };
        this.host_setup();
        this.install(10, outcome);
        this.select(0);
        this
    }
    fn host_setup(&mut self) {
        let root = self.f.root.path();
        for (repo, tip) in [
            (&self.p.canonical, &self.approved),
            (&self.p.repository, &self.built),
        ] {
            self.api.put(
                repo,
                "",
                json!({"id":repo.id,"owner":{"id":repo.owner_id},"full_name":repo.name}),
            );
            self.api.put(
                repo,
                "branches/staging",
                json!({"name":"staging","protected":true,"commit":{"sha":tip}}),
            );
            self.api.tree(root, repo, tip);
        }
        self.api
            .ancestor(&self.p.repository, &self.approved, &self.built);
        self.api
            .ancestor(&self.p.repository, &self.built, &self.built);
        self.api.put(&self.p.repository,&format!("commits/{}/pulls?per_page=100&page=1",self.approved),json!([{"id":8,"number":8,"base":{"ref":"staging","repo":{"id":1,"full_name":"fixture/isolated"}}}]));
        self.api.put(
            &self.p.canonical,
            "collaborators/maintainer/permission",
            json!({"permission":"write","user":{"id":7,"login":"maintainer"}}),
        );
        self.api.put(
            &self.p.canonical,
            "collaborators/outsider/permission",
            json!({"permission":"read","user":{"id":9,"login":"outsider"}}),
        );
    }
    fn install(&mut self, id: u64, outcome: CaseOutcome) {
        self.install_with(id, outcome, |_, _| {});
    }
    fn install_with(
        &mut self,
        id: u64,
        outcome: CaseOutcome,
        mutate: impl FnOnce(&mut form::Plan, &mut Vec<form::Claim>),
    ) {
        let root = self.f.root.path();
        let prefix = format!("cli_manifests/codex/reports/0.156.1/evidence/native-v1/{id}");
        let mut claim = self.claims.clone();
        claim.build.dependency_graph =
            canonical(root, &format!("{prefix}/graph.json"), &self.p.graph);
        claim.build.environment = canonical(
            root,
            &format!("{prefix}/environment.json"),
            &self.p.environment,
        );
        claim.cases.raw_output = write(
            root,
            &format!("{prefix}/raw.txt"),
            format!("truthfully retained {outcome:?}"),
        );
        claim.cases.executed[0].outcome = outcome;
        claim.mapping.obligation = claim.scope.obligation.clone();
        claim.mapping.executor_identity = format!("native-executor-{id}");
        claim.mapping.reviewer_identity = "independent-review".into();
        let context = write(
            root,
            &format!("{prefix}/context.txt"),
            b"finite approved nonsecret context",
        );
        let mut plan = form::Plan {
            schema_version: 1,
            scope: claim.scope.clone(),
            packet_head: self.built.clone(),
            build: claim.build.clone(),
            native: self.p.native.clone(),
            context: write(
                root,
                &format!("{prefix}/intent.txt"),
                b"approved pre-run intent",
            ),
            collector: self.p.collector.clone(),
            requested: claim.cases.requested.clone(),
        };
        let mut all_claims: Vec<_> = self
            .p
            .specs
            .keys()
            .map(|o| {
                let mut c = claim.clone();
                c.scope.obligation = o.clone();
                c.mapping.obligation = o.clone();
                c
            })
            .collect();
        mutate(&mut plan, &mut all_claims);
        let plan_ref = canonical(root, &format!("{prefix}/plan.json"), &plan);
        let claim_refs: Vec<_> = all_claims
            .iter()
            .enumerate()
            .map(|(n, c)| canonical(root, &format!("{prefix}/claim-{n}.json"), c))
            .collect();
        let index = form::Bundle {
            schema_version: 1,
            observed_context: context.clone(),
            plan: plan_ref.clone(),
            claims: claim_refs.clone(),
            files: vec![
                plan_ref.clone(),
                claim.build.dependency_graph.clone(),
                claim.build.environment.clone(),
                self.p.toolchain.clone(),
                context,
                plan.context.clone(),
                claim.cases.raw_output.clone(),
            ],
        };
        let mut index = index;
        index.files.extend(claim_refs);
        let bundle_ref = canonical(root, &format!("{prefix}/bundle.json"), &index);
        git(root, &["add", "."]);
        git(root, &["commit", "-qm", "retain actual attempt proof"]);
        let proof = git(root, &["rev-parse", "HEAD"]);
        let start = form::Start {
            schema_version: 1,
            profile: "isolated-codex-native-v1".into(),
            repository: self.p.repository.clone(),
            pull_request: 8,
            packet_head: self.built.clone(),
            scope: claim.scope.clone(),
            plan: plan_ref,
        };
        let start_body = format!(
            "```depth-native-start-v1\n{}\n```",
            serde_json::to_string(&start).unwrap()
        );
        let registration = form::CommentRef {
            id,
            body_sha256: crate::depth_ci::reference("", start_body.as_bytes()).sha256,
        };
        let finish = form::Finish {
            schema_version: 1,
            registration: registration.clone(),
            evidence_commit: proof.clone(),
            bundle: bundle_ref,
            executor_identity: claim.mapping.executor_identity.clone(),
            population_through_id: id,
            registrations: self
                .attempts
                .iter()
                .map(|a| a.receipt.producer.registration.id)
                .chain([id])
                .collect(),
            execution_attested: true,
            outcomes_attested: true,
            registration_order_attested: true,
            completeness_attested: true,
        };
        let mappings:Vec<_>=all_claims.iter().map(|c|json!({"mapping_sha256":c.mapping.sha256().unwrap(),"reviewer_identity":c.mapping.reviewer_identity,"executor_identity":c.mapping.executor_identity,"independence_attested":true,"counterfactual_adopted":true})).collect();
        let adoption = json!({"schema_version":1,"source_commit":self.built,"mappings":mappings});
        let finish_body = format!(
            "```depth-native-finish-v1\n{}\n```\n```depth-adequacy-v1\n{}\n```",
            serde_json::to_string(&finish).unwrap(),
            adoption
        );
        let completion = form::CommentRef {
            id: id + 10,
            body_sha256: crate::depth_ci::reference("", finish_body.as_bytes()).sha256,
        };
        let receipt = form::NativeReceipt {
            schema_version: 2,
            scope: claim.scope.clone(),
            producer: form::NativeProducer {
                kind: form::Kind::MaintainerNative,
                repository: self.p.repository.clone(),
                pull_request: 8,
                registration,
                completion: completion.clone(),
                collector: self.p.collector.clone(),
            },
            build: claim.build,
            provenance: claim.provenance,
            cases: claim.cases,
            adequacy: Adequacy {
                mapping: claim.mapping.clone(),
                endorsement: Endorsement {
                    kind: EndorsementKind::IssueComment,
                    repository: self.p.repository.name.clone(),
                    pull_request: 8,
                    id: completion.id,
                    author: "maintainer".into(),
                    source_commit: self.built.clone(),
                    body_sha256: completion.body_sha256,
                    mapping_sha256: claim.mapping.sha256().unwrap(),
                },
            },
        };
        let selected = save(root, &format!("{prefix}/receipt.json"), &receipt);
        git(root, &["add", "."]);
        self.api.tree(root, &self.p.repository, &proof);
        self.api.ancestor(&self.p.repository, &self.built, &proof);
        self.api
            .ancestor(&self.p.repository, &self.approved, &proof);
        for a in &self.attempts {
            self.api.ancestor(&self.p.repository, &a.proof, &proof);
        }
        self.api.ancestor(&self.p.repository, &proof, &proof);
        self.api.put(&self.p.repository,"pulls/8",json!({"number":8,"base":{"ref":"staging","repo":{"id":1}},"head":{"sha":proof,"repo":{"id":1}}}));
        let registration = json!({"id":id,"body":start_body,"user":{"id":7,"login":"maintainer"}});
        let completion =
            json!({"id":id+10,"body":finish_body,"user":{"id":7,"login":"maintainer"}});
        self.attempts.push(Attempt {
            receipt,
            selected,
            registration,
            completion,
            proof,
        });
        self.comments();
    }
    fn comments(&mut self) {
        let rows: Vec<_> = self
            .attempts
            .iter()
            .flat_map(|a| [a.registration.clone(), a.completion.clone()])
            .collect();
        self.api.put(
            &self.p.repository,
            "issues/8/comments?per_page=100&page=1",
            json!(rows),
        );
    }
    fn select(&mut self, n: usize) {
        self.f.selected = self.attempts[n].selected.clone();
        self.f.current.record.execution.as_mut().unwrap().results[0].evidence =
            vec![self.f.selected.clone()];
        seal(&mut self.f.current.record);
    }
    fn evaluate(&self) -> Result<Verdict, String> {
        let mut view = GitInputs::new(self.f.root.path())?;
        let raw = read(&mut view, &self.f.selected)?.ok_or("missing test receipt")?;
        let r = form::NativeReceipt::parse(text(&raw)?)?;
        native::admit(
            self.f.root.path(),
            &mut view,
            &self.f.selected,
            &self.f.current,
            &r,
            &self.p,
            &self.api,
        )
    }
}
#[test]
fn authenticated_native_bundle_reaches_shared_predicate_with_distinct_target_roles() {
    let f = NativeFixture::new();
    let verdict = f.evaluate().unwrap();
    assert_eq!(verdict.result, DepthResult::Verified, "{}", verdict.reason);
    assert_ne!(f.p.environment.target, f.p.native.target);
    let public = evaluate_workspace(f.f.root.path(), &f.f.selected, &f.f.current).unwrap();
    assert_eq!(public.result, DepthResult::Unverified);
    assert!(public.reason.contains("not installed"));
}
#[test]
fn native_complete_population_preserves_counterexamples_and_later_missing_attempts() {
    let mut f = NativeFixture::new();
    f.install(30, CaseOutcome::Failed);
    f.select(0);
    assert_eq!(f.evaluate().unwrap().result, DepthResult::Unverified);
    f.select(1);
    assert_eq!(f.evaluate().unwrap().result, DepthResult::Unverified);
    f.attempts.remove(0);
    f.attempts[0].completion["body"] = json!(f.attempts[0].completion["body"]
        .as_str()
        .unwrap()
        .replace("[10,30]", "[30]"));
    f.comments();
    assert_ne!(
        f.evaluate().map(|v| v.result).ok(),
        Some(DepthResult::Verified)
    );
}
