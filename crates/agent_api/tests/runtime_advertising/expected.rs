//! Independent full-set oracle transcribed from the five implementations at
//! 2992389f8cbdbb5e9f7bc1e107f5dc4ee773e810, before the shared-model bridge.
use std::collections::BTreeSet;

pub fn expected(agent: &str, os: &str, arch: &str, write: bool, sandbox: bool) -> BTreeSet<String> {
    let mut ids = BTreeSet::from([
        "agent_api.run",
        "agent_api.events",
        "agent_api.events.live",
        "agent_api.config.model.v1",
    ]);
    if matches!(agent, "codex" | "claude_code" | "opencode") {
        ids.extend(["agent_api.session.resume.v1", "agent_api.session.fork.v1"]);
    }
    if matches!(agent, "codex" | "claude_code") {
        ids.extend([
            "agent_api.control.cancel.v1",
            "agent_api.tools.structured.v1",
            "agent_api.tools.results.v1",
            "agent_api.artifacts.final_text.v1",
            "agent_api.session.handle.v1",
            "agent_api.exec.add_dirs.v1",
            "agent_api.exec.non_interactive",
        ]);
        let list = if agent == "codex" {
            os == "linux" && arch == "x86_64"
        } else {
            matches!(
                (os, arch),
                ("linux", "x86_64") | ("macos", "aarch64") | ("windows", "x86_64")
            )
        };
        let get = if agent == "codex" {
            list
        } else {
            os == "windows" && arch == "x86_64"
        };
        if list {
            ids.insert("agent_api.tools.mcp.list.v1");
        }
        if get {
            ids.insert("agent_api.tools.mcp.get.v1");
            if write {
                ids.extend([
                    "agent_api.tools.mcp.add.v1",
                    "agent_api.tools.mcp.remove.v1",
                ]);
            }
        }
        if sandbox {
            ids.insert("agent_api.exec.external_sandbox.v1");
        }
    }
    match agent {
        "codex" => ids.extend([
            "backend.codex.exec_stream",
            "backend.codex.exec.approval_policy",
            "backend.codex.exec.sandbox_mode",
        ]),
        "claude_code" => {
            ids.insert("backend.claude_code.print_stream_json");
        }
        "aider" | "gemini_cli" | "opencode" => {}
        _ => panic!("unknown baseline backend {agent}"),
    }
    ids.into_iter().map(str::to_owned).collect()
}
