//! Performance benchmarks for index factory.
//!
//! Compares factory-created indexes against directly created indexes
//! to ensure there's no performance overhead.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rank_retrieve::dense::ann::factory::index_factory;
use rank_retrieve::dense::ann::ANNIndex;

fn generate_vectors(num: usize, dim: usize) -> Vec<Vec<f32>> {
    use rand::Rng;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    
    let mut rng = StdRng::seed_from_u64(42);
    let mut vectors = Vec::new();
    
    for _ in 0..num {
        let mut vec = Vec::with_capacity(dim);
        let mut norm = 0.0;
        
        for _ in 0..dim {
            let val = rng.gen::<f32>() * 2.0 - 1.0;
            norm += val * val;
            vec.push(val);
        }
        
        let norm = norm.sqrt();
        if norm > 0.0 {
            for val in &mut vec {
                *val /= norm;
            }
        }
        
        vectors.push(vec);
    }
    
    vectors
}

#[cfg(feature = "hnsw")]
fn bench_factory_vs_direct_hnsw(c: &mut Criterion) {
    let dimension = 128;
    let num_vectors = 1000;
    let vectors = generate_vectors(num_vectors, dimension);
    
    let mut group = c.benchmark_group("factory_vs_direct_hnsw");
    
    // Benchmark factory creation
    group.bench_function("factory_create", |b| {
        b.iter(|| {
            let _index = black_box(index_factory(dimension, "HNSW32").unwrap());
        });
    });
    
    // Benchmark direct creation
    group.bench_function("direct_create", |b| {
        b.iter(|| {
            use rank_retrieve::dense::hnsw::HNSWIndex;
            let _index = black_box(HNSWIndex::new(dimension, 32, 32).unwrap());
        });
    });
    
    // Benchmark factory add + build
    group.bench_function("factory_add_build", |b| {
        b.iter(|| {
            let mut index = index_factory(dimension, "HNSW32").unwrap();
            for (i, vec) in vectors.iter().enumerate() {
                index.add(i as u32, vec.clone()).unwrap();
            }
            index.build().unwrap();
            black_box(index);
        });
    });
    
    // Benchmark direct add + build
    group.bench_function("direct_add_build", |b| {
        b.iter(|| {
            use rank_retrieve::dense::hnsw::HNSWIndex;
            let mut index = HNSWIndex::new(dimension, 32, 32).unwrap();
            for (i, vec) in vectors.iter().enumerate() {
                index.add(i as u32, vec.clone()).unwrap();
            }
            index.build().unwrap();
            black_box(index);
        });
    });
    
    group.finish();
}

#[cfg(feature = "ivf_pq")]
fn bench_factory_vs_direct_ivf_pq(c: &mut Criterion) {
    let dimension = 128;
    let num_vectors = 1000;
    let vectors = generate_vectors(num_vectors, dimension);
    
    let mut group = c.benchmark_group("factory_vs_direct_ivf_pq");
    
    // Benchmark factory creation
    group.bench_function("factory_create", |b| {
        b.iter(|| {
            let _index = black_box(index_factory(dimension, "IVF64,PQ8").unwrap());
        });
    });
    
    // Benchmark direct creation
    group.bench_function("direct_create", |b| {
        b.iter(|| {
            use rank_retrieve::dense::ivf_pq::{IVFPQIndex, IVFPQParams};
            let params = IVFPQParams {
                num_clusters: 64,
                nprobe: 8,
                num_codebooks: 8,
                codebook_size: 256,
            };
            let _index = black_box(IVFPQIndex::new(dimension, params).unwrap());
        });
    });
    
    // Benchmark factory add + build
    group.bench_function("factory_add_build", |b| {
        b.iter(|| {
            let mut index = index_factory(dimension, "IVF64,PQ8").unwrap();
            for (i, vec) in vectors.iter().enumerate() {
                index.add(i as u32, vec.clone()).unwrap();
            }
            index.build().unwrap();
            black_box(index);
        });
    });
    
    // Benchmark direct add + build
    group.bench_function("direct_add_build", |b| {
        b.iter(|| {
            use rank_retrieve::dense::ivf_pq::{IVFPQIndex, IVFPQParams};
            let params = IVFPQParams {
                num_clusters: 64,
                nprobe: 8,
                num_codebooks: 8,
                codebook_size: 256,
            };
            let mut index = IVFPQIndex::new(dimension, params).unwrap();
            for (i, vec) in vectors.iter().enumerate() {
                index.add(i as u32, vec.clone()).unwrap();
            }
            index.build().unwrap();
            black_box(index);
        });
    });
    
    group.finish();
}

#[cfg(feature = "hnsw")]
fn bench_factory_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("factory_parsing");
    
    group.bench_function("parse_hnsw", |b| {
        b.iter(|| {
            let _ = black_box(index_factory(128, "HNSW32"));
        });
    });
    
    group.bench_function("parse_ivf_pq", |b| {
        b.iter(|| {
            let _ = black_box(index_factory(128, "IVF1024,PQ8"));
        });
    });
    
    group.bench_function("parse_scann", |b| {
        b.iter(|| {
            let _ = black_box(index_factory(128, "SCANN256"));
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    #[cfg(feature = "hnsw")]
    bench_factory_vs_direct_hnsw,
    #[cfg(feature = "ivf_pq")]
    bench_factory_vs_direct_ivf_pq,
    #[cfg(feature = "hnsw")]
    bench_factory_parsing
);
criterion_main!(benches);
