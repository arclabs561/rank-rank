# Using rank-rank Shared Utilities

## Quick Start

### 1. Setup

```bash
cd rank-rank
./scripts/setup.sh
```

This installs:
- mdpreview (Go tool for markdown rendering)
- Playwright (Node.js for screenshots)
- Checks for uv (Python dependency management)

### 2. Set API Keys (Optional)

```bash
export ANTHROPIC_API_KEY="sk-ant-..."  # Preferred
export OPENAI_API_KEY="sk-..."          # Fallback
```

**Note**: API keys are optional. Tools work without them (screenshots only).

### 3. Use Tools

```bash
# Inspect all READMEs across rank-* repos
./scripts/inspect_all_rank_readmes.sh

# Inspect single README
python3 scripts/vlm_inspect_readme.py ../rank-fusion/rank-fusion/README.md
```

## From rank-* Repositories

Reference shared utilities from your rank-* repo:

```bash
# From rank-fusion
python3 ../../rank-rank/scripts/vlm_inspect_readme.py README.md

# From rank-refine
python3 ../../rank-rank/scripts/vlm_inspect_readme.py README.md
```

## Tools Available

### VLM Visual Inspection

**vlm_inspect_readme.py**
- Renders markdown with mdpreview
- Takes full-page screenshots
- Uses VLM for visual critique (if API keys available)
- Gracefully degrades without keys

**inspect_all_rank_readmes.sh**
- Batch inspects all rank-* READMEs
- Organizes output by repo name
- Works with or without API keys

### Legacy Tools

**verify_readme_viz.py**
- Updated to be opportunistic
- Backward compatible
- Uses environment API keys

## Output Structure

```
readme_vlm_inspections/
├── rank-fusion-core/
│   ├── README_screenshot.png
│   └── README_vlm_critique.txt
├── rank-refine-core/
│   ├── README_screenshot.png
│   └── README_vlm_critique.txt
└── ...
```

## Integration Examples

### CI/CD

```yaml
- name: VLM Inspect READMEs
  env:
    ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
  run: |
    cd rank-rank
    ./scripts/inspect_all_rank_readmes.sh
```

### Local Development

```bash
# Quick check (screenshots only)
./scripts/inspect_all_rank_readmes.sh

# Full inspection (with VLM)
export ANTHROPIC_API_KEY="sk-ant-..."
./scripts/inspect_all_rank_readmes.sh
```

## Troubleshooting

### mdpreview not found
```bash
go install github.com/henrywallace/mdpreview@latest
```

### Playwright not installed
```bash
cd rank-rank/scripts
npm install playwright
npx playwright install chromium
```

### API key errors
- Check key is set: `echo $ANTHROPIC_API_KEY`
- Verify format (Claude: `sk-ant-...`, OpenAI: `sk-...`)
- Check network connectivity

## Cost

- **Screenshots**: Free (local generation)
- **VLM API**: ~$0.01-0.05 per README
- **6 READMEs**: ~$0.06-0.30 per run

## See Also

- [rank-rank/README.md](README.md) - Overview
- [rank-rank/scripts/README.md](scripts/README.md) - Script documentation

