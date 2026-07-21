# Feature Specification: CI Release Pipeline and Semantic Versioning

**Feature Branch**: `005-ci-release-versioning`

**Created**: 2026-07-21

**Status**: Draft

**Input**: Add a basic GitHub Actions pipeline that validates Iklo and publishes the REPL CLI binary as a GitHub Release asset, while introducing explicit semantic versioning and release notes generated from commit history between releases.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Maintainer gets reliable CI feedback on every change (Priority: P1)

A maintainer opens a PR and gets an automated pass/fail signal from GitHub Actions for build and tests before merging.

**Why this priority**: Release automation is unsafe without a stable quality gate baseline.

**Independent Test**: Open a PR that touches Rust code and verify workflow runs `make test` and `make build`, failing on regressions.

**Acceptance Scenarios**:

1. **Given** a pull request to `main`, **When** the CI workflow runs, **Then** it executes the documented quality gate and reports status in the PR checks.
2. **Given** a failing test/build, **When** the workflow completes, **Then** the PR check is failed and no release step is executed.

---

### User Story 2 - Maintainer can publish an Iklo CLI release artifact (Priority: P1)

A maintainer creates a release tag and receives a GitHub Release containing the `iklo-cli` binary artifact produced by CI.

**Why this priority**: This is the direct delivery mechanism users consume.

**Independent Test**: Push a SemVer tag and verify a GitHub Release is created with the packaged CLI binary attached.

**Acceptance Scenarios**:

1. **Given** a valid release tag, **When** the release workflow runs, **Then** it builds `iklo-cli` in release mode and uploads binary assets to the GitHub Release.
2. **Given** a release workflow failure during build/package, **When** the workflow completes, **Then** no partial/invalid release is published.

---

### User Story 3 - Team has deterministic versioning and useful release notes (Priority: P1)

Each release follows SemVer and includes an incrementing build number plus a clear list of implemented changes derived from commit history between the previous and current release tags.

**Why this priority**: Without predictable version semantics and changelog quality, releases are hard to trust and consume.

**Independent Test**: Publish two consecutive releases and verify version progression and generated notes reflect `previous_tag..current_tag` commits.

**Acceptance Scenarios**:

1. **Given** two sequential releases, **When** the second is published, **Then** release metadata includes a strictly increasing build number.
2. **Given** a release tag, **When** notes are generated, **Then** the release notes include a commit-derived change list covering only commits since the previous release tag.
3. **Given** a first-ever release with no previous tag, **When** notes are generated, **Then** the workflow falls back to repository-history-based notes without failing.

### Edge Cases

- Invalid or non-SemVer tags must fail fast with a clear workflow error.
- Re-publishing an existing tag/version must be rejected to prevent ambiguous artifacts.
- If commits lack conventional prefixes, notes generation must still include them under a generic section.
- If artifact upload fails, release publication must stop and report the error.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A CI workflow MUST run on pull requests targeting `main` and execute at least `make test` and `make build`.
- **FR-002**: A release workflow MUST run on SemVer tag pushes and build the `iklo-cli` binary in release mode.
- **FR-003**: The release workflow MUST publish `iklo-cli` binary assets to GitHub Releases.
- **FR-004**: Release versioning MUST follow Semantic Versioning (`MAJOR.MINOR.PATCH`) with tags in `vMAJOR.MINOR.PATCH` format.
- **FR-005**: Every CI/release run MUST produce an incrementing build number, and that number MUST be attached to release metadata and/or artifact naming.
- **FR-006**: Release notes MUST be generated from commit history diff (`previous_release_tag..current_release_tag`) and included in the GitHub Release body.
- **FR-007**: Generated release notes MUST provide a human-readable “implemented changes” list grouped by commit intent where possible (e.g., `feat`, `fix`, `docs`, `chore`), with fallback grouping for unmatched commits.
- **FR-008**: The workflow MUST fail and avoid publishing when tag format, build, tests, packaging, or note generation steps fail.
- **FR-009**: The first release (no previous tag) MUST be supported with a deterministic fallback note-generation strategy.
- **FR-010**: The repository MUST define and document where the canonical version is sourced (tag, Cargo workspace version, or both) and how mismatches are handled.

### Key Entities

- **Release Tag**: Git tag in SemVer form (`vMAJOR.MINOR.PATCH`) that triggers release automation.
- **Build Number**: Monotonic CI build identifier (e.g., GitHub run number) attached to artifacts/release metadata.
- **Release Artifact**: Packaged `iklo-cli` binary produced by release workflow.
- **Release Notes Model**: Structured changelog content derived from commit history between release tags.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Pull requests to `main` show CI status from GitHub Actions with test/build checks.
- **SC-002**: A SemVer tag publish produces a GitHub Release with at least one downloadable `iklo-cli` binary artifact.
- **SC-003**: Two consecutive release runs show a strictly increasing build number in release metadata/artifacts.
- **SC-004**: Release notes include all commits in `previous_tag..current_tag` and exclude older commits.
- **SC-005**: Invalid tag format and duplicate release-tag attempts fail without publishing artifacts.

## Assumptions

- Initial release scope can target one primary platform binary first; multi-platform matrix can follow in later expansion.
- Commit history is available in CI with sufficient depth to compare against previous tags.
- Existing Makefile targets remain the quality gate baseline.
- This epic introduces release automation only; package manager distribution channels (Homebrew, apt, etc.) are out of scope.
