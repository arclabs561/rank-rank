//! Dataset utilities for benchmarking.

#[cfg(feature = "dense")]
use crate::simd;

/// Dataset for benchmarking.
#[derive(Clone)]
pub struct Dataset {
    pub train: Vec<Vec<f32>>,
    pub test: Vec<Vec<f32>>,
    pub dimension: usize,
}

/// Generate synthetic dataset following ann-benchmarks patterns.
///
/// Creates normalized random vectors similar to SIFT/GloVe datasets.
pub fn generate_synthetic_dataset(
    num_vectors: usize,
    dimension: usize,
    seed: u64,
) -> Vec<Vec<f32>> {
    #[cfg(feature = "rand")]
    {
        use rand::Rng;
        use rand::SeedableRng;
        use rand::rngs::StdRng;
    
    let mut rng = StdRng::seed_from_u64(seed);
    let mut vectors = Vec::new();
    
    for _ in 0..num_vectors {
        let mut vec = Vec::with_capacity(dimension);
        let mut norm = 0.0;
        
        // Generate random vector
        for _ in 0..dimension {
            let val = rng.gen::<f32>() * 2.0 - 1.0;
            norm += val * val;
            vec.push(val);
        }
        
        // Normalize (for cosine similarity)
        let norm = f32::sqrt(norm);
        if norm > 0.0 {
            for val in &mut vec {
                *val /= norm;
            }
        }
        
        vectors.push(vec);
    }
    
    vectors
    }
    
    #[cfg(not(feature = "rand"))]
    {
        // Fallback: simple deterministic generation without rand
        let mut vectors = Vec::new();
        for i in 0..num_vectors {
            let mut vec = Vec::with_capacity(dimension);
            let mut norm = 0.0f32;
            
            for j in 0..dimension {
                // Simple deterministic pseudo-random using seed and indices
                let val = ((i * dimension + j) as f32 + seed as f32) * 0.001;
                norm += val * val;
                vec.push(val);
            }
            
            let norm = f32::sqrt(norm);
            if norm > 0.0 {
                for val in &mut vec {
                    *val /= norm;
                }
            }
            
            vectors.push(vec);
        }
        vectors
    }
}

/// Compute ground truth using exact search (brute-force).
///
/// Following ann-benchmarks methodology: always use exact search for ground truth.
pub fn compute_ground_truth(
    query: &[f32],
    dataset: &[Vec<f32>],
    k: usize,
) -> Vec<u32> {
    let mut candidates: Vec<(u32, f32)> = dataset
        .iter()
        .enumerate()
        .map(|(i, vec)| {
            #[cfg(feature = "dense")]
            let dist = 1.0 - simd::dot(query, vec);
            #[cfg(not(feature = "dense"))]
            let dist = {
                let mut dot = 0.0;
                for i in 0..query.len().min(vec.len()) {
                    dot += query[i] * vec[i];
                }
                1.0 - dot
            };
            (i as u32, dist)
        })
        .collect();
    
    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    candidates.iter().take(k).map(|(id, _)| *id).collect()
}

/// Create standard benchmark dataset.
///
/// Splits vectors into train/test sets following ann-benchmarks convention.
pub fn create_benchmark_dataset(
    num_train: usize,
    num_test: usize,
    dimension: usize,
    seed: u64,
) -> Dataset {
    let all_vectors = generate_synthetic_dataset(num_train + num_test, dimension, seed);
    let train = all_vectors[..num_train].to_vec();
    let test = all_vectors[num_train..].to_vec();
    
    Dataset {
        train,
        test,
        dimension,
    }
}

/// Standard benchmark dataset configurations following ann-benchmarks.
#[derive(Debug, Clone, Copy)]
pub enum StandardDataset {
    /// SIFT-1M equivalent: 1M vectors, 128 dimensions, L2 distance
    /// Synthetic equivalent with similar characteristics
    SIFT1M,
    
    /// GloVe-100 equivalent: 1.2M vectors, 100 dimensions, cosine similarity
    /// Synthetic equivalent with normalized vectors
    GloVe100,
    
    /// MNIST equivalent: 60k vectors, 784 dimensions, L2 distance
    /// Synthetic equivalent with high-dimensional sparse-like vectors
    MNIST,
    
    /// NYTimes equivalent: 290k vectors, 256 dimensions, cosine similarity
    /// Synthetic equivalent with normalized vectors
    NYTimes,
    
    /// Random synthetic dataset with custom size and dimension
    Random { num_vectors: usize, dimension: usize },
}

impl StandardDataset {
    /// Get dataset name.
    pub fn name(&self) -> &'static str {
        match self {
            StandardDataset::SIFT1M => "sift-1m",
            StandardDataset::GloVe100 => "glove-100",
            StandardDataset::MNIST => "mnist",
            StandardDataset::NYTimes => "nytimes",
            StandardDataset::Random { .. } => "random",
        }
    }
    
    /// Get dataset configuration.
    pub fn config(&self) -> (usize, usize) {
        match self {
            StandardDataset::SIFT1M => (1_000_000, 128),
            StandardDataset::GloVe100 => (1_200_000, 100),
            StandardDataset::MNIST => (60_000, 784),
            StandardDataset::NYTimes => (290_000, 256),
            StandardDataset::Random { num_vectors, dimension } => (*num_vectors, *dimension),
        }
    }
    
    /// Generate standard benchmark dataset.
    ///
    /// Creates train/test split following ann-benchmarks convention:
    /// - 90% train, 10% test for large datasets (>10k vectors)
    /// - 80% train, 20% test for smaller datasets
    pub fn generate(&self, seed: u64) -> Dataset {
        let (num_vectors, dimension) = self.config();
        
        // Standard train/test split from ann-benchmarks
        let (num_train, num_test) = if num_vectors > 10_000 {
            // Large datasets: 90/10 split
            let num_train = (num_vectors as f32 * 0.9) as usize;
            (num_train, num_vectors - num_train)
        } else {
            // Small datasets: 80/20 split
            let num_train = (num_vectors as f32 * 0.8) as usize;
            (num_train, num_vectors - num_train)
        };
        
        create_benchmark_dataset(num_train, num_test, dimension, seed)
    }
    
    /// Generate smaller version for testing (1/10th size).
    pub fn generate_small(&self, seed: u64) -> Dataset {
        let (num_vectors, dimension) = self.config();
        let small_size = (num_vectors / 10).max(1000);
        
        let (num_train, num_test) = if small_size > 10_000 {
            let num_train = (small_size as f32 * 0.9) as usize;
            (num_train, small_size - num_train)
        } else {
            let num_train = (small_size as f32 * 0.8) as usize;
            (num_train, small_size - num_train)
        };
        
        create_benchmark_dataset(num_train, num_test, dimension, seed)
    }
}

/// Generate all standard datasets for comprehensive benchmarking.
pub fn generate_all_standard_datasets(seed: u64) -> Vec<(String, Dataset)> {
    vec![
        (StandardDataset::SIFT1M.name().to_string(), StandardDataset::SIFT1M.generate(seed)),
        (StandardDataset::GloVe100.name().to_string(), StandardDataset::GloVe100.generate(seed)),
        (StandardDataset::MNIST.name().to_string(), StandardDataset::MNIST.generate(seed)),
        (StandardDataset::NYTimes.name().to_string(), StandardDataset::NYTimes.generate(seed)),
    ]
}

/// Generate small versions of all standard datasets for quick testing.
pub fn generate_all_standard_datasets_small(seed: u64) -> Vec<(String, Dataset)> {
    vec![
        (StandardDataset::SIFT1M.name().to_string(), StandardDataset::SIFT1M.generate_small(seed)),
        (StandardDataset::GloVe100.name().to_string(), StandardDataset::GloVe100.generate_small(seed)),
        (StandardDataset::MNIST.name().to_string(), StandardDataset::MNIST.generate_small(seed)),
        (StandardDataset::NYTimes.name().to_string(), StandardDataset::NYTimes.generate_small(seed)),
    ]
}
