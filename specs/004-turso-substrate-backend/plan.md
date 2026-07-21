# Implementation Plan: Turso-backed Substrate Backend

**Branch**: `004-turso-substrate-backend` | **Date**: 2026-07-21 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/004-turso-substrate-backend/spec.md`

## Summary

Implement a new `iklo-substrate-turso` crate that satisfies the existing
`Substrate`/`Transaction` contract while preserving `iklo-runtime` semantics,
keeping implementation adapter-first, and treating Turso fork work as an
escalation path only (per [ADR-0005](../decisions/ADR-0005-turso-fork-governance.md)).

## Technical Context

**Language/Version**: Rust (workspace edition 2021)

**Primary Dependencies**:
- Existing crates: `iklo-substrate`, `iklo-runtime`, `iklo-cli`
- New integration dependency: Turso Rust client (exact crate/API validated in implementation tasks)

**Storage**: Turso-backed SQLite-compatible database for bindings + revision metadata; in-memory substrate remains default.

**Testing**:
- `cargo test -p iklo-substrate`
- `cargo test -p iklo-substrate-turso`
- `cargo test -p iklo-runtime`
- `cargo test -p iklo-cli`
- `make test`

**Target Platform**: Native CLI/runtime environment currently supported by workspace.

**Project Type**: Rust workspace language runtime + CLI.

**Performance Goals**:
- Preserve interactive REPL responsiveness.
- Keep transaction behavior correct before tuning throughput.

**Constraints**:
- No VDBE compiler/opcode work in this epic.
- No Turso fork implementation inside this epic.
- `iklo-runtime` remains semantic reference; behavior regressions are not acceptable.

**Scale/Scope**:
- New crate + targeted runtime/CLI wiring.
- Contract, persistence, and config behavior fully covered by tests.

## Constitution Check

Checked against [`.specify/memory/constitution.md`](../../.specify/memory/constitution.md):

- **I. Test-First**: Contract/persistence/config behavior are test-first tasks.
- **II. One Epic In Flight**: `004-turso-substrate-backend` is the active epic.
- **III. Substrate Before Feature**: Work stays behind `Substrate` boundary.
- **IV. Kebab-Case Iklo, Idiomatic Rust**: Preserved.
- **V. Comments Justify Themselves**: No explanatory noise planned.
- **VI. ADRs for Load-Bearing Decisions**: Fork governance in ADR-0005.
- **VII. No Workarounds Left Standing**: Blockers must be classified and actioned.

No constitutional violations planned.

## Project Structure

### Documentation (this feature)

```text
specs/004-turso-substrate-backend/
├── spec.md
├── plan.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/
├── iklo-substrate/
│   └── src/
│       ├── lib.rs
│       ├── contract.rs
│       └── memory.rs
├── iklo-substrate-turso/          # NEW
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── schema.rs
│       ├── codec.rs
│       └── tests.rs
├── iklo-runtime/
│   └── src/lib.rs
└── iklo-cli/
    └── src/main.rs
```

**Structure Decision**: Add a dedicated `iklo-substrate-turso` crate to keep
backend concerns isolated and avoid dependency cycles with `iklo-substrate`.

## Key Design Decisions

1. **Adapter-first integration**
   - Build Turso backend through exposed APIs only.
   - Fork-required outcomes are tracked, not executed, in this epic.

2. **Canonical blocker inventory**
   - `tasks.md` owns blocker inventory with mandatory schema:
     `ID`, `classification`, `invariant impacted`, `evidence`,
     `chosen action`, `rationale`.

3. **Persistence contract before breadth**
   - Start with currently required persisted-`V` shapes.
   - Version the codec/schema up front and define mismatch behavior.

4. **Explicit CLI config semantics**
   - `--substrate` mode selection + flag/env precedence + invalid-combo errors
     are defined and tested.

5. **Concurrency behavior policy**
   - Define retryable vs non-retryable failures, retry bounds/backoff, and
     immediate-surface error classes.

## Complexity Tracking

None currently.
