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
        for (step, interactive) in [("initial", false), ("resume-id", false), ("initial", true)] {
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
            if step == "resume-id" {
                request.extensions.insert(
                    "agent_api.session.resume.v1".into(),
                    json!({"selector":"id","id":"source"}),
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
    let script = r##"#!/usr/bin/python3
import sys,json,pathlib
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
  elif method=='config/read': result={'origins':{},'config':{'model':'sk-sentinel-model','model_provider':'sk-sentinel-provider','approval_policy':'sk-sentinel-policy','sandbox_mode':'sk-sentinel-sandbox','private_secret':'sk-sentinel-layer'}}
  elif method=='configRequirements/read': result={'requirements':None}
  else:
   print(json.dumps({'id':req['id'],'error':{'code':-1,'message':'sk-sentinel-error'}}),flush=True)
   continue
  print(json.dumps({'id':req['id'],'result':result}),flush=True)
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
    assert!(!managed.layers_complete && managed.requirements_empty);
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
