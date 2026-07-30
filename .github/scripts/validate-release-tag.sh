#!/usr/bin/env bash
# Validates a release tag against Cargo.toml's canonical workspace version
# (FR-010). Previous-release-tag resolution (FR-011) is a separate concern,
# implemented in previous-release-tag.sh. See
# specs/005-ci-release-versioning/spec.md.
set -euo pipefail

usage() {
  echo "usage: $0 <tag> [cargo_toml_path]" >&2
  exit 2
}

[ $# -ge 1 ] || usage
tag="$1"
cargo_toml="${2:-Cargo.toml}"

if [ ! -f "$cargo_toml" ]; then
  echo "error: $cargo_toml not found" >&2
  exit 1
fi

if ! [[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: tag '$tag' is not a valid SemVer release tag (expected vMAJOR.MINOR.PATCH)" >&2
  exit 1
fi

workspace_version="$(awk '
  /^[[:space:]]*\[workspace\.package\]/ { in_section = 1; next }
  /^[[:space:]]*\[/ { in_section = 0 }
  in_section && /^[[:space:]]*version[[:space:]]*=/ {
    match($0, /"[^"]+"/)
    print substr($0, RSTART + 1, RLENGTH - 2)
    exit
  }
' "$cargo_toml")"

if [ -z "$workspace_version" ]; then
  echo "error: could not find [workspace.package].version in $cargo_toml" >&2
  exit 1
fi

expected_tag="v${workspace_version}"

if [ "$tag" != "$expected_tag" ]; then
  echo "error: tag/version mismatch: tag is '$tag', but [workspace.package].version is '$workspace_version' (expected tag '$expected_tag')" >&2
  exit 1
fi

echo "ok: tag '$tag' matches workspace version '$workspace_version'"
