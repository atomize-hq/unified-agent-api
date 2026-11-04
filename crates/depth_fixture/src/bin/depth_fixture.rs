//! Synthetic upstream CLI for the isolated support-depth machinery proof. It qualifies nothing.
//!
//! Surface: `--version`, `--help`, `--surface-json`, and
//! `run [--json] [--model <NAME>] [--] <PROMPT>`. `DEPTH_FIXTURE_SCENARIO` selects a scripted
//! behavior so wrapper tests can reach every promised path; it is part of this fixture's API.

use std::io::{self, IsTerminal, Read, Write};
use std::process::ExitCode;
use std::{env, thread, time::Duration};

const VERSION: &str = match option_env!("DEPTH_FIXTURE_UPSTREAM_VERSION") {
    Some(version) => version,
    None => "0.1.0",
};

const ROOT_HELP: &str = "depth_fixture - synthetic CLI for the support-depth machinery proof

Usage: depth_fixture [OPTIONS] <COMMAND>

Commands:
  run  Run one prompt

Options:
      --surface-json  Print the command surface as JSON
  -h, --help          Print help
  -V, --version       Print version
";

const RUN_HELP: &str = "Run one prompt

Usage: depth_fixture run [OPTIONS] [--] <PROMPT>

Arguments:
  <PROMPT>  The prompt

Options:
      --json          Emit JSON Lines events
      --model <NAME>  Model name
  -h, --help          Print help
";

const SURFACE: &str = r#"[
  {"path":[],"about":"synthetic CLI for the support-depth machinery proof","flags":[
    {"long":"--help","short":"-h","takes_value":false},
    {"long":"--surface-json","takes_value":false},
    {"long":"--version","short":"-V","takes_value":false}]},
  {"path":["run"],"about":"Run one prompt","usage":"depth_fixture run [OPTIONS] [--] <PROMPT>",
   "args":[{"name":"PROMPT","required":true,"variadic":false}],
   "flags":[
    {"long":"--help","short":"-h","takes_value":false},
    {"long":"--json","takes_value":false},
    {"long":"--model","takes_value":true,"value_name":"NAME"}]}
]"#;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("-V" | "--version") => out(&format!("depth_fixture {VERSION}\n")),
        Some("-h" | "--help") | None => out(ROOT_HELP),
        Some("--surface-json") => out(&format!("{SURFACE}\n")),
        Some("run") => run(&args[1..]),
        Some(other) => usage(&format!("unknown command `{other}`")),
    }
}

fn out(text: &str) -> ExitCode {
    print!("{text}");
    ExitCode::SUCCESS
}

fn usage(message: &str) -> ExitCode {
    eprintln!("error: {message}");
    ExitCode::from(2)
}

fn run(args: &[String]) -> ExitCode {
    let (mut json, mut model, mut prompt) = (false, None, None);
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "-h" | "--help" => return out(RUN_HELP),
            "--json" => json = true,
            "--model" => match rest.next() {
                Some(value) => model = Some(value.clone()),
                None => return usage("--model requires a value"),
            },
            "--" => {
                prompt = rest.next().cloned();
                if rest.next().is_some() {
                    return usage("exactly one prompt is accepted");
                }
                break;
            }
            flag if flag.starts_with('-') => return usage(&format!("unknown flag `{flag}`")),
            _ if prompt.is_none() => prompt = Some(arg.clone()),
            _ => return usage("exactly one prompt is accepted"),
        }
    }
    let Some(prompt) = prompt else {
        return usage("missing <PROMPT>");
    };
    let model = model.unwrap_or_else(|| "fixture-default".to_owned());
    if !json {
        return out(&format!("{prompt}\n"));
    }
    let scenario = env::var("DEPTH_FIXTURE_SCENARIO").unwrap_or_else(|_| "ok".to_owned());
    let started = r#"{"type":"session.started","session_id":"fx-1"}"#;
    let message = serde_json::json!({"type": "message", "text": prompt, "model": model});
    let completed = r#"{"type":"run.completed","ok":true}"#;
    let mut stdout = io::stdout().lock();
    let mut emit = |line: &str| {
        // A closed reader is the wrapper's choice; the fixture just stops.
        let _ = writeln!(stdout, "{line}").and_then(|()| stdout.flush());
    };
    match scenario.as_str() {
        "ok" => {
            emit(started);
            emit(&message.to_string());
            emit(completed);
        }
        "unknown_event" => {
            emit(started);
            emit(r#"{"type":"future.kind","detail":1}"#);
            emit(completed);
        }
        "malformed" => {
            emit(started);
            emit("{not json");
        }
        "fail" => {
            emit(started);
            eprintln!("fixture failure");
            return ExitCode::from(3);
        }
        "no_terminal" => {
            emit(started);
            emit(&message.to_string());
        }
        "completed_then_fail" => {
            emit(started);
            emit(completed);
            return ExitCode::from(4);
        }
        "slow" => {
            emit(started);
            thread::sleep(Duration::from_secs(30));
            emit(completed);
        }
        "large" => {
            emit(started);
            emit(&serde_json::json!({"type": "message", "text": "x".repeat(200_000)}).to_string());
            emit(completed);
        }
        "environment" => {
            // Reports what the wrapper let through, for environment and credential tests.
            let seen = |name: &str| env::var(name).unwrap_or_else(|_| "<unset>".to_owned());
            let cwd = env::current_dir()
                .map(|d| d.display().to_string())
                .unwrap_or_default();
            let stdin_tty = io::stdin().is_terminal();
            let mut stdin = String::new();
            let _ = io::stdin().read_to_string(&mut stdin);
            let text = format!(
                "home={} secret={} cwd={} stdin_bytes={} stdin_tty={}",
                seen("HOME"),
                seen("DEPTH_FIXTURE_SECRET"),
                cwd,
                stdin.len(),
                stdin_tty
            );
            emit(started);
            emit(&serde_json::json!({"type": "message", "text": text}).to_string());
            emit(completed);
        }
        "burst" => {
            // More output than the pipe plus the wrapper's buffer, for backpressure tests.
            emit(started);
            for i in 0..10_000 {
                emit(&serde_json::json!({"type": "message", "text": format!("line {i:05} {}", "y".repeat(64))}).to_string());
            }
            emit(completed);
        }
        "malformed_binary" => {
            emit(started);
            let _ = stdout
                .write_all(&[0xFF; 300])
                .and_then(|()| stdout.write_all(b"\n"));
        }
        "stderr_sentinel" => {
            // Over 64 KiB of stderr, ending with what the child can see of a parent sentinel.
            emit(started);
            let mut stderr = io::stderr().lock();
            for _ in 0..2_000 {
                let _ = writeln!(stderr, "{}", "z".repeat(63));
            }
            let secret = env::var("DEPTH_FIXTURE_SECRET").unwrap_or_else(|_| "<unset>".to_owned());
            let _ = writeln!(stderr, "secret={secret}");
            return ExitCode::from(5);
        }
        other => return usage(&format!("unknown scenario `{other}`")),
    }
    ExitCode::SUCCESS
}
