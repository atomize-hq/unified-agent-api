# FR2 canonical output expectations

These are complete reviewed output expectations for the fixed synthetic MA2 and ON2 connected
fixtures. They are test inputs. Tests never write or refresh them, and never obtain expected
commands, packet prose, contract fields or reporting rows from a planner or writer renderer.
`inventory.json` explicitly enumerates every expected output in each lane.

Review the full HANDOFF and prompt, request execution contract, recovery and closeout instructions,
all seven ON2 packet files, lifecycle/record fields, reporting rows and Markdown together when
changing these files. An appended command or modified support Markdown must fail these tests even
when the writer and admission planner share the same renderer regression.

The two lanes share literal publication text and runtime projection. The single
`{{DEPTH_FIXTURE_ROWS}}` slot in each publication template expands only to the lane's literal
`.rows` file. Every reconstructed output byte, including whitespace, is compared. Git mode and
object kind are also asserted. The fixed inputs name version `0.2.0`, maintenance event
`abcdef0` / `2026-11-03T00:00:00Z`, and ON2 run `fr2-freeze` with the two distinct clocks
`2026-11-14T00:00:00Z` and `2026-11-14T00:00:01Z`.

The ON2 input contract is compared structurally over every field. Its `baseline: null` slot is
replaced by an independent enumeration of raw regular predecessor Git leaves and their SHA-256
values. This avoids embedding hundreds of unrelated file hashes, checks the complete snapshot,
and never uses the production snapshot function.

ON2's real producer embeds a binary hash and generated wrapper timestamp, and its approved Event
names an actual fixture Git commit. Their provenance slots are resolved from raw predecessor
approval, registry and acquired blobs. Expected P and O identities are SHA-256 values computed
independently over the complete literal expected parts after those slots are filled. All selection,
command, reporting and execution fields remain literal; no expected field comes from candidate
output or a production planner/identity function. MA2 uses a fixed union collection timestamp via
the ordinary union engine's explicit test clock.

ON2's packet files are local preparation artifacts. The connected test checks all seven against
these literals from the actual local writer output, admits ordinary staging under the repository's
real ignore policy, then force-adds the packet and verifies exact candidate bytes and modes.
