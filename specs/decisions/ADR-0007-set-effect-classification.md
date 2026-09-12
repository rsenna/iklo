# ADR-0007 — `let` is lexical-only (always pure); `set` is the sole mutable-engine write path (always an effect)

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

**`let` is restricted to the lexical engine only — it can no longer target
`graph` / `dynamic` / `reactive` / `synchronized` — which makes lexical
`let :x be <expr>` unconditionally pure whenever `<expr>` is pure, with no
engine-dependent exception left; `set` becomes the sole write path for
every mutable engine, creating the binding if absent or mutating it if
present (upsert either way), and is classified as an effect
unconditionally, independent of whether the enclosing transaction ever
commits.**

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
observable mutation, no boundary-crossing). Its own draft carried a
corollary — a `let` targeting a mutable engine is effectful because it
commits engine state — as a hedge on top of that general rule. Working
through *why* that corollary held surfaced a cleaner option: instead of
`let`'s purity depending on which engine it targets, restrict `let` to the
one engine where introduction never touches shared, mutable state at all.
That is what this ADR decides — not merely restating ADR-0006's corollary,
but replacing it with a syntactic restriction that removes the
engine-dependent hedge entirely.

Previously, `AGENTS.md`'s non-negotiable rule read: "`set` mutates an
existing binding; `let` introduces a new one … `set` should only reach the
mutable engines (graph / dynamic / reactive / synchronized); `set` on a
plain lexical binding is an error." That rule described introduce-vs-mutate
as the split, with either verb able to reach any engine except that `set`
was barred from lexical. This ADR changes the split itself to
engine-vs-engine — `let` ↔ lexical, `set` ↔ every mutable engine — and,
per `AGENTS.md`'s own instruction that such a non-negotiable rule requires
an ADR to revisit, this is that ADR. §4 below states the amended rule.

## What this commits us to

### 1. `set` is always an effect

- `set $a to <expr>` (`LANGUAGE.md`'s own example, dynamic engine) is
  classified **effectful (mutation)**, full stop. Any form whose evaluation
  calls `set` is impure — there is no condition under which a `set`-calling
  form is nonetheless pure. `:x` denotes a *lexical* reference (`AGENTS.md`:
  "`:name` is the lexical-value sigil"), so `set` never targets `:x` — using
  it as a `set` example would itself violate this ADR's own rule that `set`
  on a lexical binding is an error.
- This is unconditional on **which** mutable engine is targeted
  (`graph` / `dynamic` / `reactive` / `synchronized` — `LANGUAGE.md`'s
  Transaction contract section abbreviates the last as `sync`; epic 008
  settles the canonical name), on **whether `<expr>` itself is pure** —
  evaluating `<expr>` may be pure, but the `set` that consumes its result is
  not — and on **whether the target binding already exists**. `set` is an
  **upsert**: it creates the binding in that engine if absent, or mutates
  it if present, and both cases are effectful identically. This is a
  deliberate change from `set`'s prior "must already exist" mechanics
  (§4 below) — `set` is now the *only* way to write a mutable-engine
  binding at all, so it has to cover first introduction too, not just
  updates to something `let` already created there.
- **`set` is exempt from the `^action ^t` pattern, deliberately.**
  ADR-0006 models IO effects (`echo`, `cp`, file/network forms) as building
  an `^action ^t` value that is pure to construct and effectful only when
  `run`/`do` executes it. `set` does not follow that pattern: it mutates
  the binding **immediately, during ordinary evaluation**, gated by the
  transaction contract (§2 below), not by an effect boundary. This is not
  an oversight — binding mutation is a language primitive tied to the
  transactional image, not a composable IO operation with a meaningful
  "build now, run later" split (there is no useful value to hand around
  representing "the intent to `set` `$a`" the way `cp "a" "b"` usefully
  represents "the intent to copy"). `set` was already classified this way
  in the design note's "Currently implemented" table, before any `^action`
  discussion existed — this ADR keeps it there rather than retrofitting it
  into the newer model.

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

### 3. `let` targets the lexical engine only — and is therefore unconditionally pure

- **`let` is restricted to the lexical engine.** It can no longer be
  written against `graph` / `dynamic` / `reactive` / `synchronized` at
  all — those engines are written exclusively through `set` (§1, §4). This
  is a real narrowing of `let`'s syntax, not merely a purity reclassification:
  the design note's own "Graph `let ^bool :x be …`" example (§6) is no
  longer valid syntax under this ADR — see §4's `LANGUAGE.md`/design-note
  fixes.
- Because `let` can only ever introduce a **private, per-scope, never-mutated**
  lexical name, it never has an engine-level commit to account for — there
  is no longer an engine-dependent case to hedge on. `let :x be <expr>` is
  pure **iff `<expr>` is itself pure** — nothing else to check. `let :x be
  run :copy` is impure not because of anything about the lexical engine,
  but because `run` crosses an effect boundary during evaluation regardless
  of where the result would land.
- The asymmetry with `set` is now total, not partial: `let` can *only*
  target lexical; `set` can *only* target a mutable engine (`AGENTS.md`'s
  rule that `set` on a lexical binding is an error already established
  `set`'s side of this; §4 below establishes `let`'s side). Neither verb
  can reach where the other lives. `set` therefore never has a pure case;
  `let` always does (given a pure `<expr>`).

### 4. `AGENTS.md`'s non-negotiable rule is amended

Per `AGENTS.md`'s own instruction that a non-negotiable syntax rule needs
an ADR to revisit, this ADR replaces its `let`/`set` rule. Old text:

> `set` mutates an existing binding; `let` introduces a new one (even if
> it shadows a previous name). `set` should only reach the mutable engines
> (graph / dynamic / reactive / synchronized); `set` on a plain lexical
> binding is an error.

New text (landed in the same PR that accepts this ADR):

> `let` introduces a lexical binding — the only engine it can target.
> `set` is the sole write path for the mutable engines (graph / dynamic /
> reactive / synchronized): it creates the binding if absent or mutates it
> if present (upsert), always effectful either way. `set` on a lexical
> binding is an error; `let` on a mutable engine is a syntax error — the
> two verbs partition the engines completely, with no overlap.

This also corrects two other stale spots the old introduce-vs-mutate
framing left behind:
- `LANGUAGE.md`'s Bindings section ("`lexical` values are *usually*
  constant, but can be declared mutable with `set`") directly contradicted
  even the *old* rule (`set` was already barred from lexical) — fixed to
  state lexical values are immutable once bound, full stop.
- The design note's §2/§5/§6 rows and `LANGUAGE.md`'s "Algebraic Data
  Types" example block used `let ^bool be …`-style type/graph-binding
  definitions — genuinely *new* graph bindings, written with `let`. Under
  this ADR those become `set ^bool to …` etc.: defining a type is
  introducing a graph binding, which `set` now owns.

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
- Does **not** decide the concrete parser/runtime mechanism for
  distinguishing "`set` creates" from "`set` updates" inside a given
  mutable engine (e.g. whether the engine needs to expose that distinction
  at all, or an upsert is genuinely uniform all the way down) — epic 009
  implementation detail. §1's upsert rule fixes the *effect classification*
  only, not the mechanics.
- Does **not** revisit `key%token`/`~token` ("static" engine, `AGENTS.md`:
  "cannot be rebound") — it is neither `let`'s lexical engine nor one of
  `set`'s mutable engines, and this ADR says nothing about how it is
  introduced.

## Consequences

- **Positive**
  - The keyword alone now determines both purity and engine, with no
    cross-checking needed: `let` is always lexical and always pure (given
    a pure `<expr>`); `set` is always a mutable engine and always
    effectful. The prior rule required knowing *which engine* a `let`
    targeted before its purity was decidable — that lookup is gone.
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
  - `set`'s upsert semantics mean there is no longer a way to assert "this
    binding must already exist" versus "create it if missing" at the
    keyword level — `set`'s single form covers both. If that distinction
    ever matters, epic 009 has to add it as a separate check, not recover
    it from which verb was used.

## Follow-ups

- Epic 008 (binding model taxonomy) classifies `set` as an effect in its
  vocabulary and documents the `Engine` column mapping this ADR does not
  cover.
- Epic 009 (binding kinds implementation) implements the mutable engines
  `set` targets against this rule.
- Update [`specs/execution-queue.md`](../execution-queue.md) epic 008 (and
  epic 009's `set` portion) start criteria to name **ADR-0007** (this ADR)
  in place of the interim "ADR 4.4" wording, once accepted.
