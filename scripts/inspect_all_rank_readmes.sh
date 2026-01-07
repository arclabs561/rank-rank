#!/bin/bash
# Visually inspect all README files across all rank-* repositories
# Uses shared utilities from rank-rank

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RANK_RANK_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$RANK_RANK_DIR/.." && pwd)"

OUTPUT_DIR="${1:-$REPO_ROOT/readme_vlm_inspections}"

echo "🔍 VLM Inspection of All rank-* READMEs"
echo "========================================"
echo ""

# Check for API keys
if [ -n "${ANTHROPIC_API_KEY:-}" ]; then
    echo "✅ ANTHROPIC_API_KEY found"
else
    echo "⚠️  ANTHROPIC_API_KEY not set"
fi

if [ -n "${OPENAI_API_KEY:-}" ]; then
    echo "✅ OPENAI_API_KEY found"
else
    echo "⚠️  OPENAI_API_KEY not set"
fi

if [ -z "${ANTHROPIC_API_KEY:-}" ] && [ -z "${OPENAI_API_KEY:-}" ]; then
    echo ""
    echo "⚠️  No API keys found. Will only generate screenshots."
    echo "   Set ANTHROPIC_API_KEY or OPENAI_API_KEY to enable VLM inspection."
    echo ""
fi

echo ""

# Discover all rank-* repositories dynamically
mkdir -p "$OUTPUT_DIR"

# Find all rank-* directories (check both parent and current directory)
RANK_REPOS=()
for dir in "$REPO_ROOT"/rank-* "$RANK_RANK_DIR"/rank-*; do
    if [ -d "$dir" ] && [ "$(basename "$dir")" != "rank-rank" ]; then
        # Avoid duplicates
        if [[ ! " ${RANK_REPOS[@]} " =~ " ${dir} " ]]; then
            RANK_REPOS+=("$dir")
        fi
    fi
done

# Find all README.md files in discovered repos
readmes=()
for repo in "${RANK_REPOS[@]}"; do
    # Find README.md files (both root and nested, maxdepth 3 to avoid going too deep)
    mapfile -t found_readmes < <(find "$repo" -maxdepth 3 -name "README.md" -type f 2>/dev/null | sort)
    for readme in "${found_readmes[@]}"; do
        readmes+=("$readme")
    done
done

if [ ${#readmes[@]} -eq 0 ]; then
    echo "⚠️  No README.md files found in rank-* repositories"
    exit 1
fi

echo "Found ${#readmes[@]} README files across ${#RANK_REPOS[@]} repositories"
echo ""

for readme in "${readmes[@]}"; do
    # Extract repo name from path
    repo_name=$(basename "$(dirname "$readme")")
    
    # Handle nested structure (e.g., rank-fusion/rank-fusion/README.md)
    parent_dir=$(dirname "$(dirname "$readme")")
    parent_name=$(basename "$parent_dir")
    
    if [ "$repo_name" = "$parent_name" ]; then
        # Nested structure: use parent name with "-core" suffix
        repo_name="${parent_name}-core"
    else
        # Root README: use parent directory name
        repo_name="$parent_name"
    fi
    
    echo "📄 Inspecting: $repo_name"
    echo "   File: $readme"
    
    python3 "$SCRIPT_DIR/vlm_inspect_readme.py" "$readme" "$OUTPUT_DIR/$repo_name"
    
    echo ""
done

echo "✅ All READMEs inspected!"
echo "📁 Results in: $OUTPUT_DIR"

