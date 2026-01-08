//! Neural Learning to Rank models.
//!
//! Neural ranking models that use rank-soft for differentiable operations.
//!
//! This module provides neural LTR architectures that leverage differentiable
//! ranking operations from rank-soft for end-to-end training.

use rank_soft::{spearman_loss, RankingMethod};

/// Neural ranking model configuration.
#[derive(Debug, Clone)]
pub struct NeuralLTRConfig {
    /// Embedding dimension
    pub embedding_dim: usize,

    /// Hidden layer dimensions
    pub hidden_dims: Vec<usize>,

    /// Regularization strength for soft ranking
    pub regularization_strength: f32,

    /// Ranking method to use
    pub ranking_method: RankingMethod,
}

impl Default for NeuralLTRConfig {
    fn default() -> Self {
        Self {
            embedding_dim: 128,
            hidden_dims: vec![64, 32],
            regularization_strength: 1.0,
            ranking_method: RankingMethod::Sigmoid,
        }
    }
}

/// Neural LTR model with feed-forward network.
///
/// Implements a simple multi-layer perceptron for scoring query-document pairs.
/// Uses rank-soft for differentiable ranking operations during training.
pub struct NeuralLTRModel {
    config: NeuralLTRConfig,
    // Simple feed-forward network weights (for demonstration)
    // In production, use a proper neural network library (candle, burn, etc.)
    weights: Vec<Vec<Vec<f32>>>,
}

impl NeuralLTRModel {
    /// Create a new neural LTR model with random initialization.
    pub fn new(config: NeuralLTRConfig) -> Self {
        let weights = Self::initialize_weights(&config);
        Self { config, weights }
    }

    /// Initialize network weights using Xavier initialization.
    fn initialize_weights(config: &NeuralLTRConfig) -> Vec<Vec<Vec<f32>>> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut weights = Vec::new();
        let mut dims = vec![config.embedding_dim * 2]; // Query + document concatenated
        dims.extend_from_slice(&config.hidden_dims);
        dims.push(1); // Output layer

        for i in 0..dims.len() - 1 {
            let mut layer = Vec::new();
            for j in 0..dims[i + 1] {
                let mut neuron = Vec::new();
                for k in 0..dims[i] + 1 {
                    // +1 for bias
                    // Simple deterministic initialization based on indices
                    let mut hasher = DefaultHasher::new();
                    (i, j, k).hash(&mut hasher);
                    let hash = hasher.finish();
                    // Xavier initialization: uniform in [-sqrt(6/(fan_in+fan_out)), sqrt(6/(fan_in+fan_out))]
                    let fan_in = dims[i] as f32;
                    let fan_out = dims[i + 1] as f32;
                    let limit = (6.0 / (fan_in + fan_out)).sqrt();
                    let weight = ((hash % 10000) as f32 / 5000.0 - 1.0) * limit;
                    neuron.push(weight);
                }
                layer.push(neuron);
            }
            weights.push(layer);
        }

        weights
    }

    /// Score documents using feed-forward network.
    ///
    /// Concatenates query and document embeddings, passes through network.
    pub fn score(&self, query: &[f32], documents: &[Vec<f32>]) -> Vec<f32> {
        if query.len() != self.config.embedding_dim {
            return vec![0.0; documents.len()];
        }

        documents
            .iter()
            .map(|doc| {
                if doc.len() != self.config.embedding_dim {
                    return 0.0;
                }

                // Concatenate query and document
                let mut input = Vec::with_capacity(self.config.embedding_dim * 2);
                input.extend_from_slice(query);
                input.extend_from_slice(doc);

                // Forward pass through network
                self.forward(&input)
            })
            .collect()
    }

    /// Forward pass through the network.
    fn forward(&self, input: &[f32]) -> f32 {
        let mut activations = input.to_vec();

        // Add bias term
        activations.push(1.0);

        for layer in &self.weights {
            let mut next_activations = Vec::new();

            for neuron in layer {
                // Compute dot product + bias
                let sum: f32 = activations
                    .iter()
                    .zip(neuron.iter())
                    .map(|(a, w)| a * w)
                    .sum();

                // ReLU activation (except last layer uses identity)
                let activation = if layer == self.weights.last().unwrap() {
                    sum // Identity for output layer
                } else {
                    sum.max(0.0) // ReLU
                };

                next_activations.push(activation);
            }

            // Add bias for next layer
            next_activations.push(1.0);
            activations = next_activations;
        }

        // Remove bias term and return output
        activations[0]
    }

    /// Compute soft ranks for scores using rank-soft.
    pub fn soft_rank_scores(&self, scores: &[f32]) -> Vec<f64> {
        // Convert f32 to f64 for rank-soft
        let scores_f64: Vec<f64> = scores.iter().map(|&x| x as f64).collect();
        self.config
            .ranking_method
            .compute(&scores_f64, self.config.regularization_strength as f64)
    }

    /// Compute loss using rank-soft's Spearman loss.
    pub fn compute_loss(&self, predictions: &[f32], targets: &[f32]) -> f64 {
        // Convert f32 to f64 for rank-soft
        let pred_f64: Vec<f64> = predictions.iter().map(|&x| x as f64).collect();
        let targ_f64: Vec<f64> = targets.iter().map(|&x| x as f64).collect();
        spearman_loss(
            &pred_f64,
            &targ_f64,
            self.config.regularization_strength as f64,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neural_ltr() {
        let config = NeuralLTRConfig::default();
        let model = NeuralLTRModel::new(config);

        let scores = vec![0.1, 0.9, 0.3];
        let ranks = model.soft_rank_scores(&scores);

        assert_eq!(ranks.len(), 3);
        // Highest score should have highest rank
        assert!(ranks[1] > ranks[0] as f64);
        assert!(ranks[1] > ranks[2] as f64);
    }

    #[test]
    fn test_neural_ltr_initialization() {
        let config = NeuralLTRConfig {
            embedding_dim: 64,
            hidden_dims: vec![32, 16],
            regularization_strength: 1.0,
            ranking_method: RankingMethod::Sigmoid,
        };

        let model = NeuralLTRModel::new(config.clone());

        assert_eq!(model.config.embedding_dim, 64);
        assert_eq!(model.config.hidden_dims, vec![32, 16]);
        assert!(!model.weights.is_empty());
    }

    #[test]
    fn test_score_dimension_mismatch() {
        let config = NeuralLTRConfig::default();
        let model = NeuralLTRModel::new(config);

        let query = vec![1.0; 128];
        let wrong_query = vec![1.0; 32];

        let documents = vec![vec![0.5; 128], vec![0.7; 128]];

        let scores = model.score(&query, &documents);
        assert_eq!(scores.len(), 2);

        let scores_wrong = model.score(&wrong_query, &documents);
        assert_eq!(scores_wrong, vec![0.0, 0.0]);
    }

    #[test]
    fn test_score_consistency() {
        let config = NeuralLTRConfig::default();
        let model = NeuralLTRModel::new(config);

        let query = vec![1.0; 128];
        let documents = vec![vec![0.9; 128], vec![0.1; 128]];

        let scores = model.score(&query, &documents);
        assert_ne!(scores[0], scores[1]);
        assert!(scores[0].is_finite());
        assert!(scores[1].is_finite());
    }

    #[test]
    fn test_compute_loss() {
        let config = NeuralLTRConfig::default();
        let model = NeuralLTRModel::new(config);

        let predictions = vec![0.1, 0.9, 0.3];
        let targets = vec![0.2, 0.8, 0.4];

        let loss = model.compute_loss(&predictions, &targets);
        assert!(loss >= 0.0);
        assert!(loss.is_finite());
    }
}
