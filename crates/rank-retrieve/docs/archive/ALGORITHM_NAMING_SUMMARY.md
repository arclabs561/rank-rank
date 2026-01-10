# Algorithm Naming Summary

## Technical Names vs. Vendor Names

We use **technical algorithm names** in code comments and documentation, with vendor names noted for reference.

### Key Corrections

| Vendor/Project Name | Technical Algorithm Name | Code Name | Notes |
|---------------------|---------------------------|-----------|-------|
| **Annoy** (Spotify) | **Random Projection Tree Forest** | `AnnoyIndex` | Forest of independent random projection trees |
| **SCANN/ScaNN** (Google) | **Anisotropic Vector Quantization with k-means Partitioning** | `SCANNIndex` | Three-stage: partitioning + quantization + re-ranking |
| **HNSW** | **Hierarchical Navigable Small World** | `HNSWIndex` | Correct technical name (acronym acceptable) |
| **SNG** | **Sparse Neighborhood Graph** | `SNGIndex` | Correct technical name (acronym acceptable) |
| **OPT-SNG** | **Optimized Sparse Neighborhood Graph** | `SNGIndex` | Correct technical name |

## Algorithm Relationships

### Graph-Based Family (Sparse Neighborhood Graphs)

All share SNG foundation:
- **HNSW**: Hierarchical variant with fixed α=1, single-pass construction
- **NSG**: Tunable α, candidate set optimization
- **Vamana**: Random init, two-pass, better for SSD serving
- **SNG**: Base structure
- **OPT-SNG**: Martingale-based pruning, auto-tuning, 5.9× speedup

**Common**: Greedy search, proximity graphs, O(log n) search complexity

### Quantization Family

Compression methods for memory efficiency:
- **PQ (Product Quantization)**: Foundation, subspace quantization
- **AVQ (Anisotropic Vector Quantization)**: Preserves inner products (MIPS)
- **SAQ**: Dimension segmentation + code adjustment (80% error reduction)
- **TurboQuant**: Online quantization, near-optimal distortion
- **RaBitQ**: Theoretical error bounds

**Common**: Can combine with IVF (Inverted File Index) for coarse quantization

### Hybrid Methods

Combine multiple techniques:
- **Anisotropic VQ + k-means (SCANN)**: k-means partitioning + AVQ + re-ranking
- **IVF-PQ**: k-means clustering + Product Quantization
- **DiskANN**: Graph structure + disk I/O + cache

### Tree-Based Family

Space partitioning methods:
- **Random Projection Tree Forest**: Multiple RP-trees (Annoy's algorithm)
- **KD-Tree**: Alternating dimension splits (d < 20)
- **Ball Tree**: Hierarchical ball partitioning (20 < d < 100)
- **RP-Trees**: General category

## Implementation Status

All algorithms use correct technical names in:
- Code comments and documentation
- Module-level documentation
- Struct/type documentation

Vendor names are noted for reference but technical names are primary.

## Supporting Methods

### EVōC (Embedding Vector Oriented Clustering)

**What it is**: Fast clustering library for embeddings (NOT an ANN algorithm)

**Relationship to ANN**:
- **Complementary, not competitive**: EVōC provides clustering/routing, not query-time search
- **Position**: Partitioning/routing stage (similar to k-means in IVF-PQ or Anisotropic VQ + k-means (SCANN))
- **Can replace**: k-means in partitioning stages
- **Provides**: Hierarchical clustering, near-duplicate detection, multi-granularity clusters

**Typical Architecture**:
1. EVōC clusters corpus offline → hierarchical clusters
2. Build ANN index per cluster (HNSW, IVF-PQ, etc.)
3. Query: Route to clusters (centroid similarity) → Search within clusters

**Why it matters**: Provides hierarchical clustering optimized for embeddings, which can improve routing in multi-stage ANN systems.

See `EVOC_AND_ANN_RELATIONSHIP.md` for detailed analysis.

See `ANN_ALGORITHM_NAMES_AND_RELATIONSHIPS.md` for detailed relationships.
