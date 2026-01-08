# rank-rank

Monorepo for ranking and retrieval crates in Rust.

## Crates

### Pipeline Stages

- **[`rank-retrieve`](crates/rank-retrieve/)** - First-stage retrieval: BM25, dense ANN, sparse vectors
- **[`rank-fusion`](crates/rank-fusion/)** - Rank fusion: RRF, ISR, CombMNZ for hybrid search
- **[`rank-rerank`](crates/rank-rerank/)** - SIMD-accelerated reranking: MaxSim (ColBERT), cosine, diversity
- **[`rank-eval`](crates/rank-eval/)** - IR evaluation metrics: NDCG, MAP, MRR, TREC format

### Training

- **[`rank-soft`](crates/rank-soft/)** - Differentiable ranking operations for ML training
- **[`rank-learn`](crates/rank-learn/)** - Learning to Rank: LambdaRank, LambdaMART, Ranking SVM

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

Each crate has its own README and documentation. See individual crate directories.

## License

MIT OR Apache-2.0
