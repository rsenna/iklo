# Feature Specification: Substrate Capability Boundary

**Feature Branch**: `001-substrate`

**Created**: 2026-07-17

**Status**: Draft

**Input**: Introduce a `Substrate` trait in a new `iklo-substrate` crate, refactor `iklo-runtime` to access all image state through it, and ship an in-memory implementation that preserves today's tree-walking interpreter behaviour exactly. Ground: [ADR-0001](../../specs/decisions/ADR-0001-substrate-boundary.md).

## User Scenarios & Testing *(mandatory)*

The "users" here are Iklo *contributors*. This is a refactoring epic; end-user
behaviour does not change. Each story is a developer journey.

### User Story 1 - Language-design contributor sees no change (Priority: P1) 🎯 MVP

A contributor working on grammar, evaluation semantics, or the REPL uses Iklo
exactly as before. `cargo run -p iklo-cli`, `let :x be 21 * 2`, `.env`,
`.revision`, `examples/hello.iklo` — all identical outputs, identical prompts,
identical behaviour. The refactor is invisible to them.

**Why this priority**: This is the load-bearing invariant. If we break it, we
have made the language worse to justify future capability. Every other story
depends on this one holding.

**Independent Test**: Run the existing `iklo-runtime` test suite unchanged; it
passes. Run `examples/hello.iklo`; the output byte-matches the pre-epic
snapshot. Manual REPL smoke: `let :x be 42`, `:x`, `.env` shows `x = 42`,
`.revision` shows `1`.

**Acceptance Scenarios**:

1. **Given** an untouched `iklo-runtime` test file, **When** `cargo test -p iklo-runtime` runs, **Then** `let_returns_bound_value` and `rollback_keeps_image_unchanged` both pass without any change to their source.
2. **Given** a fresh REPL session, **When** the user runs `let :x be 21 * 2` and then `.env`, **Then** the environment shows `x = 42` and `.revision` returns `1`.
3. **Given** `examples/hello.iklo`, **When** run via `cargo run -p iklo-cli -- examples/hello.iklo`, **Then** the output is identical to the pre-epic run.

---

### User Story 2 - Future substrate-backend author has a trait to implement (Priority: P2)

A contributor arriving later to add a Turso-backed image implementation opens
the `iklo-substrate` crate, reads one trait definition, and knows exactly what
they must implement. They do not have to read `iklo-runtime` to understand the
image contract. They can validate their implementation against a reusable
contract test suite that lives in `iklo-substrate`.

**Why this priority**: The whole *point* of this epic is to make the second
implementation cheap. If the boundary isn't clean enough that a new backend
can be added without touching `iklo-runtime`, we've failed the intent.

**Independent Test**: A reviewer reading only `crates/iklo-substrate/src/lib.rs`
can list the operations a backend must implement, the transaction contract,
and the revision semantics. The contract test suite is written against the
`Substrate` trait; a small harness at the boundary selects the concrete
implementation under test.

**Acceptance Scenarios**:

1. **Given** the `iklo-substrate` crate as shipped, **When** a reader inspects `Cargo.toml`, **Then** it depends on **nothing** from `iklo-runtime`, `iklo-parser`, `iklo-ast`, or `iklo-lexer`.
2. **Given** the contract test suite in `iklo-substrate` (a generic function over `S: Substrate<Value = i64>`), **When** it is instantiated with `InMemorySubstrate<i64>`, **Then** all cases pass; **When** the same suite is (in principle) instantiated with a hypothetical `TursoSubstrate<i64>`, **Then** no case body needs to change — only the harness that supplies the implementation.

---

### User Story 3 - Interpreter author reads a thinner runtime (Priority: P3)

A contributor debugging or extending the tree-walker opens
`crates/iklo-runtime/src/lib.rs` and finds a file that translates AST nodes to
substrate calls. There are no `HashMap`s, no `Vec<HashMap>`s, no `RefCell`s
holding binding state. Storage decisions have left the file.

**Why this priority**: Nice to have; falls out of P1+P2 if done well.
Explicit here as a lint on the outcome — if `iklo-runtime` still holds
storage code after the refactor, the boundary is leaky.

**Independent Test**: `grep -E 'HashMap|Vec<HashMap>|RefCell' crates/iklo-runtime/src/` returns no matches for binding storage (uses inside test-only or unrelated code may remain, but must be justified).

**Acceptance Scenarios**:

1. **Given** the post-refactor `iklo-runtime` source, **When** a reader searches for direct storage types holding binding state, **Then** none are found; all such state lives behind the substrate trait.

---

### Edge Cases

- **What happens on rollback failure?** The substrate's `rollback(self)` consumes the transaction; it cannot itself fail in the in-memory case (dropping cloned state). If a future backend can fail on rollback, the trait signature already returns `Result<(), SubstrateError>` (see design notes).
- **What happens if two transactions are begun concurrently?** Statically prevented by the borrow checker: `Substrate::begin(&mut self)` returns a `Tx<'_>` that reborrows the substrate, so a second `begin` won't compile while the first is live.
- **What happens when `snapshot()` is called mid-transaction?** Not permitted. `begin` takes `&mut self` and `snapshot` takes `&self`; calling `snapshot` while a transaction is open won't compile. This is a feature, not a bug — it makes "am I reading committed or uncommitted state?" a compile-time question instead of a runtime hazard. Callers who want a pre-transaction view snapshot *before* calling `begin`.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A new crate `iklo-substrate` MUST exist in the workspace and MUST depend on **nothing** from `iklo-runtime`, `iklo-parser`, `iklo-ast`, or `iklo-lexer`.
- **FR-002**: `iklo-substrate` MUST expose a `Substrate` trait with methods covering: transaction lifecycle (`begin`, `commit`, `rollback`), revision observation, and a snapshot of committed bindings. Creation is via concrete constructors on each implementation (not a trait method); teardown is via the `Drop` trait (not a `close` method) — both idiomatic Rust and per reviewer feedback from gemini-code-assist on PR #1.
- **FR-003**: `iklo-substrate` MUST expose a `Transaction` associated trait with `get(name)`, `set(name, value)`, `commit(self)`, and `rollback(self)`.
- **FR-004**: `iklo-substrate` MUST ship an `InMemorySubstrate<V>` implementation, generic over the value type, behaviourally identical to today's `Env`.
- **FR-005**: `iklo-substrate` MUST expose a `SubstrateError` type used by all fallible trait methods; it MUST NOT depend on `RuntimeError`.
- **FR-006**: `iklo-runtime` MUST NOT contain direct storage types (`HashMap`/`Vec<HashMap>`/`RefCell<HashMap>`) that hold binding state; all binding state MUST live behind `Substrate`.
- **FR-007**: `iklo-runtime` MUST preserve its current public API surface (`RuntimeImage::new`, `.revision()`, `.eval_in_tx(&Program)`, `.bindings()`) so that `iklo-cli` and future consumers are unaffected. **`.bindings()`'s return type changes from `&HashMap<String, Value>` to owned `HashMap<String, Value>`** — materialised on the fly from the substrate's `snapshot()`. Existing tests using `image.bindings().get("x")` continue to compile because the temporary lives until the end of the expression.
- **FR-008**: `RuntimeError` MUST gain a variant (or `From` impl) that wraps `SubstrateError`.
- **FR-009**: The revision counter contract MUST be preserved: revision starts at 0, increments on commit, does not increment on rollback, and is observable at any time (outside of an open transaction — see edge cases).
- **FR-010**: A contract test suite MUST live in `iklo-substrate` written **generically over the `Substrate` trait** (a function `run_contract_suite<S: Substrate<Value = i64>>(make: impl Fn() -> S)` or equivalent). Only a thin harness (the `#[test]` functions themselves) references a concrete implementation. This is what makes the suite reusable for a future Turso backend.
- **FR-011**: `make test`, `make build`, and `make release` MUST all succeed after the refactor.
- **FR-012**: No Turso dependency MAY be added and no VDBE code MAY be written in this epic; both are explicitly deferred.

### Key Entities

- **Substrate**: The capability boundary hiding *where* the image lives. Generic over an associated `Value` type (bounded on `Clone + Debug`) so it never sees Iklo `Value`.
- **Transaction**: A short-lived handle representing an in-progress mutation of the image. Owned by-value so that `commit(self)` / `rollback(self)` statically prevent reuse. Reborrows the substrate mutably (`Tx<'_>`) so no second transaction can start while it lives.
- **Image**: The runtime state (bindings, revision) that a substrate holds. Not a type — a concept realised by whatever `Substrate` implementation is active.
- **InMemorySubstrate\<V\>**: The reference implementation. HashMap + revision counter. Transactions clone bindings and write back on commit.
- **SubstrateError**: The trait's error type. Runtime errors wrap substrate errors, never the other way around.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001** ✅: `cargo test -p iklo-runtime` passes with both existing tests **unchanged in source**.
- **SC-002** ✅: `cargo test -p iklo-substrate` passes at least 7 contract tests covering: revision-starts-at-zero, commit-increments-revision, rollback-does-not-increment-revision, get-after-set-inside-tx-sees-value, get-after-rollback-does-not-see-value, get-after-commit-sees-value-from-fresh-tx, snapshot-returns-only-committed-state.
- **SC-003** ✅: `cargo tree -p iklo-substrate` shows zero workspace-internal dependencies (only stdlib and any third-party deps deliberately added).
- **SC-004** ✅: A `grep` for `HashMap|Vec<HashMap>|RefCell<HashMap>` in `crates/iklo-runtime/src/` returns no matches related to binding storage.
- **SC-005** ✅: `examples/hello.iklo` output byte-matches the pre-epic snapshot.
- **SC-006** ✅: `make test && make build && make release` all exit 0.
- **SC-007** ✅: LANGUAGE.md and AGENTS.md reflect the new state — a fresh reader learns nothing false about where the runtime image lives.

## Assumptions

- The `Substrate` trait is generic over an associated `Value` type; `iklo-runtime` plugs its own `Value` in. `iklo-substrate` never sees Iklo `Value`.
- The in-memory implementation lives inside `iklo-substrate` as a `memory` module. Extracting to `iklo-substrate-memory` is speculative until a second implementation justifies it.
- Transaction safety is enforced at **compile time**, not runtime: `commit(self)` / `rollback(self)` are self-consuming (so double-finalisation won't compile), and `begin(&mut self) -> Tx<'_>` mutably borrows the substrate (so a second `begin` or any `&self` method — including `snapshot` and `revision` — won't compile while a transaction is live). No lifetime tricks beyond this are required.
- The trait is synchronous. Async lands only if a future backend forces it, which is an ADR for that day.
- The mutable engines beyond `lexical` (`graph`, `dynamic`, `reactive`, `sync`) get trait *surface* only — no populated implementations. `let` on `lexical` is the only exercised path today.
