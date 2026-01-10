#!/usr/bin/env python3
"""Test script to preview benchmark visualizations with sample data."""

import matplotlib.pyplot as plt
import numpy as np
from collections import defaultdict
import csv
import io

# Generate sample benchmark data
np.random.seed(42)

algorithms = ['HNSW', 'NSW', 'Anisotropic-VQ-kmeans', 'IVF-PQ', 'OPT-SNG', 'LSH', 'RP-Tree-Forest', 'KD-Tree', 'Ball-Tree']
datasets = ['sift-1m', 'glove-100', 'mnist', 'nytimes']
k_values = [1, 10, 100]

# Create realistic sample data
results = []
for alg in algorithms:
    for dataset in datasets:
        for k in k_values:
            # Realistic performance characteristics
            base_recall = {
                'HNSW': 0.95, 'NSW': 0.92, 'Anisotropic-VQ-kmeans': 0.88,
                'IVF-PQ': 0.85, 'OPT-SNG': 0.90, 'LSH': 0.75,
                'RP-Tree-Forest': 0.80, 'KD-Tree': 0.70, 'Ball-Tree': 0.72
            }[alg]
            
            base_time = {
                'HNSW': 0.5, 'NSW': 0.4, 'Anisotropic-VQ-kmeans': 0.3,
                'IVF-PQ': 0.25, 'OPT-SNG': 0.45, 'LSH': 0.2,
                'RP-Tree-Forest': 0.35, 'KD-Tree': 0.15, 'Ball-Tree': 0.18
            }[alg]
            
            # Add variation
            recall = base_recall + np.random.normal(0, 0.05)
            recall = max(0.5, min(1.0, recall))
            
            time = base_time * (1 + np.random.normal(0, 0.2))
            time = max(0.1, time)
            
            results.append({
                'algorithm': alg,
                'dataset': dataset,
                'k': k,
                'recall_mean': recall,
                'recall_std': 0.02,
                'recall_p50': recall - 0.01,
                'recall_p95': recall + 0.02,
                'recall_p99': recall + 0.03,
                'query_time_mean': time,
                'query_time_p50': time * 0.9,
                'query_time_p95': time * 1.5,
                'query_time_p99': time * 2.0,
                'build_time': np.random.uniform(1, 10),
                'memory_usage': np.random.uniform(100, 1000) * 1024 * 1024,
                'throughput': 1000 / time,
            })

# Generate CSV
csv_data = "algorithm,dataset,k,recall_mean,recall_std,recall_p50,recall_p95,recall_p99,query_time_mean,query_time_p50,query_time_p95,query_time_p99,build_time,memory_usage,throughput\n"
for r in results:
    csv_data += f"{r['algorithm']},{r['dataset']},{r['k']},{r['recall_mean']:.4f},{r['recall_std']:.4f},{r['recall_p50']:.4f},{r['recall_p95']:.4f},{r['recall_p99']:.4f},{r['query_time_mean']:.4f},{r['query_time_p50']:.4f},{r['query_time_p95']:.4f},{r['query_time_p99']:.4f},{r['build_time']:.4f},{int(r['memory_usage'])},{r['throughput']:.2f}\n"

# Parse and plot
reader = csv.DictReader(io.StringIO(csv_data))
results_parsed = []
for row in reader:
    results_parsed.append({
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
    })

# Group by algorithm and K
by_algorithm = defaultdict(lambda: defaultdict(list))
for r in results_parsed:
    by_algorithm[r['algorithm']][r['k']].append(r)

k_values = sorted(set(r['k'] for r in results_parsed))
algorithms = sorted(set(r['algorithm'] for r in results_parsed))

# Color palette
colors = plt.cm.tab20(np.linspace(0, 1, len(algorithms)))
color_map = {alg: colors[i] for i, alg in enumerate(algorithms)}

# Create figure
fig = plt.figure(figsize=(20, 12))

# 1-3. Recall@K vs Query Time
for idx, k in enumerate(k_values):
    ax = plt.subplot(3, 3, idx + 1)
    ax.set_xlabel('Query Time (ms, p50)', fontsize=10)
    ax.set_ylabel(f'Recall@{k}', fontsize=10)
    ax.set_title(f'Recall@{k} vs Query Time (All Datasets)', fontsize=11, fontweight='bold')
    ax.grid(True, alpha=0.3)
    
    for algorithm in algorithms:
        points = []
        for r in results_parsed:
            if r['algorithm'] == algorithm and r['k'] == k:
                points.append((r['query_time_p50'], r['recall_mean']))
        
        if points:
            points = sorted(points)
            times = [p[0] for p in points]
            recalls = [p[1] for p in points]
            ax.plot(times, recalls, marker='o', label=algorithm, linewidth=2, 
                   markersize=6, color=color_map[algorithm], alpha=0.8)
    
    ax.legend(fontsize=8, loc='best', ncol=2)
    ax.set_xscale('log')
    ax.set_ylim([0, 1.05])

# 4. Build Time
ax4 = plt.subplot(3, 3, 4)
build_times = defaultdict(list)
for r in results_parsed:
    if r['k'] == k_values[0]:
        build_times[r['algorithm']].append(r['build_time'])

alg_names = []
times = []
for alg in algorithms:
    if alg in build_times:
        avg_time = np.mean(build_times[alg])
        alg_names.append(alg)
        times.append(avg_time)

if alg_names:
    bars = ax4.barh(alg_names, times, color=[color_map[alg] for alg in alg_names], alpha=0.7)
    ax4.set_xlabel('Build Time (seconds)', fontsize=10)
    ax4.set_title('Index Build Time Comparison', fontsize=11, fontweight='bold')
    ax4.grid(True, alpha=0.3, axis='x')
    ax4.set_xscale('log')
    plt.setp(ax4.get_yticklabels(), fontsize=8)

# 5. Memory Usage
ax5 = plt.subplot(3, 3, 5)
memory_usage = defaultdict(list)
for r in results_parsed:
    if r['k'] == k_values[0]:
        memory_usage[r['algorithm']].append(r['memory_usage'] / (1024 * 1024))

alg_names_mem = []
memories = []
for alg in algorithms:
    if alg in memory_usage:
        avg_mem = np.mean(memory_usage[alg])
        alg_names_mem.append(alg)
        memories.append(avg_mem)

if alg_names_mem:
    bars = ax5.barh(alg_names_mem, memories, color=[color_map[alg] for alg in alg_names_mem], alpha=0.7)
    ax5.set_xlabel('Memory Usage (MB)', fontsize=10)
    ax5.set_title('Memory Usage Comparison', fontsize=11, fontweight='bold')
    ax5.grid(True, alpha=0.3, axis='x')
    ax5.set_xscale('log')
    plt.setp(ax5.get_yticklabels(), fontsize=8)

# 6. Throughput
ax6 = plt.subplot(3, 3, 6)
throughputs = defaultdict(list)
for r in results_parsed:
    if r['k'] == k_values[0]:
        throughputs[r['algorithm']].append(r['throughput'])

alg_names_thru = []
thru_values = []
for alg in algorithms:
    if alg in throughputs:
        avg_thru = np.mean(throughputs[alg])
        alg_names_thru.append(alg)
        thru_values.append(avg_thru)

if alg_names_thru:
    bars = ax6.barh(alg_names_thru, thru_values, color=[color_map[alg] for alg in alg_names_thru], alpha=0.7)
    ax6.set_xlabel('Throughput (QPS)', fontsize=10)
    ax6.set_title('Query Throughput Comparison', fontsize=11, fontweight='bold')
    ax6.grid(True, alpha=0.3, axis='x')
    ax6.set_xscale('log')
    plt.setp(ax6.get_yticklabels(), fontsize=8)

# 7. Speed/Accuracy Trade-off
ax7 = plt.subplot(3, 3, 7)
for algorithm in algorithms:
    recalls = []
    throughputs = []
    for r in results_parsed:
        if r['algorithm'] == algorithm and r['k'] == k_values[-1]:
            recalls.append(r['recall_mean'])
            throughputs.append(r['throughput'])
    
    if recalls:
        ax7.scatter(throughputs, recalls, label=algorithm, s=100, alpha=0.7, 
                   color=color_map[algorithm], edgecolors='black', linewidth=1)

ax7.set_xlabel('Throughput (QPS)', fontsize=10)
ax7.set_ylabel(f'Recall@{k_values[-1]}', fontsize=10)
ax7.set_title('Speed/Accuracy Trade-off', fontsize=11, fontweight='bold')
ax7.grid(True, alpha=0.3)
ax7.legend(fontsize=8, loc='best', ncol=2)
ax7.set_xscale('log')
ax7.set_ylim([0, 1.05])

# 8. Query Time Distribution
ax8 = plt.subplot(3, 3, 8)
query_times_p50 = defaultdict(list)
query_times_p95 = defaultdict(list)
query_times_p99 = defaultdict(list)

for r in results_parsed:
    if r['k'] == k_values[0]:
        query_times_p50[r['algorithm']].append(r['query_time_p50'])
        query_times_p95[r['algorithm']].append(r['query_time_p95'])
        query_times_p99[r['algorithm']].append(r['query_time_p99'])

x_pos = np.arange(len(algorithms))
width = 0.25

p50_means = [np.mean(query_times_p50[alg]) if alg in query_times_p50 else 0 for alg in algorithms]
p95_means = [np.mean(query_times_p95[alg]) if alg in query_times_p95 else 0 for alg in algorithms]
p99_means = [np.mean(query_times_p99[alg]) if alg in query_times_p99 else 0 for alg in algorithms]

ax8.bar(x_pos - width, p50_means, width, label='p50', alpha=0.7)
ax8.bar(x_pos, p95_means, width, label='p95', alpha=0.7)
ax8.bar(x_pos + width, p99_means, width, label='p99', alpha=0.7)

ax8.set_xlabel('Algorithm', fontsize=10)
ax8.set_ylabel('Query Time (ms)', fontsize=10)
ax8.set_title('Query Time Distribution (Percentiles)', fontsize=11, fontweight='bold')
ax8.set_xticks(x_pos)
ax8.set_xticklabels(algorithms, rotation=45, ha='right', fontsize=8)
ax8.legend(fontsize=8)
ax8.grid(True, alpha=0.3, axis='y')
ax8.set_yscale('log')

# 9. Recall Distribution
ax9 = plt.subplot(3, 3, 9)
recall_means = defaultdict(list)
recall_p50s = defaultdict(list)
recall_p95s = defaultdict(list)
recall_p99s = defaultdict(list)

for r in results_parsed:
    if r['k'] == k_values[-1]:
        recall_means[r['algorithm']].append(r['recall_mean'])
        recall_p50s[r['algorithm']].append(r['recall_p50'])
        recall_p95s[r['algorithm']].append(r['recall_p95'])
        recall_p99s[r['algorithm']].append(r['recall_p99'])

mean_vals = [np.mean(recall_means[alg]) if alg in recall_means else 0 for alg in algorithms]
p50_vals = [np.mean(recall_p50s[alg]) if alg in recall_p50s else 0 for alg in algorithms]
p95_vals = [np.mean(recall_p95s[alg]) if alg in recall_p95s else 0 for alg in algorithms]
p99_vals = [np.mean(recall_p99s[alg]) if alg in recall_p99s else 0 for alg in algorithms]

ax9.bar(x_pos - width * 1.5, mean_vals, width, label='Mean', alpha=0.7)
ax9.bar(x_pos - width * 0.5, p50_vals, width, label='p50', alpha=0.7)
ax9.bar(x_pos + width * 0.5, p95_vals, width, label='p95', alpha=0.7)
ax9.bar(x_pos + width * 1.5, p99_vals, width, label='p99', alpha=0.7)

ax9.set_xlabel('Algorithm', fontsize=10)
ax9.set_ylabel(f'Recall@{k_values[-1]}', fontsize=10)
ax9.set_title('Recall Distribution (Percentiles)', fontsize=11, fontweight='bold')
ax9.set_xticks(x_pos)
ax9.set_xticklabels(algorithms, rotation=45, ha='right', fontsize=8)
ax9.legend(fontsize=8)
ax9.grid(True, alpha=0.3, axis='y')
ax9.set_ylim([0, 1.05])

plt.suptitle('Comprehensive ANN Benchmark Results (Test Data)', fontsize=16, fontweight='bold', y=0.995)
plt.tight_layout(rect=[0, 0, 1, 0.99])
plt.savefig('test_benchmark_plot.png', dpi=300, bbox_inches='tight')
print("Test plot saved to: test_benchmark_plot.png")
