# Feature Specification: Turso-backed Substrate Backend

**Feature Branch**: `004-turso-substrate-backend`

**Created**: 2026-07-21

**Status**: Draft

**Input**: Implement the next milestone committed by [ADR-0001](../decisions/ADR-0001-substrate-boundary.md): add a Turso-backed `Substrate` implementation as a separate crate, keeping `iklo-runtime` as the semantic reference and deferring all VDBE compiler work.

## Integration Strategy (Decision Record)
Fork-governance policy is defined in [ADR-0005](../decisions/ADR-0005-turso-fork-governance.md). This spec applies that policy to the implementation sequence below.

This epic follows an explicit three-phase strategy for Turso integration:

1. **Adapter-first (no fork default)**: build `iklo-substrate-turso` against exposed/stable Turso interfaces only.
2. **Fork-on-blocker (gated)**: if a required `Substrate` invariant cannot be satisfied in Iklo's adapter layer, record the blocker and escalate per ADR-0005.
3. **Fork-governed (controlled)**: any fork work belongs to a follow-up ADR/epic, not this epic, and any approved fork change stays bounded, documented, and evaluated for upstreaming.

Decision rule for each integration issue:

- Change **Iklo** when the issue is adapter mapping, serialization, CLI/runtime policy, or call-site behavior.
- Change **Turso** only when `Substrate` invariants (transactional atomicity, rollback visibility, revision semantics, or required transactional guarantees) cannot be met through exposed APIs.
- Prefer **upstream contribution** when the needed Turso change is general-purpose and not Iklo-specific.

## Readiness Gaps (Tracked)

The following execution gaps are considered in-scope tracking items for this epic:

1. Missing implementation artifacts (`plan.md`, `tasks.md`) under `specs/004-turso-substrate-backend/`.
2. Blocker inventory storage/location and schema are not yet explicit.
3. Fork-escalation mechanics (trigger, evidence, approval handoff) need precise workflow rules.
4. Value persistence/serialization shape is not yet constrained.
5. Concurrency conflict handling and retry policy are still too open-ended.
6. CLI configuration precedence and invalid-combination behavior need explicit rules.
7. Branch-sync hygiene before implementation must be treated as an execution prerequisite.
## User Scenarios & Testing *(mandatory)*

### User Story 1 - Runtime contributor can run Iklo on a persistent backend (Priority: P1)

A contributor can opt into a Turso-backed substrate and execute the current interpreter semantics (`let`, `set`, lexical reads, transactional top-level eval) with committed state surviving process restarts.

**Why this priority**: This is the direct goal of the milestone and unlocks real durability without changing language semantics.

**Independent Test**: Start a runtime instance on a fresh Turso database, evaluate bindings, restart the process with the same database target, and observe the committed bindings and revision counter still present.

**Acceptance Scenarios**:

1. **Given** a new Turso-backed substrate instance, **When** `Transaction::set("x", 42)` is committed, **Then** a fresh instance opened against the same database can read `Some(42)` from `get("x")`.
2. **Given** a Turso-backed transaction that calls `set("x", 42)` and then rolls back, **When** a new transaction calls `get("x")`, **Then** the rolled-back value is not visible.
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

**Independent Test**: Run the `iklo` executable once in default mode and once in Turso mode; confirm default mode remains ephemeral while Turso mode persists state between runs.

**Acceptance Scenarios**:

1. **Given** default CLI invocation with no persistence flags, **When** the process restarts, **Then** prior bindings are not retained.
2. **Given** CLI invocation configured for Turso substrate, **When** the process restarts with the same database target, **Then** prior committed bindings are retained.
3. **Given** invalid Turso connection settings, **When** CLI starts, **Then** it fails with an explicit error and does not silently fall back to in-memory mode.

### Edge Cases

- If the Turso database is unreachable at startup, substrate initialization fails fast with an explicit error and no fallback to in-memory mode.
- If a commit fails due to transport/auth/database error, the transaction is treated as failed, the error is surfaced to the caller, and no uncommitted mutation becomes visible.
- If two runtime instances contend over the same binding keys, correctness follows database transaction guarantees: each top-level evaluation is atomic, with no torn writes and no partial visibility.
- Schema initialization is idempotent: first run creates required tables/indexes, and subsequent runs verify compatibility without destructive migration.
- If integration blockers are discovered, each blocker is classified as: adapter-fixable, upstream-fixable, or fork-required, with rationale captured in this epic's artifacts.

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
- **FR-011**: CLI substrate selection MUST be explicit and stable: `--substrate memory|turso` (default `memory`) and Turso mode MUST require `--turso-db-url <url>` (or `IKLO_TURSO_DB_URL`) plus optional `--turso-auth-token <token>` (or `IKLO_TURSO_AUTH_TOKEN`).
- **FR-012**: Schema bootstrap MUST be idempotent and validated on startup; incompatible schema versions MUST fail with an explicit migration/version error.
- **FR-013**: Multi-instance contention behavior MUST be defined by transactional correctness guarantees (atomic commit/rollback visibility), with conflicts surfaced as explicit errors or retries per backend semantics.
- **FR-014**: Implementation sequencing MUST be adapter-first: `iklo-substrate-turso` starts with no Turso fork and uses only exposed/stable interfaces.
- **FR-015**: Turso fork work MUST be gated by explicit blocker evidence showing a required `Substrate` invariant cannot be implemented in Iklo's adapter layer, and any fork execution is out of scope for this epic.
- **FR-016**: For each blocker, the project MUST record classification (`adapter-fixable`, `upstream-fixable`, `fork-required`) and chosen action with rationale.
- **FR-017**: If a blocker is classified as fork-required, the next step MUST be a follow-up ADR/epic under ADR-0005 governance (patch scope limits, upstream-first policy when feasible, and upstream sync cadence), rather than fork implementation in this epic.
- **FR-018**: Before implementation starts, this epic MUST produce `plan.md` and `tasks.md` under `specs/004-turso-substrate-backend/` and treat them as the execution source of truth.
- **FR-019**: Blocker inventory MUST have a single canonical home in this epic's artifacts (under `tasks.md` or a linked subsection from it), with each blocker recording: ID, classification, invariant impacted, evidence, chosen action, and rationale.
- **FR-020**: Fork-escalation workflow MUST be explicit: a blocker may be marked `fork-required` only with a reproducible failing case, documented adapter attempts, and an upstream-feasibility assessment, followed by approval handoff to maintainers for follow-up ADR/epic creation.
- **FR-021**: `TursoSubstrate<V>` value persistence MUST define a versioned serialization contract for currently supported persisted-`V` shapes, including behavior for unsupported/unknown shapes and an explicit migration policy for schema/data version mismatches.
- **FR-022**: Concurrency handling MUST define where retries are permitted, which errors are retryable, retry bounds/backoff policy, and which failures must surface immediately.
- **FR-023**: CLI configuration semantics MUST define precedence (`CLI flags` over `env`), required/optional fields by substrate mode, and explicit error behavior for invalid combinations.
- **FR-024**: Implementation branches for this epic MUST be cut from an up-to-date `main` and include a recorded baseline commit in the implementation PR description.

### Key Entities

- **`TursoSubstrate<V>`**: Turso-backed `Substrate` implementation that stores bindings and revision state transactionally.
- **Substrate Record**: Persisted representation of a binding (`name`, `value`, revision visibility).
- **Revision State**: Persisted monotonic counter tracking committed top-level evaluations.
- **CLI Substrate Mode**: Explicit runtime selection between in-memory and Turso-backed substrate.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo test --workspace` passes, including contract-suite instantiation in both `iklo-substrate` (in-memory) and `iklo-substrate-turso` (Turso backend).
- **SC-002**: A restart persistence test demonstrates committed bindings and revision continuity in Turso mode.
- **SC-003**: Existing `iklo-runtime` behavior tests pass unchanged in source.
- **SC-004**: CLI default mode remains in-memory and non-persistent, while explicit Turso mode persists state.
- **SC-005**: `make test && make build && make release` succeed in a clean workspace.
- **SC-006**: A blocker inventory exists for Turso integration, with every blocker classified and linked to a concrete action (adapter fix, upstream proposal, or fork patch).
- **SC-007**: If any fork-required blockers exist, a follow-up ADR/epic is opened under ADR-0005 before any fork implementation work proceeds.
- **SC-008**: `specs/004-turso-substrate-backend/plan.md` and `tasks.md` exist before `/speckit.implement`, and include explicit FR-ID traceability that maps every functional requirement (`FR-001` onward) to one or more tasks.
- **SC-009**: Every blocker captured during implementation has complete inventory fields (ID, classification, invariant impacted, evidence, chosen action, rationale) in the canonical tracker.
- **SC-010**: Serialization contract tests cover supported persisted-`V` shapes, unknown/unsupported shape handling behavior, and migration-policy behavior for schema/data version mismatches.
- **SC-011**: Concurrency/retry behavior is validated by targeted tests for conflict and transient failure scenarios.
- **SC-012**: CLI configuration precedence and invalid-combination behavior are validated by targeted CLI tests.

## Assumptions

- Turso APIs and authentication are available in a form stable enough for a storage-backend implementation without requiring a Turso fork.
- Medium confidence is sufficient for this strategy decision; unresolved uncertainty is handled by explicit blocker classification and gated fork criteria rather than immediate fork adoption.
- Serialization strategy for Iklo runtime values can be implemented incrementally while preserving current value coverage required by tests.
- Concurrency semantics will follow Turso/SQLite transactional guarantees plus the existing `Substrate` contract.
- VDBE-targeted compiler work remains out of scope and stays gated behind a future, separate ADR.
- Any Turso fork implementation work is out of scope for this epic and requires a separate ADR/epic.
