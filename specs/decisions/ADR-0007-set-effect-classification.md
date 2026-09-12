# ADR-0007 — `set` is an effect; `let` is pure only for the lexical engine

- **Status:** Proposed — drafted from the epic 006 design note's open
  question §4.4. Accepting this ADR means editing this line to `Accepted`
  (with the acceptance date), as part of — not automatically implied by —
  merging this PR; a merge that leaves this line reading `Proposed` has not
  accepted it.
- **Date:** 2026-09-12
- **Deciders:** @rsenna (with Claude as sounding board)
- **Supersedes:** —
- **Superseded by:** —

## Decision (one sentence)

**`set :x to <expr>` (mutating an existing binding) is classified as an
effect, unconditionally and independent of whether the enclosing
transaction ever commits; `let :x be <expr>` (introducing a new binding) is
pure only when it targets the lexical engine — matching ADR-0006's
engine-scoped `let` rule, restated here for completeness.**

## Context

The epic 006 design note
([`specs/006-strictness-effects-spike/design-note.md`](../006-strictness-effects-spike/design-note.md))
§4.4 asked whether `set` is "an effect" in the type-system sense, or merely
a binding operation that leaves a calling form pure. It flagged a specific
wrinkle: `set` performed inside a transaction that later rolls back "was
never observable" — so does that make it not-quite-an-effect after all?
This blocks epic 008 (binding model taxonomy), which must classify `set`
before it can ratify a binding-mode vocabulary, and epic 009 (binding
kinds), which implements the mutable engines `set` targets.

This is a narrow decision, deliberately split out of ADR-0006: ADR-0006
already decided *how* purity works in general (evaluation-based: no
observable mutation, no boundary-crossing) and, as a corollary, that a
`let` targeting a mutable engine is effectful because it commits engine
state — the pure classification is scoped to the lexical engine. What
ADR-0006 left open is `set` specifically, and the transaction-rollback
question the design note raised about it.

`LANGUAGE.md` already states the mechanical rule in prose: "`set` mutates
existing bindings" and only reaches "the mutable engines (`graph`,
`dynamic`, `reactive`, `sync`)" — `set` on a plain lexical binding is an
error (`AGENTS.md`). This ADR settles the *type-system* classification, not
the mechanics, which were never in question.

## What this commits us to

### 1. `set` is always an effect

- `set :x to <expr>` is classified **effectful (mutation)**, full stop. Any
  form whose evaluation calls `set` is impure — there is no condition under
  which a `set`-calling form is nonetheless pure.
- This is unconditional on **which** mutable engine is targeted
  (`graph` / `dynamic` / `reactive` / `sync`) and on **whether `<expr>`
  itself is pure** — evaluating `<expr>` may be pure, but the `set` that
  consumes its result is not.

### 2. Transaction rollback does not change the classification

- Per ADR-0006, purity is a property of **evaluation**, determined
  statically (at the point a form is type-checked), not of eventual
  **runtime outcome**. Whether the transaction containing a `set` commits or
  is rolled back (`tx.rollback`, or an implicit top-level rollback on
  failure) is known only at runtime, after evaluation — it cannot retroactively
  change a classification decided before that outcome exists.
- Concretely: `set` **attempts** a mutation the moment it evaluates, inside
  the implicit or explicit transaction in scope (`LANGUAGE.md`
  §"Transaction contract": "Every top-level eval runs inside an implicit
  transaction"). That attempt is the effect being classified — not whether
  the image ends up reflecting it. A rolled-back `set` was still evaluated
  as a mutation attempt; it is effectful by the same reasoning a `run`
  whose action later errors is still effectful, not retroactively pure.
- Treating rollback as purity-restoring would make purity a runtime,
  data-dependent property — exactly what ADR-0006 rules out. It would also
  be unsound for a type checker: whether a transaction rolls back can
  depend on other bindings' state, not on `set`'s own arguments, so no
  static rule could ever correctly grant "purity on rollback."

### 3. `let` is pure only for the lexical engine (restated, not re-decided)

- Consistent with ADR-0006 §"Effects are a marker type" (its `let` note):
  lexical `let :x be <expr>` is pure; a `let` that introduces a binding
  into a mutable engine (e.g. graph `let ^bool :x be …`) is effectful,
  because introducing a binding there requires the same engine-level commit
  a mutation does. Restated here so this ADR gives a complete `let`/`set`
  picture for epic 008 to build on, without re-opening ADR-0006.
- The asymmetry is intentional and not arbitrary: `let` *can* target the
  lexical engine (where introduction is free of engine commit), but `set`
  *cannot* — `AGENTS.md`'s non-negotiable rule is that `set` on a plain
  lexical binding is an error. `set` therefore never has a pure case to
  begin with; `let` does.

## Non-decisions

- Does **not** define the full binding-mode vocabulary or the `Engine`
  column mapping from `LANGUAGE.md` — that is epic 008's own job; this ADR
  only unblocks it.
- Does **not** decide whether other transaction forms (`tx.begin`,
  `tx.commit`, `tx.rollback`, `tx.retry`) are themselves effects at the
  language-surface level (as opposed to runtime-internal transaction
  management) — out of scope; presumed effectful by the same evaluation
  rule but not itemized here.
- Does **not** add enforcement to a type checker — none exists yet (per
  ADR-0006's Non-decisions). This ADR fixes the classification so epic 008
  and any later checker build to the same target.
- Does **not** revisit ADR-0006's `let`-into-mutable-engine rule; §3 above
  restates it, not re-decides it.

## Consequences

- **Positive**
  - Epic 008 can classify `set` in its binding-mode taxonomy without
    ambiguity, and epic 009 can implement mutable binding kinds against a
    fixed effect rule.
  - The rule is simple to state and to check: `set` anywhere in a form's
    body makes that form impure, full stop — no transaction-outcome
    tracking needed for purity classification.
  - Resolves the design note's own open wrinkle (rollback) rather than
    leaving it to resurface later, e.g. once epic 009 implements real
    rollback-capable engines and someone asks "but is it *really* impure if
    it rolled back?"
- **Negative**
  - No "pure by luck" allowance for a `set` inside a transaction that the
    author knows will roll back (e.g. a speculative/retry pattern) — such a
    form is impure by classification even if, on a given run, the image
    never observably changes. Judged acceptable: static purity must not
    depend on runtime outcome, and the alternative (outcome-dependent
    purity) is unsound for any future type checker.
  - Authors who want a "try a mutation, roll back, stay pure" pattern must
    express it some other way (e.g. as an `^action` that a caller
    explicitly `run`s) rather than relying on `set` retaining purity.

## Follow-ups

- Epic 008 (binding model taxonomy) classifies `set` as an effect in its
  vocabulary and documents the `Engine` column mapping this ADR does not
  cover.
- Epic 009 (binding kinds implementation) implements the mutable engines
  `set` targets against this rule.
- Update [`specs/execution-queue.md`](../execution-queue.md) epic 008 (and
  epic 009's `set` portion) start criteria to name **ADR-0007** (this ADR)
  in place of the interim "ADR 4.4" wording, once accepted.
