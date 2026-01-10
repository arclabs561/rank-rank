# Classic ANN Methods Implementation Plan

This document outlines classic/traditional approximate nearest neighbor search algorithms that are still relevant and worth implementing alongside modern methods.

## Why Implement Classic Methods?

1. **Baseline Comparisons**: Essential for benchmarking and validating new methods
2. **Specific Use Cases**: Some methods excel in particular scenarios
3. **Educational Value**: Understanding evolution of ANN algorithms
4. **Completeness**: Comprehensive ANN library should include both classic and modern methods
5. **Hybrid Approaches**: Classic methods can complement modern ones

## Classic Methods to Implement

### 1. LSH (Locality Sensitive Hashing) ⭐⭐⭐⭐

**Priority**: High

**Why**:
- **Theoretical guarantees**: Provable approximation bounds
- **Fast construction**: O(n) construction time
- **Memory efficient**: Hash-based indexing
- **Still used**: Industry standard for certain applications
- **Good for**: High-dimensional data, when exact recall isn't critical

**Key Variants**:
- **Random Projection LSH** (for cosine similarity)
- **E2LSH** (for L2 distance)
- **MinHash LSH** (for Jaccard similarity)
- **LSH Forest** (adaptive parameter selection)

**Complexity**:
- Construction: O(n)
- Search: O(n^ρ) where ρ < 1 (depends on hash family)
- Memory: O(n)

**Implementation Notes**:
- Multiple hash functions (hash family)
- Hash table indexing
- Candidate generation and verification
- Parameter tuning (number of hash functions, hash table size)

**References**:
- Indyk & Motwani (1998): "Approximate nearest neighbors: towards removing the curse of dimensionality"
- Datar et al. (2004): "Locality-sensitive hashing scheme based on p-stable distributions"

---

### 2. Random Projection (RP-Trees) ⭐⭐⭐

**Priority**: Medium

**Why**:
- **Simple**: Easy to implement and understand
- **Fast**: O(log n) search in expectation
- **Good baseline**: Useful for comparison
- **Low memory**: Tree structure is compact

**Key Features**:
- Random hyperplane splits
- Binary tree structure
- Multiple trees for better recall

**Complexity**:
- Construction: O(n log n)
- Search: O(log n) per tree
- Memory: O(n)

**Implementation Notes**:
- Random hyperplane generation
- Tree construction with splits
- Forest of trees for better recall
- SIMD-accelerated distance computation

**References**:
- Dasgupta & Freund (2008): "Random projection trees and low dimensional manifolds"

---

### 3. KD-Tree (K-Dimensional Tree) ⭐⭐⭐

**Priority**: Medium

**Why**:
- **Classic method**: One of the earliest ANN methods
- **Good for low dimensions**: Excellent for d < 20
- **Exact search**: Can be used for exact NN in low dimensions
- **Educational**: Fundamental data structure

**Limitations**:
- **Curse of dimensionality**: Performance degrades in high dimensions
- **Not ideal for d > 20**: Modern methods outperform

**Complexity**:
- Construction: O(n log n)
- Search: O(log n) for low d, O(n) for high d
- Memory: O(n)

**Implementation Notes**:
- Recursive space partitioning
- Alternating dimension splits
- Nearest neighbor search with backtracking
- Approximate variants (early termination)

**References**:
- Bentley (1975): "Multidimensional binary search trees used for associative searching"

---

### 4. Ball Tree ⭐⭐⭐

**Priority**: Medium

**Why**:
- **Better than KD-tree** for high dimensions
- **Metric spaces**: Works with any metric
- **Good for**: Medium-dimensional data (20 < d < 100)

**Key Features**:
- Hierarchical ball partitioning
- Metric-based (not just Euclidean)
- Better than KD-tree for high d

**Complexity**:
- Construction: O(n log n)
- Search: O(log n) for medium d
- Memory: O(n)

**Implementation Notes**:
- Ball construction (centroid + radius)
- Recursive partitioning
- Metric distance computation
- Approximate search variants

**References**:
- Omohundro (1989): "Five balltree construction algorithms"

---

### 5. Random Projection Tree Forest (vendor: Annoy) ⭐⭐⭐⭐

**Priority**: High

**Why**:
- **Production-proven**: Battle-tested in production (Spotify's Annoy)
- **Memory-mapped**: Can work with disk-backed data
- **Simple API**: Easy to use
- **Good performance**: Competitive with modern methods
- **Still relevant**: Used in production systems

**Key Features**:
- Random projection trees
- Forest of trees
- Memory-mapped files
- Thread-safe search

**Complexity**:
- Construction: O(n log n)
- Search: O(log n) per tree
- Memory: O(n)

**Implementation Notes**:
- Random projection tree forest
- Memory-mapped index support
- Thread-safe operations
- SIMD distance computation

**References**:
- Spotify Engineering Blog: "Annoy: Approximate Nearest Neighbors in C++/Python"

---

### 6. FLANN (Fast Library for Approximate Nearest Neighbors) Methods ⭐⭐⭐

**Priority**: Medium

**Why**:
- **Auto-tuning**: Automatically selects best method
- **Multiple algorithms**: KD-tree, K-means tree, hierarchical k-means
- **Good for**: When you don't know which method to use

**Key Algorithms**:
- **KD-tree**: For low dimensions
- **K-means tree**: Hierarchical clustering
- **Composite index**: Combines multiple methods

**Complexity**: Varies by method

**Implementation Notes**:
- Multiple algorithm implementations
- Auto-tuning framework
- Performance-based method selection
- Can be complex to implement fully

**References**:
- Muja & Lowe (2009): "Fast approximate nearest neighbors with automatic algorithm configuration"

---

### 7. Random Sampling / Brute-Force Variants ⭐⭐

**Priority**: Low

**Why**:
- **Baseline**: Essential for benchmarking
- **Simple**: Already have brute-force, but can add sampling variants
- **Educational**: Understanding trade-offs

**Variants**:
- Random sampling (sample subset)
- Block-based brute force
- SIMD-accelerated brute force (already have this)

---

## Implementation Priority

### Phase 1: High Priority (Immediate)
1. **LSH** - Still widely used, theoretical guarantees
2. **Random Projection Tree Forest (vendor: Annoy)** - Production-proven, simple API

### Phase 2: Medium Priority (Next)
3. **Random Projection Trees** - Good baseline, simple
4. **KD-Tree** - Classic method, good for low dimensions
5. **Ball Tree** - Better than KD-tree for medium dimensions

### Phase 3: Lower Priority (Later)
6. **FLANN Methods** - Complex, but useful for auto-tuning
7. **Sampling Variants** - Enhance existing brute-force

---

## Architecture

### Module Structure

```
crates/rank-retrieve/src/dense/classic/
├── mod.rs              # Classic ANN methods
├── lsh/                # Locality Sensitive Hashing
│   ├── mod.rs
│   ├── random_projection.rs  # Random projection LSH
│   ├── e2lsh.rs              # E2LSH for L2
│   ├── minhash.rs            # MinHash LSH
│   └── forest.rs             # LSH Forest
├── trees/              # Tree-based methods
│   ├── mod.rs
│   ├── random_projection.rs  # RP-Trees
│   ├── kdtree.rs             # KD-Tree
│   ├── balltree.rs           # Ball Tree
│   └── annoy.rs              # Annoy implementation
└── flann/              # FLANN methods (optional)
    ├── mod.rs
    ├── kmeans_tree.rs
    └── composite.rs
```

### Unified API

All classic methods implement the same `ANNIndex` trait:

```rust
use rank_retrieve::dense::classic::{LSHIndex, AnnoyIndex, KDTreeIndex};

// LSH
let mut lsh = LSHIndex::new(128, LSHParams::default())?;
lsh.add(0, vec![0.1; 128])?;
lsh.build()?;
let results = lsh.search(&vec![0.15; 128], 10)?;

// Random Projection Tree Forest (vendor: Annoy)
let mut annoy = AnnoyIndex::new(128, AnnoyParams::default())?;
annoy.add(0, vec![0.1; 128])?;
annoy.build()?;
let results = annoy.search(&vec![0.15; 128], 10)?;
```

---

## Performance Characteristics

| Method | Construction | Search | Recall | Memory | Best For |
|--------|--------------|--------|--------|--------|----------|
| LSH | O(n) | O(n^ρ) | Medium | Low | High-d, hash-based |
| Random Projection | O(n log n) | O(log n) | Medium | Low | General purpose |
| KD-Tree | O(n log n) | O(log n) | High (low d) | Low | d < 20 |
| Ball Tree | O(n log n) | O(log n) | High (med d) | Low | 20 < d < 100 |
| RP-Tree Forest (Annoy) | O(n log n) | O(log n) | High | Low | General purpose |
| HNSW | O(n log n) | O(log n) | Very High | Medium | General purpose (modern) |

---

## Use Cases

### When to Use Classic Methods

1. **LSH**:
   - High-dimensional data (d > 1000)
   - When exact recall isn't critical
   - Hash-based systems
   - Distributed systems (hash-based partitioning)

2. **KD-Tree**:
   - Low-dimensional data (d < 20)
   - Exact nearest neighbor needed
   - Educational/research purposes

3. **Ball Tree**:
   - Medium-dimensional data (20 < d < 100)
   - Non-Euclidean metrics
   - When KD-tree fails

4. **Random Projection Tree Forest (vendor: Annoy)**:
   - Production systems (battle-tested)
   - Memory-mapped indices needed
   - Simple API requirements
   - Medium-scale datasets

5. **Random Projection**:
   - Baseline comparisons
   - Simple implementations needed
   - Educational purposes

### When to Use Modern Methods

- **HNSW**: Best general-purpose, high recall
- **Anisotropic VQ + k-means (SCANN)**: MIPS, very large datasets
- **IVF-PQ**: Billion-scale, memory-constrained
- **DiskANN**: Very large, disk-based

---

## Implementation Strategy

### Sprint 1: LSH
- Random projection LSH (cosine similarity)
- Hash table indexing
- Candidate generation
- **Time**: 1 week

### Sprint 2: Annoy
- Random projection tree forest
- Memory-mapped support (optional)
- Thread-safe search
- **Time**: 1 week

### Sprint 3: Tree Methods
- KD-Tree (low dimensions)
- Ball Tree (medium dimensions)
- Random Projection Trees
- **Time**: 1-2 weeks

### Sprint 4: Integration & Testing
- Unified API
- Benchmarks vs modern methods
- Documentation
- **Time**: 1 week

---

## Dependencies

**Minimal dependencies** (same as modern methods):
- `rand` (already have) - Random number generation
- `smallvec` (already have) - Small vector optimization
- No additional dependencies needed

---

## References

- Indyk & Motwani (1998): "Approximate nearest neighbors: towards removing the curse of dimensionality"
- Datar et al. (2004): "Locality-sensitive hashing scheme based on p-stable distributions"
- Dasgupta & Freund (2008): "Random projection trees and low dimensional manifolds"
- Bentley (1975): "Multidimensional binary search trees"
- Omohundro (1989): "Five balltree construction algorithms"
- Muja & Lowe (2009): "Fast approximate nearest neighbors with automatic algorithm configuration"
- Spotify Engineering: "Annoy: Approximate Nearest Neighbors" (Random Projection Tree Forest)
