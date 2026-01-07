#!/bin/bash
# Sync shared Cursor rules from rank-rank to all rank-* repositories

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RANK_RANK_DIR="$(dirname "$SCRIPT_DIR")"
WORKSPACE_ROOT="$(dirname "$RANK_RANK_DIR")"

echo "Syncing Cursor rules from rank-rank to all rank-* repositories..."
echo ""

# Find all rank-* directories
RANK_REPOS=()

# First check parent directory (standard expectation)
for dir in "$WORKSPACE_ROOT"/rank-*; do
    if [ -d "$dir" ] && [ "$(basename "$dir")" != "rank-rank" ]; then
        RANK_REPOS+=("$dir")
    fi
done

# If none found, check if we are in a nested structure (e.g. _rank-rank is the root)
if [ ${#RANK_REPOS[@]} -eq 0 ]; then
    echo "No rank-* repos found in parent. Checking current _rank-rank directory..."
    for dir in "$RANK_RANK_DIR"/rank-*; do
        if [ -d "$dir" ] && [ "$(basename "$dir")" != "rank-rank" ]; then
            RANK_REPOS+=("$dir")
        fi
    done
fi

if [ ${#RANK_REPOS[@]} -eq 0 ]; then
    echo "No rank-* repositories found in $WORKSPACE_ROOT or $RANK_RANK_DIR"
    exit 1
fi

echo "Found ${#RANK_REPOS[@]} repositories:"
for repo in "${RANK_REPOS[@]}"; do
    echo "  - $(basename "$repo")"
done
echo ""

# Source template directory
TEMPLATE_DIR="$RANK_RANK_DIR/.cursor/rules"
if [ ! -d "$TEMPLATE_DIR" ]; then
    echo "Error: Template directory not found: $TEMPLATE_DIR"
    exit 1
fi

# Sync shared-base.mdc to each repo
SYNCED=0
SKIPPED=0

for repo in "${RANK_REPOS[@]}"; do
    repo_name=$(basename "$repo")
    target_dir="$repo/.cursor/rules"
    
    echo "Processing $repo_name..."
    
    # Create .cursor/rules directory if it doesn't exist
    mkdir -p "$target_dir"
    
    # Copy shared-base.mdc
    if [ -f "$TEMPLATE_DIR/shared-base.mdc" ]; then
        cp "$TEMPLATE_DIR/shared-base.mdc" "$target_dir/shared-base.mdc"
        echo "  ✅ Copied shared-base.mdc"
        SYNCED=$((SYNCED + 1))
    else
        echo "  ⚠️  Template file not found: shared-base.mdc"
        SKIPPED=$((SKIPPED + 1))
    fi
    
    # Copy README.md if it doesn't exist
    if [ ! -f "$target_dir/README.md" ] && [ -f "$TEMPLATE_DIR/README.md" ]; then
        cp "$TEMPLATE_DIR/README.md" "$target_dir/README.md"
        echo "  ✅ Copied README.md"
    fi
done

echo ""
echo "Summary:"
echo "  ✅ Synced to $SYNCED repositories"
if [ $SKIPPED -gt 0 ]; then
    echo "  ⚠️  Skipped $SKIPPED repositories (template not found)"
fi
echo ""
echo "Next steps:"
echo "  1. Review changes in each repository"
echo "  2. Add repo-specific rules in each .cursor/rules/repo-specific.mdc"
echo "  3. Commit changes to each repository"
echo ""

