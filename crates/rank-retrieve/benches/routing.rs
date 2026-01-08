//! Query routing benchmarks.
//!
//! Benchmarks query feature extraction, routing decisions, and routing overhead.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rank_retrieve::routing::{QueryFeatures, QueryRouter, RetrieverId};

fn generate_queries(n: usize) -> Vec<Vec<String>> {
    (0..n)
        .map(|i| {
            match i % 3 {
                0 => vec!["the".to_string(), "quick".to_string(), "brown".to_string()], // Keyword
                1 => vec!["machine".to_string(), "learning".to_string(), "artificial".to_string(), "intelligence".to_string()], // Semantic
                _ => vec!["what".to_string(), "is".to_string(), "prime".to_string(), "rate".to_string()], // Hybrid
            }
        })
        .collect()
}

fn bench_feature_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("routing_feature_extraction");

    for n_queries in [10, 100, 1000].iter() {
        let queries = generate_queries(*n_queries);

        group.bench_with_input(
            BenchmarkId::new("extract_features", n_queries),
            &queries,
            |b, queries| {
                b.iter(|| {
                    for terms in queries {
                        let _ = black_box(QueryFeatures::from_terms(terms));
                    }
                })
            },
        );
    }

    group.finish();
}

fn bench_routing_decision(c: &mut Criterion) {
    let mut group = c.benchmark_group("routing_decision");

    for n_queries in [10, 100, 1000].iter() {
        let queries: Vec<QueryFeatures> = generate_queries(*n_queries)
            .iter()
            .map(|terms| QueryFeatures::from_terms(terms))
            .collect();
        let router = QueryRouter::new().with_routing();

        group.bench_with_input(
            BenchmarkId::new("route_queries", n_queries),
            &queries,
            |b, queries| {
                b.iter(|| {
                    for features in queries {
                        let _ = black_box(router.route(features));
                    }
                })
            },
        );
    }

    group.finish();
}

fn bench_routing_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("routing_overhead");

    let queries: Vec<QueryFeatures> = generate_queries(1000)
        .iter()
        .map(|terms| QueryFeatures::from_terms(terms))
        .collect();

    // Benchmark with routing enabled
    let router_enabled = QueryRouter::new().with_routing();
    group.bench_function("routing_enabled", |b| {
        b.iter(|| {
            for features in &queries {
                let _ = black_box(router_enabled.route(features));
            }
        })
    });

    // Benchmark with routing disabled (fixed retriever)
    let router_disabled = QueryRouter::fixed(RetrieverId::Bm25);
    group.bench_function("routing_disabled", |b| {
        b.iter(|| {
            for features in &queries {
                let _ = black_box(router_disabled.route(features));
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_feature_extraction,
    bench_routing_decision,
    bench_routing_overhead
);
criterion_main!(benches);

