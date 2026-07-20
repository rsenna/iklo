# ADR-0004 — REPL meta-commands move from `.` to `/`

- **Status:** Accepted
- **Date:** 2026-07
- **Deciders:** @rsenna (with Claude as implementer)
- **Supersedes:** —
- **Superseded by:** —

## Decision (one sentence)

**REPL meta-commands (`quit`, `revision`, `env`) move from a leading `.` to a leading `/` — `/quit`, `/revision`, `/env` — recognized only as the first character of a fresh (non-continuation) REPL input line and offered via tab-completion, implemented entirely inside `iklo-cli`'s REPL input loop with zero change to `iklo-lexer`, `iklo-parser`, or `iklo-ast`.**

## Context

`AGENTS.md`'s "Non-negotiable syntax rules" section states: *"REPL commands use a leading `.` (`.quit`, `.revision`, `.env`) to avoid colliding with `:name` and to keep `/paths` free for shell mode."* That same section says such rules "shouldn't be casually revisited. If a change is needed, open an ADR." Moving REPL commands to `/` sits inside that ADR-gated list, so this ADR exists even though — as the decision below makes precise — the actual mechanism never touches anything the "grammar changes... require an ADR" clause is really protecting against.

Two things prompted the change:

1. **Ergonomics.** `/`-prefixed commands are the near-universal convention for interactive command interfaces (Slack, Discord, Claude Code itself, among others) — familiar muscle memory that `.`-prefixed commands don't carry.
2. **Rustyline adoption** (see [`specs/003-repl-improvements/`](../003-repl-improvements/spec.md)), which brings real tab-completion to the REPL for the first time. Completion is a natural fit for `/`-triggered commands; today's `.`-commands only ever did exact-string dispatch on Enter, with no completion at all.

The apparent conflict with "keep `/paths` free for shell mode" dissolves once the actual mechanism is specified precisely:

- **Slash-commands are a REPL-input-loop feature, not an Iklo language construct.** They are recognized by `iklo-cli`'s own dispatcher and rustyline `Completer`, both gated on the same condition: `/` is the first character of a **fresh, non-continuation** input line. A `/` anywhere else — mid-line, mid-continuation, or as the division operator in `10 / 2` — is untouched and handed to `iklo_parser::parse()` exactly as before. No new token kind, AST node, or grammar production is introduced anywhere.
- **Today, `iklo-lexer` has no pipe lexeme and `iklo-cli` has no shell mode** (confirmed via code inspection, and independently by `chatgpt-codex-connector`'s review on PR #8 for ADR-0003) — so there is no existing shell-mode path-parsing behavior for this to collide with. The reservation `AGENTS.md` describes is for a design that doesn't exist yet.

## What the decision commits us to

1. `iklo-cli`'s REPL dispatches `/quit`, `/revision`, `/env` only when `/` is the first character of a fresh input line — the same restriction `.`-commands have today, sigil swapped. This condition is enforced by a single, pure, unit-tested gate function shared by both the submit-time dispatcher and the rustyline `Completer`'s gating logic (two call sites, one source of truth — see [`specs/003-repl-improvements/plan.md`](../003-repl-improvements/plan.md) Key Design Decision 1).
2. The `.`-prefixed commands are fully removed, not kept alongside the new ones (no dual-syntax transition period).
3. `AGENTS.md`'s REPL bullet is updated to describe the new behavior and to drop the now-inaccurate "`/paths` free for shell mode" framing.
4. **This ADR does not decide anything about eventual shell-mode syntax.** If and when shell mode is actually designed, and if it turns out to want `/`-prefixed paths at the start of a line too, that collision gets resolved then — with real information about what shell mode's grammar actually needs (contextual prompt modes, positional disambiguation, or something else) — not pre-emptively guessed at here.

## Alternatives considered

- **A — Keep `.`-prefixed commands, add rustyline without changing the sigil.** Rejected as the primary path: it ships the line-editing half of the ask but not the requested slash-command ergonomics, and defers a decision that turns out to have no real technical cost today (no shell mode exists to collide with).
- **B — Support both `.` and `/` syntaxes simultaneously.** Rejected: two ways to invoke the same three commands is exactly the kind of workaround Constitution §VII ("No Workarounds Left Standing") warns against, for no compensating benefit — nobody's muscle memory needs a transition period for a REPL command set this small.

## Consequences

- **Positive:** REPL commands match near-universal `/`-command convention; tab-completion makes the command set discoverable for the first time; the fresh-prompt-only gate is centralized in one tested function instead of duplicated across the dispatcher and completer.
- **Negative:** None identified — this is a pure CLI-layer UX change with no grammar surface.
- **Reversal cost:** low. Reverting to `.`-prefixed commands (or supporting both) is a small, localized change to `iklo-cli` only; nothing outside that crate depends on the sigil choice.

## Follow-ups

- If shell-mode design work later reintroduces `/`-prefixed paths as a REPL/shell input, resolve the collision with real shell-mode requirements in hand — this ADR takes no position on how.
- Update `AGENTS.md`'s "What is actually implemented today" CLI bullet alongside the syntax-rules bullet (tracked as [`specs/003-repl-improvements/tasks.md`](../003-repl-improvements/tasks.md) T010).
