#!/usr/bin/env bash
set -euo pipefail

# Release script for trancelatorRT
# Usage: ./scripts/release.sh <version>
# Example: ./scripts/release.sh 0.1.0

VERSION="${1:?Usage: $0 <version> (e.g. 0.1.0)}"
TAG="v${VERSION}"

echo "=== trancelatorRT Release ${TAG} ==="

# Validate version format
if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$'; then
    echo "Error: Invalid version format. Use semver (e.g. 0.1.0, 1.0.0-beta.1)"
    exit 1
fi

# Check clean working directory
if [ -n "$(git status --porcelain)" ]; then
    echo "Error: Working directory is not clean. Commit or stash changes first."
    exit 1
fi

# Check on main branch
BRANCH=$(git branch --show-current)
if [ "$BRANCH" != "main" ]; then
    echo "Error: Must be on main branch (currently on ${BRANCH})"
    exit 1
fi

# Update version in tauri.conf.json
echo "Updating version to ${VERSION}..."
sed -i "s/\"version\": \".*\"/\"version\": \"${VERSION}\"/" src-tauri/tauri.conf.json

# Update version in Cargo.toml
sed -i "s/^version = \".*\"/version = \"${VERSION}\"/" src-tauri/Cargo.toml

# Update version in package.json
sed -i "s/\"version\": \".*\"/\"version\": \"${VERSION}\"/" package.json

# Run tests
echo "Running Rust tests..."
(cd src-tauri && cargo test)

echo "Running frontend tests..."
npm test

echo "TypeScript check..."
npx tsc --noEmit

# Build frontend
echo "Building frontend..."
npm run build

# Commit version bump
git add src-tauri/tauri.conf.json src-tauri/Cargo.toml package.json
git commit -m "chore: bump version to ${VERSION}"

# Create tag
echo "Creating tag ${TAG}..."
git tag -a "$TAG" -m "Release ${TAG}"

echo ""
echo "=== Release ${TAG} prepared ==="
echo ""
echo "Next steps:"
echo "  1. Review: git log --oneline -5"
echo "  2. Push:   git push && git push --tags"
echo "  3. GitHub Actions will build APK and create the release"
echo ""
