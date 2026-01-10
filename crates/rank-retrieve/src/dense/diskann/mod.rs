//! DiskANN implementation.
//!
//! Disk-based approximate nearest neighbor search for very large datasets
//! that don't fit in memory.
//!
//! # References
//!
//! - Jayaram Subramanya et al. (2019): "DiskANN: Fast Accurate Billion-point
//!   Nearest Neighbor Search on a Single Node"

mod graph;
mod disk_io;
mod cache;

pub use graph::DiskANNIndex;
pub use graph::DiskANNParams;
