//! Benchmark runner following ann-benchmarks structure.

use crate::benchmark::MetricStatistics;
use crate::benchmark::datasets::{Dataset, compute_ground_truth};
#[cfg(any(feature = "hnsw", feature = "nsw", feature = "scann", feature = "ivf_pq", feature = "diskann", feature = "sng", feature = "lsh", feature = "annoy", feature = "kdtree", feature = "balltree", feature = "rptree", feature = "kmeans_tree"))]
use crate::benchmark::BenchmarkMetrics;
#[cfg(any(feature = "hnsw", feature = "nsw", feature = "scann", feature = "ivf_pq", feature = "diskann", feature = "sng", feature = "lsh", feature = "annoy", feature = "kdtree", feature = "balltree", feature = "rptree", feature = "kmeans_tree"))]
use crate::benchmark::metrics::recall_at_k;
#[cfg(any(feature = "hnsw", feature = "nsw", feature = "scann", feature = "ivf_pq", feature = "diskann", feature = "sng", feature = "lsh", feature = "annoy", feature = "kdtree", feature = "balltree", feature = "rptree", feature = "kmeans_tree"))]
use crate::error::RetrieveError;
#[cfg(any(feature = "hnsw", feature = "nsw", feature = "scann", feature = "ivf_pq", feature = "diskann", feature = "sng", feature = "lsh", feature = "annoy", feature = "kdtree", feature = "balltree", feature = "rptree", feature = "kmeans_tree"))]
use crate::dense::ann::ANNIndex;
#[cfg(any(feature = "hnsw", feature = "nsw", feature = "scann", feature = "ivf_pq", feature = "diskann", feature = "sng", feature = "lsh", feature = "annoy", feature = "kdtree", feature = "balltree", feature = "rptree", feature = "kmeans_tree"))]
use std::time::Instant;

/// Benchmark result for a single algorithm/dataset/k combination.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BenchmarkResult {
    pub algorithm: String,
    pub dataset: String,
    pub k: usize,
    pub stats: MetricStatistics,
}

/// Benchmark runner following ann-benchmarks methodology.
pub struct BenchmarkRunner {
    pub datasets: Vec<(String, Dataset)>,
    pub k_values: Vec<usize>,
    /// Pre-computed ground truth: (dataset_name, k) -> `Vec<Vec<u32>>`
    ground_truth_cache: std::collections::HashMap<(String, usize), Vec<Vec<u32>>>,
    /// Maximum number of test queries to use (None = use all)
    pub max_test_queries: Option<usize>,
}

impl BenchmarkRunner {
    /// Create new benchmark runner.
    pub fn new() -> Self {
        Self {
            datasets: Vec::new(),
            k_values: vec![1, 10, 100],  // Standard K values from ann-benchmarks
            ground_truth_cache: std::collections::HashMap::new(),
            max_test_queries: None,  // Use all queries by default
        }
    }
    
    /// Set maximum number of test queries to use (for faster benchmarks).
    pub fn with_max_test_queries(mut self, max: usize) -> Self {
        self.max_test_queries = Some(max);
        self
    }
    
    /// Pre-compute ground truth for all datasets and K values.
    /// This significantly speeds up benchmarks by computing ground truth once.
    pub fn precompute_ground_truth(&mut self) {
        println!("Pre-computing ground truth for all datasets...");
        for (dataset_name, dataset) in &self.datasets {
            for &k in &self.k_values {
                let key = (dataset_name.clone(), k);
                if self.ground_truth_cache.contains_key(&key) {
                    continue;
                }
                
                // Limit number of queries if specified
                let num_queries = self.max_test_queries
                    .map(|max| max.min(dataset.test.len()))
                    .unwrap_or(dataset.test.len());
                
                println!("  Computing ground truth for {} (k={}, {} queries)...", 
                         dataset_name, k, num_queries);
                let mut ground_truths = Vec::new();
                for (i, query) in dataset.test.iter().take(num_queries).enumerate() {
                    if i % 100 == 0 && i > 0 {
                        print!("    Progress: {}/{}\r", i, num_queries);
                        use std::io::Write;
                        std::io::stdout().flush().ok();
                    }
                    let gt = compute_ground_truth(query, &dataset.train, k);
                    ground_truths.push(gt);
                }
                println!("    Completed: {}/{} queries", num_queries, num_queries);
                self.ground_truth_cache.insert(key, ground_truths);
            }
        }
        println!("Ground truth pre-computation complete.\n");
    }
    
    /// Add a dataset.
    pub fn add_dataset(&mut self, name: String, dataset: Dataset) {
        self.datasets.push((name, dataset));
    }
    
    /// Run benchmark for a single algorithm.
    #[cfg(any(feature = "hnsw", feature = "nsw", feature = "scann", feature = "ivf_pq", feature = "diskann", feature = "sng", feature = "lsh", feature = "annoy", feature = "kdtree", feature = "balltree", feature = "rptree", feature = "kmeans_tree"))]
    pub fn run_algorithm<I: ANNIndex>(
        &self,
        algorithm_name: &str,
        index: I,
        dataset_name: &str,
        dataset: &Dataset,
    ) -> Result<Vec<BenchmarkResult>, RetrieveError> {
        let mut results = Vec::new();
        
        // Build index and measure time
        let build_start = Instant::now();
        let mut index = index; // Make mutable for building
        for (i, vec) in dataset.train.iter().enumerate() {
            index.add(i as u32, vec.clone())?;
        }
        index.build()?;
        let build_time = build_start.elapsed().as_secs_f32();
        
        // Approximate memory usage
        let memory_usage = std::mem::size_of_val(&index) + 
                           dataset.train.len() * dataset.dimension * std::mem::size_of::<f32>();
        
        // Evaluate for each K value
        for &k in &self.k_values {
            let mut recall_values = Vec::new();
            let mut query_times = Vec::new();
            
            // Get pre-computed ground truth or compute on-the-fly
            let ground_truths = self.ground_truth_cache
                .get(&(dataset_name.to_string(), k))
                .map(|v| v.as_slice())
                .unwrap_or_else(|| {
                    // Fallback: compute on-the-fly if not cached
                    &[]
                });
            
            // Limit number of queries if specified
            let num_queries = self.max_test_queries
                .map(|max| max.min(dataset.test.len()))
                .unwrap_or(dataset.test.len());
            
            for (i, query) in dataset.test.iter().take(num_queries).enumerate() {
                // Get ground truth (cached or compute)
                let ground_truth = if let Some(gt) = ground_truths.get(i) {
                    gt.clone()
                } else {
                    // Fallback: compute on-the-fly
                    compute_ground_truth(query, &dataset.train, k)
                };
                
                // Query and measure time
                let query_start = Instant::now();
                let retrieved_results = index.search(query, k)?;
                let query_time = query_start.elapsed().as_secs_f32() * 1000.0; // ms
                
                // Compute recall
                let retrieved: Vec<u32> = retrieved_results.iter().map(|(id, _)| *id).collect();
                let recall = recall_at_k(&ground_truth, &retrieved, k);
                
                recall_values.push(recall);
                query_times.push(query_time);
            }
            
            // Compute throughput
            let total_time: f32 = query_times.iter().sum();
            let throughput = num_queries as f32 / (total_time / 1000.0);
            
            let metrics = BenchmarkMetrics {
                recall_at_k: recall_values,
                query_times,
                build_time,
                memory_usage,
                throughput,
            };
            
            results.push(BenchmarkResult {
                algorithm: algorithm_name.to_string(),
                dataset: dataset_name.to_string(), // Use actual dataset name from parameters
                k,
                stats: metrics.statistics(),
            });
        }
        
        Ok(results)
    }
}

impl Default for BenchmarkRunner {
    fn default() -> Self {
        Self::new()
    }
}
