# rank-rank

Ranking and retrieval crates for Rust IR pipelines.

[![CI](https://github.com/arclabs561/rank-rank/actions/workflows/ci.yml/badge.svg)](https://github.com/arclabs561/rank-rank/actions)

Unified APIs for information retrieval in Rust. Python frameworks (LlamaIndex, Haystack) provide unified retrieval, but Rust developers must manually compose multiple crates. These crates fill that gap: consistent interfaces, seamless integration, minimal dependencies.

**Why multi-stage pipelines**: IR systems balance speed and precision. First-stage retrieval (BM25, dense ANN) narrows 10M documents to 1000 candidates quickly. Fusion combines multiple retrieval methods (hybrid search). Reranking (MaxSim, cross-encoder) precisely scores 1000 candidates down to 100 results. Evaluation metrics measure quality.

**Pipeline**: `rank-retrieve` → `rank-fusion` → `rank-rerank` → `rank-eval`  
**Training**: `rank-soft` (differentiable ranking), `rank-learn` (LTR algorithms)

## Crates

### Pipeline Stages

- **[`rank-retrieve`](crates/rank-retrieve/)** - First-stage retrieval: BM25, dense ANN, sparse vectors  
  [crates.io](https://crates.io/crates/rank-retrieve) · [docs.rs](https://docs.rs/rank-retrieve) · [PyPI](https://pypi.org/project/rank-retrieve/)

- **[`rank-fusion`](crates/rank-fusion/)** - Rank fusion: RRF, ISR, CombMNZ for hybrid search  
  [crates.io](https://crates.io/crates/rank-fusion) · [docs.rs](https://docs.rs/rank-fusion) · [PyPI](https://pypi.org/project/rank-fusion/) · [npm](https://www.npmjs.com/package/@arclabs561/rank-fusion)

- **[`rank-rerank`](crates/rank-rerank/)** - SIMD-accelerated reranking: MaxSim (ColBERT), cosine, diversity  
  [crates.io](https://crates.io/crates/rank-rerank) · [docs.rs](https://docs.rs/rank-rerank) · [PyPI](https://pypi.org/project/rank-rerank/) · [npm](https://www.npmjs.com/package/@arclabs561/rank-rerank)

- **[`rank-eval`](crates/rank-eval/)** - IR evaluation metrics: NDCG, MAP, MRR, TREC format  
  [crates.io](https://crates.io/crates/rank-eval) · [docs.rs](https://docs.rs/rank-eval) · [PyPI](https://pypi.org/project/rank-eval/)

### Training

- **[`rank-soft`](crates/rank-soft/)** - Differentiable ranking operations for ML training  
  [crates.io](https://crates.io/crates/rank-soft) · [docs.rs](https://docs.rs/rank-soft) · [PyPI](https://pypi.org/project/rank-soft/)

- **[`rank-learn`](crates/rank-learn/)** - Learning to Rank: LambdaRank, LambdaMART, Ranking SVM  
  [crates.io](https://crates.io/crates/rank-learn) · [docs.rs](https://docs.rs/rank-learn)

## Quick Start

```rust
use rank_retrieve::bm25::{InvertedIndex, retrieve_bm25, Bm25Params};
use rank_fusion::rrf;
use rank_rerank::simd::maxsim_vecs;

// 1. Retrieve (10M docs → 1000 candidates)
let mut index = InvertedIndex::new();
index.add_document(0, &["machine".to_string(), "learning".to_string()]);
let bm25_results = retrieve_bm25(&index, &["learning".to_string()], 1000, Bm25Params::default())?;

// 2. Fuse with dense results
let fused = rrf(&bm25_results, &dense_results);

// 3. Rerank with MaxSim (1000 → 100 results)
let reranked = maxsim_vecs(&query_tokens, &candidates, 100)?;
```

## Installation

### Rust

```bash
cargo add rank-retrieve
```

### Python

```bash
uv pip install rank-retrieve
```

### Node.js / WASM

```bash
npm install @arclabs561/rank-fusion
```

## Limitations

- **In-memory only**: No persistent storage. Use `tantivy` or vector DBs for persistence.
- **Basic implementations**: Not optimized for very large scale. Use specialized backends for production.
- **Not a full RAG framework**: No document loading, chunking, or LLM integration.

## Documentation

- `SETUP.md` - Setup instructions
- `USAGE.md` - Usage guide
- `docs/` - Integration guides, performance, theory
- `PLAN.md` - Design constraints, architecture, implementation details

Each crate has its own README and documentation. See individual crate directories.

## License

MIT OR Apache-2.0
