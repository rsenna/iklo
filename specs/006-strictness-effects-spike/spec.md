# Feature Specification: Strictness and Side-Effects Model Spike

**Feature Branch**: `006-strictness-effects-spike`

**Created**: 2026-07-21

**Status**: Draft (Queued; activates after epic 004 leaves Draft)

**Input**: Define how strictness and side effects should be exposed as language
constructs versus implemented internally in the runtime.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Language designer gets a shared model

**Priority**: P1

A language designer needs one vocabulary for strictness, laziness, purity, and
side effects before more syntax is added.

**Why this priority**: Naming drift here will create contradictory epics later.

**Independent Test**: A short design note classifies the current and proposed
language surfaces into strict/pure/effectful categories with no conflicting
terms.

### User Story 2 - Runtime implementer gets a boundary

**Priority**: P1

A runtime implementer needs to know which behaviors are language-visible and
which are internal execution policies.

**Why this priority**: This boundary prevents accidental runtime coupling in future work.

**Independent Test**: The design note states where side effects are surfaced,
where they are hidden, and which parts require ADRs.

### User Story 3 - Future epics can reference the model

**Priority**: P2

Future epics need a stable reference for how to expose strictness and effects.

**Why this priority**: Follow-up epics can proceed only after P1 terminology and boundaries are stable.

**Independent Test**: The spike produces a named taxonomy and a recommendation
for the next implementation epic.

## Requirements *(mandatory)*

- **FR-001**: The spike MUST define a canonical vocabulary for strictness,
  laziness, purity, and side effects.
- **FR-002**: The spike MUST distinguish language constructs from internal
  runtime implementation details.
- **FR-003**: The spike MUST identify which decisions are load-bearing enough to
  require ADRs.
- **FR-004**: The spike MUST recommend whether effect control belongs in
  surface syntax, standard library APIs, runtime metadata, or a combination.

## Success Criteria *(mandatory)*

- **SC-001**: The spike produces a versioned design note under `specs/` with
  explicit sections for terminology, surface constructs, internal policies, and
  ADR-needed decisions.
- **SC-002**: The design note classifies at least five concrete examples from
  current/proposed language behavior as strict/pure/effectful with no
  contradictory labels.
- **SC-003**: The note lists at least two enabled follow-up epics and the
  decisions each one depends on.
