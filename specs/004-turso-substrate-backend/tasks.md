---
description: "Task list for Turso-backed substrate backend"
status: draft
---

# Tasks: Turso-backed Substrate Backend

**Input**: Design documents from `/specs/004-turso-substrate-backend/`

**Prerequisites**: [spec.md](spec.md) and [plan.md](plan.md)

**Tests**: Required (Constitution I). All behavior changes must be test-first.
RED tests must fail locally before implementation, but are not committed as
standalone red-only commits.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel
- **[Story]**: US1, US2, or US3

## Path Conventions

- Rust workspace rooted at repository root
- New backend crate at `crates/iklo-substrate-turso/`

## Blocker Inventory (Canonical Tracker)

Use this table for FR-019 / SC-009 tracking during implementation.

| Blocker ID | Classification | Invariant Impacted | Evidence | Chosen Action | Rationale |
|---|---|---|---|---|---|
| _example_ | adapter-fixable | rollback visibility | failing test name/log | adjust adapter transaction wrapping | API supports needed primitive |

Allowed `Classification` values:
- `adapter-fixable`
- `upstream-fixable`
- `fork-required`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create backend crate and integration scaffolding.

- [ ] **T001** [US1] Add `crates/iklo-substrate-turso/` to workspace and scaffold `Cargo.toml` + `src/lib.rs`, gated behind an explicit Cargo feature per ADR-0001 sequencing.
- [ ] **T002** [US1] Select and pin Turso Rust client dependency in `crates/iklo-substrate-turso/Cargo.toml` under that feature gate, with a short rationale comment in plan-aligned notes.
- [ ] **T003** [US1] Define crate-level error types and conversion boundaries in `crates/iklo-substrate-turso/src/lib.rs`.
- [ ] **T004** [US1] Add schema bootstrap module `crates/iklo-substrate-turso/src/schema.rs` with idempotent create/verify entrypoints.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Establish persistence and codec contracts that all stories depend on.

**CRITICAL**: No user-story implementation starts before this phase is complete.

- [ ] **T005** [US1] Write RED tests for schema bootstrap idempotency and incompatible schema version failure in `crates/iklo-substrate-turso/src/tests.rs`.
- [ ] **T006** [US1] Implement schema versioning + compatibility checks in `schema.rs`.
- [ ] **T007** [US1] Write RED tests for versioned persisted-`V` codec behavior (supported/unsupported shapes, decode failures) in `crates/iklo-substrate-turso/src/tests.rs`.
- [ ] **T008** [US1] Implement codec module `crates/iklo-substrate-turso/src/codec.rs` with explicit version tag handling.
- [ ] **T009** [US1] Write RED tests for retry classification, contention behavior, and ambiguous commit-result handling on transient transport failures in `crates/iklo-substrate-turso/src/tests.rs`.
- [ ] **T010** [US1] Define retry policy helpers and error classification (retryable vs surface-immediately), including post-timeout commit-outcome verification before retry, in `crates/iklo-substrate-turso/src/lib.rs`.

**Checkpoint**: Schema + codec + retry/ambiguity policy in place.

---

## Phase 3: User Story 1 - Persistent backend with stable semantics (P1)

**Goal**: Turso-backed substrate persists state/revision across restarts with correct commit/rollback behavior.

**Independent Test**: Restart persistence test shows committed state and revision continuity.

### Tests for User Story 1 (write first)

- [ ] **T011** [US1] Add RED tests for commit persistence, rollback invisibility, and revision increment semantics in `crates/iklo-substrate-turso/src/tests.rs`.
- [ ] **T012** [US1] Add RED tests for connectivity/auth failure surfaces (no silent fallback) in `crates/iklo-substrate-turso/src/tests.rs`.

### Implementation for User Story 1

- [ ] **T013** [US1] Implement `TursoSubstrate<V>` struct and constructor/config parsing in `crates/iklo-substrate-turso/src/lib.rs`.
- [ ] **T014** [US1] Implement `Substrate` trait for `TursoSubstrate<V>` (begin/revision/snapshot).
- [ ] **T015** [US1] Implement transactional type for `Transaction` trait (get/set/commit/rollback) with atomic visibility guarantees.
- [ ] **T016** [US1] Wire retry policy into transactional operations with bounded retries/backoff for retryable classes only.

**Checkpoint**: US1 behavior passes backend-local tests.

---

## Phase 4: User Story 2 - Reusable contract suite on Turso backend (P1)

**Goal**: Existing generic substrate contract suite passes with Turso backend.

**Independent Test**: Contract suite runs unchanged against `TursoSubstrate<i64>`.

### Tests for User Story 2 (write first)

- [ ] **T017** [US2] Add RED contract-suite harness in `crates/iklo-substrate-turso/src/tests.rs` calling `run_contract_suite(...)`.

### Implementation for User Story 2

- [ ] **T018** [US2] Satisfy remaining trait-contract mismatches revealed by T017 without changing contract case bodies.
- [ ] **T019** [US2] Add snapshot equivalence tests (in-memory vs Turso) for identical committed operations.

**Checkpoint**: US1 + US2 green with contract parity.

---

## Phase 5: User Story 3 - Explicit CLI mode selection (P2)

**Goal**: CLI supports explicit substrate mode/precedence and errors for invalid combinations.

**Independent Test**: default mode remains in-memory; Turso mode persists; invalid config errors explicitly.

### Tests for User Story 3 (write first)

- [ ] **T020** [US3] Add RED CLI tests for precedence (`flags > env`), required fields by mode, invalid-combination errors, and persistence behavior through CLI mode selection in `crates/iklo-cli/src/main.rs` tests.

### Implementation for User Story 3

- [ ] **T021** [US3] Add CLI parsing/wiring for `--substrate`, `--turso-db-url`, and auth-token input with secure handling (prefer env var in docs/examples, redact from output/logs) plus env fallback in `crates/iklo-cli/src/main.rs`.
- [ ] **T022** [US3] Keep in-memory mode as default path; ensure no implicit fallback from Turso failures.
- [ ] **T023** [US3] Integrate runtime/bootstrap path to construct selected substrate mode.

**Checkpoint**: All user stories independently testable.

---

## Phase 6: Governance, Documentation, and Final Gate

**Purpose**: Close readiness gaps and enforce traceability.

- [ ] **T024** [US1] Add FR→Task traceability section in this file mapping `FR-001..FR-024` to task IDs (required for SC-008).
- [ ] **T025** [US1] During implementation, record every blocker in canonical inventory with full schema fields.
- [ ] **T026** [US1] If any blocker is `fork-required`, open follow-up ADR/epic before any fork code change.
- [ ] **T027** [US1] Add an explicit baseline-capture task: record the `main` baseline commit SHA in the implementation PR description (FR-024).
- [ ] **T028** [US1] Update `README.md`, `AGENTS.md`, and `LANGUAGE.md` for substrate modes and configuration semantics.
- [ ] **T029** [US1] Run final gate: `cargo test --workspace && make test && make build && make release`.

---

## Dependencies & Execution Order

### Phase Dependencies

- Phase 1 → Phase 2 → (Phases 3,4,5) → Phase 6
- User stories start only after Phase 2 checkpoint.

### Story Dependencies

- **US1**: starts after Phase 2
- **US2**: depends on foundational crate + US1 substrate behavior
- **US3**: depends on US1 substrate construction path

### Parallel Opportunities

- T002/T003 can run in parallel after T001.
- T017/T019 can run while US3 tests are being prepared if US1 is already stable.
- Documentation updates (T028) can run in parallel with final stabilization work.

---

## FR to Task Traceability

| FR | Primary Tasks |
|---|---|
| FR-001 | T001, T013-T016 |
| FR-002 | T015, T023 |
| FR-003 | T017, T018 |
| FR-004 | T011, T015 |
| FR-005 | T011, T015 |
| FR-006 | T020-T023 |
| FR-007 | T012, T022 |
| FR-008 | T026 |
| FR-009 | T029 |
| FR-010 | T028 |
| FR-011 | T020-T021 |
| FR-012 | T005, T006 |
| FR-013 | T009, T010, T016 |
| FR-014 | T001-T003 |
| FR-015 | T025, T026 |
| FR-016 | T025 |
| FR-017 | T026 |
| FR-018 | T024 |
| FR-019 | T024, T025 |
| FR-020 | T025, T026 |
| FR-021 | T007, T008 |
| FR-022 | T009, T010, T016 |
| FR-023 | T020-T021 |
| FR-024 | T027 |
