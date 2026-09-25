/// Refuse a write-side lifecycle command that is running inside a relay host session.
pub fn refuse_nested_invocation(command: &str, host_run_id: Option<&str>) -> Result<(), String> {
    let Some(host_run_id) = host_run_id
        .map(str::trim)
        .filter(|run_id| !run_id.is_empty())
    else {
        // An empty environment variable does not identify a relay session, so treat it as absent.
        return Ok(());
    };

    Err(format!(
        "`{command}` cannot run because this process is running inside the \
         `execute-agent-maintenance` session for run `{host_run_id}`. The hosted agent is the \
         executor, so re-entering the lifecycle tooling would regenerate or re-execute the packet \
         it is working in. Lifecycle queries remain available, including \
         `maintenance-audit-status` and `maintenance-stand-down-check`. \
         `XTASK_AGENT_MAINTENANCE_RUN_ID` identifies the enclosing relay session; a maintainer \
         working outside a relay session will not have it set, so its presence means this process \
         is inside one. Clearing it from inside a relay session is the nesting failure this \
         refusal exists to prevent."
    ))
}

#[cfg(test)]
mod tests {
    use super::refuse_nested_invocation;

    #[test]
    fn absent_host_run_id_allows_invocation() {
        assert_eq!(refuse_nested_invocation("refresh-agent", None), Ok(()));
    }

    #[test]
    fn empty_or_whitespace_host_run_id_allows_invocation() {
        assert_eq!(refuse_nested_invocation("refresh-agent", Some("")), Ok(()));
        assert_eq!(
            refuse_nested_invocation("refresh-agent", Some(" \t\n")),
            Ok(())
        );
    }

    #[test]
    fn real_host_run_id_refuses_invocation_with_actionable_message() {
        let message = refuse_nested_invocation(
            "execute-agent-maintenance",
            Some("codex-0.156.1-run-20260925"),
        )
        .expect_err("nested lifecycle invocation must be refused");

        for expected in [
            "execute-agent-maintenance",
            "codex-0.156.1-run-20260925",
            "XTASK_AGENT_MAINTENANCE_RUN_ID",
            "Lifecycle queries remain available",
            "a maintainer working outside a relay session will not have it set",
            "Clearing it from inside a relay session",
        ] {
            assert!(
                message.contains(expected),
                "refusal must contain `{expected}`: {message}"
            );
        }
        assert!(
            !message.contains("unset"),
            "refusal must not instruct a nested process to bypass the guard: {message}"
        );
    }
}
