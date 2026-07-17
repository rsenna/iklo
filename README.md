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
- expressions separated by `;` (statement terminator; newline handling coming next)

`let` is an expression — it evaluates to the value it bound.

## Transaction model

Every top-level evaluation runs as a transaction over the runtime image:

- success => commit (image revision increments)
- failure => rollback (image unchanged)

