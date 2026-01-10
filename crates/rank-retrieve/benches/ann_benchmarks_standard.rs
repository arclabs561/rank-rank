//! Standard ANN benchmarks following ann-benchmarks methodology.
//!
//! This benchmark suite follows the structure and metrics from:
//! - ann-benchmarks (erikbern/ann-benchmarks, 5.5k+ stars)
//! - Standard metrics: Recall@K, Query Time, Build Time, Memory Usage
//! - Standard datasets: SIFT, GloVe, Random (synthetic equivalents)

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use rank_retrieve::RetrieveError;
use std::time::{Duration, Instant};

/// Standard benchmark metrics following ann-benchmarks.
#[derive(Debug, Clone)]
pub struct BenchmarkMetrics {
    /// Recall@K values
    pub recall_at_k: Vec<f32>,  // One per query
    
    /// Query times (milliseconds)
    pub query_times: Vec<f32>,  // One per query
    
    /// Build time (seconds)
    pub build_time: f32,
    
    /// Memory usage (bytes)
    pub memory_usage: usize,
    
    /// Throughput (queries per second)
    pub throughput: f32,
}

impl BenchmarkMetrics {
    /// Compute statistics from raw metrics.
    pub fn statistics(&self) -> MetricStatistics {
        MetricStatistics {
            recall_mean: mean(&self.recall_at_k),
            recall_std: std_dev(&self.recall_at_k),
            recall_p50: percentile(&self.recall_at_k, 0.50),
            recall_p95: percentile(&self.recall_at_k, 0.95),
            recall_p99: percentile(&self.recall_at_k, 0.99),
            
            query_time_mean: mean(&self.query_times),
            query_time_p50: percentile(&self.query_times, 0.50),
            query_time_p95: percentile(&self.query_times, 0.95),
            query_time_p99: percentile(&self.query_times, 0.99),
            
            build_time: self.build_time,
            memory_usage: self.memory_usage,
            throughput: self.throughput,
        }
    }
}

/// Statistical summary of metrics.
#[derive(Debug, Clone)]
pub struct MetricStatistics {
    pub recall_mean: f32,
    pub recall_std: f32,
    pub recall_p50: f32,
    pub recall_p95: f32,
    pub recall_p99: f32,
    
    pub query_time_mean: f32,
    pub query_time_p50: f32,
    pub query_time_p95: f32,
    pub query_time_p99: f32,
    
    pub build_time: f32,
    pub memory_usage: usize,
    pub throughput: f32,
}

/// Compute recall@K following ann-benchmarks methodology.
///
/// Recall@K = |retrieved ∩ ground_truth| / |ground_truth|
pub fn recall_at_k(ground_truth: &[u32], retrieved: &[u32], k: usize) -> f32 {
    if ground_truth.is_empty() {
        return 0.0;
    }
    
    let ground_truth_set: std::collections::HashSet<u32> = ground_truth.iter().take(k).copied().collect();
    let retrieved_set: std::collections::HashSet<u32> = retrieved.iter().take(k).copied().collect();
    
    let intersection = ground_truth_set.intersection(&retrieved_set).count();
    intersection as f32 / ground_truth.len().min(k) as f32
}

/// Generate synthetic dataset following ann-benchmarks patterns.
///
/// Creates normalized random vectors similar to SIFT/GloVe datasets.
pub fn generate_synthetic_dataset(
    num_vectors: usize,
    dimension: usize,
    seed: u64,
) -> Vec<Vec<f32>> {
    use rand::Rng;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    
    let mut rng = StdRng::seed_from_u64(seed);
    let mut vectors = Vec::new();
    
    for _ in 0..num_vectors {
        let mut vec = Vec::with_capacity(dimension);
        let mut norm = 0.0;
        
        // Generate random vector
        for _ in 0..dimension {
            let val = rng.gen::<f32>() * 2.0 - 1.0;
            norm += val * val;
            vec.push(val);
        }
        
        // Normalize (for cosine similarity)
        let norm = norm.sqrt();
        if norm > 0.0 {
            for val in &mut vec {
                *val /= norm;
            }
        }
        
        vectors.push(vec);
    }
    
    vectors
}

/// Compute ground truth using exact search (brute-force).
pub fn compute_ground_truth(
    query: &[f32],
    dataset: &[Vec<f32>],
    k: usize,
) -> Vec<u32> {
    use rank_retrieve::simd;
    
    let mut candidates: Vec<(u32, f32)> = dataset
        .iter()
        .enumerate()
        .map(|(i, vec)| {
            let dist = 1.0 - simd::dot(query, vec);
            (i as u32, dist)
        })
        .collect();
    
    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    candidates.iter().take(k).map(|(id, _)| *id).collect()
}

/// Benchmark a single algorithm following ann-benchmarks structure.
pub fn benchmark_algorithm<F>(
    c: &mut Criterion,
    name: &str,
    create_index: F,
    dataset_size: usize,
    dimension: usize,
    k: usize,
) where
    F: Fn(usize) -> Result<Box<dyn rank_retrieve::dense::ann::ANNIndex>, RetrieveError>,
{
    // Generate dataset
    let dataset = generate_synthetic_dataset(dataset_size, dimension, 42);
    let queries = generate_synthetic_dataset(100, dimension, 43);
    
    // Compute ground truth for first query
    let ground_truth = compute_ground_truth(&queries[0], &dataset, k);
    
    // Benchmark build time
    c.bench_with_input(
        BenchmarkId::new(format!("{}_build", name), dataset_size),
        &dataset_size,
        |b, _| {
            b.iter(|| {
                let mut index = create_index(dimension).unwrap();
                for (i, vec) in dataset.iter().enumerate() {
                    index.add(i as u32, vec.clone()).unwrap();
                }
                index.build().unwrap();
                black_box(index);
            });
        },
    );
    
    // Build index once for query benchmarks
    let mut index = create_index(dimension).unwrap();
    for (i, vec) in dataset.iter().enumerate() {
        index.add(i as u32, vec.clone()).unwrap();
    }
    index.build().unwrap();
    
    // Benchmark query time
    c.bench_function(&format!("{}_query_k{}", name, k), |b| {
        b.iter(|| {
            for query in &queries {
                let results = index.search(query, k).unwrap();
                black_box(results);
            }
        });
    });
    
    // Benchmark recall
    c.bench_function(&format!("{}_recall_k{}", name, k), |b| {
        b.iter(|| {
            let results = index.search(&queries[0], k).unwrap();
            let retrieved: Vec<u32> = results.iter().map(|(id, _)| *id).collect();
            let recall = recall_at_k(&ground_truth, &retrieved, k);
            black_box(recall);
        });
    });
}

/// Comprehensive benchmark following ann-benchmarks methodology.
pub fn run_comprehensive_benchmark<F>(
    create_index: F,
    dataset_size: usize,
    dimension: usize,
    k_values: &[usize],
) -> Result<BenchmarkMetrics, RetrieveError>
where
    F: Fn(usize) -> Result<Box<dyn rank_retrieve::dense::ann::ANNIndex>, RetrieveError>,
{
    // Generate dataset
    let dataset = generate_synthetic_dataset(dataset_size, dimension, 42);
    let queries = generate_synthetic_dataset(100, dimension, 43);
    
    // Build index and measure time
    let build_start = Instant::now();
    let mut index = create_index(dimension)?;
    for (i, vec) in dataset.iter().enumerate() {
        index.add(i as u32, vec.clone())?;
    }
    index.build()?;
    let build_time = build_start.elapsed().as_secs_f32();
    
    // Measure memory usage (approximate)
    let memory_usage = std::mem::size_of_val(&index) + 
                       dataset_size * dimension * std::mem::size_of::<f32>();
    
    // Evaluate queries
    let mut recall_at_k = Vec::new();
    let mut query_times = Vec::new();
    
    for query in &queries {
        // Compute ground truth
        let ground_truth = compute_ground_truth(query, &dataset, k_values[0]);
        
        // Query and measure time
        let query_start = Instant::now();
        let results = index.search(query, k_values[0])?;
        let query_time = query_start.elapsed().as_secs_f32() * 1000.0; // ms
        
        // Compute recall
        let retrieved: Vec<u32> = results.iter().map(|(id, _)| *id).collect();
        let recall = recall_at_k(&ground_truth, &retrieved, k_values[0]);
        
        recall_at_k.push(recall);
        query_times.push(query_time);
    }
    
    // Compute throughput
    let total_time: f32 = query_times.iter().sum();
    let throughput = queries.len() as f32 / (total_time / 1000.0);
    
    Ok(BenchmarkMetrics {
        recall_at_k,
        query_times,
        build_time,
        memory_usage,
        throughput,
    })
}

// Helper functions

fn mean(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f32>() / values.len() as f32
}

fn std_dev(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let mean_val = mean(values);
    let variance = values.iter().map(|&x| (x - mean_val).powi(2)).sum::<f32>() / values.len() as f32;
    variance.sqrt()
}

fn percentile(values: &[f32], p: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let index = (p * (sorted.len() - 1) as f32).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}

// Benchmark implementations

#[cfg(feature = "hnsw")]
fn bench_hnsw_standard(c: &mut Criterion) {
    use rank_retrieve::dense::hnsw::{HNSWIndex, HNSWParams};
    
    let create_index = |dim: usize| -> Result<Box<dyn rank_retrieve::dense::ann::ANNIndex>, RetrieveError> {
        let params = HNSWParams::default();
        let index = HNSWIndex::new(dim, params.m, params.m_max)?;
        Ok(Box::new(index))
    };
    
    for size in [1000, 10000, 100000] {
        benchmark_algorithm(c, "hnsw", create_index, size, 128, 10);
    }
}

#[cfg(feature = "sng")]
fn bench_sng_standard(c: &mut Criterion) {
    use rank_retrieve::dense::sng::{SNGIndex, SNGParams};
    
    let create_index = |dim: usize| -> Result<Box<dyn rank_retrieve::dense::ann::ANNIndex>, RetrieveError> {
        let params = SNGParams::default();
        let index = SNGIndex::new(dim, params)?;
        Ok(Box::new(index))
    };
    
    for size in [1000, 10000, 100000] {
        benchmark_algorithm(c, "sng", create_index, size, 128, 10);
    }
}

#[cfg(feature = "lsh")]
fn bench_lsh_standard(c: &mut Criterion) {
    use rank_retrieve::dense::classic::lsh::{LSHIndex, LSHParams};
    
    let create_index = |dim: usize| -> Result<Box<dyn rank_retrieve::dense::ann::ANNIndex>, RetrieveError> {
        let params = LSHParams::default();
        let index = LSHIndex::new(dim, params)?;
        Ok(Box::new(index))
    };
    
    for size in [1000, 10000, 100000] {
        benchmark_algorithm(c, "lsh", create_index, size, 128, 10);
    }
}

#[cfg(feature = "annoy")]
fn bench_annoy_standard(c: &mut Criterion) {
    use rank_retrieve::dense::classic::trees::annoy::{AnnoyIndex, AnnoyParams};
    
    let create_index = |dim: usize| -> Result<Box<dyn rank_retrieve::dense::ann::ANNIndex>, RetrieveError> {
        let params = AnnoyParams::default();
        let index = AnnoyIndex::new(dim, params)?;
        Ok(Box::new(index))
    };
    
    for size in [1000, 10000, 100000] {
        benchmark_algorithm(c, "annoy", create_index, size, 128, 10);
    }
}

criterion_group!(
    benches_standard,
    #[cfg(feature = "hnsw")]
    bench_hnsw_standard,
    #[cfg(feature = "sng")]
    bench_sng_standard,
    #[cfg(feature = "lsh")]
    bench_lsh_standard,
    #[cfg(feature = "annoy")]
    bench_annoy_standard,
);
criterion_main!(benches_standard);
