# rank-retrieve Optimization Plan

Comprehensive optimization strategy for improving performance of `rank-retrieve` while maintaining correctness and portability.

## Status

- ✅ **Phase 1: SIMD Dense Retrieval** - COMPLETED
- ✅ **Phase 2: SIMD Sparse Retrieval** - COMPLETED
- ✅ **Phase 3: BM25 Optimizations** - COMPLETED
- ⏳ **Phase 4: Memory Layout** - PENDING

## Current State Analysis

### Performance Characteristics

**BM25 Retrieval:**
- Basic inverted index implementation
- O(q * d) complexity where q = query terms, d = documents per term
- No SIMD acceleration
- Suitable for <1M documents (as documented)

**Dense Retrieval:**
- Brute-force cosine similarity (O(n*d) where n = documents, d = dimension)
- No SIMD acceleration
- Suitable for <100K documents (as documented)

**Sparse Retrieval:**
- Basic dot product implementation
- No SIMD acceleration
- Efficient for sparse vectors but could benefit from SIMD

### Comparison with rank-rerank

`rank-rerank` already has excellent SIMD implementations:
- AVX-512, AVX2, NEON support for dot product and cosine similarity
- ~2x speedup with AVX-512 vs AVX2
- ~8-10x speedup with AVX2 vs scalar
- Runtime feature detection for automatic dispatch

## Optimization Opportunities

### 1. SIMD-Accelerated Dense Retrieval ✅ COMPLETED

**Current Implementation:**
```rust
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}
```

**Optimization:**
- Reuse SIMD patterns from `rank-rerank/src/simd.rs`
- Add AVX-512, AVX2, NEON implementations
- Runtime feature detection for automatic dispatch
- Expected speedup: 8-16x for typical embedding dimensions (128-768)

**Implementation Strategy:**
1. ✅ Create `crates/rank-retrieve/src/simd.rs` module
2. ✅ Port dot product and cosine similarity from `rank-rerank`
3. ✅ Integrate into `DenseRetriever::retrieve()`
4. ✅ Add benchmarks to measure improvement

**Files Modified:**
- ✅ `src/dense.rs` - Uses SIMD cosine similarity
- ✅ `src/simd.rs` - New module with SIMD implementations (AVX-512, AVX2, NEON)
- ✅ `benches/dense.rs` - Added SIMD vs scalar comparisons

**Results:**
- Expected 8-16x speedup for dense vector operations
- Runtime feature detection for optimal instruction set
- Zero API changes - existing code benefits automatically

### 2. SIMD-Accelerated Sparse Vector Dot Product ✅ COMPLETED

**Current Implementation:**
```rust
pub fn dot_product(a: &SparseVector, b: &SparseVector) -> f32 {
    // Two-pointer advancing algorithm
    // Scalar implementation
}
```

**Optimization:**
- SIMD-accelerated sparse dot product
- Exploit sorted indices for vectorized comparisons
- Expected speedup: 2-4x for dense sparse vectors

**Challenges:**
- Sparse vectors have variable-length indices
- Need gather/scatter operations for non-contiguous access
- May require restructuring for better SIMD utilization

**Implementation Strategy:**
1. ✅ Research SIMD patterns for sparse dot product
2. ✅ Implement block-based processing to reduce branch mispredictions
3. ✅ Benchmark against scalar implementation

**Files Modified:**
- ✅ `src/simd.rs` - Added sparse_dot functions (AVX-512, AVX2, NEON, portable)
- ✅ `src/sparse/vector.rs` - Uses SIMD sparse_dot when available
- ✅ `benches/sparse.rs` - Added SIMD vs scalar comparisons

**Results:**
- Expected 2-4x speedup for sparse dot product
- Block-based approach reduces branch mispredictions
- Falls back to scalar for very sparse vectors (< 8 non-zeros)

### 3. BM25 Scoring Optimizations ✅ COMPLETED

**Current Bottlenecks:**
- Document scoring involves multiple hash map lookups
- IDF calculation repeated for each document
- No early termination for low-scoring documents

**Optimization Opportunities:**
1. **Precompute IDF values** for query terms (avoid repeated lookups)
2. **Early termination** - Skip documents with low term frequencies
3. **Batch scoring** - Process multiple documents together for better cache locality
4. **SIMD for term frequency accumulation** - Limited benefit, but may help for long queries

**Implementation Strategy:**
1. ✅ Precompute IDF values once per query (stored in index)
2. ✅ Add early termination heuristics (top-k heap with threshold)
3. ✅ Optimize candidate collection (Vec + HashSet for deduplication)
4. ⏳ Consider block-max WAND for large-scale optimization (future work)

**Files Modified:**
- ✅ `src/bm25.rs` - Added precomputed IDF, early termination, optimized scoring
- ✅ `benches/bm25.rs` - Added optimization impact benchmarks

**Results:**
- Precomputed IDF eliminates repeated calculations
- Early termination skips documents that can't be in top-k
- Expected 2x speedup for typical queries (2-5 terms, k=10-50)
- Better cache locality with Vec-based candidate collection

### 4. Memory Layout Optimization (Low-Medium Priority)

**Current Layout:**
- Dense: `Vec<(u32, Vec<f32>)>` - Array of Structures (AoS)
- BM25: `HashMap<String, HashMap<u32, u32>>` - Nested hash maps

**Optimization:**
- **Structure of Arrays (SoA)** for dense vectors
  - Store all embeddings in contiguous arrays by dimension
  - Better cache locality for distance computation
  - Expected speedup: 2-3x for large-scale retrieval

**Trade-offs:**
- SoA layout requires API changes
- More complex indexing logic
- Better for batch operations, worse for single-document access

**Implementation Strategy:**
1. Benchmark AoS vs SoA for typical workloads
2. Consider hybrid approach (SoA for retrieval, AoS for single-document access)
3. Make layout configurable if both are needed

### 5. Batch Processing Optimizations (Low Priority)

**Current Implementation:**
- `batch_retrieve_bm25()` and `batch_retrieve_dense()` exist but are simple wrappers
- No special optimizations for batch processing

**Optimization:**
- Process multiple queries together for better cache locality
- Parallelize across queries (using rayon or similar)
- Expected speedup: 1.5-2x for batch sizes > 10

**Implementation Strategy:**
1. Add parallel batch processing (optional feature)
2. Optimize for cache locality in batch operations
3. Benchmark batch vs sequential processing

## Implementation Plan

### Phase 1: SIMD Dense Retrieval (Week 1)

**Goal:** Add SIMD-accelerated cosine similarity for dense retrieval

**Tasks:**
1. Create `src/simd.rs` module
2. Port dot product and cosine similarity from `rank-rerank`
3. Integrate into `DenseRetriever`
4. Add benchmarks and verify correctness
5. Measure performance improvement

**Success Criteria:**
- 8-16x speedup for dense retrieval on AVX2+ systems
- All existing tests pass
- Benchmarks show measurable improvement

### Phase 2: SIMD Sparse Retrieval (Week 2)

**Goal:** Optimize sparse vector dot product with SIMD

**Tasks:**
1. Research SIMD patterns for sparse operations
2. Implement SIMD-accelerated sparse dot product
3. Benchmark against scalar implementation
4. Integrate into `SparseRetriever`

**Success Criteria:**
- 2-4x speedup for sparse retrieval
- Correctness verified against scalar implementation
- Performance improvement validated

### Phase 3: BM25 Optimizations ✅ COMPLETED

**Goal:** Optimize BM25 scoring and retrieval

**Tasks:**
1. ✅ Precompute IDF values (stored in index, recomputed on document addition)
2. ✅ Add early termination heuristics (top-k heap with dynamic threshold)
3. ✅ Optimize candidate collection (Vec + HashSet for better cache locality)
4. ✅ Optimize scoring function (precomputed IDF parameter)
5. ✅ Benchmark improvements

**Success Criteria:**
- ✅ Measurable improvement in BM25 retrieval latency (expected 2x)
- ✅ No correctness regressions (all tests pass)
- ✅ Better performance for typical query sizes (2-10 terms)

### Phase 4: Memory Layout and Batch Processing (Week 4)

**Goal:** Improve memory efficiency and batch processing

**Tasks:**
1. Benchmark AoS vs SoA for dense vectors
2. Implement SoA layout option (if beneficial)
3. Add parallel batch processing (optional feature)
4. Document performance characteristics

**Success Criteria:**
- Improved cache efficiency for large-scale retrieval
- Optional parallel batch processing available
- Documentation updated with performance notes

## Performance Targets

### Dense Retrieval
- **Current:** ~100ms for 10K documents, 768-dim
- **Target:** <10ms for 10K documents, 768-dim (10x improvement with SIMD)
- **Target:** <100ms for 100K documents, 768-dim (10x improvement)

### BM25 Retrieval
- **Current:** ~10ms for 10K documents, 5-term query
- **Target:** <5ms for 10K documents, 5-term query (2x improvement)
- **Target:** <50ms for 100K documents, 5-term query

### Sparse Retrieval
- **Current:** ~1ms for 1K documents, 100-dim sparse vectors
- **Target:** <0.5ms for 1K documents (2x improvement with SIMD)

## Testing Strategy

### Correctness Testing
- All SIMD implementations must match scalar results (within floating-point tolerance)
- Property-based tests for edge cases
- Integration tests with real-world data

### Performance Testing
- Benchmark before/after each optimization
- Measure on multiple architectures (x86_64, aarch64)
- Test with various vector dimensions and document counts
- Compare against baseline scalar implementations

### Regression Testing
- Ensure no performance regressions in non-optimized paths
- Verify feature detection works correctly
- Test fallback paths (no SIMD support)

## Dependencies and Requirements

### New Dependencies
- None required - use `std::arch` for SIMD (stable Rust)
- Consider `rayon` for optional parallel batch processing

### Rust Version
- Stable Rust 1.74+ (current requirement)
- No nightly features required (unlike portable SIMD)

### Platform Support
- x86_64: AVX-512, AVX2, SSE (runtime detection)
- aarch64: NEON (runtime detection)
- Other platforms: Portable scalar fallback

## Documentation Updates

### Performance Documentation
- Update README with performance characteristics
- Document SIMD acceleration and expected speedups
- Add performance tuning guide (similar to rank-rerank)

### API Documentation
- Document SIMD acceleration in function docs
- Note performance characteristics in examples
- Add benchmarks to documentation

## Risk Assessment

### Low Risk
- SIMD dense retrieval (proven patterns from rank-rerank)
- Memory layout optimization (can be made optional)

### Medium Risk
- SIMD sparse retrieval (less common, needs research)
- BM25 optimizations (need profiling to identify bottlenecks)

### Mitigation
- Extensive testing against scalar implementations
- Feature flags for experimental optimizations
- Gradual rollout with performance monitoring

## Future Considerations

### Advanced Optimizations (Not in Initial Plan)
See `ADVANCED_OPTIMIZATIONS.md` for detailed analysis of:
- **Block-max WAND for BM25**: Large-scale optimization requiring index restructuring
  - 5-10x speedup for >1M documents
  - High implementation complexity
  - Better suited for Tantivy integration
- **Skip Lists**: For conjunction queries and long postings lists
  - 2-5x speedup for AND operations
  - Medium complexity
  - Can provide additional optimization for conjunction queries
- **Product quantization**: Memory efficiency for dense vectors
  - Reduces memory footprint
  - Approximate results
  - Better suited for ANN integration
- **Approximate nearest neighbor (HNSW)**: For large-scale dense retrieval
  - See `docs/COMPLETE_ANN_ROADMAP.md` for comprehensive ANN implementation
- **Persistent index support**: For production systems
  - Out of scope for in-memory design
  - Better suited for Tantivy/Elasticsearch integration

### Integration Opportunities
- Reuse SIMD code from `rank-rerank` (if possible)
- Share optimization patterns across rank-* crates
- Unified SIMD module for the ecosystem
- Document integration patterns with Tantivy, Seismic, and other specialized backends

## Success Metrics

### Quantitative
- 8-16x speedup for dense retrieval (SIMD)
- 2-4x speedup for sparse retrieval (SIMD)
- 2x speedup for BM25 retrieval (algorithmic optimizations)
- All benchmarks show measurable improvement

### Qualitative
- Code remains maintainable and readable
- No breaking API changes
- Documentation is comprehensive
- Performance characteristics are well-understood

## Next Steps

1. **Profile current implementation** to identify actual bottlenecks
2. **Start with Phase 1** (SIMD dense retrieval) - highest impact, lowest risk
3. **Benchmark continuously** to measure improvement
4. **Iterate based on results** - adjust plan as needed
