# Packet intake: exact proposed effect classifications

Status: proposed for maintainer decision PI-P1; no policy is installed by this appendix.
Baseline: staging `43a42bbcbc51ce2d21e5144a091d2bde5bac760f` (PR #298).
Read with the [intake design](support-depth-packet-intake-design.md). The registry remains the
sole executable classification inventory; this is its proposed review table, not a second loader.

## Population and meaning

Scope is Codex0.156.1, `codex exec`, headless JSONL, acquired `x86_64-unknown-linux-musl`.
The present intake's owning predicate includes every surface at that exact command path plus
effective global flags. Reconcile union + all original shards + every wrapper claim, preserving
name identities and target applicability. There are **54 identities**: one command, one positional
argument, 27 exec flags and 25 global flags. All observed rows occur on musl; `--full-auto` is a
wrapper-only claim. Reports are validated inputs, not another population. Nested exec subcommands
and root PROMPT are not silently assigned to this exact-path operation; adapter-flow completeness
is a separate unmet claim in PI-P3. Nothing here changes name-level exclusions or coverage levels.

`CORE` below is an explicit proposed assignment, not inheritance based on a flag's spelling.
BASE always applies. `CORE+ADMIN` explicitly adds OUT1, LC1 and FX1 for file/Git mutation.
`CORE-denial` retains the unit and requires a version-bound denied/no-effect path; it does not
qualify remote execution. `U-*` is an exact unresolved disposition: retain classification_required.
The 30 explicit assignments comprise25CORE,3CORE+ADMIN and2CORE-denial;24remain unresolved.
No assignment is `verified`, no forwarded argument proves a promise, and no live/unsafe invocation
is authorized. Independent adequacy and effect evidence are required after policy approval.

Aliases below are recorded short aliases from the pinned snapshots. A dash means no recorded
short alias. Value labels are placeholders, not acquired enumerations or defaults. Except source-
corroborated booleans and color, the help data does not establish complete accepted/default values.
Absent flags still require the actual effective config/default regime to be bound and tested.
Repeated root/local names remain distinct SurfaceIdentity values; do not deduplicate them away.
“Unclaimed” is the absence of a literal wrapper row at that exact path, not a verdict about an
effective flag covered by its separately listed global claim.

## Every selected identity

| # | surface_kind | command_path | surface_id | Recorded alias / arity | Wrapper claim | Proposed assignment | Exact reason |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `commands` | `codex exec` | `exec` | — / command | explicit | **CORE** | New noninteractive exec operation, OUT3/LC2; no claim covering resume/fork/review subcommands. |
| 2 | `flags` | `codex exec` | `--add-dir` | — / value DIR | unclaimed | **CORE** | Writable-directory scope: explicit BASE X2/X4 and validation; no separate administrative operation promised. |
| 3 | `flags` | `codex exec` | `--approve-for-me` | — / presence | unclaimed | **CORE** | Explicit auto-review approval regime under BASE G1/X2/X4; interaction adequacy remains required. |
| 4 | `flags` | `codex exec` | `--cd` | -C / value DIR | unclaimed | **CORE** | Working-root precedence under BASE G1/X2; neither a new session selector nor separate filesystem mutation. |
| 5 | `flags` | `codex exec` | `--color` | — / value COLOR | explicit | **CORE** | Finite auto/always/never diagnostic formatting; preserve JSONL OUT3 and BASE X3 bounds. |
| 6 | `flags` | `codex exec` | `--config` | -c / value key=value | unclaimed | **U-config** | Arbitrary keys/values can change effects; no finite reviewed key/value regime. |
| 7 | `flags` | `codex exec` | `--dangerously-bypass-approvals-and-sandbox` | — / presence | unclaimed | **CORE** | Explicit bypass regime under BASE G1/X2/X4; no host authorization or sandbox claim. |
| 8 | `flags` | `codex exec` | `--dangerously-bypass-hook-trust` | — / presence | unclaimed | **U-hooks** | Trusted hook sources and their effects are not a finite reviewed population. |
| 9 | `flags` | `codex exec` | `--disable` | — / value FEATURE | unclaimed | **U-feature** | Feature removal also changes effects; no per-feature classification. |
| 10 | `flags` | `codex exec` | `--enable` | — / value FEATURE | unclaimed | **U-feature** | Feature names/defaults can introduce persistence/protocol effects; no per-feature classification. |
| 11 | `flags` | `codex exec` | `--ephemeral` | — / presence | explicit | **CORE** | Explicit persistence opt-out under BASE X2; initial-session identity remains OUT3, no resume/fork selector added. |
| 12 | `flags` | `codex exec` | `--help` | -h / presence | unclaimed | **U-terminal** | Help terminates without the promised event run; a disjoint diagnostic/guard contract is needed. |
| 13 | `flags` | `codex exec` | `--ignore-rules` | — / presence | explicit | **CORE** | Explicit policy-file loading regime under BASE G1/X2/X4. |
| 14 | `flags` | `codex exec` | `--ignore-user-config` | — / presence | explicit | **CORE** | Explicit config-loading regime under BASE X1/X2; auth still uses CODEX_HOME. |
| 15 | `flags` | `codex exec` | `--image` | -i / value FILE | unclaimed | **CORE** | Image-file inputs and ordering under BASE R1/G1/X2; no output-record schema promised. |
| 16 | `flags` | `codex exec` | `--json` | — / presence | explicit | **CORE** | Presence selects the JSONL streaming run: OUT3 and LC2, not arbitrary JSON decoding. |
| 17 | `flags` | `codex exec` | `--local-provider` | — / value OSS_PROVIDER | unclaimed | **U-provider** | Provider/default selection can prompt or select another backend; exact regime unreviewed. |
| 18 | `flags` | `codex exec` | `--model` | -m / value MODEL | unclaimed | **CORE** | Model identifier/open-value routing under BASE R1/G1/A1; no blanket provider/config qualification. |
| 19 | `flags` | `codex exec` | `--oss` | — / presence | unclaimed | **U-provider** | Provider/default selection can prompt; exact provider/config regime unreviewed. |
| 20 | `flags` | `codex exec` | `--output-last-message` | -o / value FILE | explicit | **CORE+ADMIN** | Explicit output-file mutation adds OUT1/LC1/FX1; wrapper also supplies an implicit temporary output path. |
| 21 | `flags` | `codex exec` | `--output-schema` | — / value FILE | explicit | **CORE** | Read schema file and constrain final response content; wrapper does not promise a generic parsed-record API. |
| 22 | `flags` | `codex exec` | `--profile` | -p / value CONFIG_PROFILE_V2 | unclaimed | **U-profile** | External profile contents can change effects; no bound profile content. |
| 23 | `flags` | `codex exec` | `--sandbox` | -s / value SANDBOX_MODE | unclaimed | **CORE** | Explicit sandbox-policy selection under BASE G1/X2/X4; wrapper is not an OS sandbox. |
| 24 | `flags` | `codex exec` | `--skip-git-repo-check` | — / presence | explicit | **CORE** | Repository precondition override under BASE V1/G1/X2. |
| 25 | `flags` | `codex exec` | `--strict-config` | — / presence | unclaimed | **CORE** | Explicit unknown-config rejection regime under BASE V1/G1/E1; not schema approval for arbitrary config values. |
| 26 | `flags` | `codex exec` | `--thread-source` | — / value SOURCE | passthrough | **U-source** | SOURCE enum and attribution/trust implications are not acquired as value facts. |
| 27 | `flags` | `codex exec` | `--version` | -V / presence | unclaimed | **U-terminal** | Version action terminates without the promised event run; not implicitly CORE. |
| 28 | `flags` | `codex exec` | `--worktree` | — / presence | unclaimed | **CORE+ADMIN** | Managed Git checkout creation/binding adds OUT1/LC1/FX1; conflicts and partial effects require proof. |
| 29 | `global_flags` | `codex` | `--add-dir` | — / value DIR | explicit | **CORE** | Writable-directory scope: explicit BASE X2/X4 and validation; no separate administrative operation promised. |
| 30 | `global_flags` | `codex` | `--approve-for-me` | — / presence | passthrough | **CORE** | Explicit auto-review approval regime under BASE G1/X2/X4; interaction adequacy remains required. |
| 31 | `global_flags` | `codex` | `--ask-for-approval` | -a / value APPROVAL_POLICY | explicit | **U-root** | Root approval option is outside SharedCliOptions inheritance; promised headless effect needs corroboration. |
| 32 | `global_flags` | `codex` | `--cd` | -C / value DIR | explicit | **CORE** | Working-root precedence under BASE G1/X2; neither a new session selector nor separate filesystem mutation. |
| 33 | `global_flags` | `codex` | `--config` | -c / value key=value | passthrough | **U-config** | Arbitrary keys/values can change effects; no finite reviewed key/value regime. |
| 34 | `global_flags` | `codex` | `--dangerously-bypass-approvals-and-sandbox` | — / presence | explicit | **CORE** | Explicit bypass regime under BASE G1/X2/X4; no host authorization or sandbox claim. |
| 35 | `global_flags` | `codex` | `--dangerously-bypass-hook-trust` | — / presence | passthrough | **U-hooks** | Trusted hook sources and their effects are not a finite reviewed population. |
| 36 | `global_flags` | `codex` | `--disable` | — / value FEATURE | passthrough | **U-feature** | Feature removal also changes effects; no per-feature classification. |
| 37 | `global_flags` | `codex` | `--enable` | — / value FEATURE | passthrough | **U-feature** | Feature names/defaults can introduce persistence/protocol effects; no per-feature classification. |
| 38 | `global_flags` | `codex` | `--full-auto` | — / unobserved | explicit | **U-claim** | Wrapper-only claim absent from every pinned snapshot; reconcile before classifying. |
| 39 | `global_flags` | `codex` | `--help` | -h / presence | explicit | **U-terminal** | Help terminates without the promised event run; a disjoint diagnostic/guard contract is needed. |
| 40 | `global_flags` | `codex` | `--image` | -i / value FILE | explicit | **CORE** | Image-file inputs and ordering under BASE R1/G1/X2; no output-record schema promised. |
| 41 | `global_flags` | `codex` | `--local-provider` | — / value OSS_PROVIDER | explicit | **U-provider** | Provider/default selection can prompt or select another backend; exact regime unreviewed. |
| 42 | `global_flags` | `codex` | `--model` | -m / value MODEL | explicit | **CORE** | Model identifier/open-value routing under BASE R1/G1/A1; no blanket provider/config qualification. |
| 43 | `global_flags` | `codex` | `--no-alt-screen` | — / presence | unclaimed | **U-root** | TUI-root control is not propagated by exec SharedCliOptions; no headless exclusion inferred. |
| 44 | `global_flags` | `codex` | `--no-daemon` | — / presence | passthrough | **U-root** | Root-only control is outside exec SharedCliOptions inheritance; headless effect unproved. |
| 45 | `global_flags` | `codex` | `--oss` | — / presence | explicit | **U-provider** | Provider/default selection can prompt; exact provider/config regime unreviewed. |
| 46 | `global_flags` | `codex` | `--profile` | -p / value CONFIG_PROFILE_V2 | explicit | **U-profile** | External profile contents can change effects; no bound profile content. |
| 47 | `global_flags` | `codex` | `--remote` | — / value ADDR | explicit | **CORE-denial** | Pinned root dispatcher rejects remote mode for exec; verify denied/no-effect, not a REMOTE run. |
| 48 | `global_flags` | `codex` | `--remote-auth-token-env` | — / value ENV_VAR | explicit | **CORE-denial** | Pinned root dispatcher rejects remote-auth intent for exec; verify credential-safe denied/no-effect. |
| 49 | `global_flags` | `codex` | `--sandbox` | -s / value SANDBOX_MODE | explicit | **CORE** | Explicit sandbox-policy selection under BASE G1/X2/X4; wrapper is not an OS sandbox. |
| 50 | `global_flags` | `codex` | `--search` | — / presence | explicit | **U-root** | Root-only search control is not propagated by the exec arm; headless effect unproved. |
| 51 | `global_flags` | `codex` | `--strict-config` | — / presence | passthrough | **CORE** | Explicit unknown-config rejection regime under BASE V1/G1/E1; not schema approval for arbitrary config values. |
| 52 | `global_flags` | `codex` | `--version` | -V / presence | explicit | **U-terminal** | Version action terminates without the promised event run; not implicitly CORE. |
| 53 | `global_flags` | `codex` | `--worktree` | — / presence | passthrough | **CORE+ADMIN** | Managed Git checkout creation/binding adds OUT1/LC1/FX1; conflicts and partial effects require proof. |
| 54 | `positional_args` | `codex exec` | `PROMPT` | — / positional | explicit | **CORE** | Prompt/stdin and appended stdin block under BASE R1/G1/X4; no new template. |

## Source and boundary notes

The committed [musl shard](../../../cli_manifests/codex/snapshots/0.156.1/x86_64-unknown-linux-musl.json),
[union](../../../cli_manifests/codex/snapshots/0.156.1/union.json),
[wrapper coverage](../../../cli_manifests/codex/wrapper_coverage.json) and
[RULES](../../../cli_manifests/codex/RULES.json) establish names, arity, short aliases and coverage
claims. `available_on` is applicability, never execution evidence. All14 acquired inputs are below.
The binary's recorded SHA-256 is
`0b2e9301d6100dddda3b9d5c80ebaeaa3a2f1962388f2f36f6b96a9f08b1f33f`.

Effect proposals are source-informed. Official annotated tag `rust-v0.156.1` resolves to commit
`b412ff32c417f855c2b2d1581b77058eed87c84b`; this corroborates intent and is not a binary-build
attestation or live behavior proof. Relevant pinned primary sources:

- [Exec CLI](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/exec/src/cli.rs): JSONL, output paths/schema, prompt/stdin, persistence/config controls, color and nested command selectors.
- [Shared options](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/utils/cli/src/shared_options.rs): explicit root/local precedence, writable directories, provider/default and safety regimes.
- [Root dispatcher](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/cli/src/main.rs): exec remote refusal, feature-to-config conversion and the exact root inheritance call. UI-only fields outside that call do not prove a headless effect.
- [Exec implementation](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/exec/src/lib.rs): worktree mutation/conflicts, new/resume/fork paths and output/persistence regimes.

The snapshot misses hidden source aliases: `--experimental-json` for `--json`, `--not-so-yolo`
for `--approve-for-me` and `--yolo` for bypass. These are **source-only corroboration** here,
not acquired alias facts. First intake supports only aliases present coherently in its bound
acquisition; these hidden spellings refuse until a reviewed acquisition extension captures them.
No new alias input artifact is proposed for this first increment. `--json` has no short alias.

Wrapper grounding: [override argv](../../../crates/codex/src/builder/cli_overrides.rs) supplies root
flags before `exec`; [streaming](../../../crates/codex/src/exec/streaming.rs) always supplies JSON,
skip-repository-check, color and a last-message output path, using a temporary path when none is
provided. Thus mutation is not merely a hypothetical user-supplied output option. ADMIN is a
proposed conservative promise addition requiring maintainer approval, not a claim that existing
CORE evidence already proves it. `--output-schema` is input shaping; the wrapper does not thereby
claim a QUERY parsed-record interface. Initial persistence/ephemeral behavior alone adds no
session selector; actual resume/fork does require SESSION review.

U-config/U-feature/U-profile cannot be resolved by approving arbitrary strings. U-provider keeps
the exact noninteractive provider/config regime open for review. U-hooks requires a reviewed
hook boundary. U-source lacks acquired SOURCE value semantics. U-terminal needs separate
help/version mode or pre-spawn rejection behavior; CORE's event promise is not waived. U-root
requires the actual headless request/precedence/denial behavior to be reconciled. U-claim must be
reconciled against actual upstream behavior, wrapper compatibility and truthful reporting; absence
from help alone does not prove a hidden flag is impossible. None is removed or marked EXCLUDED.

## Exact acquired input bytes reviewed

These are the14 inputs the current derive path references. A later acquisition change requires
fresh classification-basis review; these rows are not an installer or runtime allowlist.

| Path | SHA-256 |
| --- | --- |
| `cli_manifests/codex/RULES.json` | `c12e256023cc452ae7aa758f67b48d9862e7becb6681876de6a8eddb985cb9d8` |
| `cli_manifests/codex/SCHEMA.json` | `96f1afe9e3388686c7c9519a9234e8d01411a2d72e7daa072ffef6deeccc74d5` |
| `cli_manifests/codex/wrapper_coverage.json` | `6cd7ef079f893eb4a3745bcd948fbae432fe025568f156bda267dcbdde881e70` |
| `cli_manifests/codex/snapshots/0.156.1/aarch64-apple-darwin.json` | `a43fff11fc62a2786e55fbd635625c95d8a4875240ad43edb50e9947eefda594` |
| `cli_manifests/codex/snapshots/0.156.1/aarch64-unknown-linux-musl.json` | `3b06cf8f5129a8cf3fbb44a17ad2c86560c99f1875abac52fd0654c0a52f88b6` |
| `cli_manifests/codex/snapshots/0.156.1/union.json` | `fb82e3d34f2f6bbe6886defaea1cb624c5b9c284255aa7bac552250e732d4006` |
| `cli_manifests/codex/snapshots/0.156.1/x86_64-pc-windows-msvc.json` | `708cb0657b6b01a9d28e2c34666656eb9faf7f52e56eecc29eb585ef423b8577` |
| `cli_manifests/codex/snapshots/0.156.1/x86_64-unknown-linux-musl.json` | `dfaac28c036883b523bf156e17e79bcdb76ca39259a06b4f0da35a2d063f894b` |
| `cli_manifests/codex/reports/0.156.1/coverage.aarch64-apple-darwin.json` | `7fb6a6ce0a27b9d58a4a01c1851dc6bdb00b92a487807946b7e04e74ef43cb6d` |
| `cli_manifests/codex/reports/0.156.1/coverage.aarch64-unknown-linux-musl.json` | `2d10c3de71991ebe7b244b63c732237b3dbe66026f5c7746bd911d0212e4c491` |
| `cli_manifests/codex/reports/0.156.1/coverage.all.json` | `1fca4c05dddafb15b349dbcbbea1cc87f0478c72be2a42ac84a12171c7c9760d` |
| `cli_manifests/codex/reports/0.156.1/coverage.any.json` | `6855143fb19ba2fac771248b02325ae12a03be7b6163f9bcb9bf879344466d15` |
| `cli_manifests/codex/reports/0.156.1/coverage.x86_64-pc-windows-msvc.json` | `5aa368afdc4cb2c2519c6366d1fc59635b8c35cdac68b0296cf35c407866e2f8` |
| `cli_manifests/codex/reports/0.156.1/coverage.x86_64-unknown-linux-musl.json` | `9bf777fe1bca1bd620663959c339754e4ba4473e73196d20dcd780dc7c67fd4b` |
