#!/usr/bin/env bash
# Fixture-based tests for previous-release-tag.sh (T005). Builds a throwaway
# git repo with a known tag history rather than touching the real repo's
# tags. Run manually:
#   bash .github/scripts/tests/test-previous-release-tag.sh
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
script="$script_dir/../previous-release-tag.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

pass=0
fail=0

check_eq() {
  local desc="$1" expected="$2" actual="$3"
  if [ "$expected" == "$actual" ]; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    echo "FAIL: $desc (expected '$expected', got '$actual')"
  fi
}

assert_fail() {
  local desc="$1"; shift
  if "$@" >/dev/null 2>&1; then
    fail=$((fail + 1))
    echo "FAIL (expected failure but succeeded): $desc"
  else
    pass=$((pass + 1))
  fi
}

(
  cd "$tmp"
  git init -q
  git config user.email "test@example.com"
  git config user.name "Test"
  # Lightweight, unsigned tags regardless of the caller's global git config
  # (e.g. tag.gpgsign=true would otherwise force an annotated/signed tag
  # here, which fails without an interactive editor or GPG key).
  git config tag.gpgsign false
  tag() { git -c tag.gpgsign=false tag "$1"; }

  echo "one" >file.txt
  git add file.txt
  git commit -q -m "first"
  tag v1.0.0

  echo "two" >file.txt
  git commit -q -am "second"
  tag v1.1.0

  echo "three" >file.txt
  git commit -q -am "third"
  tag v2.0.0
)

out="$(cd "$tmp" && bash "$script" "v1.1.0")"
check_eq "previous tag for v1.1.0 is v1.0.0" "v1.0.0" "$out"

out="$(cd "$tmp" && bash "$script" "v2.0.0")"
check_eq "previous tag for v2.0.0 is v1.1.0" "v1.1.0" "$out"

out="$(cd "$tmp" && bash "$script" "v1.0.0")"
check_eq "first release has no previous tag (empty output, FR-009 fallback)" "" "$out"

assert_fail "nonexistent tag fails" bash -c "cd '$tmp' && bash '$script' v9.9.9"

echo "----"
echo "pass=$pass fail=$fail"
[ "$fail" -eq 0 ]
