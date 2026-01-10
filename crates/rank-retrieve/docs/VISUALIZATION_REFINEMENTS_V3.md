# Visualization Refinements V3 - Expanded Plot Types

## Overview

Further refinements to match ann-benchmarks.com more closely by adding additional plot types and improving the overall visualization layout.

## New Additions

### 1. Expanded Layout (4×3 Grid = 12 Plots)

**Before:** 3×3 grid (9 plots)
**After:** 4×3 grid (12 plots)
- Larger figure size: 24×16 inches (was 20×12)
- More space for additional plot types
- Better readability with larger plots

### 2. Recall vs Build Time Plot (ann-benchmarks style)

**New Plot Type:**
- X-axis: Build Time (seconds, log scale)
- Y-axis: Recall@K
- Shows trade-off between index construction time and recall
- Helps identify algorithms that build quickly while maintaining good recall
- Reference lines at 0.9, 0.95, 0.99 recall

**Use Case:** Production deployment decisions where build time matters

### 3. Recall vs Index Size Plot (ann-benchmarks style)

**New Plot Type:**
- X-axis: Index Size (MB, log scale)
- Y-axis: Recall@K
- Shows memory efficiency trade-offs
- Helps identify algorithms that use less memory while maintaining good recall
- Reference lines at 0.9, 0.95, 0.99 recall

**Use Case:** Resource-constrained environments, cost optimization

### 4. Pareto Frontier Plot

**New Plot Type:**
- X-axis: Queries per Second (QPS, log scale)
- Y-axis: Recall@K
- Highlights Pareto-optimal algorithms (not dominated by any other)
- Red dashed line shows the Pareto frontier
- Red stars mark Pareto-optimal points
- All algorithms shown as scatter points for context

**Algorithm:** 
1. Sort all points by QPS (descending)
2. For each point, check if it has higher recall than all previous points
3. Points on the frontier are those not dominated (no other point has both higher QPS and higher recall)

**Use Case:** Identifying the best algorithms for different speed/accuracy trade-offs

## Plot Organization

### Row 1: Recall vs QPS (Primary, ann-benchmarks standard)
- Plot 1: Recall@1 vs QPS
- Plot 2: Recall@10 vs QPS
- Plot 3: Recall@100 vs QPS

### Row 2: ann-benchmarks Additional Plots
- Plot 4: Recall vs Build Time
- Plot 5: Recall vs Index Size
- Plot 6: Build Time Comparison (bar chart)

### Row 3: Resource Comparisons
- Plot 7: Memory Usage Comparison (bar chart)
- Plot 8: Throughput Comparison (bar chart)
- Plot 9: Speed/Accuracy Trade-off (scatter)

### Row 4: Distribution Analysis
- Plot 10: Query Time Distribution (percentiles)
- Plot 11: Recall Distribution (percentiles)
- Plot 12: Pareto Frontier (optimal algorithms)

## Enhanced Features

### Pareto Frontier Algorithm

The Pareto frontier identifies algorithms that are optimal in the sense that no other algorithm has both:
- Higher QPS (faster)
- Higher Recall (more accurate)

**Implementation:**
```python
# Sort by QPS descending, then find points where no other point has both higher QPS and higher recall
sorted_points = sorted(all_points, key=lambda x: (-x[0], -x[1]))
pareto_points = []
max_recall_so_far = -1
for qps, recall, alg in sorted_points:
    if recall > max_recall_so_far:
        pareto_points.append((qps, recall, alg))
        max_recall_so_far = recall
```

**Visualization:**
- Red dashed line connecting Pareto points
- Red stars marking Pareto-optimal algorithms
- All algorithms shown as scatter points for context

### Improved Styling

- **Larger figure size:** 24×16 inches for better readability
- **Consistent color palette:** Algorithm-specific colors maintained across all plots
- **Reference lines:** Recall targets (0.9, 0.95, 0.99) on relevant plots
- **Log scales:** Where appropriate (QPS, build time, index size)
- **Grid lines:** Both major and minor grid lines for better readability

## Complete Plot List (12 Total)

1. **Recall@1 vs QPS** (ann-benchmarks primary)
2. **Recall@10 vs QPS** (ann-benchmarks primary)
3. **Recall@100 vs QPS** (ann-benchmarks primary)
4. **Recall vs Build Time** (ann-benchmarks style)
5. **Recall vs Index Size** (ann-benchmarks style)
6. **Build Time Comparison** (bar chart with 95% CI)
7. **Memory Usage Comparison** (bar chart with 95% CI)
8. **Throughput Comparison** (bar chart with 95% CI)
9. **Speed/Accuracy Trade-off** (scatter with error bars)
10. **Query Time Distribution** (percentiles: p50, p95, p99)
11. **Recall Distribution** (percentiles: mean, p50, p95, p99)
12. **Pareto Frontier** (optimal algorithms highlighted)

## Quality Standards Met

| Aspect | Standard | Implementation |
|--------|---------|----------------|
| ann-benchmarks plots | ✅ Recall vs Build Time, Recall vs Index Size | ✅ Implemented |
| Pareto analysis | ✅ Identify optimal algorithms | ✅ Implemented |
| Layout | ✅ Comprehensive 12-plot grid | ✅ 4×3 grid |
| Figure size | ✅ High resolution, readable | ✅ 24×16 inches, 300 DPI |
| Statistical depth | ✅ Percentiles, CI, error bars | ✅ All implemented |
| Reference lines | ✅ Quality targets | ✅ Recall targets on relevant plots |

## Files Modified

- `src/benchmark/visualization.rs`: 
  - Expanded to 4×3 grid (12 plots)
  - Added Recall vs Build Time plot
  - Added Recall vs Index Size plot
  - Added Pareto Frontier plot
  - Updated all subplot indices
  - Increased figure size to 24×16 inches

## Usage

No changes to usage - all refinements are automatic:

```bash
cargo run --example benchmark_all_algorithms --features benchmark,...
```

The enhanced visualizations will automatically include:
- 12 comprehensive plots (up from 9)
- Recall vs Build Time (ann-benchmarks style)
- Recall vs Index Size (ann-benchmarks style)
- Pareto Frontier highlighting optimal algorithms
- All previous enhancements (CI, error bars, annotations, etc.)

## Example Output

The visualization now shows:
1. **Primary plots:** Recall@K vs QPS for K=1, 10, 100 (ann-benchmarks standard)
2. **ann-benchmarks additional plots:** Recall vs Build Time, Recall vs Index Size
3. **Resource comparisons:** Build time, memory, throughput bar charts
4. **Trade-off analysis:** Speed/accuracy scatter, Pareto frontier
5. **Distribution analysis:** Query time and recall percentiles

## Future Enhancements

Potential next steps:
1. **Recall vs Distance Computations:** Show computational cost
2. **Relative Error vs QPS:** ann-benchmarks style error analysis
3. **Recall vs Candidates Generated:** Search efficiency analysis
4. **Epsilon-based plots:** Recall at different epsilon thresholds
5. **Interactive HTML output:** Clickable plots with hover tooltips
6. **Algorithm overview pages:** Performance across all datasets
7. **Dataset-specific pages:** Multiple plot types per dataset
