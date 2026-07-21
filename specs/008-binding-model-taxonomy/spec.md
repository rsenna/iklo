# Feature Specification: Binding Model Taxonomy

**Feature Branch**: `008-binding-model-taxonomy`

**Created**: 2026-07-21

**Status**: Draft (Queued; activates after epic 004 leaves Draft)

**Input**: Define the terms and dimensions behind the current `Engine` column in
`LANGUAGE.md` so binding semantics can be discussed without ambiguity. This epic
uses **binding mode** as the canonical term for that column.

## Ubiquitous Language Conventions

This epic standardizes terminology used by follow-up epics:

- **Option** is the canonical name for `key`-style values/bindings. "Keyword"
  may appear only as a cross-language comparison note.
- **Epic** means a delivery goal; **spike** means exploratory work that reduces
  uncertainty before delivery planning.
- **Word** is deprecated in specs. Use **token** for symbolic units and
  **form** for computation constructs identified by token/pattern.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Language designer gets a better vocabulary

A language designer wants one name for the concept currently labeled
`Engine` and wants it to mean the same thing everywhere.

**Independent Test**: The reference docs use one canonical term and explain why
it was chosen.

### User Story 2 - Implementer can classify binding behaviors

An implementer needs a stable taxonomy that separates storage, evaluation
timing, mutability, and side-effect access.

**Independent Test**: Each existing binding category is placed into the same
classification table without contradiction.

### User Story 3 - Future binding epics have a map

Future implementation epics need a shared matrix for comparing binding
behaviors.

**Independent Test**: The taxonomy includes dimensions, examples, and
non-goals.

## Requirements *(mandatory)*

- **FR-001**: The epic MUST adopt **binding mode** as the canonical term for
  the current `Engine` column and update docs to use it consistently.
- **FR-002**: The epic MUST define the dimensions that distinguish binding
  behaviors.
- **FR-003**: The epic MUST explain how the taxonomy applies to user-facing
  docs and to internal implementation.
- **FR-004**: The epic MUST identify which parts are naming/terminology only
  and which parts require runtime work later.
- **FR-005**: The epic MUST codify the terms option, token, and form as
  described above and remove use of deprecated "word" in new spec artifacts.

## Success Criteria *(mandatory)*

- **SC-001**: The docs use one stable term instead of multiple synonyms.
- **SC-002**: The taxonomy can classify every current and proposed binding
  behavior.
- **SC-003**: The taxonomy is reusable by the implementation epic.
