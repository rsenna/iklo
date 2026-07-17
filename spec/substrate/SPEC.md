# SPEC — `substrate` epic

**Parent spec:** [../SPEC.md](../SPEC.md)
**Grounding decision:** [ADR-0001](../decisions/ADR-0001-substrate-boundary.md)
**Status:** Draft — not yet activated for `/plan`.

## Objective

Introduce a `Substrate` trait in a new `iklo-substrate` crate, refactor
`iklo-runtime` to access all image state through that trait, and ship an
in-memory implementation that preserves today's tree-walking interpreter
behavior *exactly*.

The purpose of this epic is **not** to gain a new capability. It is to install
a boundary. When it lands, Iklo does everything it did before, in the same
way, with the same tests passing. The value is what becomes possible *after*:
a Turso-backed implementation can be added later without touching the
interpreter, and until then, semantic work on the language proceeds against a
crate boundary that already knows how to keep storage decisions out of
`iklo-runtime`.

If we do this right, the diff is almost boring. That's the point.

## Why this epic exists (the honest version)

We are tempted, right now, to start writing Turso code. We shouldn't. Iklo's
semantics are moving weekly — `let`/`set`/`be`, sigils, effect model, macro
hygiene — and any storage layer we build today will have to be un-built when
those decisions change. The right move is to make the *shape* of the
substrate concrete (a trait, a set of operations, a transaction contract)
without committing to a specific implementation. That way, the semantic work
starts using the boundary immediately, and when Turso lands, it slots in
against a boundary that's been exercised.

## Success criteria

- [ ] `iklo-substrate` crate exists in the workspace and depends on **nothing
      from `iklo-runtime`, `iklo-parser`, `iklo-ast`, or `iklo-lexer`**. It
      knows about the image as an abstract shape, not about Iklo values.
- [ ] The `Substrate` trait covers, at minimum:
  - image lifecycle: `open`, `close`;
  - transaction lifecycle: `begin`, `commit`, `rollback`, and the
    revision-counter contract already implemented in `Env`;
  - binding read/write per engine (`graph`, `lexical`, `dynamic`, `reactive`,
    `sync`) — the read/write shape is uniform even if today only `lexical`
    is populated;
  - a way to observe the current revision.
- [ ] `iklo-substrate` ships an `InMemorySubstrate` (or similarly named)
      implementation that is behaviorally identical to today's `Env`.
- [ ] `iklo-runtime` no longer contains storage logic. It depends on
      `iklo-substrate` and calls only the trait. It contains no `HashMap`,
      `Vec<HashMap>`, `RefCell<...>`, or similar structures that hold binding
      state directly.
- [ ] Every existing test in `iklo-runtime` (currently
      `let_returns_bound_value` and `rollback_keeps_image_unchanged`) still
      passes, unchanged, against the trait-backed runtime.
- [ ] New tests in `iklo-substrate` cover the trait contract directly, so
      that a future `iklo-substrate-turso` can be validated against the same
      suite.
- [ ] `cargo test`, `cargo build`, and `cargo build --release` all succeed.
- [ ] LANGUAGE.md's "Transactional VDBE and live image runtime" section is
      updated to reference `Substrate` (currently references `ImageStore` —
      now stale) and to accurately describe the current state (in-memory
      substrate; Turso deferred).
- [ ] No Turso dependency is added. No VDBE code is written. Both are
      explicitly out of scope for this epic.

## Non-goals

- **Persistence.** The in-memory substrate does not survive process exit.
  That's fine; persistence lands with the Turso implementation, later.
- **Query.** The substrate exposes read/write, not query. Language-level
  query is a separate concern.
- **Performance work.** If the trait boundary makes the tree-walker slightly
  slower, we accept that; the tree-walker's job is to be *correct*, not
  fast.
- **New language features.** No new syntax, no new sigils, no new engines
  become populated as part of this epic. Only `lexical` is exercised end-to-end
  today; the other engines exist as trait surface but can remain unimplemented
  (`todo!()` or a documented "unpopulated" return).

## What "done" looks like from three angles

**For the interpreter author:**
`iklo-runtime` becomes a thinner crate that translates AST to trait calls.
Reading it should be easier, not harder, than reading it is today.

**For a future Turso backend author:**
There is a trait to implement, a test suite to pass, and no need to touch
anything in `iklo-runtime`. The Turso work becomes local.

**For a language-design contributor:**
Nothing visible changes. `iklo-cli` still runs the REPL. `examples/hello.iklo`
still prints the same result. `let :x be 21 * 2` still returns 42.

## Design notes (small; anything load-bearing becomes an ADR)

- **Trait granularity.** Start with one trait. If it grows past ~8 methods
  or picks up axes that clearly separate (e.g. storage vs. query vs.
  observation), split it — but *only* when the split is forced by a second
  implementation, not speculatively.
- **Error type.** The trait should use its own error type
  (`SubstrateError`), not reuse `iklo-runtime`'s `RuntimeError`. Runtime
  errors wrap substrate errors, not the other way around.
- **Async.** Not now. The tree-walker is synchronous; the trait is
  synchronous. If Turso pushes us toward async later, that's an ADR for
  that day.
- **Revision counter semantics.** The trait must preserve today's exact
  behavior: revision increments on commit, does not increment on rollback,
  and is observable at any point. This is already tested; keep the test.

## Open questions (resolve during `/plan`, not now)

- Does `iklo-substrate` also own the concept of a `Value`, or does the
  trait remain generic over an associated value type so `iklo-runtime`
  keeps ownership of `Value`?
- Should the in-memory implementation live in `iklo-substrate` itself, or
  in a small sibling crate (`iklo-substrate-memory`) so that
  `iklo-substrate` is truly interface-only?
- How do we express the transaction contract in the trait signature so
  that "you cannot commit a transaction you did not open" is a type-level
  guarantee, not a runtime check? (May be too clever for v1 — worth an
  option, not necessarily a requirement.)

These are questions for `/plan` to answer with the person doing the work.
They are listed here so they don't get lost.
