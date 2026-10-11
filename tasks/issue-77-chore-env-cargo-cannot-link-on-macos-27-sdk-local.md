# Tasks: #77 — chore(env): cargo cannot link on macOS 27 SDK — local quality gate unusable

**Source issue**: https://github.com/rsenna/iklo/issues/77
**Enrichment**: posted as a comment on #77 (see there for full analysis).

## Format

`[ID] 🔒?` then Acceptance / Verify / Files. 🔒 = ask-first gate (toolchain /
environment changes, or a decision reserved for the maintainer).

**Verify** below refers to the project's quality gate: `cargo fmt --check`,
`cargo build --workspace` and `--workspace --features iklo-cli/turso`,
`cargo test --workspace` and `--workspace --features iklo-cli/turso`, and
`cargo clippy --workspace --all-targets --features iklo-cli/turso` — see
`.claude/skills/quality-gate/SKILL.md` in the checkout, or run the commands
directly.

## Root cause (verified, so tasks don't re-derive it)

Cargo uses `cc` from `PATH` as its **link driver**. On this machine `cc`
resolves through the mise shim (`~/.local/share/mise/shims/cc`)
to **Homebrew clang 23**, which cannot parse the installed **Xcode 27.0
(27A266a)** SDK's `.tbd` stubs — `libSystem.tbd:4:20: error: unknown
architecture`, pointing at the `arm64e.x1-macos` selector. Apple's own clang 21
(`/usr/bin/cc`, same Xcode) links the same trivial C program with exit 0.

Verified fix: `RUSTFLAGS="-C linker=/usr/bin/cc"` → `cargo build --workspace`
exit 0 and `cargo test --workspace` fully green on this workspace.
Note `CC=/usr/bin/cc` does **not** work — `CC` only steers the `cc` crate's C
compilation, not cargo's link driver.

**Status note (T002)**: this no longer reproduces. `cc` is mise-managed clang
pinned `latest` in the *global* `~/.config/mise/config.toml`, so mise updated
it; the same commands now exit 0 with no overrides at all. Treat the diagnosis
above as version-dependent — an earlier clang against Xcode 27.0 breaks, clang
23.1.2 does not — rather than as the current state of this machine.

---

- [x] **T001** 🔒 Maintainer decision: choose where the linker fix lives
  (enrichment Open Question 1).
  Options: (a) `~/.cargo/config.toml` with
  `[target.aarch64-apple-darwin] linker = "/usr/bin/cc"` — machine-local,
  invisible to the repo; (b) a committed `.cargo/config.toml` — fixes it for
  every macOS contributor but bakes a machine-specific absolute path into the
  repo; (c) mise/shim ordering so `cc` resolves to Apple clang, plus a
  documented prerequisite.
  **Exit criterion**: decision recorded on #77. T002/T003 must not start before
  it exits.
  **Files**: none (decision only).
  **Done**: recorded on #77. Selected (b), drafted `.cargo/config.toml`, then
  reversed to "no repo pin, docs only" in T002 once the failure proved
  non-reproducible — a committed machine-specific linker override with no
  reproducible failure behind it would not survive review.

- [x] **T002** Apply the chosen linker configuration so the workspace links
  with no per-command override.
  **Acceptance**: `cargo build --workspace` exits 0 on
  aarch64-apple-darwin with the installed Xcode 27 SDK, with `RUSTFLAGS`,
  `CC`, and `CARGO_TARGET_*_LINKER` all unset in the invoking shell; a trivial
  `cc -o /tmp/x /tmp/x.c` also succeeds.
  **Verify**: the quality gate — at minimum `cargo build --workspace` and
  `cargo test --workspace` green with no env overrides; then T005 runs the full
  six-step sequence.
  **Files**: machine config only (`~/.cargo/config.toml`, or the mise/shim
  setup) — **no repo file changes** in this task.
  **Done**: acceptance is met with **no change at all** — the failure no longer
  reproduces. `cc` is mise-managed clang pinned as `latest` in the *global*
  `~/.config/mise/config.toml`, so mise updated it; the same command that failed
  against Xcode 27.0 now links on clang 23.1.2. Verified with `RUSTFLAGS`/`CC`/
  `CARGO_TARGET_*_LINKER` unset: `cargo build --workspace` exit 0, `cargo
  test --workspace` 59 passed, and `cc -o /tmp/x /tmp/x.c` exit 0. Counterfactual
  also checked: with `.cargo/config.toml` removed, a `cargo clean -p` +
  `cargo build --workspace` still exits 0. The drafted config was deleted;
  nothing pinned, machine or repo.

- [x] **T003** 🔒 Record the prerequisite in the repo, so the next contributor
  hitting an un-linkable workspace is not blocked for an afternoon.
  **Acceptance**: `AGENTS.md` § *Development commands* (and/or
  `.github/instructions/rust.instructions.md`, whose code-style checklist
  already expects `cargo fmt`/`clippy`/`test` to pass) states the
  symptom, the verified cause (Homebrew clang vs the Xcode 27 SDK `.tbd`),
  the fix, and links issue #77.
  **Verify**: docs-only diff — the gate still runs green (T002 applied), and
  `grep -rn "Xcode" AGENTS.md` finds the new note.
  **Files**: `AGENTS.md`, `.github/instructions/rust.instructions.md`.
  **Done**: both files updated — symptom (`libSystem.tbd:4:20: error: unknown
  architecture`), cause (older Homebrew/mise clang vs the installed Xcode SDK),
  workaround (`RUSTFLAGS="-C linker=/usr/bin/cc"`; `CC` does not help), and a
  link to #77. Written as a hazard that tracks the clang/SDK pairing rather
  than an active broken state, since T002 found it already gone.

- [x] **T004** 🔒 Make `cargo fmt --check` usable again as gate step 1 —
  without reformatting the codebase under stable rustfmt.
  The repo has **no** rustfmt config; the nightly-only options
  (`imports_granularity`, `group_imports`, `format_strings`, …) live in the
  user's `~/.rustfmt.toml`, so stable rustfmt reports 50 diffs across untouched
  crates. Installing a nightly toolchain and running `cargo +nightly fmt
  --check` honours those options as-is.
  **Acceptance**: a nightly rustfmt is installed
  (`rustup toolchain install nightly --profile minimal --component rustfmt`) and
  `cargo +nightly fmt --check` reports **zero** diffs; the gate skill's step 1
  is updated to invoke it. No `.rs` file is reformatted as part of this task.
  **Verify**: `cargo +nightly fmt --check` prints nothing and exits 0; the
  remaining five gate steps still pass.
  **Files**: `.claude/skills/quality-gate/SKILL.md`, `AGENTS.md` §
  *Development commands*; local toolchain install.
  **Note**: if nightly rustfmt still reports diffs, that is a formatting
  decision for the maintainer, not something to fix silently here.
  **Done — superseded by #79** (merged, `8f8c25e`). The prescribed approach
  (install nightly, keep the user's nightly-only options) was measured and
  rejected: rustfmt auto-loads `~/.rustfmt.toml` (a 79-key dump whose
  `required_version = "1.7.1"` hard-errors on any available rustfmt and whose
  `unstable_features = false` disables its own nightly options), and
  `cargo +nightly` does not even work with a Homebrew cargo. Instead: a
  committed `rustfmt.toml` restricted to stable-honoured options, `cargo fmt`
  applied to a 39-site/10-file diff, and `cargo fmt --check` wired into CI and
  the gate skill. `cargo fmt --check` is now green on stable 1.10.0 and nightly
  1.11.0. No `.rs` file is reformatted under this task.

- [x] **T005** Run the complete quality gate locally, top to bottom, and post
  the transcript on #77 as the issue's exit evidence.
  **Acceptance**: all six steps of `.claude/skills/quality-gate/SKILL.md`
  (fmt, build ×2 feature configs, test ×2, clippy) complete with no errors, and
  the output is pasted into a #77 comment.
  **Verify**: the gate itself.
  **Files**: none (evidence lives in the issue comment).
  **Done**: full six-step gate run with `RUSTFLAGS`/`CC`/
  `CARGO_TARGET_*_LINKER` unset — fmt clean; both builds exit 0; tests 59
  (default) and 90 (turso) passed, 0 failed; clippy 20 warnings, the same
  pre-existing set as on `main`. Transcript posted to #77.

- [x] **T006** 🔒 Maintainer decision: the fate of two pre-existing dirty
  files in the main checkout, neither of which causes this issue.
  - untracked `rust-toolchain.toml`: pins `channel = "1.78.0"` **and**
    `targets = ["x86_64-unknown-linux-gnu"]` — a Linux target on a macOS box,
    and if committed it would override CI's `toolchain: stable`
    (`.github/workflows/ci.yml`), silently changing what CI builds.
  - modified `Cargo.lock` (266 insertions / 237 deletions of dependency bumps).
  **Exit criterion**: an explicit decision on #77 — commit, revert, or leave
  local — recorded before anyone commits either file. Do **not** commit
  `rust-toolchain.toml` as-is.
  **Files**: none unless the decision is "commit", in which case it is a
  separate PR with its own gate run.
  **Done**: `rust-toolchain.toml` was deleted from the main checkout and
  replaced by a committed, corrected one in #79 (`channel = "stable"`,
  `components = ["rustfmt", "clippy"]`, no `targets`). `Cargo.lock` is
  committed in this PR alongside the T003 docs, on the maintainer's decision to
  fold the bumps into the current PR rather than open a separate one.
