# ANN Algorithm Names and Relationships

This document establishes correct technical names for all ANN algorithms (avoiding vendor/project names) and documents their relationships.

## Technical Names vs. Vendor Names

### Correct Naming Convention

We use **technical algorithm names** rather than vendor/project names:

| Vendor/Project Name | Technical Algorithm Name | Notes |
|---------------------|---------------------------|-------|
| **Annoy** (Spotify) | **Random Projection Tree Forest** | Forest of random projection trees |
| **SCANN/ScaNN** (Google) | **Anisotropic Vector Quantization with k-means Partitioning** | Combines AVQ + k-means + re-ranking |
| **HNSW** | **Hierarchical Navigable Small World** | Correct technical name |
| **SNG** | **Sparse Neighborhood Graph** | Correct technical name |
| **OPT-SNG** | **Optimized Sparse Neighborhood Graph** | Correct technical name |

## Algorithm Families and Relationships

### 1. Graph-Based Methods (Sparse Neighborhood Graphs)

All graph-based ANN methods share a common foundation: **Sparse Neighborhood Graphs (SNGs)**.

```
Sparse Neighborhood Graph (SNG)
├── HNSW (Hierarchical Navigable Small World)
│   └── Multi-layer hierarchical structure
│   └── α = 1 (fixed)
│   └── Single-pass construction
├── NSG (Navigating Spreading-out Graph)
│   └── Tunable α parameter
│   └── Candidate set includes all visited vertices
├── Vamana
│   └── Random graph initialization
│   └── Two-pass construction
│   └── Better for SSD-based serving (5-10× more points/node)
├── SNG (Sparse Neighborhood Graph)
│   └── Base structure
└── OPT-SNG (Optimized Sparse Neighborhood Graph)
    └── Martingale-based pruning
    └── Automatic parameter tuning
    └── 5.9× construction speedup
```

**Key Relationships**:
- **HNSW** is a hierarchical variant of SNG with fixed α=1
- **NSG** and **Vamana** are SNG variants with tunable parameters
- **OPT-SNG** optimizes SNG construction using martingale theory
- All use **greedy search** (or variants) for querying
- All create **proximity graphs** connecting similar points

**Common Properties**:
- Construction: O(n log n)
- Search: O(log n) expected
- Memory: O(n) to O(n log n)
- High recall (>90% typical)

### 2. Quantization Methods

Quantization methods compress vectors to reduce memory and accelerate distance computation.

```
Vector Quantization
├── Product Quantization (PQ)
│   └── Subspace quantization
│   └── Used in IVF-PQ
├── Anisotropic Vector Quantization (AVQ)
│   └── Preserves inner product (MIPS)
│   └── Used in Anisotropic VQ + k-means (SCANN)
├── SAQ (Segmented Adaptive Quantization)
│   └── Dimension segmentation + code adjustment
│   └── 80% error reduction vs PQ
├── TurboQuant
│   └── Online quantization
│   └── Near-optimal distortion
└── RaBitQ
    └── Theoretical error bounds
```

**Key Relationships**:
- **PQ** is the foundation for most quantization methods
- **AVQ** is specialized for Maximum Inner Product Search (MIPS)
- **SAQ** improves PQ through segmentation and code adjustment
- **TurboQuant** provides online/streaming capability
- All can be combined with **IVF** (Inverted File Index) for coarse quantization

**Common Properties**:
- Compression: 4× to 64× typical
- Encoding: O(d) per vector
- Distance computation: O(m) where m << d
- Trade-off: Accuracy vs. compression ratio

### 3. Hybrid Methods

Some methods combine multiple techniques:

```
Hybrid ANN Methods
├── Anisotropic VQ + k-means (SCANN, + Re-ranking)
│   ├── k-means partitioning (coarse search)
│   ├── Anisotropic quantization (fine search)
│   └── Re-ranking (accuracy refinement)
├── IVF-PQ (Inverted File + Product Quantization)
│   ├── IVF: k-means clustering (coarse)
│   └── PQ: Product quantization (fine)
├── DiskANN (Graph + Disk I/O + Cache)
│   ├── Graph structure (SNG-based)
│   ├── Disk storage
│   └── Working set cache
└── EVōC + ANN (Clustering + ANN per cluster)
    ├── EVōC: Fast embedding clustering (offline)
    ├── ANN index per cluster (HNSW, IVF-PQ, etc.)
    └── Query routing to clusters (centroid similarity)
```

**Key Relationships**:
- **Anisotropic VQ + k-means (SCANN)** = Partitioning + Quantization + Re-ranking
- **IVF-PQ** = Clustering + Quantization
- **DiskANN** = Graph + Storage optimization
- **EVōC + ANN** = Clustering (EVōC) + ANN per cluster (routing architecture)
- Hybrid methods combine coarse and fine search stages
- EVōC can replace k-means in partitioning stages (provides hierarchical clustering)

### 4. Tree-Based Methods

Tree methods partition space hierarchically:

```
Tree-Based Methods
├── Random Projection Tree Forest
│   └── Random hyperplane splits
│   └── Forest of trees (Random Projection Tree Forest, vendor: Annoy)
├── KD-Tree (K-Dimensional Tree)
│   └── Alternating dimension splits
│   └── Good for d < 20
├── Ball Tree
│   └── Hierarchical ball partitioning
│   └── Good for 20 < d < 100
├── K-Means Tree
│   └── Hierarchical clustering tree
│   └── Good for 20 < d < 200
└── Random Projection Trees (RP-Trees)
    └── General category
    └── Includes Random Projection Tree Forest
```

**Key Relationships**:
- **Random Projection Tree Forest** is a specific implementation using multiple RP-trees
- **KD-Tree** and **Ball Tree** are space-partitioning trees (not projection-based)
- **K-Means Tree** uses clustering instead of space partitioning (complementary approach)
- All tree methods use hierarchical space partitioning
- Forest approaches (multiple trees) improve recall

### 5. Hash-Based Methods

Hash methods use locality-sensitive hashing:

```
Hash-Based Methods
├── LSH (Locality Sensitive Hashing)
│   ├── Random Projection LSH (cosine similarity)
│   ├── E2LSH (L2 distance)
│   └── MinHash LSH (Jaccard similarity)
└── LSH Forest
    └── Adaptive parameter selection
```

**Key Relationships**:
- **LSH** provides theoretical guarantees (approximation bounds)
- Different hash families for different distance metrics
- Multiple hash tables improve recall
- Complementary to graph/tree methods (different trade-offs)

## Algorithm Connections

### Construction Relationships

1. **k-means clustering** is used by:
   - SCANN (partitioning)
   - IVF-PQ (coarse quantization)
   - SAQ (codebook generation)

2. **Graph construction** methods:
   - HNSW: Hierarchical, single-pass
   - NSG: Tunable α, candidate set optimization
   - Vamana: Random init, two-pass
   - OPT-SNG: Martingale pruning, auto-tuning

3. **Quantization** methods:
   - PQ: Foundation for IVF-PQ
   - AVQ: Specialized for Anisotropic VQ + k-means (SCANN, MIPS)
   - SAQ: Improves PQ (segmentation)
   - TurboQuant: Online variant

### Search Relationships

1. **Greedy search** is used by:
   - All graph-based methods (HNSW, SNG, NSG, Vamana)
   - Navigates graph structure

2. **Tree traversal** is used by:
   - Random Projection Tree Forest
   - KD-Tree, Ball Tree

3. **Hash lookup** is used by:
   - LSH (candidate generation)

4. **Two-stage search** (coarse + fine):
   - Anisotropic VQ + k-means (SCANN): k-means → quantization → re-ranking
   - IVF-PQ: IVF → PQ distance
   - DiskANN: Graph search → disk I/O

### Complementary Methods

Methods that work well together:

1. **Graph + Quantization**:
   - Can combine graph structure with quantized vectors
   - Reduces memory while maintaining graph connectivity

2. **Partitioning + Quantization**:
   - Anisotropic VQ + k-means (SCANN): k-means partitioning + AVQ
   - IVF-PQ: k-means clustering + PQ

3. **Tree + Forest**:
   - Multiple trees improve recall
   - Random Projection Tree Forest uses this

## Implementation Naming

### Code Structure

```rust
// Correct technical names in code:
pub mod random_projection_tree_forest;  // Not "annoy"
pub mod anisotropic_vq_kmeans;         // Not "scann"
pub mod hierarchical_navigable_small_world;  // "hnsw" is acceptable (acronym)
pub mod sparse_neighborhood_graph;      // "sng" is acceptable
pub mod optimized_sng;                  // "opt_sng" is acceptable
```

### Documentation

- Use full technical names in documentation
- Can include vendor names in parentheses for reference
- Example: "Random Projection Tree Forest (implemented by Spotify as Annoy)"

### API Names

- Use technical names in public APIs
- Short names (like `RPTreeIndex`) are acceptable if clear
- Avoid vendor-specific names in public APIs

## Supporting Methods: Clustering and Routing

### EVōC (Embedding Vector Oriented Clustering)

**What it is**: A fast clustering library for embedding vectors, not an ANN search algorithm itself.

**Purpose**: 
- Fast clustering of large high-dimensional embedding datasets
- Multi-granularity clusters and hierarchy extraction
- Near-duplicate detection
- Supports float, int8, and binary-quantized embeddings

**Relationship to ANN**:
- **Complementary, not competitive**: EVōC is used for **clustering/routing**, not for query-time nearest neighbor search
- **Typical architecture**:
  1. Use EVōC to cluster embedding corpus offline → get cluster assignments
  2. Build ANN index per cluster (or per cluster subset) using HNSW, IVF-PQ, etc.
  3. At query time: route query to relevant clusters (via centroid similarity), then perform ANN search within those clusters

**Connection to existing methods**:
- Similar to **IVF (Inverted File Index)** partitioning stage, but EVōC provides:
  - Multi-granularity clustering (hierarchical)
  - Fast clustering optimized for embeddings
  - Near-duplicate detection
- Can be used **instead of k-means** in IVF-PQ or Anisotropic VQ + k-means (SCANN) partitioning stages
- Provides **routing layer** for multi-stage ANN systems

**When to consider**:
- Need hierarchical clustering of embeddings
- Want to partition large corpus before building ANN indexes
- Need near-duplicate detection
- Want to route queries to relevant clusters before ANN search

**Not for**:
- Direct ANN search (use HNSW, IVF-PQ, etc. instead)
- Query-time nearest neighbor search (EVōC is offline clustering)

## References

- Malkov & Yashunin (2018): "Efficient and robust approximate nearest neighbor search using Hierarchical Navigable Small World graphs"
- Guo et al. (2020): "Accelerating Large-Scale Inference with Anisotropic Vector Quantization" (Anisotropic VQ + k-means, vendor: SCANN)
- Ma et al. (2025): "Graph-Based Approximate Nearest Neighbor Search Revisited" (OPT-SNG)
- Li et al. (2025): "SAQ: Pushing the Limits of Vector Quantization" (SAQ)
- Zandieh et al. (2025): "TurboQuant: Online Vector Quantization" (TurboQuant)
- Dasgupta & Freund (2008): "Random projection trees and low dimensional manifolds" (RP-Trees)
- EVōC: https://github.com/TutteInstitute/evoc (Embedding Vector Oriented Clustering)
