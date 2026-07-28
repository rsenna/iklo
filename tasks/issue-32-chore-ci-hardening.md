# Tasks: #32 — chore: CI hardening

**Source issue**: https://github.com/rsenna/iklo/issues/32
**Enrichment**: posted as a comment on #32 (see there for full analysis).

**Prerequisite (blocking all tasks below)**: this repo has no
`.github/workflows/` yet. These tasks assume the first CI workflow — epic
005's FR-001 (`specs/005-ci-release-versioning/spec.md`) — is being created
*as part of this same work*, cached-and-pinned from the start, rather than
as a follow-up hardening pass on an already-shipped workflow. See the
enrichment comment's Open Questions before starting: epic 005 is technically
unblocked (epic 004 left Draft in PR #31) but `specs/execution-queue.md`
records a maintainer preference to defer it until epics 007/010 stabilize.
**Do not start T001 without resolving that first.**

## Format

`[ID] 🔒?` then Acceptance / Verify / Files. 🔒 = ask-first gate (new CI
workflow / secrets-adjacent config) per this skill's boundaries.

---

- [ ] **T001** 🔒 Create a minimal `.github/workflows/ci.yml` triggered on
  pull requests targeting `main`, running `make build` and `make test`
  (epic 005 FR-001 baseline). SHA-pinning (T002) and caching (T003) land in
  this same PR, not as a later pass.
  **Acceptance**: opening a PR that touches Rust code shows a CI check that
  fails on a broken build/test and passes otherwise.
  **Verify**: local quality gate (`.claude/skills/quality-gate/SKILL.md`)
  green, plus the new workflow itself passing on its own PR.
  **Files**: `.github/workflows/ci.yml` (new)

- [ ] **T002** 🔒 [depends: T001] Pin every action reference in `ci.yml`
  (`actions/checkout`, any setup-toolchain action, cache action) to a
  reviewed commit SHA, with the version kept as a trailing comment — the
  exact pattern already documented in
  `.github/instructions/github-actions-ci-cd-best-practices.instructions.md`.
  **Acceptance**: no action reference uses a mutable tag (`@v4`, `@main`,
  `@latest`); each has `@<sha> # vX.Y.Z`.
  **Verify**: manual review of the `ci.yml` diff against the instructions
  file's convention.
  **Files**: `.github/workflows/ci.yml`

- [ ] **T003** 🔒 [depends: T001] Add `Swatinem/rust-cache` (or
  `actions/cache` on `~/.cargo` and `target/`) between checkout and the
  build/test steps in `ci.yml`, SHA-pinned per T002.
  **Acceptance**: a second CI run on the same branch shows a cache
  restore/hit in the Actions log instead of a full cold registry download.
  **Verify**: inspect the Actions run log for the cache step; local quality
  gate unaffected.
  **Files**: `.github/workflows/ci.yml`

- [ ] **T004** [depends: T002] Add `.github/dependabot.yml` with a
  `github-actions` package-ecosystem entry so pinned SHAs get automated
  update PRs.
  **Acceptance**: GitHub's config-validation check passes on the PR adding
  this file; a subsequent Dependabot run is able to open an update PR for a
  pinned action.
  **Verify**: GitHub's own dependabot config validation; no Rust changes, so
  the quality gate is unaffected.
  **Files**: `.github/dependabot.yml` (new)

## Dependency order

T001 → (T002, T003 in parallel) → T004

## Milestone note

If epic 005 is picked up as its own effort with a real `plan.md`/`tasks.md`,
merge T001-T004 into that epic's task list rather than running them as a
separate issue-#32 PR stream — avoids two independent CI-workflow creation
efforts colliding.
