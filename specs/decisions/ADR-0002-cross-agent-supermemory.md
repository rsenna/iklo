# ADR-0002 — One shared Supermemory container per project, across all agent tools

- **Status:** Accepted
- **Date:** 2026-07
- **Deciders:** @rsenna (with Claude as implementer)
- **Supersedes:** —
- **Superseded by:** —

## Decision (one sentence)

**Every agent tool that works on Iklo (OpenCode, Claude Code, Codex, GitHub
Copilot) reads from and writes to a single per-project [Supermemory](https://supermemory.ai)
container, canonicalised on OpenCode's tags `opencode_project_<remote-hash>` and
`opencode_user_<email-hash>`, with each tool wired to that container through its
own configuration mechanism.**

Everything else in this ADR is the reasoning, the per-tool wiring, the
alternatives rejected, and the follow-ups it commits us to.

## Vocabulary

- **Container tag** — Supermemory's unit of memory isolation. A search or store
  is scoped to one or more container tags. All of Iklo's cross-agent memory
  lives under two tags: a *project* tag (shared work about this repo) and a
  *user* tag (personal, cross-project preferences).
- **Deterministic hashes** — every integration derives the same two hashes for
  this repo: the **project** hash `845576da904ef1b9` (from the normalised git
  remote `rsenna/iklo`, so it is identical for every contributor) and the
  **user** hash `36176195afe587d6` (from git `user.email`, so it is
  per-contributor). Only the tag *prefix* differs between tools
  (`opencode_`, `codex_`, `claudecode_`, `repo_`).
- **Prefix silo** — the failure mode this ADR removes: because each tool used
  its own prefix, four tools on the same project wrote to four disjoint
  containers and none could see the others.

## Context

Iklo is developed with several agent tools in parallel (see `apm.yml` targets:
claude, codex, copilot, opencode). Each independently grew a Supermemory
integration. When we audited the containers:

- OpenCode had ~22 project memories under `opencode_project_845576da904ef1b9`
  (epic status, parser notes, ADR rationale) — the richest history.
- Claude Code wrote to `repo_iklo__<hash>` and, by a built-in bridge, *read*
  the Codex tags — but not OpenCode's. Its own container held 2 memories.
- Codex's container was empty.
- Copilot had not written yet; its integration is different in kind (see below).

So the tools were **prefix-siloed**: same project, same identity hashes,
mutually invisible memory. That defeats the point of a shared memory — an agent
cannot build on what another agent already learned.

Two structural facts shaped the fix:

1. **The local integrations (OpenCode, Claude, Codex) are config-drivable to an
   explicit container tag.** Claude reads `.claude/.supermemory-claude/config.json`
   (`repoContainerTag`, `personalContainerTag`); Codex reads
   `~/.codex/supermemory.json` (`projectContainerTag`, `userContainerTag`).
   Both can be pointed at an arbitrary tag.
2. **Copilot is not.** Copilot talks to the *remote* Supermemory MCP server
   (`mcp.supermemory.ai`) over OAuth. Its memories default to a single
   account-global bucket `sm_project_default` — not per-project. The MCP can be
   pinned to a container via the `x-sm-project` HTTP header, but the Copilot CLI
   only supports **user-global** MCP config (no per-repo MCP config yet —
   github/copilot-cli#2528, #1291). A global header would force *every* repo
   into Iklo's container. Copilot's per-repo lever is instead its custom
   instructions file, `.github/copilot-instructions.md`; and without the header
   the MCP leaves a `containerTag` argument available on its store/search tools.

## What the decision commits us to

1. **Canonical tags = OpenCode's.** The shared project container is
   `opencode_project_845576da904ef1b9`; the shared user container is
   `opencode_user_36176195afe587d6`. We canonicalise on OpenCode because it held
   the most history and its tags are project-scoped (remote-derived). OpenCode
   needs no change — it is the anchor.
2. **Claude** — `.claude/.supermemory-claude/config.json` sets
   `repoContainerTag` + `personalContainerTag` to the canonical tags. This file
   embeds the per-user hash, so it is **git-ignored**
   (`/.claude/.supermemory-claude/`), not committed.
3. **Codex** — `~/.codex/supermemory.json` sets `projectContainerTag` +
   `userContainerTag` to the canonical tags. (User-global file; not in the repo.)
4. **Copilot** — `.github/copilot-instructions.md` (committed; the project hash
   is remote-derived and identical for all contributors) instructs Copilot to
   pass `containerTag: "opencode_project_845576da904ef1b9"` on every supermemory
   MCP store/search **in this repo**. We deliberately do **not** set a global
   `x-sm-project` header, to preserve per-project separation across the user's
   other repositories.
5. **Migration** is by tag addition, not re-creation: existing stray memories
   (Claude's 2 docs, Copilot's test memory) get the canonical project tag added
   via `PATCH /v3/documents/{id}` so they join the pool without duplicating.

## Alternatives considered

- **A — Converge everything on Copilot's `sm_project_default`.** Rejected:
  that bucket is account-global across *all* repositories, so it cannot keep
  per-project memory separate. Wrong canonical container.
- **B — Read-only bridges (each tool keeps its own write container, only adds
  the others to its read set).** Rejected as the primary design: it gives
  cross-visibility but not a single writable pool, so "where did agent X save
  this" stays fragmented and every new tool multiplies the read list.
- **C — Global `x-sm-project` header for Copilot.** Rejected: airtight but
  pins every repo Copilot touches to Iklo's container — the exact per-project
  separation we require. Revisit if/when the Copilot CLI gains per-repo MCP
  config.

## Consequences

- **Positive:**
  - One writable pool per project; any agent builds on any other's memory.
  - OpenCode's existing history is immediately available to the others.
  - Config-only; no code, no new dependency.
- **Negative:**
  - **Copilot's scoping is advisory** — it depends on the model honouring the
    instruction to pass `containerTag`, not on an enforced header. Less airtight
    than the local integrations.
  - The wiring is spread across four tool-specific config surfaces; onboarding a
    new machine means reproducing three of them (the committed Copilot
    instruction travels with the repo; the others do not).
  - Canonicalising on an `opencode_`-prefixed tag is a historical accident of
    which tool wrote first; the prefix no longer means "OpenCode's".
- **Reversal cost:** low. The tags are just strings in config; re-point them or
  fall back to per-tool defaults at any time. Migrated documents keep their
  original tags (we *added* the canonical tag, never removed one).

## Follow-ups

- Start a fresh Copilot session in this repo so `.github/copilot-instructions.md`
  loads; verify a Copilot store/search lands in the canonical container.
- Replicate steps 2–3 on any other machine used for Iklo.
- When github/copilot-cli ships per-repo MCP config, replace Copilot's advisory
  instruction with an enforced repo-local `x-sm-project` header and note it here.
- ~~Reconcile the `spec/decisions/` vs `specs/decisions/` path referenced in
  AGENTS.md and the constitution~~ — done in this PR: all live docs now point at
  the real `specs/decisions/` directory (`refs/*.old.md` snapshots left as-is).
