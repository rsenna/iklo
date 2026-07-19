# Implementation Plan: IK0 Formal Grammar

**Branch**: `002-ik0-grammar` | **Date**: 2026-07-18 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/002-ik0-grammar/spec.md`

## Summary

Formalize the Level 0.0 syntax as a W3C EBNF grammar (`grammar.ebnf`) with a derived-forms appendix. This is a documentation-only epic — no code changes. The grammar is derived by reading the existing lexer and parser source code and expressing their combined behaviour in standard W3C EBNF notation.

## Technical Context

**Language/Version**: Rust 1.86+ (from `mise.toml`)
**Primary Dependencies**: None (documentation only)
**Storage**: N/A
**Testing**: `cargo test -p iklo-parser && cargo test -p iklo-lexer` (must pass unchanged)
**Target Platform**: N/A (documentation)
**Project Type**: Documentation epic
**Performance Goals**: N/A
**Constraints**: Grammar must be accurate to the existing parser/lexer; no code changes allowed
**Scale/Scope**: Single file (`grammar.ebnf`) with ~200 lines of EBNF

## Constitution Check

- **I. Test-First**: N/A — no code changes. Verification is: existing tests pass unchanged, grammar matches source.
- **II. One Epic In Flight**: ✅ — 001-substrate is shipped, this is the next.
- **III. Substrate Before Feature**: ✅ — grammar is documentation, not a feature that depends on substrate.
- **IV. Kebab-Case Iklo / Idiomatic Rust**: N/A — no code.
- **V. Comments Justify Themselves**: The grammar file uses EBNF comments (`(* ... *)`) to explain non-obvious decisions (e.g., newline handling).
- **VI. Load-Bearing Decisions → ADRs**: No ADR needed — this is documenting what exists, not making new design decisions.
- **VII. No Workarounds Left Standing**: N/A.

## Approach

### Source Material

The grammar is derived from three sources:

1. **Lexer** (`crates/iklo-lexer/src/lib.rs`) — `LexemeKind` enum with logos attributes defines token patterns
2. **Parser** (`crates/iklo-parser/src/lib.rs`) — Pratt parser with precedence climbing defines expression structure
3. **AST** (`crates/iklo-ast/src/lib.rs`) — `Expr` enum defines the core forms

### Grammar Structure

```
grammar.ebnf
├── §1 Lexical Grammar
│   ├── §1.1 Input elements (whitespace, comments, tokens)
│   ├── §1.2 Tokens
│   ├── §1.3 Keywords (let, be)
│   ├── §1.4 Literals (number)
│   ├── §1.5 Names (colon_name, identifier)
│   ├── §1.6 Operators (+, -, *, /)
│   ├── §1.7 Punctuation ((, ), ;)
│   └── §1.8 Newlines
├── §2 Syntactic Grammar
│   ├── §2.1 Program structure (separator, program)
│   ├── §2.2 Expressions (Pratt precedence)
│   ├── §2.3 Numeric literals
│   ├── §2.4 Lexical references
│   ├── §2.5 Let expressions
│   └── §2.6 Parenthesized expressions
├── Appendix A: Derived Forms
│   ├── A.1 Mutation (set — reserved)
│   └── A.2 Reserved tokens (=, identifiers)
└── Appendix B: Examples
```

### Key Design Decisions

1. **Newline handling**: The grammar expresses newline-as-soft-terminator via the `program` production and a prose explanation. A purely structural EBNF cannot capture the parser's contextual newline logic, so the rule is documented in comments and the `separator` production covers both newline and `;`.

2. **Derived forms**: Only `set` is listed (reserved, not implemented). This establishes the pattern for future entries.

3. **Reserved tokens**: `=` and bare identifiers are documented as reserved but not part of the syntactic grammar.

### Verification Strategy

- Each `LexemeKind` variant → lexical production
- Each parser code path → syntactic production
- Each `Expr` variant → documented in core forms section
- Existing tests pass unchanged
- Examples in Appendix B are valid under the grammar

## Project Structure

### Documentation (this feature)

```text
specs/002-ik0-grammar/
├── spec.md       # Feature specification
├── plan.md       # This file
├── grammar.ebnf    # The W3C EBNF grammar (primary deliverable)
└── tasks.md      # Task list (created by /speckit.tasks)
```

### Source Code (repository root)

No source code changes in this epic. Files referenced:

```text
crates/
├── iklo-lexer/src/lib.rs    # Source of lexical grammar
├── iklo-parser/src/lib.rs   # Source of syntactic grammar
└── iklo-ast/src/lib.rs      # Source of core form definitions
```

**Structure Decision**: Documentation-only epic. No new source files.
