# Portable native Codex results

This opt-in command runs Codex through the public UAA backend and exports the existing
[`Cases` format](../../specs/support-depth-evidence-v1.md#receipt). It replaces the ad-hoc
local smoke harness with repository-owned, credential-free test logic. It does not install
a producer profile, grant qualification or enable a maintenance writer.

## Run with your own environment

The caller supplies an installed Codex executable and a disposable Git project. Codex resolves
authentication and configuration normally. Optional `--codex-home` selects an existing home;
otherwise the SDK uses normal environment/default resolution. The command never reads or copies
auth files, logs in, edits configuration, injects an API key or records environment values.
The caller is responsible for a usable login or other native authentication. Nothing is tied to
one maintainer's directory, account or credential store.

```sh
mkdir -p target/native-results
cargo run --locked -p xtask --example codex_native_results -- \
  --live \
  --binary "$(command -v codex)" \
  --working-dir /path/to/disposable/git/project \
  --output-dir target/native-results/run-01
```

`--live` is required before any executable probe or SDK run. Calls consume the caller's normal
provider allowance. The caller's native configuration controls provider, model, service tier,
loaders and any native startup effects. `--model` is an optional ordinary SDK override, not a
new policy. This command is not an OS sandbox or a private-context qualification fixture.
Use an appropriate execution environment for your native configuration; it does not copy your
login into a repository fixture or require a special API project.

Use `--expected-version 0.156.1` when checking the existing pinned pilot. A mismatch refuses
before the first SDK run; a successful local run against 0.162.1 cannot silently satisfy that
version. Omit the option for an ordinary installed-version diagnostic; actual version is retained.
Use `--variant unattended` or `--variant interactive` to request only that six-case population.
Both are requested by default. External-sandbox bypass is never enabled.

The output parent must already exist under the source repository. The output directory must
be new and repository-relative, with no `.`, `..`, `.git`, empty or backslash path components.
Existing output is preserved: use a different directory for every attempt.

## What is checked

For each requested variant, in fixed order:

1. Initial completes with the exact `UAA_SMOKE_OK` reply and a consistent canonical session ID.
2. Resume-last keeps that initial ID.
3. Resume-id explicitly selects and keeps the initial ID.
4. Fork-last returns a distinct ID.
5. Fork-id explicitly selects the initial ID and returns another distinct ID.
6. A separate initial request is cancelled through UAA's public control handle after a valid
   session facet; the exact SDK cancelled outcome is required.

Each request has a 30-second deadline. Failed, missing or ambiguous seeds, wrong identities,
reply mismatches, unexpected tool items and failed cancellation stop all remaining requests.
No harness retry, model fallback, synthetic session or automatic approval answer is supplied.
The existing SDK owns its native process deadlines and cleanup. Native provider retry behavior
is unchanged, so twelve SDK requests do not bound HTTP requests or billing.

Managed adapters currently label all item-start/completion snapshots as tool events. The guard
looks at the actual finite item kind: `userMessage`, `agentMessage` and `reasoning` are not tool
execution. Other/missing item kinds trigger cancellation and a failed case. Cancellation is
reactive; a native effect may already have occurred. The prompt is not an execution boundary.
Use a fresh disposable project, because another native run using the same project/home could
race last-session selection. A mismatched ID fails instead of silently choosing a new seed.

## Files and maintenance consumption

The command creates two bounded files in the new directory:

- `observations.json` records actual runtime version, selected-executable and runner digests,
  observed Git commit/tree/cleanliness, fixed case names, validated session IDs, finite outcome
  fields and explicit proof limits. No model text, raw errors, native payloads, config values,
  credential bytes or credential-derived digests are retained. Native version output is accepted
  only as a bounded `codex-cli <semver>` response; other output is withheld.
- `cases.json` uses the existing `depth_evidence::Cases` type: requested population, actually
  executed case/outcome pairs, and a repository-relative path/SHA-256 reference to the exact
  observations bytes. Case executable identities bind the running collector. Unrun cases remain
  requested and absent from executed; they cannot disappear into a favorable subset.

Exit 0 means every requested diagnostic case passed and the selected executable's digest stayed
unchanged. Exit 1 retains a nonpassing/incomplete run. Exit 2 is an argument, setup or output
error; partial output is not a complete bundle. No file is staged or delivered automatically.

This supplies the case-result input a maintenance producer adapter can consume. It does **not**
manufacture an `EvidenceReceipt`, trusted execution, an adequacy mapping or an E result. The
observation always states `evidence_status: "unverified"` and null packet binding. Observed Git
identity is not a build attestation. The selected executable may be a launcher; its digest is
not the full transitive native/build/config/loader closure. Neither a successful exit, a staged
bundle nor an uploaded artifact clears those limits.

The [approved evidence trust boundary](../../specs/support-depth-evidence-v1.md#approved-initial-trust-boundary)
still recognizes only the compiled GitHub Actions profiles. Those profiles, production verdicts,
53 pilot template tuples, classification proposals and public-writer refusals remain unchanged.
Local results remain diagnostic. A later producer integration must independently bind source,
runtime/target, actual effective context, packet Event/P/O, complete case and attempt population,
build dependencies, independent adequacy and authenticated adoption. Reading this result format
cannot supply that authority. Credential handling belongs to the execution environment.

## Required CI

Unit and integration tests use synthetic observations and temporary fake executables. They cover
identity/order/cancellation failures, missing opt-in, wrong runtime version, population retention,
error withholding, output-reference binding and preservation of prior attempts. They run under
the existing all-target test gates with no login, provider request or new CI job.

Native execution remains opt-in for a maintainer or separately configured environment. Required
CI does not acquire a maintainer's ChatGPT login and never invokes this example with `--live`.
