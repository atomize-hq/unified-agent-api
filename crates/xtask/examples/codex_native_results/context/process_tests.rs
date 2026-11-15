//! Foreground fake-native children exercise the real adapter and process-global subscriber.
use super::*;
use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperBackend, AgentWrapperRunRequest,
};
use futures_util::StreamExt;
use serde_json::json;
use std::{
    fs,
    process::{Command, Stdio},
    time::Duration,
};
#[test]
fn context_process_fixture() {
    let Some(root) = std::env::var_os("UAA_CONTEXT_TEST_ROOT").map(PathBuf::from) else {
        return;
    };
    let mode = std::env::var("UAA_CONTEXT_TEST_MODE").unwrap();
    // Fixture routing is not part of the native environment.
    std::env::remove_var("UAA_CONTEXT_TEST_ROOT");
    std::env::remove_var("UAA_CONTEXT_TEST_MODE");
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        if mode == "refuse" {
            let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap();
            let parent = workspace.join("target/native-guard-proof");
            fs::create_dir_all(&parent).unwrap();
            let leaf = format!("target/native-guard-proof/{}", std::process::id());
            let args = super::super::Args {
                workspace: None,
                live: true,
                context_guard: true,
                binary: root.join("fake"),
                working_dir: root.join("work"),
                codex_home: Some(root.join("home")),
                model: None,
                expected_version: None,
                variant: None,
                output_dir: leaf,
            };
            let result = super::super::collect(&workspace, args).await;
            assert_eq!(result.unwrap_err(), "native context refused: environment");
            return;
        }
        let capture = Capture::install().unwrap();
        let backend = CodexBackend::new(CodexBackendConfig {
            binary: Some(root.join("fake")),
            codex_home: Some(root.join("home")),
            default_working_dir: Some(root.join("work")),
            ..Default::default()
        });
        let mut reports = vec![];
        for (step, interactive) in if mode == "full" {
            vec![
                ("initial", true),
                ("resume-last", true),
                ("resume-id", true),
                ("fork-last", false),
                ("fork-id", false),
            ]
        } else {
            vec![("initial", false), ("resume-id", false), ("initial", true)]
        } {
            let case = CaseIdentity {
                name: format!("test/{interactive}/{step}"),
                executable_sha256: "a".repeat(64),
            };
            capture.begin(case, route(interactive, step), true);
            let mut request = AgentWrapperRunRequest {
                prompt: "synthetic".into(),
                timeout: Some(Duration::from_secs(3)),
                ..Default::default()
            };
            if interactive {
                request
                    .extensions
                    .insert("agent_api.exec.non_interactive".into(), json!(false));
            }
            if step.starts_with("resume") || step.starts_with("fork") {
                request.extensions.insert(
                    if step.starts_with("resume") {
                        "agent_api.session.resume.v1"
                    } else {
                        "agent_api.session.fork.v1"
                    }
                    .into(),
                    if step.ends_with("-last") {
                        json!({"selector":"last"})
                    } else {
                        json!({"selector":"id","id":"source"})
                    },
                );
            }
            let run = backend.run(request).await.expect("valid fixture request");
            let (_, _) = tokio::join!(run.events.collect::<Vec<_>>(), run.completion);
            reports.push(capture.finish(true));
        }
        fs::write(
            root.join("observed.json"),
            serde_json::to_vec(&reports).unwrap(),
        )
        .unwrap();
    });
}
#[cfg(unix)]
fn fixture(mode: &str) -> tempfile::TempDir {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir(tmp.path().join("home")).unwrap();
    fs::create_dir(tmp.path().join("work")).unwrap();
    fs::write(
        tmp.path().join("home/auth.json"),
        "sk-synthetic-auth-never-read",
    )
    .unwrap();
    fs::write(tmp.path().join("mode"), mode).unwrap();
    let script = r##"#!/usr/bin/python3
import sys,json,pathlib,os
p=pathlib.Path(__file__).parent
(p/'invoked').touch()
if 'app-server' not in sys.argv:
 print(json.dumps({'type':'thread.started','thread_id':'aaaaaaaa-1111-4111-8111-111111111111'}))
 print(json.dumps({'type':'turn.completed','usage':{'input_tokens':1,'cached_input_tokens':0,'output_tokens':1}}))
else:
 for line in sys.stdin:
  req=json.loads(line)
  if 'id' not in req: continue
  method=req['method']
  if method=='initialize': result={'userAgent':'fixture'}
  elif method=='config/read':
   home=os.environ['CODEX_HOME'] if (p/'mode').read_text()!='foreign' else '/sk-native-home'
   user={'type':'user','file':home+'/config.toml'}
   result={'origins':{'model':{'name':user,'version':'sk-sentinel-version'}},'config':{'model':'sk-sentinel-model','model_provider':'sk-sentinel-provider','approval_policy':'sk-sentinel-policy','sandbox_mode':'sk-sentinel-sandbox','private_secret':'sk-sentinel-layer'}}
   if req['params'].get('includeLayers'):
    result['layers']=[{'name':user,'version':'sk-sentinel-version','config':{'model':'sk-sentinel-user-layer'}},{'name':{'type':'system','file':'/etc/codex/config.toml'},'version':'sk-sentinel-system','config':{}}]
    if (p/'mode').read_text()=='foreign': result['layers'].insert(0,{'name':{'type':'enterpriseManaged','id':'sk-native-id','name':'sk-native-name'},'version':'sk-native-v','config':{'model':'sk-native-cloud'}})
  elif method=='thread/list': result={'data':[{'id':'source','cwd':req['params']['cwd'],'createdAt':1,'updatedAt':2}],'nextCursor':None}
  elif method in ['thread/start','thread/resume','thread/fork']:
   result={'thread':{'id':'aaaaaaaa-1111-4111-8111-111111111111'},'model':'gpt-5','modelProvider':'openai','serviceTier':None,'approvalPolicy':'on-request','approvalsReviewer':'user','sandbox':{'type':'workspaceWrite'},'cwd':req['params']['cwd'],'runtimeWorkspaceRoots':req['params'].get('runtimeWorkspaceRoots',[]),'instructionSources':[]}
   if (p/'mode').read_text()=='foreign': result.update({'model':'sk-native-model','modelProvider':'sk-native-provider','serviceTier':'sk-native-tier','approvalPolicy':'sk-native-policy','approvalsReviewer':'sk-native-reviewer','sandbox':{'type':'sk-native-sandbox'},'cwd':'/sk-native-cwd','runtimeWorkspaceRoots':['/sk-native-root'],'instructionSources':['/sk-native-instruction']})
   if (p/'mode').read_text()=='missing': result={'thread':result['thread']}
  elif method=='turn/start': result={'turn':{'id':'native-turn','status':'inProgress','items':[]}}
  elif method=='configRequirements/read': result={'requirements':None}
  else:
   print(json.dumps({'id':req['id'],'error':{'code':-1,'message':'sk-sentinel-error'}}),flush=True)
   continue
  print(json.dumps({'id':req['id'],'result':result}),flush=True)
  if method=='turn/start': print(json.dumps({'method':'turn/completed','params':{'threadId':'aaaaaaaa-1111-4111-8111-111111111111','turn':{'id':'native-turn','status':'completed','items':[]}}}),flush=True)
"##;
    fs::write(tmp.path().join("fake"), script).unwrap();
    fs::set_permissions(tmp.path().join("fake"), fs::Permissions::from_mode(0o700)).unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "context::process_tests::context_process_fixture",
            "--nocapture",
        ])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", tmp.path())
        .env("OPENAI_API_KEY", "sk-synthetic-environment-secret")
        .env("UAA_CONTEXT_TEST_ROOT", tmp.path())
        .env("UAA_CONTEXT_TEST_MODE", mode)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if mode == "refuse" {
        command.env("UNREVIEWED_NATIVE_OVERRIDE", "sk-sentinel-unknown-channel");
    }
    assert!(
        command.status().unwrap().success(),
        "foreground context fixture failed"
    );
    tmp
}
#[cfg(unix)]
#[test]
fn actual_exec_resume_and_managed_children_emit_only_finite_observations() {
    let tmp = fixture("observe");
    let raw = fs::read_to_string(tmp.path().join("observed.json")).unwrap();
    assert!(
        !raw.contains("sentinel")
            && !raw.contains("synthetic-auth")
            && !raw.contains("environment-secret")
    );
    let reports: Vec<CaseContext> = serde_json::from_str(&raw).unwrap();
    assert_eq!(reports.len(), 3);
    for (c, expected) in
        reports
            .iter()
            .zip([Route::InitialExec, Route::ResumeExec, Route::AppServer])
    {
        assert_eq!(c.route, expected);
        assert!(c.diagnostics_valid);
        assert_eq!(c.launches.len(), 2);
        assert!(!c.launches[0].spawned && c.launches[1].spawned);
        assert!(c.bindings.is_empty());
        assert_eq!(c.missing.len(), context::Fact::ALL.len());
    }
    assert!(reports[0].config.is_none() && reports[1].config.is_none());
    let managed = reports[2].config.as_ref().unwrap();
    assert!(
        !managed.model_gpt5
            && !managed.provider_openai
            && managed.approval.is_none()
            && managed.sandbox.is_none()
    );
    assert!(managed.requirements_empty);
    // The actual managed child was asked for its stack; only finite classes/roles survive.
    let layers = managed
        .layers
        .as_ref()
        .expect("requested native layer stack");
    assert_eq!(
        layers.entries,
        [context::LayerClass::User, context::LayerClass::System].map(|class| context::Layer {
            class,
            enabled: true,
            empty: class == context::LayerClass::System,
            role_matches: true,
        })
    );
    assert_eq!(layers.origin_classes, [context::LayerClass::User]);
    assert!(layers.origins_enabled);
    assert_eq!(
        fs::read_to_string(tmp.path().join("home/auth.json")).unwrap(),
        "sk-synthetic-auth-never-read"
    );
}
#[cfg(unix)]
#[test]
fn producer_guard_refuses_unknown_ambient_channel_before_version_probe() {
    let tmp = fixture("refuse");
    assert!(!tmp.path().join("invoked").exists());
}

#[cfg(unix)]
#[test]
fn actual_managed_and_fork_replies_are_partial_observations_not_complete_context() {
    let tmp = fixture("full");
    let raw = fs::read(tmp.path().join("observed.json")).unwrap();
    let reports: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(reports.as_array().unwrap().len(), 5);
    for c in reports.as_array().unwrap() {
        assert_eq!(c["thread_settings"]["model_gpt5"], true);
        assert_eq!(c["thread_settings"]["provider_openai"], true);
        assert_eq!(c["thread_settings"]["instructions_empty"], true);
        assert_eq!(c["thread_settings"]["cwd_matches"], true);
        assert_eq!(c["turn_accepted"], true);
        assert_eq!(c["missing"].as_array().unwrap().len(), 6);
        assert!(c["bindings"].as_array().unwrap().is_empty());
    }
    assert!(!String::from_utf8(raw).unwrap().contains("sk-sentinel"));
}

#[cfg(unix)]
#[test]
fn unknown_or_missing_native_settings_never_become_empty_defaults_or_leak() {
    for mode in ["foreign", "missing"] {
        let tmp = fixture(mode);
        let bytes = fs::read(tmp.path().join("observed.json")).unwrap();
        let reports: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let thread = &reports[2]["thread_settings"];
        assert_eq!(thread["model_gpt5"], false);
        assert_eq!(thread["provider_openai"], false);
        assert_eq!(thread["service_tier_default"], false);
        assert!(thread["approval"].is_null() && thread["sandbox_mode"].is_null());
        assert_eq!(thread["reviewer_user"], false);
        assert_eq!(thread["cwd_matches"], false);
        assert_eq!(thread["runtime_roots_match"], false);
        assert_eq!(thread["instructions_empty"], false);
        assert_eq!(reports[2]["missing"].as_array().unwrap().len(), 6);
        if mode == "foreign" {
            let layers = &reports[2]["config"]["layers"];
            assert_eq!(layers["entries"][0]["class"], "enterprise_managed");
            assert_eq!(layers["entries"][1]["class"], "user");
            assert_eq!(layers["entries"][1]["role_matches"], false);
        }
        assert!(!String::from_utf8(bytes).unwrap().contains("sk-"));
    }
}
