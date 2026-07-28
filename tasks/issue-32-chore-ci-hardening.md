# Tasks: #32 — chore: CI hardening

**Source issue**: https://github.com/rsenna/iklo/issues/32
**Enrichment**: posted as a comment on #32 (see there for full analysis).

**Prerequisite (blocking all tasks below)**: this repo has no
`.github/workflows/` yet. T001-T004 below are drafted as the *first*
CI-workflow task epic 005 (`specs/005-ci-release-versioning/spec.md`, FR-001)
would need anyway — the intent is for them to become that epic's initial
task(s) once it gets a real `plan.md`/`tasks.md`, not to run as a second,
independent CI-creation effort in parallel with it. See the enrichment
comment's Open Questions before starting: epic 005 is technically unblocked
(epic 004 left Draft in PR #31) but `specs/execution-queue.md` records a
maintainer preference to defer it until epics 007/010 stabilize.
**Do not start T001 without resolving that first.**

## Format

`[ID] 🔒?` then Acceptance / Verify / Files. 🔒 = ask-first gate (new CI
workflow / secrets-adjacent config) per this skill's boundaries.

**Verify** below refers to the project's quality gate: `cargo fmt --check`,
`cargo build --workspace` and `--workspace --features iklo-cli/turso`,
`cargo test --workspace` and `--workspace --features iklo-cli/turso`, and
`cargo clippy --workspace --features iklo-cli/turso` — see
`.claude/skills/quality-gate/SKILL.md` if present in the checkout, or run
the commands directly.

---

- [ ] **T001** 🔒 Create a minimal `.github/workflows/ci.yml` triggered on
  pull requests targeting `main`, running `make build` and `make test`
  (epic 005 FR-001 baseline), with explicit least-privilege `permissions:`
  for `GITHUB_TOKEN` declared at the workflow or job level from the start
  (per this repo's own
  `.github/instructions/github-actions-ci-cd-best-practices.instructions.md`).
  SHA-pinning (T002) and caching (T003) land in later tasks, not this same
  PR, to keep each PR to one concern.
  **Acceptance**: opening *any* pull request targeting `main` (not just one
  touching Rust code) shows a CI check that fails on a broken build/test and
  passes otherwise; the workflow declares explicit, minimal `permissions:`
  rather than relying on the default token scope.
  **Verify**: the project quality gate, plus the new workflow itself passing
  on its own PR.
  **Files**: `.github/workflows/ci.yml` (new)

- [ ] **T002** 🔒 [depends: T001] Pin every action reference in `ci.yml`
  (`actions/checkout`, any setup-toolchain action) to a reviewed commit SHA,
  with the version kept as a trailing comment — the exact pattern already
  documented in
  `.github/instructions/github-actions-ci-cd-best-practices.instructions.md`
  (confirmed present in this repo).
  **Acceptance**: no action reference uses a mutable tag (`@v4`, `@main`,
  `@latest`); each has `@<sha> # vX.Y.Z`.
  **Verify**: manual review of the `ci.yml` diff against the instructions
  file's convention.
  **Files**: `.github/workflows/ci.yml`

- [ ] **T003** 🔒 [depends: T001, T002] Add `Swatinem/rust-cache` (or
  `actions/cache` on `~/.cargo` and `target/`) between checkout and the
  build/test steps in `ci.yml`, pinned per T002's convention (the cache
  action is itself an action reference, so it can't be added correctly
  before T002's pinning convention is established).
  **Acceptance**: the cache step is correctly configured (correct paths,
  a cache key derived from `Cargo.lock`) and shows a restore/hit on a run
  with a matching prior cache entry. A cold run with no matching entry yet
  is expected to miss and must still succeed — a guaranteed hit on every
  second run is not required, since cache availability legitimately varies
  with key changes, eviction, and runner scope.
  **Verify**: inspect the Actions run log for the cache step's
  configuration and (when applicable) a hit; local quality gate unaffected.
  **Files**: `.github/workflows/ci.yml`

- [ ] **T004** [depends: T002, T003] Add `.github/dependabot.yml` with a
  `github-actions` package-ecosystem entry so pinned SHAs (both the
  toolchain/checkout actions from T002 and the cache action from T003) get
  automated update PRs.
  **Acceptance**: GitHub's config-validation check passes on the PR adding
  this file; a subsequent Dependabot run is able to open an update PR for a
  pinned action.
  **Verify**: GitHub's own Dependabot config validation; no Rust changes, so
  the quality gate is unaffected.
  **Files**: `.github/dependabot.yml` (new)

## Dependency order

T001 → T002 → T003 → T004 (fully serial: T003 needs T002's pinning
convention already established to pin the cache action itself, and T004
needs every action from both T002 and T003 present to cover them all).

## Milestone note

If epic 005 is picked up as its own effort with a real `plan.md`/`tasks.md`,
merge T001-T004 into that epic's task list rather than running them as a
separate issue-#32 PR stream — avoids two independent CI-workflow creation
efforts colliding.
