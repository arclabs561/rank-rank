# rank-rank

Monorepo for ranking and retrieval crates in Rust.

## Crates

- **`rank-retrieve`** - First-stage retrieval: BM25, dense ANN, sparse vectors
- **`rank-fusion`** - Rank fusion: RRF, ISR, CombMNZ for hybrid search
- **`rank-rerank`** - SIMD-accelerated reranking: MaxSim (ColBERT), cosine, diversity
- **`rank-soft`** - Differentiable ranking operations for ML training
- **`rank-learn`** - Learning to Rank: LambdaRank, LambdaMART, Ranking SVM
- **`rank-eval`** - IR evaluation metrics: NDCG, MAP, MRR, TREC format

## Quick Start

```bash
# Add a crate
cargo add rank-retrieve

# Or use multiple crates in a pipeline
cargo add rank-retrieve rank-fusion rank-rerank rank-eval
```

## Documentation

- `SETUP.md` - Setup instructions
- `USAGE.md` - Usage guide
- `docs/` - Integration guides, performance, theory

Each crate has its own README and documentation. See `crates/*/README.md`.

## License

MIT OR Apache-2.0
