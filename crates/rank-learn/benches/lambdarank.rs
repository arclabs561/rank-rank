//! LambdaRank benchmarks.
//!
//! Compares performance against Python implementations (XGBoost, LightGBM).

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rank_learn::lambdarank::LambdaRankTrainer;

fn generate_scores_and_relevance(n: usize) -> (Vec<f32>, Vec<f32>) {
    let scores: Vec<f32> = (0..n).map(|i| (i as f32) * 0.1).collect();
    let relevance: Vec<f32> = (0..n).map(|i| ((n - i) as f32) * 0.1).collect(); // Perfect ranking
    (scores, relevance)
}

fn bench_compute_gradients(c: &mut Criterion) {
    let mut group = c.benchmark_group("lambdarank_compute_gradients");
    
    // Reduced sizes for faster benchmarks - focus on realistic use cases
    for n in [10, 50, 100, 200, 500].iter() {
        let (scores, relevance) = generate_scores_and_relevance(*n);
        let trainer = LambdaRankTrainer::default();

        group.bench_with_input(
            BenchmarkId::new("compute_gradients", n),
            &(scores, relevance),
            |b, (s, r)| {
                b.iter(|| {
                    let _ = black_box(trainer.compute_gradients(s, r, None));
                })
            },
        );
    }

    group.finish();
}

fn bench_ndcg_computation(c: &mut Criterion) {
    use rank_learn::lambdarank::ndcg_at_k;

    let mut group = c.benchmark_group("lambdarank_ndcg");

    // Reduced sizes for faster benchmarks
    for n in [10, 50, 100, 200, 500].iter() {
        let relevance: Vec<f32> = (0..*n).map(|i| ((*n - i) as f32) * 0.1).collect();

        group.bench_with_input(
            BenchmarkId::new("ndcg_at_k", n),
            &relevance,
            |b, r| {
                b.iter(|| {
                    let _ = black_box(ndcg_at_k(r, None, true));
                })
            },
        );
    }

    group.finish();
}

fn bench_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("lambdarank_batch");

    // Reduced sizes for faster benchmarks - focus on realistic batch sizes
    for (batch_size, list_size) in [
        (10, 50),
        (20, 100),
        (50, 100),
    ].iter() {
        let batches: Vec<(Vec<f32>, Vec<f32>)> = (0..*batch_size)
            .map(|_| generate_scores_and_relevance(*list_size))
            .collect();
        let trainer = LambdaRankTrainer::default();

        group.bench_with_input(
            BenchmarkId::new("batch", format!("{}x{}", batch_size, list_size)),
            &batches,
            |b, batches| {
                b.iter(|| {
                    for (scores, relevance) in batches {
                        let _ = black_box(trainer.compute_gradients(scores, relevance, None));
                    }
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_compute_gradients,
    bench_ndcg_computation,
    bench_batch_processing
);
criterion_main!(benches);

