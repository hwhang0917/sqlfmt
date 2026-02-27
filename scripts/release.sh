#!/bin/sh
set -e

CARGO_TOML="Cargo.toml"

usage() {
    printf "Usage: %s <major|minor|patch>\n\n" "$(basename "$0")"
    printf "Bump the version in Cargo.toml, commit, tag, and push to trigger a release.\n\n"
    printf "Arguments:\n"
    printf "  major   Bump major version (e.g. 0.1.0 -> 1.0.0)\n"
    printf "  minor   Bump minor version (e.g. 0.1.0 -> 0.2.0)\n"
    printf "  patch   Bump patch version (e.g. 0.1.0 -> 0.1.1)\n"
    exit 0
}

if [ $# -eq 0 ] || [ $# -gt 1 ]; then
    usage
fi

BUMP="$1"
case "$BUMP" in
    major|minor|patch) ;;
    *) usage ;;
esac

current=$(grep '^version' "$CARGO_TOML" | head -1 | sed 's/.*"\(.*\)".*/\1/')
if [ -z "$current" ]; then
    printf "error: could not read version from %s\n" "$CARGO_TOML" >&2
    exit 1
fi

IFS='.' read -r major minor patch <<EOF
$current
EOF

case "$BUMP" in
    major) major=$((major + 1)); minor=0; patch=0 ;;
    minor) minor=$((minor + 1)); patch=0 ;;
    patch) patch=$((patch + 1)) ;;
esac

next="${major}.${minor}.${patch}"
tag="v${next}"

printf "Bumping version: %s -> %s\n" "$current" "$next"

sed -i "s/^version = \"$current\"/version = \"$next\"/" "$CARGO_TOML"

git add "$CARGO_TOML"
git commit -m "Release ${tag}"
git tag "$tag" -m "Release ${tag}"
git push origin HEAD
git push origin "$tag"

printf "Pushed %s — release workflow will run at:\n" "$tag"
printf "  https://github.com/hwhang0917/sqlfmt/actions\n"
