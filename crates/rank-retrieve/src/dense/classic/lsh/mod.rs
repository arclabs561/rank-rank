//! LSH (Locality Sensitive Hashing) implementation.
//!
//! Pure Rust implementation of classic LSH algorithms:
//! - Random Projection LSH (for cosine similarity)
//! - Hash table indexing
//! - Candidate generation and verification
//!
//! # References
//!
//! - Indyk & Motwani (1998): "Approximate nearest neighbors: towards removing
//!   the curse of dimensionality"

mod random_projection;
mod hash_table;
pub mod search;

pub use search::{LSHIndex, LSHParams};
