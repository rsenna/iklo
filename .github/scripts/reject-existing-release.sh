#!/usr/bin/env bash
# Rejects re-publishing an existing tag/version (edge case, spec.md): fails
# if a GitHub Release already exists for <tag>. Uses `gh api` against the
# real GitHub Release API -- unlike validate-release-tag.sh/
# previous-release-tag.sh (T005), the real API call cannot be exercised
# offline against a throwaway git repo fixture, since there's no local
# stand-in for GitHub's own Release API (the argument/environment
# validation is still test-first, see tests/test-reject-existing-release.sh).
# Requires GH_TOKEN/GITHUB_TOKEN (gh CLI's own env var detection) and
# GITHUB_REPOSITORY ("owner/repo", set automatically by GitHub Actions).
set -euo pipefail

usage() {
  echo "usage: $0 <tag>" >&2
  exit 2
}

[ $# -ge 1 ] || usage
tag="$1"

: "${GITHUB_REPOSITORY:?GITHUB_REPOSITORY must be set (owner/repo)}"

# `gh api` exits 1 for EVERY failure -- a 404 (no release yet, the
# expected/desired case) is indistinguishable by exit code alone from a
# bad token, a network blip, or an API outage. Since this check exists
# specifically to block duplicate publishes, treating any failure as
# "no release exists, proceed" would silently disable the guard exactly
# when it can't confirm the answer -- fail closed instead by parsing the
# actual HTTP status line (--include --silent prints headers, not body).
set +e
response="$(gh api "repos/${GITHUB_REPOSITORY}/releases/tags/${tag}" --include --silent 2>&1)"
set -e
status="$(printf '%s\n' "$response" | sed -n '1s@^HTTP/[0-9.]* \([0-9]*\).*@\1@p')"

case "$status" in
  200)
    echo "error: a GitHub Release already exists for tag '$tag' -- refusing to re-publish" >&2
    exit 1
    ;;
  404)
    echo "ok: no existing release for tag '$tag'"
    ;;
  *)
    echo "error: could not determine whether a release exists for tag '$tag' (HTTP status: ${status:-none}) -- refusing to proceed" >&2
    printf '%s\n' "$response" >&2
    exit 1
    ;;
esac
