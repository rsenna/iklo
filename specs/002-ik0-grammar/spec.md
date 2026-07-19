# Feature Specification: IK0 Formal Grammar

**Feature Branch**: `002-ik0-grammar`

**Created**: 2026-07-18

**Status**: Draft

**Input**: Formalize the Level 0.0 syntax as a W3C EBNF grammar with a derived-forms appendix. Grammar is the source of truth; parser follows it, not vice versa. First-ever grammar for the language — expected to change.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Language-design contributor has an unambiguous reference (Priority: P1) 🎯 MVP

A contributor arriving to work on the grammar, parser, or tooling reads `grammar.ebnf` and knows exactly what Level 0.0 accepts. The grammar is the single source of truth — not the parser code, not the examples, not LANGUAGE.md. If the parser disagrees with the grammar, the parser is wrong.

**Why this priority**: Without a formal grammar, every syntax discussion is hand-waving. This is the foundation for all future syntax work, tree-sitter generation, and language evolution.

**Independent Test**: A reader can compare the grammar against the parser source (`crates/iklo-parser/src/lib.rs`) and the lexer source (`crates/iklo-lexer/src/lib.rs`) and find no contradictions. Every example in `examples/` and `LANGUAGE.md` that claims to be Level 0.0 parses under the grammar.

**Acceptance Scenarios**:

1. **Given** `grammar.ebnf` exists with a complete W3C EBNF for Level 0.0, **When** a contributor reads it, **Then** they can determine whether any given input string is valid Level 0.0 syntax.
2. **Given** the grammar's lexical rules, **When** compared against `crates/iklo-lexer/src/lib.rs`, **Then** every token kind in the lexer corresponds to a lexical production in the grammar, and vice versa.
3. **Given** the grammar's syntactic rules, **When** compared against `crates/iklo-parser/src/lib.rs`, **Then** every production the parser implements corresponds to a syntactic production in the grammar, and vice versa.

---

### User Story 2 - Future contributor expands syntax with a derived form (Priority: P2)

A contributor wants to add syntactic sugar (e.g., `if`, `while`, `fn`) to the language. They read the grammar, understand the core forms vs. derived forms, and can express the new syntax as a derived form that expands to core forms. The appendix gives them the pattern to follow.

**Why this priority**: Derived forms are how the language grows without bloating the runtime. The appendix establishes the pattern for all future syntax extensions.

**Independent Test**: A reviewer reads the derived-forms appendix and can write an equivalent expansion for each listed form using only core AST nodes.

**Acceptance Scenarios**:

1. **Given** the derived-forms appendix, **When** a contributor reads a derived form entry, **Then** they see the surface syntax, the expansion to core forms, and which AST node(s) it produces.
2. **Given** a derived form in the appendix, **When** the expansion is substituted for the surface syntax in any valid program, **Then** the resulting program is still valid under the grammar.

---

### User Story 3 - Tooling author generates a parser from the grammar (Priority: P3)

A contributor wants to generate a tree-sitter grammar or other tooling from the EBNF. The grammar is structured in W3C format with clear lexical/syntactic separation, making it machine-parseable.

**Why this priority**: Future goal (tree-sitter generation). Not needed now, but the grammar should be structured to enable it.

**Independent Test**: The grammar's lexical and syntactic sections are cleanly separated, and productions are named consistently.

**Acceptance Scenarios**:

1. **Given** `grammar.ebnf`, **When** a tooling author parses it, **Then** lexical and syntactic productions are distinguishable by section headers.

---

### Edge Cases

- **What about `set`?** `set` is lexed but not yet parsed or evaluated. The grammar MUST include `set` as a lexical production and note it as reserved for future use (not in the syntactic grammar yet).
- **What about `=`?** `=` is lexed but unused in Level 0.0. The grammar MUST include it as a lexical production and note it as reserved.
- **What about `if`, `while`, `fn`, `be` as a keyword in contexts other than `let`?** These are not in Level 0.0. The grammar MUST NOT include them; they belong to future levels.
- **What about the `;` token?** Hard terminator. The grammar MUST define its role explicitly.
- **What about newline as soft terminator?** This is the trickiest part. The grammar MUST capture the newline-termination rule: newline ends the current expression only when that expression is already complete and the next line can't continue it. Inside `( )`, newlines are ignored.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `grammar.ebnf` MUST exist at `specs/002-ik0-grammar/grammar.ebnf` and contain a complete W3C EBNF grammar for Level 0.0.
- **FR-002**: The grammar MUST be split into two sections: **Lexical Grammar** (token-level rules) and **Syntactic Grammar** (expression-level rules).
- **FR-003**: Lexical productions MUST correspond 1:1 with `LexemeKind` variants in `crates/iklo-lexer/src/lib.rs`.
- **FR-004**: Syntactic productions MUST correspond to the parsing logic in `crates/iklo-parser/src/lib.rs`.
- **FR-005**: The grammar MUST use W3C EBNF notation: `=` for definition, `,` for concatenation, `|` for alternation, `[ ]` for optional, `{ }` for repetition, `( )` for grouping, `"..."` / `'...'` for terminals, `(* ... *)` for comments, `? ... ?` for special sequences (regex patterns for tokens).
- **FR-006**: The grammar MUST include a **Derived Forms** appendix listing every syntactic sugar currently implemented, with its expansion to core forms.
- **FR-007**: Core forms in the grammar MUST map to `Expr` variants in `crates/iklo-ast/src/lib.rs`: `Number(f64)` → numeric literal, `LexRef(String)` → `:name`, `Let { name, value }` → `let :name be expr`, `Binary { op, left, right }` → `expr op expr`.
- **FR-008**: The grammar MUST document the newline-as-soft-terminator rule precisely enough that a tool could implement it.
- **FR-009**: The grammar MUST note reserved tokens (`set`, `=`) that are lexed but not yet used syntactically.
- **FR-010**: The grammar MUST NOT include features beyond Level 0.0 (no `if`, `while`, `fn`, `be`-as-keyword-outside-`let`, etc.).
- **FR-011**: The grammar file MUST include a header comment stating: (a) this is Level 0.0, expected to change; (b) grammar is source of truth; (c) parser follows grammar, not vice versa.
- **FR-012**: The existing parser tests (`cargo test -p iklo-parser`) MUST pass unchanged — no code changes in this epic.
- **FR-013**: `make test && make build` MUST still succeed.

### Key Entities

- **Grammar**: The W3C EBNF specification defining valid Level 0.0 syntax. Lives in `grammar.ebnf`. Source of truth for what the language accepts.
- **Core Form**: An expression that maps directly to an `Expr` variant: number literal, lexical reference, `let` binding, binary operation. These are the only forms the runtime evaluates.
- **Derived Form**: Syntactic sugar that expands to core forms. Implemented in the parser (or could be a macro pass). Documented in the appendix with their expansion rules.
- **Lexical Grammar**: Token-level rules defining valid tokens (numbers, identifiers, operators, keywords, punctuation). Derived from the lexer.
- **Syntactic Grammar**: Expression-level rules defining how tokens combine into valid programs. Derived from the parser.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `specs/002-ik0-grammar/grammar.ebnf` exists and contains a complete W3C EBNF grammar covering all Level 0.0 syntax.
- **SC-002**: Every `LexemeKind` in `crates/iklo-lexer/src/lib.rs` has a corresponding lexical production in the grammar.
- **SC-003**: Every production the parser implements has a corresponding syntactic production in the grammar.
- **SC-004**: The derived-forms appendix lists at least: `set :name be expr` (mutation), and notes that `=` is reserved.
- **SC-005**: The grammar is written in valid W3C EBNF notation.
- **SC-006**: `cargo test -p iklo-parser` passes unchanged.
- **SC-007**: `cargo test -p iklo-lexer` passes unchanged.
- **SC-008**: `make test && make build` succeeds.

## Assumptions

- This is the first formal grammar for Iklo. It is expected to be revised as the language evolves.
- The grammar is a design document, not executable code. It may be used to generate a parser in the future, but that is out of scope for this epic.
- Level 0.0 excludes stdlib, syntax sugar beyond what's currently implemented, and any features not yet in the parser.
- The grammar must be precise enough to be the source of truth, but does not need to be formal enough for a proof of correctness.
- `set` and `=` are reserved in the grammar for future use; they are not part of the syntactic grammar yet.
