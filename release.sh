#!/usr/bin/env bash
# Releases SIMPLE in two steps (main only accepts changes through PRs):
#
#   ./release.sh prepare patch|minor|major   (`make release_patch`, etc.)
#       From an up-to-date main, bumps `version` in Cargo.toml (and
#       Cargo.lock) on a new `release/v<version>` branch, runs `make validate`,
#       commits the reports, pushes the branch and opens a PR.
#
#   ./release.sh tag                          (`make release`)
#       Once that PR is merged, tags main as `v<version in Cargo.toml>` and
#       pushes the tag, which triggers .github/workflows/release.yaml: tests,
#       a GitHub Release with binaries, and the website and docs published
#       from the tag.
#
# Both refuse to go on unless the release is set up properly: an up-to-date,
# clean main, a new and higher version, a Cargo.lock that matches it, and
# validation reports that are not older than the code.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

fail() {
    echo "ERROR: $1" >&2
    exit 1
}

confirm() {
    read -r -p "$1 [y/N] " answer
    [ "$answer" = "y" ] || [ "$answer" = "Y" ] || fail "aborted"
}

current_version() {
    sed -n '/^\[package\]/,/^\[/s/^version *= *"\(.*\)"/\1/p' Cargo.toml
}

# Fails unless on a clean main that matches origin/main
check_main() {
    branch=$(git rev-parse --abbrev-ref HEAD)
    [ "$branch" = "main" ] || fail "releases are made from 'main', but you are on '$branch'"

    [ -z "$(git status --porcelain)" ] || fail "there are uncommitted or untracked changes:
$(git status --short)"

    git fetch --quiet --tags origin main
    [ "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)" ] ||
        fail "local main and origin/main differ; pull and/or push first"
}

# Fails unless `$1` (e.g., v0.2.0) is new and higher than every release
check_new_tag() {
    if git rev-parse --quiet --verify "refs/tags/$1" >/dev/null; then
        fail "tag $1 already exists; bump the version (make release_patch, release_minor or release_major)"
    fi

    latest=$(git tag --list 'v*' --sort=-v:refname | head -1)
    if [ -n "$latest" ]; then
        highest=$(printf '%s\n%s\n' "$latest" "$1" | sort -V | tail -1)
        [ "$highest" = "$1" ] || fail "$1 is not higher than the latest release, $latest"
    fi
}

check_validation() {
    if ! report=$(./docs/check_validation.sh); then
        fail "the validation reports are stale; run 'make validate' and merge docs/validation first:
$(echo "$report" | sed 's/^::error:://')"
    fi
}

prepare() {
    part=${1:-}
    case "$part" in
    patch | minor | major) ;;
    *) fail "usage: $0 prepare patch|minor|major" ;;
    esac

    check_main

    old=$(current_version)
    [[ "$old" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)$ ]] ||
        fail "version '$old' in Cargo.toml is not MAJOR.MINOR.PATCH"
    major=${BASH_REMATCH[1]}
    minor=${BASH_REMATCH[2]}
    patch=${BASH_REMATCH[3]}
    case "$part" in
    patch) new="$major.$minor.$((patch + 1))" ;;
    minor) new="$major.$((minor + 1)).0" ;;
    major) new="$((major + 1)).0.0" ;;
    esac
    tag="v$new"
    release_branch="release/$tag"

    check_new_tag "$tag"
    if git rev-parse --quiet --verify "refs/heads/$release_branch" >/dev/null ||
        [ -n "$(git ls-remote --heads origin "$release_branch")" ]; then
        fail "branch $release_branch already exists"
    fi

    echo "This will bump $old -> $new on branch $release_branch, run 'make validate'"
    echo "(it takes a while), push the branch and open a PR."
    confirm "Continue?"

    git switch --quiet -c "$release_branch"

    # Only the `version` of the [package] section
    awk -v new="$new" '
        /^\[/ { in_package = ($0 == "[package]") }
        in_package && !done && /^version *=/ { $0 = "version = \"" new "\""; done = 1 }
        { print }
    ' Cargo.toml >Cargo.toml.tmp && mv Cargo.toml.tmp Cargo.toml
    [ "$(current_version)" = "$new" ] || fail "could not update the version in Cargo.toml"
    cargo update --quiet --workspace
    git commit --quiet -m "Bump version to $new" -- Cargo.toml Cargo.lock

    # After the bump: check_validation.sh counts Cargo.toml as code
    make validate || fail "'make validate' failed. You are on $release_branch, with the version bumped;
fix it there and re-run 'make validate', or start over with:
    git switch main && git branch -D $release_branch"
    git add docs/validation
    git commit --quiet -m "Validate $tag"
    check_validation

    git push --quiet -u origin "$release_branch"
    gh pr create --base main --head "$release_branch" --title "Release $tag" --body "Bumps the version to \`$new\` and re-runs the validation.

Once merged, run \`git switch main && git pull && make release\` to tag \`$tag\` and publish it."

    echo
    echo "Once the PR is merged: git switch main && git pull && make release"
}

tag() {
    version=$(current_version)
    [ -n "$version" ] || fail "could not read the version from Cargo.toml"
    tag="v$version"

    echo "Checking that $tag can be released..."
    check_main
    check_new_tag "$tag"

    cargo metadata --locked --format-version 1 >/dev/null 2>&1 ||
        fail "Cargo.lock is out of date (did you bump the version without building?); run 'cargo check', commit Cargo.lock"

    check_validation

    echo
    echo "Ready to release $tag from $(git log -1 --format='%h %s')"
    confirm "Create and push tag $tag?"

    git tag -a "$tag" -m "Release $tag"
    git push origin "$tag"
    echo "Pushed $tag. Follow the release at https://github.com/germolinal/simple/actions"
}

case "${1:-}" in
prepare) prepare "${2:-}" ;;
tag) tag ;;
*) fail "usage: $0 prepare patch|minor|major | $0 tag" ;;
esac
