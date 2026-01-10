//! Dense retrieval benchmarks.
//!
//! Compares cosine similarity performance with and without SIMD acceleration.

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

fn bench_indexing(c: &mut Criterion) {
    let mut group = c.benchmark_group("dense_indexing");

    for (n_docs, dim) in [(100, 128), (1000, 256), (10000, 384), (100000, 768)].iter() {
        let embeddings = generate_embeddings(*n_docs, *dim);

        group.bench_with_input(
            BenchmarkId::new("add_documents", format!("{}docs_dim{}", n_docs, dim)),
            &embeddings,
            |b, embs| {
                b.iter(|| {
                    let mut retriever = DenseRetriever::new();
                    for (i, emb) in embs.iter().enumerate() {
                        retriever.add_document(i as u32, emb.clone());
                    }
                    black_box(retriever);
                })
            },
        );
    }

    group.finish();
}

fn bench_retrieval(c: &mut Criterion) {
    let mut group = c.benchmark_group("dense_retrieval");

    for (n_docs, dim, k) in [
        (1000, 128, 10),
        (10000, 256, 20),
        (100000, 384, 50),
        (1000000, 768, 100),
    ]
    .iter()
    {
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
            BenchmarkId::new("retrieve", format!("{}docs_k{}_dim{}", n_docs, k, dim)),
            &query,
            |b, q| {
                b.iter(|| {
                    let _ = black_box(retriever.retrieve(q, *k));
                })
            },
        );
    }

    group.finish();
}

fn bench_scoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("dense_scoring");

    for (n_docs, dim) in [(1000, 128), (10000, 256), (100000, 384)].iter() {
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
            BenchmarkId::new("score_document", format!("{}docs_dim{}", n_docs, dim)),
            &query,
            |b, q| {
                b.iter(|| {
                    // Score first 100 documents
                    for doc_id in 0..100.min(*n_docs as u32) {
                        black_box(retriever.score(doc_id, q));
                    }
                })
            },
        );
    }

    group.finish();
}

fn bench_simd_vs_scalar(c: &mut Criterion) {
    let mut group = c.benchmark_group("dense_simd_comparison");

    for (dim, n_docs) in [(128, 1000), (256, 10000), (384, 100000), (768, 100000)].iter() {
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

        // Benchmark SIMD-accelerated retrieval (current implementation)
        group.bench_with_input(
            BenchmarkId::new("retrieve_simd", format!("{}docs_dim{}", n_docs, dim)),
            &query,
            |b, q| {
                b.iter(|| {
                    let _ = black_box(retriever.retrieve(q, 10));
                })
            },
        );

        // Benchmark SIMD dot product directly
        #[cfg(any(feature = "dense", feature = "sparse"))]
        {
            let doc_emb = &embeddings[0];
            group.bench_with_input(
                BenchmarkId::new("dot_simd", format!("dim{}", dim)),
                &(doc_emb, &query),
                |b, (a, b_vec)| {
                    b.iter(|| {
                        let _ = black_box(rank_retrieve::simd::dot(a, b_vec));
                    })
                },
            );

            // Benchmark portable (scalar) dot product for comparison
            group.bench_with_input(
                BenchmarkId::new("dot_portable", format!("dim{}", dim)),
                &(doc_emb, &query),
                |b, (a, b_vec)| {
                    b.iter(|| {
                        let _ = black_box(rank_retrieve::simd::dot_portable(a, b_vec));
                    })
                },
            );
        }
    }

    group.finish();
}

criterion_group!(benches, bench_indexing, bench_retrieval, bench_scoring, bench_simd_vs_scalar);
criterion_main!(benches);
