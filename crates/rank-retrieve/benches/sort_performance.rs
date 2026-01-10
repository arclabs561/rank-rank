//! Benchmark comparing sort_by vs sort_unstable_by performance.
//!
//! Measures the performance improvement from using unstable sorting
//! for ranking operations where stability is not required.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::cmp::Ordering;

fn generate_scores(n: usize) -> Vec<(u32, f32)> {
    (0..n)
        .map(|i| {
            // Generate scores with some duplicates to test stability behavior
            let score = ((i * 7 + 13) % 1000) as f32 / 1000.0;
            (i as u32, score)
        })
        .collect()
}

fn bench_sort_stable_vs_unstable(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort_performance");

    for size in [100, 1000, 10000, 100000] {
        let mut scores = generate_scores(size);

        group.bench_with_input(
            BenchmarkId::new("sort_by", size),
            &scores,
            |b, s| {
                b.iter(|| {
                    let mut data = s.clone();
                    data.sort_by(|a, b| b.1.total_cmp(&a.1));
                    black_box(data);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("sort_unstable_by", size),
            &scores,
            |b, s| {
                b.iter(|| {
                    let mut data = s.clone();
                    data.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));
                    black_box(data);
                })
            },
        );
    }

    group.finish();
}

fn bench_sort_with_top_k(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort_with_top_k");

    for (size, k) in [(10000, 10), (100000, 100), (1000000, 1000)] {
        let scores = generate_scores(size);

        // Full sort then take top-k
        group.bench_with_input(
            BenchmarkId::new("full_sort_then_take", format!("{}_k{}", size, k)),
            &scores,
            |b, s| {
                b.iter(|| {
                    let mut data = s.clone();
                    data.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));
                    let _ = black_box(data.into_iter().take(k).collect::<Vec<_>>());
                })
            },
        );

        // Heap-based top-k (simulating early termination)
        group.bench_with_input(
            BenchmarkId::new("heap_top_k", format!("{}_k{}", size, k)),
            &scores,
            |b, s| {
                use std::collections::BinaryHeap;
                use std::cmp::Reverse;

                #[derive(PartialEq, PartialOrd)]
                struct FloatOrd(f32);
                impl Eq for FloatOrd {}
                impl Ord for FloatOrd {
                    fn cmp(&self, other: &Self) -> Ordering {
                        self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal)
                    }
                }

                b.iter(|| {
                    let mut heap: BinaryHeap<Reverse<(FloatOrd, u32)>> =
                        BinaryHeap::with_capacity(k + 1);

                    for (id, score) in s.iter() {
                        let score_ord = FloatOrd(*score);
                        if heap.len() < k {
                            heap.push(Reverse((score_ord, *id)));
                        } else if let Some(&Reverse((FloatOrd(min_score), _))) = heap.peek() {
                            if *score > min_score {
                                heap.pop();
                                heap.push(Reverse((score_ord, *id)));
                            }
                        }
                    }

                    let mut results: Vec<(u32, f32)> = heap
                        .into_iter()
                        .map(|Reverse((FloatOrd(score), id))| (id, score))
                        .collect();
                    results.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));
                    black_box(results);
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_sort_stable_vs_unstable, bench_sort_with_top_k);
criterion_main!(benches);
