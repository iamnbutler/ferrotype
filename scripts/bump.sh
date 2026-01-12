#!/usr/bin/env bash
set -euo pipefail

# Version bump script for ferrotype crates
# Usage: ./scripts/bump.sh patch|minor|major

BUMP_TYPE="${1:-}"

if [[ -z "$BUMP_TYPE" ]] || [[ ! "$BUMP_TYPE" =~ ^(patch|minor|major)$ ]]; then
    echo "Usage: $0 patch|minor|major"
    exit 1
fi

# Crate paths (in dependency order)
DERIVE_TOML="crates/ferrotype-derive/Cargo.toml"
FERROTYPE_TOML="crates/ferrotype/Cargo.toml"
GEN_TOML="crates/ferrotype-gen/Cargo.toml"

# Get current version from ferro-type-derive (source of truth)
get_current_version() {
    grep '^version = ' "$DERIVE_TOML" | head -1 | sed 's/version = "\(.*\)"/\1/'
}

# Calculate new version
bump_version() {
    local current="$1"
    local bump_type="$2"

    IFS='.' read -r major minor patch <<< "$current"

    case "$bump_type" in
        major)
            echo "$((major + 1)).0.0"
            ;;
        minor)
            echo "$major.$((minor + 1)).0"
            ;;
        patch)
            echo "$major.$minor.$((patch + 1))"
            ;;
    esac
}

# Update version in a Cargo.toml file's [package] section
update_package_version() {
    local file="$1"
    local new_version="$2"

    sed -i '' "s/^version = \"[0-9]*\.[0-9]*\.[0-9]*\"/version = \"$new_version\"/" "$file"
}

# Update a dependency version in Cargo.toml
update_dependency_version() {
    local file="$1"
    local dep_name="$2"
    local new_version="$3"

    # Match: dep_name = { version = "x.y.z", ...
    sed -i '' "s/\($dep_name = { version = \"\)[0-9]*\.[0-9]*\.[0-9]*/\1$new_version/" "$file"
}

CURRENT_VERSION=$(get_current_version)
NEW_VERSION=$(bump_version "$CURRENT_VERSION" "$BUMP_TYPE")

echo "Bumping version: $CURRENT_VERSION -> $NEW_VERSION"

# Update package versions in all crates
echo "Updating package versions..."
update_package_version "$DERIVE_TOML" "$NEW_VERSION"
update_package_version "$FERROTYPE_TOML" "$NEW_VERSION"
update_package_version "$GEN_TOML" "$NEW_VERSION"

# Update inter-crate dependency versions
echo "Updating dependency versions..."
update_dependency_version "$FERROTYPE_TOML" "ferro-type-derive" "$NEW_VERSION"
update_dependency_version "$GEN_TOML" "ferro-type" "$NEW_VERSION"

echo "Creating git commit and tag..."
git add "$DERIVE_TOML" "$FERROTYPE_TOML" "$GEN_TOML"
git commit -m "Bump all crates to v$NEW_VERSION"
git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION"

echo "Done! Version bumped to v$NEW_VERSION"
echo "Run 'git push && git push --tags' to publish"
