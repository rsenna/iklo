---
description: "Task list for REPL improvements (rustyline + slash commands)"
status: in-progress (T001-T002 done; T003-T015 remaining)
---

# Tasks: REPL Improvements — rustyline & slash commands

**Input**: Design documents from `/specs/003-repl-improvements/`

**Prerequisites**: [plan.md](plan.md) (required), [spec.md](spec.md) (required for user stories)

**Tests**: Tests are REQUIRED (Constitution I) for every pure, testable decision this feature introduces: the REPL-command eligibility gate, the exact-match command parser, and the history-save-or-skip predicate (see plan.md Key Design Decisions 1 and 5). The interactive rustyline UX itself (arrow-key editing, live tab-completion, actual file I/O) is verified by manual REPL smoke test, not unit tests — rustyline's terminal I/O isn't practically unit-testable, matching how the existing `.env`/`.revision`/`.quit` behavior was verified.

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

- [x] **T001** [US1] Add `rustyline` (pin the current stable version — re-check via Context7 at implementation time, since this plan was written 2026-07-20) to `crates/iklo-cli/Cargo.toml`. **Acceptance**: `cargo build -p iklo-cli` succeeds; `make test` still green.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Replace the raw stdin loop with a working rustyline `Editor` before either user story's specifics land.

**⚠️ CRITICAL**: No user-story-specific work (command dispatch, completion) can begin until this phase compiles and the REPL still runs via rustyline with zero behavior change beyond line editing.

- [x] **T002** [US1] Replace `io::stdin().read_line()` in `run_repl()` (`crates/iklo-cli/src/main.rs`) with `rustyline::DefaultEditor`. Preserve exact current behavior: `iklo> ` / `iklo. ` prompts, blank-line-cancels-continuation, EOF-exits. Do NOT yet touch `.quit`/`.revision`/`.env` dispatch — that's Phase 4. **Acceptance**: `cargo build -p iklo-cli` succeeds; manual smoke test confirms unchanged prompt/continuation/EOF behavior, now with arrow-key line editing.

**Checkpoint**: Foundation ready — rustyline is live, behavior is unchanged except for editing.

---

## Phase 3: User Story 1 - REPL user gets real line editing (Priority: P1) 🎯 MVP

**Goal**: Arrow-key history navigation, persisted across sessions — without ever creating a history file for a session that entered nothing.

**Independent Test**: Enter a few expressions, Up arrow recalls them; quit and restart, history is still there. Separately: start the REPL fresh, quit immediately with no input — no `.iklo_history` file appears.

### Tests for User Story 1 (write FIRST, ensure they FAIL) ⚠️

- [ ] **T003** [US1] Write RED unit tests (in `crates/iklo-cli/src/main.rs` or a new `repl.rs`, `#[cfg(test)] mod tests`) for:
  - `should_save_history(had_pre_existing_file: bool, entries_added_this_session: bool) -> bool` — pure predicate, no I/O: `(false, false) -> false` (the critical case — a fresh session with zero input must not create a file, per spec.md Acceptance Scenario 3); `(false, true) -> true`; `(true, false) -> true` (an existing file is still saved even if this session added nothing, keeping behavior simple and predictable); `(true, true) -> true`.
  - A load-history smoke test using a real scratch path (e.g. `std::env::temp_dir().join(format!("iklo-test-history-{}", std::process::id()))`, cleaned up after): loading from a path that does not exist must not error or panic.
  **Acceptance**: tests fail to compile (`should_save_history` doesn't exist yet) — RED state, verified locally, never pushed alone.

### Implementation for User Story 1

- [ ] **T004** [US1] Implement `should_save_history` per T003. **Acceptance**: T003's tests pass, 0 failed.
- [ ] **T005** [US1] Load history from `.iklo_history` (cwd) at REPL startup via `rl.load_history(...)`; treat a missing/unreadable file as non-fatal (per spec Edge Cases); record whether the file existed at startup (`path.exists()`, checked once, before `load_history` is called) for T006's use. **Acceptance**: starting the REPL with no existing history file does not error.
- [ ] **T006** [US1] Call `rl.add_history_entry(...)` for each complete, submitted top-level line (not a blank continuation-cancel); track whether any entry was added this session. On REPL exit (`/quit`, and on EOF), call `should_save_history(had_pre_existing_file, entries_added_this_session)` and only call `rl.save_history(...)` if it returns `true`. **Acceptance**: manual smoke test — Up arrow recalls prior entries within a session; after quit+restart, prior session's history is still reachable; a fresh session quit with zero input leaves no `.iklo_history` file (verify by removing any pre-existing one first).

**Checkpoint**: User Story 1 fully functional — real line editing, persisted history, no spurious empty history file.

---

## Phase 4: User Story 2 - Slash commands with completion (Priority: P1)

**Goal**: `.quit`/`.revision`/`.env` become `/quit`/`/revision`/`/env`, dispatched and completed only at a fresh prompt; a `/` anywhere else is untouched.

**Independent Test**: `/` + Tab at a fresh prompt offers `quit`/`revision`/`env`; `10 / 2` at a fresh prompt evaluates as division, never as a command; `  /quit` (leading whitespace) at a fresh prompt is NOT treated as a command.

### Tests for User Story 2 (write FIRST, ensure they FAIL) ⚠️

- [ ] **T007** [US2] Write RED unit tests for the two functions from plan.md Key Design Decision 1:
  - `is_repl_command_position(buffer_is_empty: bool, line: &str) -> bool` — fresh prompt + `"/"` → `true`; fresh prompt + `"/q"` → `true` (partial input must still be eligible — this is the case the earlier, since-corrected single-function design got wrong); fresh prompt + `"foo"` → `false`; **non-fresh prompt (continuation) + `"/quit"` → `false`** (critical negative case); fresh prompt + `"  /quit"` (leading whitespace) → `false` (critical negative case — `/` must be byte zero of the untrimmed line, per FR-003).
  - `parse_repl_command(line: &str) -> Option<ReplCommand>` — `"/quit"` → `Some(Quit)`; `"/revision"` → `Some(Revision)`; `"/env"` → `Some(Env)`; `"/foo"` → `None`; `"10 / 2"` → `None` (defensive — in practice `is_repl_command_position` already excludes this); `"/quit"` with trailing whitespace/newline → `Some(Quit)` (trailing-only trim tolerance).
  **Acceptance**: tests fail to compile (neither function exists yet) — RED state, verified locally, never pushed alone.
- [ ] **T008** [US2] Implement `is_repl_command_position` and `parse_repl_command` per T007. **Acceptance**: T007's tests pass, 0 failed.

### Implementation for User Story 2

- [ ] **T009** [US2] Remove the `.quit`/`.revision`/`.env` string-equality checks from `run_repl()`; replace the submit-time dispatch with: call `is_repl_command_position(buffer.is_empty(), &line)`, and only if `true`, call `parse_repl_command(&line)` and dispatch on the result — falling through to ordinary parsing on `None` at either step (matching FR-006's "unrecognized `/foo` falls through to the parser" requirement). **Acceptance**: manual smoke test — `/quit`, `/revision`, `/env` behave identically to today's `.`-commands; `.quit` etc. no longer do anything special (fall through to the parser, producing a parse error); `  /quit` (leading whitespace) is not treated as a command.
- [ ] **T010** [US2] Implement a custom `Completer` (bundled into a `Helper` via rustyline's derive macros per plan.md's Technical Context) that calls `is_repl_command_position` — using the helper's own continuation-state field, updated via `rl.helper_mut()` by `run_repl()` immediately before each `rl.readline(...)` call, per plan.md Key Design Decision 2 — to decide eligibility, then does its own prefix-filtering over `["quit", "revision", "env"]` against whatever partial text follows `/` (standard rustyline completion pattern; does NOT call `parse_repl_command`, per Key Design Decision 1). Wire via `rl.set_helper(...)`. **Acceptance**: manual smoke test — typing `/` + Tab at a fresh prompt offers the three commands; typing `/q` + Tab offers `quit`; typing `/` + Tab mid-continuation offers nothing.
- [ ] **T011** [US2] Manual smoke test covering spec.md's full Acceptance Scenarios list for User Story 2 (all 6 scenarios) plus Edge Cases (Ctrl-C/Ctrl-D behavior preserved; `/` mid-continuation inert; leading-whitespace `/quit` inert). **Acceptance**: every scenario behaves as specified; discrepancies get a task, not a silent skip.

**Checkpoint**: User Stories 1 AND 2 both work — real line editing, persisted history, slash-command dispatch and completion (including partial-input completion), zero grammar/lexer/parser changes.

---

## Phase 5: Polish & Cross-Cutting Concerns

- [ ] **T012** [P] Update `AGENTS.md`: the REPL bullet under "Non-negotiable syntax rules" (currently `.quit`/`.revision`/`.env`, "to keep `/paths` free for shell mode") and the CLI description in "What is actually implemented today" — both need to describe `/`-prefixed commands, fresh-prompt-only, completion-backed, per ADR-0004.
- [ ] **T013** [P] Update `README.md` if it references the REPL's `.`-commands anywhere.
- [ ] **T014** Run the full gate: `make test && make build && make release`. All three must exit 0. **Acceptance**: three green exits captured in the commit message.
- [ ] **T015** Mark all Success Criteria checkboxes in [spec.md § Success Criteria](spec.md#success-criteria) as ✅ complete in the commit that closes the epic. Open a PR from `003-repl-improvements` → `main`.

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
T001 → T002 → T003 → T004 → T005 → T006 → T007 → T008 → T009 → T010 → T011 → T012 [P] → T014 → T015
                                                                              ↳ T013 [P] ↗
```

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to a user story from spec.md for traceability
- Commit after each task; commit subject uses conventional prefix (`feat:`, `refactor:`, `test:`, `docs:`, `cli:`)
- Include the agent co-author trailer on agent-authored commits
- If a task blows up its acceptance criterion, do not paper over — revise the task or its predecessor
