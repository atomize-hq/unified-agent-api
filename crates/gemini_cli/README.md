# Gemini CLI Rust Wrapper

Async wrapper around the official Gemini CLI focused on the headless
`--output-format stream-json` flow.

- crates.io package: `unified-agent-api-gemini-cli`
- Rust library crate: `gemini_cli`

Design goals:
- Keep the public surface aligned to the documented stream-json contract.
- Preserve raw event payloads so callers can tolerate upstream shape changes.
- Support deterministic headless execution for backend and automation use cases.

## Quickstart

```rust,no_run
use futures_util::StreamExt;
use gemini_cli::{GeminiCliClient, GeminiStreamJsonRunRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = GeminiCliClient::builder().build();
    let handle = client
        .stream_json(GeminiStreamJsonRunRequest::new("Reply with OK."))
        .await?;

    let mut events = handle.events;
    while let Some(event) = events.next().await {
        println!("{:?}", event?);
    }

    let completion = handle.completion.await?;
    println!("status={}", completion.status);
    Ok(())
}
```
