# Implementation Plan: IK0 Formal Grammar

**Branch**: `002-ik0-grammar` | **Date**: 2026-07-18 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/002-ik0-grammar/spec.md`

## Summary

Replace the hand-written Pratt parser with a LALRPOP-generated grammar that is both the formal specification and the parser implementation. The grammar.lalrpop file becomes the single source of truth for Level 0.0 syntax.

## Technical Context

**Language/Version**: Rust 1.86+ (from `mise.toml`)
**Primary Dependencies**: `lalrpop` 0.22 (build), `lalrpop-util` 0.22 (runtime)
**Storage**: N/A
**Testing**: `cargo test --workspace`
**Target Platform**: All Rust targets
**Project Type**: Parser implementation
**Constraints**: All existing tests must continue to pass

## Approach

### Architecture

```
iklo-lexer (Logos) → token.rs (TokenStream adapter + newline filtering) → LALRPOP parser (grammar.lalrpop) → AST
```

### Key Design Decisions

1. **`let :name be` is statement-level**: Placed at lowest precedence (like C's `=`) inside `Statement`, not `Expr`. This eliminates LALRPOP LR ambiguity.
2. **Newline filtering in token stream**: Lexer wrapper drops newlines based on previous token. Grammar sees only meaningful newlines.
3. **`@L`/`@R` for locations**: LALRPOP's location extraction used for span information.
4. **Named terminals**: All extern terminals must be named (not anonymous) to mix with `@L`/`@R`.

## Project Structure

### Source Code

```text
crates/iklo-parser/
├── grammar.lalrpop          # Authoritative grammar (LALRPOP)
├── build.rs                 # LALRPOP build script
└── src/
    ├── token.rs             # Token enum, TokenStream adapter
    └── lib.rs               # Parser entry point + tests
```

## What was implemented

### Completed

1. **LALRPOP grammar** (`grammar.lalrpop`):
   - Program → Separator* (Statement Separator*)*
   - Statement → LetStmt | Expr
   - LetStmt → "let" colon_name "be" Expr
   - Expr → AddExpr
   - AddExpr → MulExpr ("+" | "-") AddExpr | MulExpr
   - MulExpr → Atom ("*" | "/") MulExpr | Atom
   - Atom → ParenExpr | NumberExpr | LexRefExpr

2. **Token stream adapter** (`token.rs`):
   - Token enum (14 variants matching LALRPOP extern)
   - TokenStream implementing Iterator with newline filtering
   - Newlines dropped when prev is +, -, *, /, let, be, or :name when next is be

3. **Build script** (`build.rs`):
   - Uses `Configuration::new().set_in_dir().set_out_dir().process_file()` because LALRPOP defaults to `cwd/src`
   - Grammar compiles to `grammar.rs` in OUT_DIR

4. **Parser wrapper** (`lib.rs`):
   - `parse(source) -> Result<Program, ParseError>`
   - 10 tests covering all core syntax
