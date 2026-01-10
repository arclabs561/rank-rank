# rank-rank

Ranking and retrieval crates for Rust IR pipelines.

[![CI](https://github.com/arclabs561/rank-rank/actions/workflows/ci.yml/badge.svg)](https://github.com/arclabs561/rank-rank/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Maintenance](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)](https://github.com/arclabs561/rank-rank)

Unified APIs for information retrieval in Rust. Python frameworks (LlamaIndex, Haystack) provide unified retrieval, but Rust developers must manually compose multiple crates. These crates fill that gap: consistent interfaces, seamless integration, minimal dependencies.

**Why multi-stage pipelines**: IR systems balance speed and precision. First-stage retrieval (BM25, dense ANN) narrows 10M documents to 1000 candidates quickly. Fusion combines multiple retrieval methods (hybrid search). Reranking (MaxSim, cross-encoder) precisely scores 1000 candidates down to 100 results. Evaluation metrics measure quality.

**Pipeline**: `rank-retrieve` → `rank-fusion` → `rank-rerank` → `rank-eval`  
**Training**: `rank-soft` (differentiable ranking, LTR algorithms: LambdaRank, Ranking SVM)

### Pipeline Flow

```
10M docs → rank-retrieve → 1000 candidates (fast, broad)
   1000 → rank-fusion → combined results (hybrid search)
   1000 → rank-rerank → 100 results (precise, MaxSim/ColBERT)
    100 → rank-eval → metrics (NDCG, MAP, MRR)
```

**Training crate** (`rank-soft`) is used to train ranking models, not part of the inference pipeline. Use it when you need to train custom ranking models on your data. It provides differentiable ranking operations and complete LTR algorithms (LambdaRank, Ranking SVM, neural LTR).

## Crates

### Pipeline Stages

- **[`rank-retrieve`](crates/rank-retrieve/README.md)** - First-stage retrieval: BM25, dense ANN, sparse vectors  
  [crates.io](https://crates.io/crates/rank-retrieve) · [docs.rs](https://docs.rs/rank-retrieve) · [PyPI](https://pypi.org/project/rank-retrieve/)

- **[`rank-fusion`](crates/rank-fusion/README.md)** - Rank fusion: RRF, ISR, CombMNZ for hybrid search  
  [crates.io](https://crates.io/crates/rank-fusion) · [docs.rs](https://docs.rs/rank-fusion) · [PyPI](https://pypi.org/project/rank-fusion/) · [npm](https://www.npmjs.com/package/@arclabs561/rank-fusion)

- **[`rank-rerank`](crates/rank-rerank/README.md)** - SIMD-accelerated reranking: MaxSim (ColBERT), cosine, diversity  
  [crates.io](https://crates.io/crates/rank-rerank) · [docs.rs](https://docs.rs/rank-rerank) · [PyPI](https://pypi.org/project/rank-rerank/) · [npm](https://www.npmjs.com/package/@arclabs561/rank-rerank)

- **[`rank-eval`](crates/rank-eval/README.md)** - IR evaluation metrics: NDCG, MAP, MRR, TREC format  
  [crates.io](https://crates.io/crates/rank-eval) · [docs.rs](https://docs.rs/rank-eval) · [PyPI](https://pypi.org/project/rank-eval/)

### Training

- **[`rank-soft`](crates/rank-soft/README.md)** - Differentiable ranking operations and Learning to Rank algorithms  
  [crates.io](https://crates.io/crates/rank-soft) · [docs.rs](https://docs.rs/rank-soft) · [PyPI](https://pypi.org/project/rank-soft/)  
  Provides: soft ranking, LambdaRank, Ranking SVM, neural LTR models

## Quick Start

Complete pipeline example:

```rust
use rank_retrieve::bm25::{InvertedIndex, retrieve_bm25, Bm25Params};
use rank_fusion::rrf;
use rank_rerank::colbert::rank_with_top_k;
use rank_eval::binary::ndcg_at_k;
use std::collections::HashSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Retrieve (10M docs → 1000 candidates)
    let mut index = InvertedIndex::new();
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["deep".to_string(), "learning".to_string()]);
    let query = vec!["learning".to_string()];
    let bm25_results = retrieve_bm25(&index, &query, 1000, Bm25Params::default())?;

    // 2. Fuse with dense results (hybrid search)
    let dense_results = vec![(0, 0.9), (1, 0.8)];  // Example dense retrieval results
    let fused = rrf(&bm25_results, &dense_results);

    // 3. Rerank with MaxSim (1000 → 100 results)
    let query_tokens = vec![vec![0.1, 0.2], vec![0.3, 0.4]];  // Query token embeddings
    let doc_tokens = vec![
        (0, vec![vec![0.1, 0.2], vec![0.3, 0.4]]),  // Doc 0 token embeddings
        (1, vec![vec![0.2, 0.3], vec![0.4, 0.5]]),  // Doc 1 token embeddings
    ];
    let reranked = rank_with_top_k(&query_tokens, &doc_tokens, Some(100));

    // 4. Evaluate (optional)
    let ranked_ids: Vec<usize> = reranked.iter().map(|(id, _)| *id).collect();
    let relevant: HashSet<usize> = [0].into_iter().collect();  // Doc 0 is relevant
    let ndcg = ndcg_at_k(&ranked_ids, &relevant, 10);
    println!("NDCG@10: {:.4}", ndcg);
    
    Ok(())
}
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
npm install @arclabs561/rank-rerank
```

**Note**: Only `rank-fusion` and `rank-rerank` have WASM/npm packages. Other crates are Rust/Python only.

## When to Use rank-*

✅ **Good fit for**:
- Building Rust-native RAG pipelines without Python FFI overhead
- Hybrid search combining multiple retrieval methods (BM25 + dense + sparse)
- Researching/experimenting with retrieval methods
- Need generative retrieval (LTRGR) - unique in Rust ecosystem
- Integrating with rank-* ecosystem (seamless crate integration)
- Prototyping and development (in-memory indexes)

❌ **Not a good fit for**:
- Only need one retrieval method → Use specialized crates (`tantivy` for BM25, `hnsw_rs` for dense ANN)
- Need persistent storage → Use `tantivy` or vector databases (Qdrant, Milvus, Pinecone)
- Very large scale (billions of documents) → Use specialized backends optimized for scale
- Need full RAG framework → Use Python frameworks (LlamaIndex, Haystack) with document loading, chunking, LLM integration
- Production deployments requiring distributed systems → Use vector databases with sharding/replication

## Limitations

- **In-memory only**: No persistent storage by default. Use `tantivy` or vector databases for persistence.
- **Basic implementations**: Not optimized for very large scale (billions of documents). Use specialized backends for production scale.
- **Not a full RAG framework**: No document loading, chunking, or LLM integration. Focuses on retrieval, fusion, reranking, and evaluation.
- **Zero dependencies by default**: Some crates (`rank-fusion`, `rank-rerank`) have zero dependencies by default. Others (`rank-retrieve`) use feature flags to keep dependencies optional.

## Documentation

- [`SETUP.md`](SETUP.md) - Setup instructions
- [`USAGE.md`](USAGE.md) - Usage guide
- [`docs/`](docs/) - Integration guides, performance, theory
- [`PLAN.md`](PLAN.md) - Design constraints, architecture, implementation details

Each crate has its own README and documentation:
- [`rank-retrieve`](crates/rank-retrieve/README.md) - First-stage retrieval documentation
- [`rank-fusion`](crates/rank-fusion/README.md) - Rank fusion algorithms
- [`rank-rerank`](crates/rank-rerank/README.md) - Reranking and similarity scoring
- [`rank-eval`](crates/rank-eval/README.md) - Evaluation metrics
- [`rank-soft`](crates/rank-soft/README.md) - Differentiable ranking operations and Learning to Rank algorithms

## License

MIT OR Apache-2.0
