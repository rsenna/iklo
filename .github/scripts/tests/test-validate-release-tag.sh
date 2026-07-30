#!/usr/bin/env bash
# Fixture-based tests for validate-release-tag.sh (T005, Constitution I:
# written before/alongside the script, not after). Run manually:
#   bash .github/scripts/tests/test-validate-release-tag.sh
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
script="$script_dir/../validate-release-tag.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

pass=0
fail=0

assert_ok() {
  local desc="$1"; shift
  if "$@" >/tmp/out.$$ 2>&1; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    echo "FAIL (expected success): $desc"
    cat /tmp/out.$$
  fi
  rm -f /tmp/out.$$
}

assert_fail() {
  local desc="$1"; shift
  if "$@" >/tmp/out.$$ 2>&1; then
    fail=$((fail + 1))
    echo "FAIL (expected failure but succeeded): $desc"
    cat /tmp/out.$$
  else
    pass=$((pass + 1))
  fi
  rm -f /tmp/out.$$
}

cargo_toml="$tmp/Cargo.toml"
cat >"$cargo_toml" <<'EOF'
[workspace]
members = ["crates/*"]

[workspace.package]
version = "1.2.3"
edition = "2021"
EOF

assert_ok "matching tag passes" bash "$script" "v1.2.3" "$cargo_toml"
assert_fail "mismatched patch fails" bash "$script" "v1.2.4" "$cargo_toml"
assert_fail "mismatched minor fails" bash "$script" "v1.3.3" "$cargo_toml"
assert_fail "missing v prefix fails" bash "$script" "1.2.3" "$cargo_toml"
assert_fail "non-semver tag fails" bash "$script" "v1.2" "$cargo_toml"
assert_fail "pre-release suffix fails (out of scope per spec.md Assumptions)" bash "$script" "v1.2.3-rc1" "$cargo_toml"
assert_fail "missing Cargo.toml fails" bash "$script" "v1.2.3" "$tmp/does-not-exist.toml"

# Mismatch error message must name both values (SC-006).
out="$(bash "$script" "v9.9.9" "$cargo_toml" 2>&1 || true)"
if [[ "$out" == *"v9.9.9"* && "$out" == *"1.2.3"* ]]; then
  pass=$((pass + 1))
else
  fail=$((fail + 1))
  echo "FAIL: mismatch error must name both tag and workspace version, got: $out"
fi

echo "----"
echo "pass=$pass fail=$fail"
[ "$fail" -eq 0 ]
