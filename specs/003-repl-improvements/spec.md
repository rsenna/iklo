# Feature Specification: REPL Improvements — rustyline & slash commands

**Feature Branch**: `003-repl-improvements`

**Created**: 2026-07-20

**Status**: Implemented

**Input**: Replace the CLI REPL's raw stdin loop with rustyline (line editing, arrow-key history, persistent history file — matching prior art from LogoScript's CLI). Replace the `.`-prefixed meta-commands (`.quit`, `.revision`, `.env`) with `/`-prefixed ones (`/quit`, `/revision`, `/env`), recognized only as the first character of a fresh (non-continuation) REPL input line and offered via tab-completion. This is a REPL-input-loop feature only — it must never reach `iklo-lexer`/`iklo-parser`/`iklo-ast`; a `/` anywhere else in input (mid-line, mid-continuation, or as part of an expression such as `10 / 2`) is untouched and parsed as ordinary Iklo syntax.

## User Scenarios & Testing

### User Story 1 - REPL user gets real line editing (Priority: P1) 🎯 MVP

A person typing at the `iklo>` prompt can use arrow keys to move within and recall previous input, edit in place (not just backspace-to-end), and have their input history persist across separate REPL sessions — the baseline expectation for any modern REPL, and parity with LogoScript's CLI.

**Why this priority**: Without this, every REPL session starts from zero — no history, no in-line editing beyond backspace. This is the foundational UX gap; everything else builds on having a real line editor in place.

**Independent Test**: Start the REPL, enter a few expressions, press Up arrow — the previous input reappears and is editable. Quit and restart the REPL — the same history is still reachable via Up arrow.

**Acceptance Scenarios**:

1. **Given** a fresh REPL session, **When** the user presses Up arrow at the prompt, **Then** the most recently entered line reappears, fully editable (cursor movable, characters insertable/deletable anywhere in the line).
2. **Given** a REPL session with several entries, **When** the user quits and starts a new REPL session, **Then** the previous session's history is still navigable via Up arrow.
3. **Given** a fresh REPL session, **When** the user quits without entering anything, **Then** no history file is created or corrupted.

---

### User Story 2 - REPL meta-commands become `/`-prefixed with completion (Priority: P1)

A person at the `iklo>` prompt types `/` and gets tab-completion over the available REPL meta-commands (`/quit`, `/revision`, `/env`), matching the muscle memory of virtually every modern `/`-command interface (Slack, Discord, Claude Code itself). Anywhere else — mid-line, mid-continuation, or as the division operator in an expression like `10 / 2` — a `/` is untouched and handed to the real Iklo parser.

**Why this priority**: Requested directly, and it's the other half of "improve the REPL." Depends on User Story 1 landing first, since completion is implemented via rustyline's `Helper`/`Completer` traits.

**Independent Test**: At a fresh prompt, type `/` and press Tab — `quit`, `revision`, `env` are offered. Type `/quit` and press Enter — the REPL exits. Separately, type `10 / 2` at a fresh prompt — it evaluates as division (`5`), never triggering command dispatch.

**Acceptance Scenarios**:

1. **Given** a fresh (non-continuation) prompt, **When** the user types `/` and presses Tab, **Then** rustyline offers `quit`, `revision`, `env` as completions.
2. **Given** a fresh prompt, **When** the user types `/quit` and presses Enter, **Then** the REPL exits (same behavior as today's `.quit`).
3. **Given** a fresh prompt, **When** the user types `/revision` and presses Enter, **Then** the current image revision is printed (same behavior as today's `.revision`).
4. **Given** a fresh prompt, **When** the user types `/env` and presses Enter, **Then** all current bindings are printed (same behavior as today's `.env`).
5. **Given** any prompt state, **When** the user types an expression containing `/` that is not at the start of a fresh input line (e.g. `10 / 2`, or `/` appearing mid multi-line continuation), **Then** it is passed unmodified to `iklo_parser::parse()` — no REPL-level interception occurs.
6. **Given** a fresh prompt, **When** the user types an unrecognized `/foo`, **Then** it falls through to the parser exactly as an unrecognized `.foo` does today (producing a parse error), with no new REPL-level error path invented.

### Edge Cases

- What happens if `/` is typed while mid multi-line continuation (buffer non-empty, prompt is `iklo. `)? → No special handling; behaves as ordinary input, same as today's `.`-prefixed commands are restricted to a fresh prompt only.
- What happens on Ctrl-C / Ctrl-D at the prompt? → Must preserve today's existing behavior (blank line cancels multi-line input at a continuation prompt; EOF at a fresh prompt exits cleanly) inside rustyline's `ReadlineError` handling.
- What happens if the history file is missing, unreadable, or corrupted on startup? → REPL must still start; a missing/unreadable history file is not a fatal error (matches rustyline's own `load_history` failure-is-non-fatal convention).

## Requirements

### Functional Requirements

- **FR-001**: `iklo-cli`'s REPL MUST use `rustyline` for line input, replacing the current raw `io::stdin().read_line()` loop.
- **FR-002**: The REPL MUST persist input history across sessions to `.iklo_history` (already reserved in `.gitignore`) in the current working directory.
- **FR-003**: REPL meta-commands (`/quit`, `/revision`, `/env`) MUST be recognized only when `/` is the first character of a fresh (non-continuation) input line — the same restriction `.`-prefixed commands have today, with the sigil swapped.
- **FR-004**: A `/`-prefixed meta-command MUST offer tab-completion via rustyline's `Completer`/`Helper` traits, scoped to firing only at a fresh prompt.
- **FR-005**: A `/` appearing anywhere else in input (mid-line, mid-continuation, or as part of an expression) MUST be passed through unmodified to `iklo_parser::parse()` — the command dispatcher MUST NOT intercept or rewrite it under any circumstance.
- **FR-006**: The `.`-prefixed commands (`.quit`, `.revision`, `.env`) MUST be fully replaced, not kept alongside the new `/`-prefixed ones.
- **FR-007**: `crates/iklo-lexer`, `crates/iklo-parser`, and `crates/iklo-ast` MUST NOT be modified by this feature — it is entirely contained in `crates/iklo-cli`.
- **FR-008**: `cargo test --workspace` MUST continue to pass.

### Key Entities

- **REPL Helper**: A `rustyline::Helper`-bundling struct (via rustyline's derive macros) owning a custom `Completer` that offers the known command set only under the fresh-prompt-and-leading-slash condition. Lives in `crates/iklo-cli`.
- **History file** (`.iklo_history`): Plain-text rustyline history, persisted in the current working directory, already reserved in `.gitignore`.

## Success Criteria

- **SC-001** ✅: A user can navigate REPL input history with the Up/Down arrow keys within a session.
- **SC-002** ✅: REPL history persists across separate REPL invocations via `.iklo_history`.
- **SC-003** ✅: Typing `/` at a fresh REPL prompt and pressing Tab offers completion over `quit`, `revision`, `env`.
- **SC-004** ✅: A `/` anywhere else in input (e.g. `10 / 2`) is parsed as ordinary Iklo syntax with zero REPL-level interception.
- **SC-005** ✅: `cargo test --workspace` passes with zero changes to `iklo-lexer`, `iklo-parser`, or `iklo-ast`.

## Assumptions

- `/`-prefixed commands fully replace `.`-prefixed ones; there is no transition period supporting both syntaxes simultaneously.
- No new REPL commands are introduced beyond the existing three (`quit`, `revision`, `env`) — this is a syntax migration plus completion, not a command-surface expansion.
- Command dispatch remains exact-string matching (`/quit`, not `/q` or other abbreviations); tab-completion is the discoverability mechanism, not prefix-abbreviation matching.
- This feature requires an ADR (per `AGENTS.md`'s "Non-negotiable syntax rules" — the REPL's `.`-prefix rule is listed there, even though the mechanism never touches the grammar) — see [ADR-0004](../../specs/decisions/ADR-0004-repl-slash-commands.md), authored alongside this epic.
