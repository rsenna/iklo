# Feature Specification: IK1 Core Language

**Feature Branch**: `007-ik1-core-language`  
**Created**: 2026-07-21  
**Status**: Draft

**Input**: Build a minimum but reasonably complete language layer ("IK1") with
standard-library IO, function definitions, conditionals, loops, and a coherent
primitive-type set.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Script author can do IO

A script author needs to read from stdio and write to stdout/stderr without
turning IO into a language primitive.

**Independent Test**: A small script reads a line, transforms it, and prints a
result using standard-library functions.

### User Story 2 - Author can define reusable functions

A contributor needs closures and function definition syntax that works with
`fn` and `let`, with `to` deferred until a later syntax decision.

**Independent Test**: A function captures an outer binding, returns a value,
and can be stored in a lexical binding.

### User Story 3 - Author can express control flow

A user needs `cond` and `repeat` to write nontrivial programs without relying
on ad hoc primitives.

**Independent Test**: A sample program uses `cond` and `repeat` to produce
predictable output.

### User Story 4 - Language has a stable primitive inventory

A contributor needs a fixed primitive-type set with short, consistent names.

**Independent Test**: The type inventory is documented and the parser/runtime
accept the agreed primitive names.

## Requirements *(mandatory)*

- **FR-001**: IO MUST be provided through the standard library, not as a
  primitive language form.
- **FR-002**: The language MUST support function definitions with `fn` and
  lexical capture via `let`/closures.
- **FR-003**: The language MUST support `cond` for conditionals.
- **FR-004**: The language MUST support `repeat` for looping.
- **FR-005**: The language MUST define a canonical primitive-type inventory
  with short names and consistent width/name rules.
- **FR-006**: The language MUST document how primitive names map to literal
  syntax and to standard-library APIs.

## Success Criteria *(mandatory)*

- **SC-001**: A small program can read, branch, loop, and write output.
- **SC-002**: Function values can be created, stored, and called.
- **SC-003**: The primitive type list is stable and documented.

