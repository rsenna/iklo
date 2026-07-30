#!/usr/bin/env bash
# Computes the deterministic build identifier for a release run (FR-005):
# GITHUB_RUN_NUMBER.GITHUB_RUN_ATTEMPT, both required to be present and
# numeric so "strictly increase" can later be evaluated numerically as the
# pair (GITHUB_RUN_NUMBER, GITHUB_RUN_ATTEMPT) (SC-003).
set -euo pipefail

run_number="${GITHUB_RUN_NUMBER:-}"
run_attempt="${GITHUB_RUN_ATTEMPT:-}"

if [ -z "$run_number" ] || [ -z "$run_attempt" ]; then
  echo "error: GITHUB_RUN_NUMBER and GITHUB_RUN_ATTEMPT must both be set" >&2
  exit 1
fi

if ! [[ "$run_number" =~ ^[0-9]+$ ]] || ! [[ "$run_attempt" =~ ^[0-9]+$ ]]; then
  echo "error: GITHUB_RUN_NUMBER ('$run_number') and GITHUB_RUN_ATTEMPT ('$run_attempt') must both be base-10 integers" >&2
  exit 1
fi

echo "${run_number}.${run_attempt}"
