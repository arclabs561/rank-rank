#!/bin/bash
# Generate Typst documentation (PDF and HTML) for all rank-* repositories
#
# Usage: ./scripts/generate_typst_docs.sh [repo-name]
#   If repo-name is provided, only generate for that repo
#   Otherwise, generate for all rank-* repos

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RANK_RANK_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DEV_DIR="$(cd "$RANK_RANK_DIR/.." && pwd)"

# Check if typst is installed
if ! command -v typst &> /dev/null; then
    echo "❌ typst not found. Install with: cargo install typst-cli"
    echo "   Or: brew install typst (on macOS)"
    exit 1
fi

# Repositories to process
REPOS=("rank-fusion" "rank-refine" "rank-relax" "rank-eval")

# If repo name provided, only process that one
if [ $# -gt 0 ]; then
    REPOS=("$1")
fi

for REPO in "${REPOS[@]}"; do
    REPO_DIR="$DEV_DIR/$REPO"
    
    if [ ! -d "$REPO_DIR" ]; then
        echo "⚠️  Skipping $REPO: directory not found"
        continue
    fi
    
    echo "📚 Generating Typst docs for $REPO..."
    
    # Check if typst source exists
    TYPST_SRC="$REPO_DIR/docs/main.typ"
    if [ ! -f "$TYPST_SRC" ]; then
        echo "⚠️  No Typst source found at $TYPST_SRC, skipping..."
        continue
    fi
    
    # Create output directory
    OUTPUT_DIR="$REPO_DIR/docs/output"
    mkdir -p "$OUTPUT_DIR"
    
    # Generate PDF
    echo "  → Generating PDF..."
    typst compile "$TYPST_SRC" "$OUTPUT_DIR/${REPO}_documentation.pdf" || {
        echo "❌ Failed to generate PDF for $REPO"
        continue
    }
    
    # Generate HTML from Typst source
    echo "  → Generating HTML..."
    python3 "$RANK_RANK_DIR/scripts/typst_to_html.py" \
        "$TYPST_SRC" \
        "$OUTPUT_DIR/${REPO}_documentation.html" || {
        echo "⚠️  HTML generation failed"
    }
    
    echo "✅ Generated docs for $REPO:"
    echo "   PDF: $OUTPUT_DIR/${REPO}_documentation.pdf"
    if [ -f "$OUTPUT_DIR/${REPO}_documentation.html" ]; then
        echo "   HTML: $OUTPUT_DIR/${REPO}_documentation.html"
    fi
    
done

echo ""
echo "🎉 Documentation generation complete!"

