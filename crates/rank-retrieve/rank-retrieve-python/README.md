# rank-retrieve-python

Python bindings for [`rank-retrieve`](../README.md) — first-stage retrieval for information retrieval pipelines.

[![PyPI](https://img.shields.io/pypi/v/rank-retrieve.svg)](https://pypi.org/project/rank-retrieve/)
[![Python](https://img.shields.io/pypi/pyversions/rank-retrieve.svg)](https://pypi.org/project/rank-retrieve/)

## Installation

### From PyPI

```bash
pip install rank-retrieve
```

### From source

```bash
# Using uv (recommended)
cd rank-retrieve-python
uv venv
source .venv/bin/activate
uv tool install maturin
maturin develop --uv

# Or using pip
pip install maturin
maturin develop --release
```

## Quick Start

```python
import rank_retrieve

# BM25 retrieval
index = rank_retrieve.InvertedIndex()
index.add_document(0, ["the", "quick", "brown", "fox"])
index.add_document(1, ["the", "lazy", "dog"])
results = index.retrieve(["quick"], 10)
# Returns: [(0, 1.234), ...]  # (doc_id, score) tuples

# Dense retrieval
retriever = rank_retrieve.DenseRetriever()
retriever.add_document(0, [1.0, 0.0, 0.0])
retriever.add_document(1, [0.0, 1.0, 0.0])
results = retriever.retrieve([1.0, 0.0, 0.0], 10)

# Sparse retrieval
retriever = rank_retrieve.SparseRetriever()
vector = rank_retrieve.SparseVector([0, 1, 2], [1.0, 0.5, 0.3])
retriever.add_document(0, vector)
results = retriever.retrieve(vector, 10)
```

## API Reference

### BM25 Retrieval

#### `InvertedIndex`

Main class for BM25 retrieval.

```python
index = rank_retrieve.InvertedIndex()

# Add documents
index.add_document(doc_id: int, terms: List[str])

# Retrieve top-k documents
results = index.retrieve(
    query_terms: List[str],
    k: int,
    params: Optional[Bm25Params] = None
) -> List[Tuple[int, float]]

# Calculate IDF for a term
idf = index.idf(term: str) -> float
```

#### `Bm25Params`

Configuration for BM25 scoring.

```python
# Default parameters (k1=1.2, b=0.75, variant=Standard)
params = rank_retrieve.Bm25Params()

# Custom parameters
params = rank_retrieve.Bm25Params(k1=1.5, b=0.8)

# Chain configuration
params = rank_retrieve.Bm25Params().with_k1(1.5).with_b(0.8)
```

**Parameters:**
- `k1` (float, default=1.2): Term frequency saturation parameter
- `b` (float, default=0.75): Length normalization parameter

### Dense Retrieval

#### `DenseRetriever`

Cosine similarity-based dense retrieval.

```python
retriever = rank_retrieve.DenseRetriever()

# Add documents with embeddings
retriever.add_document(doc_id: int, embedding: List[float])

# Retrieve top-k documents
results = retriever.retrieve(
    query_embedding: List[float],
    k: int
) -> List[Tuple[int, float]]

# Score a specific document
score = retriever.score(
    doc_id: int,
    query_embedding: List[float]
) -> Optional[float]
```

**Note**: Embeddings should be L2-normalized for cosine similarity.

### Sparse Retrieval

#### `SparseRetriever`

Dot product-based sparse retrieval (e.g., SPLADE, learned sparse).

```python
retriever = rank_retrieve.SparseRetriever()

# Add documents with sparse vectors
retriever.add_document(doc_id: int, vector: SparseVector)

# Retrieve top-k documents
results = retriever.retrieve(
    query_vector: SparseVector,
    k: int
) -> List[Tuple[int, float]]

# Score a specific document
score = retriever.score(
    doc_id: int,
    query_vector: SparseVector
) -> Optional[float]
```

#### `SparseVector`

Sparse vector representation.

```python
# Create sparse vector
vector = rank_retrieve.SparseVector(
    indices: List[int],  # Sorted, unique indices
    values: List[float],  # Corresponding values
    validate: bool = True  # Validate indices are sorted and unique
)

# Access properties
indices = vector.indices  # List[int]
values = vector.values    # List[float]

# Prune low values
pruned = vector.prune(threshold: float) -> SparseVector
```

#### Utility Functions

```python
# Dot product between two sparse vectors
score = rank_retrieve.sparse_dot_product(a: SparseVector, b: SparseVector) -> float
```

## Complete Example: Hybrid Search Pipeline

```python
import rank_retrieve
import rank_fusion  # pip install rank-fusion
import rank_rerank  # pip install rank-rerank

# Step 1: Build indexes
bm25_index = rank_retrieve.InvertedIndex()
bm25_index.add_document(0, ["machine", "learning"])
bm25_index.add_document(1, ["deep", "learning"])

dense_retriever = rank_retrieve.DenseRetriever()
dense_retriever.add_document(0, [0.8, 0.2, 0.1])
dense_retriever.add_document(1, [0.7, 0.3, 0.2])

# Step 2: Retrieve from multiple sources
query_terms = ["learning"]
query_embedding = [0.75, 0.25, 0.15]

bm25_results = bm25_index.retrieve(query_terms, 1000)
dense_results = dense_retriever.retrieve(query_embedding, 1000)

# Step 3: Fuse results
fused = rank_fusion.rrf(bm25_results, dense_results, k=60)

# Step 4: Rerank (optional)
# query_tokens = [[0.75, 0.25], [0.15, 0.85]]
# doc_tokens = [[[0.8, 0.2], [0.1, 0.9]], ...]
# reranked = rank_rerank.maxsim_vecs(query_tokens, doc_tokens)

# Step 5: Return top-k for LLM context
top_k = [doc_id for doc_id, score in fused[:10]]
```

## Error Handling

All retrieval methods return results directly. Invalid inputs produce empty results or `None` for score queries:

```python
# Empty query returns empty results
results = index.retrieve([], 10)  # Returns []

# Document not found returns None
score = retriever.score(999, query_embedding)  # Returns None
```

## Performance

Python bindings add minimal overhead:
- **BM25 (1000 docs)**: ~500μs (vs 450μs in Rust)
- **Dense (1000 docs, 128-dim)**: ~2ms (vs 1.8ms in Rust)
- **Sparse (1000 docs)**: ~1ms (vs 900μs in Rust)

Overhead comes from Python object conversion (~50-100μs per call).

## See Also

- **[Core crate documentation](../README.md)** - Complete API reference and theory
- **[Getting Started Guide](../docs/GETTING_STARTED.md)** - Step-by-step tutorial
- **[Use Cases](../docs/USE_CASES.md)** - When to use rank-retrieve vs alternatives
- **[Rust API Documentation](https://docs.rs/rank-retrieve)** - Full Rust API reference

## License

MIT OR Apache-2.0
