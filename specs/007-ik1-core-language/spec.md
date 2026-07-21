# Feature Specification: IK1 Core Language

**Feature Branch**: `007-ik1-core-language`

**Created**: 2026-07-21

**Status**: Draft (Queued; activates after epic 004 leaves Draft)

**Input**: Build a minimum but reasonably complete language layer ("IK1") with
standard-library IO, function definitions, conditionals, loops, and a coherent
primitive-type set.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Script author can do IO

**Priority**: P1

A script author needs to read from stdio and write to stdout/stderr without
turning IO into a language primitive.

**Why this priority**: A practical language needs usable IO before broader feature growth.

**Independent Test**: A small script reads a line, transforms it, and prints a
result using standard-library functions.

**Acceptance Scenarios**:

1. **Given** an IK1 script using stdlib IO forms, **When** it runs in the CLI,
   **Then** it can read from stdin and write to stdout.
2. **Given** the same script, **When** IO is used, **Then** no new IO primitive
   syntax is required.

### User Story 2 - Author can define reusable functions

**Priority**: P1

A contributor needs closures and function definition syntax that works with
`fn` and `let`, with `to` deferred until a later syntax decision.

**Why this priority**: Reusable abstraction is required for non-trivial programs.

**Independent Test**: A function captures an outer binding, returns a value,
and can be stored in a lexical binding.

**Acceptance Scenarios**:

1. **Given** a closure created with `fn`, **When** it references an outer
   binding, **Then** lexical capture behaves predictably.
2. **Given** a function value, **When** stored and called via `let`, **Then**
   the call returns the expected value.

### User Story 3 - Author can express control flow

**Priority**: P1

A user needs `cond` and `repeat` to write nontrivial programs without relying
on ad hoc primitives.

**Why this priority**: Control flow is part of the minimum complete language bar.

**Independent Test**: A sample program uses `cond` and `repeat` to produce
predictable output.

**Acceptance Scenarios**:

1. **Given** multiple conditional branches, **When** `cond` evaluates, **Then**
   only the first matching branch result is produced.
2. **Given** a bounded iteration case, **When** `repeat` runs, **Then** loop
   semantics are deterministic.

### User Story 4 - Language has a stable primitive inventory

**Priority**: P2

A contributor needs a fixed primitive-type set with short, consistent names.

**Why this priority**: IK1 should name only what it needs; canonical full
inventory is owned by epic 010.

**Independent Test**: The type inventory is documented and the parser/runtime
accept the agreed primitive names.

**Acceptance Scenarios**:

1. **Given** IK1 documentation, **When** a contributor inspects primitive
   types, **Then** the IK1-required subset is explicit and references epic 010
   as canonical authority.

## Requirements *(mandatory)*

- **FR-001**: IO MUST be provided through the standard library, not as a
  primitive language form.
- **FR-002**: The language MUST support function definitions with `fn` and
  lexical capture via `let`/closures; this epic MAY extend parser/runtime rules
  beyond today's lexical-binding-only `let :name be <expr>` shape as needed.
- **FR-003**: The language MUST support `cond` for conditionals.
- **FR-004**: The language MUST support `repeat` for looping.
- **FR-005**: The language MUST define the IK1-required primitive subset and
  apply the short-name rule to that subset.
- **FR-006**: This epic MUST reference `specs/010-types-literals/` as the
  canonical full primitive inventory and literal-constructor authority.

## Success Criteria *(mandatory)*

- **SC-001**: A small program can read, branch, loop, and write output.
- **SC-002**: Function values can be created, stored, and called.
- **SC-003**: The primitive type list is stable and documented.
