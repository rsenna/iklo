# iklo (minimum project)

Minimal Rust workspace for Iklo with:

- `iklo-lexer`: tokenization with source spans
- `iklo-parser`: small expression + `let` parser
- `iklo-runtime`: transactional live image core
- `iklo-cli`: runnable REPL/file runner

## Run

```bash
cargo run -p iklo-cli
```

or run a file:

```bash
cargo run -p iklo-cli -- examples/hello.iklo
```

## Language subset (IK0)

- number literals: `1`, `2.5`
- arithmetic: `+ - * /` (whitespace required around infix operators)
- lexical value binding: `let :x be 40 + 2`
- lexical value read: `:x`
- expressions separated by newline or `;`

`let` is an expression — it evaluates to the value it bound.

### Statement termination

A newline ends the current expression only if that expression is already
valid. If it isn't (e.g. a trailing binary operator, or we're waiting for
`:name` after `let`), the newline is treated as whitespace and parsing
continues on the next line. `;` always ends the current expression,
regardless of validity. Inside `( ... )` newlines are always whitespace.

```
let :x be 1 +
  2            # one expression: 1 + 2

1 + 2
* 3            # error: '1 + 2' is valid, so newline ends it

let :x be 1; :x   # two expressions, forced by ';'
```

## Transaction model

Every top-level evaluation runs as a transaction over the runtime image:

- success => commit (image revision increments)
- failure => rollback (image unchanged)

