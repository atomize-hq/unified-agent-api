# Claude Code 2.1.274 wrapper-only evidence

This packet adjudicates the four wrapper-only identities present in the acquisition report at commit `8053bb3b3a2cc71426948f941f8f6830d43c2914`. The categories and identity rows live in `maintenance-closeout.json`; this note records their evidence and its limits. The baseline is `cli_manifests/claude_code/reports/2.1.274/coverage.any.json`, selected before the three withdrawn claims were removed from publication.

## Historical root flag

The committed `cli_manifests/claude_code/snapshots/2.1.29/union.json` lists root flag `--mcp-debug` on all three targets. The 2.1.274 union does not list it. Retain its wrapper declaration for historical compatibility, categorized as `older_upstream_only`, with 2.1.29 as the last retained version with supporting evidence. This does not identify the exact intervening release that removed it or prove current hidden support.

## Withdrawn plugin claims

The 2.1.274 union does not contain `plugin manifest`, `plugin manifest marketplace`, or `plugin marketplace repo`. The 2.1.29 union contains path records for these identities, but the first two show usage `claude plugin [options] [command]`, and the third shows `claude plugin marketplace [options] [command]`. That is parent-command help, not affirmative evidence for the requested command paths. These snapshots do not establish a supported historical version for those commands.

The three publication claims are withdrawn from the candidate while their legacy public request types and client methods remain for compatibility. They are categorized as `unsubstantiated_wrapper_claim`: the repository-backed evidence did not establish the asserted upstream surfaces, and the wrapper coverage publication was removed. This does not assert that upstream ever supported the paths, that it removed them, or that the wrapper's public API was removed.

## Pinned artifact checks

Additional read-only checks used the darwin-arm64 artifacts already pinned in `cli_manifests/claude_code/artifacts.lock.json`:

| Version | Download artifact SHA-256 | Extracted executable SHA-256 |
| --- | --- | --- |
| 2.1.29 | `283e85de5aaabc707b366ef2b52e544a41480210b04b2bc9706bc8c7fe7623ba` | same artifact, bare executable |
| 2.1.274 | `fac64a3570d084fa255199b8ac4a9541f292a773036c7e9dbd87c64265ff1efb` | `3509913f9d1576316c8845b88837f8fd3bbbcf26625833ac82cfb6b8985da94a` |

Both `--version` outputs matched their pins. For each binary, `plugin manifest --help` and `plugin manifest marketplace --help` returned byte-identical output to `plugin --help`; `plugin marketplace repo --help` returned byte-identical output to `plugin marketplace --help`. All exited zero. Thus a successful help probe alone is not affirmative evidence for these paths.

Inspection of the embedded plugin command-registration sections found the ordinary plugin and marketplace children, but no literal `manifest` or `repo` command registrations. This corroborates the help-fallback finding; it does not prove what every earlier release shipped. The wrapper request types were introduced in the 2.1.29 coverage work at `5b21d64b78eb739e7e370e3ae59ceaaf30cedca2`.

The repository-backed observations do not support current hidden support, historical support, upstream removal, or a missing union surface for these three claims. The `unsubstantiated_wrapper_claim` category records that limited conclusion and requires the completed publication withdrawal; it adds no support debt or follow-on obligation.

## Verification boundary

No authenticated Claude inference was used. Evidence comes from the committed snapshots, the wrapper declarations, and regenerated reports. Argument-forwarding tests use a local fake executable. The acquisition baseline had four wrapper-only identities: one `older_upstream_only` root flag and three withdrawn `unsubstantiated_wrapper_claim` plugin paths. Prepare carries the latter withdrawal adjudications forward after regeneration, even though the live report no longer lists them.
