# rank-rank

Monorepo: Central repository for all rank-* crates.

Helm Design: Central control and introspection for all rank-* crates.

Main repository containing all rank-* crates, organized as a monorepo with shared tooling and coordination.

## Purpose

This directory contains tools and scripts that apply to multiple rank-* repositories:
- `rank-retrieve` - First-stage retrieval (BM25, dense ANN, sparse)
- `rank-fusion` - Rank fusion algorithms
- `rank-rerank` - Reranking and late interaction scoring (MaxSim, cross-encoder)
- `rank-soft` - Soft ranking and differentiable sorting (renamed from rank-relax)
- `rank-learn` - Learning to Rank frameworks (LambdaRank, XGBoost)
- `rank-eval` - Ranking evaluation metrics
- `rank-sparse` - Sparse vector utilities

## Helm Design

rank-rank is the central control point that can:
1. Introspect all rank-* repos to discover what's actually implemented
2. Verify documentation matches reality
3. Sync shared configuration (Cursor rules, etc.)
4. Provide shared tools (VLM inspection, visualization patterns)

Principle: You should immediately be able to introspect and tell if something is benefiting us.

## Structure

```
rank-rank/
├── scripts/              # Shared scripts
│   ├── introspect_rank_eval.py      # Discover what rank-eval actually has
│   ├── introspect_all_rank_repos.sh  # Introspect all repos
│   ├── sync_cursor_rules.sh          # Sync Cursor rules to all repos
│   ├── verify_cursor_rules.sh        # Verify Cursor rules setup
│   ├── vlm_inspect_readme.py         # VLM-based README inspection
│   └── inspect_all_rank_readmes.sh   # Inspect all READMEs
├── .cursor/
│   └── rules/            # Template for shared Cursor rules
├── hack/
│   └── viz/              # Shared visualization utilities
└── README.md             # This file
```

## Introspection System

### Discover What's Actually Implemented

```bash
# Introspect rank-eval to see all metrics, datasets, features
python3 scripts/introspect_rank_eval.py rank-eval

# Introspect all rank-* repos
./scripts/introspect_all_rank_repos.sh
```

Purpose: The README might say "supports NDCG" but the code might have 20 metrics. Introspection reveals the gap between documentation and reality.

### Example Output

```
## Metrics
Total Implemented: 20
In README: 12
Missing from README: 8

### Binary Metrics
- `ndcg_at_k()`
- `precision_at_k()`
- `err_at_k()`  # Missing from README!
- `rbp_at_k()`  # Missing from README!
...
```

## Typst Documentation

Generate PDF and HTML documentation from Typst source:

```bash
# Generate docs for all repos
./scripts/generate_typst_docs.sh

# Generate for specific repo
./scripts/generate_typst_docs.sh rank-relax
```

Output: Each repo's `docs/output/` directory contains:
- `{repo}_documentation.pdf` - PDF version
- `{repo}_documentation.html` - HTML version

Source: Each repo has `docs/main.typ` with Typst source.

See `README_TYPST.md` for details.

## Shared Tools

### VLM Visual Inspection

Opportunistically uses API keys from environment to visually inspect rendered README pages:

```bash
# Set API keys (optional)
export ANTHROPIC_API_KEY="sk-ant-..."
# or
export OPENAI_API_KEY="sk-..."

# Inspect all READMEs
./scripts/inspect_all_rank_readmes.sh
```

Features:
- Renders markdown with mdpreview (GitHub-flavored)
- Takes full-page screenshots with Playwright
- Uses VLM (Claude Vision or GPT-4 Vision) for visual critique
- Gracefully degrades if no API keys (screenshots only)

### Visualization Generation

Shared visualization scripts with statistical depth matching pre-AI quality:

```bash
# Generate visualizations for any rank-* repo
cd rank-fusion/hack/viz
uv run generate_rrf_real_data.py
```

Quality Standards:
- Real data from actual code execution
- Statistical depth (distribution fitting, hypothesis testing)
- Large sample sizes (1000+)
- Code-driven and reproducible

### Cursor Rules Management

Sync shared Cursor rules to all rank-* repositories:

```bash
cd rank-rank
./scripts/sync_cursor_rules.sh    # Copy rules to all repos
./scripts/verify_cursor_rules.sh # Verify setup
```

Each repository will have:
- `.cursor/rules/shared-base.mdc` - Shared rules (auto-synced)
- `.cursor/rules/*-specific.mdc` - Repo-specific rules

## Usage

### From rank-* Repositories

Reference shared utilities from rank-rank:

```bash
# VLM inspection
python3 ../../rank-rank/scripts/vlm_inspect_readme.py README.md

# Introspection
python3 ../../rank-rank/scripts/introspect_rank_eval.py .

# Visualization helpers
cp ../../rank-rank/hack/viz/template.py hack/viz/
```

### Standalone

Use rank-rank tools directly:

```bash
cd rank-rank
./scripts/introspect_all_rank_repos.sh
./scripts/inspect_all_rank_readmes.sh
```

## Setup

### Cursor Rules

Sync shared Cursor rules to all rank-* repositories:

```bash
cd rank-rank
./scripts/sync_cursor_rules.sh    # Copy rules to all repos
./scripts/verify_cursor_rules.sh # Verify setup
```

### Dependencies

```bash
# mdpreview (Go)
go install github.com/henrywallace/mdpreview@latest

# Playwright (Node.js)
cd scripts
npm install playwright
npx playwright install chromium

# Python dependencies (auto-installed via PEP 723)
# Just run: uv run <script>.py
```

### API Keys (Optional)

```bash
export ANTHROPIC_API_KEY="sk-ant-..."  # Preferred
export OPENAI_API_KEY="sk-..."         # Fallback
```

## Integration

### CI/CD

```yaml
- name: Introspect rank-eval
  run: |
    cd rank-rank
    python3 scripts/introspect_rank_eval.py ../rank-eval > introspection_report.md

- name: VLM Inspect READMEs
  env:
    ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
  run: |
    cd rank-rank
    ./scripts/inspect_all_rank_readmes.sh
```

## Benefits

1. Single Source of Truth: rank-rank knows what all repos have
2. Gap Detection: Immediately see what's implemented vs documented
3. Consistency: Shared tools and patterns across all repos
4. Introspection: Can verify if work is actually benefiting us
5. Centralized Control: Update once, apply everywhere

## License

MIT OR Apache-2.0 (same as rank-* repositories)
