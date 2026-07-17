# `spec/` — per-epic specifications

Each subdirectory here is one epic. An epic has exactly one file to start with,
`SPEC.md`, and inherits everything from the repo-level [`../SPEC.md`](../SPEC.md).

## Layout

```
spec/
  README.md            → this file
  <epic-slug>/
    SPEC.md            → Objective + Success Criteria (+ overrides if any)
```

## What an epic SPEC.md contains

Minimum:

- **Objective** — one paragraph. What are we building and why?
- **Success Criteria** — a bulleted, checkable list. When are we done?

Optional, if the epic genuinely diverges from the repo spec:

- **Scope** / **Non-goals**.
- **Overrides** — list any section from [`../SPEC.md`](../SPEC.md) this epic
  intentionally overrides.
- **Design notes** — brief. For load-bearing choices, write an ADR under
  [`../design/decisions/`](../design/decisions/) instead.

## Lifecycle

- Authored during `/spec`.
- Read by `/plan` to produce [`../tasks/plan.md`](../tasks/) and
  [`../tasks/todo.md`](../tasks/) for the epic.
- Never overwritten once accepted; if the direction changes materially, write
  a follow-up epic or an ADR that supersedes the earlier decision.

## Current epics

_None yet._ The Turso/`ImageStore` epic will be the first — see
[ADR-0001](../design/decisions/ADR-0001-turso-vdbe-image-store.md).
