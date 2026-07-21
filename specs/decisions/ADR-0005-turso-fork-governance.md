# ADR-0005 — Turso fork governance and execution gate

- **Status:** Accepted
- **Date:** 2026-07-21
- **Deciders:** @rsenna (with Copilot as sounding board)
- **Supersedes:** —
- **Superseded by:** —

## Decision (one sentence)

**Iklo adopts an adapter-first Turso integration policy, and any Turso fork execution is gated behind explicit blocker evidence plus a separate follow-up ADR/epic before code changes to Turso are allowed.**

## Context

ADR-0001 set sequencing: establish the `Substrate` boundary, then deliver a
Turso-backed substrate milestone, while deferring VDBE-targeted compiler work.
As planning for `004-turso-substrate-backend` advanced, a practical question
appeared: when a blocker is found, should we modify Iklo, upstream Turso, or a
Turso fork?

That decision is load-bearing and easy to mishandle if left implicit.

## What this commits us to

1. **Adapter-first default**
   - Implement Turso integration in `iklo-substrate-turso` using exposed/stable
     Turso interfaces first.
   - Prefer changes in Iklo when issues are adapter mapping, serialization,
     runtime/CLI policy, or call-site behavior.

2. **Blocker classification is mandatory**
   - Every integration blocker is recorded as one of:
     - `adapter-fixable`
     - `upstream-fixable`
     - `fork-required`
   - Each classification includes rationale and chosen next action.

3. **Fork execution gate**
   - A Turso fork is not implemented inside the active storage-backend epic.
   - If a blocker is `fork-required`, the team must open a follow-up ADR/epic
     that defines fork scope and delivery plan before any Turso fork code work.

4. **Fork governance (when gate is passed)**
   - Patch scope is bounded and explicit.
   - Upstream-first contribution is preferred when feasible.
   - A sync cadence with upstream Turso is defined and tracked.

## Non-decisions

- This ADR does **not** authorize VDBE compiler work.
- This ADR does **not** authorize immediate Turso fork implementation.
- This ADR does **not** change Iklo semantics; `iklo-runtime` remains the
  semantic reference.

## Consequences

- **Positive:** stronger consistency in integration decisions, reduced ad-hoc
  fork risk, clearer review expectations.
- **Negative:** extra process when blockers are hard, because fork execution
  requires explicit follow-up design/approval.

## Follow-ups

- Apply this policy in `specs/004-turso-substrate-backend/spec.md`.
- If fork-required blockers appear, open a new ADR/epic before fork code.
