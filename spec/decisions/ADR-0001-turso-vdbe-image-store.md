# ADR-0001 — Turso/VDBE as the eventual image substrate, behind an `ImageStore` boundary

- **Status:** Accepted
- **Date:** 2025-01
- **Deciders:** @rsenna (with Copilot as sounding board)
- **Supersedes:** —
- **Superseded by:** —

## Context

Iklo's language design (see [LANGUAGE.md](../../LANGUAGE.md), especially the
"Transactional VDBE and live image runtime" section) rests on three claims:

1. Iklo's runtime is a *persistent, transactional live image* — every top-level
   evaluation runs inside a transaction; committed state is durable; failed
   transactions never mutate the image.
2. Bindings across the five engines (`graph`, `lexical`, `dynamic`, `reactive`,
   `sync`) are updated atomically at commit boundaries.
3. Committed runtime state is queryable and can back
   graph/document/reactive workloads without a separate database.

The naïve implementation of (1)–(3) is a large amount of infrastructure:
pager + WAL + B-tree + a bytecode VM whose stepping is what defines
"transaction". [Turso](https://turso.tech/) — an in-process, SQLite-lineage
database with its own VDBE bytecode VM — already provides all of it. The
[Turso VDBE Doom demo](https://github.com/tursodatabase/turso-vdbe-doom-example)
proves VDBE is expressive enough to host a general-purpose program (Doom, via
LLVM IR → VDBE via a purpose-built `vdbecc`).

The temptation is to jump straight to "compile Iklo to VDBE bytecode". Three
observations make that premature:

- **Iklo's semantics are not yet stable.** Sigil taxonomy, `let`/`set` split,
  effect model, macro hygiene, laziness — these are being decided *right now*.
  Committing to a bytecode target before the semantic ground is settled will
  either burn the compiler or freeze the semantics prematurely.
- **VDBE was designed for SQL execution.** Its opcode set is oriented around
  cursors, sorts, joins, aggregates. To host a general-purpose language on it,
  memory becomes a single blob accessed via `BlobRead`/`BlobWrite`/`get_byte`/
  `set_byte`, and the calling convention is hand-rolled with fixed transfer
  registers + `Gosub`. This is a real compiler backend, not a "target".
- **Turso does not expose a public API for hosting foreign bytecode.** The
  Doom demo forks Turso to add a load path. Any serious VDBE adoption inherits
  that fork or blocks on upstreaming a feature.

## Decision

**Adopt Turso as the eventual substrate for Iklo's transactional live image,
but reach it via an intermediate capability boundary — an `ImageStore` trait —
rather than by committing to VDBE bytecode as the compilation target.**

Concretely:

1. Keep the tree-walking interpreter in `crates/iklo-runtime` as the *semantic
   reference*. All language behaviour is defined by what this interpreter does.
2. Refactor `iklo-runtime` so that image state (bindings across engines,
   revision counter, transaction lifecycle) is accessed only through an
   `ImageStore` trait. Provide an in-memory implementation now; keep this
   crate free of Turso, SQLite, or storage dependencies.
3. When (and only when) the semantic surface is stable enough to earn it,
   land a Turso-backed `ImageStore` implementation as a separate crate (e.g.
   `iklo-image-turso`) behind a Cargo feature flag. This unlocks persistence,
   real transactions, WAL, and queryable committed state at the language level.
4. VDBE-as-compilation-target is a *later* decision, to be reopened via a new
   ADR after the Turso-backed image store proves out. If we get there, the
   forking cost, calling convention, and TCO questions get their own ADR.

## Alternatives considered

- **A — Fork Turso now and target VDBE bytecode from day one.** Highest
  potential ceiling. Rejected: semantics too unstable to commit to a backend
  this specific; would freeze design decisions we haven't yet made; forks the
  language on someone else's roadmap; months of infrastructure before `let :x
  be 42` runs again.
- **C — Use Turso only as a language-level primitive (e.g. `//db`), never as
  a runtime substrate.** Simplest. Rejected as the *only* path: it gives up
  the "runtime data as database substrate" design intent — the image itself
  would still need reinventing pager/WAL/B-tree if we ever want real
  persistence. C is not incompatible with this ADR; a Turso-primitive module
  can still ship independently of the `ImageStore` decision.

## Consequences

- **Positive:**
  - Semantic work (macros, effects, sigils, forms) proceeds against a stable
    reference interpreter, unblocked by backend concerns.
  - The `ImageStore` boundary is useful on its own — it makes the transaction
    contract explicit and testable, and enables in-memory `ImageStore`
    implementations for fast tests.
  - When Turso is ready to be swapped in, semantics don't move.
- **Negative:**
  - We pay for the trait boundary now even if we never adopt Turso.
  - Some transactional guarantees the tree-walker can approximate cheaply may
    be harder to fake in the in-memory `ImageStore` than in Turso itself.
- **Reversal cost:** low. If we decide against Turso later, we drop the
  never-shipped `iklo-image-turso` crate; the boundary and interpreter stay.

## Follow-ups

- Author `spec/image-store/SPEC.md` — the first epic under the new spec-driven
  workflow.
- Do not write any VDBE code, or add Turso as a dependency, until the
  `ImageStore` epic has shipped and been exercised.
