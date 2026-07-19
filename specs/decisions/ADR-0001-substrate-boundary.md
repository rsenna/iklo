# ADR-0001 — Defer the substrate choice via a `Substrate` capability boundary

- **Status:** Accepted
- **Date:** 2025-01
- **Deciders:** @rsenna (with Copilot as sounding board)
- **Supersedes:** —
- **Superseded by:** —

## Decision (one sentence)

**We are deferring the choice of runtime substrate by introducing a `Substrate`
capability boundary in `iklo-runtime` now, because Iklo's language semantics
are not yet stable enough to commit to any specific backend — and we want the
option to adopt [Turso](https://turso.tech/) (and, later, VDBE bytecode)
without freezing semantics around it.**

Everything else in this ADR is either the reasoning behind that sentence, the
alternatives we rejected, or the follow-ups it commits us to.

## Vocabulary

Two nouns that sound alike and are not:

- **Image** — the runtime state of Iklo: bindings across all engines, compiled
  code, macro definitions, type descriptors, annotation graph, metadata.
  Singular. Live. Grows as you use the REPL. Inherited from the
  Smalltalk/Lisp-Machine lineage; already load-bearing terminology in
  [LANGUAGE.md](../../LANGUAGE.md).
- **Substrate** — the *interface* that hides where the image lives. The image
  is the *what*; the substrate is the *what-it-runs-on*. Reader intuition:
  *"the substrate hosts the image."*

An earlier draft of this ADR called the boundary `ImageStore`. That name
invited the wrong reader intuition ("a store of images") and was replaced.

## Context

Iklo's language design (see [LANGUAGE.md](../../LANGUAGE.md), especially the
"Transactional VDBE and live image runtime" section) rests on three claims:

1. Iklo's runtime is a *persistent, transactional live image* — every top-level
   evaluation runs inside a transaction; committed state is durable; failed
   transactions never mutate the image.
2. Bindings across the five engines (`graph`, `lexical`, `dynamic`, `reactive`,
   `sync`) are updated atomically at commit boundaries.
3. Committed runtime state is queryable and can back graph/document/reactive
   workloads without a separate database.

Implementing (1)–(3) from scratch is a large amount of infrastructure —
pager, WAL, B-tree, a bytecode VM whose stepping is what defines
"transaction". [Turso](https://turso.tech/) — an in-process, SQLite-lineage
database with its own VDBE bytecode VM — already provides all of it. The
[Turso VDBE Doom demo](https://github.com/tursodatabase/turso-vdbe-doom-example)
proves VDBE is expressive enough to host a general-purpose program (Doom,
via LLVM IR → VDBE via a purpose-built `vdbecc`).

The temptation is to jump straight to "compile Iklo to VDBE bytecode." Three
observations make that premature:

- **Iklo's semantics are not yet stable.** Sigil taxonomy, `let`/`set` split,
  effect model, macro hygiene, laziness — these are being decided *right
  now*. Committing to a bytecode target before the semantic ground is settled
  will either burn the compiler or freeze the semantics prematurely.
- **VDBE was designed for SQL execution.** Its opcode set is oriented around
  cursors, sorts, joins, aggregates. To host a general-purpose language on
  it, memory becomes a single blob accessed via `BlobRead`/`BlobWrite`/
  `get_byte`/`set_byte`, and the calling convention is hand-rolled with fixed
  transfer registers + `Gosub`. This is a real compiler backend, not a
  "target."
- **Turso does not expose a public API for hosting foreign bytecode.** The
  Doom demo forks Turso to add a load path. Any serious VDBE adoption
  inherits that fork or blocks on upstreaming a feature.

## What the decision commits us to

1. Keep the tree-walking interpreter in `crates/iklo-runtime` as the
   **semantic reference**. All language behaviour is defined by what this
   interpreter does — no other implementation gets to disagree with it.
2. Introduce an `iklo-substrate` crate that defines the `Substrate` trait
   plus an in-memory implementation. Refactor `iklo-runtime` so that image
   state (bindings across engines, revision counter, transaction lifecycle)
   is accessed only through `Substrate`. `iklo-runtime` must not depend on
   Turso, SQLite, or any storage crate.
3. When (and only when) the semantic surface is stable enough to earn it,
   land a Turso-backed `Substrate` implementation as a separate crate
   (`iklo-substrate-turso`) behind a Cargo feature flag. This unlocks
   persistence, real transactions, WAL, and queryable committed state at the
   language level.
4. **VDBE-as-compilation-target is a later, separate decision.** It gets its
   own ADR after the Turso-backed substrate proves out. Forking cost, calling
   convention, and TCO all belong to *that* decision, not this one.

## Alternatives considered

- **A — Fork Turso now and target VDBE bytecode from day one.** Highest
  potential ceiling. Rejected: semantics too unstable to commit to a backend
  this specific; would freeze design decisions we haven't yet made; couples
  the language to someone else's roadmap; months of infrastructure before
  `let :x be 42` runs again.
- **C — Use Turso only as a language-level primitive (e.g. `//db`), never as
  a runtime substrate.** Simplest. Rejected as the *only* path because it
  gives up the "runtime data as database substrate" design intent — the image
  itself would still need someone to reinvent pager/WAL/B-tree if we ever
  want real persistence. C is *not* incompatible with this ADR; a
  Turso-primitive module can still ship independently of `Substrate`.

## Consequences

- **Positive:**
  - Semantic work (macros, effects, sigils, forms) proceeds against a stable
    reference interpreter, unblocked by backend concerns.
  - The `Substrate` boundary is useful on its own — it makes the transaction
    contract explicit and testable, and enables fast in-memory
    implementations for tests.
  - When Turso is ready to be swapped in, semantics don't move.
- **Negative:**
  - We pay for the trait boundary now even if we never adopt Turso.
  - Some transactional guarantees the tree-walker can approximate cheaply may
    be harder to fake in the in-memory `Substrate` than in Turso itself.
- **Reversal cost:** low. If we decide against Turso later, we drop the
  never-shipped `iklo-substrate-turso` crate; the trait and interpreter stay.

## Follow-ups

- Author [`specs/001-substrate/spec.md`](../../specs/001-substrate/spec.md) — the first epic
  under the new spec-driven workflow.
- Do not write any VDBE code, or add Turso as a dependency, until the
  `substrate` epic has shipped and been exercised.

## Status note

**Decision (2026-07):** The technical blockers above still hold — checked
against Turso's own July 2026 positioning:

- No public/stable API for hosting foreign bytecode. Turso says its VDBE
  bytecode language "is not *exposed* or *specified*, nor is it modularized";
  the [Doom demo](https://turso.tech/blog/running-unmodified-doom-in-the-sqlite-bytecode-language)
  remains a *forked* proof-of-concept (a purpose-built C→VDBE compiler).
- The VDBE is still SQL-shaped — "not a general purpose language like the JVM,
  .NET or WASM" — though Turso now frames it aspirationally as
  ["the LLVM of databases"](https://turso.tech/blog/a-new-modern-version-of-postgres-in-rust)
  (a shared backend for many *SQL/DB* frontends, not general-purpose languages).
- Turso itself is pre-release: "a foundation, not a finished product," with no
  published packages yet (build-from-source only).

**What changed is our appetite, not the facts.** Iklo is a research vehicle as
much as a product, so the immaturity, the SQL-shaped VM, and even the prospect
of *forking or upstreaming Turso* are now judged acceptable — and contributing
changes back to Turso is viewed as a **plus**, not merely a cost. This tempers
the original rejection of Alternative A: its cautions (premature commitment,
coupling to someone else's roadmap) still apply, but the "months of
infrastructure" and "someone else's roadmap" costs weigh less for a research
tool that welcomes upstream contribution.

**What still stands (unchanged):** the sequencing. The tree-walker remains the
semantic reference; the `Substrate` boundary + in-memory impl come first; a
Turso-backed `Substrate` — **not** the bytecode VM — is the next Turso milestone;
and VDBE-as-compilation-target remains a separate, later ADR. This note does
**not** authorise adding Turso as a dependency yet (see Follow-ups). Actually
advancing the timeline — adopting a Turso-backed substrate before the substrate
epic is exercised, or starting VDBE work — warrants its own ADR that supersedes
the relevant commitments here.
