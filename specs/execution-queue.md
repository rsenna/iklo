# Epic Execution Queue

This document defines the current execution order for open epics and the
minimum start/done criteria for each one.

## Dependency order

`004 -> 006 -> 008 -> 007 -> 010 -> 009`

Epic `005` may start after `004`, but is preferred after `007/010` stabilization
to reduce release-policy churn while language/runtime semantics are changing.

## Queue

### 1. Epic 004 — Turso-backed Substrate Backend

- **Spec**: [004-turso-substrate-backend/spec.md](004-turso-substrate-backend/spec.md)
- **Start criteria**:
  - PR/state for spec-004 is merged/aligned.
  - `spec.md`, `plan.md`, and `tasks.md` are consistent.
  - Blocker inventory schema is explicit.
- **Done criteria**:
  - `iklo-substrate-turso` exists and integrates through `Substrate`.
  - Contract behavior matches in-memory substrate expectations.
  - CLI substrate mode wiring is complete with no silent fallback.
  - Blocker inventory is fully populated during implementation.
  - If a blocker is `fork-required`, a follow-up ADR/epic is opened (fork work
    stays outside epic 004).

### 2. Epic 006 — Strictness and Side-Effects Spike

- **Spec**: [006-strictness-effects-spike/spec.md](006-strictness-effects-spike/spec.md)
- **Start criteria**:
  - Epic 004 is no longer Draft.
- **Done criteria**:
  - A versioned design note is produced under `specs/`.
  - Strict/pure/effectful taxonomy is defined with concrete examples.
  - Language-surface vs runtime-internal boundaries are explicit.
  - ADR-needed decisions are listed.

### 3. Epic 008 — Binding Model Taxonomy

- **Spec**: [008-binding-model-taxonomy/spec.md](008-binding-model-taxonomy/spec.md)
- **Start criteria**:
  - Epic 006 design note is accepted.
- **Done criteria**:
  - Canonical vocabulary is ratified (`binding mode`, `option`, `token`, `form`).
  - `Engine` column mapping from `LANGUAGE.md` is documented.
  - Ambiguous/deprecated terms are removed from new spec artifacts.

### 4. Epic 007 — IK1 Core Language

- **Spec**: [007-ik1-core-language/spec.md](007-ik1-core-language/spec.md)
- **Start criteria**:
  - Epics 006 and 008 are accepted and linked.
- **Done criteria**:
  - Stdio IO is provided by standard library APIs (not a primitive).
  - `fn` + lexical `let :name be <expr>` closure flow works.
  - `cond` and `repeat` are implemented.
  - IK1 primitive subset is documented and explicitly linked to epic 010 as
    canonical full inventory owner.

### 5. Epic 010 — Types and Literal Constructors (Canonical)

- **Spec**: [010-types-literals/spec.md](010-types-literals/spec.md)
- **Start criteria**:
  - IK1 subset from epic 007 is stable enough to map forward.
- **Done criteria**:
  - Full primitive inventory is ratified.
  - Naming/width rules are finalized.
  - Literal constructors are specified with deterministic failure behavior.
  - IK1 subset cleanly maps to the canonical inventory.

### 6. Epic 009 — Binding Kinds Implementation

- **Spec**: [009-binding-kinds/spec.md](009-binding-kinds/spec.md)
- **Start criteria**:
  - Epic 008 terminology is ratified.
  - Baseline semantics from epics 007 and 010 are stable.
- **Done criteria**:
  - Target binding kinds are implemented in phased delivery.
  - Option static-mode semantics are enforced.
  - Proving tests exist per implemented binding kind.
  - Runtime behavior matches the taxonomy matrix.

### 7. Epic 005 — CI/Release/Versioning Implementation

- **Spec**: [005-ci-release-versioning/spec.md](005-ci-release-versioning/spec.md)
- **Start criteria**:
  - Epic 004 is no longer Draft.
  - Preferably after 007/010 stabilization.
- **Done criteria**:
  - PR CI runs `make test` and `make build`.
  - SemVer tag release pipeline is active.
  - Canonical version guard (`Cargo.toml` workspace version vs tag) is enforced.
  - Build identifier policy is implemented.
  - Commit-diff release notes and release checksums are published.

## Maintenance rule

When an epic enters/finishes implementation or dependencies change, update this
document in the same PR that changes epic status.
