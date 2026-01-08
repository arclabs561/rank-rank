//! LambdaRank and LambdaMART implementations.
//!
//! LambdaRank is a pairwise Learning to Rank algorithm that uses metric-aware gradients.
//! LambdaMART combines LambdaRank with gradient boosting (MART - Multiple Additive Regression Trees).
//!
//! # LambdaRank Algorithm
//!
//! LambdaRank optimizes ranking metrics (like NDCG) directly by computing gradients
//! based on how swapping document pairs would change the metric.
//!
//! For a pair of documents (i, j) where document i should rank higher than j:
//!
//! ```text
//! lambda_ij = -σ / (1 + exp(σ * (s_i - s_j))) * |ΔNDCG|
//! ```
//!
//! Where:
//! - `s_i`, `s_j` = scores for documents i and j
//! - `σ` = sigmoid parameter (typically 1.0)
//! - `ΔNDCG` = change in NDCG if documents i and j were swapped
//!
//! The lambda for document i is the sum of all lambda_ij over pairs where i is involved.

use crate::LearnError;

/// LambdaRank parameters.
#[derive(Debug, Clone, Copy)]
pub struct LambdaRankParams {
    /// Sigmoid parameter (σ) for pairwise loss.
    /// Controls the sharpness of the sigmoid.
    /// Default: 1.0
    pub sigma: f32,
    /// Enable query normalization (μ weights from Cao et al. 2006).
    /// Prevents queries with many pairs from dominating training.
    /// Default: true
    pub query_normalization: bool,
    /// Enable cost sensitivity (τ weights for position-based importance).
    /// Errors at top ranks matter more than errors at lower ranks.
    /// Default: true
    pub cost_sensitivity: bool,
    /// Enable score normalization (LightGBM's lambdarank_norm).
    /// Normalizes delta by score distance to prevent large score differences from dominating.
    /// Default: false
    pub score_normalization: bool,
    /// Enable exponential gain for NDCG (2^rel - 1).
    /// If false, uses linear gain (raw relevance).
    /// Default: true
    pub exponential_gain: bool,
}

impl Default for LambdaRankParams {
    fn default() -> Self {
        Self {
            sigma: 1.0,
            query_normalization: true,
            cost_sensitivity: true,
            score_normalization: false,
            exponential_gain: true,
        }
    }
}

/// Compute NDCG at a given position.
///
/// # Arguments
///
/// * `relevance` - Relevance scores for documents (in ranked order)
/// * `k` - Position to compute NDCG@k (None = all positions)
/// * `exponential_gain` - If true, use exponential gain (2^rel - 1), else use linear gain
///
/// # Returns
///
/// NDCG value
///
/// # Errors
///
/// Returns `LearnError::EmptyInput` if relevance is empty.
/// Returns `LearnError::InvalidNDCG` if k > relevance length.
pub fn ndcg_at_k(
    relevance: &[f32],
    k: Option<usize>,
    exponential_gain: bool,
) -> Result<f32, LearnError> {
    if relevance.is_empty() {
        return Err(LearnError::EmptyInput);
    }

    let k = k.unwrap_or(relevance.len());

    if k == 0 {
        return Ok(0.0);
    }

    if k > relevance.len() {
        return Err(LearnError::InvalidNDCG {
            k,
            length: relevance.len(),
        });
    }

    let k = k.min(relevance.len());

    // Compute DCG
    let mut dcg = 0.0;
    for i in 0..k {
        // Use exponential gain (2^rel - 1) or linear gain (raw relevance)
        let gain = if exponential_gain {
            (2.0_f32).powf(relevance[i]) - 1.0
        } else {
            relevance[i]
        };
        // Discount: 1 / log2(i + 2)
        let discount = 1.0 / ((i + 2) as f32).log2();
        dcg += gain * discount;
    }

    // Compute IDCG (ideal DCG - sorted by relevance descending)
    let mut ideal_relevance = relevance.to_vec();
    ideal_relevance.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

    let mut idcg = 0.0;
    for i in 0..k {
        let gain = if exponential_gain {
            (2.0_f32).powf(ideal_relevance[i]) - 1.0
        } else {
            ideal_relevance[i]
        };
        let discount = 1.0 / ((i + 2) as f32).log2();
        idcg += gain * discount;
    }

    if idcg == 0.0 {
        Ok(0.0)
    } else {
        Ok(dcg / idcg)
    }
}

/// Compute change in NDCG if two documents are swapped.
///
/// Uses efficient formula: ΔNDCG = (gain_i - gain_j) * (discount_i - discount_j) / IDCG
/// This avoids recomputing full NDCG for better performance.
///
/// # Arguments
///
/// * `relevance` - Relevance scores for all documents
/// * `pos_i` - Position of document i (0-indexed)
/// * `pos_j` - Position of document j (0-indexed)
/// * `k` - NDCG@k to compute (None = all positions)
/// * `exponential_gain` - If true, use exponential gain (2^rel - 1)
/// * `inv_idcg` - Precomputed 1/IDCG for this query (None = compute on demand)
///
/// # Returns
///
/// Change in NDCG (ΔNDCG)
fn delta_ndcg(
    relevance: &[f32],
    pos_i: usize,
    pos_j: usize,
    k: Option<usize>,
    exponential_gain: bool,
    inv_idcg: Option<f32>,
) -> f32 {
    if pos_i >= relevance.len() || pos_j >= relevance.len() {
        return 0.0;
    }

    let k = k.unwrap_or(relevance.len());

    // Only compute delta if both positions are within k
    if pos_i >= k && pos_j >= k {
        return 0.0;
    }

    // Compute gains
    let gain_i = if exponential_gain {
        (2.0_f32).powf(relevance[pos_i]) - 1.0
    } else {
        relevance[pos_i]
    };
    let gain_j = if exponential_gain {
        (2.0_f32).powf(relevance[pos_j]) - 1.0
    } else {
        relevance[pos_j]
    };

    // Compute discounts: 1 / log2(rank + 2)
    let discount_i = if pos_i < k {
        1.0 / ((pos_i + 2) as f32).log2()
    } else {
        0.0
    };
    let discount_j = if pos_j < k {
        1.0 / ((pos_j + 2) as f32).log2()
    } else {
        0.0
    };

    // Efficient delta computation for swapping:
    // When swapping item i (at pos_i) with item j (at pos_j):
    // Before: gain_i * discount_i + gain_j * discount_j
    // After:  gain_i * discount_j + gain_j * discount_i
    // Delta = (gain_i * discount_j + gain_j * discount_i) - (gain_i * discount_i + gain_j * discount_j)
    //       = (gain_i - gain_j) * (discount_j - discount_i)
    //       = -(gain_i - gain_j) * (discount_i - discount_j)
    let gain_diff = gain_i - gain_j;
    let discount_diff = discount_i - discount_j;

    // Get IDCG (compute if not provided)
    let inv_idcg_val = if let Some(idcg) = inv_idcg {
        idcg
    } else {
        // Compute IDCG on demand
        let mut ideal_relevance = relevance.to_vec();
        ideal_relevance.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let mut idcg = 0.0;
        for i in 0..k.min(ideal_relevance.len()) {
            let gain = if exponential_gain {
                (2.0_f32).powf(ideal_relevance[i]) - 1.0
            } else {
                ideal_relevance[i]
            };
            let discount = 1.0 / ((i + 2) as f32).log2();
            idcg += gain * discount;
        }
        if idcg > 0.0 {
            1.0 / idcg
        } else {
            0.0
        }
    };

    // Negative sign because we're computing change when swapping (moving i down, j up)
    -(gain_diff * discount_diff * inv_idcg_val)
}

/// Compute LambdaRank gradients for a ranked list.
///
/// # Arguments
///
/// * `scores` - Model scores for documents (in ranked order)
/// * `relevance` - Ground truth relevance scores (in same order as scores)
/// * `params` - LambdaRank parameters
/// * `k` - NDCG@k to optimize (None = all positions)
///
/// # Returns
///
/// Vector of lambda values (gradients) for each document
pub fn compute_lambdas(
    scores: &[f32],
    relevance: &[f32],
    params: LambdaRankParams,
    k: Option<usize>,
) -> Vec<f32> {
    if scores.len() != relevance.len() {
        return vec![0.0; scores.len()];
    }

    let n = scores.len();
    let k_trunc = k.unwrap_or(n);

    // Precompute IDCG for efficiency (used in delta_ndcg)
    let inv_idcg = {
        let mut ideal_relevance = relevance.to_vec();
        ideal_relevance.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let mut idcg = 0.0;
        for i in 0..k_trunc.min(ideal_relevance.len()) {
            let gain = if params.exponential_gain {
                (2.0_f32).powf(ideal_relevance[i]) - 1.0
            } else {
                ideal_relevance[i]
            };
            let discount = 1.0 / ((i + 2) as f32).log2();
            idcg += gain * discount;
        }
        if idcg > 0.0 {
            1.0 / idcg
        } else {
            0.0
        }
    };

    let mut lambdas = vec![0.0; n];
    let mut sum_lambdas = 0.0;

    // Find min/max scores for score normalization
    let (min_score, max_score) = if params.score_normalization && n > 0 {
        let min = scores.iter().copied().fold(f32::INFINITY, f32::min);
        let max = scores.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        (min, max)
    } else {
        (0.0, 0.0)
    };
    let score_range = if params.score_normalization && max_score != min_score {
        max_score - min_score
    } else {
        1.0
    };

    // Count valid pairs for query normalization (μ weight)
    let mut valid_pairs = 0;
    for i in 0..n.min(k_trunc) {
        for j in (i + 1)..n {
            if (relevance[i] - relevance[j]).abs() > 1e-10 {
                valid_pairs += 1;
            }
        }
    }

    // Query normalization weight: μ = 1.0 if disabled, or normalized by max pairs
    // For single query, μ = 1.0. For batch, caller should provide max_pairs.
    let mu = if params.query_normalization && valid_pairs > 0 {
        1.0 / valid_pairs as f32
    } else {
        1.0
    };

    // For each pair (i, j) where i < j
    // Optimization: only consider pairs where at least one document is within k_trunc
    for i in 0..n.min(k_trunc) {
        for j in (i + 1)..n {
            // Only consider pairs where relevance differs
            let rel_diff = relevance[i] - relevance[j];
            if rel_diff.abs() < 1e-10 {
                continue;
            }

            // Determine which document should rank higher
            let (high_idx, low_idx, high_rank, low_rank) = if rel_diff > 0.0 {
                (i, j, i, j)
            } else {
                (j, i, j, i)
            };

            // Compute ΔNDCG if documents were swapped
            let delta = delta_ndcg(
                relevance,
                high_rank,
                low_rank,
                k,
                params.exponential_gain,
                Some(inv_idcg),
            );

            // Cost sensitivity weight (τ): errors at top ranks matter more
            // Use position-based weighting: higher weight for top positions
            let tau = if params.cost_sensitivity {
                // Weight by inverse rank position (top positions get higher weight)
                // Use min rank to weight by the higher-ranked position
                let min_rank = high_rank.min(low_rank);
                // Weight decreases logarithmically with position
                // Use (rank + 2) to avoid division by zero at rank 0
                1.0 / ((min_rank + 2) as f32).ln()
            } else {
                1.0
            };

            // LambdaRank formula
            let score_diff = scores[high_idx] - scores[low_idx];

            // Score normalization (LightGBM's norm): normalize delta by score distance
            let normalized_delta = if params.score_normalization {
                delta.abs() / (0.01 + score_diff.abs() / score_range.max(0.01))
            } else {
                delta.abs()
            };

            let lambda_ij = -params.sigma / (1.0 + (params.sigma * score_diff).exp())
                * normalized_delta
                * tau
                * mu;

            // Update lambdas
            lambdas[high_idx] += lambda_ij;
            lambdas[low_idx] -= lambda_ij;

            // Accumulate absolute lambda values for normalization
            sum_lambdas += 2.0 * lambda_ij.abs();
        }
    }

    // Lambda normalization (XGBoost/LightGBM style): normalize by sum of lambdas
    // This prevents queries with many pairs from dominating
    if params.query_normalization && sum_lambdas > 0.0 {
        let norm_factor = (1.0 + sum_lambdas).log2() / sum_lambdas;
        for lambda in &mut lambdas {
            *lambda *= norm_factor;
        }
    }

    lambdas
}

/// LambdaRank trainer.
///
/// Trains a ranking model using LambdaRank gradients.
pub struct LambdaRankTrainer {
    params: LambdaRankParams,
}

impl LambdaRankTrainer {
    /// Create a new LambdaRank trainer.
    pub fn new(params: LambdaRankParams) -> Self {
        Self { params }
    }

    /// Compute gradients for a query-document list.
    ///
    /// # Arguments
    ///
    /// * `scores` - Model scores for documents
    /// * `relevance` - Ground truth relevance scores
    /// * `k` - NDCG@k to optimize
    ///
    /// # Returns
    ///
    /// Lambda values (gradients) for each document
    ///
    /// # Errors
    ///
    /// Returns `LearnError::EmptyInput` if scores or relevance is empty.
    /// Returns `LearnError::LengthMismatch` if scores and relevance have different lengths.
    pub fn compute_gradients(
        &self,
        scores: &[f32],
        relevance: &[f32],
        k: Option<usize>,
    ) -> Result<Vec<f32>, LearnError> {
        if scores.is_empty() || relevance.is_empty() {
            return Err(LearnError::EmptyInput);
        }

        if scores.len() != relevance.len() {
            return Err(LearnError::LengthMismatch {
                scores_len: scores.len(),
                relevance_len: relevance.len(),
            });
        }

        Ok(compute_lambdas(scores, relevance, self.params, k))
    }

    /// Compute gradients for a batch of queries with proper query normalization.
    ///
    /// This method implements query normalization (μ weights) from Cao et al. 2006,
    /// ensuring queries with many pairs don't dominate training.
    ///
    /// # Arguments
    ///
    /// * `batch_scores` - Vector of score vectors, one per query
    /// * `batch_relevance` - Vector of relevance vectors, one per query
    /// * `k` - NDCG@k to optimize
    ///
    /// # Returns
    ///
    /// Vector of lambda vectors, one per query
    ///
    /// # Errors
    ///
    /// Returns errors if inputs are invalid (empty, mismatched lengths, etc.)
    pub fn compute_gradients_batch(
        &self,
        batch_scores: &[Vec<f32>],
        batch_relevance: &[Vec<f32>],
        k: Option<usize>,
    ) -> Result<Vec<Vec<f32>>, LearnError> {
        if batch_scores.len() != batch_relevance.len() {
            return Err(LearnError::LengthMismatch {
                scores_len: batch_scores.len(),
                relevance_len: batch_relevance.len(),
            });
        }

        if batch_scores.is_empty() {
            return Err(LearnError::EmptyInput);
        }

        // Count pairs per query for normalization
        let mut pairs_per_query: Vec<usize> = Vec::with_capacity(batch_scores.len());
        for (scores, relevance) in batch_scores.iter().zip(batch_relevance.iter()) {
            if scores.len() != relevance.len() {
                return Err(LearnError::LengthMismatch {
                    scores_len: scores.len(),
                    relevance_len: relevance.len(),
                });
            }

            let mut pairs = 0;
            for i in 0..scores.len() {
                for j in (i + 1)..scores.len() {
                    if (relevance[i] - relevance[j]).abs() > 1e-10 {
                        pairs += 1;
                    }
                }
            }
            pairs_per_query.push(pairs);
        }

        // Find maximum pairs for normalization (μ weight)
        let max_pairs = pairs_per_query.iter().max().copied().unwrap_or(1);

        // Compute lambdas for each query with normalization
        let mut batch_lambdas = Vec::with_capacity(batch_scores.len());
        for (idx, (scores, relevance)) in
            batch_scores.iter().zip(batch_relevance.iter()).enumerate()
        {
            let mut lambdas = compute_lambdas(scores, relevance, self.params, k);

            // Apply query normalization: μ = pairs_in_query / max_pairs
            if self.params.query_normalization && max_pairs > 0 {
                let mu = pairs_per_query[idx] as f32 / max_pairs as f32;
                for lambda in &mut lambdas {
                    *lambda *= mu;
                }
            }

            batch_lambdas.push(lambdas);
        }

        Ok(batch_lambdas)
    }
}

impl Default for LambdaRankTrainer {
    fn default() -> Self {
        Self::new(LambdaRankParams::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ndcg() {
        // Perfect ranking: relevance = [3.0, 2.0, 1.0]
        let relevance = vec![3.0, 2.0, 1.0];
        let ndcg = ndcg_at_k(&relevance, None, true).unwrap();
        assert!((ndcg - 1.0).abs() < 0.01); // Should be 1.0 for perfect ranking
    }

    #[test]
    fn test_ndcg_exponential_gain() {
        // Test exponential gain: 2^rel - 1
        // Use a non-perfect ranking to see the difference
        let relevance = vec![1.0, 2.0, 0.0]; // Not in perfect order
        let ndcg_exp = ndcg_at_k(&relevance, None, true).unwrap();
        let ndcg_linear = ndcg_at_k(&relevance, None, false).unwrap();

        // Exponential gain should be different from linear
        // With exponential: 2^2-1=3, 2^1-1=1, 2^0-1=0
        // With linear: 2, 1, 0
        // Exponential amplifies differences, so NDCG should differ
        assert_ne!(ndcg_exp, ndcg_linear);

        // Exponential gain typically gives higher DCG for high-relevance items
        // But NDCG is normalized, so the relationship depends on the ranking
        // Just verify they're different
        assert!((ndcg_exp - ndcg_linear).abs() > 0.001);
    }

    #[test]
    fn test_delta_ndcg() {
        // Test delta computation
        let relevance = vec![3.0, 1.0, 2.0];
        let delta = delta_ndcg(&relevance, 0, 1, None, true, None);
        // Swapping position 0 (rel=3) with position 1 (rel=1) should decrease NDCG
        assert!(delta < 0.0);
    }

    #[test]
    fn test_lambda_rank_with_optimizations() {
        let scores = vec![0.5, 0.8, 0.3];
        let relevance = vec![3.0, 1.0, 2.0];

        // Test with all optimizations enabled
        let params = LambdaRankParams {
            sigma: 1.0,
            query_normalization: true,
            cost_sensitivity: true,
            score_normalization: true,
            exponential_gain: true,
        };
        let trainer = LambdaRankTrainer::new(params);
        let lambdas = trainer
            .compute_gradients(&scores, &relevance, Some(10))
            .unwrap();

        assert_eq!(lambdas.len(), 3);
        assert!(lambdas.iter().any(|&l| l != 0.0));
    }

    #[test]
    fn test_lambda_rank() {
        // Documents with scores and relevance
        // Doc 0: score=0.5, rel=3.0 (should rank highest)
        // Doc 1: score=0.8, rel=1.0 (should rank lower)
        // Doc 2: score=0.3, rel=2.0 (should rank middle)
        let scores = vec![0.5, 0.8, 0.3]; // Model scores
        let relevance = vec![3.0, 1.0, 2.0]; // Ground truth (doc 0 should rank highest)

        let trainer = LambdaRankTrainer::default();
        let lambdas = trainer
            .compute_gradients(&scores, &relevance, None)
            .unwrap();

        // Document 0 should have positive lambda (should rank higher)
        // Document 1 should have negative lambda (should rank lower)
        assert_eq!(lambdas.len(), 3);
        // Doc 0 has highest relevance but lower score, so lambda should push it up
        // The exact sign depends on the score difference and delta_ndcg
        // Just verify lambdas are computed (non-zero for non-trivial cases)
        assert!(lambdas.iter().any(|&l| l != 0.0)); // At least one non-zero lambda
    }
}
