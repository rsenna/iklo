# Feature Specification: IK0 Formal Grammar

**Feature Branch**: `002-ik0-grammar`

**Created**: 2026-07-18

**Status**: Implemented

**Input**: Formalize the Level 0.0 syntax as an LALRPOP grammar that is both the authoritative specification and the parser implementation. Grammar.lalrpop is the single source of truth — if the parser disagrees with the grammar, the parser is wrong (because the parser IS the grammar). First-ever grammar for the language — expected to change.

## User Scenarios & Testing

### User Story 1 - Language-design contributor has an unambiguous reference (Priority: P1) MVP

A contributor arriving to work on the grammar, parser, or tooling reads `grammar.lalrpop` and knows exactly what Level 0.0 accepts. The LALRPOP file is both the specification and the parser — no separate EBNF needed. If there's a disagreement, it's a compile error.

**Why this priority**: Without a formal grammar, every syntax discussion is hand-waving. This is the foundation for all future syntax work.

**Independent Test**: A reader can compare `grammar.lalrpop` against `token.rs` (token stream wrapper) and the AST types and find no contradictions. Every example in `examples/` that claims to be Level 0.0 parses under the grammar.

**Acceptance Scenarios**:

1. **Given** `grammar.lalrpop` exists with a complete LR grammar for Level 0.0, **When** a contributor reads it, **Then** they can determine whether any given input string is valid Level 0.0 syntax.
2. **Given** the grammar's extern token declarations, **When** compared against `token.rs`, **Then** every token kind in the token stream corresponds to a terminal in the grammar.
3. **Given** the grammar's productions, **When** compared against `iklo_ast`, **Then** every production produces a valid `Expr` variant.

---

## Requirements

### Functional Requirements

- **FR-001**: `grammar.lalrpop` MUST exist at `crates/iklo-parser/grammar.lalrpop` and contain a complete LALRPOP grammar for Level 0.0.
- **FR-002**: The grammar MUST be parseable by LALRPOP without conflicts (LR(1)).
- **FR-003**: The grammar's extern token declarations MUST correspond 1:1 with the `Token` enum in `token.rs`.
- **FR-004**: The grammar's productions MUST produce `Spanned<Expr>` values matching `iklo_ast`.
- **FR-005**: The grammar MUST handle `let :name be` as a statement-level construct (lowest precedence).
- **FR-006**: The existing parser tests (`cargo test -p iklo-parser`) MUST pass unchanged after the grammar replaces the hand-written parser.
- **FR-007**: `cargo test --workspace` MUST succeed.

### Key Entities

- **Grammar**: The LALRPOP specification defining valid Level 0.0 syntax. Lives in `grammar.lalrpop`. Source of truth and parser implementation in one.
- **Token Stream** (`token.rs`): Adapts the Logos lexer output to the LALRPOP token format, including newline filtering.
- **AST** (`iklo_ast`): `Expr` enum defines the core forms produced by the grammar.

## Success Criteria

- **SC-001**: `grammar.lalrpop` exists and is a valid LALRPOP grammar.
- **SC-002**: `cargo build -p iklo-parser` succeeds (grammar compiles).
- **SC-003**: All 10 parser tests pass.
- **SC-004**: `cargo test --workspace` succeeds.

## What was built

- `grammar.lalrpop` — LALRPOP grammar: Program, Separator, Statement, LetStmt, Expr, AddExpr, MulExpr, Atom, NumberExpr, LexRefExpr, ParenExpr
- `token.rs` — Token enum, LexicalError, TokenStream (Logos adapter with newline filtering)
- `lib.rs` — Thin wrapper over LALRPOP-generated parser
- `build.rs` — LALRPOP build script

## Assumptions

- This is the first formal grammar for Iklo. It is expected to be revised as the language evolves.
- The grammar IS the parser — no separate EBNF or hand-written parser.
- Level 0.0 excludes stdlib, syntax sugar beyond what's currently implemented, and any features not yet in the parser.
