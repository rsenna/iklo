# Todo — `substrate` epic

Ordered. One at a time. Each task: RED (failing test or missing scaffold) →
GREEN (make it work) → regression (`make test`) → commit.

See [plan.md](plan.md) for the design rationale behind these tasks.

---

- [ ] **T1 — Scaffold `iklo-substrate` crate**
  - Create `crates/iklo-substrate/` with `Cargo.toml` (edition/version from workspace) and empty `src/lib.rs`.
  - Add to `[workspace] members` in root `Cargo.toml`.
  - **Acceptance:** `cargo build -p iklo-substrate` succeeds; `cargo test` still green.

- [ ] **T2 — Define `Substrate` + `Transaction` traits + `SubstrateError`**
  - `pub trait Substrate` with associated `type Value: Clone + Debug`, associated `type Tx<'a>: Transaction<Value = Self::Value>`, methods: `begin(&mut self) -> Self::Tx<'_>`, `revision(&self) -> u64`, `snapshot(&self) -> Vec<(String, Self::Value)>`.
  - `pub trait Transaction` with `type Value`, methods: `get(&self, name: &str) -> Option<Self::Value>`, `set(&mut self, name: &str, value: Self::Value)`, `commit(self)`, `rollback(self)`.
  - `pub struct SubstrateError { message: String }` + `Display`, `Error`.
  - Investigate: does `iklo-cli`'s `.env` command need `&HashMap<String, Value>` or just an iterator? Adjust `bindings()` plan if needed.
  - **Acceptance:** `cargo build -p iklo-substrate` succeeds; trait signatures compile; investigation result noted in commit message.

- [ ] **T3 — Implement `memory::InMemorySubstrate<V>` + contract tests**
  - `pub struct InMemorySubstrate<V> { bindings: HashMap<String, V>, revision: u64 }`.
  - Its `Tx<'a>` clones bindings; `commit` writes back + `revision += 1`; `rollback` drops.
  - Tests (in `iklo-substrate`):
    - `revision_starts_at_zero`
    - `commit_increments_revision`
    - `rollback_does_not_increment_revision`
    - `get_after_set_inside_tx_sees_value`
    - `get_after_rollback_does_not_see_value`
    - `get_after_commit_sees_value_from_fresh_tx`
    - `snapshot_returns_only_committed_state`
  - Tests use a concrete `V = i64` (or `String`) — proves the impl is generic.
  - **Acceptance:** `cargo test -p iklo-substrate` — all 7 pass.

- [ ] **T4 — Refactor `iklo-runtime` onto the substrate**
  - Add `iklo-substrate = { path = "../iklo-substrate" }` to `iklo-runtime/Cargo.toml`.
  - Replace `RuntimeImage`'s `HashMap` with `InMemorySubstrate<Value>`. Keep the public methods (`new`, `revision`, `eval_in_tx`, `bindings`) with the same signatures — delegate internally.
  - Replace the internal `Transaction` struct's `HashMap` with a substrate `Tx`. `eval_expr`'s `LexRef` and `Let` arms call `.get` / `.set` on the tx.
  - `eval_in_tx` opens a tx, runs the program, then `commit()` on success or `rollback()` on error.
  - Add `From<SubstrateError> for RuntimeError`.
  - Consider a `type IkloSubstrate = InMemorySubstrate<Value>` alias if signatures get noisy.
  - **Acceptance:** `cargo build -p iklo-runtime` succeeds; `cargo test -p iklo-runtime` passes with both existing tests **unchanged**.

- [ ] **T5 — Verify no regression across the workspace**
  - `make test` (full suite).
  - Manual smoke: `cargo run -p iklo-cli` then `let :x be 21 * 2` → `.env` should list `x = 42`; `.revision` should show `1`; `.quit`.
  - Run `examples/hello.iklo` via `cargo run -p iklo-cli -- examples/hello.iklo`.
  - **Acceptance:** all green; REPL behaves identically to before.

- [ ] **T6 — Docs update**
  - `AGENTS.md` — add `iklo-substrate` to "What is actually implemented today"; mention `RuntimeImage` is a façade over `InMemorySubstrate<Value>`.
  - `LANGUAGE.md` — the "Transactional VDBE and live image runtime" callout gets a small addendum: "As of the substrate epic, the runtime image lives behind a `Substrate` trait (in the `iklo-substrate` crate). The active implementation is in-memory; a Turso-backed implementation is deferred per ADR-0001."
  - `spec/substrate/SPEC.md` — mark success criteria checkboxes as complete.
  - **Acceptance:** docs match reality; a fresh reader learns nothing false.

- [ ] **T7 — Close the epic**
  - Commit doc updates from T6.
  - Then `CLEANING_TASKS` commit: reset `spec/tasks/plan.md` + `spec/tasks/todo.md` to placeholder state (or delete their bodies and leave a "no epic active" note).
  - **Acceptance:** `git log --grep=CLEANING_TASKS` shows the boundary; `spec/substrate/SPEC.md` is fully checked off.
