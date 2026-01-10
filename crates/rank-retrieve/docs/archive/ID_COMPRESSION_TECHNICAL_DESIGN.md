# ID Compression Technical Design

Detailed technical design for implementing lossless ID compression in rank-retrieve.

## Core Compression Module Structure

```
src/compression/
├── mod.rs              # Public API, re-exports
├── error.rs            # CompressionError type
├── traits.rs           # Compression trait definitions
├── ans.rs              # ANS encoder/decoder wrapper
├── roc.rs              # Random Order Coding implementation
├── elias_fano.rs       # Elias-Fano baseline (for comparison)
└── utils.rs            # Helper functions (factorial, log, etc.)
```

## Type Definitions

### Compression Error

```rust
// src/compression/error.rs
#[derive(Debug, thiserror::Error)]
pub enum CompressionError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
    
    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),
    
    #[error("ANS encoding error: {0}")]
    AnsError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Compression Traits

```rust
// src/compression/traits.rs
use crate::compression::error::CompressionError;

/// Trait for compressing sets of IDs where order doesn't matter.
pub trait IdSetCompressor {
    /// Compress a set of IDs (order-invariant).
    /// 
    /// # Arguments
    /// 
    /// * `ids` - Sorted, unique IDs (must be sorted for correctness)
    /// * `universe_size` - Maximum possible ID value (for entropy calculation)
    /// 
    /// # Returns
    /// 
    /// Compressed representation as byte vector.
    fn compress_set(
        &self,
        ids: &[u32],
        universe_size: u32,
    ) -> Result<Vec<u8>, CompressionError>;
    
    /// Decompress a set of IDs.
    /// 
    /// # Returns
    /// 
    /// Sorted vector of IDs.
    fn decompress_set(
        &self,
        compressed: &[u8],
        universe_size: u32,
    ) -> Result<Vec<u32>, CompressionError>;
    
    /// Estimate compressed size without full compression.
    /// 
    /// Useful for deciding whether to compress.
    fn estimate_size(&self, num_ids: usize, universe_size: u32) -> usize;
    
    /// Get compression ratio for given parameters.
    /// 
    /// Returns bits per ID.
    fn bits_per_id(&self, num_ids: usize, universe_size: u32) -> f64;
}
```

## ANS Implementation

### ANS Wrapper

```rust
// src/compression/ans.rs
use constriction::stream::{stack::DefaultAnsCoder, model::DefaultEncoderModel};

/// ANS encoder/decoder wrapper using constriction crate.
pub struct AnsCoder {
    // Internal ANS state
    // Using constriction's stack-based ANS
}

impl AnsCoder {
    /// Create new ANS coder with given precision.
    pub fn new(precision: u32) -> Self {
        // Initialize constriction ANS coder
    }
    
    /// Encode a symbol with given probability.
    pub fn encode(&mut self, symbol: u32, probability: f64) -> Result<(), CompressionError> {
        // Use constriction API
    }
    
    /// Decode a symbol with given probability model.
    pub fn decode(&mut self, probability_model: &ProbabilityModel) -> Result<u32, CompressionError> {
        // Use constriction API
    }
    
    /// Get compressed data.
    pub fn into_bytes(self) -> Vec<u8> {
        // Extract compressed bitstream
    }
    
    /// Create from compressed data.
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, CompressionError> {
        // Reconstruct ANS state
    }
}
```

**Alternative (if constriction doesn't work well)**:

```rust
// Minimal ANS implementation based on paper
pub struct AnsCoder {
    state: u64,  // ANS state
    precision: u32,  // Quantization precision (typically 2^12 or 2^16)
}

impl AnsCoder {
    fn encode(&mut self, symbol: u32, cdf: &[u32]) -> Result<(), CompressionError> {
        // Implementation from paper Equation (1)
        // encode_p(s, x) = r * floor(s / p_x) + c_x + (s mod p_x)
    }
    
    fn decode(&mut self, cdf: &[u32]) -> Result<u32, CompressionError> {
        // Implementation from paper Equation (2-3)
        // x = s' mod r
        // s = decode_p(s', x)
    }
}
```

## ROC Implementation

### ROC Compressor

```rust
// src/compression/roc.rs
use crate::compression::traits::IdSetCompressor;
use crate::compression::ans::AnsCoder;
use crate::compression::error::CompressionError;

/// Random Order Coding compressor for sets.
/// 
/// Implements bits-back coding with ANS to compress sets of IDs
/// where order doesn't matter.
pub struct RocCompressor {
    ans_precision: u32,  // ANS quantization precision
}

impl RocCompressor {
    pub fn new() -> Self {
        Self {
            ans_precision: 1 << 12,  // 4096, good balance
        }
    }
    
    pub fn with_precision(precision: u32) -> Self {
        Self { ans_precision: precision }
    }
}

impl IdSetCompressor for RocCompressor {
    fn compress_set(
        &self,
        ids: &[u32],
        universe_size: u32,
    ) -> Result<Vec<u8>, CompressionError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        // Validate: IDs must be sorted and unique
        for i in 1..ids.len() {
            if ids[i] <= ids[i-1] {
                return Err(CompressionError::InvalidInput(
                    "IDs must be sorted and unique".to_string()
                ));
            }
        }
        
        // Build probability model
        // Uniform model: p(x) = 1 / (universe_size - num_seen)
        // As we encode, the remaining universe shrinks
        
        let mut coder = AnsCoder::new(self.ans_precision);
        
        // Bits-back coding:
        // 1. Sample permutation z from approximate posterior q(z|ids)
        // 2. Encode (ids, z) using joint model p(ids, z)
        // 3. The "bits back" from sampling z provides the savings
        
        // For ROC, we use uniform permutation model
        // q(z|ids) = 1 / n! where n = ids.len()
        
        // Encode using bits-back:
        // - Sample permutation (removes log(n!) bits from state)
        // - Encode IDs in permuted order (adds log(C(N, n)) bits)
        // - Net: log(C(N, n)) bits (optimal for sets)
        
        // Implementation:
        // 1. Initialize ANS state with random bits (for initial sampling)
        let mut state = initialize_ans_state();
        
        // 2. For each position in permutation:
        for i in 0..ids.len() {
            // Sample which remaining ID to place here
            let remaining = ids.len() - i;
            let remaining_universe = universe_size - ids[i];
            
            // Uniform probability over remaining IDs
            let prob = 1.0 / (remaining_universe as f64);
            
            // Decode (sample) from ANS state
            let sampled_idx = coder.decode_with_uniform(remaining)?;
            
            // Encode the actual ID
            coder.encode(ids[i], prob)?;
        }
        
        // 3. Extract compressed data
        Ok(coder.into_bytes())
    }
    
    fn decompress_set(
        &self,
        compressed: &[u8],
        universe_size: u32,
    ) -> Result<Vec<u32>, CompressionError> {
        // Reverse process:
        // 1. Reconstruct ANS state
        let mut coder = AnsCoder::from_bytes(compressed.to_vec())?;
        
        // 2. Decode IDs in reverse order (ANS is stack-like)
        let mut ids = Vec::new();
        
        // Decode in reverse (last encoded first)
        // ... implementation ...
        
        // 3. Sort to return canonical order
        ids.sort();
        Ok(ids)
    }
    
    fn estimate_size(&self, num_ids: usize, universe_size: u32) -> usize {
        // Theoretical: log(C(universe_size, num_ids)) bits
        // Approximate using Stirling's approximation
        if num_ids == 0 {
            return 0;
        }
        
        // log(C(N, n)) ≈ n * log(N/n) + O(n)
        let bits = (num_ids as f64) * ((universe_size as f64) / (num_ids as f64)).log2();
        (bits / 8.0).ceil() as usize
    }
    
    fn bits_per_id(&self, num_ids: usize, universe_size: u32) -> f64 {
        if num_ids == 0 {
            return 0.0;
        }
        
        // Optimal bits per ID for a set
        let total_bits = (num_ids as f64) * ((universe_size as f64) / (num_ids as f64)).log2();
        total_bits / (num_ids as f64)
    }
}
```

**Note**: The actual bits-back implementation is more complex. This is a simplified version. The full implementation needs:
- Proper permutation sampling
- Correct CDF construction
- Handling of initial bits issue

## Integration with IVF-PQ

### Modified Cluster Structure

```rust
// src/dense/ivf_pq/search.rs (modifications)

use crate::compression::traits::IdSetCompressor;
use crate::compression::roc::RocCompressor;

/// Storage for cluster IDs (compressed or uncompressed).
enum ClusterStorage {
    /// Uncompressed IDs (current implementation).
    Uncompressed(Vec<u32>),
    
    /// Compressed IDs using ROC.
    Compressed {
        data: Vec<u8>,
        num_ids: usize,
        universe_size: u32,
    },
}

/// Cluster (inverted list) with optional compression.
pub(crate) struct Cluster {
    storage: ClusterStorage,
    // Cache for decompressed IDs (cleared after search)
    #[cfg(feature = "id-compression")]
    decompressed_cache: Option<Vec<u32>>,
}

impl Cluster {
    /// Create uncompressed cluster.
    pub fn new(ids: Vec<u32>) -> Self {
        Self {
            storage: ClusterStorage::Uncompressed(ids),
            #[cfg(feature = "id-compression")]
            decompressed_cache: None,
        }
    }
    
    /// Create compressed cluster.
    #[cfg(feature = "id-compression")]
    pub fn new_compressed(
        ids: Vec<u32>,
        compressor: &RocCompressor,
        universe_size: u32,
    ) -> Result<Self, CompressionError> {
        // Sort IDs (required for compression)
        let mut sorted_ids = ids;
        sorted_ids.sort();
        sorted_ids.dedup();
        
        // Compress
        let compressed = compressor.compress_set(&sorted_ids, universe_size)?;
        
        Ok(Self {
            storage: ClusterStorage::Compressed {
                data: compressed,
                num_ids: sorted_ids.len(),
                universe_size,
            },
            decompressed_cache: None,
        })
    }
    
    /// Get IDs (decompress if needed).
    pub fn get_ids(&mut self) -> Result<&[u32], CompressionError> {
        match &self.storage {
            ClusterStorage::Uncompressed(ids) => Ok(ids),
            ClusterStorage::Compressed { data, universe_size, .. } => {
                // Check cache first
                if let Some(ref cached) = self.decompressed_cache {
                    return Ok(cached);
                }
                
                // Decompress
                let compressor = RocCompressor::new();
                let decompressed = compressor.decompress_set(data, *universe_size)?;
                
                // Cache (will be cleared after search)
                self.decompressed_cache = Some(decompressed);
                Ok(self.decompressed_cache.as_ref().unwrap())
            }
        }
    }
    
    /// Get number of IDs.
    pub fn len(&self) -> usize {
        match &self.storage {
            ClusterStorage::Uncompressed(ids) => ids.len(),
            ClusterStorage::Compressed { num_ids, .. } => *num_ids,
        }
    }
    
    /// Clear decompression cache (call after search).
    #[cfg(feature = "id-compression")]
    pub fn clear_cache(&mut self) {
        self.decompressed_cache = None;
    }
}
```

### Modified IVFPQIndex

```rust
// src/dense/ivf_pq/search.rs (modifications)

impl IVFPQIndex {
    /// Build index with optional compression.
    pub fn build(&mut self) -> Result<(), RetrieveError> {
        // ... existing k-means clustering ...
        
        // Assign vectors to clusters
        let assignments = kmeans.assign_clusters(&self.vectors, self.num_vectors);
        self.clusters = vec![Cluster::new(Vec::new()); self.params.num_clusters];
        
        for (vector_idx, &cluster_idx) in assignments.iter().enumerate() {
            self.clusters[cluster_idx].add_vector(vector_idx as u32);
        }
        
        // Compress clusters if enabled
        if let Some(compression_method) = &self.params.id_compression {
            self.compress_clusters(compression_method)?;
        }
        
        self.built = true;
        Ok(())
    }
    
    #[cfg(feature = "id-compression")]
    fn compress_clusters(
        &mut self,
        method: &IdCompressionMethod,
    ) -> Result<(), CompressionError> {
        let compressor = match method {
            IdCompressionMethod::Roc => RocCompressor::new(),
            _ => return Ok(()), // Other methods not implemented yet
        };
        
        let universe_size = self.num_vectors as u32;
        
        // Compress each cluster
        for cluster in &mut self.clusters {
            let ids = cluster.get_ids()?.to_vec();
            
            // Only compress if large enough (threshold)
            if ids.len() > 100 {
                *cluster = Cluster::new_compressed(ids, &compressor, universe_size)?;
            }
        }
        
        Ok(())
    }
    
    /// Search with automatic decompression.
    pub fn search(
        &self,
        query: &[f32],
        k: usize,
    ) -> Result<Vec<(u32, f32)>, RetrieveError> {
        // ... existing cluster selection ...
        
        // Search in top nprobe clusters
        let mut candidates = Vec::new();
        
        for (cluster_idx, _) in cluster_distances.iter().take(self.params.nprobe) {
            let cluster = &mut self.clusters[*cluster_idx];  // Need mut for decompression
            
            // Get IDs (decompresses if needed, caches result)
            let ids = cluster.get_ids()?;
            
            for &vector_idx in ids {
                // ... existing distance computation ...
            }
        }
        
        // Clear caches after search
        #[cfg(feature = "id-compression")]
        for cluster in &mut self.clusters {
            cluster.clear_cache();
        }
        
        // ... existing sorting and return ...
    }
}
```

**Issue**: `search()` needs `&mut self` for decompression cache, but current API uses `&self`.

**Solution**: Use interior mutability:

```rust
use std::sync::Mutex;  // Or RwLock for concurrent access

struct Cluster {
    storage: ClusterStorage,
    decompressed_cache: Mutex<Option<Vec<u32>>>,  // Interior mutability
}

impl Cluster {
    fn get_ids(&self) -> Result<Vec<u32>, CompressionError> {  // Now &self
        match &self.storage {
            ClusterStorage::Uncompressed(ids) => Ok(ids.clone()),
            ClusterStorage::Compressed { .. } => {
                let mut cache = self.decompressed_cache.lock().unwrap();
                if let Some(ref cached) = *cache {
                    return Ok(cached.clone());
                }
                
                // Decompress and cache
                let decompressed = /* ... */;
                *cache = Some(decompressed.clone());
                Ok(decompressed)
            }
        }
    }
}
```

## Integration with HNSW

### Modified Layer Structure

```rust
// src/dense/hnsw/graph.rs (modifications)

use crate::compression::traits::IdSetCompressor;
use crate::compression::roc::RocCompressor;

/// Storage for neighbor lists (compressed or uncompressed).
enum NeighborStorage {
    /// Uncompressed neighbors (current).
    Uncompressed(Vec<SmallVec<[u32; 16]>>),
    
    /// Compressed neighbors.
    Compressed {
        data: Vec<CompressedNeighborList>,
        compressor_id: CompressorId,
    },
}

struct CompressedNeighborList {
    data: Vec<u8>,
    num_neighbors: usize,
}

pub(crate) struct Layer {
    storage: NeighborStorage,
    #[cfg(feature = "id-compression")]
    decompressed_cache: Mutex<HashMap<u32, SmallVec<[u32; 16]>>>,
}

impl Layer {
    /// Get neighbors for a node (decompress if needed).
    pub fn get_neighbors(&self, node: u32) -> Result<SmallVec<[u32; 16]>, CompressionError> {
        match &self.storage {
            NeighborStorage::Uncompressed(neighbors) => {
                Ok(neighbors[node as usize].clone())
            }
            NeighborStorage::Compressed { data, .. } => {
                // Check cache
                let mut cache = self.decompressed_cache.lock().unwrap();
                if let Some(cached) = cache.get(&node) {
                    return Ok(cached.clone());
                }
                
                // Decompress
                let compressed = &data[node as usize];
                let compressor = RocCompressor::new();
                let decompressed = compressor.decompress_set(
                    &compressed.data,
                    /* universe_size */,
                )?;
                
                let neighbors: SmallVec<[u32; 16]> = decompressed.into();
                cache.insert(node, neighbors.clone());
                Ok(neighbors)
            }
        }
    }
}
```

## Performance Optimizations

### 1. Threshold-Based Compression

```rust
// Only compress if beneficial
fn should_compress(num_ids: usize, universe_size: u32, method: &IdCompressionMethod) -> bool {
    match method {
        IdCompressionMethod::Roc => {
            // ROC benefits from larger sets
            // Overhead: initial bits + ANS state
            // Benefit: log(n!) bits saved
            // Threshold: compress if n > 100 (empirically determined)
            num_ids > 100
        }
        IdCompressionMethod::EliasFano => {
            // EF always beneficial for sorted sequences
            num_ids > 10
        }
        _ => false,
    }
}
```

### 2. Batch Decompression

```rust
// Decompress multiple clusters in parallel
fn decompress_clusters_parallel(
    clusters: &[Cluster],
    indices: &[usize],
) -> Result<Vec<Vec<u32>>, CompressionError> {
    use rayon::prelude::*;
    
    indices.par_iter()
        .map(|&idx| {
            let cluster = &clusters[idx];
            cluster.get_ids().map(|ids| ids.to_vec())
        })
        .collect()
}
```

### 3. SIMD-Optimized Decompression

For large decompressed sets, use SIMD for:
- Sorting decompressed IDs
- Merging with existing candidates
- Distance computation

## Testing

### Unit Tests

```rust
// src/compression/roc.rs (tests)

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_roc_round_trip() {
        let compressor = RocCompressor::new();
        let ids: Vec<u32> = vec![1, 5, 10, 20, 50, 100];
        let universe_size = 1000;
        
        let compressed = compressor.compress_set(&ids, universe_size).unwrap();
        let decompressed = compressor.decompress_set(&compressed, universe_size).unwrap();
        
        // Should get same set (order may differ, so sort)
        let mut expected = ids.clone();
        expected.sort();
        let mut actual = decompressed;
        actual.sort();
        
        assert_eq!(expected, actual);
    }
    
    #[test]
    fn test_roc_compression_ratio() {
        let compressor = RocCompressor::new();
        let num_ids = 1000;
        let universe_size = 1_000_000;
        
        let ids: Vec<u32> = (0..num_ids).map(|i| i * 1000).collect();
        
        let compressed = compressor.compress_set(&ids, universe_size).unwrap();
        let uncompressed_size = num_ids * 4;  // 4 bytes per u32
        let compressed_size = compressed.len();
        
        let ratio = uncompressed_size as f64 / compressed_size as f64;
        
        // Should achieve ~5-7x compression for large sets
        assert!(ratio > 4.0, "Compression ratio too low: {}", ratio);
    }
    
    #[test]
    fn test_roc_empty_set() {
        let compressor = RocCompressor::new();
        let compressed = compressor.compress_set(&[], 1000).unwrap();
        assert!(compressed.is_empty());
        
        let decompressed = compressor.decompress_set(&[], 1000).unwrap();
        assert!(decompressed.is_empty());
    }
}
```

### Integration Tests

```rust
// tests/ivf_compression.rs

#[test]
fn test_ivf_with_compression() {
    let mut index = IVFPQIndex::new(
        128,
        IVFPQParams {
            num_clusters: 1024,
            id_compression: Some(IdCompressionMethod::Roc),
            ..Default::default()
        },
    ).unwrap();
    
    // Add vectors
    for i in 0..10000 {
        index.add(i, vec![0.0; 128]).unwrap();
    }
    
    // Build (compression happens here)
    index.build().unwrap();
    
    // Verify clusters are compressed
    for cluster in &index.clusters {
        if cluster.len() > 100 {
            assert!(matches!(cluster.storage, ClusterStorage::Compressed { .. }));
        }
    }
    
    // Search should work identically
    let query = vec![1.0; 128];
    let results = index.search(&query, 10).unwrap();
    
    assert_eq!(results.len(), 10);
}
```

## Benchmarking

```rust
// benches/id_compression.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rank_retrieve::compression::roc::RocCompressor;

fn bench_roc_compress(c: &mut Criterion) {
    let compressor = RocCompressor::new();
    let ids: Vec<u32> = (0..10000).map(|i| i * 10).collect();
    let universe_size = 1_000_000;
    
    c.bench_function("roc_compress_10k", |b| {
        b.iter(|| {
            compressor.compress_set(black_box(&ids), black_box(universe_size))
        })
    });
}

fn bench_roc_decompress(c: &mut Criterion) {
    let compressor = RocCompressor::new();
    let ids: Vec<u32> = (0..10000).map(|i| i * 10).collect();
    let universe_size = 1_000_000;
    let compressed = compressor.compress_set(&ids, universe_size).unwrap();
    
    c.bench_function("roc_decompress_10k", |b| {
        b.iter(|| {
            compressor.decompress_set(black_box(&compressed), black_box(universe_size))
        })
    });
}

criterion_group!(benches, bench_roc_compress, bench_roc_decompress);
criterion_main!(benches);
```

## Next Steps

1. **Start with ANS wrapper**: Implement basic ANS using `constriction` or `rans`
2. **Implement ROC**: Start with simplified version, iterate
3. **Integrate with IVF**: Add compression option, test with real data
4. **Benchmark**: Measure compression ratio and performance impact
5. **Optimize**: Profile and optimize hot paths
6. **Extend**: Add HNSW support, persistence integration
