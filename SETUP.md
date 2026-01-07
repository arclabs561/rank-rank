# rank-rank Setup Guide

Complete setup guide for rank-rank shared utilities.

## Quick Start

```bash
cd rank-rank
./scripts/setup.sh
```

## What Gets Set Up

1. **mdpreview** - Markdown rendering (Go)
2. **Playwright** - Screenshot generation (Node.js)
3. **uv** - Python dependency management (checked)
4. **API Keys** - Environment variable check (optional)

## Cursor Configuration

### Workspace-Level Rules

The `.cursorrules` file in rank-rank applies to all rank-* repositories when:
- Opening the workspace root
- Opening individual repos (with repo-specific overrides)

### Structure

```
rank-rank/
├── .cursorrules          # Shared rules
└── .cursor/              # Additional config
    └── README.md
```

### Usage

**Workspace mode** (recommended):
```bash
# Open workspace root
cd /Users/arc/Documents/dev
# Cursor uses rank-rank/.cursorrules automatically
```

**Individual repo mode**:
```bash
# Open individual repo
cd rank-fusion
# Cursor uses rank-rank/.cursorrules + repo-specific rules
```

## Shared Tools

### VLM Visual Inspection

```bash
# Set API keys (optional)
export ANTHROPIC_API_KEY="sk-ant-..."

# Inspect all READMEs
./scripts/inspect_all_rank_readmes.sh
```

### Visualization Generation

Each rank-* repo has its own visualization scripts in `hack/viz/`, but they follow shared patterns documented in `rank-rank/hack/viz/`.

## Dependencies

### Required

- **Go**: For mdpreview
- **Node.js**: For Playwright
- **Python 3.8+**: For scripts (uv handles dependencies)

### Optional

- **API Keys**: For VLM inspection (ANTHROPIC_API_KEY or OPENAI_API_KEY)

## Verification

After setup, verify everything works:

```bash
# Check tools
mdpreview --version
node --version
uv --version

# Test VLM inspection (without API keys - screenshots only)
python3 scripts/vlm_inspect_readme.py ../rank-fusion/rank-fusion/README.md
```

## Troubleshooting

See [USAGE.md](USAGE.md) for detailed troubleshooting.

## Next Steps

1. ✅ Run `./scripts/setup.sh`
2. ✅ Set API keys (optional)
3. ✅ Test VLM inspection
4. ✅ Generate visualizations in rank-* repos

