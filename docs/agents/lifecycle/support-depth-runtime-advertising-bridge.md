# Runtime advertising bridge evidence

Baseline: `2992389f8cbdbb5e9f7bc1e107f5dc4ee773e810` (slice 2a merged to staging).
This evidence accompanies [the normative model](../../specs/unified-agent-api/runtime-advertising-model.md).
It concerns advertising preservation only; it does not establish support-depth admission.

## Baseline correspondence

The pre-refactor capabilities bodies, constants, target helpers, config derives/constructors,
kind identities and feature/module edges were inspected at that immutable commit. These SHA-256
file digests identify those inputs independently of the branch name:

| Baseline file | SHA-256 |
| --- | --- |
| `crates/xtask/data/agent_registry.toml` | `86fc2fac89d86c67f9ef68cba07e4e2ce421ca9ae2a5fb6d944234274fcdacab` |
| `crates/agent_api/Cargo.toml` | `74cdcab38e6b1f73cd2f42311d7203ac71e4d4ebf75f67d9ad608939b2a3111f` |
| `crates/agent_api/src/backends/mod.rs` | `892199fb370f7f82eec38d8a55afa5b32b7a24f2aa3b74ca60ca32ebf6351770` |
| `crates/agent_api/src/lib.rs` | `dd4550fb91a0b06e33b2576b4066fa47c8e1c814728271dc0d9c9b1028e12422` |
| `crates/agent_api/src/backends/codex/mod.rs` | `2b47d8e00a08ab4b52ba627f14b1b119f68a31d4aa00f5c76ec101f6f6820e14` |
| `crates/agent_api/src/backends/codex/backend.rs` | `181a164dc36a08574ed9cd27c81cc5efc48619864fb60d8edecd57f004f958f5` |
| `crates/agent_api/src/backends/claude_code/mod.rs` | `e595d0273e33804c0687abc04120ef0890e3d7612754ea104329dfef9b6e3702` |
| `crates/agent_api/src/backends/claude_code/backend.rs` | `0196db53a311bf6f856c1075442f65be527a4814c4f46afef04a18a5b00e9c13` |
| `crates/agent_api/src/backends/aider/mod.rs` | `0069c24a8a497297dc80384dc96787ec10bcaeae56fcc20b228bfedf8fa16bba` |
| `crates/agent_api/src/backends/aider/backend.rs` | `c4a62704afc9bdc2cbc82ca506a3f8bb06cc53c9548d7d02909bb9ee1f1e3890` |
| `crates/agent_api/src/backends/gemini_cli/mod.rs` | `46fe993f0b426471eed9abb6f34c71ba8177f578570318f9e64353a697405fd4` |
| `crates/agent_api/src/backends/gemini_cli/backend.rs` | `42e39c7a4c85a8ebeba7ad1d3d333d57a048bf0d218bd4269e7f533986c7b6b9` |
| `crates/agent_api/src/backends/opencode/mod.rs` | `966805f45df1b2fe013c58f541ca32c6fe143b46ff99a8e08b7cbf8b48c3f239` |
| `crates/agent_api/src/backends/opencode/backend.rs` | `34e66025de308d5ee854e4d82ddd9872ea7c8369d5dc22b8ab9cd6ae0c9d5664` |

## Complete-set proof

`crates/agent_api/tests/runtime_advertising/expected.rs` is an independent literal oracle
transcribed from those old bodies and constants; it is test evidence, not a production inventory.
The old four common ids remain unconditional for every backend. OpenCode retains session
resume/fork. Codex and Claude retain their complete common, backend-specific, session, control
and execution ids. Exact set equality detects both removals and additions.

The old Codex MCP helper is `cfg!(all(target_os = "linux", target_arch = "x86_64"))`.
Its new label interpretation is exactly Linux/x86_64, including both GNU and musl.
Claude's old list predicate is the OR of Linux/x86_64, macOS/aarch64 and Windows/x86_64;
get and writes require Windows/x86_64. Their new host pairs are identical. Old write gates
read `allow_mcp_write`; the external sandbox id reads `allow_external_sandbox_exec`.
Both fields derive false defaults and constructors store the received config unchanged.
Other three configs have no advertising inputs. Canonical kind literals are preserved, and
runtime gateway registration resolves each backend under that same identity.

The old and new same-name feature guards select the same modules; defaults enable none.
The shared evaluator's only conditions are those exact host-pair equalities and booleans.
Thus the 4 OS classes (Linux, macOS, Windows, other) crossed with 3 architecture classes
(x86_64, aarch64, other) partition all hosts, and every predicate is constant within its cell.
This is recognized-source reasoning about all members, including GNU/musl, rather than an
assumption from a finite host sample.

The independent model test checks all five agents across these 12 cells, 64 requested-feature/
default-feature contexts, four explicit bool assignments and the user-default config context:
19,200 complete-set comparisons. Native bridge tests independently instantiate the actual
five backends for all four bool assignments, assert complete sets, false defaults and runtime
kind/gateway identities. Existing backend unit tests retain independent cfg-based target oracles.

## Platform evidence and limits

| Host | Evidence |
| --- | --- |
| Linux x86_64 GNU | Native bridge execution before and after conversion, complete sets and all bool assignments; native gateway assertions after conversion. |
| Linux x86_64 musl | Recognized-source equality of the old OS/arch cfg and new pair predicate; independent full-set cell tests. No native musl execution claimed. |
| macOS aarch64 | Recognized-source equality of old list/get/write cfg predicates and new pair predicates; independent full-set cell tests. No native execution claimed. |
| Windows x86_64 | Recognized-source equality of old list/get/write cfg predicates and new pair predicates; independent full-set cell tests. No native execution claimed. |
| Remaining host cells | Uniformity follows from the restricted equality grammar; independent complete-set tests for every cell. |

No cross-build or emulation is claimed. Native tests bind the real host adapter and real backend
config/storage to the model; constrained-source conformance and independent full-set tests cover
the remaining cells. Publication matrix freshness/audit remain separate checks and its committed
bytes are unchanged. The baseline itself has no revision-1 table, so immutable model reads of it
correctly fail as unsupported; future check 9 comparisons begin from a post-bridge baseline.

## Conformance regression evidence

The two independent implementation review lanes demonstrated compiling bypasses through
qualified/macro-generated backend constructors and an inherent config `default()` in an existing
descendant module. They also demonstrated Cargo's extra implicit feature from an optional
`serde` dependency. These are rejected by the reachable-module declaration/expansion guard and
optional-dependency validation. Regression fixtures cover parent, descendant, nested and newly
selected module declarations, aliases, opaque expansion and immutable old-tree corruption;
an unrelated method-body edit remains accepted. The existing normalization guard permits the
literal in the exact generated data file while retaining its raw-selection-access prohibition.

A separately compiled OpenCode-to-Aider kind mutation was rejected by source conformance. Its
independent runtime gateway probe resolved session resume/fork under Aider and found no OpenCode
registration, demonstrating why checked identity is necessary beyond table/set equality.
