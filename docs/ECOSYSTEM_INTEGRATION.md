# Ecosystem Integration Guide
## Connecting rank-rank with the Rust ML Ecosystem

This guide demonstrates how to integrate `rank-rank` crates with popular Rust ML frameworks and tools.

---

## Table of Contents

1. [Candle Integration](#candle-integration)
2. [Burn Integration](#burn-integration)
3. [ONNX Runtime Integration](#onnx-runtime-integration)
4. [Vector Database Integration](#vector-database-integration)
5. [End-to-End RAG Pipeline](#end-to-end-rag-pipeline)

---

## Candle Integration

### Overview

Candle (Hugging Face) provides GPU-accelerated tensor operations. `rank-rank` supports Candle for:
- GPU-accelerated MaxSim computation (`rank-rerank`)
- Differentiable ranking operations (`rank-soft`)

### Installation

Add Candle to your `Cargo.toml`:

```toml
[dependencies]
rank-rerank = { version = "0.7", features = ["candle"] }
rank-soft = { version = "0.1", features = ["candle"] }
candle-core = "0.3"
```

### rank-soft with Candle

**Differentiable ranking operations:**

```rust
use candle_core::{Device, Tensor};
use rank_soft::candle::{soft_rank_candle, spearman_loss_candle};

let device = Device::Cpu;  // or Device::new_cuda(0)? for GPU

// Soft ranking
let values = Tensor::new(&[5.0f32, 1.0, 2.0, 4.0, 3.0], &device)?;
let ranks = soft_rank_candle(&values, 1.0, &device)?;

// Spearman loss for training
let predictions = Tensor::new(&[0.1f32, 0.9, 0.3, 0.7, 0.5], &device)?;
let targets = Tensor::new(&[0.0f32, 1.0, 0.2, 0.8, 0.4], &device)?;
let loss = spearman_loss_candle(&predictions, &targets, 1.0, &device)?;
```

**GPU Acceleration:**

```rust
// Use CUDA for NVIDIA GPUs
let gpu_device = Device::new_cuda(0)?;
let values_gpu = values.to_device(&gpu_device)?;
let ranks_gpu = soft_rank_candle(&values_gpu, 1.0, &gpu_device)?;

// Use Metal for Apple Silicon
let metal_device = Device::new_metal(0)?;
let values_metal = values.to_device(&metal_device)?;
let ranks_metal = soft_rank_candle(&values_metal, 1.0, &metal_device)?;
```

### rank-rerank with Candle

**GPU-accelerated MaxSim:**

```rust
use candle_core::{Device, Tensor};
use rank_rerank::candle::{maxsim_candle, maxsim_cosine_candle};

let device = Device::Cpu;  // or Device::new_cuda(0)? for GPU

// Query tokens: [num_query_tokens, embedding_dim]
let query_tokens = Tensor::randn(0f32, 1.0, (3, 128), &device)?;

// Document tokens: [num_doc_tokens, embedding_dim]
let doc_tokens = Tensor::randn(0f32, 1.0, (10, 128), &device)?;

// Compute MaxSim
let score = maxsim_candle(&query_tokens, &doc_tokens, &device)?;

// Or with cosine similarity
let score_cosine = maxsim_cosine_candle(&query_tokens, &doc_tokens, &device)?;
```

**Benefits:**
- GPU acceleration for large batches
- Unified tensor API
- Leverages Candle's optimizations

**See:** `crates/rank-rerank/examples/candle_maxsim.rs` for complete example.

---

## Burn Integration

### Overview

Burn provides backend-agnostic tensor operations, supporting multiple backends (CUDA, Metal, Vulkan, WebGPU).

### Status

🚧 **Planned** - Burn integration is in progress. Current status:
- Feature flag exists: `burn = []`
- Implementation planned for `rank-soft`
- Will support all Burn backends automatically

### Planned API

```rust
use burn::tensor::{Tensor, Backend};
use rank_soft::burn::{soft_rank_burn, spearman_loss_burn};

fn training_step<B: Backend>(
    predictions: Tensor<B, 1>,
    targets: Tensor<B, 1>,
) -> Tensor<B, 1> {
    // Differentiable ranking loss
    spearman_loss_burn(&predictions, &targets, 1.0)
    // Gradients automatically flow through Burn's autograd
}
```

**Benefits:**
- Multi-backend support (future-proof)
- Training + inference support
- Native quantization support

---

## ONNX Runtime Integration

### Overview

ONNX Runtime enables running models exported from PyTorch/TensorFlow in Rust.

### Installation

```toml
[dependencies]
rank-rerank = { version = "0.7", features = ["ort", "crossencoder"] }
```

### Python Usage

```python
import rank_rerank

# Load ONNX cross-encoder model
encoder = rank_rerank.OrtCrossEncoder.from_file(
    "model.onnx",
    tokenizer_path="tokenizer.json",  # Optional
    max_length=512
)

# Score a query-document pair
score = encoder.score("query text", "document text")

# Batch scoring
scores = encoder.score_batch("query text", ["doc1", "doc2", "doc3"])
```

### Rust Usage

```rust
use rank_rerank::crossencoder::ort::OrtCrossEncoder;

// Load model
let encoder = OrtCrossEncoder::from_file("model.onnx", None, 512)?;

// Score single pair
let score = encoder.score("query", "document");

// Batch scoring
let scores = encoder.score_batch("query", &["doc1", "doc2", "doc3"]);
```

### Exporting Models to ONNX

Use the Python utilities in `rank-rerank-python/rank_rerank/onnx_export.py`:

```python
from rank_rerank.onnx_export import export_cross_encoder, export_colbert_encoder

# Export cross-encoder
export_cross_encoder(
    "cross-encoder/ms-marco-MiniLM-L-6-v2",
    "model.onnx",
    max_length=512
)

# Export ColBERT encoder
export_colbert_encoder(
    "colbert-ir/colbertv2.0",
    "colbert_encoder.onnx",
    max_length=512
)
```

**Note:** `ort` 2.0.0-rc.11 is production-ready but API may change before stable 2.0.0 release.

---

## Vector Database Integration

### Qdrant Integration

**Example workflow:**

```rust
// 1. Dense retrieval from Qdrant
let qdrant = QdrantClient::from_url("http://localhost:6333")?;
let dense_results = qdrant.search(&collection, query_embedding, 10).await?;

// 2. Rerank with rank-rerank
use rank_rerank::colbert::rank;
let reranked = rank(&query_tokens, &candidates)?;

// 3. Return final results
```

**See:** `crates/rank-retrieve/examples/qdrant_integration.rs` for complete example.

### usearch Integration

**HNSW for approximate nearest neighbor:**

```rust
// 1. Build HNSW index with usearch
let index = usearch::Index::new(128, usearch::Metric::Cosine)?;
// ... add vectors to index

// 2. Approximate nearest neighbor search
let candidates = index.search(&query_vector, 10)?;

// 3. Rerank with rank-rerank
use rank_rerank::simd::maxsim_vecs;
let scores: Vec<f32> = candidates.iter()
    .map(|candidate| maxsim_vecs(&query_tokens, &candidate.tokens))
    .collect();
```

---

## End-to-End RAG Pipeline

### Complete Workflow

```rust
use rank_retrieve::Bm25Index;
use rank_rerank::colbert::rank;
use rank_fusion::reciprocal_rank_fusion;
use rank_eval::ndcg_at_k;

// 1. First-stage retrieval (sparse)
let mut bm25 = Bm25Index::new();
// ... add documents
let sparse_results = bm25.retrieve(&query_terms, 100, Default::default())?;

// 2. First-stage retrieval (dense) - from vector DB
let dense_results = qdrant.search(&query_embedding, 100).await?;

// 3. Rank fusion
let fused_results = reciprocal_rank_fusion(&[
    sparse_results,
    dense_results,
], 60)?;

// 4. Rerank top-K with MaxSim
let top_k = fused_results.iter().take(20);
let reranked = rank(&query_tokens, &top_k)?;

// 5. Evaluate
let ndcg = ndcg_at_k(&reranked, &ground_truth, 10)?;
```

### Python Pipeline

```python
import rank_retrieve
import rank_rerank
import rank_fusion
import rank_eval

# 1. Sparse retrieval
bm25 = rank_retrieve.Bm25Index()
# ... add documents
sparse = bm25.retrieve(query_terms, k=100)

# 2. Dense retrieval (from Qdrant or similar)
dense = qdrant_client.search(query_embedding, limit=100)

# 3. Rank fusion
fused = rank_fusion.reciprocal_rank_fusion([sparse, dense], k=60)

# 4. Rerank
reranked = rank_rerank.colbert_rank(query_tokens, candidates)

# 5. Evaluate
ndcg = rank_eval.ndcg_at_k(reranked, ground_truth, k=10)
```

---

## Best Practices

### 1. **Hybrid Retrieval**

Combine sparse (BM25) and dense (embeddings) retrieval:
- Sparse: Good for keyword matching
- Dense: Good for semantic similarity
- Fusion: Best of both worlds

### 2. **GPU Acceleration**

Use Candle for:
- Large batch processing (>100 documents)
- GPU-available environments
- Real-time inference requirements

### 3. **ONNX for Production**

Export models to ONNX for:
- Model portability
- Quantization support
- Multi-platform deployment

### 4. **Vector Databases**

Use vector DBs for:
- Large-scale dense retrieval
- Persistent storage
- Real-time updates

---

## Examples

- **Candle MaxSim:** `crates/rank-rerank/examples/candle_maxsim.rs`
- **Candle Training:** `crates/rank-soft/examples/candle_training.rs`
- **Qdrant Integration:** `crates/rank-retrieve/examples/qdrant_integration.rs`
- **ONNX Export:** `crates/rank-rerank/rank-rerank-python/rank_rerank/onnx_export.py`

---

## Troubleshooting

### Candle Compilation Issues

If you encounter dependency conflicts with `candle-core`:
- Try updating to latest version: `cargo update candle-core`
- Check Rust version compatibility (requires 1.74+)
- Consider using specific version: `candle-core = "0.3.2"`

### ONNX Runtime Issues

If `ort` feature doesn't compile:
- Ensure you're using `ort = "2.0.0-rc.11"` or later
- Check that `crossencoder` feature is also enabled
- Verify ONNX model file is valid

### GPU Not Detected

If GPU operations fail:
- Verify CUDA/Metal drivers are installed
- Check device creation: `Device::new_cuda(0)?` or `Device::new_metal(0)?`
- Fall back to CPU: `Device::Cpu`

---

**Last Updated:** January 2025  
**See Also:** `RUST_ML_ECOSYSTEM_ALIGNMENT.md` for strategic analysis

