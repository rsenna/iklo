# Implementation Plan: Substrate Capability Boundary

**Branch**: `001-substrate` | **Date**: 2026-07-17 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/001-substrate/spec.md`

## Summary

Introduce a new `iklo-substrate` crate exposing a `Substrate` trait and an
in-memory implementation, then refactor `iklo-runtime` to route all binding
state through the trait. Public API of `iklo-runtime` stays wire-compatible;
`iklo-cli` is untouched. No Turso, no VDBE, no persistence in this epic —
those land later against a boundary that already exists.

Technical approach: generic-parameterised trait (`type Value: Clone + Debug`),
in-memory impl as a module inside `iklo-substrate` (not a separate crate),
transaction ownership enforced by `self`-consuming methods rather than
lifetimes.

## Technical Context

**Language/Version**: Rust, edition 2021, workspace `resolver = "2"`.

**Primary Dependencies**: None new. `iklo-substrate` uses only `std`. `iklo-runtime` gains `iklo-substrate = { path = "../iklo-substrate" }`.

**Storage**: `HashMap<String, V>` inside `InMemorySubstrate<V>`; no persistence.

**Testing**: Rust built-in `#[test]`, inline `#[cfg(test)] mod tests` per crate. `cargo test` / `make test`.

**Target Platform**: Same as workspace (native, whatever `mise.toml` pins).

**Project Type**: Compiler/interpreter — a Rust workspace of small crates.

**Performance Goals**: None. Correctness only. Trait indirection is acceptable overhead.

**Constraints**: The refactor must preserve the two existing `iklo-runtime` tests **unchanged in source**. If either test has to change, the boundary is leaky and the plan needs revision.

**Scale/Scope**: ~150 lines of code moved and reshaped across two crates; ~7 new contract tests; docs updates in AGENTS.md and LANGUAGE.md.

## Constitution Check

Verified against [`.specify/memory/constitution.md`](../../.specify/memory/constitution.md):

- **I. Test-First** ✅ — every task in `tasks.md` writes tests (or a failing scaffold) before implementation. Contract tests in T3 are written before `InMemorySubstrate` is fleshed out.
- **II. One Epic In Flight** ✅ — this is the first epic under `specs/`; no others active.
- **III. Substrate Before Feature** ✅ — this epic *is* the substrate. It exists specifically to install the boundary before adding Turso (the second consumer).
- **IV. Kebab-Case Iklo, Idiomatic Rust** ✅ — Rust code stays `snake_case`/`PascalCase`. Iklo syntax is unaffected.
- **V. Comments Justify Themselves** ✅ — doc comments on the public trait; nothing added to internal helpers unless the *why* is non-obvious.
- **VI. ADRs for Load-Bearing Decisions** ✅ — grounded in [ADR-0001](../../specs/decisions/ADR-0001-substrate-boundary.md). Any new load-bearing sub-decision during implementation becomes ADR-0002.
- **VII. No Workarounds Left Standing** ✅ — the refactor does not leave a scaffold behind; if the boundary doesn't hold on the first pass, we fix the boundary rather than papering over it in `iklo-runtime`.

No violations; Complexity Tracking section left empty.

## Project Structure

### Documentation (this feature)

```text
specs/001-substrate/
├── plan.md              # This file
├── spec.md              # Feature spec
└── tasks.md             # Executable task list (from /speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── iklo-substrate/            # NEW — the capability boundary
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs             # Substrate + Transaction traits, SubstrateError
│       └── memory.rs          # InMemorySubstrate<V> + contract tests
├── iklo-runtime/              # REFACTORED — thin façade over substrate
│   ├── Cargo.toml             # + iklo-substrate = { path = "../iklo-substrate" }
│   └── src/
│       └── lib.rs             # RuntimeImage delegates to InMemorySubstrate<Value>;
│                              # tree-walker's Transaction wraps substrate::Tx
├── iklo-lexer/                # UNCHANGED
├── iklo-ast/                  # UNCHANGED
├── iklo-parser/               # UNCHANGED
└── iklo-cli/                  # UNCHANGED (public runtime API stays wire-compatible)

tests/                         # No dedicated dir; inline #[cfg(test)] per crate.
```

**Structure Decision**: Single-project Rust workspace. New crate slotted in
next to existing ones under `crates/`. In-memory implementation as a `memory`
module inside `iklo-substrate` rather than a separate `iklo-substrate-memory`
crate — extracting later is cheap; splitting speculatively is not.

## Complexity Tracking

*No violations to track.*

## Design Notes (post-spec elaboration)

Not part of the Constitution Check but useful for future readers:

### Trait shape (settled after PR #1 review — no `open`/`close`)

```rust
pub trait Substrate {
    type Value: Clone + std::fmt::Debug;
    type Tx<'a>: Transaction<Value = Self::Value> where Self: 'a;

    // Creation is via each impl's own constructor (e.g. `InMemorySubstrate::new()`),
    // not a trait method — different backends need different parameters.
    // Teardown is via `Drop`, not a `close` method — guaranteed cleanup on panic.

    fn begin(&mut self) -> Self::Tx<'_>;
    fn revision(&self) -> u64;
    fn snapshot(&self) -> HashMap<String, Self::Value>;
}

pub trait Transaction {
    type Value;
    fn get(&self, name: &str) -> Option<Self::Value>;
    fn set(&mut self, name: &str, value: Self::Value);
    fn commit(self) -> Result<(), SubstrateError>;
    fn rollback(self) -> Result<(), SubstrateError>;
}
```

Compile-time safety properties (per FR-002, FR-009, and the spec's edge-cases
section):

- `commit(self)` / `rollback(self)` consume the transaction — double-finalise
  won't compile.
- `begin(&mut self) -> Tx<'_>` reborrows the substrate mutably — a second
  `begin`, `snapshot`, or `revision` call won't compile while the tx is live.
  Callers who want a pre-transaction snapshot take it before calling `begin`.

### `bindings()` signature — decided (owned `HashMap`)

`RuntimeImage::bindings()` returns owned `HashMap<String, Value>`, materialised
on demand from `substrate.snapshot()`. This closes the T2 investigation
without leaking in-memory storage details (a future Turso-backed substrate
cannot return a reference to an internal HashMap it does not hold). Existing
runtime tests using `image.bindings().get("x")` continue to compile
**unchanged in source**, satisfying SC-001: the temporary map lives until the
end of the statement.

Decided via reviewer feedback from `gemini-code-assist` on PR #1.

### Contract-test shape — decided (trait-generic)

Contract tests live behind a generic function:

```rust
pub fn run_contract_suite<S: Substrate<Value = i64>>(make: impl Fn() -> S) {
    // ... the 7 named scenarios, driven only through the trait surface.
}
```

Each `#[test]` in `iklo-substrate` is a two-line harness calling
`run_contract_suite(InMemorySubstrate::<i64>::new)`. A future
`iklo-substrate-turso` adds its own harness alongside; the contract cases
themselves are reused verbatim. This satisfies FR-010 in a checkable way.

### Risk: `Value` generic virusing signatures

If the `<V>` parameter starts appearing in awkward `iklo-runtime` signatures,
add a type alias at the top of the crate:

```rust
type IkloSubstrate = iklo_substrate::memory::InMemorySubstrate<Value>;
```

Expect to add this in T4.

### What we are explicitly NOT doing

- No Turso dependency, no VDBE, no persistence, no query layer.
- No new sigils, forms, or engines. `graph` / `dynamic` / `reactive` / `sync`
  get trait surface only — not populated implementations.
- No performance work. Trait indirection cost is accepted.
- No `async`. Sync throughout.
