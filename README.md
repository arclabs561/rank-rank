# rank-rank

Monorepo for ranking and retrieval crates in Rust.

[![CI](https://github.com/arclabs561/rank-rank/actions/workflows/ci.yml/badge.svg)](https://github.com/arclabs561/rank-rank/actions)

## Crates

### Pipeline Stages

- **[`rank-retrieve`](crates/rank-retrieve/)** - First-stage retrieval: BM25, dense ANN, sparse vectors
  - [crates.io](https://crates.io/crates/rank-retrieve) | [docs.rs](https://docs.rs/rank-retrieve) | [PyPI](https://pypi.org/project/rank-retrieve/)
- **[`rank-fusion`](crates/rank-fusion/)** - Rank fusion: RRF, ISR, CombMNZ for hybrid search
  - [crates.io](https://crates.io/crates/rank-fusion) | [docs.rs](https://docs.rs/rank-fusion) | [PyPI](https://pypi.org/project/rank-fusion/) | [npm](https://www.npmjs.com/package/@arclabs561/rank-fusion)
- **[`rank-rerank`](crates/rank-rerank/)** - SIMD-accelerated reranking: MaxSim (ColBERT), cosine, diversity
  - [crates.io](https://crates.io/crates/rank-rerank) | [docs.rs](https://docs.rs/rank-rerank) | [PyPI](https://pypi.org/project/rank-rerank/) | [npm](https://www.npmjs.com/package/@arclabs561/rank-rerank)
- **[`rank-eval`](crates/rank-eval/)** - IR evaluation metrics: NDCG, MAP, MRR, TREC format
  - [crates.io](https://crates.io/crates/rank-eval) | [docs.rs](https://docs.rs/rank-eval) | [PyPI](https://pypi.org/project/rank-eval/)

### Training

- **[`rank-soft`](crates/rank-soft/)** - Differentiable ranking operations for ML training
  - [crates.io](https://crates.io/crates/rank-soft) | [docs.rs](https://docs.rs/rank-soft) | [PyPI](https://pypi.org/project/rank-soft/)
- **[`rank-learn`](crates/rank-learn/)** - Learning to Rank: LambdaRank, LambdaMART, Ranking SVM
  - [crates.io](https://crates.io/crates/rank-learn) | [docs.rs](https://docs.rs/rank-learn)

## Quick Start

### Rust

```bash
cargo add rank-retrieve
```

### Python

```bash
pip install rank-retrieve
```

### Node.js / WASM

```bash
npm install @arclabs561/rank-fusion
```

## Documentation

- `SETUP.md` - Setup instructions
- `USAGE.md` - Usage guide
- `docs/` - Integration guides, performance, theory

Each crate has its own README and documentation. See individual crate directories.

## License

MIT OR Apache-2.0
