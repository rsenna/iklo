#!/usr/bin/env bash
# Fixture-based tests for release-notes.sh (T013/T014). Builds a throwaway
# git repo with a known commit/tag history rather than touching the real
# repo's history. Run manually:
#   bash .github/scripts/tests/test-release-notes.sh
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
script="$script_dir/../release-notes.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

pass=0
fail=0

check_contains() {
  local desc="$1" haystack="$2" needle="$3"
  if printf '%s' "$haystack" | grep -qF -- "$needle"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    echo "FAIL: $desc (expected to find '$needle')"
  fi
}

check_not_contains() {
  local desc="$1" haystack="$2" needle="$3"
  if printf '%s' "$haystack" | grep -qF -- "$needle"; then
    fail=$((fail + 1))
    echo "FAIL: $desc (unexpectedly found '$needle')"
  else
    pass=$((pass + 1))
  fi
}

(
  cd "$tmp"
  git init -q
  git config user.email "test@example.com"
  git config user.name "Test"
  git config tag.gpgsign false
  tag() { git -c tag.gpgsign=false tag "$1"; }

  echo "one" >file.txt
  git add file.txt
  git commit -q -m "chore: repo init"
  tag v1.0.0

  echo "two" >file.txt
  git commit -q -am "feat: add substrate boundary"
  echo "three" >file.txt
  git commit -q -am "fix: correct rollback semantics"
  echo "four" >file.txt
  git commit -q -am "docs: update AGENTS.md"
  echo "five" >file.txt
  git commit -q -am "chore: bump toolchain"
  echo "six" >file.txt
  git commit -q -am "tidy up variable names"
  tag v1.1.0
)

# --- conventional-commit-prefix grouping (feat/fix/docs/chore), FR-007 ---
out="$(cd "$tmp" && bash "$script" v1.1.0)"
check_contains "feat commit grouped under Features" "$out" "### Features"
check_contains "feat commit message present" "$out" "add substrate boundary"
check_contains "fix commit grouped under Fixes" "$out" "### Fixes"
check_contains "fix commit message present" "$out" "correct rollback semantics"
check_contains "docs commit grouped under Documentation" "$out" "### Documentation"
check_contains "docs commit message present" "$out" "update AGENTS.md"
check_contains "chore commit grouped under Chores" "$out" "### Chores"
check_contains "chore commit message present" "$out" "bump toolchain"

# --- fallback bucket for unmatched commit subjects, FR-007 ---
check_contains "non-conventional commit grouped under Other Changes" "$out" "### Other Changes"
check_contains "non-conventional commit message present verbatim" "$out" "tidy up variable names"

# --- only commits in previous_tag..current_tag are included, not the
# tagging commit itself (v1.0.0's "chore: repo init" must NOT appear) ---
check_not_contains "v1.0.0's own commit is excluded from v1.1.0's notes" "$out" "repo init"

# --- first-release fallback path when no previous tag exists, FR-009 ---
out_first="$(cd "$tmp" && bash "$script" v1.0.0)"
check_contains "first release includes its own commit (full-history fallback)" "$out_first" "repo init"

# --- breaking-change marker (feat!:, fix(scope)!:) groups with its
# non-breaking counterpart, not the fallback bucket ---
(
  cd "$tmp"
  echo "seven" >file.txt
  git commit -q -am "feat!: drop deprecated flag"
  echo "eight" >file.txt
  git commit -q -am "fix(cli)!: change exit code semantics"
  git -c tag.gpgsign=false tag v1.2.0
)
out_breaking="$(cd "$tmp" && bash "$script" v1.2.0)"
check_contains "feat! groups under Features" "$out_breaking" "### Features"
check_contains "feat! message stripped of prefix" "$out_breaking" "drop deprecated flag"
check_contains "fix(scope)! groups under Fixes" "$out_breaking" "### Fixes"
check_contains "fix(scope)! message stripped of prefix" "$out_breaking" "change exit code semantics"

echo "----"
echo "pass=$pass fail=$fail"
[ "$fail" -eq 0 ]
