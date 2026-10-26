#!/usr/bin/env bash
# Fails if any validation report in docs/validation is stale, i.e., if code
# that can change simulation results was committed after the report was last
# updated. Run `make validate` and commit docs/validation to fix it.
#
# Usage: docs/check_validation.sh [ref]   (defaults to HEAD; needs full history)
set -euo pipefail

ref=${1:-HEAD}
cd "$(git rev-parse --show-toplevel)"

# Anything that can change simulation results
sources=(':(glob)**/*.rs' ':(glob)**/Cargo.toml' 'Cargo.lock' ':(glob)**/tests/**')

reports=$(git ls-tree -r --name-only "$ref" -- docs/validation | grep '\.html$' || true)
if [ -z "$reports" ]; then
    echo "::error::No validation reports found in docs/validation at $ref"
    exit 1
fi

stale=0
for report in $reports; do
    last=$(git log -1 --format=%H "$ref" -- "$report")
    changes=$(git log --format='    %h %cs %s' "$last..$ref" -- "${sources[@]}")
    if [ -n "$changes" ]; then
        echo "::error file=$report::$report is stale (last updated $(git log -1 --format=%cs "$last")). Code changed since:"
        echo "$changes" | head -10
        stale=1
    else
        echo "OK  $report (last updated $(git log -1 --format=%cs "$last"))"
    fi
done

if [ "$stale" -ne 0 ]; then
    echo "Validation reports are stale: run 'make validate' and commit docs/validation."
    exit 1
fi
