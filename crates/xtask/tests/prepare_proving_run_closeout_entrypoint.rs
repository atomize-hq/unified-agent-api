use clap::{CommandFactory, Parser, Subcommand};
use xtask::prepare_proving_run_closeout;

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Project automation tasks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    PrepareProvingRunCloseout(prepare_proving_run_closeout::Args),
}

#[test]
fn prepare_proving_run_closeout_help_text_includes_required_surface() {
    let top_help = Cli::command().render_help().to_string();
    assert!(top_help.contains("prepare-proving-run-closeout"));

    let err = Cli::try_parse_from(["xtask", "prepare-proving-run-closeout", "--help"])
        .expect_err("subcommand help should short-circuit parsing");
    assert_eq!(err.exit_code(), 0);
    let help_text = err.to_string();
    assert!(help_text.contains("--approval"));
    assert!(help_text.contains("--check"));
    assert!(help_text.contains("--write"));
}
