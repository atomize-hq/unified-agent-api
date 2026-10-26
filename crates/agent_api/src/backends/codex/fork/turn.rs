use codex::mcp::{CodexAppServer, TurnInterruptParams};

pub(super) async fn interrupt(server: &CodexAppServer, thread_id: &str, turn_id: &str) {
    // Queuing the request precedes closing stdin, which drains the writer queue.
    let _ = server
        .turn_interrupt(TurnInterruptParams {
            thread_id: Some(thread_id.to_string()),
            turn_id: turn_id.to_string(),
        })
        .await;
}
