---
name: quality-gate
description: Run Iklo's full Cargo quality gate (build, tests in both feature configs, Clippy) and report a single pass/fail summary. Use before pushing any Rust change, before opening a PR via the pull-request-process skill, or whenever asked to "run the gate" / "check everything's green".
---

# Quality Gate

Iklo's real quality gate is wider than `AGENTS.md`'s `make build`/`make test`
shorthand: the `turso` feature on `iklo-cli`/`iklo-substrate-turso` is
**not** default, so a change can pass with default features and still be
broken under `--features iklo-cli/turso` (or vice versa). Every command
below must be run — do not stop at the first one that passes.

## Steps

Run each command from the repo root (or the relevant worktree root). Do not
skip a step because an earlier one already failed *in a different feature
configuration* — steps 2/4 (default features) and 3/5/6 (the `turso`
feature) are independent, and a failure in one tells you nothing about the
other. (Within the *same* configuration, an earlier failure does predict
later ones — e.g. a default-features build failure guarantees the
default-features test step fails too — but run it anyway for a complete,
unambiguous report rather than stopping early.)

1. **Format check** (fast, run first):
   ```bash
   cargo fmt --check
   ```
   If this fails, run `cargo fmt` to fix it, then re-run this step to
   confirm — do not hand-edit formatting.

2. **Build, default features:**
   ```bash
   cargo build --workspace
   ```

3. **Build, turso feature:**
   ```bash
   cargo build --workspace --features iklo-cli/turso
   ```

4. **Test, default features:**
   ```bash
   cargo test --workspace
   ```

5. **Test, turso feature:**
   ```bash
   cargo test --workspace --features iklo-cli/turso
   ```

6. **Clippy, turso feature** (broadest lint surface — covers the `turso`-gated
   code default Clippy would skip, and `--all-targets` covers tests,
   examples, and benches too, not just lib/bin targets):
   ```bash
   cargo clippy --workspace --all-targets --features iklo-cli/turso
   ```
   A Clippy *warning* that already existed before your change (check `git
   stash` / `git diff` if unsure) is not a gate failure — only new warnings
   introduced by the current change are. Don't fix unrelated pre-existing
   warnings as a drive-by unless asked.

## Reporting

After all six steps, report one of:

- **Gate green**: one line confirming all six passed, e.g. "Quality gate
  green: fmt, build×2, test×2 (44 passed), Clippy (no new warnings)."
- **Gate red**: name exactly which step(s) failed and the first real error
  from each (not the full compiler/test output) — enough for the next
  action to be obvious, not a log dump.

Do not proceed to push (`pr.sh push`) or claim work is "done" while any step
is red.

## When a step is slow or you're iterating

If you're mid-edit and want a fast sanity check rather than the full gate,
`cargo check --workspace --features iklo-cli/turso` is a reasonable
in-progress substitute for steps 2-3 — but the full gate (all six steps)
must still run before push, per the `pull-request-process` skill's rule to
never push unless the project's quality gate is green.
