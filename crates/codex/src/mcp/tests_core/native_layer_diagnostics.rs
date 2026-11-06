//! Producer-only layer diagnostics on the public `config_read` call.
use super::super::CodexAppServer;
use super::native_interactions::fixture_config;
use std::path::PathBuf;

#[derive(Clone, Default)]
struct Projections {
    events: std::sync::Arc<std::sync::Mutex<Vec<(String, String)>>>,
    // An ordinary all-target TRACE subscriber, rather than the producer's exclusive one.
    broad: bool,
}
impl tracing::Subscriber for Projections {
    fn enabled(&self, m: &tracing::Metadata<'_>) -> bool {
        self.broad || m.target() == crate::native_context::TARGET
    }
    fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::Id {
        tracing::Id::from_u64(1)
    }
    fn record(&self, _: &tracing::Id, _: &tracing::span::Record<'_>) {}
    fn record_follows_from(&self, _: &tracing::Id, _: &tracing::Id) {}
    fn enter(&self, _: &tracing::Id) {}
    fn exit(&self, _: &tracing::Id) {}
    fn event(&self, event: &tracing::Event<'_>) {
        #[derive(Default)]
        struct V(String, String);
        impl tracing::field::Visit for V {
            fn record_str(&mut self, f: &tracing::field::Field, v: &str) {
                match f.name() {
                    "kind" => self.0 = v.into(),
                    "projection" => self.1 = v.into(),
                    _ => (),
                }
            }
            fn record_debug(&mut self, _: &tracing::field::Field, _: &dyn std::fmt::Debug) {}
        }
        let mut v = V::default();
        event.record(&mut v);
        self.events.lock().unwrap().push((v.0, v.1));
    }
}

// Thread-local subscribers race tracing's global callsite interest under parallel tests,
// so the body runs alone in a re-executed process.
#[test]
fn private_owner_requests_layers_projects_them_and_returns_the_ordinary_reply() {
    for mode in ["exclusive", "broad", "late"] {
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "mcp::tests_core::native_layer_diagnostics::private_owner_layer_process",
                "--nocapture",
            ])
            .env("UAA_PRIVATE_OWNER_LAYER_PROCESS", mode)
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "{mode} subscriber");
    }
}

#[tokio::test]
async fn private_owner_layer_process() {
    let Ok(mode) = std::env::var("UAA_PRIVATE_OWNER_LAYER_PROCESS") else {
        return;
    };
    let broad = mode == "broad";
    // The exclusive subscriber is installed only after an ordinary spawn.
    let late = mode == "late";
    let (dir, mut config) = fixture_config(
        r#"
for line in sys.stdin:
 r=json.loads(line)
 inc=r['params'].get('includeLayers')
 res={'config':{'model':'gpt-5'},'origins':{'model':{'name':{'type':'user','file':'/role/home/config.toml'},'version':'v'}},'echo':inc}
 if inc: res['layers']=[{'name':{'type':'user','file':'/role/home/config.toml'},'version':'v','config':{'model':'sk-sentinel-layer'}},{'name':{'type':'system','file':'/etc/codex/config.toml'},'version':'w','config':{}}]
 send({'id':r['id'],'result':res})
"#,
    );
    config.code_home = Some(PathBuf::from("/role/home"));
    // Resolve the fixture interpreter directly rather than through a user shim manager.
    config.env = vec![("PATH".into(), "/usr/bin:/bin".into())];
    let projections = Projections {
        broad,
        ..Default::default()
    };
    let mut guard = (!late).then(|| tracing::subscriber::set_default(projections.clone()));
    let (server, _stream) = CodexAppServer::spawn_uninitialized_with_interactions(config)
        .await
        .unwrap();
    if late {
        guard = Some(tracing::subscriber::set_default(projections.clone()));
    }
    let call = server
        .config_read(PathBuf::from("/role/project"))
        .await
        .unwrap();
    let value = call.response.await.unwrap().unwrap();
    drop(guard);
    // Only the exclusive producer subscriber changes the request. Either way the caller gets
    // the reply native returns for a layer-free request.
    assert_eq!(value["echo"], !broad && !late);
    assert!(value.get("layers").is_none());
    assert_eq!(value["config"]["model"], "gpt-5");
    let events = projections.events.lock().unwrap().clone();
    let config = events
        .iter()
        .find(|(kind, _)| kind == "config")
        .map(|(_, p)| p.clone())
        .unwrap();
    assert!(!config.contains("sk-sentinel") && !config.contains("/role"));
    let projected: crate::native_context::Config = serde_json::from_str(&config).unwrap();
    if broad || late {
        // Passive diagnostics still run, but no layer stack is claimed.
        assert!(projected.layers.is_none());
        server.shutdown_and_wait().await.unwrap();
        return;
    }
    let layers = projected.layers.unwrap();
    assert!(layers.origins_enabled);
    assert!(layers.entries.iter().all(|l| l.role_matches && l.enabled));
    assert_eq!(layers.entries.len(), 2);
    server.shutdown_and_wait().await.unwrap();
    drop(dir);
}
