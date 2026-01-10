# Benchmark Visualization Guide

## Overview

The benchmark suite automatically generates comprehensive visualizations to help understand ANN algorithm performance. All visualizations are created automatically after benchmarks complete.

## Generated Files

1. **`benchmark_results.csv`** - Raw data in CSV format (compatible with ann-benchmarks)
2. **`benchmark_results.json`** - Raw data in JSON format (if `serde` feature enabled)
3. **`benchmark_summary.txt`** - Text summary with key statistics
4. **`plot_benchmarks.py`** - Python script to regenerate plots
5. **`benchmark_plot.png`** - Comprehensive visualization (12 subplots in 4×3 grid)

## Visualization Types

The comprehensive plot (`benchmark_plot.png`) includes 12 subplots organized in a 4×3 grid:

### Row 1: Primary Plots (ann-benchmarks standard)

#### 1-3. Recall@K vs Queries per Second (by K value)
- **Purpose**: Compare algorithms across different K values (1, 10, 100)
- **X-axis**: Queries per second (QPS, log scale) - higher is better
- **Y-axis**: Recall@K (0-1)
- **Insight**: Shows speed/accuracy trade-offs for different K values
- **Best**: Top-right corner (high recall, high QPS)
- **Note**: This matches ann-benchmarks.com standard format

### Row 2: ann-benchmarks Additional Plots

#### 4. Recall vs Build Time (ann-benchmarks style)
- **Purpose**: Trade-off between index construction time and recall
- **X-axis**: Build time in seconds (log scale)
- **Y-axis**: Recall@K (for largest K)
- **Insight**: Identifies algorithms that build quickly while maintaining good recall
- **Best**: Top-left region (high recall, fast build)
- **Use Case**: Production deployment decisions where build time matters

#### 5. Recall vs Index Size (ann-benchmarks style)
- **Purpose**: Memory efficiency trade-offs
- **X-axis**: Index size in MB (log scale)
- **Y-axis**: Recall@K (for largest K)
- **Insight**: Identifies algorithms that use less memory while maintaining good recall
- **Best**: Top-left region (high recall, small index)
- **Use Case**: Resource-constrained environments, cost optimization

#### 6. Build Time Comparison (bar chart)
- **Purpose**: Compare index construction time
- **X-axis**: Build time in seconds (log scale)
- **Y-axis**: Algorithm names
- **Bars**: Mean with 95% confidence intervals
- **Insight**: Identifies algorithms with fast/slow index construction
- **Best**: Left side (faster build time)

### Row 3: Resource Comparisons

#### 7. Memory Usage Comparison (bar chart)
- **Purpose**: Compare memory footprint
- **X-axis**: Memory usage in MB (log scale)
- **Y-axis**: Algorithm names
- **Bars**: Mean with 95% confidence intervals
- **Insight**: Identifies memory-efficient algorithms
- **Best**: Left side (lower memory usage)

#### 8. Throughput Comparison (bar chart)
- **Purpose**: Compare query throughput
- **X-axis**: Queries per second (QPS, log scale)
- **Y-axis**: Algorithm names
- **Bars**: Mean with 95% confidence intervals
- **Insight**: Identifies high-throughput algorithms
- **Best**: Right side (higher QPS)

#### 9. Speed/Accuracy Trade-off (scatter)
- **Purpose**: Visualize the fundamental trade-off
- **X-axis**: Throughput (QPS, log scale)
- **Y-axis**: Recall@K (for largest K)
- **Error bars**: Mean ± standard deviation
- **Insight**: Shows which algorithms achieve best balance
- **Best**: Top-right corner (high recall, high throughput)

### Row 4: Distribution Analysis

#### 10. Query Time Distribution (percentiles)
- **Purpose**: Understand query time variability
- **X-axis**: Algorithm names
- **Y-axis**: Query time in milliseconds (log scale)
- **Bars**: p50, p95, p99 percentiles (green, orange, red)
- **Insight**: Identifies algorithms with consistent vs variable performance
- **Best**: Lower bars with small p95-p50 gap (fast and consistent)
- **Annotation**: Best p50 performer highlighted

#### 11. Recall Distribution (percentiles)
- **Purpose**: Understand recall variability
- **X-axis**: Algorithm names
- **Y-axis**: Recall@K (for largest K)
- **Bars**: Mean, p50, p95, p99 percentiles (blue, green, orange, red)
- **Insight**: Identifies algorithms with consistent vs variable recall
- **Best**: Higher bars with small p95-p50 gap (high and consistent recall)
- **Annotation**: Best mean recall performer highlighted

#### 12. Pareto Frontier (optimal algorithms)
- **Purpose**: Identify Pareto-optimal algorithms
- **X-axis**: Queries per second (QPS, log scale)
- **Y-axis**: Recall@K (for largest K)
- **Visual**: Red dashed line connecting Pareto points, red stars marking optimal algorithms
- **Insight**: Algorithms on the frontier are not dominated (no other algorithm has both higher QPS and higher recall)
- **Best**: Points on the red frontier line
- **Use Case**: Selecting the best algorithm for different speed/accuracy trade-offs

## Interpreting Results

### Algorithm Selection Guide

1. **High Recall Required**: 
   - Look at Recall@K vs QPS plots (subplots 1-3) - choose top-right region
   - Check Recall Distribution (subplot 11) for consistency
   - Review Pareto Frontier (subplot 12) for optimal algorithms

2. **Low Latency Required**: 
   - Look at Recall@K vs QPS plots (subplots 1-3) - choose right side (high QPS)
   - Check Query Time Distribution (subplot 10) for tail latency (p95, p99)
   - Consider Speed/Accuracy Trade-off (subplot 9)

3. **High Throughput Required**: 
   - Look at Throughput Comparison (subplot 8) - choose right side
   - Consider Speed/Accuracy Trade-off (subplot 9)
   - Review Pareto Frontier (subplot 12) for optimal throughput/recall balance

4. **Memory Constrained**: 
   - Look at Recall vs Index Size (subplot 5) - choose top-left region
   - Check Memory Usage Comparison (subplot 7) - choose left side
   - Consider quantization-based methods (IVF-PQ, SCANN)

5. **Fast Index Construction**: 
   - Look at Recall vs Build Time (subplot 4) - choose top-left region
   - Check Build Time Comparison (subplot 6) - choose left side
   - Important for dynamic datasets

6. **Optimal Trade-offs**: 
   - Review Pareto Frontier (subplot 12) - algorithms on the red line are optimal
   - No other algorithm dominates them (both higher QPS and higher recall)

### Common Patterns

- **HNSW/NSW**: Typically high recall, moderate query time, fast build
- **IVF-PQ/SCANN**: Lower memory, moderate recall, fast queries
- **Tree Methods (KD-Tree, Ball Tree)**: Fast for low dimensions, slower for high dimensions
- **LSH**: Fast queries, variable recall depending on parameters

## Regenerating Plots

If you need to regenerate plots:

```bash
python3 plot_benchmarks.py
```

Or modify the script to customize visualizations.

## Customization

The Python script (`plot_benchmarks.py`) can be customized to:
- Change color schemes
- Add additional plots
- Filter specific algorithms or datasets
- Adjust plot sizes and styles

## Dependencies

The visualization script requires:
- Python 3.6+
- matplotlib
- numpy

Install with:
```bash
pip install matplotlib numpy
```
