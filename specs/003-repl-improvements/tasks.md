---
description: "Task list for REPL improvements (rustyline + slash commands)"
status: not-started
---

# Tasks: REPL Improvements — rustyline & slash commands

**Input**: Design documents from `/specs/003-repl-improvements/`

**Prerequisites**: [plan.md](plan.md) (required), [spec.md](spec.md) (required for user stories)

**Tests**: Tests are REQUIRED for the dispatch-gate logic (Constitution I). The interactive rustyline UX itself (arrow-key editing, live tab-completion) is verified by manual REPL smoke test, not unit tests — rustyline's terminal I/O isn't practically unit-testable, matching how the existing `.env`/`.revision`/`.quit` behavior was verified.

**Organization**: Tasks are grouped by phase and user story. Each task ends with a commit.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2)
- Include exact file paths in descriptions

## Path Conventions

- Rust workspace at repository root.
- All changes confined to `crates/iklo-cli/`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Bring `rustyline` into the workspace so subsequent tasks have it available.

- [ ] **T001** [US1] Add `rustyline` (pin the current stable version — re-check via Context7 at implementation time, since this plan was written 2026-07-20) to `crates/iklo-cli/Cargo.toml`. **Acceptance**: `cargo build -p iklo-cli` succeeds; `make test` still green.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Replace the raw stdin loop with a working rustyline `Editor` before either user story's specifics land.

**⚠️ CRITICAL**: No user-story-specific work (command dispatch, completion) can begin until this phase compiles and the REPL still runs via rustyline with zero behavior change beyond line editing.

- [ ] **T002** [US1] Replace `io::stdin().read_line()` in `run_repl()` (`crates/iklo-cli/src/main.rs`) with `rustyline::DefaultEditor`. Preserve exact current behavior: `iklo> ` / `iklo. ` prompts, blank-line-cancels-continuation, EOF-exits. Do NOT yet touch `.quit`/`.revision`/`.env` dispatch — that's Phase 4. **Acceptance**: `cargo build -p iklo-cli` succeeds; manual smoke test confirms unchanged prompt/continuation/EOF behavior, now with arrow-key line editing.

**Checkpoint**: Foundation ready — rustyline is live, behavior is unchanged except for editing.

---

## Phase 3: User Story 1 - REPL user gets real line editing (Priority: P1) 🎯 MVP

**Goal**: Arrow-key history navigation, persisted across sessions.

**Independent Test**: Enter a few expressions, Up arrow recalls them; quit and restart, history is still there.

### Implementation for User Story 1

- [ ] **T003** [US1] Load history from `.iklo_history` (cwd) at REPL startup via `rl.load_history(...)`; treat a missing/unreadable file as non-fatal (per spec Edge Cases). **Acceptance**: starting the REPL with no existing history file does not error.
- [ ] **T004** [US1] Call `rl.add_history_entry(...)` for each complete, submitted top-level line (not a blank continuation-cancel). Save history via `rl.save_history(...)` on REPL exit (`.quit`/`/quit`, and on EOF). **Acceptance**: manual smoke test — Up arrow recalls prior entries within a session; after quit+restart, prior session's history is still reachable.

**Checkpoint**: User Story 1 fully functional — real line editing, persisted history.

---

## Phase 4: User Story 2 - Slash commands with completion (Priority: P1)

**Goal**: `.quit`/`.revision`/`.env` become `/quit`/`/revision`/`/env`, dispatched and completed only at a fresh prompt; a `/` anywhere else is untouched.

**Independent Test**: `/` + Tab at a fresh prompt offers `quit`/`revision`/`env`; `10 / 2` at a fresh prompt evaluates as division, never as a command.

### Tests for User Story 2 (write FIRST, ensure they FAIL) ⚠️

- [ ] **T005** [US2] Write unit tests (in `crates/iklo-cli/src/main.rs` or a new `repl.rs`, `#[cfg(test)] mod tests`) for the pure dispatch-gate function extracted in T006 — e.g. `fn repl_command(buffer_is_empty: bool, trimmed_line: &str) -> Option<ReplCommand>` — covering: fresh prompt + `/quit` → `Some(Quit)`; fresh prompt + `/revision` → `Some(Revision)`; fresh prompt + `/env` → `Some(Env)`; fresh prompt + `/foo` (unrecognized) → `None`; **non-fresh prompt (continuation) + `/quit` → `None`** (the critical negative case); fresh prompt + `10 / 2` → `None`. **Acceptance**: tests fail to compile (function doesn't exist yet) — RED state, verified locally, never pushed alone.
- [ ] **T006** [US2] Implement the dispatch-gate function from T005 as a small, pure, unit-testable function (no rustyline/IO dependency) — this is what both the submit-time dispatcher AND the completer's gating condition call into, so the two dispatch points (plan.md Key Design Decision 1) share one source of truth instead of two hand-synced conditions. **Acceptance**: T005's tests pass, 0 failed.

### Implementation for User Story 2

- [ ] **T007** [US2] Remove the `.quit`/`.revision`/`.env` string-equality checks from `run_repl()`; replace with a call into T006's dispatch-gate function, keyed on `buffer.is_empty()` (the existing "fresh prompt" signal) exactly as before, sigil swapped to `/`. **Acceptance**: manual smoke test — `/quit`, `/revision`, `/env` behave identically to today's `.`-commands; `.quit` etc. no longer do anything special (fall through to the parser, producing a parse error, per FR-006).
- [ ] **T008** [US2] Implement a custom Completer (bundled into a Helper via rustyline's derive macros per plan.md's Technical Context) that offers quit/revision/env completions only when completing at position 0 of a line starting with /, AND the helper's continuation state (updated via rl.helper_mut() by run_repl() immediately before each rl.readline(...) call, per plan.md Key Design Decision 2) confirms the REPL is at a fresh prompt. Wire via rl.set_helper(...). **Acceptance**: manual smoke test — typing / + Tab at a fresh prompt offers the three commands; typing / + Tab mid-continuation offers nothing.
- [ ] **T009** [US2] Manual smoke test covering spec.md's full Acceptance Scenarios list for User Story 2 (all 6 scenarios) plus Edge Cases (Ctrl-C/Ctrl-D behavior preserved; `/` mid-continuation inert). **Acceptance**: every scenario behaves as specified; discrepancies get a task, not a silent skip.

**Checkpoint**: User Stories 1 AND 2 both work — real line editing, persisted history, slash-command dispatch and completion, zero grammar/lexer/parser changes.

---

## Phase 5: Polish & Cross-Cutting Concerns

- [ ] **T010** [P] Update `AGENTS.md`: the REPL bullet under "Non-negotiable syntax rules" (currently `.quit`/`.revision`/`.env`, "to keep `/paths` free for shell mode") and the CLI description in "What is actually implemented today" — both need to describe `/`-prefixed commands, fresh-prompt-only, completion-backed, per ADR-0004.
- [ ] **T011** [P] Update `README.md` if it references the REPL's `.`-commands anywhere.
- [ ] **T012** Run the full gate: `make test && make build && make release`. All three must exit 0. **Acceptance**: three green exits captured in the commit message.
- [ ] **T013** Mark all Success Criteria checkboxes in [spec.md § Success Criteria](spec.md#success-criteria) as ✅ complete in the commit that closes the epic. Open a PR from `003-repl-improvements` → `main`.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: no dependencies.
- **Phase 2 (Foundational)**: depends on Phase 1. Blocks both user stories.
- **Phase 3 (US1)**: depends on Phase 2.
- **Phase 4 (US2)**: depends on Phase 2; sequenced after Phase 3 to avoid merge friction within the same file (`main.rs`), even though the two stories are logically independent.
- **Phase 5 (Polish)**: depends on Phases 3–4.

### Task Dependencies (linear this epic — small scope, single crate)

```
T001 → T002 → T003 → T004 → T005 → T006 → T007 → T008 → T009 → T010 [P] → T012 → T013
                                                              ↳ T011 [P] ↗
```

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to a user story from spec.md for traceability
- Commit after each task; commit subject uses conventional prefix (`feat:`, `refactor:`, `test:`, `docs:`, `cli:`)
- Include the agent co-author trailer on agent-authored commits
- If a task blows up its acceptance criterion, do not paper over — revise the task or its predecessor
