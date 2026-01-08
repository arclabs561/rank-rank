# Shared Scripts for rank-* Repositories

Scripts that work across all rank-* repositories in this workspace.

## VLM Visual Inspection

### vlm_inspect_readme.py

Visually inspect rendered README pages using Vision Language Models.

**Features:**
- Opportunistically uses API keys from environment
- Renders markdown with mdpreview (GitHub-flavored)
- Takes full-page screenshots with Playwright
- Uses Claude Vision (preferred) or GPT-4 Vision (fallback)
- Gracefully degrades if no API keys

**Usage:**
```bash
# From rank-rank
python3 scripts/vlm_inspect_readme.py ../rank-fusion/rank-fusion/README.md

# From rank-* repo
python3 ../../rank-rank/scripts/vlm_inspect_readme.py README.md
```

**Environment Variables:**
- `ANTHROPIC_API_KEY` - Claude API key (preferred)
- `OPENAI_API_KEY` - OpenAI API key (fallback)

### vlm_inspect_all_readmes.sh

Batch inspect all README files across rank-* repositories.

**Usage:**
```bash
./scripts/vlm_inspect_all_readmes.sh [output_dir]
```

**Inspects:**
- rank-fusion/rank-fusion/README.md
- rank-rerank/README.md
- rank-soft/README.md
- rank-eval/README.md

### verify_readme_viz.py

Legacy verification script (updated to be opportunistic).

**Usage:**
```bash
python3 scripts/verify_readme_viz.py <screenshot_path> <context>
```

### screenshot_readme.js

Playwright script for taking full-page screenshots.

**Usage:**
```bash
node scripts/screenshot_readme.js <url> <output_path>
```

## Dependencies

All scripts use PEP 723 inline dependencies and can be run with:
```bash
uv run <script>.py
```

## Setup

See [rank-rank/README.md](../README.md) for full setup instructions.

