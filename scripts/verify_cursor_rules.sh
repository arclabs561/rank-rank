#!/bin/bash
# Verify Cursor rules are properly set up in all rank-* repositories

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RANK_RANK_DIR="$(dirname "$SCRIPT_DIR")"
WORKSPACE_ROOT="$(dirname "$RANK_RANK_DIR")"

echo "Verifying Cursor rules setup in all rank-* repositories..."
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

# Check each repository
PASSED=0
FAILED=0
WARNINGS=0

for repo in "${RANK_REPOS[@]}"; do
    repo_name=$(basename "$repo")
    rules_dir="$repo/.cursor/rules"
    
    echo "Checking $repo_name..."
    
    # Check if .cursor/rules directory exists
    if [ ! -d "$rules_dir" ]; then
        echo "  ❌ Missing .cursor/rules/ directory"
        FAILED=$((FAILED + 1))
        continue
    fi
    
    # Check for shared-base.mdc
    if [ ! -f "$rules_dir/shared-base.mdc" ]; then
        echo "  ❌ Missing shared-base.mdc"
        FAILED=$((FAILED + 1))
        continue
    fi
    
    # Validate frontmatter in shared-base.mdc
    if ! grep -q "^---$" "$rules_dir/shared-base.mdc" || ! grep -q "^alwaysApply:" "$rules_dir/shared-base.mdc"; then
        echo "  ⚠️  shared-base.mdc missing proper frontmatter"
        WARNINGS=$((WARNINGS + 1))
    else
        echo "  ✅ shared-base.mdc exists with frontmatter"
    fi
    
    # Check for repo-specific rules (optional but recommended)
    # Try different naming patterns
    found_specific=false
    for pattern in "${repo_name}-specific.mdc" "${repo_name%-*}-specific.mdc" "*-specific.mdc"; do
        if ls "$rules_dir"/$pattern 2>/dev/null | grep -q .; then
            found_specific=true
            break
        fi
    done
    
    if [ "$found_specific" = true ]; then
        echo "  ✅ Repo-specific rules found"
    else
        echo "  ⚠️  No repo-specific rules found (optional)"
        WARNINGS=$((WARNINGS + 1))
    fi
    
    PASSED=$((PASSED + 1))
    echo ""
done

echo "Summary:"
echo "  ✅ Passed: $PASSED repositories"
if [ $FAILED -gt 0 ]; then
    echo "  ❌ Failed: $FAILED repositories"
fi
if [ $WARNINGS -gt 0 ]; then
    echo "  ⚠️  Warnings: $WARNINGS"
fi
echo ""

if [ $FAILED -eq 0 ]; then
    echo "✅ All repositories have Cursor rules properly configured!"
    exit 0
else
    echo "❌ Some repositories need attention. Run sync_cursor_rules.sh to fix."
    exit 1
fi

