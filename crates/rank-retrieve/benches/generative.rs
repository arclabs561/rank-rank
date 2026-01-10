//! Generative retrieval benchmarks.
//!
//! Benchmarks identifier generation, heuristic scoring, and full retrieval pipeline.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rank_retrieve::generative::{
    AutoregressiveModel, GenerativeRetriever, HeuristicScorer, IdentifierType,
    MockAutoregressiveModel,
};

fn generate_corpus(n_docs: usize) -> Vec<(usize, String)> {
    (0..n_docs)
        .map(|i| {
            (
                i,
                format!(
                    "Document {}: This is a sample passage about topic {} with some content.",
                    i,
                    i % 10
                ),
            )
        })
        .collect()
}

fn bench_heuristic_scoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("generative_heuristic_scoring");

    for (n_docs, n_identifiers) in [(10, 5), (100, 15), (1000, 30)].iter() {
        let corpus = generate_corpus(*n_docs);
        let identifiers: Vec<(String, f32)> = (0..*n_identifiers)
            .map(|i| (format!("Title {}", i), 10.0 - (i as f32 * 0.5)))
            .collect();
        let scorer = HeuristicScorer::default();

        group.bench_with_input(
            BenchmarkId::new(
                "score_passages",
                format!("{}docs_{}ids", n_docs, n_identifiers),
            ),
            &(corpus, identifiers),
            |b, (corpus, ids)| {
                b.iter(|| {
                    for (_, passage_text) in corpus {
                        black_box(scorer.score_passage(passage_text, ids));
                    }
                })
            },
        );
    }

    group.finish();
}

fn bench_identifier_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("generative_identifier_generation");

    for beam_size in [5, 10, 15, 30].iter() {
        let model = MockAutoregressiveModel::new();
        let query = "What is machine learning?";

        group.bench_with_input(
            BenchmarkId::new("generate_identifiers", beam_size),
            beam_size,
            |b, &beam| {
                b.iter(|| {
                    for identifier_type in [
                        IdentifierType::Title,
                        IdentifierType::Substring,
                        IdentifierType::PseudoQuery,
                    ] {
                        let prefix = identifier_type.prefix();
                        let _ = black_box(model.generate(query, prefix, beam, None));
                    }
                })
            },
        );
    }

    group.finish();
}

fn bench_full_retrieval(c: &mut Criterion) {
    let mut group = c.benchmark_group("generative_full_retrieval");

    for (n_docs, beam_size, k) in [(10, 5, 5), (100, 10, 10), (1000, 15, 20)].iter() {
        let corpus = generate_corpus(*n_docs);
        let model = MockAutoregressiveModel::new();
        let mut retriever = GenerativeRetriever::new(model).with_beam_size(*beam_size);
        
        // Add documents to retriever
        for (doc_id, passage_text) in &corpus {
            retriever.add_document(*doc_id as u32, passage_text);
        }
        
        let query = "What is machine learning?";

        group.bench_with_input(
            BenchmarkId::new(
                "retrieve",
                format!("{}docs_beam{}_k{}", n_docs, beam_size, k),
            ),
            &(retriever, query),
            |b, (retriever, query)| {
                b.iter(|| {
                    let _ = black_box(retriever.retrieve(query, *k).unwrap());
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_heuristic_scoring,
    bench_identifier_generation,
    bench_full_retrieval
);
criterion_main!(benches);
