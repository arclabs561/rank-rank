//! Sparse retrieval benchmarks.
//!
//! Compares sparse vector dot product performance.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rank_retrieve::sparse::SparseRetriever;
use rank_retrieve::sparse::SparseVector;

fn generate_sparse_vectors(n_docs: usize, vocab_size: usize, sparsity: f32) -> Vec<SparseVector> {
    (0..n_docs)
        .map(|_| {
            let n_nonzero = (vocab_size as f32 * sparsity) as usize;
            let mut indices: Vec<u32> = (0..vocab_size as u32).collect();
            // Shuffle to get random indices
            for i in 0..n_nonzero {
                let j = (i * 7 + 13) % vocab_size;
                indices.swap(i, j);
            }
            let indices: Vec<u32> = indices[..n_nonzero].to_vec();
            let values: Vec<f32> = (0..n_nonzero)
                .map(|i| ((i * 11) % 100) as f32 / 100.0)
                .collect();
            SparseVector::new(indices, values).unwrap()
        })
        .collect()
}

fn bench_indexing(c: &mut Criterion) {
    let mut group = c.benchmark_group("sparse_indexing");

    for (n_docs, vocab_size, sparsity) in [
        (100, 1000, 0.1),
        (1000, 10000, 0.05),
        (10000, 100000, 0.01),
        (100000, 1000000, 0.005),
    ].iter() {
        let vectors = generate_sparse_vectors(*n_docs, *vocab_size, *sparsity);

        group.bench_with_input(
            BenchmarkId::new("add_documents", format!("{}docs_vocab{}_sparse{}", n_docs, vocab_size, sparsity)),
            &vectors,
            |b, vecs: &Vec<SparseVector>| {
                b.iter(|| {
                    let mut retriever = SparseRetriever::new();
                    for (i, vec) in vecs.iter().enumerate() {
                        retriever.add_document(i as u32, vec.clone());
                    }
                    black_box(retriever);
                })
            },
        );
    }

    group.finish();
}

fn bench_retrieval(c: &mut Criterion) {
    let mut group = c.benchmark_group("sparse_retrieval");

    for (n_docs, vocab_size, sparsity, k) in [
        (1000, 10000, 0.05, 10),
        (10000, 100000, 0.01, 20),
        (100000, 1000000, 0.005, 50),
    ].iter() {
        let vectors = generate_sparse_vectors(*n_docs, *vocab_size, *sparsity);

        // Build retriever
        let mut retriever = SparseRetriever::new();
        for (i, vec) in vectors.iter().enumerate() {
            retriever.add_document(i as u32, vec.clone());
        }

        // Generate query vector
        let query_n_nonzero = (*vocab_size as f32 * *sparsity) as usize;
        let mut query_indices: Vec<u32> = (0..*vocab_size as u32).collect();
        for i in 0..query_n_nonzero {
            let j = (i * 17 + 19) % *vocab_size;
            query_indices.swap(i, j);
        }
        let query_indices: Vec<u32> = query_indices[..query_n_nonzero].to_vec();
        let query_values: Vec<f32> = (0..query_n_nonzero)
            .map(|i| ((i * 13) % 100) as f32 / 100.0)
            .collect();
        let query = SparseVector::new(query_indices, query_values).unwrap();

        group.bench_with_input(
            BenchmarkId::new("retrieve", format!("{}docs_k{}_vocab{}", n_docs, k, vocab_size)),
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
    let mut group = c.benchmark_group("sparse_scoring");

    for (n_docs, vocab_size, sparsity) in [
        (1000, 10000, 0.05),
        (10000, 100000, 0.01),
        (100000, 1000000, 0.005),
    ].iter() {
        let vectors = generate_sparse_vectors(*n_docs, *vocab_size, *sparsity);

        // Build retriever
        let mut retriever = SparseRetriever::new();
        for (i, vec) in vectors.iter().enumerate() {
            retriever.add_document(i as u32, vec.clone());
        }

        // Generate query vector
        let query_n_nonzero = (*vocab_size as f32 * *sparsity) as usize;
        let mut query_indices: Vec<u32> = (0..*vocab_size as u32).collect();
        for i in 0..query_n_nonzero {
            let j = (i * 17 + 19) % *vocab_size;
            query_indices.swap(i, j);
        }
        let query_indices: Vec<u32> = query_indices[..query_n_nonzero].to_vec();
        let query_values: Vec<f32> = (0..query_n_nonzero)
            .map(|i| ((i * 13) % 100) as f32 / 100.0)
            .collect();
        let query = SparseVector::new(query_indices, query_values).unwrap();

        group.bench_with_input(
            BenchmarkId::new("score_document", format!("{}docs_vocab{}", n_docs, vocab_size)),
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

criterion_group!(
    benches,
    bench_indexing,
    bench_retrieval,
    bench_scoring
);
criterion_main!(benches);

