# Feature Specification: Turso-backed Substrate Backend

**Feature Branch**: `004-turso-substrate-backend`

**Created**: 2026-07-21

**Status**: Draft

**Input**: Implement the next milestone committed by ADR-0001: add a Turso-backed `Substrate` implementation as a separate crate, keeping `iklo-runtime` as the semantic reference and deferring all VDBE compiler work.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Runtime contributor can run Iklo on a persistent backend (Priority: P1) 🎯 MVP

A contributor can opt into a Turso-backed substrate and execute the current interpreter semantics (`let`, `set`, lexical reads, transactional top-level eval) with committed state surviving process restarts.

**Why this priority**: This is the direct goal of the milestone and unlocks real durability without changing language semantics.

**Independent Test**: Start a runtime instance on a fresh Turso database, evaluate bindings, restart the process with the same database target, and observe the committed bindings and revision counter still present.

**Acceptance Scenarios**:

1. **Given** a new Turso-backed substrate instance, **When** the first transaction commits `x = 42`, **Then** a fresh instance opened against the same database can read `x = 42`.
2. **Given** a Turso-backed transaction that sets a value then rolls back, **When** a new transaction reads the same key, **Then** the rolled-back value is not visible.
3. **Given** a sequence of successful commits, **When** revision is queried, **Then** it increases by one per commit and never increases on rollback.

---

### User Story 2 - Backend author has one reusable contract suite (Priority: P1)

A contributor implementing another substrate backend can validate behavior by running the same trait-level contract suite currently used by `InMemorySubstrate`, with no test body changes.

**Why this priority**: The boundary only pays off if behavior is portable and enforceable across backends.

**Independent Test**: Instantiate the existing generic contract suite with `TursoSubstrate<i64>` and pass all scenarios already required for `InMemorySubstrate<i64>`.

**Acceptance Scenarios**:

1. **Given** the `iklo-substrate` contract suite, **When** it is instantiated with `TursoSubstrate<i64>`, **Then** all contract cases pass unchanged.
2. **Given** both in-memory and Turso implementations, **When** their snapshots are compared after identical committed operations, **Then** they produce equivalent key/value state.

---

### User Story 3 - CLI user can choose persistence mode explicitly (Priority: P2)

A CLI user can run with either in-memory behavior (default) or Turso-backed persistence via explicit configuration, without ambiguity.

**Why this priority**: Required for practical usage and for safe rollout without changing existing defaults.

**Independent Test**: Run `iklo-cli` once in default mode and once in Turso mode; confirm default mode remains ephemeral while Turso mode persists state between runs.

**Acceptance Scenarios**:

1. **Given** default CLI invocation with no persistence flags, **When** the process restarts, **Then** prior bindings are not retained.
2. **Given** CLI invocation configured for Turso substrate, **When** the process restarts with the same database target, **Then** prior committed bindings are retained.
3. **Given** invalid Turso connection settings, **When** CLI starts, **Then** it fails with an explicit error and does not silently fall back to in-memory mode.

### Edge Cases

- What happens when the Turso database is unreachable at startup?
- What happens when a commit partially fails due to transport or auth errors?
- What happens when two runtime instances contend over the same logical binding keys?
- How is schema initialization handled on an empty database versus an existing one?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A new crate `crates/iklo-substrate-turso` MUST provide a `TursoSubstrate<V>` implementation of the existing `Substrate` trait.
- **FR-002**: `iklo-runtime` MUST continue to define Iklo semantics and MUST NOT change observable language behavior for existing tests.
- **FR-003**: The existing substrate contract suite MUST run against `TursoSubstrate<i64>` without modifying contract case bodies.
- **FR-004**: Committed state and revision metadata MUST persist across process restarts when using Turso substrate.
- **FR-005**: Rollback MUST preserve atomicity: uncommitted mutations MUST NOT become visible.
- **FR-006**: CLI MUST expose an explicit, opt-in way to select Turso substrate while keeping in-memory mode as default.
- **FR-007**: Startup/configuration errors for Turso mode MUST be surfaced explicitly; no silent fallback is allowed.
- **FR-008**: This epic MUST NOT introduce VDBE bytecode compilation, opcode work, or Turso forks; scope is storage backend only.
- **FR-009**: `make test`, `make build`, and `make release` MUST pass with Turso support enabled in CI-reproducible configuration.
- **FR-010**: Documentation (`AGENTS.md`, `README.md`, `LANGUAGE.md`) MUST describe both substrate modes and the default selection behavior accurately.

### Key Entities

- **TursoSubstrate\<V\>**: Turso-backed `Substrate` implementation that stores bindings and revision state transactionally.
- **Substrate Record**: Persisted representation of a binding (`name`, `value`, revision visibility).
- **Revision State**: Persisted monotonic counter tracking committed top-level evaluations.
- **CLI Substrate Mode**: Explicit runtime selection between in-memory and Turso-backed substrate.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo test -p iklo-substrate` passes with contract tests instantiated for both in-memory and Turso implementations.
- **SC-002**: A restart persistence test demonstrates committed bindings and revision continuity in Turso mode.
- **SC-003**: Existing `iklo-runtime` behavior tests pass unchanged in source.
- **SC-004**: CLI default mode remains in-memory and non-persistent, while explicit Turso mode persists state.
- **SC-005**: `make test && make build && make release` succeed in a clean workspace.

## Assumptions

- Turso APIs and authentication are available in a form stable enough for a storage-backend implementation without requiring a Turso fork.
- Serialization strategy for Iklo runtime values can be implemented incrementally while preserving current value coverage required by tests.
- Concurrency semantics will follow Turso/SQLite transactional guarantees plus the existing `Substrate` contract.
- VDBE-targeted compiler work remains out of scope and stays gated behind a future, separate ADR.
