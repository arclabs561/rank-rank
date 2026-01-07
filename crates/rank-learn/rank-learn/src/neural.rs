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

/// Neural LTR model (simplified interface).
///
/// In a full implementation, this would contain:
/// - Embedding layers
/// - Feed-forward networks
/// - Output scoring layer
///
/// For now, this provides the interface and uses rank-soft for ranking operations.
pub struct NeuralLTRModel {
    config: NeuralLTRConfig,
}

impl NeuralLTRModel {
    /// Create a new neural LTR model.
    pub fn new(config: NeuralLTRConfig) -> Self {
        Self { config }
    }
    
    /// Score documents (placeholder - would use actual neural network).
    ///
    /// In a full implementation, this would:
    /// 1. Embed query and documents
    /// 2. Pass through neural network
    /// 3. Return scores
    pub fn score(&self, _query: &[f32], _documents: &[Vec<f32>]) -> Vec<f32> {
        // Placeholder: return random scores
        // In real implementation, would use neural network
        vec![0.5; _documents.len()]
    }
    
    /// Compute soft ranks for scores using rank-soft.
    pub fn soft_rank_scores(&self, scores: &[f32]) -> Vec<f64> {
        // Convert f32 to f64 for rank-soft
        let scores_f64: Vec<f64> = scores.iter().map(|&x| x as f64).collect();
        self.config.ranking_method.compute(&scores_f64, self.config.regularization_strength as f64)
    }
    
    /// Compute loss using rank-soft's Spearman loss.
    pub fn compute_loss(&self, predictions: &[f32], targets: &[f32]) -> f64 {
        // Convert f32 to f64 for rank-soft
        let pred_f64: Vec<f64> = predictions.iter().map(|&x| x as f64).collect();
        let targ_f64: Vec<f64> = targets.iter().map(|&x| x as f64).collect();
        spearman_loss(&pred_f64, &targ_f64, self.config.regularization_strength as f64)
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
}

