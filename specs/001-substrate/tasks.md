---
description: "Task list for substrate capability boundary"
---

# Tasks: Substrate Capability Boundary

**Input**: Design documents from `/specs/001-substrate/`

**Prerequisites**: [plan.md](plan.md) (required), [spec.md](spec.md) (required for user stories)

**Tests**: Tests are REQUIRED. Test-first is a constitutional principle (I. Test-First (Non-Negotiable)).

**Organization**: Tasks are grouped by phase and user story. Each task ends with a commit.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- Rust workspace at repository root.
- New crate at `crates/iklo-substrate/`.
- Existing crate at `crates/iklo-runtime/`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Bring the new crate into the workspace so subsequent tasks have a home.

- [ ] **T001** [US2] Scaffold `crates/iklo-substrate/` with `Cargo.toml` (edition 2021, workspace-inherited version) and empty `src/lib.rs`. Add the crate to `[workspace] members` in the root `Cargo.toml`. **Acceptance**: `cargo build -p iklo-substrate` succeeds; `make test` still green.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Define the trait shape before any user story can proceed. Blocks all downstream work.

**⚠️ CRITICAL**: No user story implementation can begin until this phase is complete.

- [ ] **T002** [US2] Define `Substrate` + `Transaction` traits and `SubstrateError` in `crates/iklo-substrate/src/lib.rs`. Signatures per [plan.md § Trait shape](plan.md#trait-shape-starting-point-subject-to-t2-refinement). Include doc comments on the public trait explaining the transaction contract and revision semantics. Implementation methods can be `todo!()` at this stage — the goal is compilation of the trait surface. Also investigate whether `iklo-cli`'s `.env` command needs `&HashMap` or just an iterator; note the decision in the commit message. **Acceptance**: `cargo build -p iklo-substrate` succeeds; trait signatures compile; investigation result recorded.

**Checkpoint**: Foundation ready — user story implementation can begin.

---

## Phase 3: User Story 2 - Future backend author has a trait to implement (Priority: P2) 🎯 MVP-of-the-boundary

**Goal**: A working, tested reference implementation of the `Substrate` trait, so the boundary is proven (not just declared).

**Independent Test**: `cargo test -p iklo-substrate` passes 7 contract tests. `cargo tree -p iklo-substrate` shows zero workspace-internal dependencies.

### Tests for User Story 2 (write FIRST, ensure they FAIL) ⚠️

- [ ] **T003** [US2] Add the 7 contract tests to `crates/iklo-substrate/src/memory.rs` (or a `#[cfg(test)] mod tests` at the bottom): `revision_starts_at_zero`, `commit_increments_revision`, `rollback_does_not_increment_revision`, `get_after_set_inside_tx_sees_value`, `get_after_rollback_does_not_see_value`, `get_after_commit_sees_value_from_fresh_tx`, `snapshot_returns_only_committed_state`. Tests instantiate `InMemorySubstrate::<i64>::new()` (a concrete `V` proves generics work). Tests must FAIL because `InMemorySubstrate` does not yet exist. **Acceptance**: `cargo test -p iklo-substrate` shows 7 failing tests with compile errors on the missing type.

### Implementation for User Story 2

- [ ] **T004** [US2] Implement `memory::InMemorySubstrate<V>` in `crates/iklo-substrate/src/memory.rs`: `bindings: HashMap<String, V>`, `revision: u64`. Its `Tx<'a>` clones bindings on `begin`; `commit(self)` writes back and increments `revision`; `rollback(self)` drops. `get` reads from the tx's clone; `set` writes to it; `snapshot()` returns only committed state (from the substrate's own `bindings`, not any open tx). Declare `pub mod memory;` in `lib.rs`. **Acceptance**: all 7 contract tests from T003 pass. `cargo test -p iklo-substrate` — 7 passed, 0 failed.

**Checkpoint**: User Story 2 fully functional — the boundary is real and validated.

---

## Phase 4: User Story 1 - Language-design contributor sees no change (Priority: P1) 🎯 MVP-of-the-refactor

**Goal**: `iklo-runtime` routes all binding state through `Substrate`; the existing runtime tests pass unchanged; CLI behaviour is byte-identical.

**Independent Test**: `cargo test -p iklo-runtime` passes with the two existing tests **unchanged in source**. `examples/hello.iklo` output byte-matches the pre-epic snapshot.

### Tests for User Story 1

- [ ] **T005** [US1] Capture the current output of `examples/hello.iklo` to a scratch file (`/tmp/iklo-hello.pre` or session workspace) and record its SHA-256 in T007's acceptance criteria. This is the byte-identity baseline for the refactor. **Acceptance**: baseline captured; hash recorded.

### Implementation for User Story 1

- [ ] **T006** [US1] Refactor `crates/iklo-runtime/src/lib.rs`:
  - Add `iklo-substrate = { path = "../iklo-substrate" }` to `crates/iklo-runtime/Cargo.toml`.
  - Replace `RuntimeImage`'s internal `HashMap` with `InMemorySubstrate<Value>`. Consider a `type IkloSubstrate = iklo_substrate::memory::InMemorySubstrate<Value>;` alias if signatures get noisy.
  - Public methods (`new`, `revision`, `bindings`, `eval_in_tx`) keep their current signatures; internally they delegate to the substrate.
  - Replace the internal `Transaction` struct's `HashMap` with a substrate `Tx`. `eval_expr`'s `LexRef` and `Let` arms call `.get` / `.set` on the tx.
  - `eval_in_tx` opens a tx via `substrate.begin()`, runs the program, calls `tx.commit()` on success or `tx.rollback()` on error.
  - Add `impl From<SubstrateError> for RuntimeError` (or a `RuntimeError::Substrate(SubstrateError)` variant).
  - **Acceptance**: `cargo build -p iklo-runtime` succeeds; `cargo test -p iklo-runtime` passes with `let_returns_bound_value` and `rollback_keeps_image_unchanged` **unchanged in source**.

### Verification for User Story 1

- [ ] **T007** [US1] Re-run `examples/hello.iklo` and compare its output SHA-256 to the T005 baseline; they must match byte-for-byte. Manual REPL smoke: `cargo run -p iklo-cli`, then `let :x be 21 * 2` → `:x` returns `42`; `.env` shows `x = 42`; `.revision` shows `1`; `.quit`. **Acceptance**: hash matches; REPL smoke passes as described.

**Checkpoint**: User Stories 1 AND 2 both work; the refactor is complete and invisible.

---

## Phase 5: User Story 3 - Interpreter author reads a thinner runtime (Priority: P3)

**Goal**: `iklo-runtime` no longer holds storage types for bindings. This is largely a lint on the outcome of Phase 4.

**Independent Test**: `grep -E 'HashMap|Vec<HashMap>|RefCell' crates/iklo-runtime/src/` returns no matches related to binding storage.

- [ ] **T008** [US3] Run the grep above; if any match relates to binding storage, refactor further to remove it (state lives behind the trait). If matches are unrelated (e.g., in a test helper), document why in a comment. **Acceptance**: grep clean, or every remaining match has a justifying comment.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Documentation and epic closure.

- [ ] **T009** [P] Update `AGENTS.md` "What is actually implemented today" section: add `iklo-substrate` (trait + in-memory implementation); note that `RuntimeImage` is now a façade over `InMemorySubstrate<Value>`.
- [ ] **T010** [P] Update `LANGUAGE.md`'s "Transactional VDBE and live image runtime" section: add a note that as of this epic the runtime image lives behind a `Substrate` trait (in `iklo-substrate`); the active implementation is in-memory; Turso is deferred per [ADR-0001](../../spec/decisions/ADR-0001-substrate-boundary.md).
- [ ] **T011** Run the full gate: `make test && make build && make release`. All three must exit 0. **Acceptance**: three green exits captured in the commit message.
- [ ] **T012** Mark all Success Criteria checkboxes in [spec.md § Success Criteria](spec.md#success-criteria-mandatory) as ✅ complete in the commit that closes the epic. Open a PR from `001-substrate` → `main`.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: no dependencies.
- **Phase 2 (Foundational)**: depends on Phase 1. Blocks all user stories.
- **Phase 3 (US2)**: depends on Phase 2. Deliverable: proven boundary.
- **Phase 4 (US1)**: depends on Phase 3 (needs `InMemorySubstrate` to plug into runtime).
- **Phase 5 (US3)**: depends on Phase 4 (verifies its cleanliness).
- **Phase 6 (Polish)**: depends on Phases 3–5.

### Task Dependencies (linear this epic — small scope)

```
T001 → T002 → T003 → T004 → T005 → T006 → T007 → T008 → T009 [P] → T011 → T012
                                                       ↘ T010 [P] ↗
```

### Parallel Opportunities

- T009 (`AGENTS.md`) and T010 (`LANGUAGE.md`) touch different files; run in parallel if convenient.
- Everything else is sequential because it's the same small crate under active refactor.

---

## Implementation Strategy

### MVP-of-the-boundary first (Phase 3)

`InMemorySubstrate` with all 7 contract tests passing is the smallest artefact
that proves the epic is viable. If it doesn't cleanly emerge, we stop and
reconsider the trait shape before touching `iklo-runtime`.

### Then MVP-of-the-refactor (Phase 4)

Route `iklo-runtime` through the substrate. If either existing runtime test
requires a source change to pass, we've broken the constitutional promise
(User Story 1's independent test) — stop and revise T006's approach.

### Then close the loop (Phases 5, 6)

Verify cleanliness, update docs, run the gate, open the PR.

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to a user story from spec.md for traceability
- Commit after each task; commit subject uses conventional prefix (`feat:`, `refactor:`, `test:`, `docs:`, `chore:`)
- Include the Copilot co-author trailer on agent-authored commits
- If a task blows up its acceptance criterion, do not paper over — revise the task or its predecessor
