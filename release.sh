#!/usr/bin/env bash
# Tags the current commit as `v<version in Cargo.toml>` and pushes the tag,
# which triggers .github/workflows/release.yaml and docs-release.yaml.
#
# To release: bump `version` in Cargo.toml, commit, push to main, run `make release`.
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

if ! report=$(./docs/check_validation.sh); then
    echo
    echo "WARNING:"
    echo "$report" | sed 's/^::error:://'
    echo
    read -r -p "Release with stale validation reports anyway? [y/N] " answer
    [ "$answer" = "y" ] || [ "$answer" = "Y" ] || fail "aborted"
fi

echo
echo "Ready to release $tag from $(git log -1 --format='%h %s')"
read -r -p "Create and push tag $tag? [y/N] " answer
[ "$answer" = "y" ] || [ "$answer" = "Y" ] || fail "aborted"

git tag -a "$tag" -m "Release $tag"
git push origin "$tag"
echo "Pushed $tag. Follow the release at https://github.com/germolinal/simple/actions"
