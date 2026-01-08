//! Dense retrieval comparison benchmarks.
//!
//! Compares rank-retrieve dense retrieval against similar implementations.
//!
//! Note: This requires external dependencies to be added to Cargo.toml
//! when comparison libraries are available.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rank_retrieve::dense::DenseRetriever;

fn generate_embeddings(n_docs: usize, dim: usize) -> Vec<Vec<f32>> {
    (0..n_docs)
        .map(|i| {
            (0..dim)
                .map(|j| ((i * 7 + j * 11) % 100) as f32 / 100.0 - 0.5)
                .collect()
        })
        .collect()
}

fn bench_rank_retrieve_dense(c: &mut Criterion) {
    let mut group = c.benchmark_group("dense_comparison");

    for (n_docs, dim) in [
        (1000, 128),
        (10000, 256),
        (100000, 384),
    ].iter() {
        let embeddings = generate_embeddings(*n_docs, *dim);

        // Build retriever
        let mut retriever = DenseRetriever::new();
        for (i, emb) in embeddings.iter().enumerate() {
            retriever.add_document(i as u32, emb.clone());
        }

        // Generate query
        let query: Vec<f32> = (0..*dim)
            .map(|j| ((j * 13) % 100) as f32 / 100.0 - 0.5)
            .collect();

        group.bench_with_input(
            BenchmarkId::new("rank-retrieve", format!("{}docs_dim{}", n_docs, dim)),
            &query,
            |b, q| {
                b.iter(|| {
                    let _ = black_box(retriever.retrieve(q, 20));
                })
            },
        );
    }

    group.finish();
}

// TODO: Add benchmarks for other dense retrieval implementations when available:
// - hnsw
// - usearch
// - etc.

criterion_group!(
    benches,
    bench_rank_retrieve_dense
);
criterion_main!(benches);

