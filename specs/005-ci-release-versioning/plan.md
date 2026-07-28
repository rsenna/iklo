# Implementation Plan: CI Release Pipeline and Semantic Versioning

**Branch**: `005-ci-release-versioning` | **Date**: 2026-07-29 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/005-ci-release-versioning/spec.md`

## Summary

Add a GitHub Actions CI workflow that gates every pull request to `main` on
`make build`/`make test`, plus a separate tag-triggered release workflow that
validates the SemVer tag against the workspace version, builds the `iklo`
executable in release mode, generates commit-derived release notes with a
deterministic build identifier, and publishes the binary plus SHA-256
checksums to a GitHub Release. Ships cached-and-pinned from the first PR,
folding in the caching/SHA-pinning/least-privilege asks already scoped in
issue [#32](https://github.com/rsenna/iklo/issues/32) and its task breakdown
(`tasks/issue-32-chore-ci-hardening.md`) rather than as a follow-up hardening
pass.

## Technical Context

**Language/Version**: Rust (workspace edition 2021); workflow YAML for
GitHub Actions.

**Primary Dependencies**:
- `actions/checkout`, `actions/upload-artifact` (or equivalent release-asset
  step), a Rust toolchain setup action (or `mise`, already used locally per
  `mise.toml` — decide during T001/T002 whichever is more reliable in CI).
- `Swatinem/rust-cache` (or `actions/cache` on `~/.cargo`/`target/`) for
  registry/build caching.
- No release-notes SaaS/action dependency: a small repo-local script
  (`.github/scripts/release-notes.sh` or similar) parses
  `previous_tag..current_tag` commit subjects by conventional-commit prefix
  (`feat`, `fix`, `docs`, `chore`, fallback bucket) — keeps FR-006/FR-007
  auditable and dependency-free rather than pulling in a third-party
  changelog generator.
- All actions pinned to reviewed commit SHAs with a trailing version comment,
  per `.github/instructions/github-actions-ci-cd-best-practices.instructions.md`
  (already established repo convention, previously with nothing to apply it
  to — see enrichment on #32).

**Storage**: N/A — no database/persistence surface in this epic; it's pure
CI/CD infrastructure.

**Testing**:
- The CI workflow tests itself: its own first PR is the acceptance test for
  FR-001 (fails on a broken build/test, passes otherwise).
- Release-notes script gets its own `cargo test`-independent unit coverage —
  either a small shell/`bats`-style test or, if written in a way that's
  easier to unit test, a thin Rust binary/xtask instead of raw shell (decide
  in T0xx; default to shell + a couple of scripted fixture cases given the
  small scope).
- `make test` / `make build` remain the underlying gate both workflows call.

**Target Platform**: GitHub-hosted `ubuntu-latest` runners initially (per
spec.md Assumptions: single primary platform binary first, multi-platform
matrix is a later expansion, out of scope here).

**Project Type**: CI/CD infrastructure (no runtime/language code changes
expected, aside from a possible small xtask/script for release notes).

**Performance Goals**:
- CI turnaround fast enough not to discourage frequent pushes — caching
  (Swatinem/rust-cache) is in scope from the first workflow, not a later
  optimization pass.

**Constraints**:
- No paid/external CI services — GitHub Actions only.
- Actions pinned to commit SHAs from day one; mutable tags (`@v4`, `@main`,
  `@latest`) are never acceptable, per repo convention.
- Explicit least-privilege `permissions:` on every workflow/job — no relying
  on the default `GITHUB_TOKEN` scope.
- Canonical version source is `Cargo.toml` `[workspace.package].version`
  (FR-010) — the release workflow must not invent a second source of truth.

**Scale/Scope**:
- Two workflows (`ci.yml`, `release.yml`), one `dependabot.yml`, one small
  release-notes script, plus doc updates (README/AGENTS.md) describing the
  release process.
- Single-binary artifact (the `iklo` executable) per release, one target
  platform, per spec.md Assumptions.

## Constitution Check

Checked against [`.specify/memory/constitution.md`](../../.specify/memory/constitution.md):

- **I. Test-First**: The CI workflow's own correctness is proven by its first
  PR (red on a broken build, green otherwise); the release-notes script gets
  scripted fixture tests before the release workflow calls it.
- **II. One Epic In Flight**: `005-ci-release-versioning` becomes the active
  epic now that epic 004 is Implemented; `specs/execution-queue.md` is
  updated in the same PR as this plan/tasks pair, per its own maintenance
  rule.
- **III. Substrate Before Feature**: N/A — this epic has no runtime/substrate
  surface.
- **IV. Kebab-Case Iklo, Idiomatic Rust**: N/A for workflow YAML; any Rust
  code added (e.g. an xtask) follows idiomatic Rust as usual.
- **V. Comments Justify Themselves**: Workflow YAML gets comments only where
  a step's *why* isn't obvious from its name (e.g. why a step is pinned a
  particular way, why release-notes fallback exists) — not narration of
  what each action does.
- **VI. ADRs for Load-Bearing Decisions**: The release-notes-generation
  approach (repo-local script vs. third-party action) and single-platform
  scope are the two decisions here with real reversal cost; recorded as
  inline `**Decision**` notes below rather than a full ADR, since neither
  changes a cross-crate invariant the way ADR-0001/0005 did for epic 004.
  Promote to an ADR later only if this needs revisiting under pressure.
- **VII. No Workarounds Left Standing**: Any CI flakiness discovered while
  building this (e.g. cache-key issues) gets fixed as part of this epic, not
  left as a "for now" workaround — or filed as its own tracked issue if
  genuinely out of scope.

No constitutional violations planned.

## Project Structure

### Documentation (this feature)

```text
specs/005-ci-release-versioning/
├── spec.md
├── plan.md
└── tasks.md
```

### Source Code (repository root)

```text
.github/
├── workflows/
│   ├── ci.yml               # NEW — PR gate (FR-001)
│   └── release.yml          # NEW — tag-triggered release (FR-002..FR-012)
├── dependabot.yml            # NEW — github-actions ecosystem updates
└── scripts/
    └── release-notes.sh      # NEW — commit-diff notes generation (FR-006/FR-007)
```

**Structure Decision**: Keep all CI/release logic under `.github/` (workflows
+ a small `scripts/` directory for the release-notes generator), matching
GitHub Actions convention and keeping it out of the Rust workspace's own
`crates/` tree — this is tooling, not a language/runtime concern.

## Key Design Decisions

1. **Two separate workflows, not one.** `ci.yml` (pull_request → main) and
   `release.yml` (tag push) have different triggers, different permission
   needs, and different failure semantics (a broken PR shouldn't block a
   release, and vice versa) — keeping them separate avoids conditional logic
   sprawl inside a single file.

2. **Canonical version source.** `Cargo.toml` `[workspace.package].version`
   is the single source of truth (FR-010); `release.yml`'s first real step
   is validating `v<workspace-version> == <tag>` and failing fast on
   mismatch (SC-006), before any build/package work runs.

3. **Build identifier.** `GITHUB_RUN_NUMBER.GITHUB_RUN_ATTEMPT`
   (dot-separated base-10 integers per FR-005) computed once and threaded
   into both the release metadata and artifact filenames — no separate
   counter to maintain.

4. **Release notes: repo-local script, not a third-party action.** A small
   script under `.github/scripts/` computes `previous_tag..current_tag` via
   `git describe --tags --match "v[0-9]*" --abbrev=0 <current_tag>^`
   (FR-011), groups commit subjects by conventional-commit prefix with a
   fallback bucket for unmatched commits (FR-007), and falls back to
   full-history notes when there's no previous tag (FR-009). Keeping this
   in-repo (not a marketplace action) keeps the logic auditable and
   test-covered like any other repo artifact.

5. **CI hardening ships from the first PR, not as a follow-up.** Caching,
   SHA-pinning, least-privilege `permissions:`, and `dependabot.yml` land in
   `ci.yml`'s very first task (Phase 1 below), directly reusing the analysis
   already done for issue #32 rather than re-deriving it — see that issue's
   task breakdown, which explicitly recommended folding into this epic.

6. **Checksums.** SHA-256 checksums for every release artifact (FR-012) are
   generated as a release-workflow step immediately after packaging, using
   `sha256sum`/`shasum -a 256` (whichever the runner provides) — no
   additional dependency.

## Complexity Tracking

None currently. The release-notes script is the one piece of genuinely new
logic (as opposed to wiring existing `make` targets into a workflow); it's
kept intentionally small (single script, conventional-commit-prefix
grouping, two fallback paths) rather than reaching for a changelog framework.
