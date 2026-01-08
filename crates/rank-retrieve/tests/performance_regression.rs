//! Performance regression tests for rank-retrieve.
//!
//! These tests ensure that optimizations and refactorings don't degrade performance.
//! Run with: `cargo test --release --test performance_regression`

use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
use rank_retrieve::generative::{GenerativeRetriever, MockAutoregressiveModel};
use std::time::Instant;

/// Test BM25 retrieval performance for typical workload (1K documents, top-100).
///
/// Target: <10ms for this workload
#[test]
fn test_bm25_performance_regression() {
    let mut index = InvertedIndex::new();

    // Add 1000 documents
    for i in 0..1000 {
        let terms: Vec<String> = (0..20)
            .map(|j| format!("term{}", (i * 7 + j * 11) % 100))
            .collect();
        index.add_document(i, &terms);
    }

    let query = vec![
        "term10".to_string(),
        "term20".to_string(),
        "term30".to_string(),
    ];
    let params = Bm25Params::default();

    let start = Instant::now();
    let _results = index.retrieve(&query, 100, params).unwrap();
    let elapsed = start.elapsed();

    // Regression threshold: should complete in <10ms
    assert!(
        elapsed.as_millis() < 10,
        "Performance regression: BM25(1K docs, top-100) took {}ms (threshold: 10ms)",
        elapsed.as_millis()
    );
}

/// Test generative retrieval performance for typical workload (100 documents, beam=15).
///
/// Target: <50ms for this workload
#[test]
fn test_generative_retrieval_performance_regression() {
    let model = MockAutoregressiveModel::new();
    let mut retriever = GenerativeRetriever::new(model).with_beam_size(15);

    // Add 100 documents
    for i in 0..100 {
        retriever.add_document(
            i,
            &format!("Document {} about various topics and subjects", i),
        );
    }

    let start = Instant::now();
    let _results = retriever.retrieve("What is the topic?", 10).unwrap();
    let elapsed = start.elapsed();

    // Regression threshold: should complete in <50ms
    // Generative retrieval is slower due to identifier generation
    assert!(
        elapsed.as_millis() < 50,
        "Performance regression: generative retrieval(100 docs, beam=15) took {}ms (threshold: 50ms)",
        elapsed.as_millis()
    );
}

/// Test BM25 scaling with document count.
///
/// Ensures that performance scales reasonably (sub-linear or linear).
#[test]
fn test_bm25_scaling() {
    let small_size = 100;
    let large_size = 1000; // 10x increase

    // Small index
    let mut small_index = InvertedIndex::new();
    for i in 0..small_size {
        let terms: Vec<String> = (0..20)
            .map(|j| format!("term{}", (i * 7 + j) % 50))
            .collect();
        small_index.add_document(i, &terms);
    }

    // Large index
    let mut large_index = InvertedIndex::new();
    for i in 0..large_size {
        let terms: Vec<String> = (0..20)
            .map(|j| format!("term{}", (i * 7 + j) % 50))
            .collect();
        large_index.add_document(i, &terms);
    }

    let query = vec!["term10".to_string(), "term20".to_string()];
    let params = Bm25Params::default();

    let small_start = Instant::now();
    let _small_results = small_index.retrieve(&query, 100, params).unwrap();
    let small_elapsed = small_start.elapsed();

    let large_start = Instant::now();
    let _large_results = large_index.retrieve(&query, 100, params).unwrap();
    let large_elapsed = large_start.elapsed();

    let time_ratio = large_elapsed.as_nanos() as f64 / small_elapsed.as_nanos() as f64;
    let size_ratio = (large_size as f64) / (small_size as f64);

    // Expect sub-linear or linear scaling: time_ratio should be less than size_ratio^2
    // With 10x size increase, we allow up to 20x time increase (very lenient for initial baseline)
    assert!(
        time_ratio < size_ratio * 2.0,
        "Performance scaling too poor: {}x size = {}x time (expected < {}x). This threshold will be tightened after baseline establishment.",
        size_ratio,
        time_ratio,
        size_ratio * 2.0
    );
}

/// Test heuristic scorer batch performance.
///
/// Target: <5ms for 100 passages with 20 identifiers
#[test]
fn test_heuristic_scorer_batch_performance() {
    use rank_retrieve::generative::HeuristicScorer;

    let scorer = HeuristicScorer::new();
    let passage_strings: Vec<String> = (0..100)
        .map(|i| format!("Document {} about various topics", i))
        .collect();
    let passages: Vec<(u32, &str)> = passage_strings
        .iter()
        .enumerate()
        .map(|(i, s)| (i as u32, s.as_str()))
        .collect();

    let identifiers: Vec<(String, f32)> = (0..20)
        .map(|i| (format!("term{}", i), (i + 1) as f32))
        .collect();

    let start = Instant::now();
    let _results = scorer.score_batch(&passages, &identifiers);
    let elapsed = start.elapsed();

    // Regression threshold: should complete in <5ms
    assert!(
        elapsed.as_millis() < 5,
        "Performance regression: heuristic scorer batch(100 passages, 20 identifiers) took {}ms (threshold: 5ms)",
        elapsed.as_millis()
    );
}
