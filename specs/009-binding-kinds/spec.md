# Feature Specification: Binding Kinds Implementation

**Feature Branch**: `009-binding-kinds`

**Created**: 2026-07-21

**Status**: Draft (Queued; activates after epic 004 leaves Draft)

**Input**: Implement binding kinds using the terminology and matrix ratified by
`specs/008-binding-model-taxonomy/`. Current target set from `LANGUAGE.md`:
transactional (`gra`), form (`fm`), interface (`if`), computation (`cp`),
option (`key`, with static binding mode), lexical (`val`), dynamic
(`var`), reactive (`rx`), and synchronised (`sync`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Contributor can use the binding kinds intentionally

A contributor wants to choose a binding kind based on semantics rather than
guessing from the name.

**Independent Test**: The reference examples show how each kind behaves in a
small program.

### User Story 2 - Runtime can enforce the semantics

The runtime needs each binding kind to have a clear update/read/visibility
model.

**Independent Test**: Each kind has tests that prove its defining behavior.

### User Story 3 - Existing kinds remain distinguishable

A maintainer needs the binding system to stay internally coherent as more kinds
are added.

**Independent Test**: The matrix from the taxonomy epic maps cleanly to
implementation tests.

## Requirements *(mandatory)*

- **FR-001**: This epic MUST implement binding kinds and binding-mode behavior
  according to the taxonomy and names ratified in epic 008.
- **FR-002**: The runtime MUST support the graph-backed kinds:
  transactional (`gra`), form (`fm`), interface (`if`), and computation (`cp`).
- **FR-003**: The runtime MUST support lexical (`val`) and dynamic (`var`)
  kinds.
- **FR-004**: The runtime MUST support option (`key`) with static binding mode
  semantics (self-bound, global, and non-rebindable).
- **FR-005**: The runtime MUST support reactive (`rx`) and synchronised
  (`sync`) kinds.
- **FR-006**: Delivery SHOULD be phased (baseline then advanced kinds), but the
  final epic scope MUST cover all target kinds above.

## Success Criteria *(mandatory)*

- **SC-001**: Each binding kind has at least one proving test.
- **SC-002**: The user-visible documentation explains when to use each kind.
- **SC-003**: The runtime behavior matches the taxonomy epic.
