#[path = "mcp_management_v1/capabilities.rs"]
mod capabilities;
#[cfg(feature = "codex")]
#[path = "mcp_management_v1/codex_failures.rs"]
mod codex_failures;
#[cfg(feature = "codex")]
#[path = "mcp_management_v1/codex_read_ops.rs"]
mod codex_read_ops;
#[cfg(feature = "codex")]
#[path = "mcp_management_v1/codex_write_ops.rs"]
mod codex_write_ops;
#[path = "mcp_management_v1/support.rs"]
mod support;
#[path = "mcp_management_v1/support_smoke.rs"]
mod support_smoke;
