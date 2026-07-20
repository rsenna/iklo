# Implementation Plan: REPL Improvements — rustyline & slash commands

**Branch**: `003-repl-improvements` | **Date**: 2026-07-20 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/003-repl-improvements/spec.md`

## Summary

Replace `iklo-cli`'s raw `io::stdin().read_line()` REPL loop with `rustyline`, and replace the `.`-prefixed meta-commands with `/`-prefixed ones dispatched (and completed) entirely at the REPL-input-loop level — never touching the lexer, parser, or AST. Grounded in [ADR-0004](../../specs/decisions/ADR-0004-repl-slash-commands.md), which records why this is a CLI-only change despite `AGENTS.md`'s `.`-prefix rule living inside the "Non-negotiable syntax rules" list.

## Technical Context

**Language/Version**: Rust, edition 2021 (workspace).

**Primary Dependencies**: `rustyline` (new dependency, `crates/iklo-cli` only). Confirmed current API via Context7 (`/kkawakam/rustyline`): `Editor<H, DefaultHistory>` with a custom `Helper` (derived via `#[derive(Completer, Hinter, Highlighter, Validator, Helper)]`, marking the completing field `#[rustyline(Completer)]`), `rl.set_helper(...)`, `rl.load_history(path)` / `rl.save_history(path)` (missing file is non-fatal), `rl.readline(prompt) -> Result<String, ReadlineError>`, `rl.add_history_entry(...)`.

**Storage**: `.iklo_history` (plain text, cwd), already reserved in `.gitignore`.

**Testing**: `cargo test -p iklo-cli` for the completer's dispatch-boundary logic (fresh-prompt-and-leading-slash gating); manual REPL smoke test for the interactive editing/completion UX itself (not practically unit-testable through rustyline's terminal I/O).

**Target Platform**: Same as workspace (native, whatever `mise.toml` pins). rustyline supports Unix and Windows.

**Project Type**: CLI REPL enhancement — single crate (`iklo-cli`) touched.

**Performance Goals**: None beyond "feels instant" — REPL-scale input, not a hot path.

**Constraints**: `iklo-lexer`, `iklo-parser`, `iklo-ast` MUST NOT change (FR-007). The fresh-prompt-only gate (FR-003/FR-005) is the one correctness-critical property — cross-checked by both the completer (interactive) and the submit-time dispatcher (on Enter), so a slash mid-continuation is never treated as a command from either angle.

**Scale/Scope**: One new dependency, one crate touched, ~150–250 LOC in `crates/iklo-cli/src/main.rs` (or a new `repl.rs` module if `main.rs` gets noisy).

## Constitution Check

Verified against [`.specify/memory/constitution.md`](../../.specify/memory/constitution.md):

- **I. Test-First** ✅ — the fresh-prompt-and-leading-slash dispatch gate is genuinely unit-testable (pure function of buffer state + line content) and gets tests written first; the interactive rustyline UX itself is verified via manual REPL smoke test, same as the existing `.env`/`.revision`/`.quit` behavior was.
- **II. One Epic In Flight** ✅ — `001-substrate` is explicitly paused at its Phase 4 checkpoint (per its `tasks.md` status line) before this epic starts.
- **III. Substrate Before Feature** N/A — this feature doesn't touch runtime image state.
- **IV. Kebab-Case Iklo, Idiomatic Rust** ✅ — REPL commands (`/quit` etc.) are CLI meta-syntax, not Iklo-level identifiers; Rust code stays idiomatic.
- **V. Comments Justify Themselves** ✅ — the fresh-prompt-gate's non-obviousness (why a slash elsewhere is never intercepted) is exactly the kind of thing worth one clarifying comment at the dispatch site.
- **VI. ADRs for Load-Bearing Decisions** ✅ — [ADR-0004](../../specs/decisions/ADR-0004-repl-slash-commands.md) grounds the `.`→`/` change, since `AGENTS.md`'s existing rule lives inside the ADR-gated "Non-negotiable syntax rules" list even though the actual mechanism (per this plan) never reaches the grammar.
- **VII. No Workarounds Left Standing** ✅ — full replacement of `.`-commands (FR-006), not a dual-syntax shim.

No violations; Complexity Tracking left empty.

## Project Structure

### Documentation (this feature)

```text
specs/003-repl-improvements/
├── plan.md              # This file
├── spec.md              # Feature spec
└── tasks.md             # Executable task list
```

### Source Code (repository root)

```text
crates/iklo-cli/
├── Cargo.toml            # + rustyline dependency
└── src/
    └── main.rs            # REPL loop rewritten on rustyline; command
                            # dispatch + completer live here (or a new
                            # repl.rs module, task-writer's call, if
                            # main.rs would otherwise exceed a screenful)
```

**Structure Decision**: Single-crate change. No new crate — matches the constitution's "prefer adding a new module over a new crate" preference; a `repl.rs` submodule inside `iklo-cli` is the ceiling of new structure this feature needs.

## Key Design Decisions

1. **Two dispatch points share one gate, not one each.** The fresh-prompt-and-leading-slash condition governs both (a) rustyline's `Completer::complete` (interactive, as-you-type) and (b) the submit-time command dispatcher (on Enter). Both must independently agree a line qualifies before it's ever treated as a command — this is what makes FR-005 (division/mid-line `/` is untouched) actually safe rather than merely intended.
2. **Buffer-continuation state must reach the `Completer`.** Rustyline's `Completer::complete(&self, line, pos, ctx)` only sees the current line being edited — it has no notion of "is this REPL mid multi-line continuation." The `Helper` struct needs a shared handle (e.g. `Rc<Cell<bool>>`) set by the outer REPL loop immediately before each `rl.readline(...)` call, mirroring today's `buffer.is_empty()` check.
3. **No new command surface.** Exactly `quit`, `revision`, `env` — a syntax migration, not a feature expansion (spec Assumptions).
4. **Full replacement, not dual syntax.** `.`-commands are deleted in the same change that adds `/`-commands (FR-006, Constitution VII).

## Complexity Tracking

*No violations to track.*
