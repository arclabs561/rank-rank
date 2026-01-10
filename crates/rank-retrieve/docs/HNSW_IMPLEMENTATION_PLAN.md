# HNSW Implementation Plan: Pure Rust Optimized ANN

Comprehensive plan for implementing Hierarchical Navigable Small World (HNSW) approximate nearest neighbor search in pure Rust, optimized for performance.

## Goals

1. **Pure Rust implementation** - No FFI dependencies, fully native
2. **SIMD-accelerated** - Leverage existing `simd` module for distance computation
3. **Cache-optimized** - Memory layout designed for CPU cache efficiency
4. **Production-ready** - Handle millions of vectors efficiently
5. **Feature-gated** - Optional backend, doesn't bloat default builds

## Algorithm Overview

HNSW (Malkov & Yashunin, 2016) constructs a multi-layer graph where:
- **Upper layers**: Sparse, long-range connections for fast navigation
- **Lower layers**: Dense, local connections for precise search
- **Search**: Start at top layer, navigate down to base layer, greedy search

**Key parameters:**
- `m` - Maximum connections per node (typically 16)
- `m_max` - Maximum connections for new nodes (typically 16)
- `m_L` - Layer assignment probability (typically 1/ln(2) ≈ 1.44)
- `ef_construction` - Search width during construction (typically 200)
- `ef_search` - Search width during query (typically 50-200)

## Research-Based Optimizations

### 1. SIMD-Accelerated Distance Computation

**Current state:** `rank-retrieve` already has SIMD-accelerated dot product (AVX-512/AVX2/NEON).

**Optimization:** Use existing `simd::dot()` for cosine similarity in HNSW distance calculations.

**Implementation:**
```rust
// In HNSW search/construction
use crate::simd;

fn distance(a: &[f32], b: &[f32]) -> f32 {
    // For normalized vectors: cosine = dot product
    // Use existing SIMD-accelerated dot product
    1.0 - simd::dot(a, b)  // Convert similarity to distance
}
```

**Expected speedup:** 8-16x for distance computation (already proven in existing SIMD code).

### 2. Memory Layout Optimization

**Problem:** Graph structure can cause cache misses during traversal.

**Optimizations:**

1. **SoA (Structure of Arrays) for vectors:**
   - Store all vectors contiguously: `Vec<Vec<f32>>` → `Vec<f32>` with stride
   - Enables SIMD-friendly batch distance computation
   - Better cache locality when accessing multiple vectors

2. **Compact neighbor lists:**
   - Use `SmallVec<[u32; 16]>` for neighbor lists (most nodes have <16 neighbors)
   - Avoid heap allocations for typical case
   - Better cache locality

3. **Layer storage:**
   - Store layer assignments compactly (u8 for layer number)
   - Separate arrays per layer for cache-friendly access

**Implementation:**
```rust
struct HNSWIndex {
    // SoA: All vectors stored contiguously
    vectors: Vec<f32>,  // Flattened: [v0[0..d], v1[0..d], ...]
    dimension: usize,
    
    // Graph structure: one array per layer
    layers: Vec<Layer>,
    
    // Layer assignments: compact storage
    layer_assignments: Vec<u8>,
}

struct Layer {
    // Neighbor lists: SmallVec for cache efficiency
    neighbors: Vec<SmallVec<[u32; 16]>>,
}
```

### 3. Early Termination Strategies

**Research findings:**
- Distance threshold-based termination
- Dynamic quality-based termination
- Saturation-based termination ("Patience in Proximity")

**Implementation:**
```rust
struct SearchState {
    candidates: BinaryHeap<Candidate>,  // Min-heap by distance
    visited: HashSet<u32>,
    best_distance: f32,
    no_improvement_count: usize,
}

fn search_with_early_termination(
    &self,
    query: &[f32],
    k: usize,
    ef: usize,
) -> Vec<(u32, f32)> {
    let mut state = SearchState::new();
    
    // Early termination conditions:
    // 1. Distance threshold: if best_distance < threshold, stop
    // 2. Saturation: if no improvement for N iterations, stop
    // 3. Quality convergence: if improvement rate < threshold, stop
}
```

### 4. Neighbor Selection Optimization

**Research:** RNG (Relative Neighborhood Graph) based selection outperforms naive closest-neighbor selection.

**Current approach (naive):** Select M closest neighbors.

**Optimized approach:** Use RNG-based selection to ensure diversity:
- Avoid clustering neighbors too close together
- Ensure good graph connectivity
- Reduce local minima in search

**Implementation:**
```rust
fn select_neighbors_rng(
    candidates: &[Candidate],
    m: usize,
) -> Vec<u32> {
    // RNG selection: prefer diverse neighbors
    // Avoid selecting neighbors that are too similar to each other
    // Ensures better graph connectivity
}
```

### 5. Batch Distance Computation

**Optimization:** Compute distances to multiple candidates in batch using SIMD.

**Implementation:**
```rust
fn batch_distance_simd(
    query: &[f32],
    candidates: &[&[f32]],
) -> Vec<f32> {
    // Process multiple candidates in parallel using SIMD
    // Better instruction-level parallelism
    // Reduces function call overhead
}
```

### 6. Graph Construction Optimizations

**Optimizations:**
1. **Parallel insertion:** Use rayon for parallel graph construction (if safe)
2. **Incremental construction:** Support adding vectors after initial build
3. **Reverse connections:** Enhance connectivity by adding reverse edges

## Architecture Design

### Module Structure

```
crates/rank-retrieve/src/
├── dense/
│   ├── mod.rs              # Existing brute-force (keep as fallback)
│   └── hnsw/
│       ├── mod.rs          # Public API
│       ├── graph.rs        # Graph structure
│       ├── search.rs       # Search algorithm
│       ├── construction.rs # Graph construction
│       ├── memory.rs       # Memory layout optimizations
│       └── distance.rs     # Distance computation (uses simd)
```

### Core Types

```rust
// Feature-gated module
#[cfg(feature = "hnsw")]
pub mod hnsw {
    use crate::simd;
    use smallvec::SmallVec;
    
    pub struct HNSWIndex {
        vectors: Vec<f32>,           // SoA storage
        dimension: usize,
        layers: Vec<Layer>,
        layer_assignments: Vec<u8>,
        m: usize,                    // Max connections
        m_max: usize,                 // Max connections for new nodes
        ef_construction: usize,
    }
    
    struct Layer {
        neighbors: Vec<SmallVec<[u32; 16]>>,
    }
    
    struct Candidate {
        id: u32,
        distance: f32,
    }
}
```

### API Design

```rust
impl HNSWIndex {
    /// Create new HNSW index
    pub fn new(dimension: usize, m: usize, m_max: usize) -> Self;
    
    /// Add vector to index (incremental construction)
    pub fn add(&mut self, doc_id: u32, vector: Vec<f32>) -> Result<(), RetrieveError>;
    
    /// Build index (required before search)
    pub fn build(&mut self) -> Result<(), RetrieveError>;
    
    /// Search for k nearest neighbors
    pub fn search(
        &self,
        query: &[f32],
        k: usize,
        ef: usize,
    ) -> Result<Vec<(u32, f32)>, RetrieveError>;
}
```

### Integration with Existing Code

```rust
// In dense.rs - add HNSW variant
#[cfg(feature = "hnsw")]
use crate::dense::hnsw::HNSWIndex;

// Make HNSW implement Backend trait
#[cfg(feature = "hnsw")]
impl crate::integration::Backend for HNSWIndex {
    fn retrieve(&self, query: &[f32], k: usize) -> Result<Vec<(u32, f32)>, RetrieveError> {
        self.search(query, k, self.ef_search)
    }
    
    fn add_document(&mut self, doc_id: u32, embedding: &[f32]) -> Result<(), RetrieveError> {
        self.add(doc_id, embedding.to_vec())
    }
    
    fn build(&mut self) -> Result<(), RetrieveError> {
        self.build()
    }
}
```

## Implementation Phases

### Phase 1: Core Algorithm (Week 1-2)

**Goal:** Working HNSW implementation with correct algorithm.

**Tasks:**
1. Implement graph structure (layers, neighbors)
2. Implement layer assignment (exponential distribution)
3. Implement basic search (greedy search, no optimizations)
4. Implement basic construction (insertion algorithm)
5. Unit tests for correctness

**Success criteria:**
- Correct results on small datasets (<1000 vectors)
- Matches brute-force results (within approximation tolerance)

### Phase 2: SIMD Integration (Week 2-3)

**Goal:** Use existing SIMD infrastructure for distance computation.

**Tasks:**
1. Integrate `simd::dot()` for distance computation
2. Batch distance computation where possible
3. Benchmark SIMD vs scalar performance

**Success criteria:**
- 8-16x speedup for distance computation
- Correct results maintained

### Phase 3: Memory Optimizations (Week 3-4)

**Goal:** Optimize memory layout for cache efficiency.

**Tasks:**
1. Convert to SoA (Structure of Arrays) for vectors
2. Use SmallVec for neighbor lists
3. Optimize layer storage
4. Profile cache misses (perf, cachegrind)

**Success criteria:**
- Reduced cache misses (measured)
- Faster search on large datasets

### Phase 4: Search Optimizations (Week 4-5)

**Goal:** Implement early termination and search optimizations.

**Tasks:**
1. Implement early termination strategies
2. Optimize candidate management (better data structures)
3. RNG-based neighbor selection
4. Tune parameters (m, ef_construction, ef_search)

**Success criteria:**
- Faster search with same recall
- Configurable quality/speed tradeoff

### Phase 5: Integration & Benchmarking (Week 5-6)

**Goal:** Integrate with rank-retrieve and benchmark.

**Tasks:**
1. Implement Backend trait
2. Feature gating
3. Comprehensive benchmarks (vs brute-force, vs other implementations)
4. Documentation

**Success criteria:**
- Feature-gated integration works
- Benchmarks show significant speedup
- Documentation complete

## Feature Gating

```toml
# Cargo.toml
[dependencies]
smallvec = { version = "1.11", optional = true }

[features]
default = []
dense = []  # Basic brute-force (current)
hnsw = ["dep:smallvec"]  # HNSW implementation
```

**Usage:**
```rust
// Default: brute-force only
use rank_retrieve::dense::DenseRetriever;

// With HNSW feature
use rank_retrieve::dense::hnsw::HNSWIndex;
```

## Performance Targets

**Search performance:**
- <1ms for 1M vectors, k=10, ef=50 (on modern CPU)
- 95%+ recall vs brute-force
- O(log n) complexity (verified with scaling tests)

**Memory efficiency:**
- <2x memory overhead vs brute-force (graph structure)
- Cache-friendly access patterns

**Construction:**
- <1s for 100K vectors (single-threaded)
- Incremental construction supported

## Testing Strategy

1. **Correctness tests:**
   - Small datasets: verify exact match with brute-force
   - Large datasets: verify recall (95%+ of top-k found)

2. **Performance tests:**
   - Benchmark vs brute-force (speedup measurement)
   - Benchmark vs other implementations (if available)
   - Scaling tests (1K, 10K, 100K, 1M vectors)

3. **Property tests:**
   - Graph invariants (max connections, layer structure)
   - Search correctness (monotonicity, completeness)

## Research References

1. **Original HNSW paper:** Malkov & Yashunin (2016) - "Efficient and robust approximate nearest neighbor search using Hierarchical Navigable Small World graphs"
2. **Early termination:** Elastic Search Labs blog on HNSW early termination
3. **RNG selection:** Research on Relative Neighborhood Graph for better connectivity
4. **Memory optimization:** Cache-friendly data structures for graph algorithms

## Next Steps

1. Start with Phase 1: Core algorithm implementation
2. Use existing SIMD code from `simd.rs`
3. Iterate on optimizations based on profiling
4. Benchmark against brute-force and other implementations
