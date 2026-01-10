//! Error types for rank-learn.

use std::fmt;

/// Errors that can occur during learning to rank operations.
#[derive(Debug, Clone, PartialEq)]
pub enum LearnError {
    /// Empty input provided.
    EmptyInput,
    /// Length mismatch between scores and relevance.
    LengthMismatch {
        scores_len: usize,
        relevance_len: usize,
    },
    /// Invalid parameter value.
    InvalidParameter(String),
    /// Invalid relevance scores (e.g., negative values when not allowed).
    InvalidRelevance(String),
    /// Invalid NDCG computation (e.g., k=0 or k > length).
    InvalidNDCG { k: usize, length: usize },
}

impl fmt::Display for LearnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LearnError::EmptyInput => write!(f, "Input is empty"),
            LearnError::LengthMismatch {
                scores_len,
                relevance_len,
            } => {
                write!(
                    f,
                    "Length mismatch: scores has {} elements, relevance has {}",
                    scores_len, relevance_len
                )
            }
            LearnError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            LearnError::InvalidRelevance(msg) => write!(f, "Invalid relevance: {}", msg),
            LearnError::InvalidNDCG { k, length } => {
                write!(f, "Invalid NDCG@k: k={} but length={}", k, length)
            }
        }
    }
}

impl std::error::Error for LearnError {}
