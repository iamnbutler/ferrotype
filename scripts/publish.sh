#!/usr/bin/env bash
set -euo pipefail

# Publish script for ferrotype crates
# Usage: ./scripts/publish.sh [--dry-run]
#
# Publishes all crates to crates.io in dependency order:
# 1. ferro-type-derive (no deps)
# 2. ferro-type (depends on derive)
# 3. ferro-type-gen (depends on core)

DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN=true
    echo "Running in dry-run mode (no actual publishing)"
fi

# Crate directories (in dependency order)
CRATES=(
    "crates/ferrotype-derive"
    "crates/ferrotype"
    "crates/ferrotype-gen"
)

# Wait time between publishes for crates.io index propagation (seconds)
INDEX_WAIT=30

# Verify working directory is clean
check_git_clean() {
    if [[ -n "$(git status --porcelain)" ]]; then
        echo "Error: Working directory is not clean."
        echo "Please commit or stash your changes before publishing."
        git status --short
        exit 1
    fi
    echo "✓ Working directory is clean"
}

# Run tests
run_tests() {
    echo "Running tests..."
    cargo test --workspace
    echo "✓ All tests passed"
}

# Publish a single crate
publish_crate() {
    local crate_dir="$1"
    local crate_name
    crate_name=$(grep '^name = ' "$crate_dir/Cargo.toml" | head -1 | sed 's/name = "\(.*\)"/\1/')

    echo "Publishing $crate_name..."

    if $DRY_RUN; then
        cargo publish --dry-run -p "$crate_name"
    else
        cargo publish -p "$crate_name"
    fi

    echo "✓ Published $crate_name"
}

# Wait for crates.io index to update
wait_for_index() {
    if $DRY_RUN; then
        echo "Skipping wait (dry-run mode)"
        return
    fi

    echo "Waiting ${INDEX_WAIT}s for crates.io index to propagate..."
    sleep "$INDEX_WAIT"
}

main() {
    echo "═══════════════════════════════════════════════════════════════"
    echo "  Publishing ferrotype crates to crates.io"
    echo "═══════════════════════════════════════════════════════════════"
    echo

    check_git_clean
    echo

    run_tests
    echo

    local total=${#CRATES[@]}
    local count=0

    for crate_dir in "${CRATES[@]}"; do
        count=$((count + 1))
        echo "[$count/$total] $crate_dir"
        publish_crate "$crate_dir"

        # Wait between publishes (except for the last one)
        if [[ $count -lt $total ]]; then
            wait_for_index
        fi
        echo
    done

    echo "═══════════════════════════════════════════════════════════════"
    echo "  ✓ All crates published successfully!"
    echo "═══════════════════════════════════════════════════════════════"
}

main
