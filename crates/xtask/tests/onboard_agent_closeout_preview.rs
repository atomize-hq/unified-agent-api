use std::path::Path;

use clap::{Parser, Subcommand};
use xtask::onboard_agent;

#[allow(dead_code)]
#[path = "../src/close_proving_run.rs"]
mod close_proving_run;

#[path = "support/onboard_agent_harness.rs"]
mod harness;

#[path = "onboard_agent_closeout_preview/approval_artifact_validation.rs"]
mod approval_artifact_validation;
#[path = "onboard_agent_closeout_preview/close_proving_run_paths.rs"]
mod close_proving_run_paths;
#[path = "onboard_agent_closeout_preview/close_proving_run_write.rs"]
mod close_proving_run_write;
#[path = "onboard_agent_closeout_preview/closeout_schema_validation.rs"]
mod closeout_schema_validation;
#[path = "onboard_agent_closeout_preview/preview_states.rs"]
mod preview_states;

use harness::HarnessOutput;

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Project automation tasks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    CloseProvingRun(close_proving_run::Args),
    OnboardAgent(Box<onboard_agent::Args>),
}

fn run_cli<I, S>(argv: I, workspace_root: &Path) -> HarnessOutput
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = argv
        .into_iter()
        .map(|arg| arg.as_ref().to_string())
        .collect::<Vec<_>>();

    match Cli::try_parse_from(args) {
        Ok(cli) => {
            let mut stdout = Vec::new();
            let mut stderr = String::new();
            let exit_code = match cli.command {
                Command::OnboardAgent(args) => {
                    match onboard_agent::run_in_workspace(workspace_root, *args, &mut stdout) {
                        Ok(()) => 0,
                        Err(err) => {
                            stderr = format!("{err}\n");
                            err.exit_code()
                        }
                    }
                }
                Command::CloseProvingRun(args) => {
                    match close_proving_run::run_in_workspace(workspace_root, args, &mut stdout) {
                        Ok(()) => 0,
                        Err(err) => {
                            stderr = format!("{err}\n");
                            err.exit_code()
                        }
                    }
                }
            };
            HarnessOutput {
                exit_code,
                stdout: String::from_utf8(stdout).expect("stdout must be utf-8"),
                stderr,
            }
        }
        Err(err) => HarnessOutput {
            exit_code: err.exit_code(),
            stdout: String::new(),
            stderr: err.to_string(),
        },
    }
}
