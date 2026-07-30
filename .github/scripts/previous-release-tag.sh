#!/usr/bin/env bash
# Resolves the nearest reachable prior SemVer release tag from
# <current_tag> (FR-011). Prints the previous tag on stdout if one exists;
# prints nothing and exits 0 if this is the first release (FR-009's
# fallback path decides what to do with an empty result -- this script
# only reports the fact).
set -euo pipefail

usage() {
  echo "usage: $0 <current_tag>" >&2
  exit 2
}

[ $# -ge 1 ] || usage
current_tag="$1"

if ! git rev-parse -q --verify "refs/tags/${current_tag}" >/dev/null; then
  echo "error: tag '$current_tag' does not exist in this repository" >&2
  exit 1
fi

git describe --tags --match "v[0-9]*" --abbrev=0 "${current_tag}^" 2>/dev/null || true
