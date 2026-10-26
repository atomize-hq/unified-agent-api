use super::{backend, request};
use agent_api::backends::codex::CodexInteractionReply;
use std::{
    io::{Read, Write},
    os::unix::net::UnixListener,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::oneshot;
struct Dropped(Option<oneshot::Sender<()>>);
impl Drop for Dropped {
    fn drop(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}
async fn gated(scenario: &str) {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("gate");
    let listener = UnixListener::bind(&socket).unwrap();
    let _wake = super::WakeGate(socket.clone());
    let gate = tokio::task::spawn_blocking(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(4)))
            .unwrap();
        let mut ready = [0; 5];
        stream.read_exact(&mut ready).unwrap();
        assert_eq!(&ready, b"ready");
        stream
    });
    let (ready_tx, ready_rx) = oneshot::channel();
    let ready = Arc::new(Mutex::new(Some(ready_tx)));
    let (drop_tx, drop_rx) = oneshot::channel();
    let dropped = Arc::new(Mutex::new(Some(drop_tx)));
    let mut config = super::CodexBackendConfig {
        binary: Some(env!("CARGO_BIN_EXE_fake_codex_native_bridge").into()),
        env: [
            ("NATIVE_SCENARIO".into(), scenario.into()),
            ("NATIVE_GATE".into(), socket.display().to_string()),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    };
    let replies = dir.path().join("replies");
    config
        .env
        .insert("NATIVE_REPLY_LOG".into(), replies.display().to_string());
    let backend = super::CodexBackend::new(config).with_interaction_handler(move |_| {
        let ready = ready.clone();
        let dropped = dropped.clone();
        async move {
            let _guard = Dropped(dropped.lock().unwrap().take());
            let _ = ready.lock().unwrap().take().unwrap().send(());
            std::future::pending::<Result<CodexInteractionReply, super::CodexInteractionError>>()
                .await
        }
    });
    let run = super::AgentWrapperBackend::run(&backend, request())
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(3), ready_rx)
        .await
        .unwrap()
        .unwrap();
    let mut gate = tokio::time::timeout(Duration::from_secs(3), gate)
        .await
        .unwrap()
        .unwrap();
    gate.write_all(b"x").unwrap();
    // Host work must be cancelled while the public event consumer is still unpolled.
    tokio::time::timeout(Duration::from_secs(3), drop_rx)
        .await
        .unwrap()
        .unwrap();
    drop(run.events);
    let completion = tokio::time::timeout(Duration::from_secs(3), run.completion)
        .await
        .unwrap();
    if scenario.contains("resolved") || scenario.contains("terminal") {
        assert!(completion.is_ok());
    } else {
        assert!(completion.is_err());
    }
    let replies = std::fs::read_to_string(replies).unwrap_or_default();
    assert!(
        !replies.contains("\"result\""),
        "late native result: {replies}"
    );
}
#[tokio::test]
async fn changed_completed_resolved_terminal_disconnect_cancel_owned_handler() {
    for scenario in [
        "file_drift",
        "file_replacement",
        "file_completed",
        "file_resolved",
        "file_terminal",
        "file_disconnect",
        "file_closed",
        "mcp_null_resolved",
        "mcp_absent_terminal",
        "mcp_null_closed",
        "mcp_absent_disconnect",
    ] {
        gated(scenario).await;
    }
}
#[tokio::test]
async fn cancel_and_timeout_drop_file_and_standalone_handlers_without_event_polling() {
    for scenario in [
        "file_wait",
        "file_output_wait",
        "mcp_absent_wait",
        "mcp_null_wait",
    ] {
        for cancelled in [true, false] {
            let (ready_tx, ready_rx) = oneshot::channel();
            let ready = Arc::new(Mutex::new(Some(ready_tx)));
            let (drop_tx, drop_rx) = oneshot::channel();
            let dropped = Arc::new(Mutex::new(Some(drop_tx)));
            let backend = backend(scenario).with_interaction_handler(move |_| {
                let ready = ready.clone();
                let dropped = dropped.clone();
                async move {
                    let _guard = Dropped(dropped.lock().unwrap().take());
                    let _ = ready.lock().unwrap().take().unwrap().send(());
                    std::future::pending::<
                        Result<CodexInteractionReply, super::CodexInteractionError>,
                    >()
                    .await
                }
            });
            let mut request = request();
            request.timeout = Some(if cancelled {
                Duration::ZERO
            } else {
                Duration::from_secs(1)
            });
            let run = super::AgentWrapperBackend::run_control(&backend, request)
                .await
                .unwrap();
            tokio::time::timeout(Duration::from_secs(3), ready_rx)
                .await
                .unwrap()
                .unwrap();
            if cancelled {
                run.cancel.cancel();
            }
            tokio::time::timeout(Duration::from_secs(3), drop_rx)
                .await
                .unwrap()
                .unwrap();
            drop(run.handle.events);
            assert!(
                tokio::time::timeout(Duration::from_secs(3), run.handle.completion)
                    .await
                    .unwrap()
                    .is_err()
            );
        }
    }
}
