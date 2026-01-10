//! Learning-to-rank training for generative retrieval (LTRGR).
//!
//! This module is part of the `generative` module. See `generative/mod.rs` for the main entry point.
//!
//! LTRGR adds a learning-to-rank training phase to generative retrieval,
//! optimizing passage ranking directly using margin-based rank loss.

/// Configuration for LTRGR training.
#[derive(Debug, Clone)]
pub struct LTRGRConfig {
    /// Margin for rank loss (default: 500.0).
    pub margin: f32,
    /// Weight for generation loss in multi-task training (default: 1000.0).
    pub lambda: f32,
    /// Number of top passages to retrieve for training (default: 200).
    pub top_k: usize,
    /// Maximum number of identifiers to keep per passage (default: 40).
    pub max_identifiers_per_passage: usize,
}

impl Default for LTRGRConfig {
    fn default() -> Self {
        Self {
            margin: 500.0,
            lambda: 1000.0,
            top_k: 200,
            max_identifiers_per_passage: 40,
        }
    }
}

impl LTRGRConfig {
    /// Create a new LTRGR configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the margin for rank loss.
    pub fn with_margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }

    /// Set the weight for generation loss.
    pub fn with_lambda(mut self, lambda: f32) -> Self {
        self.lambda = lambda;
        self
    }

    /// Set the number of top passages to retrieve.
    pub fn with_top_k(mut self, top_k: usize) -> Self {
        self.top_k = top_k;
        self
    }

    /// Set the maximum identifiers per passage.
    pub fn with_max_identifiers(mut self, max: usize) -> Self {
        self.max_identifiers_per_passage = max;
        self
    }
}

/// LTRGR trainer for learning-to-rank training phase.
///
/// Implements the margin-based rank loss from the LTRGR paper:
/// L_rank = max(0, s(q, p_n) - s(q, p_p) + m)
///
/// Where:
/// - s(q, p_p) = score of positive passage
/// - s(q, p_n) = score of negative passage
/// - m = margin
pub struct LTRGRTrainer {
    config: LTRGRConfig,
}

impl LTRGRTrainer {
    /// Create a new LTRGR trainer with default configuration.
    pub fn new() -> Self {
        Self {
            config: LTRGRConfig::default(),
        }
    }

    /// Create a new LTRGR trainer with custom configuration.
    pub fn with_config(config: LTRGRConfig) -> Self {
        Self { config }
    }

    /// Compute margin-based rank loss.
    ///
    /// Implements the margin-based rank loss: `L = max(0, margin + negative_score - positive_score)`.
    ///
    /// # Arguments
    ///
    /// * `positive_score` - Score of positive (relevant) passage
    /// * `negative_score` - Score of negative (irrelevant) passage
    ///
    /// # Returns
    ///
    /// The rank loss value. Returns 0 if the positive passage is ranked correctly
    /// (positive_score > negative_score + margin).
    ///
    /// # Example
    ///
    /// ```rust
    /// use rank_retrieve::generative::LTRGRTrainer;
    ///
    /// let trainer = LTRGRTrainer::new();
    /// // Positive passage has higher score, loss should be small
    /// let loss = trainer.compute_rank_loss(10.0, 5.0);
    /// assert!(loss < 500.0); // margin is 500.0 by default
    /// ```
    ///
    /// # Performance
    ///
    /// O(1) operation. Very fast, suitable for use in tight training loops.
    pub fn compute_rank_loss(&self, positive_score: f32, negative_score: f32) -> f32 {
        (self.config.margin + negative_score - positive_score).max(0.0)
    }

    /// Compute rank loss with highest-scoring positive and negative.
    ///
    /// This is L_rank1 from the paper, using the highest-scoring positive
    /// and negative passages from the rank list.
    pub fn compute_rank_loss_1(
        &self,
        passage_scores: &[(u32, f32)],
        positive_passage_ids: &[u32],
    ) -> f32 {
        // Find highest-scoring positive and negative
        let positive_set: std::collections::HashSet<u32> =
            positive_passage_ids.iter().copied().collect();

        let mut best_positive_score = f32::NEG_INFINITY;
        let mut best_negative_score = f32::NEG_INFINITY;

        for (passage_id, score) in passage_scores {
            if positive_set.contains(passage_id) {
                if *score > best_positive_score {
                    best_positive_score = *score;
                }
            } else if *score > best_negative_score {
                best_negative_score = *score;
            }
        }

        if best_positive_score == f32::NEG_INFINITY || best_negative_score == f32::NEG_INFINITY {
            return 0.0;
        }

        self.compute_rank_loss(best_positive_score, best_negative_score)
    }

    /// Compute rank loss with randomly sampled positive and negative.
    ///
    /// This is L_rank2 from the paper, using randomly sampled passages.
    ///
    /// # Random Sampling
    ///
    /// When the `ltrgr` feature is enabled, this method randomly samples one positive
    /// and one negative passage. Without the feature, it uses the first positive/negative
    /// (deterministic but less optimal for training).
    pub fn compute_rank_loss_2(
        &self,
        passage_scores: &[(u32, f32)],
        positive_passage_ids: &[u32],
    ) -> f32 {
        use std::collections::HashSet;
        let positive_set: HashSet<u32> = positive_passage_ids.iter().copied().collect();

        // Sample one positive and one negative
        let positive_samples: Vec<_> = passage_scores
            .iter()
            .filter(|(id, _)| positive_set.contains(id))
            .collect();

        let negative_samples: Vec<_> = passage_scores
            .iter()
            .filter(|(id, _)| !positive_set.contains(id))
            .collect();

        if positive_samples.is_empty() || negative_samples.is_empty() {
            return 0.0;
        }

        #[cfg(feature = "ltrgr")]
        {
            use rand::seq::SliceRandom;
            use rand::thread_rng;
            let mut rng = thread_rng();

            // Randomly sample one positive and one negative
            let (_, pos_score) = positive_samples.choose(&mut rng).unwrap();
            let (_, neg_score) = negative_samples.choose(&mut rng).unwrap();

            self.compute_rank_loss(*pos_score, *neg_score)
        }

        #[cfg(not(feature = "ltrgr"))]
        {
            // Fallback: use first positive and first negative (deterministic)
            let (_, pos_score) = positive_samples[0];
            let (_, neg_score) = negative_samples[0];

            self.compute_rank_loss(*pos_score, *neg_score)
        }
    }

    /// Compute total loss combining rank losses and generation loss.
    ///
    /// L = L_rank1 + L_rank2 + λ * L_gen
    pub fn compute_total_loss(
        &self,
        rank_loss_1: f32,
        rank_loss_2: f32,
        generation_loss: f32,
    ) -> f32 {
        rank_loss_1 + rank_loss_2 + self.config.lambda * generation_loss
    }

    /// Get the configuration.
    pub fn config(&self) -> &LTRGRConfig {
        &self.config
    }
}

impl Default for LTRGRTrainer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank_loss_basic() {
        let trainer = LTRGRTrainer::new();

        // Positive should have higher score than negative
        let loss = trainer.compute_rank_loss(10.0, 5.0);
        assert_eq!(loss, 495.0); // margin (500) + neg (5) - pos (10) = 495

        // If positive is much higher, loss should be 0
        let loss = trainer.compute_rank_loss(600.0, 5.0);
        assert_eq!(loss, 0.0); // margin (500) + neg (5) - pos (600) = -95, max(0, -95) = 0
    }

    #[test]
    fn test_rank_loss_1() {
        let trainer = LTRGRTrainer::new();
        let passage_scores = vec![
            (0u32, 10.0), // positive
            (1u32, 8.0),  // positive
            (2u32, 5.0),  // negative
            (3u32, 3.0),  // negative
        ];
        let positive_ids = vec![0u32, 1u32];

        let loss = trainer.compute_rank_loss_1(&passage_scores, &positive_ids);
        // Best positive: 10.0, best negative: 5.0
        // Loss: max(0, 500 + 5 - 10) = 495.0
        assert_eq!(loss, 495.0);
    }

    #[test]
    fn test_rank_loss_2() {
        let trainer = LTRGRTrainer::new();
        let passage_scores = vec![
            (0u32, 10.0), // positive
            (1u32, 8.0),  // positive
            (2u32, 5.0),  // negative
            (3u32, 3.0),  // negative
        ];
        let positive_ids = vec![0u32, 1u32];

        let loss = trainer.compute_rank_loss_2(&passage_scores, &positive_ids);
        // Should use first positive (10.0) and first negative (5.0)
        assert_eq!(loss, 495.0);
    }

    #[test]
    fn test_total_loss() {
        let trainer = LTRGRTrainer::new();
        let rank_loss_1 = 100.0;
        let rank_loss_2 = 50.0;
        let gen_loss = 2.0;

        let total = trainer.compute_total_loss(rank_loss_1, rank_loss_2, gen_loss);
        // 100 + 50 + 1000 * 2 = 2150
        assert_eq!(total, 2150.0);
    }

    #[test]
    fn test_config_custom() {
        let config = LTRGRConfig::new()
            .with_margin(100.0)
            .with_lambda(500.0)
            .with_top_k(100);

        assert_eq!(config.margin, 100.0);
        assert_eq!(config.lambda, 500.0);
        assert_eq!(config.top_k, 100);
    }
}
