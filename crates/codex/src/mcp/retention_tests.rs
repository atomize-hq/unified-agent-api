use super::interactions::{InteractionState, MAX_FRAME};
use super::jsonrpc::WriteMessage;
use serde_json::json;
use tokio::sync::mpsc;

#[tokio::test]
async fn opaque_utf8_results_charge_the_complete_queued_frame_until_discard() {
    for excess in [0, 1] {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (errors, _) = mpsc::unbounded_channel();
        let state = InteractionState::new(tx, errors);
        let params = json!({"threadId":"t","serverName":"s","mode":"url","message":"q","elicitationId":"e","url":"https://example.invalid"});
        let request = state
            .request(
                json!("opaque"),
                "mcpServer/elicitation/request".into(),
                params,
                300,
            )
            .unwrap();
        let result =
            json!({"action":"accept","content":{"opaque":["界".repeat(100_000), {"nested":"é"}]}});
        let frame = json!({"jsonrpc":"2.0","id":"opaque","result":result})
            .to_string()
            .len();
        assert!(frame < MAX_FRAME);
        // Hold all but exactly one complete response frame in the same shared budget.
        let held = state
            .reserve(16 * MAX_FRAME - 300 - frame + excess)
            .unwrap();
        assert_eq!(request.reply(result).is_ok(), excess == 0);
        if excess == 0 {
            assert!(
                state.reserve(1).is_err(),
                "queued result must consume the remaining bytes"
            );
            let queued = rx.recv().await.unwrap();
            assert!(matches!(queued, WriteMessage::Reply { .. }));
            assert!(
                state.reserve(1).is_err(),
                "moving to writer must preserve accounting"
            );
            drop(queued);
            assert!(
                state.reserve(frame).is_ok(),
                "actual discard releases queued JSON"
            );
        }
        drop(held);
    }
}
