//! IVF-PQ (Inverted File Index with Product Quantization) implementation.
//!
//! Memory-efficient ANN algorithm combining:
//! - **IVF**: Inverted file index (clustering-based partitioning)
//! - **PQ**: Product quantization (vector compression)
//!
//! Best for billion-scale datasets with memory constraints.
//!
//! # References
//!
//! - Jégou et al. (2011): "Product Quantization for Nearest Neighbor Search"

mod ivf;
#[cfg(feature = "scann")]
mod opq;
#[cfg(feature = "scann")]
mod online_pq;
mod pq;
pub mod search;

#[cfg(feature = "scann")]
pub use opq::OptimizedProductQuantizer;
#[cfg(feature = "scann")]
pub use online_pq::OnlineProductQuantizer;
pub use search::{IVFPQIndex, IVFPQParams};
