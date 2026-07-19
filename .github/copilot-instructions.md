# Copilot instructions — Iklo

Read [AGENTS.md](../AGENTS.md) first — it is the source of truth for how this
project works and what is actually implemented. Everything there applies to you.

## Supermemory (cross-agent memory)

This repo shares one Supermemory pool across every agent tool (OpenCode, Claude
Code, Codex, and you). The other tools scope memories to this project with the
container tag `opencode_project_845576da904ef1b9` (derived from the git remote,
so it is the same for every contributor).

Your Supermemory access is the **remote MCP server** (`mcp.supermemory.ai`),
which otherwise defaults to a single global bucket (`sm_project_default`) shared
across *all* your repositories. To keep this project's memories together with
the other agents' — and separate from your other projects — you MUST scope every
call:

- When **storing** a memory via the supermemory MCP tools in this repo, pass
  `containerTag: "opencode_project_845576da904ef1b9"`.
- When **searching / recalling** via the supermemory MCP tools in this repo,
  pass the same `containerTag: "opencode_project_845576da904ef1b9"` so you see
  what the other agents saved.

Do not use this container tag in other repositories — each project has its own.
