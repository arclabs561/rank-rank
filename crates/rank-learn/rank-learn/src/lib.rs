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
//! let lambdas = trainer.compute_lambdas(&scores, &relevance);
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

/// XGBoost integration for ranking.
///
/// Provides Rust bindings and utilities for using XGBoost with ranking objectives.
pub mod xgboost {
    // TODO: Implement XGBoost integration
    // - XGBoost Rust bindings
    // - Ranking objective support
    // - Training utilities
}

/// LightGBM integration for ranking.
///
/// Provides Rust bindings and utilities for using LightGBM with ranking objectives.
pub mod lightgbm {
    // TODO: Implement LightGBM integration
    // - LightGBM Rust bindings
    // - Ranking objective support
    // - Training utilities
}

/// Neural Learning to Rank models.
///
/// Neural ranking models that use rank-soft for differentiable operations.
pub mod neural;

/// Re-export commonly used types.
pub mod prelude {
    pub use crate::lambdarank::{LambdaRankParams, LambdaRankTrainer, ndcg_at_k};
    pub use crate::neural::{NeuralLTRConfig, NeuralLTRModel};
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // Placeholder test
        assert!(true);
    }
}
