use super::*;
use crate::agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH};
use serde_json::json;
use std::{collections::BTreeMap, fs, sync::Mutex};

#[derive(Default)]
pub(super) struct Fake(
    pub(super) Mutex<BTreeMap<String, Value>>,
    pub(super) Mutex<BTreeMap<String, std::collections::VecDeque<Value>>>,
    pub(super) std::sync::atomic::AtomicUsize,
);
impl Api for Fake {
    fn json(&self, endpoint: &str) -> Result<Value, String> {
        self.2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if let Some(value) = self
            .1
            .lock()
            .unwrap()
            .get_mut(endpoint)
            .and_then(|values| values.pop_front())
        {
            return Ok(value);
        }
        self.0
            .lock()
            .unwrap()
            .get(endpoint)
            .cloned()
            .ok_or_else(|| format!("missing {endpoint}"))
    }
}
impl Fake {
    pub(super) fn put(&self, endpoint: &str, value: Value) {
        self.0.lock().unwrap().insert(endpoint.into(), value);
    }
}
pub(super) struct Fixture {
    pub(super) root: tempfile::TempDir,
    pub(super) api: Arc<Fake>,
    pub(super) baseline: String,
}
pub(super) fn git(root: &Path, args: &[&str]) -> String {
    let out = crate::git_inputs::command(root)
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
pub(super) fn write(root: &Path, path: &str, bytes: impl AsRef<[u8]>) {
    fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
    fs::write(root.join(path), bytes).unwrap();
}
pub(super) fn declaration() -> Value {
    json!({"enabled_paths":["maintenance"],"enrollments":[{
        "lifecycle_path":"maintenance","version":"0.156.1","targets":["x86_64-unknown-linux-musl"],
        "modes":[{"selector":"run-json","command_path":"codex exec","mode":"headless","predicates":[{"flag":"--json"}]}],
        "operations":[{"id":"run","serves":["run-json"],"templates":["CORE"],"shared_mapping":"mapped","forwarding":false,
        "promise":{"r1":"r","v1":"v","g1":"g","e1":"e","a1":"a","x1":"x","x2":"x","x3":"x","x4":"x"}}],
        "capabilities":[{"id":"agent_api.run","operations":["run"]}]}]})
}
impl Fixture {
    pub(super) fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        git(root.path(), &["init", "-b", "staging"]);
        git(
            root.path(),
            &["config", "user.email", "binding@example.invalid"],
        );
        git(root.path(), &["config", "user.name", "Binding Fixture"]);
        git(
            root.path(),
            &[
                "remote",
                "add",
                "origin",
                "https://github.com/fixture/isolated.git",
            ],
        );
        let mut registry =
            AgentRegistry::parse_unenforced(include_str!("../../../data/agent_registry.toml"))
                .unwrap();
        let codex = registry
            .agents
            .iter_mut()
            .find(|a| a.agent_id == "codex")
            .unwrap();
        codex.support_depth = Some(serde_json::from_value(declaration()).unwrap());
        // The production registry remains unchanged; serialize the fixture with its existing TOML.
        let mut document: toml_edit::DocumentMut =
            include_str!("../../../data/agent_registry.toml")
                .parse()
                .unwrap();
        let agents = document["agents"].as_array_of_tables_mut().unwrap();
        let table = agents
            .iter_mut()
            .find(|a| a["agent_id"].as_str() == Some("codex"))
            .unwrap();
        let depth = toml_edit::ser::to_document(codex.support_depth.as_ref().unwrap()).unwrap();
        table.insert(
            "support_depth",
            toml_edit::Item::Table(depth.as_table().clone()),
        );
        write(root.path(), REGISTRY_RELATIVE_PATH, document.to_string());
        write(root.path(), "producer/source.rs", "approved producer\n");
        write(root.path(), "docs/input.txt", "initial\n");
        git(root.path(), &["add", "."]);
        git(root.path(), &["commit", "-m", "fixture original baseline"]);
        let baseline = git(root.path(), &["rev-parse", "HEAD"]);
        let fixture = Self {
            root,
            api: Arc::new(Fake::default()),
            baseline,
        };
        fixture.host();
        fixture
    }
    pub(super) fn profile(&self) -> Profile {
        Profile {
            isolated: Repository {
                id: 10,
                owner: 20,
                name: "fixture/isolated",
            },
            canonical: Repository {
                id: 30,
                owner: 40,
                name: "fixture/canonical",
            },
            baseline: self.baseline.clone(),
            closure: vec![("producer/source.rs", b"approved producer\n", false)],
            scopes: vec!["producer"],
        }
    }
    pub(super) fn host(&self) {
        let head = git(self.root.path(), &["rev-parse", "HEAD"]);
        for (name, id, owner) in [("fixture/isolated", 10, 20), ("fixture/canonical", 30, 40)] {
            self.api.put(
                &format!("repos/{name}"),
                json!({"id":id,"owner":{"id":owner},"full_name":name}),
            );
            self.api.put(
                &format!("repos/{name}/branches/staging"),
                json!({"name":"staging","protected":true,"commit":{"sha":head}}),
            );
        }
        self.api.put(&format!("repos/fixture/isolated/compare/{}...{head}", self.baseline), json!({"status":if head == self.baseline { "identical" } else { "ahead" },"merge_base_commit":{"sha":self.baseline}}));
        self.api.put(
            &format!("repos/fixture/isolated/compare/{head}...{head}"),
            json!({"status":"identical","merge_base_commit":{"sha":head}}),
        );
        self.api.put(
            &format!("repos/fixture/isolated/commits/{head}"),
            json!({"sha":head}),
        );
        let registry = git(
            self.root.path(),
            &["rev-parse", &format!("HEAD:{REGISTRY_RELATIVE_PATH}")],
        );
        self.api.put(
            &format!("repos/fixture/isolated/contents/{REGISTRY_RELATIVE_PATH}?ref={head}"),
            json!({"path":REGISTRY_RELATIVE_PATH,"type":"file","sha":registry}),
        );
        let oid = git(self.root.path(), &["rev-parse", "HEAD:producer/source.rs"]);
        self.api.put(&format!("repos/fixture/canonical/git/trees/{head}?recursive=1"), json!({"truncated":false,"tree":[{"path":"producer","type":"tree","mode":"040000","sha":"unused"},{"path":"producer/source.rs","type":"blob","mode":"100644","sha":oid}]}));
    }
    pub(super) fn resolve(&self) -> Result<Context, String> {
        Context::resolve(self.root.path(), Some(self.profile()), self.api.clone())
    }
    pub(super) fn stage(&self) {
        git(self.root.path(), &["add", "."]);
    }
}
impl Fixture {
    pub(super) fn commit(&self) {
        self.stage();
        git(
            self.root.path(),
            &["commit", "-m", "fixture protected change"],
        );
        self.host();
    }
    pub(super) fn registry(&self, edit: impl FnOnce(&mut toml_edit::DocumentMut)) {
        let raw = fs::read_to_string(self.root.path().join(REGISTRY_RELATIVE_PATH)).unwrap();
        let mut doc = raw.parse().unwrap();
        edit(&mut doc);
        write(self.root.path(), REGISTRY_RELATIVE_PATH, doc.to_string());
    }
    pub(super) fn retained(&self) -> String {
        use crate::{
            agent_registry::support_depth::{ObligationId, ObligationTemplate},
            depth_record::{part_identity, PolicyPart, Selection, TemplateRule},
        };
        let registry = AgentRegistry::parse_unenforced(
            &fs::read_to_string(self.root.path().join(REGISTRY_RELATIVE_PATH)).unwrap(),
        )
        .unwrap();
        let mut policy = PolicyPart {
            identity: "0".repeat(64),
            selection: Selection {
                agent_id: "codex".into(),
                enrollment: registry
                    .find("codex")
                    .unwrap()
                    .support_depth
                    .as_ref()
                    .unwrap()
                    .enrollments
                    .as_ref()
                    .unwrap()[0]
                    .clone(),
            },
            rules_revision: "1".into(),
            base: ObligationId::BASE.to_vec(),
            templates: vec![TemplateRule {
                template: ObligationTemplate::Core,
                obligations: vec![ObligationId::Lc2, ObligationId::Out3],
            }],
            path_enabled: true,
            debt_baseline: vec![],
        };
        policy
            .base
            .sort_by_key(|id| serde_json::to_string(id).unwrap());
        policy.identity = part_identity(&policy).unwrap();
        let raw = json!({"schema_version":1,"event":{"lifecycle_path":"maintenance","request_commit":"abc","request_recorded_at":"2026-10-25T00:00:00Z","trigger_kind":"upstream_release_detected","opened_from":"fixture.yml"},"policy":policy,"obligations":null,"execution":null,"acceptance_entries":[]}).to_string();
        crate::depth_record::DepthRecord::parse(&raw, "codex", "0.156.1").unwrap();
        raw
    }
}
