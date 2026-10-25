//! Synthetic adapter fixture, never an upstream Codex executable.
use std::io::{self, Read, Write};
fn main() -> io::Result<()> {
    // Observe every invocation before argv/stdin validation or any other fixture behavior.
    if let Some(marker) = std::env::var_os("DEPTH_CORE_SPAWN_MARKER") {
        std::fs::write(marker, b"spawned")?;
    }
    let args: Vec<_> = std::env::args().collect();
    assert!(args.iter().any(|a| a == "exec"));
    assert!(args.iter().any(|a| a == "--json"));
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    assert_eq!(input.trim_end(), "depth-core-prompt");
    let mut out = io::stdout().lock();
    if let Ok(expected) = std::env::var("DEPTH_CORE_EXPECT_CWD") {
        assert_eq!(std::env::current_dir()?, std::path::PathBuf::from(expected));
        assert_eq!(
            std::env::var("DEPTH_CORE_CREDENTIAL").unwrap(),
            "SYNTHETIC_DEPTH_CREDENTIAL"
        );
    }
    writeln!(
        out,
        "{}",
        serde_json::json!({"type":"thread.started","thread_id":"depth-core-thread"})
    )?;
    writeln!(
        out,
        "{}",
        serde_json::json!({"type":"depth.future.unknown","value":"fixture"})
    )?;
    let text = if std::env::var_os("DEPTH_CORE_LARGE").is_some() {
        "x".repeat(131_072)
    } else {
        "depth-result".into()
    };
    if let Some(i) = args.iter().position(|a| a == "--output-last-message") {
        std::fs::write(&args[i + 1], &text)?;
    }
    writeln!(
        out,
        "{}",
        serde_json::json!({"type":"item.completed","thread_id":"depth-core-thread","turn_id":"depth-core-turn","item":{"id":"m1","type":"agent_message","text":text}})
    )?;
    writeln!(
        out,
        "{}",
        serde_json::json!({"type":"turn.completed","thread_id":"depth-core-thread","turn_id":"depth-core-turn","usage":{"input_tokens":1,"cached_input_tokens":0,"output_tokens":1}})
    )?;
    out.flush()
}
