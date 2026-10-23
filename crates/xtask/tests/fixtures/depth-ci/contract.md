# Isolated channel-validation source contract, version 0.0.0

This synthetic `controlled_channel` operation exercises the existing
`wrapper_events::ValidatedChannelString::new` production boundary. It accepts a channel string
only if it is 1–64 ASCII bytes, starts with an ASCII letter or digit, and contains only ASCII
letters/digits or `.`, `_`, `/`, `-`. Successful validation preserves the exact value.

The isolated V1 obligation covers these input rules, including limits and negative cases.
Removing validation so empty, oversized, non-ASCII, invalid-prefix or invalid-character inputs
can succeed violates the assertions. The adequacy argument concerns substantive behavior;
redundant guards need not fail independently when another guard still enforces the same rule.

This is a collector/evaluator/publication proof fixture, not a Codex CLI grammar or release.
Its version 0.0.0 and isolated origin never qualify any production depth enrollment. The other
obligations of the synthetic frozen promise remain unverified. No acceptance entry is created.
