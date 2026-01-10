# Seed Selection and Neighborhood Diversification Guide

## Overview

Based on the 2025 experimental evaluation of graph-based vector search methods, this guide explains how to choose seed selection and neighborhood diversification strategies for optimal performance.

## Seed Selection Strategies

### StackedNSW (SN) - Default

**Best for**: Billion-scale datasets

**How it works**: Uses hierarchical multi-resolution graphs. Starts from entry point in highest layer, navigates down layer by layer.

**Performance**:
- Logarithmic adaptation to dataset growth
- Best scalability on large datasets
- Higher indexing overhead (182M-22.3B more distance calculations than KS)

**Usage**:
```rust
use rank_retrieve::dense::hnsw::{HNSWIndex, HNSWParams, SeedSelectionStrategy};

let params = HNSWParams {
    seed_selection: SeedSelectionStrategy::StackedNSW,  // Default
    ..Default::default()
};
```

### KSampledRandom (KS)

**Best for**: Medium-scale datasets (1M-25GB)

**How it works**: Samples k random nodes per query, uses closest as entry point.

**Performance**:
- Lower indexing overhead
- Faster indexing (can answer 45K-1.17M queries before SN finishes construction)
- Requires more samples on large datasets

**Usage**:
```rust
use rank_retrieve::dense::hnsw::{HNSWIndex, HNSWParams, SeedSelectionStrategy};

let params = HNSWParams {
    seed_selection: SeedSelectionStrategy::KSampledRandom { k: 50 },  // Sample 50 random seeds
    ..Default::default()
};
```

### Recommendations

- **Billion-scale (1B+ vectors)**: Use StackedNSW
- **Medium-scale (1M-25GB)**: Use KSampledRandom
- **Small-scale (<1M)**: Either works well

## Neighborhood Diversification Strategies

### Relative Neighborhood Diversification (RND) - Default

**Best overall performance**

**How it works**: Formula: `dist(X_q, X_j) < dist(X_i, X_j)` for all neighbors X_i

**Performance**:
- Highest pruning ratios (20-25%)
- Best query performance
- Smallest graph sizes
- Lowest memory usage

**Usage**:
```rust
use rank_retrieve::dense::hnsw::{HNSWIndex, HNSWParams, NeighborhoodDiversification};

let params = HNSWParams {
    neighborhood_diversification: NeighborhoodDiversification::RelativeNeighborhood,  // Default
    ..Default::default()
};
```

### Maximum-Oriented Neighborhood Diversification (MOND)

**Second-best performance**

**How it works**: Maximizes angles between neighbors. Formula: `∠(X_j X_q X_i) > θ` (typically θ ≥ 60°)

**Performance**:
- Moderate pruning (2-4%)
- Second-best query performance
- Angle-based diversification

**Usage**:
```rust
use rank_retrieve::dense::hnsw::{HNSWIndex, HNSWParams, NeighborhoodDiversification};

let params = HNSWParams {
    neighborhood_diversification: NeighborhoodDiversification::MaximumOriented {
        min_angle_degrees: 60.0,  // 60° minimum angle
    },
    ..Default::default()
};
```

### Relaxed Relative Neighborhood Diversification (RRND)

**Less effective, used by Vamana**

**How it works**: Formula: `dist(X_q, X_j) < α · dist(X_i, X_j)` with α ≥ 1.5

**Performance**:
- Lowest pruning (0.6-0.7%)
- Creates larger graphs
- Higher memory usage
- Used in Vamana's two-pass construction

**Usage**:
```rust
use rank_retrieve::dense::hnsw::{HNSWIndex, HNSWParams, NeighborhoodDiversification};

let params = HNSWParams {
    neighborhood_diversification: NeighborhoodDiversification::RelaxedRelative {
        alpha: 1.3,  // Relaxation factor (typically 1.3-1.5)
    },
    ..Default::default()
};
```

### Recommendations

- **Always use ND**: Significant performance improvement, especially at scale
- **Default choice**: RND (best performance, highest pruning)
- **Alternative**: MOND (second-best, angle-based)
- **Avoid**: RRND unless specifically needed (creates larger graphs)

## Combined Recommendations

### Small to Medium Datasets (1M-25GB)
```rust
let params = HNSWParams {
    seed_selection: SeedSelectionStrategy::KSampledRandom { k: 50 },
    neighborhood_diversification: NeighborhoodDiversification::RelativeNeighborhood,
    ..Default::default()
};
```

### Large Datasets (100GB-1B)
```rust
let params = HNSWParams {
    seed_selection: SeedSelectionStrategy::StackedNSW,  // Default
    neighborhood_diversification: NeighborhoodDiversification::RelativeNeighborhood,  // Default
    ..Default::default()
};
```

### Hard Datasets/Workloads (High LID, Low LRC)
```rust
// Consider DC-based methods (ELPIS, SPTAG) for hard datasets
// For HNSW, use default RND + SN
let params = HNSWParams {
    seed_selection: SeedSelectionStrategy::StackedNSW,
    neighborhood_diversification: NeighborhoodDiversification::RelativeNeighborhood,
    ..Default::default()
};
```

## Performance Impact

Based on 2025 research:

**Seed Selection**:
- SN vs KS: ~1M-10M difference in distance calculations for 0.99 recall on 1B datasets
- KS indexing: 45K-1.17M queries can be answered before SN finishes construction

**Neighborhood Diversification**:
- RND: 20-25% pruning, best performance
- MOND: 2-4% pruning, second-best
- RRND: 0.6-0.7% pruning, less effective
- NoND: Worst performance, especially at scale

## References

- Azizi, I., Echihabi, K., & Palpanas, T. (2025). Graph-Based Vector Search: An Experimental Evaluation of the State-of-the-Art. *Proc. ACM Manag. Data*, 3(1), Article 43.
