# Design Note: Strictness and Side-Effects Model

**Epic:** 006-strictness-effects-spike
**Status:** Active
**Date:** 2026-09-03
**Supersedes:** —

## 1. Canonical Vocabulary

| Term | Definition | Scope |
|------|-----------|-------|
| **strict** | Expression is fully evaluated before the enclosing form receives its value. No thunk wrapper. | Language + runtime |
| **lazy** | Expression is wrapped in a thunk `(code, env, state)` and evaluated on first force (call-by-need, memoized). | Language + runtime |
| **pure** | Form evaluates to a value with no observable side effects. Same inputs always produce same output. | Language surface |
| **effectful** | Form performs observable IO or state mutation during evaluation. | Language surface |
| **action** | A value describing a side effect to be performed later. Building an action is pure; running it is effectful. | Language surface |
| **thunk** | Runtime object `(code, env, state)` representing a deferred evaluation. States: `new`, `running`, `forced(value)`, `failed(error)`. | Runtime internal |
| **effect boundary** | A runtime execution context where actions may run: top-level runner, `run`, `do` block, `;` sequencing of action-typed expressions. | Runtime internal |
| **slot mode** | Per-argument strictness declaration in a form's interface: `strict` (default) or `lazy`. | Language surface |

## 2. Strictness/Purity/Effect Classification

### Currently implemented

| Construct | Strictness | Purity | Effect | Notes |
|-----------|-----------|--------|--------|-------|
| `let :x be <expr>` | strict | pure (binding) | none | Evaluates `<expr>`, binds result. Transactional commit on success. |
| `set :x to <expr>` | strict | effectful (mutation) | mutates existing binding | Requires mutable binding engine (`graph`, `dynamic`, `reactive`, `sync`). |
| `+ - * /` (arithmetic) | strict | pure | none | All operands evaluated before computation. |
| `:x` (lexical read) | strict | pure | none | Returns bound value. |
| Newline / `;` | strict | pure | none | Expression termination; no side effects. |

### Aspirational (LANGUAGE.md, not yet implemented)

| Construct | Strictness | Purity | Effect | Notes |
|-----------|-----------|--------|--------|-------|
| `fn` / `to` (function def) | — | pure (definition) | none | Closure captures lexical env. Body is lazy-evaluated on call. |
| `cond` | strict branches | pure (control flow) | none | Each branch is strict; only taken branch is evaluated. |
| `repeat` | strict body | pure (control flow) | none | Body evaluated N times. |
| `lazy <expr>` | lazy | pure | none | Wraps expression in thunk. |
| `strict <x>` | strict (force) | pure | none | Forces thunk; runtime error if cyclic. |
| `run <action>` | strict | effectful | executes action | The sole way to run an action outside top-level. |
| `do ... end` | strict, sequential | effectful block | executes actions in source order | Failing action short-circuits rest. |
| `then` | strict, sequential | effectful chain | passes value forward | `action1 then action2`. |
| `%deref <expr>` (`*expr`) | strict | pure | none | Forces a thunk or dereferences a ref. |
| Shell executable call | strict | effectful | process IO | `(vim start)` — unbound head resolves to an OS executable. Whether this runs immediately or yields an `^action` is **ADR 4.5**. |
| Graph transaction (`tx.begin/commit`) | strict | effectful | multi-binding mutation | Atomic cross-engine commit. |
| Macro expansion | — | pure (compile-time) | none | Operates on syntax objects. No runtime side effects. |
| Literal constructors | strict | pure | none | Per LANGUAGE.md contract. |

## 3. Language-Surface vs Runtime-Internal Boundary

### Language surface (what authors write and reason about)

- `lazy` / `strict` keywords — control evaluation timing.
- `run` — the only way to execute an action value.
- `do ... end` — synchronous sequential action execution.
- `then` — sequential action chaining with value forwarding.
- `^action ^t` — type annotation for action values.
- `strict` / `lazy` slot modes in form interfaces.
- `*expr` (deref) — force a thunk or dereference.
- Purity is **inferred by the type system**, not declared. A form is pure if it returns no `^action` type and performs no observable mutation.

### Runtime internal (invisible to language authors)

- Thunk object lifecycle (`new` -> `running` -> `forced`/`failed`).
- Effect boundary enforcement (which contexts allow action execution).
- Cycle detection during force (`cyclic-force` error).
- Memoization of forced thunk results.
- Transaction boundary management (`tx.begin/commit/rollback`).
- Binding engine selection (graph vs lexical vs dynamic vs reactive vs sync).

### Where the line matters

| Concern | Surface or Internal | Rationale |
|---------|-------------------|-----------|
| "Is this form pure?" | Surface | Authors must know for reasoning about determinism and testability. |
| "How is the thunk memoized?" | Internal | Implementation detail; different backends may differ. |
| "Can I run an action here?" | Surface (boundary) | Authors must know which contexts are effect boundaries. |
| "How does the transaction commit?" | Internal | Substrate handles it; language sees only atomic commit/rollback. |
| "Is this slot strict or lazy?" | Surface | Authors declare it; the runtime honors it. |
| "What states can a thunk be in?" | Internal | Authors see only `forced` or `cyclic-force` error. |

## 4. ADR-Needed Decisions

These decisions are load-bearing enough to require their own ADR before implementation:

### 4.1 Effect type system shape

**Question:** Are effects tracked via a dedicated type (`^action ^t`), a row/column polymorphism on an effect row, or a simpler tag system?

**Why ADR:** This determines whether effect tracking composes cleanly with future type system features (generics, traits). Haskell's approach (full row polymorphism) is powerful but cognitively expensive. A simpler tag system (`^action` as a marker type) may be sufficient for Iklo's scope.

**Blocks:** Epic 007 (IK1 core language) needs at least a minimal effect annotation to distinguish IO forms from pure forms.

### 4.2 Lazy-by-default vs strict-by-default for form arguments

**Question:** Should form arguments default to strict (current LANGUAGE.md design) or lazy (opt-in strictness)?

**Why ADR:** Default determines the cognitive cost. Strict-by-default means authors must opt in to laziness (`lazy :x` in interface). Lazy-by-default means authors must opt in to strictness. Iklo's stated preference is strict-by-default, but this interacts with the upcoming closure design (`fn`/`to`) — if function bodies are lazy, should arguments also be?

**Blocks:** Epic 007 (function definition syntax), Epic 009 (binding kinds implementation).

### 4.3 Effect boundary strictness for `do` blocks

**Question:** Must `do` blocks execute actions synchronously in source order, or may a runtime implementation reorder independent actions?

**Why ADR:** LANGUAGE.md's Design constraints require "Deterministic effect order for all synchronous actions" and that `do` "executes actions synchronously in source order." But future parallelism features (futures, async) may want to relax this. The boundary between "synchronous do" and "parallel do" needs a clear design point.

**Blocks:** Epic 007 (`do`/`cond`/`repeat` implementation).

### 4.4 `set` and effect classification

**Question:** Is `set` an effect (mutation visible to the runtime) or a pure operation (rebinding a name)? Currently LANGUAGE.md says `set` mutates existing bindings — but is that an "effect" in the type system sense?

**Why ADR:** If `set` is an effect, then any form that calls `set` is effectful and cannot be pure. If `set` is not an effect (just a binding operation), then forms calling `set` could still be considered pure. This interacts with transaction semantics — `set` inside a transaction that rolls back was never "observable."

**Blocks:** Epic 008 (binding model taxonomy), Epic 009 (binding kinds).

### 4.5 Shell-mode executable calls and the effect boundary

**Question:** When an unbound form head resolves to an OS executable in shell mode (`(vim start)`), does it (a) execute immediately, (b) produce an `^action ^t` that runs only at an effect boundary like every other IO form, or (c) execute immediately *only* at the top-level runner (itself an effect boundary) while yielding an `^action` everywhere else?

**Why ADR:** Option (a) directly violates LANGUAGE.md's "No implicit side-effect execution during ordinary expression evaluation" constraint and makes shell calls the one IO path that bypasses `run`/`do`. Option (b) is consistent with every other effectful form but costs shell ergonomics (every command needs `run`). Option (c) preserves both but needs a precise definition of "top-level" once nested REPL contexts, macro expansion, and `do` blocks are in play.

**Blocks:** Epic 007 (shell-mode form resolution and stdlib IO).

## 5. Recommended Effect Control Strategy

Based on LANGUAGE.md's stated goals and the analysis above:

### Surface syntax: minimal, explicit

Effect control belongs in **surface syntax** for the most common cases:

- `lazy` / `strict` keywords for per-expression timing.
- `run` for action execution.
- `do ... end` for sequential effect blocks.
- `^action ^t` type annotation for form return types.

This is the "practical Haskell without monads" goal stated in LANGUAGE.md. The surface syntax makes effects visible without requiring monadic wrapping.

### Standard library: IO forms return actions

Standard library forms that perform IO (`echo`, `cp`, file ops, network) return `^action ^t` values. They are pure to build, effectful to run. This means:

```iklo
let :copy be cp "a" "b"    # pure: builds an action value
run :copy                    # effectful: executes the copy
```

Shell executable calls are an unresolved tension: shell-mode ergonomics want `(vim start)` to run immediately, but LANGUAGE.md's "No implicit side-effect execution during ordinary expression evaluation" constraint says a bare executable call inside a pure expression must not. This is flagged as **ADR 4.5**. Until it is resolved, treat immediate shell execution as a top-level-runner affordance only (that runner is itself an effect boundary) — not a license to perform process IO inside pure expressions.

### Runtime metadata: not the primary mechanism

Runtime metadata (annotations) should **not** be the primary way to declare purity/effects. Annotations are for introspection and tooling, not for core language semantics. A form's purity should be inferrable from its type signature, not from a metadata tag.

### Combination: type inference + surface keywords

The recommended approach:

1. **Surface keywords** (`lazy`, `strict`, `run`, `do`) control the most common cases.
2. **Type inference** determines purity from *both* the inferred body (no observable mutation, e.g. `set` or a graph `let`) *and* the return type (not `^action`). Either one alone is insufficient — a form that mutates a binding but returns a plain value is still effectful.
3. **Runtime enforcement** ensures effects only run in effect boundaries.
4. **Annotations** provide optional hints for tooling and diagnostics.

This avoids the "annotation pollution" problem while keeping effects visible and enforceable.

## 6. Concrete Examples (Classification Verification)

| Example | Strictness | Purity | Effect | Correct? |
|---------|-----------|--------|--------|----------|
| `let :x be 1 + 2` | strict | pure | none | Yes — arithmetic is pure and strict. |
| `set :x to 5` | strict | effectful (mutation) | binding mutation | Yes — rebinds an existing `:x` in a mutable engine; effectful despite returning no `^action` (see §5 point 2). |
| `let :x be lazy 1 + 2` | lazy | pure | none | Yes — thunk created, not evaluated. |
| `strict :x` | strict (force) | pure | none | Yes — forces thunk, may error on cycle. |
| `let :copy be cp "a" "b"` | strict | pure (builds action) | none (action not run) | Yes — action value created, not executed. |
| `run :copy` | strict | effectful | file IO | Yes — action boundary, effect executes. |
| `do echo "hi"; cp "a" "b" end` | strict, sequential | effectful | terminal + file IO | Yes — two effects in sequence. |
| `echo "hi" then cp "a" "b"` | strict, sequential | effectful | chained IO | Yes — value forwarded between effects. |
| `cond (-true) 1 else 2` | strict (branch) | pure | none | Yes — only taken branch evaluated. |
| `repeat 4 [forward :side]` | strict (body) | pure | none | Yes — body evaluated 4 times, pure. |
| `to adder :n do fn do :n + 1 end end` | — | pure (definition) | none | Yes — defines closure, no side effects. |
| Graph `let ^bool :x be -true` | strict | effectful (mutation) | graph commit | Yes — modifies graph binding engine. |
| `(vim start)` (shell) | strict | effectful | process IO | Effectful is settled; *when* it runs (immediately vs `^action` at a boundary) is **ADR 4.5**. |
| `` `[ a ~:b _:c d ] `` (syntax-quote) | — | pure (compile-time) | none | Yes — macro template, no runtime effect. |
| `map %{ a->1, b->2 }` | strict | pure | none | Yes — literal constructor, pure per contract. |

Every example above is classified consistently with no contradictions.

## 7. Enabled Follow-Up Epics

### Epic 007 — IK1 Core Language

**Depends on:** ADR 4.1 (effect type shape), ADR 4.2 (strict/lazy defaults), ADR 4.5 (shell-mode executable calls).

IK1 needs `fn`/`to` closures, `cond`, `repeat`, and stdlib IO. The effect model determines whether IO forms return `^action ^t` values (recommended) or execute immediately. The strict/lazy default determines how function arguments are evaluated. ADR 4.5 settles whether shell-mode executable calls join the `^action` discipline or stay an immediate-execution carve-out.

### Epic 008 — Binding Model Taxonomy

**Depends on:** ADR 4.4 (`set` and effect classification).

Binding taxonomy must classify `set` as either an effect or a binding operation. This affects whether `set`-using forms are considered pure. The taxonomy also needs to align with this design note's vocabulary (strict, lazy, pure, effectful).

### Epic 009 — Binding Kinds Implementation

**Depends on:** ADRs 4.2 and 4.4.

Implementation of `reactive`, `synchronized`, and other binding kinds must honor the strictness/effect model. Reactive bindings (`rx%token`) are inherently effectful (event-sourced). Synchronized bindings (`sync%token`) require transactional enforcement.
