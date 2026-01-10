# EVōC Implementation

## Overview

**EVōC** (Embedding Vector Oriented Clustering) is a fast hierarchical clustering library optimized for embedding vectors. This is a pure Rust implementation following the algorithm from the Tutte Institute.

## Algorithm

EVōC combines two stages:

1. **Dimensionality Reduction**: UMAP-style reduction from original dimension (e.g., 1536) to intermediate space (~15 dimensions), preserving clustering structure
2. **Hierarchical Clustering**: HDBSCAN-style clustering using Minimum Spanning Tree (MST) on the reduced space

### Key Features

- **Multi-granularity clustering**: Automatically extracts clusters at multiple granularity levels
- **Automatic cluster selection**: No need to specify number of clusters
- **Near-duplicate detection**: Identifies potential duplicate embeddings
- **Optimized for embeddings**: Specialized for high-dimensional embedding vectors

## Usage

### Basic Clustering

```rust
use rank_retrieve::dense::evoc::clustering::{EVoC, EVoCParams};

let dimension = 1536;
let num_vectors = 10000;
let vectors: Vec<f32> = /* your embeddings */;

let params = EVoCParams {
    intermediate_dim: 15,
    min_cluster_size: 10,
    noise_level: 0.0,
    min_number_clusters: None,
};

let mut evoc = EVoC::new(dimension, params)?;
let assignments = evoc.fit_predict(&vectors, num_vectors)?;

// Get multi-granularity layers
let layers = evoc.cluster_layers();
for (i, layer) in layers.iter().enumerate() {
    println!("Layer {}: {} clusters", i, layer.num_clusters);
}

// Get near-duplicates
let duplicates = evoc.duplicates();
```

### As Partitioning Alternative to k-means

EVōC can replace k-means in Anisotropic VQ + k-means (SCANN)/IVF-PQ partitioning:

```rust
use rank_retrieve::dense::partitioning::{Partitioner, EVoCPartitioner};

let mut partitioner = EVoCPartitioner::new(dimension, num_partitions)?;
partitioner.fit(&vectors, num_vectors)?;
let assignments = partitioner.assign(&vectors, num_vectors)?;
```

## Relationship to ANN Methods

EVōC is **not** an ANN search algorithm. It's a clustering/routing tool that complements ANN:

- **Position**: Partitioning/routing stage (similar to k-means in IVF-PQ or Anisotropic VQ + k-means (SCANN))
- **Can replace**: k-means in partitioning stages
- **Provides**: Hierarchical clustering, near-duplicate detection, multi-granularity clusters

### Typical Architecture

```
Traditional (k-means):
1. k-means partition corpus → flat clusters
2. Build ANN index per cluster (HNSW, IVF-PQ, etc.)
3. Query: route to cluster → search within cluster

EVōC Alternative:
1. EVōC cluster corpus → hierarchical clusters
2. Build ANN index per cluster (HNSW, IVF-PQ, etc.)
3. Query: route to cluster → search within cluster
4. Bonus: Can use different granularity levels for different queries
```

## Implementation Details

### Dimensionality Reduction

- Reduces from original dimension to `intermediate_dim` (~15)
- Uses simplified PCA/random projection (production would use UMAP-style manifold learning)
- Preserves neighborhood structure for clustering

### Hierarchical Clustering

- Builds Minimum Spanning Tree (MST) from pairwise distances in reduced space
- Extracts clusters at multiple distance thresholds
- Uses `min_cluster_size` to filter noise

### Multi-Granularity Extraction

- Automatically identifies stable clusters across different thresholds
- Finest layer: most clusters, smallest size
- Coarsest layer: fewest clusters, largest size

## Performance

- **Time complexity**: O(n²) for MST construction (can be optimized with approximate nearest neighbors)
- **Space complexity**: O(n²) for distance matrix (can be optimized)
- **Optimized for**: High-dimensional embeddings (384-1536 dimensions)

## References

- Tutte Institute: https://github.com/TutteInstitute/evoc
- Combines UMAP dimensionality reduction + HDBSCAN hierarchical clustering
- Optimized specifically for embedding vectors
