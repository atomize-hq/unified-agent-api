use std::{
    env, fs,
    io::{self, Write},
    thread,
    time::Duration,
};

const STEP_START: &str = include_str!("../../tests/fixtures/run_json/v1/step_start.jsonl");
const TEXT: &str = include_str!("../../tests/fixtures/run_json/v1/text.jsonl");
const STEP_FINISH: &str = include_str!("../../tests/fixtures/run_json/v1/step_finish.jsonl");

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
    let capture_path = match env::var("FAKE_OPENCODE_CAPTURE") {
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
        env::var("FAKE_OPENCODE_SCENARIO").unwrap_or_else(|_| "three_events_delayed".to_string());

    let step_start = first_nonempty_line(STEP_START);
    let text = first_nonempty_line(TEXT);
    let step_finish = first_nonempty_line(STEP_FINISH);
    let mut out = io::stdout().lock();

    match scenario.as_str() {
        "capture_args" => {
            write_line(&mut out, &format!("{step_start}\n"))?;
            write_line(&mut out, &format!("{text}\n"))?;
            write_line(&mut out, &format!("{step_finish}\n"))?;
        }
        "crlf_blank_lines" => {
            write_line(&mut out, "\r\n")?;
            write_line(&mut out, "   \r\n")?;
            write_line(&mut out, &format!("{step_start}\r\n"))?;
            write_line(&mut out, "\r\n")?;
            write_line(&mut out, &format!("{text}\r\n"))?;
            write_line(&mut out, &format!("{step_finish}\r\n"))?;
        }
        "parse_error_redaction" => {
            let secret = "VERY_SECRET_SHOULD_NOT_APPEAR";
            write_line(&mut out, &format!("not json {secret}\n"))?;
            write_line(&mut out, &format!("{step_start}\n"))?;
            write_line(&mut out, &format!("{text}\n"))?;
            write_line(&mut out, &format!("{step_finish}\n"))?;
        }
        "slow_until_killed" => {
            write_line(&mut out, &format!("{step_start}\n"))?;
            thread::sleep(Duration::from_secs(30));
        }
        _ => {
            write_line(&mut out, &format!("{step_start}\n"))?;
            thread::sleep(Duration::from_millis(150));
            write_line(&mut out, &format!("{text}\n"))?;
            thread::sleep(Duration::from_millis(150));
            write_line(&mut out, &format!("{step_finish}\n"))?;
        }
    }

    Ok(())
}
