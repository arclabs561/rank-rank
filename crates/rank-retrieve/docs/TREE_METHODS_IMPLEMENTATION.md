# Tree-Based ANN Methods Implementation

## Overview

Three classic tree-based ANN methods have been implemented:

1. **KD-Tree** - Space-partitioning tree for low dimensions (d < 20)
2. **Ball Tree** - Hypersphere-based partitioning for medium dimensions (20 < d < 100)
3. **Random Projection Tree** - Single RP-Tree with random hyperplane splits

## KD-Tree

**Location**: `src/dense/classic/trees/kdtree.rs`

**Algorithm**:
- Recursive space partitioning by alternating dimensions
- Each node splits space along one dimension
- Median-based splitting for balanced trees

**Best For**:
- Low-dimensional data (d < 20)
- Exact nearest neighbor search in low dimensions
- Educational/baseline comparisons

**Limitations**:
- Performance degrades in high dimensions (curse of dimensionality)
- Not recommended for d > 50

**Usage**:
```rust
use rank_retrieve::dense::classic::trees::kdtree::{KDTreeIndex, KDTreeParams};

let mut index = KDTreeIndex::new(16, KDTreeParams::default())?;
index.add(0, vec![0.1; 16])?;
index.build()?;
let results = index.search(&vec![0.15; 16], 10)?;
```

## Ball Tree

**Location**: `src/dense/classic/trees/balltree.rs`

**Algorithm**:
- Recursive space partitioning using hyperspheres (balls)
- Each node represents a ball (center + radius) containing its vectors
- Farthest-pair splitting for tree construction

**Best For**:
- Medium-dimensional data (20 < d < 100)
- Better than KD-Tree for medium dimensions
- More robust to high-dimensional data

**Usage**:
```rust
use rank_retrieve::dense::classic::trees::balltree::{BallTreeIndex, BallTreeParams};

let mut index = BallTreeIndex::new(64, BallTreeParams::default())?;
index.add(0, vec![0.1; 64])?;
index.build()?;
let results = index.search(&vec![0.15; 64], 10)?;
```

## Random Projection Tree

**Location**: `src/dense/classic/trees/random_projection.rs`

**Algorithm**:
- Single tree using random hyperplane splits
- Each node splits space with a random hyperplane
- Simpler than Random Projection Tree Forest (vendor: Annoy, uses multiple RP-Trees)

**Best For**:
- Baseline comparisons
- Foundation for RP-Tree Forest (vendor: Annoy)
- General-purpose approximate search

**Usage**:
```rust
use rank_retrieve::dense::classic::trees::random_projection::{RPTreeIndex, RPTreeParams};

let mut index = RPTreeIndex::new(128, RPTreeParams::default())?;
index.add(0, vec![0.1; 128])?;
index.build()?;
let results = index.search(&vec![0.15; 128], 10)?;
```

## Relationships

### Tree Methods Hierarchy

```
Tree-Based ANN Methods
├── KD-Tree (d < 20)
│   └── Dimension-aligned splits
├── Ball Tree (20 < d < 100)
│   └── Hypersphere-based splits
└── Random Projection Tree
    ├── Single RP-Tree (this implementation)
    └── RP-Tree Forest (vendor: Annoy - multiple trees)
```

### Comparison with Other Methods

- **vs Graph-based (HNSW, SNG)**: Trees use space partitioning, graphs use proximity relationships
- **vs Hash-based (LSH)**: Trees provide deterministic structure, LSH uses probabilistic hashing
- **vs Quantization (IVF-PQ, Anisotropic VQ + k-means (SCANN))**: Trees don't compress vectors, quantization methods do

## Implementation Details

### Common Features

All three tree methods:
- Use SIMD-accelerated distance computation
- Support SoA (Structure of Arrays) vector storage
- Implement unified `ANNIndex` trait
- Feature-gated for minimal builds

### Tree Construction

- **KD-Tree**: Alternating dimension splits, median-based
- **Ball Tree**: Farthest-pair splitting, ball (center + radius) representation
- **RP-Tree**: Random hyperplane splits, median projection threshold

### Search Algorithm

All methods:
1. Traverse tree from root to leaves
2. Collect candidate vectors from visited nodes
3. Compute distances and return top-k

(Pruning optimizations can be added for better performance)

## Performance Characteristics

| Method | Construction | Search | Best Dimension Range | Memory |
|--------|--------------|--------|---------------------|--------|
| KD-Tree | O(n log n) | O(log n) | d < 20 | Low |
| Ball Tree | O(n log n) | O(log n) | 20 < d < 100 | Low |
| RP-Tree | O(n log n) | O(log n) | All | Low |

## References

- Bentley (1975): "Multidimensional binary search trees used for associative searching"
- Omohundro (1989): "Five balltree construction algorithms"
- Dasgupta & Freund (2008): "Random projection trees and low dimensional manifolds"
- Friedman et al. (1977): "An algorithm for finding best matches in logarithmic expected time"
