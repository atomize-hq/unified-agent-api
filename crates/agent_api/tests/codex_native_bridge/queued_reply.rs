use super::{request, CodexBackend, CodexBackendConfig, CodexInteractionReply};
use agent_api::AgentWrapperBackend;
use serde_json::json;
use std::{
    io::{Read, Write},
    os::unix::net::UnixListener,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::oneshot;

#[tokio::test]
async fn completed_file_revokes_already_queued_grant_before_writer_first_io() {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("gate");
    let listener = UnixListener::bind(&socket).unwrap();
    let wake = super::WakeGate(socket.clone());
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
    let (queued_tx, queued_rx) = oneshot::channel();
    let queued = Arc::new(Mutex::new(Some(queued_tx)));
    let (processed_tx, processed_rx) = oneshot::channel();
    let processed = Arc::new(Mutex::new(Some(processed_tx)));
    let replies = dir.path().join("replies");
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(env!("CARGO_BIN_EXE_fake_codex_native_bridge").into()),
        env: [
            ("NATIVE_SCENARIO".into(), "queued_completion".into()),
            ("NATIVE_GATE".into(), socket.display().to_string()),
            ("NATIVE_REPLY_LOG".into(), replies.display().to_string()),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    })
    .with_interaction_handler(move |request| {
        let queued = queued.clone();
        let processed = processed.clone();
        async move {
            if request.item_context.is_some() {
                // This current-thread runtime cannot poll the test again until the broker
                // consumes Ready and queues this result: there is no intervening await.
                queued.lock().unwrap().take().unwrap().send(()).unwrap();
                return Ok(CodexInteractionReply::Result(
                    json!({"decision":"acceptForSession"}),
                ));
            }
            if queued.lock().unwrap().is_none() {
                if let Some(tx) = processed.lock().unwrap().take() {
                    let _ = tx.send(());
                }
                return Ok(CodexInteractionReply::Result(json!({"action":"decline"})));
            }
            Ok(CodexInteractionReply::Result(
                json!({"action":"accept","content":{"opaque":"界".repeat(240_000)}}),
            ))
        }
    });
    let run = backend.run(request()).await.unwrap();
    drop(backend);
    drop(run.events);
    let ready = tokio::time::timeout(Duration::from_secs(3), queued_rx).await;
    // Always wake/reap a blocked listener, even if setup or callback dispatch failed.
    if ready.is_err() || ready.as_ref().is_ok_and(|r| r.is_err()) {
        drop(wake);
        let _ = gate.await;
        panic!("file callback did not queue: {ready:?}");
    }
    let mut gate = gate.await.unwrap();
    gate.write_all(b"x").unwrap();
    // The later callback proves item/completed was processed while stdin is blocked.
    let processed = tokio::time::timeout(Duration::from_secs(3), processed_rx).await;
    gate.write_all(b"x").unwrap();
    let completion = tokio::time::timeout(Duration::from_secs(5), run.completion).await;
    assert!(
        processed.is_ok_and(|result| result.is_ok()),
        "no processing barrier"
    );
    let raw = std::fs::read_to_string(replies).unwrap();
    assert!(
        !raw.contains("acceptForSession"),
        "completed file grant reached stdin"
    );
    assert!(completion.unwrap().unwrap().status.success());
}

#[tokio::test]
async fn completion_after_native_observes_written_file_approval_remains_successful() {
    let backend = super::backend("file_written").with_interaction_handler(|_| async {
        Ok(CodexInteractionReply::Result(
            json!({"decision":"acceptForSession"}),
        ))
    });
    let run = backend.run(request()).await.unwrap();
    drop(run.events);
    assert!(run.completion.await.unwrap().status.success());
}
