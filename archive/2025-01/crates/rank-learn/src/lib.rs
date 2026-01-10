//! Learning to Rank (LTR) frameworks.
//!
//! Complete LTR algorithms for training ranking models. This crate provides
//! full LTR frameworks, while `rank-soft` provides the differentiable operations
//! used internally.
//!
//! # Relationship to rank-soft
//!
//! - **rank-soft**: Differentiable ranking operations (mathematical primitives)
//!   - Soft ranking, differentiable sorting
//!   - Loss functions (ListNet, ListMLE, Spearman)
//!   - Framework-agnostic building blocks
//!
//! - **rank-learn**: Complete LTR frameworks (full ML systems)
//!   - LambdaRank, LambdaMART
//!   - XGBoost/LightGBM integration for ranking
//!   - Neural LTR models
//!   - Uses `rank-soft` for differentiable operations
//!
//! # Quick Start
//!
//! ```rust
//! use rank_learn::lambdarank::{LambdaRankTrainer, ndcg_at_k};
//!
//! let trainer = LambdaRankTrainer::default();
//! let scores = vec![0.5, 0.8, 0.3];
//! let relevance = vec![3.0, 1.0, 2.0];
//!
//! let lambdas = trainer.compute_gradients(&scores, &relevance, None);
//! // Use lambdas as gradients to update your ranking model
//! ```
//!
//! # Features
//!
//! - **LambdaRank**: Pairwise LTR with metric-aware gradients
//! - **LambdaMART**: Gradient boosting for ranking (MART + LambdaRank)
//! - **XGBoost Integration**: XGBoost with ranking objectives
//! - **LightGBM Integration**: LightGBM with ranking objectives
//! - **Neural LTR**: Neural ranking models using rank-soft

/// LambdaRank and LambdaMART implementations.
///
/// LambdaRank is a pairwise LTR algorithm that uses metric-aware gradients.
/// LambdaMART combines LambdaRank with gradient boosting (MART).
pub mod lambdarank;

/// Ranking SVM implementation.
///
/// Ranking SVM is a pairwise LTR method that converts ranking into binary
/// classification in an expanded space of document pairs.
pub mod ranking_svm;

/// Neural Learning to Rank models.
///
/// Neural ranking models that use rank-soft for differentiable operations.
pub mod neural;

/// Error types for learning to rank operations.
pub mod error;

pub use error::LearnError;

/// Re-export commonly used types.
pub mod prelude {
    pub use crate::lambdarank::{ndcg_at_k, LambdaRankParams, LambdaRankTrainer};
    pub use crate::neural::{NeuralLTRConfig, NeuralLTRModel};
    pub use crate::ranking_svm::{RankingSVMParams, RankingSVMTrainer};
    pub use crate::LearnError;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // Placeholder test
        assert!(true);
    }
}
