#!/usr/bin/env bash
# Generates release-notes body text for <current_tag> (FR-006). Groups
# commit subjects in previous_release_tag..current_tag by conventional-
# commit prefix (feat/fix/docs/chore), with a fallback bucket for anything
# else (FR-007). previous_release_tag comes from T005's
# previous-release-tag.sh (FR-011); when it's empty (first release, no
# previous tag), falls back to the full history reachable from
# current_tag (FR-009).
#
# No bash arrays: macOS's default /usr/bin/env bash resolves to bash 3.2,
# which raises "unbound variable" on an empty array expansion under
# `set -u`. Buckets are plain temp files instead -- portable across bash
# 3.2 (local/macOS) and the GitHub Actions runner's bash 5.
set -euo pipefail

usage() {
  echo "usage: $0 <current_tag>" >&2
  exit 2
}

[ $# -ge 1 ] || usage
current_tag="$1"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
previous_tag="$("$script_dir/previous-release-tag.sh" "$current_tag")"

if [ -n "$previous_tag" ]; then
  range="${previous_tag}..${current_tag}"
else
  range="$current_tag"
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

: > "$tmp/feat"
: > "$tmp/fix"
: > "$tmp/docs"
: > "$tmp/chore"
: > "$tmp/other"

# `--`: without it, a first-release history that happens to collide with a
# path in the working tree (e.g. a tag "v1.0.0" and a directory
# "v1.0.0/") makes git treat the argument as ambiguous and fail; `--`
# forces it to be read as a revision. The previous..current range form is
# immune (a `..` argument is never path-ambiguous) but costs nothing to
# guard uniformly.
#
# `!` variants (feat!:, fix(scope)!:, ...) are Conventional Commits'
# breaking-change marker -- grouped with their non-breaking counterparts
# rather than a fallback bucket, since they're still feat/fix/etc in
# intent; `${line#*: }` already strips the `!` along with the rest of the
# prefix correctly.
git log --format=%s "$range" -- | while IFS= read -r line; do
  [ -z "$line" ] && continue
  case "$line" in
    feat:*|feat\(*\):*|feat!:*|feat\(*\)!:*)     echo "- ${line#*: }" >> "$tmp/feat" ;;
    fix:*|fix\(*\):*|fix!:*|fix\(*\)!:*)         echo "- ${line#*: }" >> "$tmp/fix" ;;
    docs:*|docs\(*\):*|docs!:*|docs\(*\)!:*)     echo "- ${line#*: }" >> "$tmp/docs" ;;
    chore:*|chore\(*\):*|chore!:*|chore\(*\)!:*) echo "- ${line#*: }" >> "$tmp/chore" ;;
    *)                                          echo "- $line" >> "$tmp/other" ;;
  esac
done

print_section() {
  local title="$1" file="$2"
  [ -s "$file" ] || return 0
  echo "### $title"
  cat "$file"
  echo
}

if [ ! -s "$tmp/feat" ] && [ ! -s "$tmp/fix" ] && [ ! -s "$tmp/docs" ] \
  && [ ! -s "$tmp/chore" ] && [ ! -s "$tmp/other" ]; then
  echo "No changes since the previous release."
  exit 0
fi

print_section "Features" "$tmp/feat"
print_section "Fixes" "$tmp/fix"
print_section "Documentation" "$tmp/docs"
print_section "Chores" "$tmp/chore"
print_section "Other Changes" "$tmp/other"
