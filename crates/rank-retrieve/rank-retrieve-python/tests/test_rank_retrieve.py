"""Tests for rank-retrieve Python bindings."""

import pytest
import rank_retrieve


def test_bm25_basic():
    """Test basic BM25 retrieval."""
    index = rank_retrieve.InvertedIndex()
    
    # Add documents
    index.add_document(0, ["the", "quick", "brown", "fox"])
    index.add_document(1, ["the", "lazy", "dog"])
    index.add_document(2, ["quick", "brown", "fox", "jumps"])
    
    # Query
    results = index.retrieve(["quick", "fox"], 10)
    
    # Should return results
    assert len(results) > 0
    # Results should be (doc_id, score) tuples
    assert all(isinstance(r, tuple) and len(r) == 2 for r in results)
    # Scores should be positive
    assert all(score > 0.0 for _, score in results)


def test_bm25_with_params():
    """Test BM25 with custom parameters."""
    index = rank_retrieve.InvertedIndex()
    index.add_document(0, ["test", "document"])
    
    params = rank_retrieve.Bm25Params(k1=1.5, b=0.8)
    results = index.retrieve(["test"], 10, params)
    
    assert len(results) > 0


def test_bm25_idf():
    """Test IDF calculation."""
    index = rank_retrieve.InvertedIndex()
    index.add_document(0, ["common", "term"])
    index.add_document(1, ["common", "word"])
    index.add_document(2, ["rare", "term"])
    
    # Rare term should have higher IDF
    idf_common = index.idf("common")
    idf_rare = index.idf("rare")
    
    assert idf_rare > idf_common


def test_dense_retrieval():
    """Test dense retrieval."""
    retriever = rank_retrieve.DenseRetriever()
    
    # Add documents with normalized embeddings
    retriever.add_document(0, [1.0, 0.0, 0.0])
    retriever.add_document(1, [0.707, 0.707, 0.0])
    
    # Query
    query = [1.0, 0.0, 0.0]
    results = retriever.retrieve(query, 10)
    
    # Document 0 should score highest (exact match)
    assert len(results) == 2
    assert results[0][0] == 0  # doc 0 should be first
    assert results[0][1] > results[1][1]  # higher score


def test_dense_score():
    """Test dense scoring."""
    retriever = rank_retrieve.DenseRetriever()
    retriever.add_document(0, [1.0, 0.0])
    
    score = retriever.score(0, [1.0, 0.0])
    assert score is not None
    assert abs(score - 1.0) < 0.001  # cosine similarity of identical vectors
    
    # Non-existent document
    score = retriever.score(999, [1.0, 0.0])
    assert score is None


def test_sparse_vector():
    """Test sparse vector creation and operations."""
    # Create with validation
    vector = rank_retrieve.SparseVector([0, 1, 2], [1.0, 0.5, 0.3], validate=True)
    
    assert vector.indices == [0, 1, 2]
    assert vector.values == [1.0, 0.5, 0.3]
    
    # Prune
    pruned = vector.prune(0.4)
    assert len(pruned.indices) == 1  # Only 1.0 > 0.4
    assert pruned.indices[0] == 0


def test_sparse_vector_invalid():
    """Test sparse vector validation."""
    # Mismatched lengths
    with pytest.raises(ValueError):
        rank_retrieve.SparseVector([0, 1], [1.0], validate=True)
    
    # Unsorted indices
    with pytest.raises(ValueError):
        rank_retrieve.SparseVector([1, 0], [1.0, 0.5], validate=True)


def test_sparse_retrieval():
    """Test sparse retrieval."""
    retriever = rank_retrieve.SparseRetriever()
    
    # Document 0: terms 0, 1, 2
    doc0 = rank_retrieve.SparseVector([0, 1, 2], [1.0, 0.5, 0.3], validate=False)
    retriever.add_document(0, doc0)
    
    # Document 1: terms 1, 2, 3
    doc1 = rank_retrieve.SparseVector([1, 2, 3], [0.8, 0.6, 0.4], validate=False)
    retriever.add_document(1, doc1)
    
    # Query: terms 0, 1
    query = rank_retrieve.SparseVector([0, 1], [1.0, 1.0], validate=False)
    results = retriever.retrieve(query, 10)
    
    # Document 0 should score higher (has term 0)
    assert len(results) == 2
    assert results[0][0] == 0  # doc 0 should be first


def test_sparse_dot_product():
    """Test sparse vector dot product."""
    v1 = rank_retrieve.SparseVector([1, 3, 5], [1.0, 2.0, 3.0], validate=False)
    v2 = rank_retrieve.SparseVector([1, 4, 5], [0.5, 2.0, 0.5], validate=False)
    
    # Match at 1 (1.0 * 0.5 = 0.5) and 5 (3.0 * 0.5 = 1.5)
    result = rank_retrieve.sparse_dot_product(v1, v2)
    assert abs(result - 2.0) < 0.001  # 0.5 + 1.5 = 2.0


def test_empty_query_error():
    """Test error handling for empty query."""
    index = rank_retrieve.InvertedIndex()
    index.add_document(0, ["test"])
    
    with pytest.raises(ValueError, match="empty"):
        index.retrieve([], 10)


def test_empty_index_error():
    """Test error handling for empty index."""
    index = rank_retrieve.InvertedIndex()
    
    with pytest.raises(ValueError, match="empty"):
        index.retrieve(["test"], 10)


def test_dimension_mismatch_error():
    """Test error handling for dimension mismatch."""
    retriever = rank_retrieve.DenseRetriever()
    retriever.add_document(0, [1.0, 0.0, 0.0])
    
    # Query with wrong dimension
    with pytest.raises(ValueError, match="dimension"):
        retriever.retrieve([1.0, 0.0], 10)  # 2D vs 3D


def test_bm25_multiple_documents():
    """Test BM25 with multiple documents."""
    index = rank_retrieve.InvertedIndex()
    index.add_document(0, ["the", "quick", "brown", "fox"])
    index.add_document(1, ["the", "lazy", "dog"])
    index.add_document(2, ["quick", "brown", "fox", "jumps"])
    index.add_document(3, ["the", "quick", "fox"])
    
    results = index.retrieve(["quick", "fox"], 10)
    
    # Should return multiple documents
    assert len(results) >= 2
    # Results should be sorted by score descending
    for i in range(len(results) - 1):
        assert results[i][1] >= results[i + 1][1]


def test_dense_multiple_documents():
    """Test dense retrieval with multiple documents."""
    retriever = rank_retrieve.DenseRetriever()
    retriever.add_document(0, [1.0, 0.0, 0.0])
    retriever.add_document(1, [0.707, 0.707, 0.0])
    retriever.add_document(2, [0.0, 1.0, 0.0])
    
    query = [1.0, 0.0, 0.0]
    results = retriever.retrieve(query, 10)
    
    assert len(results) == 3
    # Document 0 should score highest (exact match)
    assert results[0][0] == 0
    # Scores should be sorted descending
    for i in range(len(results) - 1):
        assert results[i][1] >= results[i + 1][1]


def test_sparse_multiple_documents():
    """Test sparse retrieval with multiple documents."""
    retriever = rank_retrieve.SparseRetriever()
    
    # Document 0: terms 0, 1, 2
    doc0 = rank_retrieve.SparseVector([0, 1, 2], [1.0, 0.5, 0.3], validate=False)
    retriever.add_document(0, doc0)
    
    # Document 1: terms 1, 2, 3
    doc1 = rank_retrieve.SparseVector([1, 2, 3], [0.8, 0.6, 0.4], validate=False)
    retriever.add_document(1, doc1)
    
    # Document 2: terms 0, 3
    doc2 = rank_retrieve.SparseVector([0, 3], [0.9, 0.7], validate=False)
    retriever.add_document(2, doc2)
    
    # Query: terms 0, 1
    query = rank_retrieve.SparseVector([0, 1], [1.0, 1.0], validate=False)
    results = retriever.retrieve(query, 10)
    
    assert len(results) == 3
    # Document 0 should score highest (has both terms 0 and 1)
    assert results[0][0] == 0
    # Scores should be sorted descending
    for i in range(len(results) - 1):
        assert results[i][1] >= results[i + 1][1]


def test_bm25_idf_consistency():
    """Test that IDF values are consistent."""
    index = rank_retrieve.InvertedIndex()
    index.add_document(0, ["common", "term"])
    index.add_document(1, ["common", "word"])
    index.add_document(2, ["rare", "term"])
    
    # Common term should have lower IDF
    idf_common = index.idf("common")
    idf_rare = index.idf("rare")
    
    assert idf_rare > idf_common
    
    # IDF should be non-negative
    assert idf_common >= 0.0
    assert idf_rare >= 0.0


def test_dense_cosine_similarity_bounds():
    """Test that cosine similarity scores are in valid range."""
    retriever = rank_retrieve.DenseRetriever()
    retriever.add_document(0, [1.0, 0.0, 0.0])
    retriever.add_document(1, [-1.0, 0.0, 0.0])  # Opposite direction
    retriever.add_document(2, [0.0, 1.0, 0.0])   # Perpendicular
    
    query = [1.0, 0.0, 0.0]
    results = retriever.retrieve(query, 10)
    
    for _, score in results:
        assert -1.0 <= score <= 1.0, f"Cosine similarity {score} not in [-1, 1]"

