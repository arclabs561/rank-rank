"""Tests for rank-learn Python bindings."""

import pytest
import rank_learn


def test_ndcg_perfect_ranking():
    """Test NDCG with perfect ranking."""
    relevance = [3.0, 2.0, 1.0]
    ndcg = rank_learn.ndcg_at_k(relevance)
    
    assert 0.0 <= ndcg <= 1.0
    assert abs(ndcg - 1.0) < 0.01  # Perfect ranking should be ~1.0


def test_ndcg_at_k():
    """Test NDCG@k with specific k."""
    relevance = [3.0, 1.0, 2.0, 0.5, 1.5]
    ndcg = rank_learn.ndcg_at_k(relevance, k=3)
    
    assert 0.0 <= ndcg <= 1.0


def test_ndcg_empty_error():
    """Test NDCG with empty relevance raises error."""
    with pytest.raises(ValueError, match="empty"):
        rank_learn.ndcg_at_k([])


def test_ndcg_invalid_k_error():
    """Test NDCG with k > length raises error."""
    relevance = [3.0, 2.0, 1.0]
    with pytest.raises(ValueError, match="Invalid NDCG"):
        rank_learn.ndcg_at_k(relevance, k=100)


def test_lambdarank_params_default():
    """Test LambdaRankParams with default values."""
    params = rank_learn.LambdaRankParams()
    assert params.sigma == 1.0


def test_lambdarank_params_custom():
    """Test LambdaRankParams with custom sigma."""
    params = rank_learn.LambdaRankParams(sigma=2.0)
    assert params.sigma == 2.0


def test_lambdarank_trainer_default():
    """Test LambdaRankTrainer with default parameters."""
    trainer = rank_learn.LambdaRankTrainer()
    assert trainer is not None


def test_lambdarank_trainer_custom_params():
    """Test LambdaRankTrainer with custom parameters."""
    params = rank_learn.LambdaRankParams(sigma=1.5)
    trainer = rank_learn.LambdaRankTrainer(params=params)
    assert trainer is not None


def test_lambdarank_compute_gradients():
    """Test LambdaRank gradient computation."""
    trainer = rank_learn.LambdaRankTrainer()
    
    # Documents with scores and relevance
    # Doc 0: score=0.5, rel=3.0 (should rank highest)
    # Doc 1: score=0.8, rel=1.0 (should rank lower)
    # Doc 2: score=0.3, rel=2.0 (should rank middle)
    scores = [0.5, 0.8, 0.3]
    relevance = [3.0, 1.0, 2.0]
    
    lambdas = trainer.compute_gradients(scores, relevance)
    
    assert len(lambdas) == len(scores)
    assert all(isinstance(l, float) for l in lambdas)
    assert all(l.is_finite() for l in lambdas)


def test_lambdarank_gradients_length_match():
    """Test that gradients match scores length."""
    trainer = rank_learn.LambdaRankTrainer()
    scores = [0.1, 0.9, 0.3, 0.7, 0.2]
    relevance = [1.0, 3.0, 2.0, 2.5, 1.5]
    
    lambdas = trainer.compute_gradients(scores, relevance)
    
    assert len(lambdas) == len(scores)


def test_lambdarank_empty_scores_error():
    """Test LambdaRank with empty scores raises error."""
    trainer = rank_learn.LambdaRankTrainer()
    
    with pytest.raises(ValueError, match="empty"):
        trainer.compute_gradients([], [1.0, 2.0])


def test_lambdarank_empty_relevance_error():
    """Test LambdaRank with empty relevance raises error."""
    trainer = rank_learn.LambdaRankTrainer()
    
    with pytest.raises(ValueError, match="empty"):
        trainer.compute_gradients([1.0, 2.0], [])


def test_lambdarank_length_mismatch_error():
    """Test LambdaRank with length mismatch raises error."""
    trainer = rank_learn.LambdaRankTrainer()
    
    with pytest.raises(ValueError, match="Length mismatch"):
        trainer.compute_gradients([1.0, 2.0], [3.0, 4.0, 5.0])


def test_lambdarank_with_k():
    """Test LambdaRank with specific k."""
    trainer = rank_learn.LambdaRankTrainer()
    scores = [0.5, 0.8, 0.3, 0.9, 0.2]
    relevance = [3.0, 1.0, 2.0, 3.0, 1.0]
    
    lambdas = trainer.compute_gradients(scores, relevance, k=3)
    
    assert len(lambdas) == len(scores)


def test_ndcg_bounds():
    """Test that NDCG is always in [0, 1]."""
    # Test with various relevance patterns
    test_cases = [
        [3.0, 2.0, 1.0],  # Perfect ranking
        [1.0, 2.0, 3.0],  # Worst ranking
        [2.0, 1.0, 3.0],  # Mixed
        [0.0, 0.0, 0.0],  # All zeros
        [5.0, 4.0, 3.0, 2.0, 1.0],  # Longer list
    ]
    
    for relevance in test_cases:
        ndcg = rank_learn.ndcg_at_k(relevance)
        assert 0.0 <= ndcg <= 1.0, f"NDCG {ndcg} not in [0, 1] for relevance {relevance}"


def test_lambdarank_gradients_finite():
    """Test that all LambdaRank gradients are finite."""
    trainer = rank_learn.LambdaRankTrainer()
    
    # Test with various score/relevance combinations
    test_cases = [
        ([0.1, 0.9, 0.3], [1.0, 3.0, 2.0]),
        ([0.5, 0.5, 0.5], [3.0, 1.0, 2.0]),
        ([1.0, 0.0, 0.5], [2.0, 3.0, 1.0]),
        ([-1.0, 0.0, 1.0], [1.0, 2.0, 3.0]),
    ]
    
    for scores, relevance in test_cases:
        lambdas = trainer.compute_gradients(scores, relevance)
        assert all(l.is_finite() for l in lambdas), \
            f"Non-finite lambda found for scores={scores}, relevance={relevance}"

