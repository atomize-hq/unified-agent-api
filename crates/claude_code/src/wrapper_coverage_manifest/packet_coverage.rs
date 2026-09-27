use super::{CoverageLevel, WrapperArgCoverageV1, WrapperCommandCoverageV1, WrapperFlagCoverageV1};

const NEW_COMMAND_NOTE: &str = "Forwarded verbatim by ClaudeNonTuiCommandRequest.";
const EXISTING_EXTENSION_NOTE: &str =
    "Forwarded verbatim by ClaudeCommandRequest and ClaudeClient::run_command.";

fn flag(key: &str, note: &str) -> WrapperFlagCoverageV1 {
    WrapperFlagCoverageV1 {
        key: key.to_string(),
        level: CoverageLevel::Passthrough,
        note: Some(note.to_string()),
        scope: None,
    }
}

fn arg(name: &str, note: &str) -> WrapperArgCoverageV1 {
    WrapperArgCoverageV1 {
        name: name.to_string(),
        level: CoverageLevel::Passthrough,
        note: Some(note.to_string()),
        scope: None,
    }
}

fn command(path: &[&str], flags: &[&str], args: &[&str], note: &str) -> WrapperCommandCoverageV1 {
    WrapperCommandCoverageV1 {
        path: path.iter().map(ToString::to_string).collect(),
        level: CoverageLevel::Passthrough,
        note: Some(note.to_string()),
        scope: None,
        flags: (!flags.is_empty()).then(|| flags.iter().map(|key| flag(key, note)).collect()),
        args: (!args.is_empty()).then(|| args.iter().map(|name| arg(name, note)).collect()),
    }
}

/// New command paths from the 2.1.274 packet that have no prior declaration.
pub(super) fn new_command_coverage() -> Vec<WrapperCommandCoverageV1> {
    vec![
        command(
            &["agents"],
            &["--all", "--cwd", "--json"],
            &[],
            NEW_COMMAND_NOTE,
        ),
        command(&["attach"], &[], &["id"], NEW_COMMAND_NOTE),
        command(&["auth"], &[], &[], NEW_COMMAND_NOTE),
        command(
            &["auth", "login"],
            &["--claudeai", "--console", "--email", "--sso"],
            &[],
            NEW_COMMAND_NOTE,
        ),
        command(&["auth", "logout"], &[], &[], NEW_COMMAND_NOTE),
        command(
            &["auth", "status"],
            &["--json", "--text"],
            &[],
            NEW_COMMAND_NOTE,
        ),
        command(&["auto-mode"], &[], &[], NEW_COMMAND_NOTE),
        command(&["auto-mode", "config"], &[], &[], NEW_COMMAND_NOTE),
        command(&["auto-mode", "critique"], &[], &[], NEW_COMMAND_NOTE),
        command(
            &["auto-mode", "defaults"],
            &["--label"],
            &[],
            NEW_COMMAND_NOTE,
        ),
        command(&["auto-mode", "reset"], &["--yes"], &[], NEW_COMMAND_NOTE),
        command(&["gateway"], &["--config"], &[], NEW_COMMAND_NOTE),
        command(&["import"], &["--dry-run", "--yes"], &[], NEW_COMMAND_NOTE),
        command(&["logs"], &[], &["id"], NEW_COMMAND_NOTE),
        command(
            &["mcp", "login"],
            &["--no-browser"],
            &["name"],
            NEW_COMMAND_NOTE,
        ),
        command(&["mcp", "logout"], &[], &["name"], NEW_COMMAND_NOTE),
        command(&["plugin", "details"], &[], &["name"], NEW_COMMAND_NOTE),
        command(
            &["plugin", "eval"],
            &[
                "--ablation",
                "--allow-real-servers",
                "--allow-tools",
                "--case",
                "--concurrency",
                "--eval-dir",
                "--json",
                "--judge-model",
                "--keep-temp",
                "--max-cost-usd",
                "--mocks",
                "--no-publish",
                "--no-scaffold",
                "--output-dir",
                "--publish-report",
                "--report",
                "--runs",
                "--scaffold",
                "--tag",
                "--threshold",
                "--trust-plugin",
            ],
            &[],
            NEW_COMMAND_NOTE,
        ),
        command(
            &["plugin", "eval", "init"],
            &["--eval-dir", "--interactive"],
            &[],
            NEW_COMMAND_NOTE,
        ),
        command(
            &["plugin", "init"],
            &[
                "--author",
                "--author-email",
                "--description",
                "--force",
                "--with",
            ],
            &[],
            NEW_COMMAND_NOTE,
        ),
        command(
            &["plugin", "prune"],
            &["--dry-run", "--scope", "--yes"],
            &[],
            NEW_COMMAND_NOTE,
        ),
        command(
            &["plugin", "tag"],
            &["--dry-run", "--force", "--message", "--push", "--remote"],
            &[],
            NEW_COMMAND_NOTE,
        ),
        command(&["project"], &[], &[], NEW_COMMAND_NOTE),
        command(
            &["project", "purge"],
            &["--all", "--dry-run", "--interactive", "--yes"],
            &[],
            NEW_COMMAND_NOTE,
        ),
        command(&["respawn"], &[], &[], NEW_COMMAND_NOTE),
        command(&["rm"], &[], &["id"], NEW_COMMAND_NOTE),
        command(&["stop"], &[], &["id"], NEW_COMMAND_NOTE),
        command(
            &["ultrareview"],
            &["--json", "--no-post", "--post", "--timeout"],
            &[],
            NEW_COMMAND_NOTE,
        ),
    ]
}

/// Adds packet-owned pass-through units to pre-existing typed declarations.
pub(super) fn extend_existing_coverage(coverage: &mut [WrapperCommandCoverageV1]) {
    let extensions: &[(&[&str], &[&str], &[&str])] = &[
        (
            &[],
            &[
                "--autocompact",
                "--ax-screen-reader",
                "--bare",
                "--bg",
                "--brief",
                "--cloud",
                "--effort",
                "--environment",
                "--exclude-dynamic-system-prompt-sections",
                "--forward-subagent-text",
                "--include-hook-events",
                "--name",
                "--permission-prompts",
                "--plugin-url",
                "--prompt-suggestions",
                "--remote-control",
                "--remote-control-session-name-prefix",
                "--restricted",
                "--safe-mode",
                "--system-prompt-snapshot",
                "--teleport",
                "--tmux",
                "--worktree",
            ],
            &[],
        ),
        (
            &["mcp", "add"],
            &["--callback-port", "--client-id", "--client-secret"],
            &[],
        ),
        (&["mcp", "add-json"], &["--client-secret"], &[]),
        (&["plugin", "disable"], &["--json"], &[]),
        (&["plugin", "enable"], &["--json"], &[]),
        (
            &["plugin", "install"],
            &["--accept-command", "--config", "--json", "--yes"],
            &[],
        ),
        (
            &["plugin", "marketplace", "add"],
            &["--claudeai", "--scope", "--sparse"],
            &[],
        ),
        (&["plugin", "marketplace", "remove"], &["--scope"], &[]),
        (
            &["plugin", "uninstall"],
            &["--json", "--keep-data", "--prune", "--yes"],
            &[],
        ),
        (
            &["plugin", "update"],
            &["--accept-command", "--json", "--yes"],
            &[],
        ),
        (&["plugin", "validate"], &["--json", "--strict"], &[]),
    ];

    for (path, flags, args) in extensions {
        let entry = coverage
            .iter_mut()
            .find(|entry| {
                entry
                    .path
                    .iter()
                    .map(String::as_str)
                    .eq(path.iter().copied())
            })
            .expect("packet extension path must retain its typed declaration");
        let declared_flags = entry.flags.get_or_insert_with(Vec::new);
        for key in *flags {
            if !declared_flags.iter().any(|item| item.key == *key) {
                declared_flags.push(flag(key, EXISTING_EXTENSION_NOTE));
            }
        }
        let declared_args = entry.args.get_or_insert_with(Vec::new);
        for name in *args {
            if !declared_args.iter().any(|item| item.name == *name) {
                declared_args.push(arg(name, EXISTING_EXTENSION_NOTE));
            }
        }
    }
}
