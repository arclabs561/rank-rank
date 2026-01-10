# EVōC and ANN Search: Relationship and Integration

## What is EVōC?

**EVōC** (Embedding Vector Oriented Clustering) is a **fast clustering library** for embedding vectors, developed by the Tutte Institute.

**Key characteristics**:
- **Purpose**: Clustering embeddings, not ANN search
- **Operations**: `EVoC().fit_predict(data)` returns cluster labels and cluster layers
- **Supports**: Float, int8, and binary-quantized embeddings
- **Features**: Multi-granularity clusters, hierarchy extraction, near-duplicate detection
- **Optimized for**: CPU-based clustering of large high-dimensional embedding datasets

## Relationship to ANN Methods

### EVōC is NOT an ANN Algorithm

EVōC does **not** implement approximate nearest neighbor search. It is a **clustering/routing tool** that can complement ANN systems.

### How EVōC Relates to ANN

EVōC fits into the **partitioning/routing** stage of multi-stage ANN architectures:

```
Traditional ANN Pipeline:
1. Partition corpus (k-means, IVF) → clusters
2. Build ANN index per cluster (HNSW, IVF-PQ, etc.)
3. Query: Route to clusters → Search within clusters

EVōC-Enhanced Pipeline:
1. Cluster corpus with EVōC → hierarchical clusters
2. Build ANN index per cluster (HNSW, IVF-PQ, etc.)
3. Query: Route to clusters (centroid similarity) → Search within clusters
```

### Comparison with Existing Partitioning Methods

| Method | Purpose | Characteristics | Use Case |
|--------|---------|-----------------|----------|
| **k-means** | Partitioning for IVF/Anisotropic VQ + k-means (SCANN) | Flat clustering, fast | Standard IVF-PQ, Anisotropic VQ + k-means (SCANN) |
| **EVōC** | Clustering embeddings | Hierarchical, multi-granularity, near-duplicate detection | Advanced routing, hierarchical search |
| **IVF** | Inverted file index | Cluster-based inverted lists | Memory-efficient ANN |

**Key Difference**: EVōC provides **hierarchical clustering** and **near-duplicate detection**, while k-means provides flat partitioning optimized for ANN search.

## Integration Patterns

### Pattern 1: EVōC as Partitioning Stage

Replace k-means with EVōC in IVF-PQ or Anisotropic VQ + k-means (SCANN):

```rust
// Traditional: k-means partitioning
let kmeans = KMeans::new(dimension, num_clusters)?;
kmeans.fit(&vectors, num_vectors)?;
let clusters = kmeans.assign_clusters(&vectors, num_vectors);

// Alternative: EVōC clustering
// (Would need EVōC integration)
let evoc = EVoC::new();
let clusters = evoc.fit_predict(&vectors)?;  // Returns hierarchical clusters
```

**Benefits**:
- Hierarchical clusters (multi-granularity)
- Near-duplicate detection
- Potentially better cluster quality for embeddings

**Trade-offs**:
- Different clustering algorithm (may affect ANN performance)
- Hierarchical structure adds complexity
- Need to evaluate if better than k-means for ANN

### Pattern 2: EVōC as Routing Layer

Use EVōC to create routing layer before ANN search:

```
Architecture:
1. EVōC clusters corpus → hierarchical cluster structure
2. Build ANN index (HNSW, etc.) per cluster
3. Query time:
   a. Find relevant clusters (centroid similarity)
   b. Search ANN index within selected clusters
   c. Merge results across clusters
```

**Benefits**:
- Reduces search space (only search relevant clusters)
- Hierarchical routing (coarse → fine)
- Near-duplicate handling

**Use Cases**:
- Very large corpora (billion+ vectors)
- Need hierarchical organization
- Want near-duplicate detection

### Pattern 3: EVōC for Preprocessing

Use EVōC for corpus analysis before building ANN:

- **Near-duplicate detection**: Remove or deduplicate before indexing
- **Cluster analysis**: Understand corpus structure
- **Hierarchical organization**: Build multi-level indexes

## When to Consider EVōC

### Use EVōC when:
- Need hierarchical clustering of embeddings
- Want near-duplicate detection
- Working with very large corpora requiring routing
- Need multi-granularity cluster structure
- Clustering quality matters more than ANN-specific optimization

### Don't use EVōC when:
- Need direct ANN search (use HNSW, IVF-PQ, etc.)
- Simple k-means partitioning is sufficient
- Query-time performance is critical (EVōC is offline clustering)
- Don't need hierarchical structure

## Relationship to Our Implementation

### Current Status

We currently use **k-means** for partitioning in:
- **Anisotropic VQ + k-means (SCANN)**: k-means partitioning stage
- **IVF-PQ**: k-means clustering for IVF

### Potential EVōC Integration

**Option 1**: Add EVōC as alternative partitioning method
- Feature flag: `evoc_partitioning`
- Replace k-means with EVōC in Anisotropic VQ + k-means (SCANN)/IVF-PQ
- Evaluate performance vs. k-means

**Option 2**: Add EVōC as separate clustering utility
- New module: `dense::clustering::evoc`
- Use for preprocessing, near-duplicate detection
- Optional integration with ANN methods

**Option 3**: Research only (no implementation)
- Document relationship
- Note as potential future enhancement
- Focus on proven ANN methods first

### Recommendation

**Current priority**: Focus on core ANN algorithms (HNSW, Anisotropic VQ + k-means (SCANN), IVF-PQ, etc.)

**Future consideration**: EVōC integration if:
- Hierarchical clustering proves beneficial for ANN
- Near-duplicate detection is needed
- Users request EVōC-based routing

## Technical Details

### EVōC Algorithm

EVōC uses optimized clustering algorithms specifically designed for embedding vectors:
- Fast clustering on CPU
- Handles high-dimensional embeddings efficiently
- Produces hierarchical cluster structure
- Detects near-duplicates during clustering

### Performance Characteristics

- **Clustering time**: Fast (optimized for embeddings)
- **Memory**: Efficient (supports quantized embeddings)
- **Scalability**: Handles large datasets
- **Query time**: N/A (offline clustering only)

### Comparison with k-means

| Aspect | k-means | EVōC |
|--------|---------|------|
| **Structure** | Flat clusters | Hierarchical clusters |
| **Optimization** | ANN search performance | Clustering quality |
| **Near-duplicates** | No | Yes |
| **Speed** | Very fast | Fast |
| **ANN integration** | Standard (IVF, Anisotropic VQ + k-means (SCANN)) | Requires adaptation |

## References

- EVōC GitHub: https://github.com/TutteInstitute/evoc
- Tutte Institute: Embedding Vector Oriented Clustering library
- Related to: IVF partitioning, hierarchical clustering, routing architectures

## Summary

**EVōC is a clustering library, not an ANN algorithm**. It can complement ANN systems by providing:
- Hierarchical clustering for routing
- Near-duplicate detection
- Multi-granularity cluster structure

**Relationship**: EVōC fits in the **partitioning/routing** stage, similar to k-means in IVF-PQ or Anisotropic VQ + k-means (SCANN), but provides hierarchical structure and additional features.

**Integration**: Could replace or complement k-means in partitioning stages, or serve as a routing layer before ANN search.
