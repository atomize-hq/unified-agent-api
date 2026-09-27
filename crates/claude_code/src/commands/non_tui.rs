use super::command::ClaudeCommandRequest;

/// A packet-maintained, non-interactive Claude Code command path.
///
/// Installation deliberately remains absent: it is tracked as a separate
/// architectural debt item rather than being reachable through this forwarding
/// surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaudeNonTuiCommand {
    Agents,
    Attach,
    Auth,
    AuthLogin,
    AuthLogout,
    AuthStatus,
    AutoMode,
    AutoModeConfig,
    AutoModeCritique,
    AutoModeDefaults,
    AutoModeReset,
    Gateway,
    Import,
    Logs,
    McpLogin,
    McpLogout,
    PluginDetails,
    PluginEval,
    PluginEvalInit,
    PluginInit,
    PluginPrune,
    PluginTag,
    Project,
    ProjectPurge,
    Respawn,
    Rm,
    Stop,
    Ultrareview,
}

impl ClaudeNonTuiCommand {
    /// Every command path admitted by the packet forwarding surface.
    pub const fn all() -> &'static [Self] {
        &[
            Self::Agents,
            Self::Attach,
            Self::Auth,
            Self::AuthLogin,
            Self::AuthLogout,
            Self::AuthStatus,
            Self::AutoMode,
            Self::AutoModeConfig,
            Self::AutoModeCritique,
            Self::AutoModeDefaults,
            Self::AutoModeReset,
            Self::Gateway,
            Self::Import,
            Self::Logs,
            Self::McpLogin,
            Self::McpLogout,
            Self::PluginDetails,
            Self::PluginEval,
            Self::PluginEvalInit,
            Self::PluginInit,
            Self::PluginPrune,
            Self::PluginTag,
            Self::Project,
            Self::ProjectPurge,
            Self::Respawn,
            Self::Rm,
            Self::Stop,
            Self::Ultrareview,
        ]
    }

    pub const fn path(self) -> &'static [&'static str] {
        match self {
            Self::Agents => &["agents"],
            Self::Attach => &["attach"],
            Self::Auth => &["auth"],
            Self::AuthLogin => &["auth", "login"],
            Self::AuthLogout => &["auth", "logout"],
            Self::AuthStatus => &["auth", "status"],
            Self::AutoMode => &["auto-mode"],
            Self::AutoModeConfig => &["auto-mode", "config"],
            Self::AutoModeCritique => &["auto-mode", "critique"],
            Self::AutoModeDefaults => &["auto-mode", "defaults"],
            Self::AutoModeReset => &["auto-mode", "reset"],
            Self::Gateway => &["gateway"],
            Self::Import => &["import"],
            Self::Logs => &["logs"],
            Self::McpLogin => &["mcp", "login"],
            Self::McpLogout => &["mcp", "logout"],
            Self::PluginDetails => &["plugin", "details"],
            Self::PluginEval => &["plugin", "eval"],
            Self::PluginEvalInit => &["plugin", "eval", "init"],
            Self::PluginInit => &["plugin", "init"],
            Self::PluginPrune => &["plugin", "prune"],
            Self::PluginTag => &["plugin", "tag"],
            Self::Project => &["project"],
            Self::ProjectPurge => &["project", "purge"],
            Self::Respawn => &["respawn"],
            Self::Rm => &["rm"],
            Self::Stop => &["stop"],
            Self::Ultrareview => &["ultrareview"],
        }
    }
}

/// A bounded pass-through request for a packet-maintained non-TUI command.
///
/// `global_args` are emitted before the fixed command path; `arguments` are
/// emitted after it. Both are forwarded verbatim, allowing newly surfaced
/// upstream flags to be used without pretending that they have typed wrapper
/// semantics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaudeNonTuiCommandRequest {
    pub command: ClaudeNonTuiCommand,
    pub global_args: Vec<String>,
    pub arguments: Vec<String>,
}

impl ClaudeNonTuiCommandRequest {
    pub fn new(command: ClaudeNonTuiCommand) -> Self {
        Self {
            command,
            global_args: Vec::new(),
            arguments: Vec::new(),
        }
    }

    pub fn global_arg(mut self, arg: impl Into<String>) -> Self {
        self.global_args.push(arg.into());
        self
    }

    pub fn global_args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.global_args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.arguments.push(arg.into());
        self
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.arguments.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn into_command(self) -> ClaudeCommandRequest {
        ClaudeCommandRequest::root()
            .args(self.global_args)
            .args(self.command.path().iter().copied())
            .args(self.arguments)
    }
}
