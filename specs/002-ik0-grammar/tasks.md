---
description: "Task list for IK0 formal grammar"
---

# Tasks: IK0 Formal Grammar

**Input**: Design documents from `/specs/002-ik0-grammar/`

**Prerequisites**: [plan.md](plan.md) (required), [spec.md](spec.md) (required for user stories)

**Tests**: Tests are REQUIRED where applicable. This is primarily a documentation epic; verification is by comparison against source code and test passage.

**Organization**: Tasks are grouped by phase. Each task ends with a commit.

## Format: `[ID] [P?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)

---

## Phase 1: Grammar Draft

**Purpose**: Create the primary deliverable — the W3C EBNF grammar file.

- [ ] **T001** Draft `specs/002-ik0-grammar/grammar.ebnf` with:
  - Header comment (Level 0.0 scope, source-of-truth status, expected to change)
  - §1 Lexical Grammar: all token-level rules derived from `LexemeKind` in `crates/iklo-lexer/src/lib.rs`
  - §2 Syntactic Grammar: all expression-level rules derived from the parser in `crates/iklo-parser/src/lib.rs`
  - Appendix A: Derived Forms (set — reserved, = — reserved, bare identifiers — reserved)
  - Appendix B: Examples (valid Level 0.0 programs with expected AST output)
  
  **Acceptance**: `grammar.ebnf` exists and contains W3C EBNF covering all `LexemeKind` variants and all parser productions. Every `Expr` variant in `crates/iklo-ast/src/lib.rs` is documented as a core form.

---

## Phase 2: Verification

**Purpose**: Ensure the grammar is accurate and existing tests pass.

- [ ] **T002** Cross-check grammar against source code:
  - Verify every `LexemeKind` variant has a lexical production
  - Verify every parser code path has a syntactic production
  - Verify every `Expr` variant is documented
  - Verify reserved tokens (`=`, identifiers, `set`) are noted
  
  **Acceptance**: No contradictions found between grammar and source. Document any discrepancies in the commit message.

- [ ] **T003** Run existing tests to confirm no regressions:
  - `cargo test -p iklo-parser`
  - `cargo test -p iklo-lexer`
  - `make test && make build`
  
  **Acceptance**: All tests pass. No code changes were made.

---

## Phase 3: Polish

**Purpose**: Final review and epic closure.

- [ ] **T004** Review grammar for:
  - W3C EBNF notation correctness
  - Consistent naming (kebab-case for non-terminals, matching Iklo conventions)
  - Clear comments explaining non-obvious rules (newline handling, precedence)
  - Appendix examples are valid under the grammar
  
  **Acceptance**: Grammar is clean, consistent, and ready for use as source of truth.

- [ ] **T005** Update `AGENTS.md` "What is actually implemented today" section: add note that a formal grammar exists at `specs/002-ik0-grammar/grammar.ebnf`.

  **Acceptance**: `AGENTS.md` references the grammar.

- [ ] **T006** Run the full gate: `make test && make build`. Both must exit 0. Open a PR from `002-ik0-grammar` → `main`.

  **Acceptance**: PR opened with all commits.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Grammar Draft)**: no dependencies.
- **Phase 2 (Verification)**: depends on Phase 1.
- **Phase 3 (Polish)**: depends on Phase 2.

### Task Dependencies

```
T001 → T002 → T003 → T004 → T005 → T006
```

### Parallel Opportunities

- T005 (`AGENTS.md`) can run in parallel with T004 (grammar review) if convenient, but sequentially is simpler for a documentation epic.

---

## Notes

- This is a documentation-only epic — no code changes.
- The grammar is derived by reading source code, not by running the parser.
- If a contradiction is found between grammar and source, the grammar wins (source of truth principle), but fixing the source is out of scope for this epic.
- Commit subjects use `docs:` prefix for documentation changes.
