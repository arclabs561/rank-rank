# rank-rank

Monorepo for all rank-* crates with shared tooling and coordination.

## Crates

- `rank-retrieve` - First-stage retrieval (BM25, dense ANN, sparse)
- `rank-fusion` - Rank fusion algorithms
- `rank-rerank` - Reranking and late interaction scoring (MaxSim, cross-encoder)
- `rank-soft` - Soft ranking and differentiable sorting
- `rank-learn` - Learning to Rank frameworks (LambdaRank, XGBoost)
- `rank-eval` - Ranking evaluation metrics

## Structure

```
rank-rank/
├── crates/          # All rank-* crates (flat structure)
├── scripts/         # Shared scripts
├── docs/            # Active documentation
└── archive/         # Historical documentation
```

## Usage

```bash
# Introspect all repos
./scripts/introspect_all_rank_repos.sh

# Sync Cursor rules
./scripts/sync_cursor_rules.sh
```

## Documentation

- `SETUP.md` - Setup instructions
- `USAGE.md` - Usage guide
- `CURSOR_CONFIG.md` - Cursor IDE configuration
- `SECURITY_AUDIT.md` - Security information

Additional documentation:
- `docs/` - User guides (integration, performance, feature flags)
- `docs/theory/` - Mathematical theory
- `docs/typst/` - Typst documentation guide
- `archive/` - Historical documentation

## License

MIT OR Apache-2.0 (same as rank-* repositories)
