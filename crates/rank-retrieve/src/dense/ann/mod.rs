//! Unified Approximate Nearest Neighbor (ANN) search algorithms.
//!
//! Pure Rust implementations of state-of-the-art ANN algorithms:
//! - **HNSW**: Hierarchical Navigable Small World (graph-based) - see `dense::hnsw`
//! - **AnisotropicVQ-kmeans**: Anisotropic Vector Quantization with k-means Partitioning
//!   (vendor name: SCANN/ScaNN) - see `dense::scann`
//! - **IVF-PQ**: Inverted File Index with Product Quantization - see `dense::ivf_pq`
//! - **DiskANN**: Disk-based ANN for very large datasets - see `dense::diskann`
//!
//! All algorithms are optimized with SIMD acceleration and minimal dependencies.
//!
//! # Index Factory
//!
//! Use `factory::index_factory()` to create indexes from string descriptions:
//!
//! ```rust
//! use rank_retrieve::dense::ann::factory::index_factory;
//!
//! // Create HNSW index
//! let mut index = index_factory(128, "HNSW32")?;
//!
//! // Create IVF-PQ index
//! let mut index = index_factory(128, "IVF1024,PQ8")?;
//! ```

pub mod traits;
#[cfg(any(feature = "hnsw", feature = "nsw", feature = "ivf_pq", feature = "scann", feature = "dense"))]
pub mod factory;

pub use traits::{ANNIndex, ANNStats};
#[cfg(any(feature = "hnsw", feature = "nsw", feature = "ivf_pq", feature = "scann", feature = "dense"))]
pub use factory::{index_factory, AnyANNIndex};