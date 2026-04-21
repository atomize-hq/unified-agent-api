use std::{
    env, fs,
    io::{self, Write},
    thread,
    time::Duration,
};

const INIT: &str = include_str!("../../tests/fixtures/stream_json/v1/init.jsonl");
const MESSAGE: &str = include_str!("../../tests/fixtures/stream_json/v1/message.jsonl");
const TOOL_USE: &str = include_str!("../../tests/fixtures/stream_json/v1/tool_use.jsonl");
const TOOL_RESULT: &str = include_str!("../../tests/fixtures/stream_json/v1/tool_result.jsonl");
const RESULT_SUCCESS: &str =
    include_str!("../../tests/fixtures/stream_json/v1/result_success.jsonl");
const RESULT_ERROR: &str = include_str!("../../tests/fixtures/stream_json/v1/result_error.jsonl");

fn first_nonempty_line(text: &str) -> &str {
    text.lines()
        .find(|line| !line.chars().all(|ch| ch.is_whitespace()))
        .expect("fixture contains a non-empty line")
}

fn write_line(out: &mut impl Write, line: &str) -> io::Result<()> {
    out.write_all(line.as_bytes())?;
    out.flush()?;
    Ok(())
}

fn capture_invocation() {
    let capture_path = match env::var("FAKE_GEMINI_CAPTURE") {
        Ok(path) => path,
        Err(_) => return,
    };

    let payload = serde_json::json!({
        "argv": env::args().skip(1).collect::<Vec<_>>(),
        "cwd": env::current_dir().ok().map(|path| path.display().to_string()),
    });
    fs::write(
        capture_path,
        serde_json::to_vec_pretty(&payload).expect("serialize capture"),
    )
    .expect("write capture");
}

fn main() -> io::Result<()> {
    capture_invocation();

    let scenario =
        env::var("FAKE_GEMINI_SCENARIO").unwrap_or_else(|_| "three_events_delayed".to_string());

    let init = first_nonempty_line(INIT);
    let message = first_nonempty_line(MESSAGE);
    let tool_use = first_nonempty_line(TOOL_USE);
    let tool_result = first_nonempty_line(TOOL_RESULT);
    let result_success = first_nonempty_line(RESULT_SUCCESS);
    let result_error = first_nonempty_line(RESULT_ERROR);
    let mut out = io::stdout().lock();

    match scenario.as_str() {
        "capture_args" => {
            write_line(&mut out, &format!("{init}\n"))?;
            write_line(&mut out, &format!("{message}\n"))?;
            write_line(&mut out, &format!("{result_success}\n"))?;
        }
        "crlf_blank_lines" => {
            write_line(&mut out, "\r\n")?;
            write_line(&mut out, "   \r\n")?;
            write_line(&mut out, &format!("{init}\r\n"))?;
            write_line(&mut out, "\r\n")?;
            write_line(&mut out, &format!("{message}\r\n"))?;
            write_line(&mut out, &format!("{result_success}\r\n"))?;
        }
        "parse_error_redaction" => {
            let secret = "VERY_SECRET_SHOULD_NOT_APPEAR";
            write_line(&mut out, &format!("not json {secret}\n"))?;
            write_line(&mut out, &format!("{init}\n"))?;
            write_line(&mut out, &format!("{message}\n"))?;
            write_line(&mut out, &format!("{result_success}\n"))?;
        }
        "tool_roundtrip" => {
            write_line(&mut out, &format!("{init}\n"))?;
            write_line(&mut out, &format!("{tool_use}\n"))?;
            write_line(&mut out, &format!("{tool_result}\n"))?;
            write_line(
                &mut out,
                "{\"type\":\"message\",\"timestamp\":\"2026-04-21T12:00:03Z\",\"role\":\"assistant\",\"content\":\"done\",\"delta\":true}\n",
            )?;
            write_line(&mut out, &format!("{result_success}\n"))?;
        }
        "slow_until_killed" => {
            write_line(&mut out, &format!("{init}\n"))?;
            thread::sleep(Duration::from_secs(30));
        }
        "turn_limit_exceeded" => {
            write_line(&mut out, &format!("{init}\n"))?;
            write_line(
                &mut out,
                "{\"type\":\"message\",\"timestamp\":\"2026-04-21T12:00:01Z\",\"role\":\"assistant\",\"content\":\"partial\",\"delta\":true}\n",
            )?;
            write_line(&mut out, &format!("{result_error}\n"))?;
            std::process::exit(53);
        }
        "invalid_input" => {
            write_line(
                &mut out,
                "{\"type\":\"result\",\"timestamp\":\"2026-04-21T12:00:02Z\",\"status\":\"error\",\"error\":{\"type\":\"input_error\",\"message\":\"Prompt rejected\"}}\n",
            )?;
            std::process::exit(42);
        }
        _ => {
            write_line(&mut out, &format!("{init}\n"))?;
            thread::sleep(Duration::from_millis(150));
            write_line(&mut out, &format!("{message}\n"))?;
            thread::sleep(Duration::from_millis(150));
            write_line(&mut out, &format!("{result_success}\n"))?;
        }
    }

    Ok(())
}
