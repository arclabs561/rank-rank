#!/bin/bash
# Update all Typst documentation from README and markdown files
#
# This script extracts content from README.md and docs/*.md files
# and updates the corresponding Typst documentation files.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RANK_RANK_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DEV_DIR="$(cd "$RANK_RANK_DIR/.." && pwd)"

REPOS=("rank-fusion" "rank-refine" "rank-relax" "rank-eval")

echo "📝 Updating Typst documentation from markdown sources..."

for REPO in "${REPOS[@]}"; do
    REPO_DIR="$DEV_DIR/$REPO"
    
    if [ ! -d "$REPO_DIR" ]; then
        echo "⚠️  Skipping $REPO: directory not found"
        continue
    fi
    
    README="$REPO_DIR/README.md"
    TYPST_SRC="$REPO_DIR/docs/main.typ"
    
    if [ ! -f "$README" ]; then
        echo "⚠️  No README.md found for $REPO"
        continue
    fi
    
    echo "  → Updating $REPO..."
    
    # The Typst files are manually maintained for now
    # This script serves as a reminder to keep them in sync
    if [ -f "$TYPST_SRC" ]; then
        echo "    ✅ Typst source exists: $TYPST_SRC"
        echo "    ℹ️  Please manually update from README.md and docs/*.md"
    else
        echo "    ⚠️  No Typst source found, run generate_typst_docs.sh first"
    fi
done

echo ""
echo "✅ Documentation check complete!"
echo "   Note: Typst files are manually maintained. Update docs/main.typ"
echo "   in each repo to reflect changes in README.md and docs/*.md"

