//! Comprehensive benchmark example for all ANN algorithms.
//!
//! This example demonstrates how to use the benchmark infrastructure to
//! compare all implemented ANN algorithms following ann-benchmarks methodology.
//!
//! **Metrics Collected:**
//! - Recall@K (K=1, 10, 100)
//! - Query Time (mean, p50, p95, p99)
//! - Build Time
//! - Memory Usage
//! - Throughput (QPS)
//!
//! **Output:**
//! - CSV files for analysis
//! - JSON files for programmatic access
//! - Python plotting scripts for visualization

#[cfg(feature = "benchmark")]
use rank_retrieve::benchmark::{
    BenchmarkRunner, generate_all_standard_datasets_small,
    generate_csv, generate_python_plot_script,
};
#[cfg(feature = "benchmark")]
#[cfg(feature = "serde")]
use rank_retrieve::benchmark::generate_json;

#[cfg(all(feature = "benchmark", feature = "hnsw"))]
use rank_retrieve::dense::hnsw::{HNSWIndex, HNSWParams};
#[cfg(all(feature = "benchmark", feature = "nsw"))]
use rank_retrieve::dense::nsw::{NSWIndex, NSWParams};
#[cfg(all(feature = "benchmark", feature = "scann"))]
use rank_retrieve::dense::scann::{SCANNIndex, SCANNParams};
#[cfg(all(feature = "benchmark", feature = "ivf_pq"))]
use rank_retrieve::dense::ivf_pq::{IVFPQIndex, IVFPQParams};
#[cfg(all(feature = "benchmark", feature = "sng"))]
use rank_retrieve::dense::sng::{SNGIndex, SNGParams};
#[cfg(all(feature = "benchmark", feature = "lsh"))]
use rank_retrieve::dense::classic::lsh::{LSHIndex, LSHParams};
#[cfg(all(feature = "benchmark", feature = "annoy"))]
use rank_retrieve::dense::classic::trees::annoy::{AnnoyIndex, AnnoyParams};
#[cfg(all(feature = "benchmark", feature = "kmeans_tree"))]
use rank_retrieve::dense::classic::trees::kmeans_tree::{KMeansTreeIndex, KMeansTreeParams};

#[cfg(feature = "benchmark")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Comprehensive ANN Benchmark Suite ===\n");

    // Generate standard datasets (small versions for quick testing)
    println!("Generating benchmark datasets...");
    let datasets = generate_all_standard_datasets_small(42);
    
    // Create benchmark runner with limited queries for faster execution
    // Remove `.with_max_test_queries(1000)` to use all test queries
    let mut runner = BenchmarkRunner::new()
        .with_max_test_queries(1000);  // Use 1000 queries per dataset for faster benchmarks
    
    // Add datasets
    for (name, dataset) in datasets.iter() {
        runner.add_dataset(name.clone(), dataset.clone());
        println!("  Added dataset: {}", name);
    }
    
    // Pre-compute ground truth (much faster than computing for each algorithm)
    runner.precompute_ground_truth();
    
    println!("Running benchmarks on all algorithms...\n");
    
    let mut all_results = Vec::new();
    
    // Benchmark HNSW
    #[cfg(feature = "hnsw")]
    {
        println!("Benchmarking HNSW...");
        for (name, dataset) in &runner.datasets {
            let index = HNSWIndex::with_params(dataset.dimension, HNSWParams::default())?;
            let results = runner.run_algorithm("HNSW", index, name, dataset)?;
            all_results.extend(results);
        }
    }
    
    // Benchmark NSW
    #[cfg(feature = "nsw")]
    {
        println!("Benchmarking NSW...");
        for (name, dataset) in &runner.datasets {
            let index = NSWIndex::with_params(dataset.dimension, NSWParams::default())?;
            let results = runner.run_algorithm("NSW", index, name, &dataset)?;
            all_results.extend(results);
        }
    }
    
    // Benchmark Anisotropic VQ + k-means (SCANN)
    #[cfg(feature = "scann")]
    {
        println!("Benchmarking Anisotropic VQ + k-means (SCANN)...");
        for (name, dataset) in &runner.datasets {
            let params = SCANNParams {
                num_partitions: 256,
                num_reorder: 100,
                quantization_bits: 8,
            };
            let index = SCANNIndex::new(dataset.dimension, params)?;
            let results = runner.run_algorithm("Anisotropic-VQ-kmeans", index, name, &dataset)?;
            all_results.extend(results);
        }
    }
    
    // Benchmark IVF-PQ
    #[cfg(feature = "ivf_pq")]
    {
        println!("Benchmarking IVF-PQ...");
        for (name, dataset) in &runner.datasets {
            let index = IVFPQIndex::new(dataset.dimension, IVFPQParams::default())?;
            let results = runner.run_algorithm("IVF-PQ", index, name, &dataset)?;
            all_results.extend(results);
        }
    }
    
    // Benchmark OPT-SNG
    #[cfg(feature = "sng")]
    {
        println!("Benchmarking OPT-SNG...");
        for (name, dataset) in &runner.datasets {
            let index = SNGIndex::new(dataset.dimension, SNGParams::default())?;
            let results = runner.run_algorithm("OPT-SNG", index, name, &dataset)?;
            all_results.extend(results);
        }
    }
    
    // Benchmark LSH
    #[cfg(feature = "lsh")]
    {
        println!("Benchmarking LSH...");
        for (name, dataset) in &runner.datasets {
            let index = LSHIndex::new(dataset.dimension, LSHParams::default())?;
            let results = runner.run_algorithm("LSH", index, name, &dataset)?;
            all_results.extend(results);
        }
    }
    
    // Benchmark Random Projection Tree Forest (Annoy)
    #[cfg(feature = "annoy")]
    {
        println!("Benchmarking Random Projection Tree Forest (Annoy)...");
        for (name, dataset) in &runner.datasets {
            let index = AnnoyIndex::new(dataset.dimension, AnnoyParams::default())?;
            let results = runner.run_algorithm("RP-Tree-Forest", index, name, &dataset)?;
            all_results.extend(results);
        }
    }
    
    // Benchmark K-Means Tree
    #[cfg(feature = "kmeans_tree")]
    {
        println!("Benchmarking K-Means Tree...");
        for (name, dataset) in &runner.datasets {
            let params = KMeansTreeParams::default();
            let index = KMeansTreeIndex::new(dataset.dimension, params)?;
            let results = runner.run_algorithm("K-Means-Tree", index, name, &dataset)?;
            all_results.extend(results);
        }
    }
    
    println!("\n=== Generating Reports ===\n");
    
    // Generate CSV
    let csv_output = generate_csv(&all_results);
    std::fs::write("benchmark_results.csv", csv_output)?;
    println!("  Generated: benchmark_results.csv");
    
    // Generate JSON (if serde feature enabled)
    #[cfg(all(feature = "serde", feature = "serde_json"))]
    {
        let json_output = generate_json(&all_results)?;
        std::fs::write("benchmark_results.json", json_output)?;
        println!("  Generated: benchmark_results.json");
    }
    
    // Generate Python plotting script
    let plot_script = generate_python_plot_script(&all_results, "benchmark_plot.png");
    std::fs::write("plot_benchmarks.py", plot_script)?;
    println!("  Generated: plot_benchmarks.py");
    
    // Generate summary report
    use rank_retrieve::benchmark::VisualizationSummary;
    let summary = VisualizationSummary::from_results(&all_results);
    let summary_text = summary.to_string();
    std::fs::write("benchmark_summary.txt", &summary_text)?;
    println!("  Generated: benchmark_summary.txt");
    
    println!("\n=== Benchmark Complete ===");
    println!("\nResults:");
    println!("  - CSV: benchmark_results.csv");
    #[cfg(all(feature = "serde", feature = "serde_json"))]
    println!("  - JSON: benchmark_results.json");
    println!("  - Plot script: plot_benchmarks.py");
    println!("  - Summary: benchmark_summary.txt");
    println!("\nGenerating visualizations...");
    
    // Automatically run the Python script to generate plots
    let output = std::process::Command::new("python3")
        .arg("plot_benchmarks.py")
        .current_dir(".")
        .output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                println!("  ✓ Visualizations generated successfully!");
                println!("  ✓ Plot saved to: benchmark_plot.png");
                if !output.stdout.is_empty() {
                    print!("{}", String::from_utf8_lossy(&output.stdout));
                }
            } else {
                println!("  ⚠ Python script execution had issues (plots may still be generated)");
                if !output.stderr.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                }
            }
        }
        Err(e) => {
            println!("  ⚠ Could not auto-generate plots: {}", e);
            println!("  → Run manually: python3 plot_benchmarks.py");
        }
    }
    
    println!("\nTo view results:");
    println!("  - View summary: cat benchmark_summary.txt");
    println!("  - View plots: open benchmark_plot.png");
    println!("  - Regenerate plots: python3 plot_benchmarks.py");
    
    Ok(())
}

#[cfg(not(feature = "benchmark"))]
fn main() {
    eprintln!("This example requires the 'benchmark' feature.");
    eprintln!("Run with: cargo run --example benchmark_all_algorithms --features benchmark,hnsw,nsw,scann,ivf_pq,sng,lsh,annoy,serde");
}
