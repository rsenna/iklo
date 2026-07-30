#!/usr/bin/env bash
# Fixture-based tests for build-identifier.sh (T006, Constitution I). Run
# manually:
#   bash .github/scripts/tests/test-build-identifier.sh
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
script="$script_dir/../build-identifier.sh"

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

out="$(GITHUB_RUN_NUMBER=42 GITHUB_RUN_ATTEMPT=1 bash "$script")"
check_eq "basic case" "42.1" "$out"

out="$(GITHUB_RUN_NUMBER=1 GITHUB_RUN_ATTEMPT=3 bash "$script")"
check_eq "retry attempt" "1.3" "$out"

assert_fail "missing GITHUB_RUN_NUMBER fails" env -u GITHUB_RUN_NUMBER GITHUB_RUN_ATTEMPT=1 bash "$script"
assert_fail "missing GITHUB_RUN_ATTEMPT fails" env GITHUB_RUN_NUMBER=1 -u GITHUB_RUN_ATTEMPT bash "$script"
assert_fail "non-numeric run number fails" env GITHUB_RUN_NUMBER=abc GITHUB_RUN_ATTEMPT=1 bash "$script"
assert_fail "non-numeric run attempt fails" env GITHUB_RUN_NUMBER=1 GITHUB_RUN_ATTEMPT=abc bash "$script"
assert_fail "negative number fails (not base-10 unsigned per FR-005)" env GITHUB_RUN_NUMBER=-1 GITHUB_RUN_ATTEMPT=1 bash "$script"

# Numeric-pair ordering sanity (SC-003's "strictly increasing" comparison
# basis) -- not exercised by the script itself, but documents the contract
# callers must use rather than lexicographic string comparison.
a_run=9; a_attempt=9
b_run=10; b_attempt=1
if [ "$a_run" -lt "$b_run" ]; then
  pass=$((pass + 1))
else
  fail=$((fail + 1))
  echo "FAIL: numeric run-number comparison sanity check"
fi

echo "----"
echo "pass=$pass fail=$fail"
[ "$fail" -eq 0 ]
