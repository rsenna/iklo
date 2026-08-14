---
description: "Task list for CI release pipeline and semantic versioning"
status: draft
---

# Tasks: CI Release Pipeline and Semantic Versioning

**Input**: Design documents from `/specs/005-ci-release-versioning/`

**Prerequisites**: [spec.md](spec.md) and [plan.md](plan.md)

**Tests**: Required (Constitution I). The CI workflow's own first PR is its
red→green proof (fails on a broken build/test, passes otherwise); the
release-notes script gets scripted fixture tests before anything depends on
it.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel
- **[Story]**: US1, US2, or US3 (spec.md's three user stories)

## Path Conventions

- Workflow/CI config lives under `.github/` (workflows, `dependabot.yml`,
  `scripts/`) — not inside the Rust workspace's `crates/` tree.

## Blocker Inventory (Canonical Tracker)

Use this table for FR-008/FR-009 tracking during implementation, same schema
as epic 004's.

| Blocker ID | Classification | Invariant Impacted | Evidence | Chosen Action | Rationale |
|---|---|---|---|---|---|
| _example_ | adapter-fixable | rollback visibility | failing test name/log | adjust adapter transaction wrapping | API supports needed primitive |

None recorded yet — this epic has no known blockers going in (no external
service dependency beyond GitHub Actions itself).

Allowed `Classification` values:
- `adapter-fixable`
- `upstream-fixable`
- `fork-required`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Stand up `.github/workflows/ci.yml` cached-and-pinned from the
start — folding in issue [#32](https://github.com/rsenna/iklo/issues/32)'s
task breakdown directly rather than as a follow-up hardening pass (per that
issue's own Milestone note).

- [x] **T001** [US1] Create `.github/workflows/ci.yml` triggered on pull
  requests targeting `main`, with explicit least-privilege `permissions:`
  declared at the workflow or job level, and every action reference pinned
  to a reviewed commit SHA with a trailing version comment from the start
  (folds in issue #32's T001/T002).
  **Done 2026-07-28**: combined with T004 in the same PR — a workflow with
  no build/test steps isn't independently meaningful, so they shipped
  together. Pinned actions:
  `actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1` and
  `dtolnay/rust-toolchain@e97e2d8cc328f1b50210efc529dca0028893a2d9 # v1`.
  `permissions: contents: read` set at workflow level.
- [x] **T002** [US1] Add `Swatinem/rust-cache` (or `actions/cache` on
  `~/.cargo`/`target/`), SHA-pinned, between checkout and build/test steps
  in `ci.yml` (folds in issue #32's T003).
  **Done 2026-07-30**: added `Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4 # v2.9.1`
  after the toolchain-install step and before `make build`.
- [x] **T003** [US1] Add `.github/dependabot.yml` with a `github-actions`
  package-ecosystem entry (folds in issue #32's T004).
  **Done 2026-07-30**: weekly `github-actions` update schedule for the
  repo root.
- [x] **T004** [US1] Wire `ci.yml` to run `make build` and `make test`,
  failing the check on either failing (FR-001).
  **Done 2026-07-28**: shipped together with T001 (see above).

**Checkpoint**: Opening this phase's own PR is the acceptance test — CI must
show a check, fail it if `make build`/`make test` fail, pass otherwise.

---

## Phase 2: Foundational (Blocking Prerequisites for US2/US3)

**Purpose**: Version/tag validation and build-identifier computation are
shared by both release publishing (US2) and versioning/notes (US3).

**CRITICAL**: No US2/US3 task starts before this phase is complete.

- [x] **T005** [Test-first, per Constitution I] Write a fixture-based test
  for version/tag validation *before* implementing it: asserts a
  `v<tag>`/`Cargo.toml` mismatch fails with an error naming both values
  (SC-006), and that re-publishing an existing tag is rejected. Then
  implement: canonical version source is `Cargo.toml`
  `[workspace.package].version` (FR-010); release tag MUST equal
  `v<workspace-version>`; mismatch fails before any build/publish step.
  Also implements `previous_release_tag` selection (FR-011): nearest
  reachable prior SemVer tag via `git describe --tags --match "v[0-9]*"
  --abbrev=0 <current_tag>^`.
  **Done 2026-07-30**: `.github/scripts/validate-release-tag.sh` (tag
  format + version match, SC-006 error message) and
  `.github/scripts/previous-release-tag.sh` (FR-011), each with fixture
  tests under `.github/scripts/tests/` run against a throwaway git repo
  fixture (not the real repo's tags). Re-publishing-an-existing-tag
  rejection is **not** in these scripts — it needs to check the real
  GitHub Release API (does a release already exist for this tag?), which
  is T012's concern in `release.yml`, not something these offline,
  git-only scripts can determine.
- [x] **T006** Implement build-identifier computation: deterministic
  `GITHUB_RUN_NUMBER.GITHUB_RUN_ATTEMPT` (FR-005), evaluated numerically as
  `(GITHUB_RUN_NUMBER, GITHUB_RUN_ATTEMPT)` for "strictly increasing" checks
  (SC-003).
  **Done 2026-07-30**: `.github/scripts/build-identifier.sh`, with fixture
  tests under `.github/scripts/tests/test-build-identifier.sh` covering
  missing/non-numeric env vars, not just the happy path.

**Checkpoint**: Version/tag validation and build-identifier logic available
for both release publishing and notes generation.

---

## Phase 3: User Story 1 - Maintainer gets reliable CI feedback (P1)

**Goal**: A PR touching Rust code shows CI status; broken build/test fails
the check.

**Independent Test**: Open a PR that touches Rust code and verify the
workflow runs `make test` and `make build`, failing on regressions.

- [x] **T007** [US1] Verify the acceptance scenario end-to-end: confirm a PR
  with an intentionally broken build/test (verified locally, not actually
  pushed broken) would fail the check per T001-T004's wiring, and that a
  clean PR passes. Record the verification method used (e.g. a scratch
  branch, or reasoning from the workflow YAML directly) since this is
  largely already covered by T001-T004's own acceptance criteria.
  **Done 2026-08-13**: two-part verification, no throwaway PR needed.
  (1) YAML reasoning: in `.github/workflows/ci.yml`'s `build-and-test` job,
  the `Build (make build)` and `Test (make test)` steps are plain `run:`
  shell steps with no `continue-on-error` anywhere in the workflow —
  GitHub Actions' default behavior fails the step (and the job/check) on
  any nonzero exit. (2) Empirical: locally broke a runtime test's
  assertion (`crates/iklo-runtime/src/lib.rs`'s
  `rollback_keeps_image_unchanged`), ran `make test`, confirmed
  `make: *** [test] Error 101`; reverted, confirmed `make build` and
  `make test` both exit 0 clean.

**Checkpoint**: US1 fully satisfied by Phase 1 + this verification task.

---

## Phase 4: User Story 2 - Maintainer can publish a release artifact (P1)

**Goal**: A SemVer tag push produces a GitHub Release with the `iklo`
executable and checksums attached.

**Independent Test**: Push a SemVer tag and verify a GitHub Release is
created with the packaged CLI binary attached.

### Implementation for User Story 2

- [x] **T008** [US2] Create `.github/workflows/release.yml` triggered on
  SemVer tag pushes (`v[0-9]*`), least-privilege `permissions:` including
  `contents: write` for release creation, actions pinned per T001's
  convention. **Checkout step MUST use `fetch-depth: 0`** (full history) —
  unlike `ci.yml`'s shallow default, T014's release-notes script needs full
  tag/commit history to run `git describe` against previous tags. Calls
  T005's version/tag validation as its first real step — fails fast before
  any build/package work on mismatch (FR-002, FR-004, FR-010, SC-006).
  **Done 2026-08-13**: `.github/workflows/release.yml` — triggers on
  `v[0-9]*` tag pushes, `contents: read` (tightened from an initial
  `contents: write` after cubic-dev-ai/sourcery review feedback — this
  phase writes nothing; `write` is deferred to T012, which actually
  creates the Release), pinned `actions/checkout`
  matching `ci.yml`'s SHA, `fetch-depth: 0` + `persist-credentials: false`.
  Its only step beyond checkout calls `validate-release-tag.sh` against
  `$GITHUB_REF_NAME` (a plain env-var expansion inside `run:`, not a
  `${{ }}` template interpolation, to avoid the tag-name shell-injection
  footgun that pattern has). Verified: YAML parses (Ruby's `Psych`, no
  `pyyaml` available locally); `validate-release-tag.sh` itself already has
  fixture tests (T005) and was re-run manually against this repo's real
  `Cargo.toml` version for both a matching and a mismatched tag. Build,
  packaging, checksums, and the atomic release-creation step are separate
  tasks (T009-T012) — this workflow does nothing on a real tag push yet
  beyond validating it.
- [x] **T009** [US2] Build the `iklo` executable in release mode
  (`cargo build --release -p iklo-cli`) only after `make test` passes
  (FR-002).
  **Done 2026-08-13**: added toolchain install + `Swatinem/rust-cache`
  (same pinned SHAs as `ci.yml`, confirmed byte-identical) to
  `release.yml`, then `make test`, then `cargo build --release -p
  iklo-cli --locked` as the final step. `--locked` added after
  self-review: a release artifact should fail loud on `Cargo.lock` drift
  rather than silently building against unrecorded dependency versions.
  No `make build` (debug) step — T009's scope is only the release build,
  gated on tests passing, not a redundant debug build first. Verified
  locally: `make test` exits 0, `cargo build --release -p iklo-cli
  --locked` exits 0 and produces `target/release/iklo`. Also updated the
  workflow's least-privilege header comment, which still described the
  T008-only state before this task added test/build steps.
  Follow-up fix
  after cubic-dev-ai review: the test step now runs `cargo test --locked`
  directly rather than `make test`'s plain `cargo test` -- an unlocked
  test run silently rewrites a drifted `Cargo.lock`, which would have
  defeated the release build's own `--locked` guarantee by the time it
  ran. Scoped to `release.yml` only, not `Makefile`/`ci.yml` (those are a
  separate concern about local-dev/PR-check strictness, not this task).
- [x] **T010** [US2] Package the built executable for release (FR-003,
  SC-002). Actual upload to a GitHub Release asset happens in T012's
  atomic release-creation call, per plan.md's Key Design Decision #7 --
  this task only stages the named artifact.
  **Done 2026-08-13**: `release.yml` copies `target/release/iklo` to
  `dist/iklo-${GITHUB_REF_NAME}-x86_64-unknown-linux-gnu` -- single
  platform for now, per spec.md's Assumptions. Per plan.md's Key Design
  Decision #7, this step only *stages* the named artifact; it does not
  create or upload to a GitHub Release -- the Release itself is created
  atomically in T012, after checksums (T011) and release notes (T014)
  both succeed too. Verified locally: built the release binary, ran the
  exact packaging commands from the workflow (with
  `GITHUB_REF_NAME=v0.1.0` standing in for the real tag-push env var),
  confirmed the staged file is present, executable, and correctly named.
- [x] **T011** [US2] Generate SHA-256 checksums for every release
  artifact (FR-012, SC-007). Actual publishing (upload to a GitHub
  Release asset) happens in T012's atomic release-creation call, per
  plan.md's Key Design Decision #7 -- same staging-now/publish-in-T012
  split as T010. Include a test verifying each generated checksum file
  actually matches its artifact's content (e.g. `sha256sum -c` against
  the built binary), not just that a checksum file exists.
  **Done 2026-08-13**: `.github/scripts/generate-checksums.sh` --
  `sha256sum` if present, `shasum -a 256` fallback (macOS has no
  `sha256sum` by default, detected once up front, not per-file), no
  additional dependency, per Key Design Decision #6. Writes
  `<file>.sha256` referencing the artifact by basename (not the build
  machine's absolute path -- release assets download flat, side-by-side,
  so an absolute-path checksum file would fail verification for every
  downloader). Test-first (Constitution I):
  `.github/scripts/tests/test-generate-checksums.sh` written and
  confirmed red (script didn't exist) before implementing; green after
  (14/14 assertions) -- happy path, checksum-references-basename-not-path,
  a genuine `sha256sum -c` verification against real content, a negative
  case (tampered artifact must fail verification), multiple artifacts in
  ONE invocation with each one's checksum individually content-verified
  AND each checksum file's recorded filename field individually asserted
  correct (not just that `-c` passes and not just file existence --
  catches the for-loop attributing the wrong checksum to the wrong
  file), a dash-prefixed-basename regression case, missing-argument and
  nonexistent-file failure cases.
  **Fix after cubic-dev-ai review**: a basename starting with `-` (e.g.
  `-artifact`) was parsed as an option by `sha256sum`/`shasum` instead of
  a filename -- added `--` before the filename in both the script and the
  test helper's own verification call (the test helper had the identical
  bug, caught only once the dash-prefixed regression case was added).
  **Fix after coderabbitai review**: `sha256sum -c` only confirms the
  hash matches *some* file with the recorded name, not that the name is
  the one actually expected -- a `checksum_records_basename` helper now
  cross-checks each `.sha256` file's recorded filename field directly,
  so a bug that attributed one artifact's record to another's file (with
  content that happened to still verify) would be caught. Also replaced
  every `$([ -f ... ]; echo $?)` command-substitution existence check
  with an explicit `check_file_exists` helper for readability/consistency
  with the rest of the suite's if/else style.
  Wired into `release.yml` immediately after T010's packaging step.
  Verified end-to-end locally against the real release binary: built,
  packaged, checksummed, then `shasum -a 256 -c` against the generated
  file reported `OK`.
- [ ] **T012** [US2] Ensure any failure — tag format, build, tests,
  packaging, checksum, or note generation — stops the workflow and avoids
  publishing a partial/invalid release (FR-008); reject re-publishing an
  existing tag/version (edge case in spec.md). The GitHub Release object
  itself MUST be created only after T009-T011 and T014 (build, checksums,
  release notes) all succeed, in one call that supplies every asset and
  the notes body at once (plan.md Key Design Decision #7) — a failure in
  any earlier step must leave no Release object at all, not a Release
  missing an asset or with empty notes.

**Checkpoint**: US2 independently testable — a real tag push produces a
complete, checksummed release or a clean failure, never a partial one.

---

## Phase 5: User Story 3 - Deterministic versioning and release notes (P1)

**Goal**: Releases carry a strictly increasing build identifier and
commit-derived, human-readable release notes.

**Independent Test**: Publish two consecutive releases and verify version
progression and generated notes reflect `previous_tag..current_tag` commits.

### Tests for User Story 3 (write first)

- [ ] **T013** [US3] Write fixture-based tests for the release-notes script
  covering: conventional-commit-prefix grouping (`feat`/`fix`/`docs`/`chore`),
  a fallback bucket for unmatched commit subjects (FR-007), and the
  first-release fallback path when no previous tag exists (FR-009).

### Implementation for User Story 3

- [ ] **T014** [US3] Implement `.github/scripts/release-notes.sh` (or
  equivalent): computes `previous_release_tag..current_release_tag` (via
  T005's tag-selection logic), groups commits by conventional-commit intent
  with a fallback bucket, and produces the release body text (FR-006,
  FR-007, FR-009).
- [ ] **T015** [US3] Wire T006's build identifier into release metadata and
  artifact naming (FR-005).
- [ ] **T016** [US3] Verify SC-003/SC-004 end-to-end: two consecutive
  release runs show a strictly increasing build identifier, and each
  release's notes include exactly the commits since the previous tag (no
  older commits, no gaps).

**Checkpoint**: US3 independently testable — release notes and build
identifiers are correct and reproducible across consecutive releases.

---

## Phase 6: Governance, Documentation, and Final Gate

**Purpose**: Close readiness gaps, keep docs/traceability accurate, resolve
issue #32's T000 decision as fulfilled.

- [ ] **T017** Validate and maintain the FR→Task traceability table below.
- [ ] **T018** Update `README.md`/`AGENTS.md` describing the release process,
  SemVer/tag conventions, and how to cut a release (FR-010 consumer-facing
  half).
- [ ] **T019** Confirm `specs/execution-queue.md`'s epic 005 entry reflects
  activation (done as part of the same PR that adds this plan/tasks pair,
  per that document's own maintenance rule) and that issue #32's T000
  ("resolve whether epic 005 starts now") is marked resolved with a go
  decision, referencing this plan/tasks pair as the exit criterion.
- [ ] **T020** Run final gate: `cargo test --workspace [--all-features]`,
  `make test`, `make build`, `make release` (per the `quality-gate` skill),
  plus a live verification of both workflows (a real test PR for `ci.yml`;
  a real or dry-run tag for `release.yml` if feasible without polluting the
  release history — otherwise document the manual verification performed).

---

## Dependencies & Execution Order

### Phase Dependencies

- Phase 1 (Setup) → Phase 2 (Foundational) → (Phases 3, 4, 5) → Phase 6
- US2 and US3 both depend on Phase 2's version/tag validation and build-id
  logic; US1 only depends on Phase 1.

### Story Dependencies

- **US1**: starts after Phase 1 (independent of Phase 2)
- **US2**: starts after Phase 2
- **US3**: starts after Phase 2; T014 (release-notes script) can proceed in
  parallel with US2's T008-T012 once Phase 2 is done, since they touch
  different files (`release-notes.sh` vs `release.yml`)

### Parallel Opportunities

- T002/T003 (cache, dependabot) can proceed in parallel once T001 exists.
- T013/T014 (release-notes tests/script) can proceed in parallel with
  Phase 4's T008-T012 once Phase 2 is complete.

---

## FR to Task Traceability

| FR | Primary Tasks |
|---|---|
| FR-001 | T001, T004, T007 |
| FR-002 | T008, T009, T012 |
| FR-003 | T010, T012 |
| FR-004 | T008 |
| FR-005 | T006, T015, T016 |
| FR-006 | T013, T014 |
| FR-007 | T013, T014 |
| FR-008 | T012 |
| FR-009 | T013, T014 |
| FR-010 | T005, T008 |
| FR-011 | T005, T014 |
| FR-012 | T011 |
