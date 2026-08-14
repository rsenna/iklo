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

# Detect the available hash tool once, not per-file, and fail fast with a
# clear message if neither exists (rather than each file silently hitting
# a "command not found" only when it's its turn to be hashed).
if command -v sha256sum >/dev/null 2>&1; then
  sha_bin=(sha256sum)
elif command -v shasum >/dev/null 2>&1; then
  sha_bin=(shasum -a 256)
else
  echo "error: neither sha256sum nor shasum found" >&2
  exit 1
fi

for file in "$@"; do
  if [ ! -f "$file" ]; then
    echo "error: $file not found" >&2
    exit 1
  fi
  dir="$(dirname "$file")"
  base="$(basename "$file")"
  # `--`: without it, a basename starting with `-` (e.g. `-artifact`) would
  # be parsed as an option by sha256sum/shasum instead of a filename.
  (cd "$dir" && "${sha_bin[@]}" -- "$base") > "${file}.sha256"
  echo "wrote ${file}.sha256"
done
