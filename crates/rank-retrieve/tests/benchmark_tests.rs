//! Tests for benchmark utilities.

use rank_retrieve::benchmark::{StandardDataset, generate_all_standard_datasets_small, create_benchmark_dataset, generate_csv, VisualizationSummary, BenchmarkRunner, BenchmarkResult};

#[test]
fn test_standard_dataset_configs() {
    assert_eq!(StandardDataset::SIFT1M.name(), "sift-1m");
    assert_eq!(StandardDataset::GloVe100.name(), "glove-100");
    assert_eq!(StandardDataset::MNIST.name(), "mnist");
    assert_eq!(StandardDataset::NYTimes.name(), "nytimes");
    
    let (num_vecs, dim) = StandardDataset::SIFT1M.config();
    assert_eq!(num_vecs, 1_000_000);
    assert_eq!(dim, 128);
    
    let (num_vecs, dim) = StandardDataset::GloVe100.config();
    assert_eq!(num_vecs, 1_200_000);
    assert_eq!(dim, 100);
}

#[test]
fn test_standard_dataset_generation() {
    let dataset = StandardDataset::SIFT1M.generate_small(42);
    assert!(!dataset.train.is_empty());
    assert!(!dataset.test.is_empty());
    assert_eq!(dataset.dimension, 128);
    
    // Train should be larger than test
    assert!(dataset.train.len() > dataset.test.len());
}

#[test]
fn test_generate_all_standard_datasets_small() {
    let datasets = generate_all_standard_datasets_small(42);
    assert_eq!(datasets.len(), 4);
    
    let names: Vec<&str> = datasets.iter().map(|(name, _)| name.as_str()).collect();
    assert!(names.contains(&"sift-1m"));
    assert!(names.contains(&"glove-100"));
    assert!(names.contains(&"mnist"));
    assert!(names.contains(&"nytimes"));
}

#[test]
fn test_visualization_csv() {
    let results = vec![
        BenchmarkResult {
            algorithm: "hnsw".to_string(),
            dataset: "sift-1m".to_string(),
            k: 10,
            stats: rank_retrieve::benchmark::MetricStatistics {
                recall_mean: 0.95,
                recall_std: 0.05,
                recall_p50: 0.96,
                recall_p95: 0.98,
                recall_p99: 0.99,
                robustness: rank_retrieve::benchmark::RobustnessMetrics {
                    robustness_50: 1.0,
                    robustness_70: 0.98,
                    robustness_80: 0.95,
                    robustness_90: 0.90,
                    robustness_95: 0.85,
                    robustness_99: 0.70,
                },
                query_time_mean: 2.5,
                query_time_p50: 2.0,
                query_time_p95: 5.0,
                query_time_p99: 10.0,
                build_time: 10.5,
                memory_usage: 1000000,
                throughput: 400.0,
            },
        },
    ];
    
    let csv = generate_csv(&results);
    assert!(csv.contains("algorithm,dataset,k"));
    assert!(csv.contains("hnsw"));
    assert!(csv.contains("sift-1m"));
    assert!(csv.contains("10"));
}

#[test]
fn test_visualization_summary() {
    let results = vec![
        BenchmarkResult {
            algorithm: "hnsw".to_string(),
            dataset: "sift-1m".to_string(),
            k: 10,
            stats: rank_retrieve::benchmark::MetricStatistics {
                recall_mean: 0.95,
                recall_std: 0.05,
                recall_p50: 0.96,
                recall_p95: 0.98,
                recall_p99: 0.99,
                robustness: rank_retrieve::benchmark::RobustnessMetrics {
                    robustness_50: 1.0,
                    robustness_70: 0.98,
                    robustness_80: 0.95,
                    robustness_90: 0.90,
                    robustness_95: 0.85,
                    robustness_99: 0.70,
                },
                query_time_mean: 2.5,
                query_time_p50: 2.0,
                query_time_p95: 5.0,
                query_time_p99: 10.0,
                build_time: 10.5,
                memory_usage: 1000000,
                throughput: 400.0,
            },
        },
    ];
    
    let summary = VisualizationSummary::from_results(&results);
    assert_eq!(summary.num_algorithms, 1);
    assert_eq!(summary.num_datasets, 1);
    assert!(summary.k_values.contains(&10));
    assert!(summary.best_recall.contains_key("hnsw"));
}
