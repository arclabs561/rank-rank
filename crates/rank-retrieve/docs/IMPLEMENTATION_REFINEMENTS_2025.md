# Implementation Refinements Based on 2025 Research

## Overview

This document summarizes the refinements made to `rank-retrieve` based on the 2025 experimental evaluation of graph-based vector search methods (Azizi et al., 2025).

## Completed Refinements

### 1. Seed Selection Strategies ✅

**Implementation**: Added `SeedSelectionStrategy` enum to `HNSWParams`

**Strategies**:
- **StackedNSW (SN)**: Default, best for billion-scale datasets
- **KSampledRandom (KS)**: Best for medium-scale (1M-25GB) with lower indexing overhead

**Files Modified**:
- `src/dense/hnsw/graph.rs`: Added enum and search logic
- `src/dense/hnsw/search.rs`: Made SearchState public for KS implementation
- `examples/hnsw_seed_selection.rs`: Example demonstrating both strategies

**Tests**: `tests/hnsw_seed_selection_tests.rs`

### 2. Neighborhood Diversification Strategies ✅

**Implementation**: Added `NeighborhoodDiversification` enum to `HNSWParams`

**Strategies**:
- **RND (Relative Neighborhood)**: Default, best performance (20-25% pruning)
- **MOND (Maximum-Oriented)**: Second-best, angle-based (2-4% pruning)
- **RRND (Relaxed Relative)**: Used by Vamana, less effective (0.6-0.7% pruning)

**Files Modified**:
- `src/dense/hnsw/construction.rs`: Implemented RND, MOND, RRND algorithms
- `src/dense/hnsw/graph.rs`: Added enum to HNSWParams

**Tests**: `tests/hnsw_seed_selection_tests.rs`

### 3. Vamana Implementation ✅

**Implementation**: Complete two-pass construction with RRND + RND

**Components**:
- `src/dense/vamana/graph.rs`: Core types and index structure
- `src/dense/vamana/construction.rs`: Two-pass construction algorithm
- `src/dense/vamana/search.rs`: Beam search algorithm

**Features**:
- Random graph initialization (degree ≥ log(n))
- First pass: RRND refinement
- Second pass: RND refinement
- Beam search (similar to HNSW but without hierarchy)

**Files Created**:
- `src/dense/vamana/mod.rs`
- `src/dense/vamana/graph.rs`
- `src/dense/vamana/construction.rs`
- `src/dense/vamana/search.rs`
- `examples/vamana_basic.rs`
- `tests/vamana_tests.rs`

### 4. Documentation ✅

**New Documents**:
- `docs/GRAPH_BASED_VECTOR_SEARCH_SURVEY.md`: Comprehensive research synthesis
- `docs/SEED_SELECTION_AND_ND_GUIDE.md`: Usage guide for seed selection and ND strategies
- `docs/VAMANA_IMPLEMENTATION.md`: Vamana implementation details

**Updated Documents**:
- `README.md`: Added links to new documentation
- `src/dense.rs`: Added Vamana to feature list

## Performance Impact

### Seed Selection

- **SN vs KS**: ~1M-10M difference in distance calculations for 0.99 recall on 1B datasets
- **KS indexing**: 45K-1.17M queries can be answered before SN finishes construction
- **Recommendation**: Use SN for billion-scale, KS for medium-scale

### Neighborhood Diversification

- **RND**: 20-25% pruning, best performance, smallest graphs
- **MOND**: 2-4% pruning, second-best, angle-based
- **RRND**: 0.6-0.7% pruning, less effective, used by Vamana
- **NoND**: Worst performance, especially at scale
- **Recommendation**: Always use ND, default to RND

### Vamana

- **Indexing time**: Third fastest (after ELPIS and HNSW)
- **Query performance**: Competitive with HNSW
- **Memory**: 30% more than ELPIS, less than HNSW
- **SSD serving**: Better than HNSW (5-10× more points/node)
- **Recommendation**: Use for SSD-based serving or when two-pass quality is important

## Usage Examples

### HNSW with Custom Seed Selection and ND

```rust
use rank_retrieve::dense::hnsw::{HNSWIndex, HNSWParams, SeedSelectionStrategy, NeighborhoodDiversification};

// Medium-scale dataset: Use KS + RND
let params = HNSWParams {
    seed_selection: SeedSelectionStrategy::KSampledRandom { k: 50 },
    neighborhood_diversification: NeighborhoodDiversification::RelativeNeighborhood,
    ..Default::default()
};

// Large-scale dataset: Use SN + RND (default)
let params = HNSWParams {
    seed_selection: SeedSelectionStrategy::StackedNSW,
    neighborhood_diversification: NeighborhoodDiversification::RelativeNeighborhood,
    ..Default::default()
};
```

### Vamana

```rust
use rank_retrieve::dense::vamana::{VamanaIndex, VamanaParams};

let params = VamanaParams {
    max_degree: 64,
    alpha: 1.3,
    ef_construction: 200,
    ef_search: 50,
};

let mut index = VamanaIndex::new(128, params)?;
// ... add vectors ...
index.build()?;  // Two-pass construction
let results = index.search(&query, 10, 50)?;
```

## Testing

All new features are covered by tests:

- `tests/hnsw_seed_selection_tests.rs`: Tests for all seed selection and ND strategies
- `tests/vamana_tests.rs`: Tests for Vamana construction and search

## Future Work

### High Priority

1. **ELPIS Implementation**: Best overall performer (DC + II + ND)
   - Requires Hercules tree + HNSW per partition
   - Complex but highest performance on hard datasets

2. **Benchmarks**: Compare seed selection and ND strategies
   - Measure indexing time differences
   - Measure query performance differences
   - Validate pruning ratios

3. **Property Tests**: Verify ND strategies
   - RND pruning ratio (should be 20-25%)
   - MOND angle constraints (should be ≥ 60°)
   - RRND relaxation factor effects

### Medium Priority

4. **NSG Implementation**: Another graph-based method
5. **SSG Implementation**: MOND-based method
6. **DPG Implementation**: MOND-based method

## References

- Azizi, I., Echihabi, K., & Palpanas, T. (2025). Graph-Based Vector Search: An Experimental Evaluation of the State-of-the-Art. *Proc. ACM Manag. Data*, 3(1), Article 43.
