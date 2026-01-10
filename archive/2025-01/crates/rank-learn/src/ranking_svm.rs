//! Ranking SVM implementation.
//!
//! Ranking SVM is a pairwise learning-to-rank method that converts ranking
//! into a binary classification problem in an expanded space of document pairs.
//!
//! Based on:
//! - Herbrich et al. 1999, 2000: "Large Margin Rank Boundaries for Ordinal Regression"
//! - Joachims 2002: "Optimizing Search Engines using Clickthrough Data"
//! - Cao et al. 2006: "Adapting Ranking SVM to Document Retrieval" (with query normalization and cost sensitivity)

use crate::error::LearnError;

/// Ranking SVM parameters.
#[derive(Debug, Clone, Copy)]
pub struct RankingSVMParams {
    /// Regularization parameter (C in SVM formulation).
    /// Controls the trade-off between margin maximization and training error.
    /// Default: 1.0
    pub c: f32,
    /// Enable query normalization (μ weights from Cao et al. 2006).
    /// Prevents queries with many pairs from dominating training.
    /// Default: true
    pub query_normalization: bool,
    /// Enable cost sensitivity (τ weights for position-based importance).
    /// Errors at top ranks matter more than errors at lower ranks.
    /// Default: true
    pub cost_sensitivity: bool,
    /// Epsilon for numerical stability in comparisons.
    /// Default: 1e-10
    pub epsilon: f32,
}

impl Default for RankingSVMParams {
    fn default() -> Self {
        Self {
            c: 1.0,
            query_normalization: true,
            cost_sensitivity: true,
            epsilon: 1e-10,
        }
    }
}

/// Compute Ranking SVM loss for a pair of documents.
///
/// The hinge loss for a pair (i, j) where document i should rank higher than j:
/// loss = max(0, 1 - (score_i - score_j))
///
/// # Arguments
///
/// * `score_i` - Score for document i (should be higher)
/// * `score_j` - Score for document j (should be lower)
/// * `params` - Ranking SVM parameters
///
/// # Returns
///
/// Hinge loss value for this pair
pub fn pairwise_hinge_loss(score_i: f32, score_j: f32, _params: RankingSVMParams) -> f32 {
    let score_diff = score_i - score_j;
    (1.0 - score_diff).max(0.0)
}

/// Compute Ranking SVM gradients for a ranked list.
///
/// For each pair (i, j) where relevance[i] > relevance[j]:
/// - If score[i] - score[j] < 1: gradient[i] += C, gradient[j] -= C
///
/// # Arguments
///
/// * `scores` - Model scores for documents (in ranked order)
/// * `relevance` - Ground truth relevance scores (in same order as scores)
/// * `params` - Ranking SVM parameters
///
/// # Returns
///
/// Vector of gradient values for each document
pub fn compute_ranking_svm_gradients(
    scores: &[f32],
    relevance: &[f32],
    params: RankingSVMParams,
) -> Result<Vec<f32>, LearnError> {
    if scores.len() != relevance.len() {
        return Err(LearnError::LengthMismatch {
            scores_len: scores.len(),
            relevance_len: relevance.len(),
        });
    }

    if scores.is_empty() {
        return Err(LearnError::EmptyInput);
    }

    let n = scores.len();
    let mut gradients = vec![0.0; n];

    // Count valid pairs for query normalization (μ weight)
    let mut valid_pairs = 0;
    for i in 0..n {
        for j in (i + 1)..n {
            if (relevance[i] - relevance[j]).abs() > params.epsilon {
                valid_pairs += 1;
            }
        }
    }

    // Query normalization weight: μ = 1.0 if disabled, or normalized by number of pairs
    let mu = if params.query_normalization && valid_pairs > 0 {
        1.0 / valid_pairs as f32
    } else {
        1.0
    };

    // For each pair (i, j) where i < j
    for i in 0..n {
        for j in (i + 1)..n {
            // Only consider pairs where relevance differs
            let rel_diff = relevance[i] - relevance[j];
            if rel_diff.abs() < params.epsilon {
                continue;
            }

            // Determine which document should rank higher
            let (high_idx, low_idx) = if rel_diff > 0.0 { (i, j) } else { (j, i) };

            let score_diff = scores[high_idx] - scores[low_idx];

            // Hinge loss: max(0, 1 - (score_high - score_low))
            // Only contribute to gradient if margin is violated (score_diff < 1)
            if score_diff < 1.0 {
                // Cost sensitivity weight (τ): errors at top ranks matter more
                let tau = if params.cost_sensitivity {
                    // Weight by inverse rank position (top positions get higher weight)
                    let min_rank = high_idx.min(low_idx);
                    // Use (rank + 2) to avoid division by zero at rank 0
                    1.0 / ((min_rank + 2) as f32).ln()
                } else {
                    1.0
                };

                // Gradient contribution: C * μ * τ
                let gradient_contribution = params.c * mu * tau;

                // Update gradients
                gradients[high_idx] += gradient_contribution;
                gradients[low_idx] -= gradient_contribution;
            }
        }
    }

    Ok(gradients)
}

/// Ranking SVM trainer.
///
/// Trains a ranking model using Ranking SVM gradients.
pub struct RankingSVMTrainer {
    params: RankingSVMParams,
}

impl RankingSVMTrainer {
    /// Create a new Ranking SVM trainer.
    pub fn new(params: RankingSVMParams) -> Self {
        Self { params }
    }

    /// Compute gradients for a query-document list.
    ///
    /// # Arguments
    ///
    /// * `scores` - Model scores for documents
    /// * `relevance` - Ground truth relevance scores
    ///
    /// # Returns
    ///
    /// Gradient values for each document
    ///
    /// # Errors
    ///
    /// Returns `LearnError::EmptyInput` if scores or relevance is empty.
    /// Returns `LearnError::LengthMismatch` if scores and relevance have different lengths.
    pub fn compute_gradients(
        &self,
        scores: &[f32],
        relevance: &[f32],
    ) -> Result<Vec<f32>, LearnError> {
        compute_ranking_svm_gradients(scores, relevance, self.params)
    }
}

impl Default for RankingSVMTrainer {
    fn default() -> Self {
        Self::new(RankingSVMParams::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pairwise_hinge_loss() {
        let params = RankingSVMParams::default();

        // Correct ordering: score_i > score_j + 1 (margin satisfied)
        let loss1 = pairwise_hinge_loss(2.0, 0.5, params);
        assert_eq!(loss1, 0.0);

        // Margin violation: score_i - score_j < 1
        let loss2 = pairwise_hinge_loss(1.0, 0.5, params);
        assert!((loss2 - 0.5).abs() < 1e-6);

        // Large margin violation
        let loss3 = pairwise_hinge_loss(0.0, 1.0, params);
        assert!((loss3 - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_ranking_svm_gradients() {
        let params = RankingSVMParams::default();
        let scores = vec![0.5, 0.8, 0.3];
        let relevance = vec![3.0, 1.0, 2.0];

        let gradients = compute_ranking_svm_gradients(&scores, &relevance, params).unwrap();

        // Should have gradients for all documents
        assert_eq!(gradients.len(), 3);

        // Document 0 (rel=3) should rank highest, so should have positive gradient
        // Document 1 (rel=1) should rank lowest, so should have negative gradient
        // Document 2 (rel=2) is in the middle
        assert!(gradients[0] > 0.0); // Highest relevance
        assert!(gradients[1] < 0.0); // Lowest relevance
    }

    #[test]
    fn test_ranking_svm_trainer() {
        let trainer = RankingSVMTrainer::default();
        let scores = vec![0.5, 0.8, 0.3];
        let relevance = vec![3.0, 1.0, 2.0];

        let gradients = trainer.compute_gradients(&scores, &relevance).unwrap();
        assert_eq!(gradients.len(), 3);
        assert!(gradients.iter().all(|&g| g.is_finite()));
    }

    #[test]
    fn test_query_normalization() {
        // Test that query normalization reduces gradient magnitude for queries with many pairs
        let params_with_norm = RankingSVMParams {
            query_normalization: true,
            ..Default::default()
        };
        let params_without_norm = RankingSVMParams {
            query_normalization: false,
            ..Default::default()
        };

        // Query with many pairs
        let scores: Vec<f32> = (0..10).map(|i| i as f32 * 0.1).collect();
        let relevance: Vec<f32> = (0..10).rev().map(|i| i as f32).collect();

        let gradients_norm =
            compute_ranking_svm_gradients(&scores, &relevance, params_with_norm).unwrap();
        let gradients_no_norm =
            compute_ranking_svm_gradients(&scores, &relevance, params_without_norm).unwrap();

        // Normalized gradients should have smaller magnitude
        let norm_norm: f32 = gradients_norm.iter().map(|g| g * g).sum::<f32>().sqrt();
        let norm_no_norm: f32 = gradients_no_norm.iter().map(|g| g * g).sum::<f32>().sqrt();

        // With normalization, the gradient magnitude should be smaller
        assert!(norm_norm < norm_no_norm || (norm_norm - norm_no_norm).abs() < 1e-6);
    }

    #[test]
    fn test_cost_sensitivity() {
        // Test that cost sensitivity gives higher weights to top positions
        let params_with_cost = RankingSVMParams {
            cost_sensitivity: true,
            ..Default::default()
        };
        let params_without_cost = RankingSVMParams {
            cost_sensitivity: false,
            ..Default::default()
        };

        let scores = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let relevance = vec![5.0, 4.0, 3.0, 2.0, 1.0];

        let gradients_cost =
            compute_ranking_svm_gradients(&scores, &relevance, params_with_cost).unwrap();
        let gradients_no_cost =
            compute_ranking_svm_gradients(&scores, &relevance, params_without_cost).unwrap();

        // With cost sensitivity, top positions should have larger gradient magnitudes
        // (absolute values)
        let top_gradient_cost = gradients_cost[0].abs();
        let top_gradient_no_cost = gradients_no_cost[0].abs();

        // Top position should have larger gradient with cost sensitivity
        assert!(top_gradient_cost >= top_gradient_no_cost);
    }

    #[test]
    fn test_error_handling() {
        let params = RankingSVMParams::default();

        // Empty input
        let result = compute_ranking_svm_gradients(&[], &[], params);
        assert!(matches!(result, Err(LearnError::EmptyInput)));

        // Length mismatch
        let result = compute_ranking_svm_gradients(&[1.0, 2.0], &[1.0], params);
        assert!(matches!(result, Err(LearnError::LengthMismatch { .. })));
    }
}
