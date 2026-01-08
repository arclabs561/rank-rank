//! Error types for rank-retrieve.

use std::fmt;

/// Errors that can occur during retrieval operations.
#[derive(Debug, Clone, PartialEq)]
pub enum RetrieveError {
    /// Empty query provided.
    EmptyQuery,
    /// Empty index (no documents indexed).
    EmptyIndex,
    /// Invalid parameter value.
    InvalidParameter(String),
    /// Dimension mismatch between query and documents.
    DimensionMismatch {
        query_dim: usize,
        doc_dim: usize,
    },
    /// Invalid sparse vector (empty or malformed).
    InvalidSparseVector(String),
    /// Other error (for extensibility).
    Other(String),
}

impl fmt::Display for RetrieveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RetrieveError::EmptyQuery => write!(f, "Query is empty"),
            RetrieveError::EmptyIndex => write!(f, "Index is empty"),
            RetrieveError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            RetrieveError::DimensionMismatch { query_dim, doc_dim } => {
                write!(f, "Dimension mismatch: query has {} dimensions, document has {}", query_dim, doc_dim)
            }
            RetrieveError::InvalidSparseVector(msg) => {
                write!(f, "Invalid sparse vector: {}", msg)
            }
            RetrieveError::Other(msg) => {
                write!(f, "Error: {}", msg)
            }
        }
    }
}

impl std::error::Error for RetrieveError {}

