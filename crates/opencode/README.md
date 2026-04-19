# OpenCode Rust Wrapper

Async wrapper around the OpenCode CLI (`opencode`) focused on the canonical
`opencode run --format json` surface.

- crates.io package: `unified-agent-api-opencode`
- Rust library crate: `opencode`

Design goals:
- Keep the canonical runtime surface narrow: `run --format json` only.
- Preserve deterministic, machine-parseable event/completion behavior.
- Keep raw stdout/stderr and provider-specific diagnostics off the public API surface.

## Quickstart

```rust,no_run
use futures_util::StreamExt;
use opencode::{OpencodeClient, OpencodeRunJsonEvent, OpencodeRunRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpencodeClient::builder().build();
    let handle = client
        .run_json(OpencodeRunRequest::new("Reply with OK."))
        .await?;

    let mut events = handle.events;
    while let Some(event) = events.next().await {
        match event? {
            OpencodeRunJsonEvent::Text { text, .. } => println!("{text}"),
            other => println!("{other:?}"),
        }
    }

    let completion = handle.completion.await?;
    println!("status={}", completion.status);
    Ok(())
}
```

## Supported controls

The wrapper keeps support scoped to the canonical v1 controls already accepted
by `opencode run --format json`:

- `--model`
- `--session`
- `--continue`
- `--fork`
- `--dir`
