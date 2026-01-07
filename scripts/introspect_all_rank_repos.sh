#!/bin/bash
# Introspect all rank-* repositories to discover what's actually implemented

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RANK_RANK_DIR="$(dirname "$SCRIPT_DIR")"
WORKSPACE_ROOT="$(dirname "$RANK_RANK_DIR")"

echo "Introspecting all rank-* repositories..."
echo ""

# Find all rank-* directories
RANK_REPOS=()

# First check parent directory (standard expectation)
for dir in "$WORKSPACE_ROOT"/rank-*; do
    if [ -d "$dir" ]; then
        RANK_REPOS+=("$dir")
    fi
done

# If none found, check if we are in a nested structure (e.g. _rank-rank is the root of relevance)
if [ ${#RANK_REPOS[@]} -eq 0 ]; then
    echo "No rank-* repos found in parent. Checking current _rank-rank directory..."
    # Check if RANK_RANK_DIR itself contains the repos (which seems to be the case here)
    for dir in "$RANK_RANK_DIR"/rank-*; do
        if [ -d "$dir" ]; then
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

# Introspect each repo
for repo in "${RANK_REPOS[@]}"; do
    repo_name=$(basename "$repo")
    echo "=========================================="
    echo "Introspecting: $repo_name"
    echo "=========================================="
    
    # Check if introspection script exists for this repo type
    if [ "$repo_name" = "rank-eval" ]; then
        python3 "$SCRIPT_DIR/introspect_rank_eval.py" "$repo" 2>&1 | head -100
        echo ""
    else
        echo "No introspection script for $repo_name yet"
        echo ""
    fi
done

echo ""
echo "Summary: Use individual introspection scripts for detailed analysis."
echo "Helm design: rank-rank provides introspection tools for all rank-* repos."

