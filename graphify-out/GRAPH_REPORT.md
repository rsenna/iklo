# Graph Report - chore-graphify-refresh  (2026-09-10)

## Corpus Check
- 148 files · ~196,415 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 586 nodes · 956 edges · 65 communities (24 shown, 40 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 24 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `0513c5b6`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- main.rs
- substrate.rs
- tests.rs
- Value
- LexError
- common.sh
- iklo-parser/src/lib.rs
- CodecError
- TursoSubstrateError
- InMemorySubstrate
- run_contract_suite
- Iklo Constitution
- Turso Database Skill
- iklo-cli
- test-generate-checksums.sh
- test-release-notes.sh
- test-previous-release-tag.sh
- release-notes.sh
- test-build-identifier.sh
- test-reject-existing-release.sh
- test-validate-release-tag.sh
- package.json
- Speckit Implement Agent
- Turso Cloud Skill
- apm.lock.yaml
- generate-checksums.sh
- previous-release-tag.sh
- reject-existing-release.sh
- validate-release-tag.sh
- JavaScript SDK (@tursodatabase/database)
- CommonMark Markdown Instructions
- Speckit Checklist Prompt
- Speckit Constitution Prompt
- build-identifier.sh
- Go SDK (tursogo)
- Python SDK (pyturso)
- Rust SDK (turso crate)
- Quality Gate Skill
- GitHub Actions CI/CD Best Practices
- Dependabot Configuration
- Generic Code Review Instructions
- Speckit Analyze Prompt
- Speckit Clarify Prompt
- Speckit Converge Prompt
- Speckit Implement Prompt
- Speckit Plan Prompt
- Speckit Specify Prompt
- Speckit Tasks Prompt
- Speckit TasksToIssues Prompt
- Iklo Language Reference (Historical)
- iklo
- IK0 Formal Grammar Spec
- IK0 Formal Grammar Tasks
- Implementation Plan Template
- Feature Specification Template
- Task List Template
- GitHub Spec Kit
- Speckit Analyze Agent
- Speckit Constitution Agent
- ADR-0002: Cross-Agent Supermemory
- CI Hardening Tasks (#32)
- Repo Standard Adoption Tasks (#46)
- Turso
- VDBE

## God Nodes (most connected - your core abstractions)
1. `Value` - 21 edges
2. `TursoSubstrateError` - 21 edges
3. `parse()` - 17 edges
4. `resolve_config()` - 15 edges
5. `parse_args()` - 13 edges
6. `RuntimeError` - 13 edges
7. `SubstrateError` - 13 edges
8. `run_contract_suite()` - 12 edges
9. `run_repl()` - 11 edges
10. `Iklo Constitution` - 11 edges

## Surprising Connections (you probably didn't know these)
- `REPL Improvements Plan` --references--> `Rust Coding Conventions`  [INFERRED]
  specs/003-repl-improvements/plan.md → .github/instructions/rust.instructions.md
- `turso_substrate_satisfies_contract()` --calls--> `run_contract_suite()`  [INFERRED]
  crates/iklo-substrate-turso/src/tests.rs → crates/iklo-substrate/src/contract.rs
- `Iklo Constitution` --references--> `Rust Coding Conventions`  [EXTRACTED]
  .specify/memory/constitution.md → .github/instructions/rust.instructions.md
- `Iklo Constitution` --references--> `Self-explanatory Code Commenting`  [EXTRACTED]
  .specify/memory/constitution.md → .github/instructions/self-explanatory-code-commenting.instructions.md
- `References README` --references--> `NetLogo User Manual`  [EXTRACTED]
  refs/README.md → refs/netlogo/net-logo-7.0.4-user-manual.pdf

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Iklo Language Specification Evolution** — iklo_agents_old, spec_002_grammar_plan, spec_003_repl_plan [EXTRACTED 0.90]
- **Spec Kit Development Workflow** — speckit, specify_memory_constitution, agents_md [EXTRACTED 0.90]
- **Turso Remote-Only Pattern** — agents_skills_turso_db_sdks_serverless, agents_skills_turso_cloud_turso_cloud_go_overview, agents_skills_turso_cloud_turso_cloud_py_overview, agents_skills_turso_cloud_turso_cloud_rust_overview [EXTRACTED 0.90]
- **Repository Governance and Standards** — github_actions_ci_cd_best_practices, github_instructions_markdown_instructions, github_instructions_rust_instructions, self_explanatory_code_commenting, update_docs_on_code_change, task_issue_46_repo_standard [EXTRACTED 0.95]
- **Turso Local-First Sync Pattern** — agents_skills_turso_cloud_turso_cloud_go_overview, agents_skills_turso_cloud_turso_cloud_py_overview, agents_skills_turso_cloud_turso_cloud_rust_overview, agents_skills_turso_db_sdks_react_native, agents_skills_turso_db_sdks_wasm [EXTRACTED 0.95]
- **Iklo Substrate Architecture** — specs_001_substrate_spec, specs_001_substrate_plan, specs_001_substrate_tasks, specs_004_turso_substrate_backend_plan [EXTRACTED 1.00]
- **Speckit Agent Suite** — speckit_analyze, speckit_checklist, speckit_clarify, speckit_constitution, speckit_converge, speckit_implement [EXTRACTED 1.00]
- **Speckit SDD Core Loop** — github_agents_speckit_specify_agent, github_agents_speckit_plan_agent, github_agents_speckit_tasks_agent [EXTRACTED 1.00]
- **SpecKit Workflow Templates** — specify_templates_spec, specify_templates_plan, specify_templates_tasks [EXTRACTED 1.00]
- **Logo Influence on Iklo Design** — refs_ucblogo_summary, refs_ucblogo_understanding_ucblogo_evaluator, refs_iklo_turtle [INFERRED 0.85]
- **Project Governance & Standards** — specify_memory_constitution, github_workflows_ci, github_workflows_release [INFERRED 0.85]
- **Iklo Language Design Roadmap** — specs_006_strictness_effects_spike_spec, specs_007_ik1_core_language_spec, specs_008_binding_model_taxonomy_spec, specs_009_binding_kinds_spec, specs_010_types_literals_spec [INFERRED 0.90]

## Communities (65 total, 40 thin omitted)

### Community 0 - "main.rs"
Cohesion: 0.05
Nodes (54): Cell, Completer, Context, args(), is_repl_command_position(), load_history_from_missing_path_returns_error(), main(), multiline_continuation_preserves_newline_across_comment_boundary() (+46 more)

### Community 1 - "substrate.rs"
Cohesion: 0.09
Nodes (26): apply_write(), fire_on_retry_hook_for_test(), last_commit_attempt_for_test(), load_bindings(), read_revision(), Connection, Formatter, HashMap (+18 more)

### Community 2 - "tests.rs"
Cohesion: 0.07
Nodes (33): Arc, AtomicBool, apply_equivalence_sequence(), bootstrap_is_idempotent(), classify_uses_a_real_turso_error_obtained_from_a_live_database(), commit_persists_across_a_fresh_instance(), commit_retries_past_transient_lock_contention_and_succeeds(), commit_surfaces_an_error_after_exhausting_retries_under_sustained_contention() (+25 more)

### Community 3 - "Value"
Cohesion: 0.05
Nodes (44): eval_binop(), eval_expr(), eval_program(), let_returns_bound_value(), rollback_keeps_image_unchanged(), Default, Display, Error (+36 more)

### Community 4 - "LexError"
Cohesion: 0.12
Nodes (20): BinOp, Expr, Box, Self, String, Span, Spanned, Spanned<T> (+12 more)

### Community 5 - "common.sh"
Cohesion: 0.15
Nodes (22): check-prerequisites.sh script, check_dir(), check_file(), find_specify_root(), format_speckit_command(), get_current_branch(), get_feature_paths(), get_invoke_separator() (+14 more)

### Community 6 - "iklo-parser/src/lib.rs"
Cohesion: 0.06
Nodes (41): colon_name_reads_binding(), division_is_left_associative(), let_is_an_expression(), let_is_valid_in_nested_expression_position(), multiple_blank_lines_between_expressions(), newline_after_let_continues_to_name(), newline_terminates_when_expression_is_valid(), parens_swallow_newlines() (+33 more)

### Community 7 - "CodecError"
Cohesion: 0.15
Nodes (13): Codec, CodecError, i64, i64_codec_encodes_with_expected_tag_and_length(), i64_codec_round_trips_extremes_and_zero(), Display, Error, Formatter (+5 more)

### Community 8 - "TursoSubstrateError"
Cohesion: 0.10
Nodes (24): AmbiguousCommitResolution, classify(), iklo_substrate::SubstrateError, resolve_ambiguous_commit(), RetryClass, RetryPolicy, Display, Error (+16 more)

### Community 9 - "InMemorySubstrate"
Cohesion: 0.21
Nodes (9): InMemorySubstrate, InMemorySubstrate<V>, InMemoryTx, Default, HashMap, Self, String, Tx (+1 more)

### Community 10 - "run_contract_suite"
Cohesion: 0.50
Nodes (11): commit_increments_revision(), get_after_commit_sees_value_from_fresh_tx(), get_after_rollback_does_not_see_value(), get_after_set_inside_tx_sees_value(), in_memory_substrate_satisfies_contract(), revision_starts_at_zero(), rollback_does_not_increment_revision(), S (+3 more)

### Community 11 - "Iklo Constitution"
Cohesion: 0.07
Nodes (43): Planned Examples README, Speckit Plan Agent, Speckit Specify Agent, Speckit Tasks Agent, Speckit Tasks-to-Issues Agent, Copilot Instructions, Rust Coding Conventions, CI Workflow (+35 more)

### Community 12 - "Turso Database Skill"
Cohesion: 0.29
Nodes (7): Turso Database Skill, Turso CDC, Turso Encryption, Turso Full-Text Search, Turso MVCC, Turso Remote Sync, Turso Vector Search

### Community 13 - "iklo-cli"
Cohesion: 0.48
Nodes (7): iklo-ast, iklo-cli, iklo-lexer, iklo-parser, iklo-runtime, iklo-substrate, iklo-substrate-turso

### Community 14 - "test-generate-checksums.sh"
Cohesion: 0.67
Nodes (5): check(), check_file_exists(), checksum_records_basename(), test-generate-checksums.sh script, verify_checksum()

### Community 15 - "test-release-notes.sh"
Cohesion: 0.60
Nodes (5): check_contains(), check_contains_exact_line(), check_not_contains(), test-release-notes.sh script, tag()

### Community 16 - "test-previous-release-tag.sh"
Cohesion: 0.70
Nodes (4): assert_fail(), check_eq(), test-previous-release-tag.sh script, tag()

### Community 18 - "release-notes.sh"
Cohesion: 0.83
Nodes (3): print_section(), release-notes.sh script, usage()

### Community 19 - "test-build-identifier.sh"
Cohesion: 0.83
Nodes (3): assert_fail(), check_eq(), test-build-identifier.sh script

### Community 20 - "test-reject-existing-release.sh"
Cohesion: 0.83
Nodes (3): assert_fail(), assert_ok(), test-reject-existing-release.sh script

### Community 21 - "test-validate-release-tag.sh"
Cohesion: 0.83
Nodes (3): assert_fail(), assert_ok(), test-validate-release-tag.sh script

### Community 23 - "package.json"
Cohesion: 0.50
Nodes (3): dependencies, @opencode-ai/plugin, @opencode-ai/plugin

### Community 24 - "Speckit Implement Agent"
Cohesion: 0.50
Nodes (4): Speckit Checklist Agent, Speckit Clarify Agent, Speckit Converge Agent, Speckit Implement Agent

### Community 26 - "Turso Cloud Skill"
Cohesion: 0.25
Nodes (9): Turso Cloud Skill, Turso Cloud Authentication & Authorization, Turso Cloud Go SDK Overview, Turso Cloud JavaScript/TypeScript Overview, Turso Cloud Python SDK Overview, Turso Cloud Rust SDK Overview, Turso Vercel Marketplace Integration, React Native SDK (+1 more)

## Knowledge Gaps
- **70 isolated node(s):** `build-identifier.sh script`, `common.sh script`, `iklo`, `Spanned<T>`, `Position` (+65 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 175 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **40 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Value` connect `Value` to `InMemorySubstrate`, `substrate.rs`, `CodecError`?**
  _High betweenness centrality (0.186) - this node is a cross-community bridge._
- **Why does `eval_expr()` connect `Value` to `LexError`?**
  _High betweenness centrality (0.142) - this node is a cross-community bridge._
- **Why does `Spanned` connect `LexError` to `Value`?**
  _High betweenness centrality (0.131) - this node is a cross-community bridge._
- **What connects `build-identifier.sh script`, `common.sh script`, `iklo` to the rest of the system?**
  _70 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `main.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05261261261261261 - nodes in this community are weakly interconnected._
- **Should `substrate.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.09176788124156546 - nodes in this community are weakly interconnected._
- **Should `tests.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.06612244897959184 - nodes in this community are weakly interconnected._