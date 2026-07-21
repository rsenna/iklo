# Iklo

Iklo is a Rust-implemented language and runtime with a transactional live image.
Today it ships as a workspace centered on a REPL/file runner executable named
`iklo` (provided by the `iklo-cli` crate).

> [!IMPORTANT]
> `LANGUAGE.md` includes both implemented and aspirational design.
> For the current implemented surface, treat `AGENTS.md` as source-of-truth.

## What works today

- Lexer (`iklo-lexer`) with `Lexeme` output.
- AST (`iklo-ast`) for number, lexical reference, let, and binary expressions.
- Parser (`iklo-parser`) with LALRPOP grammar, operator precedence, and
  soft-newline termination.
- Runtime (`iklo-runtime`) with transactional evaluation over the live image.
- Substrate boundary (`iklo-substrate`) with in-memory implementation.
  A Turso-backed alternative (`iklo-substrate-turso`, local-file-only) is
  available opt-in behind a Cargo feature and CLI flag; in-memory remains
  the default.
- CLI (`iklo-cli`) REPL and file runner (`iklo` executable).

## Quickstart

### Prerequisites

- Rust toolchain (edition 2021 workspace)
- Optional: `mise` (`mise install`) to hydrate pinned toolchains

### Build and test

```bash
make build
make test
```

### Run the REPL

```bash
cargo run -p iklo-cli
```

### Run a program file

```bash
cargo run -p iklo-cli -- examples/hello.iklo
```

## Language snapshot (implemented subset)

- Numeric literals: `1`, `2.5`
- Arithmetic: `+ - * /` (whitespace required around infix operators)
- Lexical binding: `let :x be 40 + 2`
- Lexical read: `:x`
- Expression separators: newline (soft) or `;` (hard)

`let` is an expression: it returns the bound value.

### Newline semantics

Newline is a soft terminator: it ends the current expression when that
expression is already complete. If the expression is incomplete (for example,
after a trailing operator), parsing continues on the next line.
`;` always forces termination. Newlines inside `( ... )` are ignored.

```iklo
let :x be 1 +
  2            # one expression: 1 + 2

1 + 2
* 3            # error: `1 + 2` is complete, so newline terminated it

let :x be 1; :x # forced two expressions via hard terminator
```

## REPL commands

REPL commands are slash-prefixed and recognized only at a fresh prompt:

- `/quit` — **Purpose:** exit the REPL session. **Current status:** implemented and working.
- `/revision` — **Purpose:** show the current runtime-image revision counter.
  **Current status:** implemented; prints a number (`0` on a fresh session, then
  increments after each successful top-level evaluation commit).
- `/env` — **Purpose:** inspect current lexical bindings in the runtime image.
  **Current status:** implemented; prints one binding per line as `:name = value`
  (prints nothing when no bindings exist yet).

## Transaction model

Each top-level evaluation runs in a transaction:

- success -> commit (revision increments)
- failure -> rollback (image unchanged)

## Workspace layout

- `crates/iklo-lexer`
- `crates/iklo-ast`
- `crates/iklo-parser`
- `crates/iklo-runtime`
- `crates/iklo-substrate`
- `crates/iklo-substrate-turso` (opt-in, `turso` Cargo feature)
- `crates/iklo-cli`
- `examples/` runnable `.iklo` programs
- `specs/` feature specs and ADR-backed decisions

## Roadmap and governance

- Language reference: [LANGUAGE.md](LANGUAGE.md)
- Agent/project operating guide: [AGENTS.md](AGENTS.md)
- Design decisions (ADRs): [specs/decisions/](specs/decisions/)
- Active and planned epics: [specs/](specs/)

## License

Iklo's implementation in this repository is licensed under **GPL-3.0-or-later**
with an additional exception clarifying scope:

- Copyleft applies to Iklo implementation code in this repository.
- Programs, scripts, libraries, and outputs produced *using* Iklo are **not**
  automatically covered by this copyleft solely due to that use.

See [LICENSE](LICENSE) and [LICENSE.md](LICENSE.md) for the full terms.
