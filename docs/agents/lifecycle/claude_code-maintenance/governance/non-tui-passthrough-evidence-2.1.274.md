# Claude Code 2.1.274 non-TUI passthrough evidence

The frozen 2.1.274 union snapshot and `coverage.any.json` identify 185 non-TUI command, flag, and positional-argument gaps. The wrapper now exposes a bounded `ClaudeNonTuiCommand` compatibility surface for every newly discovered command path except `install`; `ClaudeNonTuiCommandRequest` forwards its declared global and command arguments verbatim to `ClaudeClient::run_non_tui_command` and then the existing command runner. Extensions of existing root, MCP, and plugin paths use the public generic route: `ClaudeCommandRequest` passed to `ClaudeClient::run_command`.

`install` and its `--force` flag remain the two preexisting debt identities. Their architectural-seam reason still holds: the packet compatibility enum intentionally omits that path, so the forwarding surface cannot accidentally claim installation support.

The forwarding tests use a local fake executable. They verify argument order and values for both a bounded packet command and the generic route carrying a root global flag plus an existing MCP extension, without starting an authenticated Claude session.
