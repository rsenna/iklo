# Issue #46 — chore: adopt repo-standard (stage=in-progress)

Source: https://github.com/rsenna/iklo/issues/46

## Tasks

- [x] **T001 — Create `repo.toml` with `stage = "in-progress"`** — ✅ merged (#50)
  Acceptance: `repo.toml` exists at the repo root and contains
  `stage = "in-progress"`; `/repo-standard audit` reports clean (every other
  required `in-progress`-tier path — `AGENTS.md`, `README.md`, `specs/`,
  `specs/decisions/`, `.specify/`, `.specify/memory/constitution.md`,
  `tasks/` — is already present, per the issue's own audit table).
  Verify: `/repo-standard audit` (or manually diff against the
  required-paths table in `repo-standard`'s `SKILL.md`); `make build` /
  `make test` unaffected (config-only change, touches no Rust code).
  Files: `repo.toml` (new)
