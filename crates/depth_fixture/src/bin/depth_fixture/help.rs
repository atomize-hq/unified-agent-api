// Shared fixture output, compiled into both the CLI and bounded acquisition validator.
pub const ROOT_HELP: &str = "depth_fixture - synthetic CLI for the support-depth machinery proof

Usage: depth_fixture [OPTIONS] <COMMAND>

Commands:
  run  Run one prompt

Options:
      --surface-json  Print the command surface as JSON
  -h, --help          Print help
  -V, --version       Print version
";

pub const RUN_HELP: &str = "Run one prompt

Usage: depth_fixture run [OPTIONS] [--] <PROMPT>

Arguments:
  <PROMPT>  The prompt

Options:
      --json          Emit JSON Lines events
      --model <NAME>  Model name
  -h, --help          Print help
";
