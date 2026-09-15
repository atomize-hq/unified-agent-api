# Claude Code Working Agreement

Read and follow [AGENTS.md](AGENTS.md) first. It is the shared repository
policy for both Claude Code and Codex; do not duplicate or override it here.

## Worker orchestration

Use `$orchestrate-workers` when work needs delegation and `$cross-review` for
independent review of a candidate change. The lead session defines the plan,
direction, acceptance criteria, and task boundaries; it also owns integration
and final validation. Delegate only bounded packets with explicit write
ownership.

Use `codex-profile-worker` for documentation, implementation, and Codex review
work. Use `opus-adversarial-reviewer` only as a read-only adversarial lane,
dispatched in parallel with a Codex review—not as an implementation worker.
Codex runs on its default configuration unless a packet names a profile; the
repository launcher always passes an explicit sandbox and stops Codex after a
timeout.
