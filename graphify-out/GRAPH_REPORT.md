# Graph Report - iklo  (2026-09-08)

## Corpus Check
- 157 files · ~196,819 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 577 nodes · 876 edges · 93 communities (30 shown, 63 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 24 edges (avg confidence: 0.8)
- Token cost: 3,608 input · 1,865 output

## Community Hubs (Navigation)
- Cli Repl Parse Command
- Substrate Turso Commit For
- Turso Substrate Error And
- Runtime Value Decode Ast
- Token Parser Lexer Ast
- Specify Scripts Bash Common
- Parser Let Expression Newline
- Codec Substrate I64 And
- Substrate Memory Result Display
- Memory Substrate Inmemorysubstrate Default
- Contract Substrate Revision Get
- Repo Speckit Agent Specify
- Turso Repo Skills References
- Cargo Substrate Ast Cli
- Generate Checksums Scripts Check
- Release Notes Scripts Check
- Tag Previous Release Scripts
- Repo Refs Planned Examples
- Release Notes Scripts Print
- Identifier Scripts Assert Fail
- Reject Existing Release Scripts
- Validate Release Tag Scripts
- Release Repo Versioning Workflow
- Package Plugin Dependencies
- Speckit Agent Repo Checklist
- Substrate Repo Capability Boundary
- Turso Cloud Repo Skills
- Apm Lock Repo Mcp
- Generate Checksums Scripts Script
- Previous Release Tag Scripts
- Reject Existing Release Scripts
- Validate Release Tag Scripts
- Plugins Graphifyplugin Important Keep
- Repo Turtle Refs Ucblogo
- Turso Substrate Repo Backend
- Turso Cloud Repo Skills
- Javascript Sdk Repo Skills
- Update Code Change Instructions
- Repl Improvements Plan Repo
- Checklist Speckit Repo Prompts
- Constitution Speckit Repo Prompts
- Identifier Scripts Script
- Ik1 Core Language Repo
- Binding Model Taxonomy Repo
- Turso Cloud Skill
- Turso Cloud Go SDK Overview
- Turso Cloud Python SDK Overview
- Turso Cloud Rust SDK Overview
- Go SDK (tursogo)
- Python SDK (pyturso)
- Rust SDK (turso crate)
- Quality Gate Skill
- GitHub Actions CI/CD Best Practices
- Copilot Instructions
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
- Iklo README (Historical)
- iklo
- The clock calculus — a summary for Iklo
- Understanding the UCBLogo evaluator
- Self-explanatory Code Commenting
- IK0 Formal Grammar Plan
- IK0 Formal Grammar Tasks
- REPL Improvements Tasks
- Implementation Plan Template
- Feature Specification Template
- Task List Template
- GitHub Spec Kit
- Speckit Analyze Agent
- Speckit Constitution Agent
- IK0 Formal Grammar Spec
- Turso-backed Substrate Backend Plan
- Turso-backed Substrate Backend Tasks
- CI Release Pipeline and Semantic Versioning Spec
- Strictness and Side-Effects Model Spike
- ADR-0002: Cross-Agent Supermemory
- ADR-0003: Queue-based Evaluation
- ADR-0004: REPL Slash Commands
- Technical Debt
- CI Hardening Tasks (#32)
- Repo Standard Adoption Tasks (#46)
- Turso
- VDBE

## God Nodes (most connected - your core abstractions)
1. `TursoSubstrateError` - 21 edges
2. `Value` - 18 edges
3. `parse()` - 17 edges
4. `resolve_config()` - 15 edges
5. `parse_args()` - 13 edges
6. `RuntimeError` - 13 edges
7. `run_contract_suite()` - 12 edges
8. `run_repl()` - 11 edges
9. `SubstrateError` - 11 edges
10. `LexicalError` - 10 edges

## Surprising Connections (you probably didn't know these)
- `turso_substrate_satisfies_contract()` --calls--> `run_contract_suite()`  [INFERRED]
  crates/iklo-substrate-turso/src/tests.rs → crates/iklo-substrate/src/contract.rs
- `REPL Improvements Plan` --references--> `Rust Coding Conventions`  [INFERRED]
  specs/003-repl-improvements/plan.md → .github/instructions/rust.instructions.md
- `Speckit Checklist Prompt` --references--> `Checklist Template`  [INFERRED]
  .github/prompts/speckit.checklist.prompt.md → .specify/templates/checklist-template.md
- `Speckit Constitution Prompt` --references--> `Constitution Template`  [INFERRED]
  .github/prompts/speckit.constitution.prompt.md → .specify/templates/constitution-template.md
- `Iklo Turtle Mascot` --conceptually_related_to--> `Berkeley Logo (UCBLogo) Reference`  [INFERRED]
  refs/iklo-turtle.jpeg → refs/ucblogo/summary.md

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Turso Local-First Sync Pattern** — agents_skills_turso_cloud_turso_cloud_go_overview, agents_skills_turso_cloud_turso_cloud_py_overview, agents_skills_turso_cloud_turso_cloud_rust_overview, agents_skills_turso_db_sdks_react_native, agents_skills_turso_db_sdks_wasm [EXTRACTED 0.95]
- **Turso Remote-Only Pattern** — agents_skills_turso_db_sdks_serverless, agents_skills_turso_cloud_turso_cloud_go_overview, agents_skills_turso_cloud_turso_cloud_py_overview, agents_skills_turso_cloud_turso_cloud_rust_overview [EXTRACTED 0.90]
- **Speckit Agent Suite** — speckit_analyze, speckit_checklist, speckit_clarify, speckit_constitution, speckit_converge, speckit_implement [EXTRACTED 1.00]
- **Repository Governance and Standards** — github_actions_ci_cd_best_practices, github_instructions_markdown_instructions, github_instructions_rust_instructions, self_explanatory_code_commenting, update_docs_on_code_change, task_issue_46_repo_standard [EXTRACTED 0.95]
- **Project Governance & Standards** — specify_memory_constitution, github_workflows_ci, github_workflows_release [INFERRED 0.85]
- **SpecKit Workflow Templates** — specify_templates_spec, specify_templates_plan, specify_templates_tasks [EXTRACTED 1.00]
- **Speckit SDD Core Loop** — github_agents_speckit_specify_agent, github_agents_speckit_plan_agent, github_agents_speckit_tasks_agent [EXTRACTED 1.00]
- **Spec Kit Development Workflow** — speckit, specify_memory_constitution, agents_md [EXTRACTED 0.90]
- **Iklo Language Specification Evolution** — iklo_agents_old, spec_002_grammar_plan, spec_003_repl_plan [EXTRACTED 0.90]
- **Logo Influence on Iklo Design** — refs_ucblogo_summary, refs_ucblogo_understanding_ucblogo_evaluator, refs_iklo_turtle [INFERRED 0.85]
- **Iklo Language Design Roadmap** — specs_006_strictness_effects_spike_spec, specs_007_ik1_core_language_spec, specs_008_binding_model_taxonomy_spec, specs_009_binding_kinds_spec, specs_010_types_literals_spec [INFERRED 0.90]
- **Iklo Substrate Architecture** — specs_001_substrate_spec, specs_001_substrate_plan, specs_001_substrate_tasks, specs_004_turso_substrate_backend_plan [EXTRACTED 1.00]

## Communities (93 total, 63 thin omitted)

### Community 0 - "Cli Repl Parse Command"
Cohesion: 0.05
Nodes (54): Cell, Completer, Context, args(), is_repl_command_position(), load_history_from_missing_path_returns_error(), main(), multiline_continuation_preserves_newline_across_comment_boundary() (+46 more)

### Community 1 - "Substrate Turso Commit For"
Cohesion: 0.05
Nodes (50): AmbiguousCommitResolution, classify(), iklo_substrate::SubstrateError, resolve_ambiguous_commit(), RetryClass, RetryPolicy, Display, Error (+42 more)

### Community 2 - "Turso Substrate Error And"
Cohesion: 0.07
Nodes (33): Arc, AtomicBool, apply_equivalence_sequence(), bootstrap_is_idempotent(), classify_uses_a_real_turso_error_obtained_from_a_live_database(), commit_persists_across_a_fresh_instance(), commit_retries_past_transient_lock_contention_and_succeeds(), commit_surfaces_an_error_after_exhausting_retries_under_sustained_contention() (+25 more)

### Community 3 - "Runtime Value Decode Ast"
Cohesion: 0.08
Nodes (35): BinOp, Expr, Box, String, Spanned, eval_binop(), eval_expr(), eval_program() (+27 more)

### Community 4 - "Token Parser Lexer Ast"
Cohesion: 0.07
Nodes (33): Self, Span, Spanned<T>, Lexeme, LexemeKind, LexError, line_col_at(), Display (+25 more)

### Community 5 - "Specify Scripts Bash Common"
Cohesion: 0.15
Nodes (22): check-prerequisites.sh script, check_dir(), check_file(), find_specify_root(), format_speckit_command(), get_current_branch(), get_feature_paths(), get_invoke_separator() (+14 more)

### Community 6 - "Parser Let Expression Newline"
Cohesion: 0.12
Nodes (23): colon_name_reads_binding(), division_is_left_associative(), let_is_an_expression(), let_is_valid_in_nested_expression_position(), multiple_blank_lines_between_expressions(), newline_after_let_continues_to_name(), newline_terminates_when_expression_is_valid(), parens_swallow_newlines() (+15 more)

### Community 7 - "Codec Substrate I64 And"
Cohesion: 0.14
Nodes (13): Codec, CodecError, i64, i64_codec_encodes_with_expected_tag_and_length(), i64_codec_round_trips_extremes_and_zero(), Display, Error, Formatter (+5 more)

### Community 8 - "Substrate Memory Result Display"
Cohesion: 0.14
Nodes (11): Display, Error, Formatter, Result, String, Substrate, SubstrateError, Transaction (+3 more)

### Community 9 - "Memory Substrate Inmemorysubstrate Default"
Cohesion: 0.21
Nodes (9): InMemorySubstrate, InMemorySubstrate<V>, InMemoryTx, Default, HashMap, Self, String, Tx (+1 more)

### Community 10 - "Contract Substrate Revision Get"
Cohesion: 0.50
Nodes (11): commit_increments_revision(), get_after_commit_sees_value_from_fresh_tx(), get_after_rollback_does_not_see_value(), get_after_set_inside_tx_sees_value(), in_memory_substrate_satisfies_contract(), revision_starts_at_zero(), rollback_does_not_increment_revision(), S (+3 more)

### Community 11 - "Repo Speckit Agent Specify"
Cohesion: 0.27
Nodes (6): Speckit Plan Agent, Speckit Specify Agent, Speckit Tasks Agent, Speckit Tasks-to-Issues Agent, Iklo Constitution, Speckit Full SDD Workflow

### Community 12 - "Turso Repo Skills References"
Cohesion: 0.29
Nodes (7): Turso Database Skill, Turso CDC, Turso Encryption, Turso Full-Text Search, Turso MVCC, Turso Remote Sync, Turso Vector Search

### Community 13 - "Cargo Substrate Ast Cli"
Cohesion: 0.48
Nodes (7): iklo-ast, iklo-cli, iklo-lexer, iklo-parser, iklo-runtime, iklo-substrate, iklo-substrate-turso

### Community 14 - "Generate Checksums Scripts Check"
Cohesion: 0.67
Nodes (5): check(), check_file_exists(), checksum_records_basename(), test-generate-checksums.sh script, verify_checksum()

### Community 15 - "Release Notes Scripts Check"
Cohesion: 0.60
Nodes (5): check_contains(), check_contains_exact_line(), check_not_contains(), test-release-notes.sh script, tag()

### Community 16 - "Tag Previous Release Scripts"
Cohesion: 0.70
Nodes (4): assert_fail(), check_eq(), test-previous-release-tag.sh script, tag()

### Community 17 - "Repo Refs Planned Examples"
Cohesion: 0.50
Nodes (4): Planned Examples README, Iklo Language, NetLogo User Manual, References README

### Community 18 - "Release Notes Scripts Print"
Cohesion: 0.83
Nodes (3): print_section(), release-notes.sh script, usage()

### Community 19 - "Identifier Scripts Assert Fail"
Cohesion: 0.83
Nodes (3): assert_fail(), check_eq(), test-build-identifier.sh script

### Community 20 - "Reject Existing Release Scripts"
Cohesion: 0.83
Nodes (3): assert_fail(), assert_ok(), test-reject-existing-release.sh script

### Community 21 - "Validate Release Tag Scripts"
Cohesion: 0.83
Nodes (3): assert_fail(), assert_ok(), test-validate-release-tag.sh script

### Community 22 - "Release Repo Versioning Workflow"
Cohesion: 0.83
Nodes (4): CI Workflow, Release Workflow, CI Release Pipeline and Semantic Versioning Plan, CI Release Pipeline and Semantic Versioning Tasks

### Community 23 - "Package Plugin Dependencies"
Cohesion: 0.50
Nodes (3): @opencode-ai/plugin, dependencies, @opencode-ai/plugin

### Community 24 - "Speckit Agent Repo Checklist"
Cohesion: 0.50
Nodes (4): Speckit Checklist Agent, Speckit Clarify Agent, Speckit Converge Agent, Speckit Implement Agent

### Community 25 - "Substrate Repo Capability Boundary"
Cohesion: 0.50
Nodes (4): Substrate Capability Boundary Plan, Substrate Capability Boundary Spec, Substrate Capability Boundary Tasks, Epic Execution Queue

### Community 26 - "Turso Cloud Repo Skills"
Cohesion: 0.67
Nodes (3): Turso Cloud JavaScript/TypeScript Overview, Turso Vercel Marketplace Integration, Serverless SDK (@tursodatabase/serverless)

### Community 33 - "Repo Turtle Refs Ucblogo"
Cohesion: 0.67
Nodes (3): Iklo Turtle Mascot, Berkeley Logo (UCBLogo) Reference, REPL Improvements Spec

### Community 34 - "Turso Substrate Repo Backend"
Cohesion: 0.67
Nodes (3): Turso-backed Substrate Backend Spec, ADR-0001: Substrate Boundary, ADR-0005: Turso Fork Governance

## Knowledge Gaps
- **90 isolated node(s):** `build-identifier.sh script`, `common.sh script`, `iklo`, `Spanned<T>`, `Position` (+85 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **63 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Value` connect `Runtime Value Decode Ast` to `Substrate Memory Result Display`, `Memory Substrate Inmemorysubstrate Default`, `Substrate Turso Commit For`, `Codec Substrate I64 And`?**
  _High betweenness centrality (0.172) - this node is a cross-community bridge._
- **Why does `eval_expr()` connect `Runtime Value Decode Ast` to `Substrate Memory Result Display`?**
  _High betweenness centrality (0.141) - this node is a cross-community bridge._
- **Why does `Spanned` connect `Runtime Value Decode Ast` to `Token Parser Lexer Ast`?**
  _High betweenness centrality (0.130) - this node is a cross-community bridge._
- **What connects `build-identifier.sh script`, `common.sh script`, `iklo` to the rest of the system?**
  _90 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Cli Repl Parse Command` be split into smaller, more focused modules?**
  _Cohesion score 0.05261261261261261 - nodes in this community are weakly interconnected._
- **Should `Substrate Turso Commit For` be split into smaller, more focused modules?**
  _Cohesion score 0.05093167701863354 - nodes in this community are weakly interconnected._
- **Should `Turso Substrate Error And` be split into smaller, more focused modules?**
  _Cohesion score 0.06612244897959184 - nodes in this community are weakly interconnected._