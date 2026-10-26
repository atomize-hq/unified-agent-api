#![cfg(all(feature = "codex", unix))]
//! DEF-01/AS-2 and DEF-02/AS-3: public controls must cover owned startup RPCs.
use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperGateway, AgentWrapperKind, AgentWrapperRunRequest,
};
use futures_util::{FutureExt, StreamExt};
use serde_json::json;
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    time::Duration,
};
use tempfile::tempdir;

async fn setup_lifecycle(stage: &str, cancelled: bool) {
    let dir = tempdir().unwrap();
    let socket = dir.path().join("gate");
    let exit = dir.path().join("exit");
    let listener = UnixListener::bind(&socket).unwrap();
    let stage_ready = tokio::task::spawn_blocking(move || {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut stream = BufReader::new(stream);
        let mut line = String::new();
        stream.read_line(&mut line).unwrap();
        (stream, line)
    });
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(env!("CARGO_BIN_EXE_fake_codex_app_server_jsonrpc_agent_api").into()),
        env: [
            (
                "FAKE_CODEX_APP_SERVER_SCENARIO".into(),
                "pinned_setup".into(),
            ),
            ("FAKE_CODEX_PAUSE_AT".into(), stage.into()),
            ("FAKE_CODEX_GATE".into(), socket.display().to_string()),
            (
                "FAKE_CODEX_APP_SERVER_EXIT_MARKER".into(),
                exit.display().to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });
    let mut gateway = AgentWrapperGateway::new();
    gateway.register(std::sync::Arc::new(backend)).unwrap();
    let run = gateway
        .run_control(
            &AgentWrapperKind::new("codex").unwrap(),
            AgentWrapperRunRequest {
                prompt: "hello".into(),
                timeout: Some(if cancelled {
                    Duration::ZERO
                } else {
                    Duration::from_secs(1)
                }),
                extensions: [(
                    "agent_api.session.fork.v1".into(),
                    if stage == "thread/list" {
                        json!({"selector":"last"})
                    } else {
                        json!({"selector":"id","id":"source"})
                    },
                )]
                .into_iter()
                .collect(),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let (mut stream, stage_line) =
        match tokio::time::timeout(Duration::from_secs(3), stage_ready).await {
            Ok(result) => result.unwrap(),
            Err(_) => {
                run.cancel.cancel();
                // Wake the blocking accept on a failed spawn so a failed test cannot strand it.
                if let Ok(mut stream) = UnixStream::connect(&socket) {
                    let _ = stream.write_all(b"aborted\n");
                }
                panic!("fixture did not reach {stage}");
            }
        };
    assert!(stage_line.starts_with("stage "));
    #[cfg(target_os = "linux")]
    let pid: u32 = stage_line
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    if cancelled {
        run.cancel.cancel();
    }
    let mut completion = run.handle.completion;
    let events = tokio::spawn(run.handle.events.collect::<Vec<_>>());
    let waiting_for_eof = tokio::task::spawn_blocking(move || {
        stream
            .get_ref()
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut line = String::new();
        let read = stream.read_line(&mut line);
        (stream, line, read)
    });
    let (mut stream, line, read) = waiting_for_eof.await.unwrap();
    // While the fixture is held after EOF, process exit has deliberately not happened.
    let premature = (&mut completion).now_or_never();
    let completed_before_exit = premature.is_some();
    let _ = stream.get_mut().write_all(b"x");
    if read.is_err() || line != "eof\n" {
        run.cancel.cancel();
    } // Clean up old broken timeout paths.
    let completion = match premature {
        Some(result) => result,
        None => tokio::time::timeout(Duration::from_secs(5), completion)
            .await
            .unwrap(),
    };
    let _ = tokio::time::timeout(Duration::from_secs(3), events)
        .await
        .unwrap()
        .unwrap();
    assert!(
        read.is_ok() && line == "eof\n",
        "{stage}: did not close stdin and await child: {read:?}, {line:?}"
    );
    assert!(
        !completed_before_exit,
        "{stage}: completion preceded child exit"
    );
    assert!(
        exit.exists(),
        "{stage}: child did not finish its EOF exit path"
    );
    #[cfg(target_os = "linux")]
    assert!(
        !std::path::Path::new(&format!("/proc/{pid}")).exists(),
        "child still exists after completion"
    );
    let expected = if cancelled {
        "backend error: cancelled"
    } else {
        "backend error: codex backend error: timeout (details redacted when unsafe)"
    };
    assert_eq!(completion.unwrap_err().to_string(), expected);
}

#[tokio::test]
async fn cancel_during_initialize_waits_for_child_exit() {
    setup_lifecycle("initialize", true).await;
}
#[tokio::test]
async fn cancel_during_list_waits_for_child_exit() {
    setup_lifecycle("thread/list", true).await;
}
#[tokio::test]
async fn cancel_during_fork_waits_for_child_exit() {
    setup_lifecycle("thread/fork", true).await;
}
#[tokio::test]
async fn unanswered_initialize_obeys_run_timeout() {
    setup_lifecycle("initialize", false).await;
}
#[tokio::test]
async fn unanswered_list_obeys_run_timeout() {
    setup_lifecycle("thread/list", false).await;
}
#[tokio::test]
async fn unanswered_fork_obeys_run_timeout() {
    setup_lifecycle("thread/fork", false).await;
}
