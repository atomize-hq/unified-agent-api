# Codex 0.156.1 wrapper-only adjudication evidence

Prepared from release-tagged upstream source on 2026-09-25. This is a manual, packet-scoped adjudication; it does not add discovery infrastructure. The input set is the 32 rows in `cli_manifests/codex/reports/0.156.1/coverage.any.json` at packet refreeze commit `b1a76ff6edf9a240ef26fd72e32cc3c26487e0c5`.

This document records source evidence only. The sole disposition list belongs in `maintenance-closeout.json` after CI succeeds. No runtime execution or cross-platform binary proof is claimed by this source review.

## Current hidden commands

- `execpolicy check`, `--rules`, `--pretty`, and `COMMAND`: the [hidden command registration](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/cli/src/main.rs#L202) reaches the [check implementation](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/execpolicy/src/execpolicycheck.rs#L17).
- `responses-api-proxy` and its four flags (`--http-shutdown`, `--port`, `--server-info`, `--upstream-url`): the [hidden registration](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/cli/src/main.rs#L235) reaches the [argument parser and proxy implementation](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/responses-api-proxy/src/lib.rs#L38).
- `stdio-to-uds` and `SOCKET_PATH`: [hidden registration](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/cli/src/main.rs#L239), [argument parser](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/cli/src/main.rs#L885), and [dispatch](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/cli/src/main.rs#L1945).

## Retained historical invocations

- Root `--full-auto`: [0.125.0 applies its approval and sandbox policy](https://github.com/openai/codex/blob/637f7dd6d737f3961e6bf32fbb3861c4953269c5/codex-rs/cli/src/main.rs#L1323). 0.126.0 removes the root parser option. [Removal commit](https://github.com/openai/codex/commit/3d10ba9f36a7c94b2d9413363df6ae5cd22ea09d).
- `exec-server --executor-id`: [0.132.0 parses it](https://github.com/openai/codex/blob/13595c36e218fcbd13df118eeadf00d4eb0e6d31/codex-rs/cli/src/main.rs#L489) and [uses it for registration](https://github.com/openai/codex/blob/13595c36e218fcbd13df118eeadf00d4eb0e6d31/codex-rs/cli/src/main.rs#L1523). 0.133.0 changes that interface.
- The 16 `sandbox help/linux/macos/windows` command, flag and argument rows: [0.133.0 has platform subcommands](https://github.com/openai/codex/blob/9474e5cfc4494b0ba319352aa86ce436c59e65c8/codex-rs/cli/src/main.rs#L353) and [their option and command parsers](https://github.com/openai/codex/blob/9474e5cfc4494b0ba319352aa86ce436c59e65c8/codex-rs/cli/src/lib.rs#L25). Clap supplies the nested help command; the committed 0.129.0 union also positively records its COMMAND argument. 0.134.0 changes sandbox to a host-specific argument parser. [Removal commit](https://github.com/openai/codex/commit/c0b16cfc6b653fe62a49ddb8982ea879dff87751).
- `mcp-server`: [0.153.4 registers it](https://github.com/openai/codex/blob/3d2ee51ca2d5db578f328aa75e20aa22c0197c9a/codex-rs/cli/src/main.rs#L156) and [dispatches to its server](https://github.com/openai/codex/blob/3d2ee51ca2d5db578f328aa75e20aa22c0197c9a/codex-rs/cli/src/main.rs#L1181). 0.154.0 removes it. [Removal commit](https://github.com/openai/codex/commit/531f3836a1e38ea61eaaba3dccda6711eb6c0dca).
- `login --api-key`: [0.44.0 performs login with the argument](https://github.com/openai/codex/blob/488ec061bf4d36916b8f477c700ea4fde4162a7a/codex-rs/cli/src/main.rs#L301). 0.45.0 replaces it with stdin-based `--with-api-key`. In 0.156.1 the hidden compatibility parser [exits with an error](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/cli/src/main.rs#L1738); this is not successful hidden support. [Removal commit](https://github.com/openai/codex/commit/69ac5153d4212d629036df8b24c2015caa615099).

## Withdrawn publication claims

`login --mcp` is absent from the complete [0.156.1 LoginCommand parser](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/cli/src/main.rs#L505). The current MCP authentication command is `mcp login`, a distinct wrapper surface. No historical upstream implementation of `login --mcp` was established. Consequently, neither hidden support nor older support can be asserted. The packet withdraws the publication claim and preserves the existing conditional wrapper method for API compatibility. This is a publication decision, not a claim that historical support or a particular removal release has been proved.

`execpolicy check --policy` is also absent from the 0.156.1 parser cited above, which requires repeatable `--rules`. The wrapper now emits `--rules` while preserving the public Rust `.policy()` and `.policies()` methods. The same source emits `decision` and `matchedRules` at the top level; the wrapper normalizes that current response into its existing public match/no-match representation while retaining legacy nested-response compatibility. The packet withdraws the unsupported `--policy` claim; no historical support or removal version is asserted.

## Release identity

| Version | Upstream commit |
| --- | --- |
| 0.44.0 | `488ec061bf4d36916b8f477c700ea4fde4162a7a` |
| 0.125.0 | `637f7dd6d737f3961e6bf32fbb3861c4953269c5` |
| 0.132.0 | `13595c36e218fcbd13df118eeadf00d4eb0e6d31` |
| 0.133.0 | `9474e5cfc4494b0ba319352aa86ce436c59e65c8` |
| 0.153.4 | `3d2ee51ca2d5db578f328aa75e20aa22c0197c9a` |
| 0.156.1 | `b412ff32c417f855c2b2d1581b77058eed87c84b` |
