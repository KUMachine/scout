#!/usr/bin/env bash
# Cut a scout release: fold CHANGELOG [Unreleased], bump version, tag.
#
# Usage:
#   1. Add bullets under [Unreleased] in CHANGELOG.md
#   2. ./scripts/release.sh X.Y.Z
#   3. git push origin main --tags
set -euo pipefail

VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
  echo "Usage: ./scripts/release.sh X.Y.Z" >&2
  exit 1
fi

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Invalid version (expected SemVer X.Y.Z): $VERSION" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ ! -f CHANGELOG.md ]]; then
  echo "CHANGELOG.md not found" >&2
  exit 1
fi

if [[ -n "$(git status --porcelain)" ]]; then
  echo "Working tree is not clean; commit or stash changes first." >&2
  exit 1
fi

if git rev-parse "v$VERSION" >/dev/null 2>&1; then
  echo "Tag v$VERSION already exists." >&2
  exit 1
fi

CURRENT="$(awk -F'"' '/^version = / { print $2; exit }' Cargo.toml)"
if [[ "$CURRENT" == "$VERSION" ]]; then
  echo "Cargo.toml is already at $VERSION." >&2
  exit 1
fi

UNRELEASED_START="$(grep -n '^## \[Unreleased\]' CHANGELOG.md | head -1 | cut -d: -f1)"
if [[ -z "$UNRELEASED_START" ]]; then
  echo "CHANGELOG.md is missing an [Unreleased] section." >&2
  exit 1
fi

UNRELEASED_TMP="$(mktemp)"
awk -v start="$UNRELEASED_START" '
  NR == start { in_unreleased = 1; next }
  in_unreleased && /^## \[/ { exit }
  in_unreleased { print }
' CHANGELOG.md > "$UNRELEASED_TMP"

HAS_CONTENT=0
while IFS= read -r line || [[ -n "$line" ]]; do
  if [[ -n "${line//[[:space:]]/}" ]]; then
    HAS_CONTENT=1
    break
  fi
done < "$UNRELEASED_TMP"

if [[ "$HAS_CONTENT" -eq 0 ]]; then
  echo "CHANGELOG.md has nothing under [Unreleased]. Add release notes first." >&2
  exit 1
fi

DATE="$(date +%Y-%m-%d)"
TAG="v$VERSION"
CHANGELOG_NEW="$(mktemp)"

{
  head -n "$UNRELEASED_START" CHANGELOG.md
  echo
  echo "## [$VERSION] - $DATE"
  cat "$UNRELEASED_TMP"
  echo
  awk -v start="$UNRELEASED_START" '
    NR > start && /^## \[/ { print; tail = 1 }
    tail { print }
  ' CHANGELOG.md
} > "$CHANGELOG_NEW"

rm -f "$UNRELEASED_TMP"

mv "$CHANGELOG_NEW" CHANGELOG.md

if [[ "$(uname -s)" == "Darwin" ]]; then
  sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
else
  sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
fi

cargo check --quiet

for doc in README.md SECURITY.md; do
  if [[ -f "$doc" ]]; then
    if [[ "$(uname -s)" == "Darwin" ]]; then
      sed -i '' "s/--tag v[0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*/--tag $TAG/g" "$doc"
    else
      sed -i "s/--tag v[0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*/--tag $TAG/g" "$doc"
    fi
  fi
done

git add CHANGELOG.md Cargo.toml Cargo.lock README.md SECURITY.md
git commit -m "Release $TAG"

if git config --get user.signingkey >/dev/null 2>&1; then
  git tag -s "$TAG" -m "$TAG"
else
  git tag -a "$TAG" -m "$TAG"
fi

echo "Released $TAG ($VERSION)."
echo "Push with: git push origin main --tags"
