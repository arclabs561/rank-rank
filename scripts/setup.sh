#!/bin/bash
# Setup script for rank-rank shared utilities
# Installs dependencies needed for VLM inspection and visualization

set -euo pipefail

echo "🔧 Setting up rank-rank shared utilities"
echo "========================================="
echo ""

# Check for mdpreview
if command -v mdpreview &> /dev/null; then
    echo "✅ mdpreview found"
else
    echo "📦 Installing mdpreview..."
    if command -v go &> /dev/null; then
        go install github.com/henrywallace/mdpreview@latest
        echo "✅ mdpreview installed"
    else
        echo "❌ Error: Go not found. Install Go to use mdpreview."
        echo "   Or install manually: go install github.com/henrywallace/mdpreview@latest"
    fi
fi

# Check for Playwright
if [ -f "scripts/node_modules/.bin/playwright" ] || command -v playwright &> /dev/null; then
    echo "✅ Playwright found"
else
    echo "📦 Installing Playwright..."
    if command -v npm &> /dev/null; then
        cd scripts
        npm install playwright
        npx playwright install chromium
        cd ..
        echo "✅ Playwright installed"
    else
        echo "❌ Error: npm not found. Install Node.js to use Playwright."
        echo "   Or install manually: cd scripts && npm install playwright"
    fi
fi

# Check for uv
if command -v uv &> /dev/null; then
    echo "✅ uv found"
else
    echo "⚠️  uv not found. Install with: curl -LsSf https://astral.sh/uv/install.sh | sh"
    echo "   Or use: pip install -r requirements.txt (if available)"
fi

# Check for API keys
echo ""
echo "API Keys (optional):"
if [ -n "${ANTHROPIC_API_KEY:-}" ]; then
    echo "✅ ANTHROPIC_API_KEY set"
else
    echo "⚠️  ANTHROPIC_API_KEY not set (VLM inspection will be limited)"
fi

if [ -n "${OPENAI_API_KEY:-}" ]; then
    echo "✅ OPENAI_API_KEY set"
else
    echo "⚠️  OPENAI_API_KEY not set (VLM inspection will be limited)"
fi

echo ""
echo "✅ Setup complete!"
echo ""
echo "Usage:"
echo "  ./scripts/inspect_all_rank_readmes.sh    # Inspect all READMEs"
echo "  python3 scripts/vlm_inspect_readme.py <readme>  # Inspect single README"

