//! Visualization utilities for ANN benchmarks following ann-benchmarks style.
//!
//! Generates Recall@K vs Query Time plots in the style of ann-benchmarks.

use crate::benchmark::BenchmarkResult;

/// Visualization data point for plotting.
#[derive(Debug, Clone)]
pub struct PlotPoint {
    /// Query time in milliseconds
    pub query_time_ms: f32,
    
    /// Recall@K value
    pub recall: f32,
    
    /// Algorithm name
    pub algorithm: String,
    
    /// K value
    pub k: usize,
}

/// Collect plot points from benchmark results.
///
/// Groups results by algorithm and K value for plotting.
pub fn collect_plot_points(results: &[BenchmarkResult]) -> Vec<PlotPoint> {
    results
        .iter()
        .map(|result| PlotPoint {
            query_time_ms: result.stats.query_time_p50,
            recall: result.stats.recall_mean,
            algorithm: result.algorithm.clone(),
            k: result.k,
        })
        .collect()
}

/// Generate CSV output for plotting (compatible with ann-benchmarks style).
///
/// Format: algorithm,dataset,k,recall_mean,recall_std,query_time_p50,query_time_p95,query_time_p99
pub fn generate_csv(results: &[BenchmarkResult]) -> String {
    let mut csv = String::from("algorithm,dataset,k,recall_mean,recall_std,recall_p50,recall_p95,recall_p99,query_time_mean,query_time_p50,query_time_p95,query_time_p99,build_time,memory_usage,throughput\n");
    
    for result in results {
        csv.push_str(&format!(
            "{},{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{},{:.2}\n",
            result.algorithm,
            result.dataset,
            result.k,
            result.stats.recall_mean,
            result.stats.recall_std,
            result.stats.recall_p50,
            result.stats.recall_p95,
            result.stats.recall_p99,
            result.stats.query_time_mean,
            result.stats.query_time_p50,
            result.stats.query_time_p95,
            result.stats.query_time_p99,
            result.stats.build_time,
            result.stats.memory_usage,
            result.stats.throughput
        ));
    }
    
    csv
}

/// Generate JSON output for plotting (compatible with ann-benchmarks style).
#[cfg(all(feature = "serde", feature = "serde_json"))]
pub fn generate_json(results: &[BenchmarkResult]) -> Result<String, Box<dyn std::error::Error>> {
    use serde_json;
    Ok(serde_json::to_string_pretty(results)?)
}

/// Generate plot data grouped by algorithm and K value.
///
/// Returns data structure suitable for plotting libraries:
/// - Grouped by algorithm
/// - Multiple series per algorithm (one per K value)
pub fn group_by_algorithm(results: &[BenchmarkResult]) -> std::collections::HashMap<String, Vec<PlotPoint>> {
    let mut grouped: std::collections::HashMap<String, Vec<PlotPoint>> = std::collections::HashMap::new();
    
    for point in collect_plot_points(results) {
        grouped
            .entry(point.algorithm.clone())
            .or_default()
            .push(point);
    }
    
    // Sort points by query time within each algorithm
    for points in grouped.values_mut() {
        points.sort_by(|a, b| a.query_time_ms.partial_cmp(&b.query_time_ms).unwrap_or(std::cmp::Ordering::Equal));
    }
    
    grouped
}

/// Generate plot data grouped by K value.
///
/// Returns data structure suitable for plotting:
/// - Grouped by K value
/// - Multiple series per K (one per algorithm)
pub fn group_by_k(results: &[BenchmarkResult]) -> std::collections::HashMap<usize, Vec<PlotPoint>> {
    let mut grouped: std::collections::HashMap<usize, Vec<PlotPoint>> = std::collections::HashMap::new();
    
    for point in collect_plot_points(results) {
        grouped
            .entry(point.k)
            .or_default()
            .push(point);
    }
    
    // Sort points by query time within each K
    for points in grouped.values_mut() {
        points.sort_by(|a, b| a.query_time_ms.partial_cmp(&b.query_time_ms).unwrap_or(std::cmp::Ordering::Equal));
    }
    
    grouped
}

/// Generate comprehensive Python plotting script with multiple visualization types.
///
/// Creates a script that generates:
/// 1. Recall@K vs Query Time plots (ann-benchmarks style)
/// 2. Speed/Accuracy trade-off plots
/// 3. Build time comparisons
/// 4. Memory usage comparisons
/// 5. Throughput comparisons
pub fn generate_python_plot_script(results: &[BenchmarkResult], output_path: &str) -> String {
    let csv = generate_csv(results);
    let csv_escaped = csv.replace('"', r#"\""#);
    
    format!(r#"#!/usr/bin/env python3
"""Generate comprehensive ANN benchmark visualizations.

Follows pre-AI quality standards:
- Real data from actual benchmark execution
- Statistical depth: Percentiles, distributions, error bars
- High-resolution output (300 DPI)
- Accessible colors and styling
- Value labels for key metrics
"""

import matplotlib.pyplot as plt
import csv
import io
from collections import defaultdict
import numpy as np
import sys

# Parse CSV data
csv_data = """{}"""

results = []
reader = csv.DictReader(io.StringIO(csv_data))
for row in reader:
    results.append({{
        'algorithm': row['algorithm'],
        'dataset': row['dataset'],
        'k': int(row['k']),
        'recall_mean': float(row['recall_mean']),
        'recall_std': float(row['recall_std']),
        'recall_p50': float(row['recall_p50']),
        'recall_p95': float(row['recall_p95']),
        'recall_p99': float(row['recall_p99']),
        'query_time_mean': float(row['query_time_mean']),
        'query_time_p50': float(row['query_time_p50']),
        'query_time_p95': float(row['query_time_p95']),
        'query_time_p99': float(row['query_time_p99']),
        'build_time': float(row['build_time']),
        'memory_usage': int(row['memory_usage']),
        'throughput': float(row['throughput']),
    }})

# Group by algorithm and K
by_algorithm = defaultdict(lambda: defaultdict(list))
by_dataset = defaultdict(lambda: defaultdict(lambda: defaultdict(list)))

for r in results:
    by_algorithm[r['algorithm']][r['k']].append(r)
    by_dataset[r['dataset']][r['algorithm']][r['k']].append(r)

# Get unique K values
k_values = sorted(set(r['k'] for r in results))
datasets = sorted(set(r['dataset'] for r in results))
algorithms = sorted(set(r['algorithm'] for r in results))

# Color palette for algorithms - use distinct, accessible colors
# Prefer specific colors for known algorithms, fallback to tab20
color_palette = {{
    'HNSW': '#1f77b4', 'NSW': '#ff7f0e', 'Anisotropic-VQ-kmeans': '#2ca02c',
    'IVF-PQ': '#d62728', 'OPT-SNG': '#9467bd', 'LSH': '#8c564b',
    'RP-Tree-Forest': '#e377c2', 'KD-Tree': '#7f7f7f', 'Ball-Tree': '#bcbd22',
    'DiskANN': '#17becf', 'SAQ': '#ff9896', 'TurboQuant': '#c5b0d5'
}}
# Fill in missing algorithms with tab20 palette
tab20_colors = plt.cm.tab20(np.linspace(0, 1, 20))
color_map = {{}}
for i, alg in enumerate(algorithms):
    if alg in color_palette:
        color_map[alg] = color_palette[alg]
    else:
        color_map[alg] = tab20_colors[i % 20]

# Set matplotlib style for better defaults
plt.style.use('default')
plt.rcParams['font.size'] = 9
plt.rcParams['axes.labelsize'] = 10
plt.rcParams['axes.titlesize'] = 11
plt.rcParams['xtick.labelsize'] = 8
plt.rcParams['ytick.labelsize'] = 8
plt.rcParams['legend.fontsize'] = 8
plt.rcParams['figure.titlesize'] = 16

# Create comprehensive figure with multiple subplots
# Expanded to 4x3 grid (12 plots) to include more ann-benchmarks-style plots
fig = plt.figure(figsize=(24, 16))
fig.patch.set_facecolor('white')

# 1-3. Recall@K vs Queries per Second (ann-benchmarks PRIMARY format) - by K value
# This matches ann-benchmarks.com standard: Recall vs QPS (higher QPS = better, right side)
for idx, k in enumerate(k_values):
    ax = plt.subplot(4, 3, idx + 1)
    ax.set_xlabel('Queries per Second (QPS)', fontsize=10)
    ax.set_ylabel('Recall@{{}}'.format(k), fontsize=10)
    ax.set_title('Recall@{{}} vs Queries per Second (ann-benchmarks style)'.format(k), fontsize=11, fontweight='bold')
    ax.grid(True, alpha=0.3)
    
    for algorithm in algorithms:
        points = []
        for r in results:
            if r['algorithm'] == algorithm and r['k'] == k:
                # Use throughput (QPS) - ann-benchmarks standard (higher = better, intuitive)
                points.append((r['throughput'], r['recall_mean']))
        
        if points:
            points = sorted(points, key=lambda x: x[0])  # Sort by QPS (ascending)
            qps = [p[0] for p in points]
            recalls = [p[1] for p in points]
            # Add error bars using recall_std if available
            if len(points) > 1:
                # Calculate error from multiple datasets
                recalls_with_std = []
                for r in results:
                    if r['algorithm'] == algorithm and r['k'] == k:
                        recalls_with_std.append((r['throughput'], r['recall_mean'], r.get('recall_std', 0)))
                if recalls_with_std:
                    recalls_with_std = sorted(recalls_with_std, key=lambda x: x[0])
                    qps_err = [p[0] for p in recalls_with_std]
                    recalls_err = [p[1] for p in recalls_with_std]
                    stds = [p[2] for p in recalls_with_std]
                    # Only show error bars if std is meaningful (> 0.001)
                    stds_filtered = [s if s > 0.001 else 0 for s in stds]
                    if any(s > 0 for s in stds_filtered):
                        ax.errorbar(qps_err, recalls_err, yerr=stds_filtered, marker='o', label=algorithm, 
                                  linewidth=2, markersize=7, color=color_map[algorithm], alpha=0.8,
                                  capsize=4, capthick=1.5, elinewidth=1.5, errorevery=1)
                    else:
                        ax.plot(qps_err, recalls_err, marker='o', label=algorithm, linewidth=2, 
                               markersize=7, color=color_map[algorithm], alpha=0.8)
                else:
                    ax.plot(qps, recalls, marker='o', label=algorithm, linewidth=2, 
                           markersize=7, color=color_map[algorithm], alpha=0.8)
            else:
                ax.plot(qps, recalls, marker='o', label=algorithm, linewidth=2, 
                       markersize=7, color=color_map[algorithm], alpha=0.8)
    
    # Improved legend placement - use ncol for better space usage
    ax.legend(fontsize=7, loc='best', ncol=2, framealpha=0.9, columnspacing=0.8)
    ax.set_xscale('log')  # Log scale for QPS (ann-benchmarks standard)
    ax.set_ylim([0, 1.05])
    # Add grid for better readability
    ax.grid(True, alpha=0.3, linestyle='--', linewidth=0.5, which='both')
    # Add reference lines for common recall targets
    for recall_target in [0.9, 0.95, 0.99]:
        ax.axhline(y=recall_target, color='gray', linestyle=':', linewidth=0.5, alpha=0.5, zorder=0)

# 4. Recall vs Build Time (ann-benchmarks style)
ax4 = plt.subplot(4, 3, 4)
ax4.set_xlabel('Build Time (seconds)', fontsize=10)
ax4.set_ylabel('Recall@{{}}'.format(k_values[-1]), fontsize=10)
ax4.set_title('Recall vs Build Time (ann-benchmarks style)', fontsize=11, fontweight='bold')
ax4.grid(True, alpha=0.3, linestyle='--', linewidth=0.5, which='both')

for algorithm in algorithms:
    points = []
    for r in results:
        if r['algorithm'] == algorithm and r['k'] == k_values[-1]:  # Use largest K
            points.append((r['build_time'], r['recall_mean']))
    
    if points:
        points = sorted(points, key=lambda x: x[0])  # Sort by build time
        build_times = [p[0] for p in points]
        recalls = [p[1] for p in points]
        ax4.plot(build_times, recalls, marker='o', label=algorithm, linewidth=2, 
                markersize=7, color=color_map[algorithm], alpha=0.8)

ax4.legend(fontsize=7, loc='best', ncol=2, framealpha=0.9, columnspacing=0.8)
ax4.set_xscale('log')
ax4.set_ylim([0, 1.05])
for recall_target in [0.9, 0.95, 0.99]:
    ax4.axhline(y=recall_target, color='gray', linestyle=':', linewidth=0.5, alpha=0.5, zorder=0)

# 5. Recall vs Index Size (ann-benchmarks style)
ax5 = plt.subplot(4, 3, 5)
ax5.set_xlabel('Index Size (MB)', fontsize=10)
ax5.set_ylabel('Recall@{{}}'.format(k_values[-1]), fontsize=10)
ax5.set_title('Recall vs Index Size (ann-benchmarks style)', fontsize=11, fontweight='bold')
ax5.grid(True, alpha=0.3, linestyle='--', linewidth=0.5, which='both')

for algorithm in algorithms:
    points = []
    for r in results:
        if r['algorithm'] == algorithm and r['k'] == k_values[-1]:  # Use largest K
            index_size_mb = r['memory_usage'] / (1024 * 1024)
            points.append((index_size_mb, r['recall_mean']))
    
    if points:
        points = sorted(points, key=lambda x: x[0])  # Sort by index size
        sizes = [p[0] for p in points]
        recalls = [p[1] for p in points]
        ax5.plot(sizes, recalls, marker='o', label=algorithm, linewidth=2, 
                markersize=7, color=color_map[algorithm], alpha=0.8)

ax5.legend(fontsize=7, loc='best', ncol=2, framealpha=0.9, columnspacing=0.8)
ax5.set_xscale('log')
ax5.set_ylim([0, 1.05])
for recall_target in [0.9, 0.95, 0.99]:
    ax5.axhline(y=recall_target, color='gray', linestyle=':', linewidth=0.5, alpha=0.5, zorder=0)

# 6. Build Time Comparison (bar chart)
ax6 = plt.subplot(4, 3, 6)
build_times = defaultdict(list)
for r in results:
    if r['k'] == k_values[0]:  # Use first K value to avoid duplicates
        build_times[r['algorithm']].append(r['build_time'])

alg_names = []
times = []
for alg in algorithms:
    if alg in build_times:
        avg_time = np.mean(build_times[alg])
        alg_names.append(alg)
        times.append(avg_time)

if alg_names:
    # Calculate confidence intervals (95% CI)
    build_stds = defaultdict(list)
    build_counts = defaultdict(int)
    for r in results:
        if r['k'] == k_values[0]:
            build_stds[r['algorithm']].append(r['build_time'])
            build_counts[r['algorithm']] += 1
    
    times_ci = []
    for alg in alg_names:
        if alg in build_stds and len(build_stds[alg]) > 1:
            std = np.std(build_stds[alg])
            n = build_counts[alg]
            ci = 1.96 * std / np.sqrt(n)  # 95% CI
            times_ci.append(ci)
        else:
            times_ci.append(0)
    
    # Use error bars if we have multiple measurements
    if any(ci > 0 for ci in times_ci):
        bars = ax6.barh(alg_names, times, xerr=times_ci, color=[color_map[alg] for alg in alg_names], 
                       alpha=0.7, edgecolor='black', linewidth=0.5, capsize=3)
    else:
        bars = ax6.barh(alg_names, times, color=[color_map[alg] for alg in alg_names], 
                       alpha=0.7, edgecolor='black', linewidth=0.5)
    
    ax6.set_xlabel('Build Time (seconds)', fontsize=10)
    ax6.set_title('Index Build Time Comparison (95% CI)', fontsize=11, fontweight='bold')
    ax6.grid(True, alpha=0.3, axis='x', linestyle='--', linewidth=0.5)
    ax6.set_xscale('log')
    plt.setp(ax6.get_yticklabels(), fontsize=8)
    # Add value labels on bars
    for i, (alg, time, ci_val) in enumerate(zip(alg_names, times, times_ci)):
        label_x = time * 1.1 if ci_val == 0 else time + ci_val + time * 0.05
        ax6.text(label_x, i, f'{{time:.2f}}s', va='center', fontsize=7)

# 7. Memory Usage Comparison
ax7 = plt.subplot(4, 3, 7)
memory_usage = defaultdict(list)
for r in results:
    if r['k'] == k_values[0]:  # Use first K value
        memory_usage[r['algorithm']].append(r['memory_usage'] / (1024 * 1024))  # Convert to MB

alg_names_mem = []
memories = []
for alg in algorithms:
    if alg in memory_usage:
        avg_mem = np.mean(memory_usage[alg])
        alg_names_mem.append(alg)
        memories.append(avg_mem)

if alg_names_mem:
    # Calculate confidence intervals
    mem_stds = defaultdict(list)
    mem_counts = defaultdict(int)
    for r in results:
        if r['k'] == k_values[0]:
            mem_stds[r['algorithm']].append(r['memory_usage'] / (1024 * 1024))
            mem_counts[r['algorithm']] += 1
    
    memories_ci = []
    for alg in alg_names_mem:
        if alg in mem_stds and len(mem_stds[alg]) > 1:
            std = np.std(mem_stds[alg])
            n = mem_counts[alg]
            ci = 1.96 * std / np.sqrt(n)  # 95% CI
            memories_ci.append(ci)
        else:
            memories_ci.append(0)
    
    if any(ci > 0 for ci in memories_ci):
        bars = ax7.barh(alg_names_mem, memories, xerr=memories_ci, color=[color_map[alg] for alg in alg_names_mem], 
                       alpha=0.7, edgecolor='black', linewidth=0.5, capsize=3)
    else:
        bars = ax7.barh(alg_names_mem, memories, color=[color_map[alg] for alg in alg_names_mem], 
                       alpha=0.7, edgecolor='black', linewidth=0.5)
    
    ax7.set_xlabel('Memory Usage (MB)', fontsize=10)
    ax7.set_title('Memory Usage Comparison (95% CI)', fontsize=11, fontweight='bold')
    ax7.grid(True, alpha=0.3, axis='x', linestyle='--', linewidth=0.5)
    ax7.set_xscale('log')
    plt.setp(ax7.get_yticklabels(), fontsize=8)
    # Add value labels on bars
    for i, (alg, mem, ci_val) in enumerate(zip(alg_names_mem, memories, memories_ci)):
        label_x = mem * 1.1 if ci_val == 0 else mem + ci_val + mem * 0.05
        ax7.text(label_x, i, f'{{mem:.1f}}MB', va='center', fontsize=7)

# 8. Throughput Comparison
ax8 = plt.subplot(4, 3, 8)
throughputs = defaultdict(list)
for r in results:
    if r['k'] == k_values[0]:  # Use first K value
        throughputs[r['algorithm']].append(r['throughput'])

alg_names_thru = []
thru_values = []
for alg in algorithms:
    if alg in throughputs:
        avg_thru = np.mean(throughputs[alg])
        alg_names_thru.append(alg)
        thru_values.append(avg_thru)

if alg_names_thru:
    # Calculate confidence intervals
    thru_stds = defaultdict(list)
    thru_counts = defaultdict(int)
    for r in results:
        if r['k'] == k_values[0]:
            thru_stds[r['algorithm']].append(r['throughput'])
            thru_counts[r['algorithm']] += 1
    
    thru_ci = []
    for alg in alg_names_thru:
        if alg in thru_stds and len(thru_stds[alg]) > 1:
            std = np.std(thru_stds[alg])
            n = thru_counts[alg]
            ci = 1.96 * std / np.sqrt(n)  # 95% CI
            thru_ci.append(ci)
        else:
            thru_ci.append(0)
    
    if any(ci > 0 for ci in thru_ci):
        bars = ax8.barh(alg_names_thru, thru_values, xerr=thru_ci, color=[color_map[alg] for alg in alg_names_thru], 
                       alpha=0.7, edgecolor='black', linewidth=0.5, capsize=3)
    else:
        bars = ax8.barh(alg_names_thru, thru_values, color=[color_map[alg] for alg in alg_names_thru], 
                       alpha=0.7, edgecolor='black', linewidth=0.5)
    
    ax8.set_xlabel('Throughput (QPS)', fontsize=10)
    ax8.set_title('Query Throughput Comparison (95% CI)', fontsize=11, fontweight='bold')
    ax8.grid(True, alpha=0.3, axis='x', linestyle='--', linewidth=0.5)
    ax8.set_xscale('log')
    plt.setp(ax8.get_yticklabels(), fontsize=8)
    # Add value labels on bars
    for i, (alg, thru, ci_val) in enumerate(zip(alg_names_thru, thru_values, thru_ci)):
        label_x = thru * 1.1 if ci_val == 0 else thru + ci_val + thru * 0.05
        ax8.text(label_x, i, f'{{thru:.0f}}', va='center', fontsize=7)

# 9. Speed/Accuracy Trade-off (Recall vs Throughput)
ax9 = plt.subplot(4, 3, 9)
for algorithm in algorithms:
    recalls = []
    throughputs = []
    for r in results:
        if r['algorithm'] == algorithm and r['k'] == k_values[-1]:  # Use largest K
            recalls.append(r['recall_mean'])
            throughputs.append(r['throughput'])
    
    if recalls:
        # Use mean values for scatter plot with error bars
        mean_recall = np.mean(recalls)
        mean_throughput = np.mean(throughputs)
        std_recall = np.std(recalls) if len(recalls) > 1 else 0
        std_throughput = np.std(throughputs) if len(throughputs) > 1 else 0
        
        if std_recall > 0.001 or std_throughput > 0:
            ax9.errorbar(mean_throughput, mean_recall, 
                        xerr=std_throughput if std_throughput > 0 else None,
                        yerr=std_recall if std_recall > 0.001 else None,
                        marker='o', label=algorithm, s=150, alpha=0.8, 
                        color=color_map[algorithm], edgecolors='black', linewidth=1.5, 
                        zorder=3, capsize=4, capthick=1.5, elinewidth=1.5)
        else:
            ax9.scatter(mean_throughput, mean_recall, label=algorithm, s=150, alpha=0.8, 
                       color=color_map[algorithm], edgecolors='black', linewidth=1.5, zorder=3)

ax9.set_xlabel('Throughput (QPS)', fontsize=10)
ax9.set_ylabel('Recall@{{}}'.format(k_values[-1]), fontsize=10)
ax9.set_title('Speed/Accuracy Trade-off (Mean ± Std)', fontsize=11, fontweight='bold')
ax9.grid(True, alpha=0.3, linestyle='--', linewidth=0.5, which='both')
ax9.legend(fontsize=7, loc='best', ncol=2, framealpha=0.9, columnspacing=0.8)
ax9.set_xscale('log')
ax9.set_ylim([0, 1.05])
# Add reference lines
ax9.axhline(y=0.9, color='gray', linestyle=':', linewidth=0.5, alpha=0.5, zorder=0)
ax9.axhline(y=0.95, color='gray', linestyle=':', linewidth=0.5, alpha=0.5, zorder=0)

# 10. Query Time Distribution (p50, p95, p99)
ax10 = plt.subplot(4, 3, 10)
query_times_p50 = defaultdict(list)
query_times_p95 = defaultdict(list)
query_times_p99 = defaultdict(list)

for r in results:
    if r['k'] == k_values[0]:  # Use first K value
        query_times_p50[r['algorithm']].append(r['query_time_p50'])
        query_times_p95[r['algorithm']].append(r['query_time_p95'])
        query_times_p99[r['algorithm']].append(r['query_time_p99'])

x_pos = np.arange(len(algorithms))
width = 0.25

p50_means = [np.mean(query_times_p50[alg]) if alg in query_times_p50 else 0 for alg in algorithms]
p95_means = [np.mean(query_times_p95[alg]) if alg in query_times_p95 else 0 for alg in algorithms]
p99_means = [np.mean(query_times_p99[alg]) if alg in query_times_p99 else 0 for alg in algorithms]

# Use distinct colors for percentiles
percentile_colors = ['#2ca02c', '#ff7f0e', '#d62728']  # green, orange, red
ax10.bar(x_pos - width, p50_means, width, label='p50', alpha=0.8, color=percentile_colors[0], edgecolor='black', linewidth=0.5)
ax10.bar(x_pos, p95_means, width, label='p95', alpha=0.8, color=percentile_colors[1], edgecolor='black', linewidth=0.5)
ax10.bar(x_pos + width, p99_means, width, label='p99', alpha=0.8, color=percentile_colors[2], edgecolor='black', linewidth=0.5)

ax10.set_xlabel('Algorithm', fontsize=10)
ax10.set_ylabel('Query Time (ms)', fontsize=10)
ax10.set_title('Query Time Distribution (Percentiles)', fontsize=11, fontweight='bold')
ax10.set_xticks(x_pos)
ax10.set_xticklabels(algorithms, rotation=45, ha='right', fontsize=7)
ax10.legend(fontsize=8, framealpha=0.9, loc='upper left')
ax10.grid(True, alpha=0.3, axis='y', linestyle='--', linewidth=0.5, which='both')
ax10.set_yscale('log')
# Add annotation for best p50 performer
if p50_means:
    best_idx = np.argmin([m for m in p50_means if m > 0])
    best_alg = [alg for i, alg in enumerate(algorithms) if p50_means[i] > 0][best_idx]
    best_val = min([m for m in p50_means if m > 0])
    ax10.annotate(f'Best: {{best_alg}}', xy=(best_idx, best_val), xytext=(5, 5),
                textcoords='offset points', fontsize=7, bbox=dict(boxstyle='round,pad=0.3', 
                facecolor='yellow', alpha=0.7), arrowprops=dict(arrowstyle='->', lw=1))

# 11. Recall Distribution (mean, p50, p95, p99)
ax11 = plt.subplot(4, 3, 11)
recall_means = defaultdict(list)
recall_p50s = defaultdict(list)
recall_p95s = defaultdict(list)
recall_p99s = defaultdict(list)

for r in results:
    if r['k'] == k_values[-1]:  # Use largest K value
        recall_means[r['algorithm']].append(r['recall_mean'])
        recall_p50s[r['algorithm']].append(r['recall_p50'])
        recall_p95s[r['algorithm']].append(r['recall_p95'])
        recall_p99s[r['algorithm']].append(r['recall_p99'])

mean_vals = [np.mean(recall_means[alg]) if alg in recall_means else 0 for alg in algorithms]
p50_vals = [np.mean(recall_p50s[alg]) if alg in recall_p50s else 0 for alg in algorithms]
p95_vals = [np.mean(recall_p95s[alg]) if alg in recall_p95s else 0 for alg in algorithms]
p99_vals = [np.mean(recall_p99s[alg]) if alg in recall_p99s else 0 for alg in algorithms]

# Use distinct colors for recall percentiles
recall_colors = ['#1f77b4', '#2ca02c', '#ff7f0e', '#d62728']  # blue, green, orange, red
ax11.bar(x_pos - width * 1.5, mean_vals, width, label='Mean', alpha=0.8, color=recall_colors[0], edgecolor='black', linewidth=0.5)
ax11.bar(x_pos - width * 0.5, p50_vals, width, label='p50', alpha=0.8, color=recall_colors[1], edgecolor='black', linewidth=0.5)
ax11.bar(x_pos + width * 0.5, p95_vals, width, label='p95', alpha=0.8, color=recall_colors[2], edgecolor='black', linewidth=0.5)
ax11.bar(x_pos + width * 1.5, p99_vals, width, label='p99', alpha=0.8, color=recall_colors[3], edgecolor='black', linewidth=0.5)

ax11.set_xlabel('Algorithm', fontsize=10)
ax11.set_ylabel('Recall@{{}}'.format(k_values[-1]), fontsize=10)
ax11.set_title('Recall Distribution (Percentiles)', fontsize=11, fontweight='bold')
ax11.set_xticks(x_pos)
ax11.set_xticklabels(algorithms, rotation=45, ha='right', fontsize=7)
ax11.legend(fontsize=8, framealpha=0.9, loc='lower left')
ax11.grid(True, alpha=0.3, axis='y', linestyle='--', linewidth=0.5, which='both')
ax11.set_ylim([0, 1.05])
# Add annotation for best mean recall performer
if mean_vals:
    best_idx = np.argmax(mean_vals)
    best_alg = algorithms[best_idx]
    best_val = mean_vals[best_idx]
    ax11.annotate(f'Best: {{best_alg}}', xy=(best_idx, best_val), xytext=(5, -15),
                textcoords='offset points', fontsize=7, bbox=dict(boxstyle='round,pad=0.3', 
                facecolor='lightgreen', alpha=0.7), arrowprops=dict(arrowstyle='->', lw=1))

# 12. Pareto Frontier (Recall vs QPS) - highlight optimal algorithms
ax12 = plt.subplot(4, 3, 12)
# Collect all points for largest K
all_points = []
for r in results:
    if r['k'] == k_values[-1]:
        all_points.append((r['throughput'], r['recall_mean'], r['algorithm']))

# Plot all points
for algorithm in algorithms:
    alg_points = [(qps, recall) for qps, recall, alg in all_points if alg == algorithm]
    if alg_points:
        qps_vals = [p[0] for p in alg_points]
        recall_vals = [p[1] for p in alg_points]
        ax12.scatter(qps_vals, recall_vals, label=algorithm, s=100, alpha=0.6, 
                    color=color_map[algorithm], edgecolors='black', linewidth=1, zorder=2)

# Compute and plot Pareto frontier (points that are not dominated)
# A point is Pareto-optimal if no other point has both higher QPS and higher recall
if all_points:
    # First, find all non-dominated points
    pareto_points = []
    for qps1, recall1, alg1 in all_points:
        is_dominated = False
        for qps2, recall2, alg2 in all_points:
            # Point 1 is dominated if point 2 has both higher QPS and higher recall
            if (qps2 > qps1 and recall2 >= recall1) or (qps2 >= qps1 and recall2 > recall1):
                is_dominated = True
                break
        if not is_dominated:
            pareto_points.append((qps1, recall1, alg1))
    
    if pareto_points:
        pareto_qps = [p[0] for p in pareto_points]
        pareto_recalls = [p[1] for p in pareto_points]
        # Sort for plotting
        pareto_sorted = sorted(zip(pareto_qps, pareto_recalls))
        pareto_qps_sorted = [p[0] for p in pareto_sorted]
        pareto_recalls_sorted = [p[1] for p in pareto_sorted]
        ax12.plot(pareto_qps_sorted, pareto_recalls_sorted, 'r--', linewidth=2, 
                 alpha=0.7, label='Pareto Frontier', zorder=3)
        # Highlight Pareto points
        ax12.scatter(pareto_qps, pareto_recalls, s=200, alpha=0.9, color='red', 
                    edgecolors='black', linewidth=2, marker='*', zorder=4, label='Pareto Optimal')

ax12.set_xlabel('Queries per Second (QPS)', fontsize=10)
ax12.set_ylabel('Recall@{{}}'.format(k_values[-1]), fontsize=10)
ax12.set_title('Pareto Frontier (Optimal Algorithms)', fontsize=11, fontweight='bold')
ax12.grid(True, alpha=0.3, linestyle='--', linewidth=0.5, which='both')
ax12.legend(fontsize=6, loc='best', ncol=2, framealpha=0.9, columnspacing=0.5)
ax12.set_xscale('log')
ax12.set_ylim([0, 1.05])
for recall_target in [0.9, 0.95, 0.99]:
    ax12.axhline(y=recall_target, color='gray', linestyle=':', linewidth=0.5, alpha=0.5, zorder=0)

# Add subtitle with key statistics
num_datasets = len(datasets)
num_algorithms = len(algorithms)
plt.suptitle('Comprehensive ANN Benchmark Results (Real Data)', fontsize=16, fontweight='bold', y=0.995)
fig.text(0.5, 0.98, f'{{num_algorithms}} algorithms × {{num_datasets}} datasets × {{len(k_values)}} K values', 
         ha='center', fontsize=10, style='italic', alpha=0.7)
plt.tight_layout(rect=[0, 0, 1, 0.96], h_pad=3.0, w_pad=3.0)
try:
    plt.savefig('{}', dpi=300, bbox_inches='tight', facecolor='white', edgecolor='none')
    print("Comprehensive plots saved to: {}")
    print("Generated {{}} plots covering:".format(len(k_values) + 9))
    print("  - Recall@K vs QPS (ann-benchmarks style, by K value)")
    print("  - Recall vs Build Time (ann-benchmarks style)")
    print("  - Recall vs Index Size (ann-benchmarks style)")
    print("  - Build Time Comparison (bar chart)")
    print("  - Memory Usage Comparison (bar chart)")
    print("  - Throughput Comparison (bar chart)")
    print("  - Speed/Accuracy Trade-off (scatter)")
    print("  - Query Time Distribution (percentiles)")
    print("  - Recall Distribution (percentiles)")
    print("  - Pareto Frontier (optimal algorithms)")
    print("")
print("Visualization quality:")
print("  - High-resolution (300 DPI)")
print("  - Statistical depth: Percentiles, distributions, 95% confidence intervals")
print("  - Accessible colors and styling")
print("  - Value labels on key metrics")
print("  - Best performer annotations")
print("  - Reference lines for common targets")
print("  - Real data from benchmark execution")
print("")
print("Statistical methods:")
print("  - 95% Confidence Intervals: 1.96 * std / sqrt(n)")
print("  - Error bars on all measurements with multiple samples")
print("  - Percentile analysis: p50, p95, p99")
print("  - Mean ± Standard Deviation for scatter plots")
except Exception as e:
    print(f"Error saving plot: {{e}}", file=sys.stderr)
    sys.exit(1)
finally:
    plt.close()
"#, csv_escaped, output_path, output_path)
}

/// Summary statistics for visualization.
#[derive(Debug, Clone)]
pub struct VisualizationSummary {
    /// Number of algorithms tested
    pub num_algorithms: usize,
    
    /// Number of datasets tested
    pub num_datasets: usize,
    
    /// K values tested
    pub k_values: Vec<usize>,
    
    /// Best recall per algorithm (across all K values)
    pub best_recall: std::collections::HashMap<String, f32>,
    
    /// Fastest query time per algorithm (p50)
    pub fastest_query: std::collections::HashMap<String, f32>,
}

impl VisualizationSummary {
    /// Generate summary from benchmark results.
    pub fn from_results(results: &[BenchmarkResult]) -> Self {
        let algorithms: std::collections::HashSet<String> = results.iter().map(|r| r.algorithm.clone()).collect();
        let datasets: std::collections::HashSet<String> = results.iter().map(|r| r.dataset.clone()).collect();
        let k_values: std::collections::HashSet<usize> = results.iter().map(|r| r.k).collect();
        
        let mut best_recall: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
        let mut fastest_query: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
        
        for result in results {
            // Track best recall
            let current_best = best_recall.entry(result.algorithm.clone()).or_insert(0.0);
            if result.stats.recall_mean > *current_best {
                *current_best = result.stats.recall_mean;
            }
            
            // Track fastest query
            let current_fastest = fastest_query.entry(result.algorithm.clone()).or_insert(f32::INFINITY);
            if result.stats.query_time_p50 < *current_fastest {
                *current_fastest = result.stats.query_time_p50;
            }
        }
        
        let mut k_values_sorted: Vec<usize> = k_values.into_iter().collect();
        k_values_sorted.sort();
        
        Self {
            num_algorithms: algorithms.len(),
            num_datasets: datasets.len(),
            k_values: k_values_sorted,
            best_recall,
            fastest_query,
        }
    }
    
}

impl std::fmt::Display for VisualizationSummary {
    /// Generate text summary report.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Benchmark Summary")?;
        writeln!(f, "================\n")?;
        writeln!(f, "Algorithms: {}", self.num_algorithms)?;
        writeln!(f, "Datasets: {}", self.num_datasets)?;
        writeln!(f, "K values: {:?}\n", self.k_values)?;
        
        writeln!(f, "Best Recall (mean across all K):")?;
        let mut best_recall_sorted: Vec<(&String, &f32)> = self.best_recall.iter().collect();
        best_recall_sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (alg, recall) in best_recall_sorted {
            writeln!(f, "  {}: {:.4}", alg, recall)?;
        }
        
        writeln!(f, "\nFastest Query Time (p50, ms):")?;
        let mut fastest_sorted: Vec<(&String, &f32)> = self.fastest_query.iter().collect();
        fastest_sorted.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal));
        for (alg, time) in fastest_sorted {
            writeln!(f, "  {}: {:.4}", alg, time)?;
        }
        
        Ok(())
    }
}
