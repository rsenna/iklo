# ADR-0006 — Effect model: `^action`-typed effects, strict-by-default evaluation, ordered `do`

- **Status:** Proposed — drafted from the epic 006 design note's open
  questions §4.1–4.3. Accepting this ADR means editing this line to
  `Accepted` (with the acceptance date), as part of — not automatically
  implied by — merging this PR; a merge that leaves this line reading
  `Proposed` has not accepted it.
- **Date:** 2026-09-11
- **Deciders:** @rsenna (with Claude as sounding board)
- **Supersedes:** —
- **Superseded by:** —

## Decision (one sentence)

**Iklo represents effects as a marker type `^action ^t` (no effect rows, no
monad threading), evaluates form arguments strictly by default with `lazy` as
an opt-in per-slot mode, and guarantees `do` executes its actions
synchronously in source order with no reordering.**

## Context

The epic 006 spike
([`specs/006-strictness-effects-spike/design-note.md`](../006-strictness-effects-spike/design-note.md))
produced a shared strict/lazy/pure/effectful vocabulary and classified the
current and aspirational language surface. It deliberately stopped short of
*deciding* five load-bearing questions (its §4). Three of them — §4.1 (effect
type shape), §4.2 (strict vs lazy default), §4.3 (`do`-block ordering) — are
tightly coupled: together they fix how effects are typed, when expressions
are evaluated, and in what order effects run. Epic 007 (IK1 core language)
cannot begin implementation without all three settled. This ADR closes them
as one coherent effect model.

`LANGUAGE.md` already states the intended shape in prose
(§"Laziness and effects (practical model)", §"Interface-level strictness",
§"Design constraints"). Per the constitution's principle VI, a decision that
"determines the shape of code across multiple crates" or "gets questioned a
second time" becomes an ADR — the design note's review raised the effect
model repeatedly, and this touches the lexer, parser, runtime, and the
future type checker. It is also a grammar-adjacent decision (`lazy`,
`strict`, `run`, `do`, `then`, `^action`), which the constitution requires an
ADR for.

The remaining two §4 questions — §4.4 (`set` as an effect) and §4.5
(shell-mode executable calls) — are narrower, gate different epics, and get
their own ADRs (see Follow-ups). They are **not** decided here.

## What this commits us to

### 1. Effects are a marker type: `^action ^t` (design note §4.1)

- A form's return type is **`^action ^t`** if and only if *running* the
  value it returns performs observable IO or state mutation. Otherwise the
  return type is a plain `^t`.
- `^action` is a **type constructor / marker**, not an effect row and not a
  polymorphic effect variable. There is no `IO e` to thread, no effect
  polymorphism, no `runST`-style scoping. This is the "practical Haskell
  without monads" goal from `LANGUAGE.md`.
- **Building** an `^action ^t` is pure; **running** it is effectful:

  ```iklo
  let :copy be cp "a" "b"    # pure — builds an ^action ^int
  run :copy                  # effectful — performs the copy
  ```

- **Purity is inferred, never declared, and is about evaluation, not the
  produced value's type.** A form is pure iff *evaluating it* performs no
  observable mutation (no `set`, no `let` into a mutable binding engine —
  pending ADR-0007's exact line on `set`) and does not itself cross an
  effect boundary (no inline `run`/`do`/boundary `;`/`then` fires during its
  own evaluation). Whether the *value* a pure form returns happens to be
  `^action ^t`-typed is irrelevant — that only means running it *later* is
  effectful, not that constructing it now was. `let :copy be cp "a" "b"`
  above is pure by this rule: it builds an `^action ^int` but never runs
  one. A form that mutates a binding is impure regardless of what it
  returns — that is the failure mode a return-type-only rule would miss
  (design note §5 point 2).
- **Annotations are not the mechanism.** `#!`-annotations may carry purity
  *hints* for tooling and diagnostics, but the type system — not a metadata
  tag — is the source of truth for whether a form is pure. A change to an
  annotation never changes evaluation semantics.
- Whether `^t` must be written explicitly or is inferred, and the full
  structural type of an action, are **out of scope** here — see epic 010
  and Follow-ups.

### 2. Strict-by-default evaluation; `lazy` is opt-in per slot (design note §4.2)

- Form arguments are **evaluated strictly before the call**, unless the
  form's interface marks that slot `lazy`:

  ```
  strict :x   # default — argument evaluated before the call
  lazy :x     # opt-in — argument passed as a thunk, forced on first use
  ```

- Laziness is therefore always a **local, visible** choice at the call
  target's interface, never a global evaluation strategy. `LANGUAGE.md`
  §"Interface-level strictness" already states "Default slot mode is
  `strict`"; this ADR ratifies it against the alternative (lazy-by-default
  with opt-in strictness) and closes the question.
- Closure bodies (`fn` / `to`) are not evaluated until the closure is
  **called**; once called, the body evaluates eagerly (subject to the same
  per-slot rule for its own arguments). "Body is lazy-evaluated on call" in
  the design note's §2 table means *deferred until call*, not call-by-need.
- `lazy` / `strict` remain **pure** in themselves: `lazy <expr>` wrapping and
  `strict <x>` forcing perform no effects. A thunk body that would mutate or
  perform IO is outside the pure classification — the precise handling
  (reject at type-check, carry effect metadata on the thunk, or require
  forcing at an effect boundary) is deferred to the type-system work under
  Follow-ups; laziness is a control feature for pure compute, not an effect
  scheduler (`LANGUAGE.md` §"Design constraints"). Forcing a thunk never
  executes hidden effects: if the forced value is an `^action ^t`, running
  it still requires `run` / `do`.

### 3. `do` executes in source order, with no reordering (design note §4.3)

- `do … end` executes its actions **synchronously, one at a time, in source
  order**. A failing action short-circuits the rest of the block.
- The runtime **may not reorder** actions in a `do` block, even ones a
  compiler could prove independent. `;`-sequencing of `^action`-typed
  expressions and `then`-chaining carry the same guarantee.
- This upholds `LANGUAGE.md` §"Design constraints": "No implicit
  side-effect execution during ordinary expression evaluation" and
  "Deterministic effect order for all synchronous actions."
- Any future concurrency is introduced as a **separate, explicitly named
  construct** (e.g. a `par` block or async actions), never as a silent
  relaxation of `do`'s ordering. The boundary between "synchronous `do`" and
  any "parallel" form is a new ADR at the time such a feature is designed.

### 4. Effect boundaries (unchanged, restated for completeness)

Effects run **only** at these boundaries, per `LANGUAGE.md` §"Effect
semantics":

- the top-level program runner,
- `run <action>`,
- a `do … end` block,
- `then` chaining,
- `;` sequencing when the sequenced expressions are `^action`-typed.

`run` is the primitive executor; `do` and `then` are effect boundaries in
their own right that build on it. Nothing else — ordinary expression
evaluation, thunk forcing, macro expansion, `let`/`set` into a lexical
binding — is an effect boundary.

## Non-decisions

- Does **not** decide whether `set` is "an effect" in the type-system sense
  — that is **ADR-0007** (design note §4.4). Until then, "no observable
  mutation" in the purity rule above is read conservatively (a `set` makes
  the enclosing form non-pure).
- Does **not** decide **shell-mode executable call** timing — that is
  **ADR-0008** (design note §4.5).
- Does **not** define the **concrete IK1 grammar** for `fn` / `cond` /
  `repeat` — that is a separate IK1 grammar ADR, a start-criterion for
  epic 007 alongside this one.
- Does **not** finalize the **primitive type inventory** or the structural
  type of `^action ^t` — that is epic 010.
- Does **not** authorize **VDBE** compilation work (ADR-0001 sequencing
  stands; `iklo-runtime` remains the semantic reference).
- Does **not** add effect tracking to the **type checker now** — there is no
  type checker yet. This ADR fixes the *model* so epic 007's implementation
  and any later checker build to the same target.

## Consequences

- **Positive**
  - Epic 007 can start: `^action`, `run`, `do`, `then`, and `strict`/`lazy`
    slot modes have a fixed target.
  - Purity is a checkable property (no mutation, no boundary-crossing during
    evaluation), not a matter of convention or annotation discipline.
  - `do` semantics are unambiguous and debuggable — the order you read is
    the order effects happen.
  - No monad ceremony in surface syntax; effects stay visible via
    `^action ^t` types and the `run`/`do` keywords.
- **Negative**
  - Strict-by-default means genuinely recursive/knot-tying data and
    self-referential definitions need an explicit `lazy` slot; there is no
    "it just works" call-by-need.
  - No automatic parallelism in `do`, even where it would be safe —
    concurrency is always an explicit, later opt-in.
  - Inferred purity is more than a one-line rule: the eventual type checker
    must track observable mutation *and* boundary-crossing through a form's
    body, not just its declared return type.
  - Committing to a marker type now (rather than an effect row) may cost a
    migration if Iklo later wants fine-grained effect categories
    (IO vs. state vs. exceptions). Judged acceptable: the marker is a strict
    subset of any richer scheme and `LANGUAGE.md`'s scope does not call for
    more.

## Follow-ups

- **ADR-0007** — `set` and effect classification (design note §4.4).
  Blocks epic 008.
- **ADR-0008** — shell-mode executable calls and the effect boundary
  (design note §4.5). Blocks epic 007.
- **IK1 grammar ADR** — concrete syntax for `fn` / `cond` / `repeat`.
  Blocks epic 007.
- Update [`specs/execution-queue.md`](../execution-queue.md) epic 007 start
  criteria to name **ADR-0006** (this ADR) in place of the interim
  "ADRs 4.1/4.2/4.3" wording, once accepted.
- Epic 007 implements against this model: the `^action` marker type,
  `strict` / `lazy` slot modes, `run` / `do` / `then`, and source-order
  effect execution.
- The type-system work (epic 010 and beyond) resolves the deferred detail on
  effectful thunk bodies (reject vs. metadata vs. boundary-forcing).
