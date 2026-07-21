# Feature Specification: Binding Kinds Implementation

**Feature Branch**: `009-binding-kinds`  
**Created**: 2026-07-21  
**Status**: Draft

**Input**: Implement the binding kinds currently named transactional, form,
interface, computation, option, static, lexical, dynamic, reactive, and
synchronised.

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

- **FR-001**: The runtime MUST support transactional bindings.
- **FR-002**: The runtime MUST support form bindings.
- **FR-003**: The runtime MUST support interface bindings.
- **FR-004**: The runtime MUST support computation bindings.
- **FR-005**: The runtime MUST support option/keyword bindings.
- **FR-006**: The runtime MUST support static bindings.
- **FR-007**: The runtime MUST support lexical bindings.
- **FR-008**: The runtime MUST support dynamic bindings.
- **FR-009**: The runtime MUST support reactive bindings.
- **FR-010**: The runtime MUST support synchronised bindings.

## Success Criteria *(mandatory)*

- **SC-001**: Each binding kind has at least one proving test.
- **SC-002**: The user-visible documentation explains when to use each kind.
- **SC-003**: The runtime behavior matches the taxonomy epic.

