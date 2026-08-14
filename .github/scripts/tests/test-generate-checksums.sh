#!/usr/bin/env bash
# Fixture-based tests for generate-checksums.sh (T011, Constitution I). Run
# manually:
#   bash .github/scripts/tests/test-generate-checksums.sh
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
script="$script_dir/../generate-checksums.sh"

pass=0
fail=0

check() {
  local desc="$1" ok="$2"
  if [ "$ok" = "0" ]; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    echo "FAIL: $desc"
  fi
}

verify_checksum() {
  # Portable sha256sum -c across macOS (shasum) and Linux (sha256sum).
  # `--`: a dash-prefixed checksum filename (e.g. "-dash-artifact.sha256")
  # would otherwise be parsed as an option, same footgun the script under
  # test guards against for the artifact filename itself.
  local checksum_file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$(dirname "$checksum_file")" && sha256sum -c -- "$(basename "$checksum_file")") >/dev/null 2>&1
  else
    (cd "$(dirname "$checksum_file")" && shasum -a 256 -c -- "$(basename "$checksum_file")") >/dev/null 2>&1
  fi
}

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# --- happy path: checksum file is created and verifies against its artifact ---
printf 'iklo release artifact fixture\n' > "$tmp/iklo-v0.1.0-x86_64-unknown-linux-gnu"
"$script" "$tmp/iklo-v0.1.0-x86_64-unknown-linux-gnu"
check "checksum file created" "$([ -f "$tmp/iklo-v0.1.0-x86_64-unknown-linux-gnu.sha256" ]; echo $?)"
if verify_checksum "$tmp/iklo-v0.1.0-x86_64-unknown-linux-gnu.sha256"; then
  check "checksum verifies against its real artifact (sha256sum -c)" 0
else
  check "checksum verifies against its real artifact (sha256sum -c)" 1
fi

# --- checksum file references the artifact by basename, not an absolute path ---
# (release assets are downloaded flat, side-by-side -- a checksum file
# containing the build machine's absolute path would fail `sha256sum -c`
# for every downloader.)
check "checksum file references artifact by basename, not absolute path" \
  "$(grep -q "$tmp" "$tmp/iklo-v0.1.0-x86_64-unknown-linux-gnu.sha256" && echo 1 || echo 0)"

# --- negative: tampering the artifact after the fact must fail verification ---
printf 'tampered content\n' > "$tmp/iklo-v0.1.0-x86_64-unknown-linux-gnu"
if verify_checksum "$tmp/iklo-v0.1.0-x86_64-unknown-linux-gnu.sha256"; then
  fail=$((fail + 1))
  echo "FAIL: tampered artifact should NOT verify, but it did"
else
  pass=$((pass + 1))
fi

# --- multiple artifacts in ONE invocation (exercises the for-loop, not
# just two separate single-arg calls) -- content-verified, not just
# existence, so a regression that attributes the wrong checksum to the
# wrong file (or writes an invalid one) would be caught. ---
printf 'second artifact\n' > "$tmp/second-artifact"
printf 'third artifact\n' > "$tmp/third-artifact"
"$script" "$tmp/second-artifact" "$tmp/third-artifact"
check "second checksum file created (multi-arg call)" "$([ -f "$tmp/second-artifact.sha256" ]; echo $?)"
check "third checksum file created (multi-arg call)" "$([ -f "$tmp/third-artifact.sha256" ]; echo $?)"
if verify_checksum "$tmp/second-artifact.sha256"; then
  check "second artifact's checksum verifies (multi-arg call)" 0
else
  check "second artifact's checksum verifies (multi-arg call)" 1
fi
if verify_checksum "$tmp/third-artifact.sha256"; then
  check "third artifact's checksum verifies (multi-arg call)" 0
else
  check "third artifact's checksum verifies (multi-arg call)" 1
fi

# --- dash-prefixed basename: regression test for the `--` fix (without
# it, sha256sum/shasum would parse "-artifact" as an option and fail) ---
printf 'dash-prefixed artifact\n' > "$tmp/-dash-artifact"
"$script" "$tmp/-dash-artifact"
check "dash-prefixed-basename checksum file created" "$([ -f "$tmp/-dash-artifact.sha256" ]; echo $?)"
if verify_checksum "$tmp/-dash-artifact.sha256"; then
  check "dash-prefixed-basename checksum verifies" 0
else
  check "dash-prefixed-basename checksum verifies" 1
fi

# --- missing argument fails ---
if "$script" >/dev/null 2>&1; then
  fail=$((fail + 1))
  echo "FAIL (expected failure but succeeded): no arguments"
else
  pass=$((pass + 1))
fi

# --- nonexistent file fails ---
if "$script" "$tmp/does-not-exist" >/dev/null 2>&1; then
  fail=$((fail + 1))
  echo "FAIL (expected failure but succeeded): nonexistent file"
else
  pass=$((pass + 1))
fi

echo "----"
echo "pass=$pass fail=$fail"
[ "$fail" -eq 0 ]
