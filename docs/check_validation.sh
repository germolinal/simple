#!/usr/bin/env bash
# Fails if the validation reports in docs/validation are stale, i.e., if code
# that can change simulation results was committed after the last full
# validation. A full `make validate` writes docs/validation/.last_validated;
# commit it together with the reports. (Reports that come out byte-identical
# cannot be re-committed, so the stamp, not each report, marks the validation.)
#
# Usage: docs/check_validation.sh [--warn] [ref]   (ref defaults to HEAD; needs full history)
#
# With --warn, stale reports are reported as warnings and the script succeeds.
set -euo pipefail

level=error
if [ "${1:-}" = "--warn" ]; then
    level=warning
    shift
fi
ref=${1:-HEAD}
cd "$(git rev-parse --show-toplevel)"

stamp=docs/validation/.last_validated

# Anything that can change simulation results
sources=(':(glob)**/*.rs' ':(glob)**/Cargo.toml' 'Cargo.lock' ':(glob)**/tests/**')

reports=$(git ls-tree -r --name-only "$ref" -- docs/validation | grep '\.html$' || true)
if [ -z "$reports" ]; then
    echo "::error::No validation reports found in docs/validation at $ref"
    exit 1
fi

last=$(git log -1 --format=%H "$ref" -- "$stamp")
if [ -z "$last" ]; then
    echo "::$level::Validation reports were never stamped ($stamp is not committed at $ref)."
    echo "Run 'make validate' and commit docs/validation."
    [ "$level" = "warning" ] || exit 1
    exit 0
fi

echo "Last full validation: $(git show "$last:$stamp")"
changes=$(git log --format='    %h %cs %s' "$last..$ref" -- "${sources[@]}")
if [ -n "$changes" ]; then
    echo "::$level::Validation reports are stale. Code changed since they were last validated:"
    echo "$changes" | head -10
    echo "Run 'make validate' and commit docs/validation."
    [ "$level" = "warning" ] || exit 1
else
    echo "OK: no code changes since then."
fi
