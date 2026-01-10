//! Benchmarks for ID compression performance and ratios.

#[cfg(all(feature = "id-compression", feature = "criterion"))]
mod benches {
    use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
    use rank_retrieve::compression::{RocCompressor, IdSetCompressor};
    use rand::Rng;

    fn generate_sorted_ids(num_ids: usize, universe_size: u32) -> Vec<u32> {
        let mut rng = rand::thread_rng();
        let mut ids: Vec<u32> = (0..num_ids)
            .map(|_| rng.gen_range(0..universe_size))
            .collect();
        ids.sort();
        ids.dedup();
        ids
    }

    fn bench_compression_ratio(c: &mut Criterion) {
        let compressor = RocCompressor::new();
        let universe_size = 1_000_000;
        
        let mut group = c.benchmark_group("compression_ratio");
        
        for num_ids in [100, 500, 1000, 5000, 10000].iter() {
            let ids = generate_sorted_ids(*num_ids, universe_size);
            let uncompressed_size = ids.len() * 4;  // 4 bytes per u32
            
            group.bench_with_input(
                BenchmarkId::from_parameter(num_ids),
                &ids,
                |b, ids| {
                    b.iter(|| {
                        let compressed = compressor.compress_set(black_box(ids), universe_size).unwrap();
                        black_box(compressed.len())
                    });
                },
            );
            
            // Measure actual compression ratio
            let compressed = compressor.compress_set(&ids, universe_size).unwrap();
            let ratio = uncompressed_size as f64 / compressed.len() as f64;
            group.throughput(criterion::Throughput::Bytes(compressed.len() as u64));
            
            println!("Compression ratio for {} IDs: {:.2}x", num_ids, ratio);
        }
        
        group.finish();
    }

    fn bench_decompression_speed(c: &mut Criterion) {
        let compressor = RocCompressor::new();
        let universe_size = 1_000_000;
        
        let mut group = c.benchmark_group("decompression_speed");
        
        for num_ids in [100, 500, 1000, 5000].iter() {
            let ids = generate_sorted_ids(*num_ids, universe_size);
            let compressed = compressor.compress_set(&ids, universe_size).unwrap();
            
            group.bench_with_input(
                BenchmarkId::from_parameter(num_ids),
                &compressed,
                |b, compressed| {
                    b.iter(|| {
                        let decompressed = compressor.decompress_set(black_box(compressed), universe_size).unwrap();
                        black_box(decompressed.len())
                    });
                },
            );
        }
        
        group.finish();
    }

    fn bench_round_trip(c: &mut Criterion) {
        let compressor = RocCompressor::new();
        let universe_size = 1_000_000;
        let ids = generate_sorted_ids(1000, universe_size);
        
        c.bench_function("round_trip_1000_ids", |b| {
            b.iter(|| {
                let compressed = compressor.compress_set(black_box(&ids), universe_size).unwrap();
                let decompressed = compressor.decompress_set(black_box(&compressed), universe_size).unwrap();
                assert_eq!(ids, decompressed);
            });
        });
    }

    criterion_group!(
        benches,
        bench_compression_ratio,
        bench_decompression_speed,
        bench_round_trip
    );
    criterion_main!(benches);
}
