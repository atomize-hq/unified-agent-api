use super::{Broker, FRAME};
use codex::mcp::{AppServerMessage, CodexAppServer, StdioServerConfig};
use futures_util::stream::FuturesUnordered;
use serde_json::Value;
use std::{
    collections::{BTreeMap, VecDeque},
    os::unix::fs::PermissionsExt,
    time::Duration,
};

#[tokio::test]
async fn preack_queue_charges_complete_utf8_method_json_and_preserves_transferred_lease() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("idle");
    std::fs::write(&script, "#!/bin/sh\ncat >/dev/null\n").unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();
    let (server, stream) = CodexAppServer::spawn_uninitialized_with_interactions_and_config(
        StdioServerConfig {
            binary: script,
            code_home: None,
            current_dir: None,
            env: Vec::new(),
            app_server_analytics_default_enabled: false,
            mirror_stdio: false,
            startup_timeout: Duration::from_secs(2),
        },
        BTreeMap::new(),
    )
    .await
    .unwrap();
    let mut broker = Broker {
        thread: None,
        turn: None,
        queue: VecDeque::new(),
        contexts: BTreeMap::new(),
        pending: BTreeMap::new(),
        callbacks: FuturesUnordered::new(),
        serial: 0,
        final_text: None,
    };
    let mut method = "界".repeat((FRAME - 40) / 3);
    while serde_json::json!({"method":method,"params":null})
        .to_string()
        .len()
        < FRAME
    {
        method.push('x');
    }
    assert_eq!(
        serde_json::json!({"method":method,"params":null})
            .to_string()
            .len(),
        FRAME
    );
    for _ in 0..16 {
        broker
            .queue(
                AppServerMessage::Notification {
                    method: method.clone(),
                    params: Value::Null,
                },
                &stream,
            )
            .unwrap();
    }
    assert!(
        stream.retain_json(1).is_err(),
        "complete methods fill exactly 16 MiB"
    );
    assert!(broker
        .queue(
            AppServerMessage::Notification {
                method: "x".into(),
                params: Value::Null
            },
            &stream
        )
        .is_err());
    let transferred = broker.queue.pop_front().unwrap();
    assert!(
        stream.retain_json(1).is_err(),
        "moving owned JSON retains its lease"
    );
    drop(transferred);
    let remainder = stream.retain_json(FRAME).unwrap();
    assert!(stream.retain_json(1).is_err());
    drop(remainder);
    drop(broker);
    assert!(
        stream.retain_json(16 * FRAME).is_ok(),
        "discard releases all queued JSON"
    );
    drop(stream);
    server.shutdown_and_wait().await.unwrap();
}
