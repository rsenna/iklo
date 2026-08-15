#!/usr/bin/env bash
# Fixture-based tests for reject-existing-release.sh (T012). Argument/
# environment validation, plus the HTTP-status-parsing logic (200/404/
# anything else) exercised via a `gh` stub on PATH -- a real call
# against GitHub's Release API still has no offline stand-in for the
# "does this repo/tag exist at all" question, but the status-parsing
# logic that decides pass/fail/ambiguous IS fully testable this way,
# closing the gap a plain "can't test gh api" would otherwise leave.
# Run manually:
#   bash .github/scripts/tests/test-reject-existing-release.sh
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
script="$script_dir/../reject-existing-release.sh"

pass=0
fail=0

assert_fail() {
  local desc="$1"; shift
  if "$@" >/dev/null 2>&1; then
    fail=$((fail + 1))
    echo "FAIL (expected failure but succeeded): $desc"
  else
    pass=$((pass + 1))
  fi
}

assert_ok() {
  local desc="$1"; shift
  if "$@" >/dev/null 2>&1; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    echo "FAIL (expected success but failed): $desc"
  fi
}

assert_fail "missing tag argument fails" env GITHUB_REPOSITORY=owner/repo bash "$script"
assert_fail "missing GITHUB_REPOSITORY fails" env -u GITHUB_REPOSITORY bash "$script" v1.0.0

# --- gh stub: fakes `gh api ... --include --silent`'s output format
# (an HTTP status line, then headers) so the status-parsing logic
# itself is exercised without a network call. ---
stub_dir="$(mktemp -d)"
trap 'rm -rf "$stub_dir"' EXIT
cat > "$stub_dir/gh" <<'STUB'
#!/usr/bin/env bash
# Minimal stand-in for `gh api <path> --include --silent`, driven by
# STUB_HTTP_STATUS. Real `gh` exits 1 for any non-2xx regardless of
# --include; this stub matches that.
printf 'HTTP/2.0 %s Stub\r\n' "${STUB_HTTP_STATUS:?}"
[ "${STUB_HTTP_STATUS}" -ge 200 ] && [ "${STUB_HTTP_STATUS}" -lt 300 ]
STUB
chmod +x "$stub_dir/gh"

assert_fail "existing release (200) is rejected"   env PATH="$stub_dir:$PATH" STUB_HTTP_STATUS=200 GITHUB_REPOSITORY=owner/repo bash "$script" v1.0.0
assert_ok "no existing release (404) proceeds"   env PATH="$stub_dir:$PATH" STUB_HTTP_STATUS=404 GITHUB_REPOSITORY=owner/repo bash "$script" v1.0.0
assert_fail "ambiguous failure (401) fails closed, not open"   env PATH="$stub_dir:$PATH" STUB_HTTP_STATUS=401 GITHUB_REPOSITORY=owner/repo bash "$script" v1.0.0
assert_fail "ambiguous failure (500) fails closed, not open"   env PATH="$stub_dir:$PATH" STUB_HTTP_STATUS=500 GITHUB_REPOSITORY=owner/repo bash "$script" v1.0.0

echo "----"
echo "pass=$pass fail=$fail"
[ "$fail" -eq 0 ]
