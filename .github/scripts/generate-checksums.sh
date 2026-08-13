#!/usr/bin/env bash
# Generates a <file>.sha256 checksum file next to each given release
# artifact (FR-012). Uses sha256sum if present, falling back to
# `shasum -a 256` (macOS has no sha256sum by default) -- no additional
# dependency beyond what the runner/OS already provides, per plan.md's
# Key Design Decision #6.
#
# The checksum file references its artifact by basename, not the full
# invocation path: release assets are downloaded flat, side-by-side, so a
# checksum file baked with the build machine's absolute path would fail
# `sha256sum -c`/`shasum -a 256 -c` for every downloader.
set -euo pipefail

usage() {
  echo "usage: $0 <file> [file...]" >&2
  exit 2
}

[ $# -ge 1 ] || usage

sha_cmd() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1"
  else
    shasum -a 256 "$1"
  fi
}

for file in "$@"; do
  if [ ! -f "$file" ]; then
    echo "error: $file not found" >&2
    exit 1
  fi
  dir="$(dirname "$file")"
  base="$(basename "$file")"
  (cd "$dir" && sha_cmd "$base") > "${file}.sha256"
  echo "wrote ${file}.sha256"
done
