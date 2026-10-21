# Built-in runtime capability advertising model

Status: Approved, preparatory revision 1 (2026-10-20).

This contract governs the restricted advertising implementation for the five registry-owned
built-ins: `aider`, `claude_code`, `codex`, `gemini_cli` and `opencode`. It uses RFC 2119 keywords.
The [capability schema](capabilities-schema-spec.md) owns capability semantics; the
[support-depth contract](../support-depth-contract.md) owns depth admission. This preparation
implements neither check 9 entry admission nor production depth enrollment.

## Authored inputs and runtime semantics

`crates/xtask/data/agent_registry.toml` MUST remain the only authored capability inventory.
`runtime-advertising-model` projects its declarations into the private generated Rust table
`crates/agent_api/src/runtime_advertising/data.rs`. All five agents MUST be represented,
independently of lifecycle publication eligibility and enabled features. Generation MUST NOT
import manifest command filters, publication targets or depth results into runtime advertising.

The runtime and tooling MUST compile the same private, standard-library-only
`crates/agent_api/src/runtime_advertising/evaluator.rs` source. Runtime evaluation MUST NOT read
a repository, parse manifests, invoke tooling or perform I/O. The public backend trait,
configuration APIs, defaults, registration and capability meanings are unchanged.

The result is the sorted set union of `always_on`, `backend_extensions`, matching `target_gated`
ids and matching `config_gated` ids. A gate without a host restriction matches every host.
A restricted host list is an OR of exact `(OS, architecture)` pairs; each pair requires both
values. A configuration condition is ANDed with the host condition. Alternative gates for the
same id MUST be unioned rather than overwritten. Empty restricted host lists and unsupported
configuration keys or target labels MUST fail validation.

Revision 1 interprets target labels as follows:

| Upstream label | Runtime OS | Runtime architecture |
| --- | --- | --- |
| `x86_64-unknown-linux-musl` | `linux` | `x86_64` |
| `linux-x64` | `linux` | `x86_64` |
| `darwin-arm64` | `macos` | `aarch64` |
| `win32-x64` | `windows` | `x86_64` |

Linux GNU and musl deliberately occupy the same runtime cell. The current implementation has
no environment/libc predicate. The host adapter MUST return `std::env::consts::OS` and `ARCH`
directly. Unconditional ids remain available on other hosts.

Codex and Claude MUST pass their actual `allow_mcp_write` and `allow_external_sandbox_exec`
fields directly. The supported default is derived `Default` on plain bool fields, both false.
Constructors MUST store the supplied configuration unchanged. The other three backends have
neither field and MUST pass the inactive configuration. A declaration requiring an absent
configuration input MUST fail rather than inventing an input or default.

## Source conformance

Finite probes alone do not establish that arbitrary Rust implements this model. Both revision
views MUST pass executable source conformance before their advertising is compared.

The checker MUST parse Rust syntax and require the supported normalized item forms. Comments,
documentation and whitespace are immaterial. Semantic attributes, alternate expression forms,
module paths, aliases or helpers on the guarded path MUST fail unless explicitly supported.
In revision 1, conformance requires:

- the whole shared evaluator and small host/module adapter match fixed supported normalized
  source fingerprints, independently of the generated `MODEL_REVISION = 1` literal;
- generated data match the deterministic registry projection after token normalization;
- the fixed crate-to-backend and crate-to-model module graph, with the same-name feature guards;
- each canonical backend type implement the canonical trait, use its own model in the single
  evaluator call, and expose its canonical runtime kind;
- canonical kind/capabilities/type imports and any referenced `AGENT_KIND` literal resolve to
  the expected items, binding Rust type, gateway key, registry id and model identity together;
- the complete guarded config/storage declarations, derives and constructor retain supported
  direct-field input/default semantics;
- relevant Cargo selection has no build script, target dependency selection, alternate library
  path, repository Cargo configuration or workspace dependency patch/replacement.

The checker MUST walk each view's reachable runtime crate module graph, including inline and
nested declarations and the fixed embedded runtime-support include. Checking only the parent
and delegate files is insufficient: Rust implementations can attach methods from another module.
Guarded config implementations and noncanonical backend implementations/producers, qualified or
aliased forms, and guarded type aliases MUST fail. Only the prescribed backend constructor,
backend trait and existing harness trait implementation sites are supported. Exact `cfg(test)`
items are excluded from this production graph; other host/feature branches are inspected.

Unsupported item macros, procedural attributes/derives, module paths, namespace rebinding and
expansion MUST fail on that graph. Revision 1 supports ordinary built-in attributes/derives and
the existing fully qualified serde/thiserror derives. Expression macros are restricted to the
recognized built-ins and existing Tokio/serde_json forms; opaque macro arguments MUST NOT hide
declarations or unsupported nested expansion. New accepted expansion forms require an explicit
conformance extension. Relevant files in this graph are read independently from each revision.

The checker extracts guarded items from larger backend files so unrelated backend method
changes do not require advertising-template changes. Unsupported source MUST be reported as a
validation failure, never an empty advertised set. Extending these accepted forms requires a
reviewed semantics/conformance change; changing the model-revision literal is insufficient.

This is within the repository/compiler/review trust boundary. It does not authenticate actors
or verify the compiler, and cannot protect against deletion of its own integration gate.
Third-party implementations of the public backend trait are outside this built-in model.

## Feature and comparison domain

The `agent_api` manifest MUST contain exactly `default` and the five backend feature keys.
Extra aliases, including empty aliases, missing keys and unknown references are unsupported.
Unsupported optional dependencies MUST also fail, since Cargo creates implicit public features
for optional dependencies even when no explicit feature key is authored.
Supported lists contain references among those five features and explicit activation of their
actual optional dependencies. Each backend feature MUST directly activate its matching wrapper,
`tokio` and `futures-util`. Cyclic feature references MUST fail.

Each view resolves its own feature definitions. Comparison MUST consider all 32 subsets of
backend feature requests, with default features on and off (64 caller contexts). A backend
whose feature is disabled contributes the empty set. A changed default or feature closure can
therefore add advertising even when the capability id occurs in both registries.

The host partition MUST use every OS and architecture literal in both supported views, plus
an OTHER class for each dimension, in a Cartesian product. The current predicates produce
12 cells. Expressions are constant throughout each cell by the restricted grammar, rather
than by empirical sampling. Evaluate all four explicit boolean assignments and an additional
user-default configuration context. The default context is interpreted independently in each
view; revision 1 supports only the checked false/false default.

`Revision::additions` returns candidate-minus-tip ids with their caller contexts. It MUST
validate both complete views before narrowing the result. This is preparatory tooling only:
version/target ownership, destination attribution and acceptance entries remain check 9 work.
It MUST NOT infer those facts from `latest_validated`.

## Generation and immutable reads

```bash
cargo run -p xtask -- runtime-advertising-model
cargo run -p xtask -- runtime-advertising-model --check
```

Default mode MUST validate source/input conformance before writing, then deterministically
regenerate only `data.rs`. It MUST NOT repair semantic source/default/feature failures.
`--check` MUST write nothing and validate both data freshness and source/input conformance.
Exit status is 0 for success, 2 for a validation mismatch and 1 for an operational failure;
existing CLI usage-error behavior is unchanged.

The reader supports workspace files or an immutable Git commit, resolving the commit once and
reading that tree's own inputs. It MUST NOT check out, execute or fetch baseline code. Relevant
files and ancestors MUST have supported regular-file/directory kinds; symlinks, unavailable
objects, invalid revisions, malformed inputs and missing/unsupported models MUST fail. Optional
input absence MUST NOT mask a symlink ancestor. A pre-bridge tree has no supported model and
MUST fail rather than imply no advertising.

The read-only check MUST run in the existing capability-matrix preflight/CI gate alongside the
unchanged publication freshness and semantic audit checks. The light staging push route still
requires a future integrated check 9 reader when relevant depth records exist.

## Bridge evidence

The behavior-preserving bridge is pinned to commit
`2992389f8cbdbb5e9f7bc1e107f5dc4ee773e810`. Its independent complete-set oracle and source-level
host/default/feature proof are recorded in
[the bridge evidence](../../agents/lifecycle/support-depth-runtime-advertising-bridge.md).
Native execution, cross-builds and recognized-source proof MUST be distinguished. A missing
native host MUST NOT be reported as a native pass. Later feature or predicate extensions need
expanded comparison/proof coverage before the reader accepts them.
