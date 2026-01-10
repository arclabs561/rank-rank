# Graph-Based Vector Search: Research Synthesis

## Overview

This document synthesizes findings from "Graph-Based Vector Search: An Experimental Evaluation of the State-of-the-Art" (Azizi et al., 2025) with `rank-retrieve`'s current implementation and design decisions.

**Paper**: Azizi, I., Echihabi, K., & Palpanas, T. (2025). Graph-Based Vector Search: An Experimental Evaluation of the State-of-the-Art. *Proc. ACM Manag. Data*, 3(1), Article 43.

**Key Contribution**: Comprehensive evaluation of 12 state-of-the-art graph-based ANN methods on datasets up to 1 billion vectors, proposing a taxonomy based on 5 design paradigms.

## Taxonomy: Five Design Paradigms

The paper classifies graph-based methods according to five key paradigms:

### 1. Seed Selection (SS)
Determines the node(s) in the graph where search initiates.

**Strategies**:
- **SN (Stacked NSW)**: Hierarchical multi-resolution graphs (HNSW)
- **KD (K-D Trees)**: Single or multiple K-D Trees on dataset sample (EFANNA, SPTAG-KDT, HCNNG)
- **KM (Balanced K-means Trees)**: BKT structures (SPTAG-BKT)
- **MD (Medoid)**: Fixed medoid node as entry point
- **KS (K-Sampled Random Seeds)**: K random nodes per query (DPG, NSG, Vamana)
- **SF (Single Fixed Random Entry Point)**: One random node fixed for all searches

**Key Finding**: SN and KS are most efficient, with SN outperforming on billion-scale datasets due to logarithmic adaptation to dataset growth.

### 2. Neighborhood Propagation (NP)
Approximates the k-NN graph by propagating neighborhood lists between connected nodes (NNDescent).

**Methods**: KGraph, IEH, EFANNA

**Key Finding**: NP-based methods perform worst overall and are least scalable. They require high memory (EFANNA needs 1.4TB for 100GB datasets) and don't scale beyond 25GB.

### 3. Incremental Insertion (II)
Builds graph by inserting one vertex at a time, connecting to nearest neighbors and some distant vertices.

**Methods**: NSW, HNSW, Vamana (with ND)

**Key Finding**: II-based approaches have **lowest indexing time** and **best scalability** across dataset sizes. ELPIS (II + DC) is 2.7× faster than HNSW and 4× faster than NSG.

### 4. Neighborhood Diversification (ND)
Sparsifies graph by pruning edges that lead to redundant directions.

**Strategies**:
- **RND (Relative Neighborhood Diversification)**: `dist(X_q, X_j) < dist(X_i, X_j)` for all neighbors X_i
- **RRND (Relaxed RND)**: `dist(X_q, X_j) < α · dist(X_i, X_j)` with α ≥ 1.5
- **MOND (Maximum-Oriented ND)**: Maximizes angles between neighbors (θ ≥ 60°)

**Key Finding**: 
- RND and MOND consistently outperform, followed by RRND
- ND is crucial for query performance, especially as dataset size increases
- RND achieves highest pruning ratios (20-25%), leading to smaller graph sizes

**Methods**: HNSW (RND), NSG (RND), Vamana (RRND + RND), DPG (MOND), SSG (MOND)

### 5. Divide-and-Conquer (DC)
Splits dataset into partitions, builds separate graphs, then searches in parallel or merges.

**Methods**: SPTAG, HCNNG, ELPIS

**Key Finding**: DC-based approaches excel on **hard datasets** (high LID, low LRC) and **hard query workloads**. ELPIS (DC + II + ND) is superior on challenging datasets like Seismic, RandPow0, RandPow50.

## Experimental Findings

### Scalability Analysis

**Indexing Performance** (Deep dataset, 1M to 1B vectors):
- **II-based methods** (HNSW, ELPIS, Vamana) scale best
- **NP-based methods** (KGraph, EFANNA, NSG, SSG, DPG) don't scale beyond 25GB-100GB
- **DC-based methods** (SPTAG, HCNNG) have high indexing time but good search performance

**Indexing Time** (Deep1B ≈ 350GB):
- ELPIS: Fastest (2× faster than HNSW, 2.7× faster than Vamana)
- HNSW: Second fastest
- Vamana: Third fastest
- Others: Cannot scale (exceed 48 hours)

**Memory Footprint** (Indexing):
- ELPIS: Lowest (40% less than HNSW, 30% less than Vamana)
- HNSW: Higher due to contiguous block allocation (optimized for direct access)
- NP-based methods: Very high (EFANNA needs 1.4TB for 100GB)

### Query Performance

**Best Performers** (by dataset size):

**1M vectors**:
- Sift1M: ELPIS, NSG/SSG
- Deep1M: NGT, SSG, NSG
- Seismic1M: HCNNG, ELPIS
- ImageNet1M: NSG/SSG, HNSW

**25GB datasets**:
- ELPIS: Best overall
- SPTAG-BKT: Competitive on SALD25GB
- NSG/SSG: Performance drops significantly

**100GB+ datasets**:
- ELPIS: Superior (up to order of magnitude faster at 0.95 recall)
- HNSW: Second best
- Vamana: Competitive

**Hard Datasets/Workloads**:
- DC-based methods (ELPIS, SPTAG-BKT) excel
- ELPIS maintains superiority across skewness levels (0 to 50)

### Seed Selection Impact

**Query Answering** (distance calculations for 0.99 recall, 1M queries):
- **SN and KS**: Most efficient overall
- **KD**: Competitive on 25GB-100GB, deteriorates on billion-scale
- **MD and SF**: Least efficient

**Indexing Performance**:
- **KS**: Lower distance calculations during construction
- **SN**: Requires 182M (Deep1M) to 22.3B (Deep25GB) more distance calculations
- **Trade-off**: KS-based graph can answer 45K-1.17M queries before SN-based graph finishes construction

**Recommendation**: Use SN for billion-scale, KS for smaller datasets.

### Neighborhood Diversification Impact

**Performance Ranking** (best to worst):
1. RND (Relative Neighborhood Diversification)
2. MOND (Maximum-Oriented ND)
3. RRND (Relaxed RND)
4. NoND (baseline)

**Pruning Ratios** (Deep25GB, Sift25GB):
- RND: 20-25% (highest)
- MOND: 2-4% (moderate)
- RRND: 0.6-0.7% (lowest)

**Key Insight**: RND leads to smaller graph sizes and reduced memory, while RRND creates larger graphs with higher memory usage.

## Comparison with rank-retrieve

### Current Implementation Status

**Implemented**:
- ✅ **HNSW**: Full implementation with RND, SN seed selection
- ✅ **NSW**: Flat variant (no hierarchy)
- ⏳ **OPT-SNG**: Planned (5.9× construction speedup)

**Not Implemented**:
- ❌ **NSG**: Navigating Spreading-out Graph (RND + NP)
- ✅ **Vamana**: Two-pass construction with RRND + RND (IMPLEMENTED)
- ❌ **ELPIS**: DC + II + ND (best overall performer)
- ❌ **SPTAG**: DC with K-D Trees or BKT
- ❌ **SSG**: MOND-based diversification
- ❌ **DPG**: MOND-based (though implementation uses RND)

### Design Decisions Alignment

**✅ Aligned with Research**:
1. **HNSW as primary graph method**: Confirmed as one of top 3 performers
2. **RND for neighborhood diversification**: Confirmed as best ND strategy
3. **SN seed selection**: Confirmed as best for large-scale
4. **II paradigm**: Confirmed as best for scalability

**⚠️ Gaps Identified**:
1. **No DC-based methods**: Missing ELPIS, SPTAG which excel on hard datasets
2. ✅ **Vamana**: Implemented with two-pass construction (RRND + RND)
3. **No NSG/SSG**: Good performance on 1M-25GB datasets
4. **Limited seed selection options**: Only SN (HNSW), could add KS, KD, KM

### Recommendations for rank-retrieve

#### High Priority

1. **Implement Vamana**:
   - Competitive with HNSW on large datasets
   - Better for SSD-based serving (5-10× more points/node)
   - Two-pass construction with RRND + RND
   - **Benefit**: Alternative to HNSW with different trade-offs

2. **Add KS seed selection option**:
   - Better for smaller datasets (1M-25GB)
   - Lower indexing overhead
   - **Benefit**: Faster indexing on medium-scale datasets

3. **Document seed selection trade-offs**:
   - SN vs KS based on dataset size
   - **Benefit**: Users can make informed choices

#### Medium Priority

4. **Consider ELPIS implementation**:
   - Best overall performer on large and hard datasets
   - DC + II + ND paradigm
   - **Challenge**: Complex implementation (Hercules tree + HNSW per partition)
   - **Benefit**: Superior performance on challenging workloads

5. **Add MOND as ND option**:
   - Second-best ND strategy (after RND)
   - Used by DPG, SSG, NSSG
   - **Benefit**: Alternative diversification strategy

#### Low Priority

6. **NSG/SSG for medium-scale**:
   - Good performance on 1M-25GB
   - **Limitation**: Doesn't scale beyond 25GB
   - **Benefit**: Better performance on medium-scale datasets

7. **KD/KM seed selection**:
   - Used by EFANNA, SPTAG
   - **Limitation**: Performance deteriorates on billion-scale
   - **Benefit**: Alternative seed selection for specific use cases

## Key Insights for Users

### When to Use Which Method

**Small to Medium Datasets (1M-25GB)**:
- **HNSW**: Excellent overall performance
- **NSG/SSG**: Competitive, especially on easier datasets
- **SPTAG-BKT**: Good on hard datasets

**Large Datasets (100GB-1B)**:
- **ELPIS**: Best overall (if available)
- **HNSW**: Second best, widely available
- **Vamana**: Competitive alternative

**Hard Datasets/Workloads** (High LID, Low LRC):
- **DC-based methods** (ELPIS, SPTAG): Superior
- **HNSW**: Good baseline

**Memory-Constrained**:
- **ELPIS**: Lowest memory footprint
- **HNSW**: Higher memory (contiguous allocation)
- **Vamana**: Moderate memory

### Seed Selection Guidelines

- **Billion-scale**: Use SN (Stacked NSW) - logarithmic adaptation
- **Medium-scale (1M-25GB)**: Use KS (K-Sampled Random) - lower indexing overhead
- **Small-scale**: Either SN or KS works well

### Neighborhood Diversification

- **Always use ND**: Significant performance improvement, especially at scale
- **RND recommended**: Best performance, highest pruning (smaller graphs)
- **MOND alternative**: Second-best, angle-based diversification
- **RRND**: Less effective, creates larger graphs

## Research Directions

The paper identifies promising research directions:

1. **Novel graph structures for NP/ND**: Improve scalability of NP and ND-based methods
2. **Data-adaptive seed selection**: Develop strategies that adapt to dataset characteristics
3. **Theoretical understanding of ND**: Deeper analysis of proximity vs sparsity trade-offs
4. **DC method improvements**: Clustering and summarization techniques tailored for DC
5. **Hybrid designs**: Combine II, ND, and DC paradigms (like ELPIS)

## References

- Azizi, I., Echihabi, K., & Palpanas, T. (2025). Graph-Based Vector Search: An Experimental Evaluation of the State-of-the-Art. *Proc. ACM Manag. Data*, 3(1), Article 43. https://doi.org/10.1145/3709693

## Related Documentation

- [ANN Algorithm Names and Relationships](ANN_ALGORITHMS_PLAN.md) - Technical naming conventions
- [ANN Implementation Complete](ANN_IMPLEMENTATION_COMPLETE.md) - Current implementation status
- [Competitive Analysis](COMPETITIVE_ANALYSIS.md) - Comparison with other implementations
- [Low-Level Insights](LOW_LEVEL_INSIGHTS.md) - Implementation details from other codebases
