---
description: "Task list for IK0 formal grammar"
---

# Tasks: IK0 Formal Grammar

**Input**: Design documents from `/specs/002-ik0-grammar/`

## Completed Tasks

- [x] **T001** Create `grammar.lalrpop` with LALRPOP grammar for Level 0.0
  - Program, Separator, Statement, LetStmt, Expr, AddExpr, MulExpr, Atom, NumberExpr, LexRefExpr, ParenExpr
  - Extern block with 14 token declarations

- [x] **T002** Create `token.rs` with Token enum, LexicalError, TokenStream (Logos adapter + newline filtering)

- [x] **T003** Create `build.rs` for LALRPOP compilation (explicit in_dir/out_dir)

- [x] **T004** Rewrite `lib.rs` as thin wrapper over LALRPOP-generated parser

- [x] **T005** Fix LALRPOP compilation issues:
  - `grammar;` declaration required in LALRPOP 0.22
  - `type Location = usize` syntax (not `Location = usize;`)
  - `extern` block at end of file
  - Named terminals required when mixing with `@L`/`@R`
  - `@L`/`@R` for span locations
  - Program rule tuple mapping

- [x] **T006** Fix warnings:
  - `#[allow(non_camel_case_types)]` on Token enum (LALRPOP convention)
  - `_end` prefix for unused variables in lib.rs

- [x] **T007** All 10 parser tests pass, full workspace green

- [x] **T008** Delete obsolete `specs/002-ik0-grammar/grammar.bnf`

- [x] **T009** Update spec.md, plan.md, tasks.md for LALRPOP approach

## Remaining Tasks

- [ ] **T010** Commit changes with conventional commit message
- [ ] **T011** Open PR from `002-ik0-grammar` → `main`
