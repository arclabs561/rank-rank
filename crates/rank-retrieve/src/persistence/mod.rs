//! Disk persistence for rank-retrieve indexes.
//!
//! This module provides crash-safe, concurrent persistence for all retrieval methods:
//! - Sparse retrieval (BM25, TF-IDF): Inverted indexes with compressed postings
//! - Dense retrieval: Vector storage with ANN indexes (HNSW, IVF-PQ, DiskANN)
//! - Hybrid retrieval: Unified persistence for combined sparse + dense systems
//!
//! # Design Philosophy
//!
//! The persistence layer prioritizes:
//! - **Correctness**: Crash-safe, ACID guarantees, data integrity
//! - **Concurrency**: Multiple readers, single writer with snapshot isolation
//! - **Performance**: Memory mapping, SIMD-accelerated compression, efficient formats
//! - **Flexibility**: Support for all retrieval methods, configurable trade-offs
//!
//! See `docs/PERSISTENCE_DESIGN.md` for comprehensive design documentation.
//! See `docs/PERSISTENCE_DESIGN_DENSE.md` for dense retrieval specifics.

pub mod directory;
pub mod format;
pub mod error;

#[cfg(feature = "persistence")]
pub mod codec;

#[cfg(feature = "persistence")]
pub mod segment;

#[cfg(feature = "persistence")]
pub mod wal;

#[cfg(feature = "persistence")]
pub mod checkpoint;

#[cfg(feature = "persistence")]
pub mod recovery;

#[cfg(all(feature = "persistence", feature = "dense"))]
pub mod dense;

#[cfg(all(feature = "persistence", feature = "hnsw"))]
pub mod hnsw;

#[cfg(all(feature = "persistence", feature = "ivf_pq"))]
pub mod ivf_pq;

#[cfg(feature = "persistence")]
pub mod locking;

pub use error::PersistenceError;
