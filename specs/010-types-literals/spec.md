# Feature Specification: Basic Types and Literal Constructors

**Feature Branch**: `010-types-literals`  
**Created**: 2026-07-21  
**Status**: Draft

**Input**: Expand the basic type system and standard literal-constructor forms
for primitive and composite values.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Source author can express values directly

A source author needs a stable literal syntax for the common primitive and
container types.

**Independent Test**: A sample program parses and evaluates literals for the
documented type set.

### User Story 2 - Runtime author can add literal constructors safely

A runtime author needs literal constructors to be pure, deterministic, and
easy to extend.

**Independent Test**: Literal constructors return values or deterministic parse
errors with source locations.

### User Story 3 - Language docs can describe one canonical inventory

A maintainer needs the type inventory and literal syntax to be documented in a
single place.

**Independent Test**: The language reference contains a canonical table of
types, names, and constructor forms.

## Requirements *(mandatory)*

- **FR-001**: The epic MUST define the canonical primitive-type inventory.
- **FR-002**: The epic MUST define the naming rule for primitive families and
  any width-suffixed variants.
- **FR-003**: The epic MUST define literal constructors for the documented
  primitive and composite types.
- **FR-004**: Literal constructors MUST be pure and return deterministic syntax
  errors when parsing fails.
- **FR-005**: Short literal forms MUST desugar to constructor calls before any
  later expansion step.

## Success Criteria *(mandatory)*

- **SC-001**: The documented type inventory matches the parser/runtime.
- **SC-002**: Literal constructors round-trip for the documented examples.
- **SC-003**: The type and literal documentation is stable enough to support
  future implementation epics.

