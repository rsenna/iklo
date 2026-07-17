# Iklo

A modern Logo dialect and interpreter, written in Rust. Iklo takes its inspiration from [Berkeley Logo (UCBLogo)](https://people.eecs.berkeley.edu/~bh/logo.html) — the evaluation model, the turtle, the exploratory feel — but it is **not** a UCBLogo-compatible implementation, and does not try to be. Lexical scope, sigils, `is … end` blocks, macros, and a shell dialect all depart from it deliberately. See [ADR-0008](design/decisions/ADR-0008-ucblogo-is-inspiration-not-a-compatibility-target.md).

> **Status: early development.** The core evaluator, parser, module system, and IntelliJ plugin are working, and the `logosh` shell dialect runs external commands with pipes and redirections. Many primitives are still marked `[TBI]`. **There is no macro system yet** — quasi-quoting exists as syntax only; see [ADR-0004](design/decisions/ADR-0004-macros-and-bounded-reader-extension.md). The language design is stable enough to write real programs but will keep evolving. See [Status & roadmap](#status--roadmap).

---

## What is Iklo?

Logo is a language built around the idea that programming should be exploratory and immediate. UCBLogo is the gold-standard free implementation. Iklo keeps that ethos and the turtle-graphics heritage, and departs wherever a modern language wants to — starting with scope:

- **Lexical scope by default** — procedures close over where they are *defined*, not where they are *called*. Dynamic scope is available via the `$name` sigil.
- **`is … end` block syntax** — every block has an explicit opening keyword and a matching `end`, with no ambiguity about where the body starts.
- **Flat operator precedence** — no PEMDAS surprises. Use parentheses to group explicitly.
- **Pattern-based (mixfix) procedure *declarations*** — `to :a nand :b is … end` parses, and literal words in the pattern are recorded. *Call sites do not yet honour the pattern* (`nand` is callable only as `nand(a, b)`); making literal words required tokens at the call site is [tracked in `spec/backlog.md` §1](spec/backlog.md) and is the precondition for macros.
- **Module system** — `export`, `import`, `use`, and `module/name` qualified access, inspired by Clojure namespaces.
- **Quasi-quoting (syntax only)** — `` `expr ``, `~expr`, `_expr` lex and parse into AST nodes, but `~`/`_` are a runtime error outside a quasi-quote: there is no macro system for them to serve yet.
- **Line comments** — `#` (Grammar 2.0). `;` is deprecated and warns; `//` and `/* … */` were removed.
- **Sigils** — `'word` for word literals, `:name` for lexical reads, `$name` for dynamic reads, with a planned *bounded*, user-definable sigil mechanism ([ADR-0004](design/decisions/ADR-0004-macros-and-bounded-reader-extension.md)).
- **JavaScript code generation** — experimental `iklo-js` backend emits ES modules.
- **IntelliJ plugin** — syntax highlighting and bracket matching for `.ls` files.
- **LogoShell dialect** — the `logosh` binary runs Iklo as an interactive Unix shell: external commands, pipelines (`|`), redirections (`>`, `>>`, `<`, `2>&1`), `NAME=value` env prefixes, and `$status`.

[`refs/ucblogo/summary.md`](ucblogo/summary.md) is the reference we consult first — but it is a *baseline for
comparison*, not a fallback specification. Where Iklo is silent, the behaviour is
**undecided**, not inherited ([ADR-0008](design/decisions/ADR-0008-ucblogo-is-inspiration-not-a-compatibility-target.md)).

---

## Quick tour

```iklo
# Recursive factorial — bracket and is…end bodies are interchangeable
to factorial :n [if :n == 0 then 1 else :n * factorial(:n - 1) end]

to factorial-iter :n is
  to go :acc :k is
    if :k == 0 then :acc else go(:acc * :k, :k - 1) end
  end
  go(1, :n)
end

print factorial(10)        # 3628800
print factorial-iter(10)   # 3628800
```

```iklo
# Lexical closures
to make-adder :n is
  fn :x is :x + :n end
end

let 'add5 make-adder 5
print apply(:add5, 3)   # 8
```

> **Note on paren-free calls.** A paren-free prefix call currently consumes **exactly one**
> argument, so `factorial :n - 1` parses as `factorial(:n) - 1` (an infinite recursion) and
> `go 1 :n` as `go(1)` then `:n`. A 0-argument procedure likewise needs `no-args()` — bare
> `no-args` evaluates to the procedure itself ([#24](https://github.com/rsenna/Iklo/issues/24)).
> Use the explicit call form until declarative arity lands; the parentheses above are a
> workaround, not the intended surface. See [`spec/backlog.md`](spec/backlog.md) §1 and
> [ADR-0004](design/decisions/ADR-0004-macros-and-bounded-reader-extension.md).

```iklo
# Module system
import math          # loads math.ls, registers math/* names

let 'area math/pi * math/square(5)
print area           # ≈ 78.54

use math             # bring exports into flat scope
print cube(3)        # 27
```

---

## Building

### Prerequisites

- [Rust](https://rustup.rs/) 1.75 or later (`rustup update stable`)
- JDK 17+ — only needed to build plugins (IntelliJ, etc.); the core crates build without it

### Common commands

```sh
make          # dev build, plugins skipped  (alias for `make build`)
make release  # release build, plugins built automatically
make test     # run all tests
make plugins  # force-build all plugins in dev profile
make plugin-iklo-intellij  # force-build one specific plugin
```

All `make` targets are thin wrappers over `cargo`; you can use `cargo` directly too:

```sh
cargo build                          # dev build
cargo build --release                # release build (includes plugins)
cargo run -p iklo-cli          # REPL
cargo run -p iklo-cli -- examples/factorial.ls  # run a file
cargo test                           # all tests
```

### Binaries

A build produces two binaries:

| Binary       | Crate              | Purpose                                                        |
|--------------|--------------------|----------------------------------------------------------------|
| `iklo` | `iklo-cli`   | Iklo interpreter + REPL (and JS emit via `--emit-js`)     |
| `logosh`     | `iklo-shell` | LogoShell — the interactive shell dialect of Iklo         |

By convention, **`.ls`** is the recommended extension for Iklo source and **`.lsh`** for
LogoShell-specific scripts. The two dialects share one reader; the dialect is currently selected by
*which binary* you run (`iklo` = strict Iklo, `logosh` = LogoShell), with shebang-based
self-selection planned (`spec/grammar-2.0/SPEC.md` §13c).

### Plugin build gating

Plugin crates live in `plugins/` and invoke external build systems (Gradle for the IntelliJ plugin). To avoid slowing down every `cargo build`, they are **skipped in the dev profile by default** and only run automatically on `cargo build --release`.

Override with the `iklo_BUILD_PLUGINS` env var:

| Command | Effect |
|---------|--------|
| `cargo build` | plugins skipped |
| `cargo build --release` | plugins built |
| `iklo_BUILD_PLUGINS=1 cargo build -p iklo-intellij` | force-build one plugin |
| `iklo_BUILD_PLUGINS=0 cargo build --release` | release build, plugins suppressed |
| `make plugins` | force-build all plugins (dev profile) |

If no JDK is found, the plugin build prints a warning and continues rather than failing. Set `iklo_REQUIRE_JDK=1` to make a missing JDK a hard error (recommended in CI).

---

## Repository layout

```
iklo/
├── crates/
│   ├── iklo-lexer/    # `logos`-generated DFA tokeniser; span tracking
│   ├── iklo-ast/      # Expr, Spanned<T>, FnPattern, BinOp, …
│   ├── iklo-parser/   # Pratt expression parser + recursive-descent stmts
│   ├── iklo-interp/   # Tree-walking interpreter; Value, Env, runtime
│   ├── iklo-js/       # Experimental JS/ES-module code generator
│   ├── iklo-cli/      # `iklo` binary — REPL, file runner, JS emit
│   └── iklo-shell/    # `logosh` binary — LogoShell interactive shell
├── plugins/
│   └── iklo-intellij/ # IntelliJ Platform plugin (Kotlin/Gradle); release-only by default
├── examples/               # ⚠ all four currently fail to run — see issue tracker
│   ├── hello.ls            # Minimal starter program
│   ├── factorial.ls        # Recursion, closures, list operations
│   ├── macros.ls           # Documents a `macro` form that does not exist
│   └── modules.ls          # import / use / export
├── design/                 # Roadmap + design records (start at design/README.md)
├── refs/                   # Third-party reference material — NOT MIT (see refs/README.md)
│   ├── ucblogo/            # summary.md, Harvey's evaluator notes, UCBLogo + CSLS PDFs
│   └── netlogo/            # NetLogo user manual
├── Makefile                # Build targets: build, release, test, plugins, plugin-<name>
├── AGENTS.md               # Canonical language reference
└── LICENSE
```

---

## Documentation

- **Language reference** — [`AGENTS.md`](../AGENTS.md) is the canonical spec: main distinctions from UCBLogo, evaluation model, sigils, block syntax, and the full [Primitive Reference](../AGENTS.md#primitive-reference) with per-primitive status tags.
- **UCBLogo comparison baseline** — [`refs/ucblogo/summary.md`](ucblogo/summary.md). Iklo does **not** inherit from it; where Iklo is silent the behaviour is *undecided* ([ADR-0008](design/decisions/ADR-0008-ucblogo-is-inspiration-not-a-compatibility-target.md)). Primary sources and licensing: [`refs/README.md`](README.md).
- **Design & planning hub** — [`design/`](design/) collects design navigation and records; start at [`design/README.md`](design/README.md), which routes to each document.

## Status & roadmap

Iklo uses **spec-driven development**. The repo-level [`SPEC.md`](../SPEC.md) holds the cross-cutting spec (commands, structure, style, testing, boundaries, and the `/spec` → `/plan` → `/build` workflow); each epic has its own spec under [`spec/`](spec/):

- [`spec/backlog.md`](spec/backlog.md): the current backlog (§0 doc/code drift, §1–§12 language design & primitives, §15 build order, §16 infrastructure). Status is cross-checked against the source rather than the spec's (sometimes stale) tags.
- [`spec/grammar-2.0/SPEC.md`](spec/grammar-2.0/SPEC.md): the unified Iklo/LogoShell reader (§13; folds in the former Grammar 2.0 design record).
- [`spec/logoshell/SPEC.md`](spec/logoshell/SPEC.md): the LogoShell runtime (§14) — tiers from MVP (commands, pipes, redirections, `$status` — largely done) to daily driver to polish.
- [`spec/ffi/SPEC.md`](spec/ffi/SPEC.md): foreign function interface & external bindings (§17).
- [`spec/workspace-mgmt/SPEC.md`](spec/workspace-mgmt/SPEC.md): UCBLogo Ch. 7 workspace management subset (§18).
- [`spec/macros/SPEC.md`](spec/macros/SPEC.md): macros & syntactic extension (§20) — static namespaces, declarative call extent, code↔data, the expander. Rationale in [ADR-0004](design/decisions/ADR-0004-macros-and-bounded-reader-extension.md).

**Bugs** are tracked as [GitHub Issues](https://github.com/rsenna/Iklo/issues) (label `bug`), not a file ledger. The *why* behind significant, hard-to-reverse decisions is recorded as [Architecture Decision Records](design/decisions/) (`design/decisions/`).

---

## License

MIT — see [LICENSE](LICENSE).
