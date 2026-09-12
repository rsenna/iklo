# ADR-0008 — Shell-mode executable calls join the `^action` model; no execution carve-out

- **Status:** Proposed — drafted from the epic 006 design note's open
  question §4.5. Accepting this ADR means editing this line to `Accepted`
  (with the acceptance date), as part of — not automatically implied by —
  merging this PR; a merge that leaves this line reading `Proposed` has not
  accepted it.
- **Date:** 2026-09-12
- **Deciders:** @rsenna (with Claude as sounding board)
- **Supersedes:** —
- **Superseded by:** —

## Decision (one sentence)

**Shell-mode executable calls (an unbound form head resolving to an OS
executable, e.g. `(vim start)`) build an `^action ^t` exactly like every
other IO-form call — no special immediate-execution primitive — and that
action executes at the same effect boundaries ADR-0006 already defined
(top-level runner, `run`, `do`, `then`, boundary `;`); nowhere else.**

## Context

The epic 006 design note
([`specs/006-strictness-effects-spike/design-note.md`](../006-strictness-effects-spike/design-note.md))
§4.5 asked how `(vim start)` should behave: (a) always execute immediately,
(b) always build an `^action ^t` requiring explicit `run`/`do` like every
other IO form, or (c) execute immediately only at the top-level runner
while yielding an `^action` everywhere else. Option (a) was rejected on
sight — it directly violates `LANGUAGE.md`'s "no implicit side-effect
execution during ordinary expression evaluation" constraint and would make
shell calls the one IO path that bypasses `run`/`do`. The design note
flagged (c) as needing "a precise definition of top-level" before it could
be adopted, and left the choice between (b) and (c) open. ADR-0006's own
Follow-ups list this ADR as blocking epic 007 (shell-mode form resolution
and stdlib IO).

The apparent tension dissolves once ADR-0006's own effect-boundary list is
taken literally: **the top-level runner and `do` are already effect
boundaries** where action-typed expressions execute (that is what "boundary"
means — `do echo "hi"; cp "a" "b" end`, per the design note's own §6
example, runs both actions in sequence *without* an explicit `run` on
either). So "immediate at the top level" (option c) is not a special case
bolted onto option (b) — it is simply what option (b) *already* implies at
a boundary ADR-0006 had already named. This ADR gives shell calls no
carve-out at all: they are an ordinary `^action`-producing form, and they
run wherever any such form would run, for the same reason.

## What this commits us to

### 1. Shell-mode executable calls build `^action ^t` — no exception

- When an unbound form head resolves to an OS executable (`(vim start)`,
  `vim` not being a known form), resolution produces a value of type
  `^action ^t`: constructing it is pure, running it performs the process
  IO. This is the exact same shape as `cp "a" "b"` or `echo "hi"` — shell
  calls are not exempt from ADR-0006's IO model, and there is no separate
  "immediate exec" primitive for them.
- This also matches `LANGUAGE.md`'s own stated roadmap (Phase 2: "Port
  shell/file/network built-ins to return `^action ^t`") — this ADR is not
  introducing new scope, it is confirming shell calls are inside that
  scope, not a permanent exception to it.

### 2. The action runs at the boundaries ADR-0006 already named — nowhere new

- A shell-exec action executes precisely where ADR-0006 §4 says any action
  executes: the top-level program runner, `run <action>`, a `do … end`
  block, `then` chaining, and `;` sequencing of `^action`-typed
  expressions. No new boundary is introduced for shell mode.
- Concretely, this is why `(vim start)` typed bare at the REPL/shell prompt
  "just runs": the prompt *is* the top-level runner, and a bare top-level
  expression whose value is `^action ^t` executes there — the same reason
  a bare `cp "a" "b"` typed at the prompt would also run without an
  explicit `run`. Inside `do … end`, `(vim start)` runs in source order
  alongside any other action in that block, for the same reason `echo`/`cp`
  do (design note §6: `do echo "hi"; cp "a" "b" end` runs both).
- **"Top-level" is precisely:** the single expression submitted as one unit
  to the top-level runner — one REPL submission up to its terminator, or
  one top-level form when running a file. A `do` block's body, a function
  body, and an argument position are never "top-level," even when they sit
  syntactically first in their own sub-block — each has its own boundary
  status (`do` is a boundary in its own right per rule above; a bare
  function-call argument is not).

### 3. Capturing the action (via `let`/`set`) suppresses auto-run, even at the top level

- `let`/`set` binding the result of a shell call does **not** run it, even
  when the whole `let`/`set` form is itself the top-level expression. This
  is not a new rule invented for shell calls — it is ADR-0006's own worked
  example applied literally: `let :copy be cp "a" "b"` is "pure — builds an
  action value, not executed," full stop, regardless of where that `let`
  sits. `let :copy be (vim start)` follows identically: `:copy` is bound to
  the unrun action; running it still requires `run :copy` (or placing the
  bare call, uncaptured, at a boundary).
- The distinguishing signal is not position but **whether the value is
  captured**: a bare expression statement's action-typed result runs at a
  boundary; a `let`/`set`-bound one does not, because the binding is a
  signal that the author wants the value, not its execution, right now.

### 4. Outside every boundary, a shell-exec action is just a value

- As a function-call argument, inside a `cond`/`repeat` branch value that
  is not itself run, or anywhere else that is not one of the listed
  boundaries, `(vim start)` evaluates to an unrun `^action ^t` — buildable,
  storable, and passable like any other action, per ADR-0006's general
  model. Nothing about shell-mode resolution changes this.

## Non-decisions

- Does **not** specify the concrete parser/runtime mechanism for resolving
  an unbound form head to an OS executable (PATH lookup, argument passing,
  exit-code/stdout representation as part of the action's `^t` payload) —
  that is epic 007 implementation detail.
- Does **not** decide whether resolution failure (no such executable) is a
  parse-time, resolution-time, or run-time error — epic 007.
- Does **not** revisit ADR-0006's effect-boundary list or its `let`
  example; §2 and §3 above apply them, not amend them.
- Does **not** add enforcement to a type checker — none exists yet (per
  ADR-0006's Non-decisions).

## Consequences

- **Positive**
  - Shell/REPL ergonomics are preserved — `(vim start)` at the prompt "just
    works" — without carving any exception into the "no implicit effects
    during ordinary evaluation" constraint: it runs *because* the prompt is
    already a named boundary, not because shell calls are special.
  - The effect model stays uniform: one rule (`^action`, run at a boundary)
    covers every IO form, shell calls included. Epic 007 implements shell
    resolution against the same machinery as `echo`/`cp`, not a parallel
    path.
  - Resolves the design note's "precise definition of top-level" ask
    directly (§2 above), rather than leaving it open for epic 007 to
    improvise.
- **Negative**
  - `let :copy be (vim start)` silently *not* running the command (it only
    builds the action) can surprise an author used to shell intuition,
    where typing a command name in an assignment context might be expected
    to run it. Judged acceptable: it is the same rule `cp`/`echo` already
    follow, so the surprise, if any, is about the general `^action` model,
    not something shell-specific.
  - Composing shell calls with other actions (piping, redirection) is not
    addressed here — deferred to epic 007's stdlib design.

## Follow-ups

- Epic 007 (IK1 core language) implements shell-mode form resolution
  against this model: build an `^action ^t`, run only at the named
  boundaries.
- `specs/execution-queue.md`'s epic 007 start criteria already link this
  ADR directly (this PR). The only thing still pending acceptance is this
  document's own Status line (see header).
