//! BM25 retrieval benchmarks.
//!
//! Compares performance with and without optimizations (precomputed IDF, early termination).

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

fn bench_indexing(c: &mut Criterion) {
    let mut group = c.benchmark_group("bm25_indexing");

    for (n_docs, terms_per_doc) in [(100, 50), (1000, 100), (10000, 200), (100000, 300)].iter() {
        let vocab_size = 1000;
        let documents = generate_documents(*n_docs, *terms_per_doc, vocab_size);

        group.bench_with_input(
            BenchmarkId::new(
                "add_documents",
                format!("{}docs_{}terms", n_docs, terms_per_doc),
            ),
            &documents,
            |b, docs| {
                b.iter(|| {
                    let mut index = InvertedIndex::new();
                    for (i, doc) in docs.iter().enumerate() {
                        index.add_document(i as u32, doc);
                    }
                    black_box(index);
                })
            },
        );
    }

    group.finish();
}

fn bench_retrieval(c: &mut Criterion) {
    let mut group = c.benchmark_group("bm25_retrieval");

    for (n_docs, terms_per_doc, query_len, k) in [
        (1000, 100, 5, 10),
        (10000, 200, 10, 20),
        (100000, 300, 15, 50),
    ]
    .iter()
    {
        let vocab_size = 1000;
        let documents = generate_documents(*n_docs, *terms_per_doc, vocab_size);

        // Build index
        let mut index = InvertedIndex::new();
        for (i, doc) in documents.iter().enumerate() {
            index.add_document(i as u32, doc);
        }

        // Generate query
        let query: Vec<String> = (0..*query_len)
            .map(|i| format!("term{}", (i * 11) % vocab_size))
            .collect();

        let params = Bm25Params::default();

        group.bench_with_input(
            BenchmarkId::new("retrieve", format!("{}docs_k{}", n_docs, k)),
            &query,
            |b, q| {
                b.iter(|| {
                    let _ = black_box(index.retrieve(q, *k, params));
                })
            },
        );
    }

    group.finish();
}

fn bench_scoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("bm25_scoring");

    for (n_docs, terms_per_doc) in [(1000, 100), (10000, 200), (100000, 300)].iter() {
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
            BenchmarkId::new("score_document", format!("{}docs", n_docs)),
            &query,
            |b, q| {
                b.iter(|| {
                    // Score first 100 documents
                    for doc_id in 0..100.min(*n_docs as u32) {
                        black_box(index.score(doc_id, q, params));
                    }
                })
            },
        );
    }

    group.finish();
}

fn bench_optimization_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("bm25_optimization_impact");

    for (n_docs, terms_per_doc, query_len, k) in [
        (1000, 100, 5, 10),
        (10000, 200, 10, 20),
        (100000, 300, 15, 50),
    ]
    .iter()
    {
        let vocab_size = 1000;
        let documents = generate_documents(*n_docs, *terms_per_doc, vocab_size);

        // Build index
        let mut index = InvertedIndex::new();
        for (i, doc) in documents.iter().enumerate() {
            index.add_document(i as u32, doc);
        }

        // Generate query
        let query: Vec<String> = (0..*query_len)
            .map(|i| format!("term{}", (i * 11) % vocab_size))
            .collect();

        let params = Bm25Params::default();

        // Benchmark optimized retrieval (with precomputed IDF and early termination)
        group.bench_with_input(
            BenchmarkId::new("retrieve_optimized", format!("{}docs_k{}", n_docs, k)),
            &query,
            |b, q| {
                b.iter(|| {
                    let _ = black_box(index.retrieve(q, *k, params));
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_indexing, bench_retrieval, bench_scoring, bench_optimization_impact);
criterion_main!(benches);
