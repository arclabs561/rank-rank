//! Comprehensive benchmarks for all ANN algorithms.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rank_retrieve::RetrieveError;

/// Generate random normalized vectors for benchmarking.
fn generate_benchmark_vectors(num: usize, dimension: usize) -> Vec<Vec<f32>> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut vectors = Vec::new();
    
    for _ in 0..num {
        let mut vec = Vec::with_capacity(dimension);
        let mut norm = 0.0;
        
        for _ in 0..dimension {
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

/// Benchmark construction time.
fn bench_construction<F>(c: &mut Criterion, name: &str, create_index: F)
where
    F: Fn(usize) -> Result<Box<dyn rank_retrieve::dense::ann::ANNIndex>, RetrieveError>,
{
    let dimension = 128;
    let sizes = vec![1000, 10000, 100000];
    
    for size in sizes {
        let vectors = generate_benchmark_vectors(size, dimension);
        
        c.bench_with_input(
            BenchmarkId::new(format!("{}_construction", name), size),
            &size,
            |b, _| {
                b.iter(|| {
                    let mut index = create_index(dimension).unwrap();
                    for (i, vec) in vectors.iter().enumerate() {
                        index.add(i as u32, vec.clone()).unwrap();
                    }
                    index.build().unwrap();
                    black_box(index);
                });
            },
        );
    }
}

/// Benchmark search time.
fn bench_search<F>(c: &mut Criterion, name: &str, create_index: F)
where
    F: Fn(usize) -> Result<Box<dyn rank_retrieve::dense::ann::ANNIndex>, RetrieveError>,
{
    let dimension = 128;
    let num_vectors = 10000;
    let k = 10;
    
    let vectors = generate_benchmark_vectors(num_vectors, dimension);
    
    // Build index once
    let mut index = create_index(dimension).unwrap();
    for (i, vec) in vectors.iter().enumerate() {
        index.add(i as u32, vec.clone()).unwrap();
    }
    index.build().unwrap();
    
    // Benchmark search
    let queries = generate_benchmark_vectors(100, dimension);
    
    c.bench_function(&format!("{}_search", name), |b| {
        b.iter(|| {
            for query in &queries {
                let results = index.search(query, k).unwrap();
                black_box(results);
            }
        });
    });
}

/// Benchmark recall vs brute-force.
fn bench_recall<F>(c: &mut Criterion, name: &str, create_index: F)
where
    F: Fn(usize) -> Result<Box<dyn rank_retrieve::dense::ann::ANNIndex>, RetrieveError>,
{
    let dimension = 128;
    let num_vectors = 1000;
    let k = 10;
    
    let vectors = generate_benchmark_vectors(num_vectors, dimension);
    
    // Build index
    let mut index = create_index(dimension).unwrap();
    for (i, vec) in vectors.iter().enumerate() {
        index.add(i as u32, vec.clone()).unwrap();
    }
    index.build().unwrap();
    
    // Benchmark recall calculation
    c.bench_function(&format!("{}_recall", name), |b| {
        let query = &vectors[0];
        
        // Brute-force ground truth
        let mut brute_force: Vec<(u32, f32)> = vectors
            .iter()
            .enumerate()
            .map(|(i, vec)| {
                let dist = 1.0 - rank_retrieve::simd::dot(query, vec);
                (i as u32, dist)
            })
            .collect();
        brute_force.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let ground_truth: Vec<u32> = brute_force.iter().take(k).map(|(id, _)| *id).collect();
        
        b.iter(|| {
            let ann_results = index.search(query, k).unwrap();
            let ann_ids: Vec<u32> = ann_results.iter().map(|(id, _)| *id).collect();
            
            let intersection = ground_truth
                .iter()
                .filter(|id| ann_ids.contains(id))
                .count();
            let recall = intersection as f32 / k as f32;
            black_box(recall);
        });
    });
}

#[cfg(feature = "hnsw")]
fn bench_hnsw(c: &mut Criterion) {
    use rank_retrieve::dense::hnsw::{HNSWIndex, HNSWParams};
    
    let create_index = |dim: usize| -> Result<Box<dyn rank_retrieve::dense::ann::ANNIndex>, RetrieveError> {
        let params = HNSWParams::default();
        let index = HNSWIndex::new(dim, params.m, params.m_max)?;
        Ok(Box::new(index))
    };
    
    bench_construction(c, "hnsw", create_index);
    bench_search(c, "hnsw", create_index);
    bench_recall(c, "hnsw", create_index);
}

#[cfg(feature = "sng")]
fn bench_sng(c: &mut Criterion) {
    use rank_retrieve::dense::sng::{SNGIndex, SNGParams};
    
    let create_index = |dim: usize| -> Result<Box<dyn rank_retrieve::dense::ann::ANNIndex>, RetrieveError> {
        let params = SNGParams::default();
        let index = SNGIndex::new(dim, params)?;
        Ok(Box::new(index))
    };
    
    bench_construction(c, "sng", create_index);
    bench_search(c, "sng", create_index);
    bench_recall(c, "sng", create_index);
}

#[cfg(feature = "scann")]
fn bench_scann(c: &mut Criterion) {
    use rank_retrieve::dense::scann::{SCANNIndex, SCANNParams};
    
    let create_index = |dim: usize| -> Result<Box<dyn rank_retrieve::dense::ann::ANNIndex>, RetrieveError> {
        let params = SCANNParams::default();
        let index = SCANNIndex::new(dim, params)?;
        Ok(Box::new(index))
    };
    
    bench_construction(c, "scann", create_index);
    bench_search(c, "scann", create_index);
    bench_recall(c, "scann", create_index);
}

#[cfg(feature = "ivf_pq")]
fn bench_ivf_pq(c: &mut Criterion) {
    use rank_retrieve::dense::ivf_pq::{IVFPQIndex, IVFPQParams};
    
    let create_index = |dim: usize| -> Result<Box<dyn rank_retrieve::dense::ann::ANNIndex>, RetrieveError> {
        let params = IVFPQParams::default();
        let index = IVFPQIndex::new(dim, params)?;
        Ok(Box::new(index))
    };
    
    bench_construction(c, "ivf_pq", create_index);
    bench_search(c, "ivf_pq", create_index);
    bench_recall(c, "ivf_pq", create_index);
}

criterion_group!(
    benches,
    #[cfg(feature = "hnsw")]
    bench_hnsw,
    #[cfg(feature = "sng")]
    bench_sng,
    #[cfg(feature = "scann")]
    bench_scann,
    #[cfg(feature = "ivf_pq")]
    bench_ivf_pq,
);
criterion_main!(benches);
