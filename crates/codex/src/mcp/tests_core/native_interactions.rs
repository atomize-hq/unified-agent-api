use super::super::{AppServerMessage, CodexAppServer, StdioServerConfig, ThreadStartParamsV2};
use serde_json::{json, Value};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, time::Duration};

pub(super) fn fixture_config(body: &str) -> (tempfile::TempDir, StdioServerConfig) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("server");
    fs::write(&path, format!("#!/usr/bin/env python3\nimport json,sys\ndef send(x):\n print(json.dumps(x),flush=True)\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    let config = StdioServerConfig {
        binary: path,
        code_home: None,
        current_dir: None,
        // Sibling binary-selection tests replace process PATH. Keep the fixture's
        // interpreter lookup tied to the build invocation, not that mutable state.
        env: vec![("PATH".into(), env!("PATH").into())],
        app_server_analytics_default_enabled: false,
        mirror_stdio: false,
        startup_timeout: Duration::from_secs(2),
    };
    (dir, config)
}

async fn fixture(
    body: &str,
) -> (
    tempfile::TempDir,
    CodexAppServer,
    super::super::AppServerMessageStream,
) {
    let (dir, config) = fixture_config(body);
    let (server, stream) = CodexAppServer::spawn_uninitialized_with_interactions(config)
        .await
        .unwrap();
    (dir, server, stream)
}

async fn next(stream: &mut super::super::AppServerMessageStream) -> AppServerMessage {
    tokio::time::timeout(Duration::from_secs(3), stream.recv())
        .await
        .unwrap()
        .unwrap()
        .unwrap()
}

#[tokio::test]
async fn ordered_requests_keep_native_ids_and_client_response_independent() {
    let (_dir, server, mut stream) = fixture(r#"
r=json.loads(sys.stdin.readline())
send({'method':'item/started','params':{'threadId':'t','turnId':'u','item':{'type':'fileChange','id':'i','changes':[{'path':'λ','kind':{'type':'add'},'diff':'+line\n'}]}}})
p={'threadId':'t','turnId':'u','itemId':'i','startedAtMs':0}
for i in [r['id'],str(r['id'])]:
 send({'id':i,'method':'item/fileChange/requestApproval','params':p})
send({'id':r['id'],'result':{'thread':{'id':'t'},'approvalPolicy':'on-request'}})
for _ in range(2):
 reply=json.loads(sys.stdin.readline())
 send({'method':'test/reply','params':reply})
for line in sys.stdin: pass
"#).await;
    let handle = server
        .thread_start_v2(ThreadStartParamsV2::default())
        .await
        .unwrap();
    assert_eq!(
        handle.response.await.unwrap().unwrap()["approvalPolicy"],
        "on-request"
    );
    let AppServerMessage::Notification { method, params } = next(&mut stream).await else {
        panic!("notification first")
    };
    assert_eq!(method, "item/started");
    assert_eq!(params["item"]["changes"][0]["path"], "λ");
    let AppServerMessage::Request(a) = next(&mut stream).await else {
        panic!("request")
    };
    let AppServerMessage::Request(b) = next(&mut stream).await else {
        panic!("request")
    };
    a.reply(json!({"decision":"acceptForSession"})).unwrap();
    b.reply(json!({"decision":"decline"})).unwrap();
    for (expected_id, decision) in [
        (json!(handle.request_id), "acceptForSession"),
        (json!(handle.request_id.to_string()), "decline"),
    ] {
        let AppServerMessage::Notification { params, .. } = next(&mut stream).await else {
            panic!("reply echo")
        };
        assert_eq!(params["id"], expected_id);
        assert_eq!(params["result"]["decision"], decision);
    }
    server.shutdown_and_wait().await.unwrap();
}

#[tokio::test]
async fn resolved_request_invalidates_before_late_reply_and_wakes_waiter() {
    let (_dir, server, mut stream)=fixture(r#"
send({'id':'private','method':'mcpServer/elicitation/request','params':{'threadId':'t','serverName':'s','mode':'url','message':'login','url':'https://example.test','elicitationId':'e','turnId':None}})
send({'method':'serverRequest/resolved','params':{'threadId':'t','requestId':'private'}})
for line in sys.stdin: pass
"#).await;
    let AppServerMessage::Request(mut r) = next(&mut stream).await else {
        panic!("request")
    };
    let _ = next(&mut stream).await;
    r.cancelled().await;
    assert!(!r.is_active());
    assert!(r
        .reply(json!({"action":"accept","content":{"proof":"x"}}))
        .is_err());
    server.shutdown_and_wait().await.unwrap();
}

#[tokio::test]
async fn new_thread_params_preserve_granular_json_and_config() {
    let policy =
        json!({"granular":{"sandbox_approval":false,"rules":true,"mcp_elicitations":false}});
    let p = ThreadStartParamsV2 {
        approval_policy: Some(policy.clone()),
        cwd: Some(PathBuf::from("/tmp/cwd")),
        config: std::collections::BTreeMap::from([(
            "model_reasoning_effort".into(),
            json!("high"),
        )]),
        ..Default::default()
    };
    let v: Value = serde_json::to_value(p).unwrap();
    assert_eq!(v["approvalPolicy"], policy);
    assert_eq!(v["cwd"], "/tmp/cwd");
    assert_eq!(v["config"]["model_reasoning_effort"], "high");
}

#[tokio::test]
async fn sixteen_pending_requests_keep_reader_live_and_seventeenth_fails_closed() {
    let (_dir, server, mut stream) = fixture(
        r#"
p={'threadId':'t','turnId':'u','itemId':'i','startedAtMs':0}
for i in range(16): send({'id':i,'method':'item/fileChange/requestApproval','params':p})
send({'method':'barrier','params':{}})
r=json.loads(sys.stdin.readline())
send({'id':16,'method':'item/fileChange/requestApproval','params':p})
for line in sys.stdin: pass
"#,
    )
    .await;
    let mut requests = Vec::new();
    for _ in 0..16 {
        let AppServerMessage::Request(r) = next(&mut stream).await else {
            panic!("request")
        };
        requests.push(r);
    }
    let AppServerMessage::Notification { method, .. } = next(&mut stream).await else {
        panic!("reader responsive")
    };
    assert_eq!(method, "barrier");
    // A client RPC is a deterministic barrier; it does not consume a server request slot.
    let _call = server
        .thread_start_v2(ThreadStartParamsV2::default())
        .await
        .unwrap();
    assert!(tokio::time::timeout(Duration::from_secs(3), stream.recv())
        .await
        .unwrap()
        .unwrap()
        .is_err());
    for r in requests {
        assert!(!r.is_active());
        assert!(r.reply(json!({"decision":"accept"})).is_err());
    }
    server.shutdown_and_wait().await.unwrap();
}

#[tokio::test]
async fn app_frame_exact_bound_is_delivered_and_above_bound_is_explicit_error() {
    for extra in [0, 1] {
        let body = format!(
            r#"
v={{'method':'bounded','params':{{'padding':''}}}}
s=json.dumps(v,separators=(',',':'))
v['params']['padding']='x'*(1048576-len(s)+{extra})
print(json.dumps(v,separators=(',',':')),flush=True)
for line in sys.stdin: pass
"#
        );
        let (_dir, server, mut stream) = fixture(&body).await;
        let result = tokio::time::timeout(Duration::from_secs(3), stream.recv())
            .await
            .unwrap()
            .unwrap();
        if extra == 0 {
            let AppServerMessage::Notification { method, params } = result.unwrap() else {
                panic!("notification")
            };
            assert_eq!(method, "bounded");
            assert!(params["padding"].as_str().unwrap().len() > 1_000_000);
        } else {
            assert!(result.is_err());
        }
        server.shutdown_and_wait().await.unwrap();
    }
}

#[tokio::test]
async fn notifications_apply_backpressure_without_losing_order() {
    let (_dir, server, mut stream) = fixture(
        r#"
for i in range(1000): send({'method':'sequence','params':{'index':i}})
for line in sys.stdin: pass
"#,
    )
    .await;
    for i in 0..1000 {
        let AppServerMessage::Notification { params, .. } = next(&mut stream).await else {
            panic!("notification")
        };
        assert_eq!(params["index"], i);
    }
    server.shutdown_and_wait().await.unwrap();
}

#[tokio::test]
async fn owner_invalidation_and_shutdown_disable_replies() {
    for shutdown in [false, true] {
        let (_dir,server,mut stream)=fixture(r#"
send({'id':-7,'method':'item/fileChange/requestApproval','params':{'threadId':'t','turnId':'u','itemId':'i','startedAtMs':0}})
for line in sys.stdin: pass
"#).await;
        let AppServerMessage::Request(mut request) = next(&mut stream).await else {
            panic!("request")
        };
        if shutdown {
            server.shutdown_and_wait().await.unwrap();
        } else {
            request.invalidate();
        }
        request.cancelled().await;
        assert!(request.reply(json!({"decision":"accept"})).is_err());
        if !shutdown {
            server.shutdown_and_wait().await.unwrap();
        }
    }
}

#[tokio::test]
async fn unknown_or_malformed_request_is_redacted_explicit_failure() {
    for (method, params) in [
        ("unknown/request", json!({"secret":"do not echo"})),
        (
            "item/fileChange/requestApproval",
            json!({"threadId":"secret"}),
        ),
    ] {
        let body = format!(
            "send({})\nfor line in sys.stdin: pass",
            json!({"id":"x","method":method,"params":params})
        );
        let (_dir, server, mut stream) = fixture(&body).await;
        let error = tokio::time::timeout(Duration::from_secs(3), stream.recv())
            .await
            .unwrap()
            .unwrap()
            .err()
            .unwrap();
        assert!(!error.to_string().contains("secret"));
        server.shutdown_and_wait().await.unwrap();
    }
}

#[tokio::test]
async fn eof_preserves_prior_notification_order_and_invalidates_pending_request() {
    let (_dir,server,mut stream)=fixture(r#"
send({'id':3,'method':'item/fileChange/requestApproval','params':{'threadId':'t','turnId':'u','itemId':'i','startedAtMs':0}})
send({'method':'final','params':{'status':'completed'}})
"#).await;
    let AppServerMessage::Request(mut r) = next(&mut stream).await else {
        panic!("request")
    };
    let AppServerMessage::Notification { method, .. } = next(&mut stream).await else {
        panic!("ordered notification")
    };
    assert_eq!(method, "final");
    assert!(stream.recv().await.unwrap().is_err());
    assert!(stream.recv().await.is_none());
    r.cancelled().await;
    assert!(r.reply(json!({"decision":"accept"})).is_err());
    server.shutdown_and_wait().await.unwrap();
}

#[tokio::test]
async fn legacy_constructor_still_refuses_native_prompt_without_consumer() {
    let (_dir, config) = fixture_config(
        r#"
r=json.loads(sys.stdin.readline())
send({'id':r['id'],'method':'item/fileChange/requestApproval','params':{'threadId':'t','turnId':'u','itemId':'i','startedAtMs':0}})
reply=json.loads(sys.stdin.readline())
assert reply['error']['code']==-32601 and 'result' not in reply
for line in sys.stdin: pass
"#,
    );
    let server = CodexAppServer::spawn_uninitialized(config).await.unwrap();
    let call = server
        .thread_start(super::super::ThreadStartParams {
            thread_id: None,
            metadata: json!({}),
        })
        .await
        .unwrap();
    assert_eq!(
        call.response.await.unwrap().unwrap_err().to_string(),
        "server reported an error: approval required"
    );
    server.shutdown_and_wait().await.unwrap();
}

#[tokio::test]
async fn all_additive_calls_keep_wire_method_full_response_and_granular_policy() {
    use super::super::{ThreadForkParamsV2, ThreadResumeParamsV2, TurnStartParamsV3, UserInputV2};
    let (_dir,server,_stream)=fixture(r#"
for line in sys.stdin:
 r=json.loads(line)
 send({'id':r['id'],'result':{'method':r['method'],'params':r['params'],'effectiveReviewer':'configured'}})
"#).await;
    let policy = json!({"granular":{"sandbox_approval":false,"rules":false,"mcp_elicitations":true,"request_permissions":false,"skill_approval":true}});
    let mut handles = Vec::new();
    handles.push((
        "thread/start",
        server
            .thread_start_v2(ThreadStartParamsV2 {
                approval_policy: Some(policy.clone()),
                ..Default::default()
            })
            .await
            .unwrap(),
    ));
    handles.push((
        "thread/resume",
        server
            .thread_resume_v2(ThreadResumeParamsV2 {
                thread_id: "t".into(),
                approval_policy: Some(policy.clone()),
                ..Default::default()
            })
            .await
            .unwrap(),
    ));
    handles.push((
        "thread/fork",
        server
            .thread_fork_v2(ThreadForkParamsV2 {
                thread_id: "t".into(),
                approval_policy: Some(policy.clone()),
                ..Default::default()
            })
            .await
            .unwrap(),
    ));
    handles.push((
        "turn/start",
        server
            .turn_start_v3(TurnStartParamsV3 {
                thread_id: "t".into(),
                input: vec![UserInputV2::text("hello")],
                approval_policy: Some(policy.clone()),
                ..Default::default()
            })
            .await
            .unwrap(),
    ));
    for (method, call) in handles {
        let result = call.response.await.unwrap().unwrap();
        assert_eq!(result["method"], method);
        assert_eq!(result["params"]["approvalPolicy"], policy);
        assert_eq!(result["effectiveReviewer"], "configured");
        if method != "thread/start" {
            assert_eq!(result["params"]["threadId"], "t");
        }
    }
    server.shutdown_and_wait().await.unwrap();
}

#[tokio::test]
async fn resolution_with_wrong_thread_does_not_invalidate_request() {
    let (_dir,server,mut stream)=fixture(r#"
send({'id':'a','method':'item/fileChange/requestApproval','params':{'threadId':'t','turnId':'u','itemId':'i','startedAtMs':0}})
send({'method':'serverRequest/resolved','params':{'threadId':'wrong','requestId':'a'}})
for line in sys.stdin:
 send({'method':'reply','params':json.loads(line)})
"#).await;
    let AppServerMessage::Request(request) = next(&mut stream).await else {
        panic!("request")
    };
    let _ = next(&mut stream).await;
    assert!(request.is_active());
    request.reply(json!({"decision":"decline"})).unwrap();
    let AppServerMessage::Notification { params, .. } = next(&mut stream).await else {
        panic!("reply")
    };
    assert_eq!(params["result"]["decision"], "decline");
    server.shutdown_and_wait().await.unwrap();
}

#[tokio::test]
async fn shared_adapter_retention_limits_native_pending_delivery() {
    let (_dir,server,mut stream)=fixture(r#"
sys.stdin.readline()
send({'id':'budget','method':'item/fileChange/requestApproval','params':{'threadId':'t','turnId':'u','itemId':'i','startedAtMs':0}})
for line in sys.stdin: pass
"#).await;
    let lease = stream.retain_json(16 * 1024 * 1024).unwrap();
    assert!(stream.retain_json(1).is_err());
    let _call = server
        .thread_start_v2(ThreadStartParamsV2::default())
        .await
        .unwrap();
    assert!(tokio::time::timeout(Duration::from_secs(3), stream.recv())
        .await
        .unwrap()
        .unwrap()
        .is_err());
    drop(lease);
    server.shutdown_and_wait().await.unwrap();
}

#[tokio::test]
async fn invalid_reply_returns_native_error_without_serializing_host_decision() {
    let (_dir,server,mut stream)=fixture(r#"
send({'id':'invalid','method':'item/fileChange/requestApproval','params':{'threadId':'t','turnId':'u','itemId':'i','startedAtMs':0}})
reply=json.loads(sys.stdin.readline())
send({'method':'reply','params':reply})
for line in sys.stdin: pass
"#).await;
    let AppServerMessage::Request(r) = next(&mut stream).await else {
        panic!("request")
    };
    let error = r
        .reply(json!({"decision":"approve","private":"host secret"}))
        .unwrap_err();
    assert!(!error.to_string().contains("secret"));
    let AppServerMessage::Notification { params, .. } = next(&mut stream).await else {
        panic!("reply")
    };
    assert_eq!(params["id"], "invalid");
    assert_eq!(params["error"]["code"], -32601);
    assert!(params.get("result").is_none());
    server.shutdown_and_wait().await.unwrap();
}

#[tokio::test]
async fn startup_config_and_read_methods_preserve_native_arguments_and_full_results() {
    let (_dir, config) = fixture_config(
        r#"
send({'method':'argv','params':sys.argv[1:]})
for line in sys.stdin:
 r=json.loads(line)
 send({'id':r['id'],'result':{'method':r['method'],'params':r['params'],'config':{'permissions':{'kept':True}}}})
"#,
    );
    let (server, mut stream) = CodexAppServer::spawn_uninitialized_with_interactions_and_config(
        config,
        std::collections::BTreeMap::from([("sandbox_mode".into(), json!("workspace-write"))]),
    )
    .await
    .unwrap();
    let AppServerMessage::Notification { params, .. } = next(&mut stream).await else {
        panic!("argv")
    };
    assert_eq!(
        params,
        json!(["--config", "sandbox_mode=\"workspace-write\"", "app-server"])
    );
    let call = server.config_read(PathBuf::from("/tmp/λ")).await.unwrap();
    let value = call.response.await.unwrap().unwrap();
    assert_eq!(value["method"], "config/read");
    assert_eq!(
        value["params"],
        json!({"cwd":"/tmp/λ","includeLayers":false})
    );
    assert_eq!(value["config"]["permissions"]["kept"], true);
    let call = server.config_requirements_read().await.unwrap();
    let value = call.response.await.unwrap().unwrap();
    assert_eq!(value["method"], "configRequirements/read");
    assert_eq!(value["params"], json!({}));
    let value = serde_json::to_value(super::super::ThreadResumeParamsV2 {
        runtime_workspace_roots: Some(vec![PathBuf::from("/ambient"), PathBuf::from("/requested")]),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        value["runtimeWorkspaceRoots"],
        json!(["/ambient", "/requested"])
    );
    server.shutdown_and_wait().await.unwrap();
}

#[tokio::test]
async fn outgoing_native_reply_bound_covers_complete_wire_frame_and_string_id() {
    const LIMIT: usize = 1024 * 1024;
    let id = "native-λ-".repeat(12);
    for extra in [0, 1] {
        let native = json!({"id":id,"method":"mcpServer/elicitation/request","params":{"threadId":"t","serverName":"s","mode":"url","message":"login","url":"https://example.test","elicitationId":"e"}});
        let body = format!(
            r#"
send({native})
line=sys.stdin.buffer.readline()
reply=json.loads(line)
send({{'method':'wire/reply','params':{{'bytes':len(line)-1,'id':reply['id'],'action':reply.get('result',{{}}).get('action'),'error':reply.get('error',{{}}).get('code')}}}})
for line in sys.stdin: pass
"#
        );
        let (_dir, server, mut stream) = fixture(&body).await;
        let AppServerMessage::Request(request) = next(&mut stream).await else {
            panic!("request")
        };
        let mut result = json!({"action":"accept","content":{"padding":""}});
        let base = json!({"jsonrpc":"2.0","id":id,"result":result})
            .to_string()
            .len();
        result["content"]["padding"] = json!("x".repeat(LIMIT - base + extra));
        assert!(
            result.to_string().len() < LIMIT,
            "both result bodies alone fit; the wire envelope determines acceptance"
        );
        assert_eq!(
            json!({"jsonrpc":"2.0","id":id,"result":result})
                .to_string()
                .len(),
            LIMIT + extra
        );
        let reply = request.reply(result);
        if extra == 0 {
            reply.expect("exactly 1MiB complete frame must be accepted");
        } else {
            assert!(
                reply.is_err(),
                "one byte above complete frame bound must be refused"
            );
        }
        let AppServerMessage::Notification { params, .. } = next(&mut stream).await else {
            panic!("wire response")
        };
        assert_eq!(params["id"], id);
        if extra == 0 {
            assert_eq!(params["bytes"], LIMIT);
            assert_eq!(params["action"], "accept");
            assert!(params["error"].is_null());
        } else {
            assert!(params["bytes"].as_u64().unwrap() < LIMIT as u64);
            assert_eq!(params["error"], -32601);
            assert!(params["action"].is_null());
        }
        server.shutdown_and_wait().await.unwrap();
    }
}

#[tokio::test]
async fn native_fixture_preserves_python_lookup_when_siblings_replace_ambient_path() {
    const CHILD: &str = "UAA_NATIVE_ISOLATED_PATH";
    if std::env::var_os(CHILD).is_some() {
        let (_dir, server, mut stream) =
            fixture("send({'method':'alive','params':{}})\nfor line in sys.stdin: pass\n").await;
        let AppServerMessage::Notification { method, .. } = next(&mut stream).await else {
            panic!("fixture startup notification")
        };
        assert_eq!(method, "alive");
        server.shutdown_and_wait().await.unwrap();
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "mcp::tests_core::native_interactions::native_fixture_preserves_python_lookup_when_siblings_replace_ambient_path", "--nocapture"])
        .env(CHILD, "1")
        .env("PATH", "/no/python/on/ambient/path")
        .output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn queued_result_shared_lease_releases_after_the_complete_frame_is_written() {
    let native = json!({"id":"leased","method":"mcpServer/elicitation/request","params":{"threadId":"t","serverName":"s","mode":"url","message":"q","url":"https://example.test","elicitationId":"e"}});
    let body = format!(
        r#"
send({native})
r=json.loads(sys.stdin.readline())
assert len(r['result']['content']['opaque']) == 100000
send({{'method':'written','params':{{}}}})
for line in sys.stdin: pass
"#
    );
    let (_dir, server, mut stream) = fixture(&body).await;
    let AppServerMessage::Request(request) = next(&mut stream).await else {
        panic!("request")
    };
    let result = json!({"action":"accept","content":{"opaque":"界".repeat(100_000)}});
    let frame = json!({"jsonrpc":"2.0","id":"leased","result":result})
        .to_string()
        .len();
    let retained = native.to_string().len() + frame;
    let _held = stream.retain_json(16 * 1024 * 1024 - retained).unwrap();
    request.reply(result).unwrap();
    assert!(
        stream.retain_json(1).is_err(),
        "the queued frame fills the shared budget before writer polling"
    );
    let AppServerMessage::Notification { method, .. } = next(&mut stream).await else {
        panic!("written notification")
    };
    assert_eq!(method, "written");
    assert!(
        stream.retain_json(retained).is_ok(),
        "actual complete write releases response and request leases"
    );
    server.shutdown_and_wait().await.unwrap();
}
