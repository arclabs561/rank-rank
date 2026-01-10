//! Benchmark metrics following ann-benchmarks standards.

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

/// Compute robustness-δ@K: proportion of queries achieving recall ≥ δ.
///
/// Robustness-δ@K addresses the tail performance problem where average recall
/// masks dramatically different user experiences. Two indexes with identical
/// average Recall@10 can have vastly different end-to-end application accuracy.
///
/// # Arguments
///
/// * `recalls` - Recall@K values for each query
/// * `delta` - Application-specific threshold (e.g., 0.8 for recommendation, 0.99 for medical)
///
/// # Returns
///
/// Proportion of queries achieving recall ≥ delta.
///
/// # References
///
/// - Research on robustness metrics (2024-2025)
/// - Tail performance analysis in vector search
pub fn robustness_delta_at_k(recalls: &[f32], delta: f32) -> f32 {
    if recalls.is_empty() {
        return 0.0;
    }
    
    let above_threshold = recalls.iter().filter(|&&r| r >= delta).count();
    above_threshold as f32 / recalls.len() as f32
}

/// Compute robustness metrics for multiple delta thresholds.
///
/// Returns robustness for common thresholds: 0.5, 0.7, 0.8, 0.9, 0.95, 0.99
pub fn robustness_metrics(recalls: &[f32]) -> RobustnessMetrics {
    RobustnessMetrics {
        robustness_50: robustness_delta_at_k(recalls, 0.5),
        robustness_70: robustness_delta_at_k(recalls, 0.7),
        robustness_80: robustness_delta_at_k(recalls, 0.8),
        robustness_90: robustness_delta_at_k(recalls, 0.9),
        robustness_95: robustness_delta_at_k(recalls, 0.95),
        robustness_99: robustness_delta_at_k(recalls, 0.99),
    }
}

/// Benchmark metrics following ann-benchmarks structure.
#[derive(Debug, Clone)]
pub struct BenchmarkMetrics {
    /// Recall@K values (one per query)
    pub recall_at_k: Vec<f32>,
    
    /// Query times in milliseconds (one per query)
    pub query_times: Vec<f32>,
    
    /// Build time in seconds
    pub build_time: f32,
    
    /// Memory usage in bytes
    pub memory_usage: usize,
    
    /// Throughput (queries per second)
    pub throughput: f32,
}

impl BenchmarkMetrics {
    /// Compute statistical summary.
    pub fn statistics(&self) -> MetricStatistics {
        MetricStatistics {
            recall_mean: mean(&self.recall_at_k),
            recall_std: std_dev(&self.recall_at_k),
            recall_p50: percentile(&self.recall_at_k, 0.50),
            recall_p95: percentile(&self.recall_at_k, 0.95),
            recall_p99: percentile(&self.recall_at_k, 0.99),
            
            robustness: robustness_metrics(&self.recall_at_k),
            
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MetricStatistics {
    pub recall_mean: f32,
    pub recall_std: f32,
    pub recall_p50: f32,
    pub recall_p95: f32,
    pub recall_p99: f32,
    
    /// Robustness metrics (proportion of queries achieving recall ≥ threshold)
    pub robustness: RobustnessMetrics,
    
    pub query_time_mean: f32,
    pub query_time_p50: f32,
    pub query_time_p95: f32,
    pub query_time_p99: f32,
    
    pub build_time: f32,
    pub memory_usage: usize,
    pub throughput: f32,
}

/// Robustness metrics for different recall thresholds.
///
/// Measures the proportion of queries achieving recall above application-specific
/// thresholds. Addresses the tail performance problem where average recall masks
/// poor performance on difficult queries.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RobustnessMetrics {
    /// Proportion of queries with recall ≥ 0.5
    pub robustness_50: f32,
    /// Proportion of queries with recall ≥ 0.7
    pub robustness_70: f32,
    /// Proportion of queries with recall ≥ 0.8
    pub robustness_80: f32,
    /// Proportion of queries with recall ≥ 0.9
    pub robustness_90: f32,
    /// Proportion of queries with recall ≥ 0.95
    pub robustness_95: f32,
    /// Proportion of queries with recall ≥ 0.99
    pub robustness_99: f32,
}

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
