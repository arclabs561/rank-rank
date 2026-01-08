//! BM25 comparison benchmarks.
//!
//! Compares rank-retrieve BM25 against similar Rust implementations.
//!
//! Note: This requires external dependencies to be added to Cargo.toml
//! when comparison libraries are available.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};

fn generate_documents(n_docs: usize, terms_per_doc: usize, vocab_size: usize) -> Vec<Vec<String>> {
    (0..n_docs)
        .map(|_| {
            (0..terms_per_doc)
                .map(|i| format!("term{}", (i * 7) % vocab_size))
                .collect()
        })
        .collect()
}

fn bench_rank_retrieve_bm25(c: &mut Criterion) {
    let mut group = c.benchmark_group("bm25_comparison");

    for (n_docs, terms_per_doc) in [
        (1000, 100),
        (10000, 200),
        (100000, 300),
    ].iter() {
        let vocab_size = 1000;
        let documents = generate_documents(*n_docs, *terms_per_doc, vocab_size);

        // Build index
        let mut index = InvertedIndex::new();
        for (i, doc) in documents.iter().enumerate() {
            index.add_document(i as u32, doc);
        }

        // Generate query
        let query: Vec<String> = (0..10)
            .map(|i| format!("term{}", (i * 11) % vocab_size))
            .collect();

        let params = Bm25Params::default();

        group.bench_with_input(
            BenchmarkId::new("rank-retrieve", format!("{}docs", n_docs)),
            &query,
            |b, q| {
                b.iter(|| {
                    let _ = black_box(index.retrieve(q, 20, params));
                })
            },
        );
    }

    group.finish();
}

// TODO: Add benchmarks for other BM25 implementations when available:
// - bm25 crate
// - tantivy
// - etc.

criterion_group!(
    benches,
    bench_rank_retrieve_bm25
);
criterion_main!(benches);

