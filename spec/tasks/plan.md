# Plan — `substrate` epic

**Epic spec:** [../substrate/SPEC.md](../substrate/SPEC.md)
**ADR:** [../decisions/ADR-0001-substrate-boundary.md](../decisions/ADR-0001-substrate-boundary.md)
**Active TDD list:** [todo.md](todo.md)

## Shape of the change

Today, `iklo-runtime` owns everything: the `Value` type, the `RuntimeImage`
(a `HashMap<String, Value>` + `revision: u64`), the `Transaction` struct
that clones-then-commits, and the tree-walker. All ~150 lines in one file.

After this epic:

```
iklo-runtime
  ├── Value          (unchanged; still owned here)
  ├── RuntimeError   (unchanged; wraps SubstrateError)
  ├── RuntimeImage   (thin façade over InMemorySubstrate<Value>)
  └── Transaction    (tree-walker; talks to substrate::Transaction, not a HashMap)

iklo-substrate           (new crate, zero deps on the rest of the workspace)
  ├── Substrate        <trait; generic over associated Value>
  │     open / close
  │     begin -> Transaction
  │     revision
  │     snapshot -> Vec<(String, Value)>     (for CLI .env)
  ├── Transaction     <associated trait on Substrate>
  │     get(name) -> Option<Value>
  │     set(name, value)
  │     commit
  │     rollback
  ├── SubstrateError
  └── memory::InMemorySubstrate<V>   (the current HashMap logic, generalized)
```

Public API of `iklo-runtime` (what `iklo-cli` calls) stays wire-compatible:
`RuntimeImage::new()`, `.revision()`, `.eval_in_tx(&Program)`, `.bindings()`.
Internally these delegate to the substrate.

## Key design decisions (defaulted here; see epic spec "Open questions")

1. **`Substrate` is generic over an associated `Value` type**, bounded on
   `Clone + Debug`. `iklo-substrate` never sees Iklo `Value`; `iklo-runtime`
   plugs its own `Value` in. This keeps the honest promise: no back-edge
   from `iklo-substrate` to `iklo-runtime`.
2. **In-memory impl lives inside `iklo-substrate`** as a `memory` module.
   Extracting to `iklo-substrate-memory` is speculative until we actually
   have a second impl to justify it.
3. **Transaction ownership is a runtime check, not a type-level guarantee.**
   `Transaction::commit(self)` already consumes by value in today's code,
   which is most of the protection anyway. Type-level "you can't commit a
   tx you didn't open" is a future refinement (would need lifetimes tying
   `Tx` to `&mut Substrate` — worth exploring later, not now).
4. **`bindings()` becomes `snapshot()` on the substrate**, returning
   `Vec<(String, Value)>`. `RuntimeImage::bindings()` keeps its current
   `&HashMap` signature only if that's cheap; otherwise we adjust the CLI
   `.env` command in the same epic (small edit, one call site).

   → Verify during T2: check whether CLI's `.env` truly needs `&HashMap` or
   just an iterator. If iterator, change `bindings()` signature; if
   HashMap-shaped, materialize.

## Sequencing (TDD, one task at a time)

The order below matches [todo.md](todo.md). Each task is atomic: leaves
the tree green, gets its own commit.

- **T1** — Scaffold `iklo-substrate` crate (empty lib, workspace member,
  compiles). Baseline.
- **T2** — Define the `Substrate` + `Transaction` traits, `SubstrateError`,
  and unit-test skeletons. No implementation yet — traits + `todo!()` stubs
  used only by the tests. `cargo build -p iklo-substrate` green.
- **T3** — Implement `memory::InMemorySubstrate<V>`. Move the HashMap +
  revision logic from `iklo-runtime`. Contract tests pass:
  - revision starts at 0, increments on commit, does not on rollback;
  - `get` after `set` inside a tx sees the value;
  - `get` after rollback does not see the value;
  - `get` after commit sees the value from a fresh tx;
  - `snapshot` returns committed state only.
- **T4** — Refactor `iklo-runtime`:
  - `RuntimeImage` becomes a wrapper around `InMemorySubstrate<Value>`;
  - `Transaction` (in `iklo-runtime`) holds a substrate `Tx` instead of a
    `HashMap`; `eval_expr` calls `.get` / `.set` on the tx;
  - `eval_in_tx` becomes `substrate.begin() → eval → tx.commit()` or
    `tx.rollback()` on error;
  - `RuntimeError` gains a `SubstrateError` variant (or wraps it via `From`).
- **T5** — Verify the two existing runtime tests
  (`let_returns_bound_value`, `rollback_keeps_image_unchanged`) still pass
  **unchanged**. If either had to change, we broke the boundary — revisit.
- **T6** — Verify CLI `.env` still shows bindings correctly (manual smoke
  or add a small integration test if easy).
- **T7** — Docs: update AGENTS.md ("what's implemented today" adds
  `iklo-substrate`) and LANGUAGE.md's VDBE section (mention the in-memory
  substrate is live; Turso still deferred).
- **T8** — Final `make test && make build && make release`, then close the
  epic by committing `CLEANING_TASKS` reset of `spec/tasks/`.

## What we are explicitly NOT doing

- No Turso dependency, no VDBE code, no persistence, no query layer.
- No new sigils, forms, or engines. `graph` / `dynamic` / `reactive` /
  `sync` engines get *trait surface only* — no populated implementations.
  If T3's contract feels like it demands them, we push back: they're future.
- No performance work. If the trait indirection slows the tree-walker,
  that's fine.
- No `async`. Tree-walker is sync; trait is sync.

## Definition of done

Every success criterion in [../substrate/SPEC.md](../substrate/SPEC.md) is
checked. Both `iklo-runtime` tests still pass unchanged. New contract tests
in `iklo-substrate` pass. `make test`, `make build`, `make release` all
green. LANGUAGE.md and AGENTS.md reflect the new state. Zero Turso mentions
as active dependencies.

## Risks / where this could bite

- **`Value` becoming a trait parameter everywhere.** If the generic
  parameter viruses through `iklo-runtime`, we'll want a type alias
  (`type IkloSubstrate = InMemorySubstrate<Value>`) at the top of the crate
  to keep signatures readable. Expect to add this in T4.
- **`bindings()` signature change breaking the CLI.** T2's investigation
  answers this before we commit to a shape.
- **Trait-object vs. generic-parameter fork.** We're going generic. If T4
  hits a call site that wants `&dyn Substrate<Value = Value>`, we accept
  the wordier signature there rather than boxing everywhere.
