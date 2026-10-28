#!/usr/bin/env bash
# Tags the current commit as `v<version in Cargo.toml>` and pushes the tag,
# which triggers .github/workflows/release.yaml: tests, a GitHub Release with
# binaries, and the website and docs published from the tag.
#
# To release: bump `version` in Cargo.toml, run `cargo check` (updates
# Cargo.lock) and then `make validate` (it must come after any code or
# Cargo.toml change), merge all that to main through a PR, pull, and run
# `make release`.
#
# Refuses to tag unless the release is set up properly: on an up-to-date,
# clean main, with a new and higher version, a Cargo.lock that matches it, and
# validation reports that are not older than the code.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

fail() {
    echo "ERROR: $1" >&2
    exit 1
}

version=$(sed -n '/^\[package\]/,/^\[/s/^version *= *"\(.*\)"/\1/p' Cargo.toml)
[ -n "$version" ] || fail "could not read the version from Cargo.toml"
tag="v$version"

echo "Checking that $tag can be released..."

branch=$(git rev-parse --abbrev-ref HEAD)
[ "$branch" = "main" ] || fail "releases are made from 'main', but you are on '$branch'"

[ -z "$(git status --porcelain)" ] || fail "there are uncommitted or untracked changes:
$(git status --short)"

git fetch --quiet --tags origin main
[ "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)" ] ||
    fail "local main and origin/main differ; pull and/or push first"

if git rev-parse --quiet --verify "refs/tags/$tag" >/dev/null; then
    fail "tag $tag already exists; bump the version in Cargo.toml"
fi

latest=$(git tag --list 'v*' --sort=-v:refname | head -1)
if [ -n "$latest" ]; then
    highest=$(printf '%s\n%s\n' "$latest" "$tag" | sort -V | tail -1)
    [ "$highest" = "$tag" ] || fail "$tag is not higher than the latest release, $latest"
fi

cargo metadata --locked --format-version 1 >/dev/null 2>&1 ||
    fail "Cargo.lock is out of date (did you bump the version without building?); run 'cargo check', commit Cargo.lock"

if ! report=$(./docs/check_validation.sh); then
    fail "the validation reports are stale; run 'make validate' and merge docs/validation first:
$(echo "$report" | sed 's/^::error:://')"
fi

echo
echo "Ready to release $tag from $(git log -1 --format='%h %s')"
read -r -p "Create and push tag $tag? [y/N] " answer
[ "$answer" = "y" ] || [ "$answer" = "Y" ] || fail "aborted"

git tag -a "$tag" -m "Release $tag"
git push origin "$tag"
echo "Pushed $tag. Follow the release at https://github.com/germolinal/simple/actions"
