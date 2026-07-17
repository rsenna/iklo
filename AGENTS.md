# AGENTS.md — Iklo agent operating guide

This file tells any AI agent (or new human contributor) how to work on Iklo.
It is intentionally short and factual. For everything else:

- **What Iklo *is* as a language** → [LANGUAGE.md](LANGUAGE.md) (the reference; much of it is aspirational, marked **TBI**/**TBD**/**BET**).
- **How this repo is built, tested, and evolved** → [SPEC.md](SPEC.md).
- **Why we made the load-bearing decisions** → [`design/decisions/`](design/decisions/).
- **What we're actively building next** → [`spec/`](spec/) (per-epic specs) and [`tasks/`](tasks/) (active plan/todo).

## What Iklo is, in one paragraph

Iklo is a Rust-implemented programming language with three faces: a
functional/data-oriented language ("a Lisp with fewer parentheses"), a Unix
shell that shares one grammar with the language, and an in-process,
transactional live-image runtime backed (eventually) by [Turso](https://turso.tech/)
and its VDBE bytecode VM. The name comes from *ikke Logo* — Danish for "not
Logo" — reflecting the influence of UCBLogo without the ambition of being a
Logo dialect.

## What is actually implemented today

Anything not on this list is aspirational. Do not assume LANGUAGE.md examples run.

- **Lexer** (`crates/iklo-lexer`) — logos-based; produces `Lexeme` values (kebab-case identifiers, numbers, `:name` lexical refs, `+ - * /` operators, parens, `let`, `be`, `set`, newline, `;`).
- **AST** (`crates/iklo-ast`) — `Program = Vec<Spanned<Expr>>`; expressions include `Number`, `LexRef`, `Let`, `Binary`.
- **Parser** (`crates/iklo-parser`) — Pratt precedence; whitespace-sensitive infix ops (so `x-1` stays one identifier); newline is a soft terminator (terminates only when the current expression is complete and can't be continued); `;` is a hard terminator; newlines are swallowed inside parens. Supports `let :name be <expr>` as an expression.
- **Runtime** (`crates/iklo-runtime`) — tree-walking interpreter with a transactional live image: `Env` supports `begin`/`commit`/`rollback` and a revision counter. `let` and `set` update the image transactionally per top-level expression.
- **CLI** (`crates/iklo-cli`) — file runner and multi-line REPL. Continuation prompt is `iklo. `; blank line cancels a multi-line input. REPL commands are `.`-prefixed (`.quit`, `.revision`, `.env`) and only recognized at a fresh prompt.

## Non-negotiable syntax rules

These are decided and shouldn't be casually revisited. If a change is needed, open an ADR.

- **Identifiers are kebab-case**, including subtraction-lookalikes: `x-1` is one identifier, `x - 1` is subtraction. Infix `+ - * /` **require whitespace on both sides**.
- **Binding introduction is `let :name be <expr>`** (not `=`). `:name` is the lexical-value sigil. `let` is an expression that returns the bound value.
- **`set` mutates an existing binding**; `let` introduces a new one (even if it shadows a previous name). `set` should only reach the mutable engines (graph / dynamic / reactive / synchronized); `set` on a plain lexical binding is an error.
- **Newline is a soft terminator**: it ends the current expression only when that expression is already complete *and* the next line can't continue it. Newlines are ignored inside `( … )`.
- **`;` is a hard terminator** and forces the current expression to end (parse error if incomplete).
- **REPL commands use a leading `.`** (`.quit`, `.revision`, `.env`) to avoid colliding with `:name` and to keep `/paths` free for shell mode.
- **`Lexeme`, not `Token`** — in Iklo code, `token` is a *value type* (the symbolic unit used for bindings). The lexer's output is called `Lexeme` to keep the two apart.

## Development commands

```bash
make build           # cargo build (dev)
make test            # cargo test (dev)
make release         # cargo build --release
make clean           # cargo clean
cargo run -p iklo-cli                    # start the REPL
cargo run -p iklo-cli -- examples/hello.iklo   # run a file
cargo test -p iklo-parser                # target one crate
```

`mise.toml` pins the Rust and Java toolchains (`mise install` to hydrate).
There are no plugins yet; the Makefile is deliberately thin.

## Coding conventions

- Rust, idiomatic per crate. Follow `.github/instructions/rust.instructions.md`.
- **Only comment code that genuinely needs clarification.** Do not narrate what code does.
- Small, focused crates; prefer adding a new module over a new crate.
- Public APIs get doc comments; internal helpers usually don't.
- Test additions live inline as `#[cfg(test)] mod tests` per crate.

## Working discipline (spec-driven)

We follow the spec-driven workflow described in [SPEC.md § Workflow](SPEC.md#workflow):
`/spec` → `/plan` → `/build` → `/review` → `/ship`. One epic is active at a
time; when switching epics, replace `tasks/plan.md` + `tasks/todo.md` in a
commit whose subject contains `CLEANING_TASKS`.

For any change big enough to be architecturally load-bearing, write an ADR
under `design/decisions/` before touching code.

## Commit rules

- **Always commit before ending a task that changed code.**
- Conventional-commit-ish subjects (`feat:`, `fix:`, `chore:`, `syntax:`, `cli:`, `docs:`) — short imperative summary.
- Include the Copilot co-author trailer on agent-authored commits:

  ```
  Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
  ```

- Never rewrite pushed history without asking.

## Where things live

```
AGENTS.md              → this file
LANGUAGE.md            → language reference (aspirational + implemented)
SPEC.md                → repo-level spec + spec-driven workflow
README.md              → short outward-facing overview
crates/                → Rust workspace (lexer, ast, parser, runtime, cli)
examples/              → runnable .iklo programs
examples/planned/      → .iklo programs that showcase syntax not yet implemented
design/decisions/      → ADRs (ADR-NNNN, never deleted; superseded/amended)
spec/                  → per-epic specs (spec-driven development)
tasks/                 → active epic's plan.md + todo.md
refs/                  → reference material (UCBLogo, NetLogo) — not code
AGENTS.old.md, README.old.md, tour.old.iklo → historical snapshots; do not edit
```
