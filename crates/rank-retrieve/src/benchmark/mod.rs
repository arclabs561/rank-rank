//! Standard ANN benchmarking utilities following ann-benchmarks methodology.
//!
//! This module provides utilities for benchmarking ANN algorithms following
//! the structure and metrics from ann-benchmarks (erikbern/ann-benchmarks).

mod metrics;
pub mod datasets;
mod runner;
mod visualization;

pub use metrics::{recall_at_k, robustness_delta_at_k, robustness_metrics, BenchmarkMetrics, MetricStatistics, RobustnessMetrics};
pub use datasets::{generate_synthetic_dataset, compute_ground_truth, create_benchmark_dataset, Dataset, StandardDataset, generate_all_standard_datasets, generate_all_standard_datasets_small};
pub use runner::{BenchmarkRunner, BenchmarkResult};
#[cfg(all(feature = "serde", feature = "serde_json"))]
pub use visualization::generate_json;
pub use visualization::{collect_plot_points, generate_csv, group_by_algorithm, group_by_k, generate_python_plot_script, VisualizationSummary, PlotPoint};
