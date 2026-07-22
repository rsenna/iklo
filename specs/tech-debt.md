# Technical Debt

Tracks review findings that are valid but were deliberately deferred rather
than fixed immediately — not a bug tracker (use GitHub Issues for those).
An entry here means: someone reviewed it, agreed it's a real improvement,
and chose not to block the PR on it.

When an item here is picked up and fixed, move it to "Resolved" with the PR/commit
that closed it, rather than deleting the row — keeps the history of what was
deferred and for how long.

## Open

| ID | Source | File:Line | Description | Why deferred |
|----|--------|-----------|--------------|---------------|
| TD-001 | PR [#27](https://github.com/rsenna/iklo/pull/27) ([gemini-code-assist](https://github.com/rsenna/iklo/pull/27#discussion_r3623691492)) | `crates/iklo-substrate-turso/src/codec.rs:77` | Simplify `i64`'s `Codec::decode` using Rust slice pattern matching (`match bytes { [CODEC_VERSION_I64, payload @ ..] => ... }`) instead of manual `.first()`/`.get(1..)`/`.unwrap_or(&[])` calls. | Current implementation is correct, panic-safety-verified, and fully tested (see epic 004's Phase 2 review). This is a readability/idiom improvement, not a correctness fix — safe to defer. |
| TD-002 | PR [#27](https://github.com/rsenna/iklo/pull/27) ([codacy-production](https://github.com/rsenna/iklo/pull/27#discussion_r3623701418)) | `crates/iklo-substrate-turso/src/schema.rs:85` | Wrap `schema::bootstrap()`'s DDL + seed-row statements in a single SQL transaction instead of sequential individual statements. | Current idempotent `CREATE TABLE IF NOT EXISTS` + `INSERT OR IGNORE` sequence is safe for the single-writer, local-file-only connectivity this epic ships (see [B001](004-turso-substrate-backend/tasks.md) scoping decision). Transaction-wrapping matters more once/if remote or multi-writer connectivity is ever added — revisit then. |
| TD-003 | PR [#27](https://github.com/rsenna/iklo/pull/27) ([cubic-dev-ai](https://github.com/rsenna/iklo/pull/27#discussion_r3624295872)) | `crates/iklo-substrate-turso/src/lib.rs:235` | `RetryPolicy::backoff_for`'s exponential growth plateaus earlier than the intended 30s cap for extremely small (sub-~7ns) `base_backoff` values, because the `u32` doubling multiplier saturates at `u32::MAX` once `exponent > 31`, regardless of how small `base_backoff` is. | Real retry policies configure `base_backoff` at millisecond scale (this epic's `COMMIT_RETRY_POLICY` does); a sub-7-nanosecond base is not a realistic configuration here, so the practical impact is negligible. Worth a cap-aware fix (`u128` nanosecond arithmetic, or checked `Duration` multiplication) if `RetryPolicy` is ever exposed to configurable/untrusted `base_backoff` values. |

## Resolved

_(none yet)_
