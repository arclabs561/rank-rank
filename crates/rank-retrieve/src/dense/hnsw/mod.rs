//! Hierarchical Navigable Small World (HNSW) approximate nearest neighbor search.
//!
//! Pure Rust implementation optimized for performance with SIMD acceleration and
//! cache-friendly memory layouts.
//!
//! # Algorithm
//!
//! HNSW constructs a multi-layer graph where:
//! - **Upper layers**: Sparse, long-range connections for fast navigation
//! - **Lower layers**: Dense, local connections for precise search
//! - **Search**: Start at top layer, navigate down to base layer, greedy search
//!
//! # Critical Note: Hierarchy Benefits
//!
//! Recent research (2024-2025) suggests that **the hierarchical structure provides
//! minimal or no practical benefit in high-dimensional settings** (d > 32). Flat
//! Navigable Small World (NSW) graphs achieve performance parity with hierarchical
//! HNSW in both median and tail latency, while using less memory.
//!
//! The explanation: **hubness** in high-dimensional spaces creates natural "highway"
//! nodes that serve the same functional role as explicit hierarchy. When metric hubs
//! already provide efficient routing, explicit hierarchical layers become redundant.
//!
//! **Implications**:
//! - For high-dimensional data (d > 32), consider flat NSW variants for memory savings
//! - Hierarchy may still help for low-dimensional data (d < 32) or angular distance metrics
//! - See `docs/CRITICAL_PERSPECTIVES_AND_LIMITATIONS.md` for detailed analysis
//!
//! # Performance
//!
//! - **SIMD-accelerated**: Uses existing `simd` module for distance computation (8-16x speedup)
//! - **Cache-optimized**: Structure of Arrays (SoA) layout for better cache locality
//! - **Early termination**: Multiple strategies to reduce unnecessary distance computations
//! - **O(log n) complexity**: Logarithmic search time vs O(n) brute-force
//!
//! # Usage
//!
//! ```rust
//! use rank_retrieve::dense::hnsw::HNSWIndex;
//!
//! # fn main() -> Result<(), rank_retrieve::RetrieveError> {
//! let mut index = HNSWIndex::new(128, 16, 16)?;
//!
//! // Add vectors
//! index.add(0, vec![0.1; 128])?;
//! index.add(1, vec![0.2; 128])?;
//!
//! // Build index (required before search)
//! index.build()?;
//!
//! // Search
//! let results = index.search(&vec![0.15; 128], 10, 50)?;
//! # Ok(())
//! # }
//! ```
//!
//! # References
//!
//! - Malkov & Yashunin (2016): "Efficient and robust approximate nearest neighbor search
//!   using Hierarchical Navigable Small World graphs"
//! - See `docs/HNSW_IMPLEMENTATION_PLAN.md` for detailed implementation notes

#[cfg(feature = "hnsw")]
pub(crate) mod graph;
#[cfg(feature = "hnsw")]
mod search;
#[cfg(feature = "hnsw")]
pub(crate) mod construction;
#[cfg(feature = "hnsw")]
mod memory;
#[cfg(feature = "hnsw")]
pub(crate) mod distance;

#[cfg(feature = "hnsw")]
pub use graph::{HNSWIndex, HNSWParams, SeedSelectionStrategy, NeighborhoodDiversification};
