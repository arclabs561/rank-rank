//! Benchmarks for early termination optimizations.
//!
//! Compares heap-based top-k vs full sort for different k values.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rank_retrieve::sparse::{SparseRetriever, SparseVector};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

fn generate_sparse_retriever(n_docs: usize, vocab_size: usize, sparsity: f32) -> SparseRetriever {
    let mut retriever = SparseRetriever::new();
    for doc_id in 0..n_docs {
        let n_nonzero = (vocab_size as f32 * sparsity) as usize;
        let mut indices: Vec<u32> = (0..n_nonzero as u32).collect();
        let values: Vec<f32> = (0..n_nonzero)
            .map(|i| ((i * 7 + doc_id as usize * 11) % 100) as f32 / 100.0)
            .collect();
        let vector = SparseVector::new(indices, values).unwrap();
        retriever.add_document(doc_id as u32, vector);
    }
    retriever
}

fn bench_early_termination_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("early_termination_sparse");

    for (n_docs, k) in [
        (1000, 10),
        (10000, 10),
        (100000, 10),
        (1000, 100),
        (10000, 100),
        (100000, 100),
        (10000, 5000), // Large k (should use full sort)
    ]
    .iter()
    {
        let retriever = generate_sparse_retriever(*n_docs, 1000, 0.1);
        let query = SparseVector::new(
            vec![0, 10, 20, 30, 40],
            vec![1.0, 0.8, 0.6, 0.4, 0.2],
        )
        .unwrap();

        group.bench_with_input(
            BenchmarkId::new("retrieve", format!("{}docs_k{}", n_docs, k)),
            &retriever,
            |b, r| {
                b.iter(|| {
                    let _ = black_box(r.retrieve(&query, *k));
                })
            },
        );
    }

    group.finish();
}

fn bench_heap_vs_sort_threshold(c: &mut Criterion) {
    let mut group = c.benchmark_group("heap_vs_sort_threshold");

    let n_docs = 100000;
    let retriever = generate_sparse_retriever(n_docs, 1000, 0.1);
    let query = SparseVector::new(
        vec![0, 10, 20, 30, 40],
        vec![1.0, 0.8, 0.6, 0.4, 0.2],
    )
    .unwrap();

    // Test different k values around the threshold (n_docs / 2 = 50000)
    for k in [10, 100, 1000, 10000, 25000, 40000, 50000, 60000, 75000, 90000] {
        group.bench_with_input(
            BenchmarkId::new("retrieve", format!("k{}", k)),
            &retriever,
            |b, r| {
                b.iter(|| {
                    let _ = black_box(r.retrieve(&query, k));
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_early_termination_sparse, bench_heap_vs_sort_threshold);
criterion_main!(benches);
