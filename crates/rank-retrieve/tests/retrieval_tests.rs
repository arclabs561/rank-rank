//! Retrieval functionality tests for rank-retrieve.
//!
//! Tests all retrieval methods (BM25, dense, sparse), batch operations,
//! error handling, and real-world scenarios. These tests use rank-retrieve
//! in isolation without external crates.
//!
//! For tests that integrate with other rank-* crates, see:
//! - `e2e_fusion_eval.rs` - Integration with rank-fusion and rank-eval
//! - `e2e_full_pipeline.rs` - Full pipeline with all crates

use rank_retrieve::batch;
#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "dense")]
use rank_retrieve::dense::DenseRetriever;
#[cfg(feature = "sparse")]
use rank_retrieve::sparse::{SparseRetriever, SparseVector};
use rank_retrieve::RetrieveError;

// Basic workflows (from integration.rs)

#[test]
fn bm25_retrieval_workflow() {
    let mut index = InvertedIndex::new();

    index.add_document(
        0,
        &[
            "machine".to_string(),
            "learning".to_string(),
            "algorithms".to_string(),
        ],
    );
    index.add_document(
        1,
        &[
            "deep".to_string(),
            "learning".to_string(),
            "neural".to_string(),
            "networks".to_string(),
        ],
    );
    index.add_document(
        2,
        &[
            "information".to_string(),
            "retrieval".to_string(),
            "search".to_string(),
        ],
    );

    let query = vec!["learning".to_string()];
    let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    assert_eq!(results.len(), 2);
    assert!(results[0].0 == 0 || results[0].0 == 1);
    assert!(results[1].0 == 0 || results[1].0 == 1);
    assert!(results[0].1 > 0.0);
}

#[test]
fn dense_retrieval_workflow() {
    let mut retriever = DenseRetriever::new();

    retriever.add_document(0, vec![0.8, 0.6, 0.0]);
    retriever.add_document(1, vec![0.0, 0.6, 0.8]);
    retriever.add_document(2, vec![0.6, 0.8, 0.0]);

    let query = vec![0.8, 0.6, 0.0];
    let results = retriever.retrieve(&query, 10).unwrap();

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].0, 0);
    assert!(results[0].1 > results[1].1);
}

#[test]
fn sparse_retrieval_workflow() {
    let mut retriever = SparseRetriever::new();

    let doc0 = SparseVector::new(vec![0, 1, 2], vec![1.0, 0.8, 0.6]).unwrap();
    let doc1 = SparseVector::new(vec![2, 3, 4], vec![0.6, 0.8, 1.0]).unwrap();
    let doc2 = SparseVector::new(vec![0, 2, 4], vec![0.5, 0.7, 0.9]).unwrap();

    retriever.add_document(0, doc0);
    retriever.add_document(1, doc1);
    retriever.add_document(2, doc2);

    let query = SparseVector::new(vec![0, 1, 2], vec![1.0, 1.0, 1.0]).unwrap();
    let results = retriever.retrieve(&query, 10).unwrap();

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].0, 0);
    assert!(results[0].1 > results[1].1);
}

#[test]
fn error_handling_empty_query() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string()]);

    let result = index.retrieve(&[], 10, Bm25Params::default());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RetrieveError::EmptyQuery));
}

#[test]
fn error_handling_empty_index() {
    let index = InvertedIndex::new();
    let result = index.retrieve(&["test".to_string()], 10, Bm25Params::default());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RetrieveError::EmptyIndex));
}

#[test]
fn error_handling_dimension_mismatch() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]);

    let result = retriever.retrieve(&[1.0, 0.0, 0.0], 10);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        RetrieveError::DimensionMismatch { .. }
    ));
}

// Retrieval methods and batch operations (from integration_tests.rs)

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_basic_retrieval() {
    let mut index = InvertedIndex::new();

    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["artificial".to_string(), "intelligence".to_string()]);
    index.add_document(2, &["machine".to_string(), "vision".to_string()]);

    let query = vec!["machine".to_string()];
    let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    assert!(!results.is_empty());
    // Both doc 0 and doc 2 contain "machine", so either could rank first
    assert!(results[0].0 == 0 || results[0].0 == 2);
    assert!(results.iter().any(|(id, _)| *id == 0));
    assert!(results.iter().any(|(id, _)| *id == 2));
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_empty_query() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string()]);

    let query = vec![];
    let result = index.retrieve(&query, 10, Bm25Params::default());
    assert!(result.is_err());
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_empty_index() {
    let index = InvertedIndex::new();
    let query = vec!["test".to_string()];
    let result = index.retrieve(&query, 10, Bm25Params::default());
    assert!(result.is_err());
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_retrieval() {
    let mut retriever = DenseRetriever::new();

    let mut vec1 = vec![1.0, 0.0];
    let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
    vec1.iter_mut().for_each(|x| *x /= norm1);

    let mut vec2 = vec![0.0, 1.0];
    let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();
    vec2.iter_mut().for_each(|x| *x /= norm2);

    retriever.add_document(0, vec1);
    retriever.add_document(1, vec2);

    let mut query = vec![0.9, 0.1];
    let norm_q: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
    query.iter_mut().for_each(|x| *x /= norm_q);

    let results = retriever.retrieve(&query, 10).unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0, 0);
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_dimension_mismatch() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]);

    let query = vec![1.0, 0.0, 0.0];
    let result = retriever.retrieve(&query, 10);
    assert!(result.is_err());
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_retrieval() {
    let mut retriever = SparseRetriever::new();

    let doc0 = SparseVector::new_unchecked(vec![0, 1, 2], vec![1.0, 0.5, 0.3]);
    let doc1 = SparseVector::new_unchecked(vec![1, 2, 3], vec![0.8, 0.6, 0.4]);

    retriever.add_document(0, doc0);
    retriever.add_document(1, doc1);

    let query = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 1.0]);
    let results = retriever.retrieve(&query, 10).unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0, 0);
}

#[cfg(feature = "bm25")]
#[test]
fn test_batch_retrieve_bm25() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["artificial".to_string(), "intelligence".to_string()]);

    let queries = vec![vec!["machine".to_string()], vec!["artificial".to_string()]];

    let results = batch::batch_retrieve_bm25(&index, &queries, 10, Bm25Params::default()).unwrap();
    assert_eq!(results.len(), 2);
    assert!(!results[0].is_empty());
    assert!(!results[1].is_empty());
}

#[cfg(feature = "dense")]
#[test]
fn test_batch_retrieve_dense() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]);
    retriever.add_document(1, vec![0.0, 1.0]);

    let queries = vec![vec![1.0, 0.0], vec![0.0, 1.0]];

    let results = batch::batch_retrieve_dense(&retriever, &queries, 10).unwrap();
    assert_eq!(results.len(), 2);
    assert!(!results[0].is_empty());
    assert!(!results[1].is_empty());
}

#[cfg(feature = "sparse")]
#[test]
fn test_batch_retrieve_sparse() {
    let mut retriever = SparseRetriever::new();
    let doc0 = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 0.5]);
    retriever.add_document(0, doc0);

    let queries = vec![
        SparseVector::new_unchecked(vec![0], vec![1.0]),
        SparseVector::new_unchecked(vec![1], vec![1.0]),
    ];

    let results = batch::batch_retrieve_sparse(&retriever, &queries, 10).unwrap();
    assert_eq!(results.len(), 2);
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_params_customization() {
    let mut index = InvertedIndex::new();
    index.add_document(
        0,
        &["test".to_string(), "test".to_string(), "test".to_string()],
    );

    let query = vec!["test".to_string()];

    let results_default = index.retrieve(&query, 10, Bm25Params::default()).unwrap();
    let custom_params = Bm25Params { k1: 2.0, b: 0.5 };
    let results_custom = index.retrieve(&query, 10, custom_params).unwrap();

    assert!(!results_default.is_empty());
    assert!(!results_custom.is_empty());
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_vector_validation() {
    let valid = SparseVector::new(vec![0, 1, 2], vec![1.0, 0.5, 0.3]);
    assert!(valid.is_some());

    let invalid = SparseVector::new(vec![0, 1], vec![1.0, 0.5, 0.3]);
    assert!(invalid.is_none());

    let invalid2 = SparseVector::new(vec![1, 0, 2], vec![1.0, 0.5, 0.3]);
    assert!(invalid2.is_none());

    let invalid3 = SparseVector::new(vec![0, 0, 1], vec![1.0, 0.5, 0.3]);
    assert!(invalid3.is_none());
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_vector_prune() {
    let v = SparseVector::new(vec![0, 1, 2, 3], vec![0.1, 0.9, 0.2, 0.8]).unwrap();
    let pruned = v.prune(0.5);

    assert_eq!(pruned.indices, vec![1, 3]);
    assert_eq!(pruned.values, vec![0.9, 0.8]);
}

#[cfg(feature = "bm25")]
#[test]
fn test_retrieval_top_k_limiting() {
    let mut index = InvertedIndex::new();
    for i in 0..20 {
        index.add_document(i, &["test".to_string()]);
    }

    let query = vec!["test".to_string()];
    let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    assert_eq!(results.len(), 10);
}

#[cfg(feature = "bm25")]
#[test]
fn test_retrieval_returns_fewer_than_k() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["unique".to_string()]);

    let query = vec!["unique".to_string()];
    let results = index.retrieve(&query, 100, Bm25Params::default()).unwrap();

    assert_eq!(results.len(), 1);
}

// Real-world scenarios (from comprehensive_integration.rs)

#[test]
fn test_hybrid_retrieval_workflow() {
    let mut bm25_index = InvertedIndex::new();
    let documents = vec![
        vec!["machine", "learning", "tutorial", "python"],
        vec!["deep", "learning", "neural", "networks", "tensorflow"],
        vec!["python", "programming", "guide", "beginners"],
        vec!["rust", "systems", "programming", "performance"],
    ];
    for (i, doc) in documents.iter().enumerate() {
        bm25_index.add_document(
            i as u32,
            &doc.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        );
    }

    let mut dense_retriever = DenseRetriever::new();
    let embeddings = vec![
        vec![1.0, 0.0, 0.0, 0.0],
        vec![0.707, 0.707, 0.0, 0.0],
        vec![0.0, 0.0, 1.0, 0.0],
        vec![0.0, 0.0, 0.0, 1.0],
    ];
    for (i, emb) in embeddings.iter().enumerate() {
        dense_retriever.add_document(i as u32, emb.clone());
    }

    let query_terms: Vec<String> = vec!["machine", "learning"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let query_embedding = vec![1.0, 0.0, 0.0, 0.0];

    let bm25_results = bm25_index
        .retrieve(&query_terms, 10, Bm25Params::default())
        .unwrap();
    let dense_results = dense_retriever.retrieve(&query_embedding, 10).unwrap();

    assert!(!bm25_results.is_empty());
    assert!(!dense_results.is_empty());
    assert_eq!(bm25_results[0].0, 0);
    assert!(dense_results[0].0 == 0 || dense_results[0].0 == 1);
}

#[test]
fn test_large_scale_retrieval() {
    let mut index = InvertedIndex::new();
    let vocab_size = 1000;

    for doc_id in 0..1000 {
        let terms: Vec<String> = (0..50)
            .map(|i| format!("term{}", (doc_id * 7 + i * 11) % vocab_size))
            .collect();
        index.add_document(doc_id, &terms);
    }

    let query: Vec<String> = (0..10).map(|i| format!("term{}", i * 100)).collect();

    let results = index.retrieve(&query, 100, Bm25Params::default()).unwrap();

    assert!(results.len() <= 100);
    assert!(!results.is_empty());

    for i in 1..results.len() {
        assert!(results[i - 1].1 >= results[i].1);
    }
}

#[test]
fn test_sparse_dense_consistency() {
    let mut dense = DenseRetriever::new();
    let mut sparse = SparseRetriever::new();

    dense.add_document(0, vec![1.0, 0.0, 0.0]);
    let sparse_vec = SparseVector::new(vec![0], vec![1.0]).unwrap();
    sparse.add_document(0, sparse_vec);

    let dense_score = dense.score(0, &[1.0, 0.0, 0.0]);
    let sparse_query = SparseVector::new(vec![0], vec![1.0]).unwrap();
    let sparse_score = sparse.score(0, &sparse_query);

    assert!(dense_score.is_some());
    assert!(sparse_score.is_some());
    assert!(dense_score.unwrap().is_finite());
    assert!(sparse_score.unwrap().is_finite());
}

#[test]
fn test_retrieval_with_varying_k() {
    let mut index = InvertedIndex::new();
    for i in 0..50 {
        index.add_document(i, &[format!("term{}", i)]);
    }

    let query = vec!["term0".to_string(), "term1".to_string()];

    for k in [1, 5, 10, 20, 50, 100] {
        let results = index.retrieve(&query, k, Bm25Params::default()).unwrap();
        assert!(results.len() <= k);
        assert!(results.len() <= 50);
    }
}

#[test]
fn test_bm25_parameter_sensitivity() {
    let mut index = InvertedIndex::new();
    index.add_document(
        0,
        &["test", "test", "test"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );
    index.add_document(
        1,
        &["test"].iter().map(|s| s.to_string()).collect::<Vec<_>>(),
    );

    let query = vec!["test".to_string()];

    let default_params = Bm25Params::default();
    let default_results = index.retrieve(&query, 10, default_params).unwrap();

    let high_k1 = Bm25Params { k1: 10.0, b: 0.75 };
    let high_k1_results = index.retrieve(&query, 10, high_k1).unwrap();

    let default_score0 = default_results
        .iter()
        .find(|(id, _)| *id == 0)
        .map(|(_, s)| *s);
    let high_k1_score0 = high_k1_results
        .iter()
        .find(|(id, _)| *id == 0)
        .map(|(_, s)| *s);

    if let (Some(s0), Some(s1)) = (default_score0, high_k1_score0) {
        assert!(s1 >= s0 || (s1 - s0).abs() < 0.001);
    }
}

#[test]
fn test_concurrent_retrieval() {
    use std::sync::Arc;
    use std::thread;

    let index = Arc::new({
        let mut idx = InvertedIndex::new();
        for i in 0..100 {
            idx.add_document(i, &[format!("term{}", i)]);
        }
        idx
    });

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let idx = Arc::clone(&index);
            thread::spawn(move || {
                let query = vec!["term0".to_string()];
                idx.retrieve(&query, 10, Bm25Params::default())
            })
        })
        .collect();

    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok());
        let results = result.unwrap();
        assert!(!results.is_empty());
    }
}

#[test]
fn test_empty_results_handling() {
    let mut index = InvertedIndex::new();
    index.add_document(
        0,
        &["apple", "banana"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );
    index.add_document(
        1,
        &["cherry", "date"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    let query = vec!["zucchini".to_string()];
    let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    assert!(results.is_empty());
}

#[test]
fn test_partial_match_retrieval() {
    let mut index = InvertedIndex::new();
    index.add_document(
        0,
        &["machine", "learning"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );
    index.add_document(
        1,
        &["machine"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );
    index.add_document(
        2,
        &["learning"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    let query = vec!["machine".to_string(), "learning".to_string()];
    let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    assert_eq!(results[0].0, 0);

    let doc_ids: Vec<u32> = results.iter().map(|(id, _)| *id).collect();
    assert!(doc_ids.contains(&1) || doc_ids.contains(&2));
}

#[test]
fn test_retrieval_score_stability() {
    let mut index = InvertedIndex::new();
    index.add_document(
        0,
        &["test", "document"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    );

    let query = vec!["test".to_string()];

    let results1 = index.retrieve(&query, 10, Bm25Params::default()).unwrap();
    let results2 = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    assert_eq!(results1.len(), results2.len());
    for ((id1, s1), (id2, s2)) in results1.iter().zip(results2.iter()) {
        assert_eq!(id1, id2);
        assert!((s1 - s2).abs() < 1e-6);
    }
}

// Format validation (from e2e_real_integration.rs - useful tests only)

#[test]
fn test_output_format_compatibility() {
    // Verify output format is compatible with rank-fusion (u32 IDs work directly)
    let mut bm25_index = InvertedIndex::new();
    bm25_index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    bm25_index.add_document(1, &["deep".to_string(), "learning".to_string()]);

    let mut dense_retriever = DenseRetriever::new();
    dense_retriever.add_document(0, vec![1.0, 0.0, 0.0]);
    dense_retriever.add_document(1, vec![0.707, 0.707, 0.0]);

    let query_terms = vec!["learning".to_string()];
    let query_emb = [1.0, 0.0, 0.0];

    let bm25_results = bm25_index
        .retrieve(&query_terms, 10, Bm25Params::default())
        .unwrap();
    let dense_results = dense_retriever.retrieve(&query_emb, 10).unwrap();

    // Verify format: Vec<(u32, f32)>
    assert!(bm25_results
        .iter()
        .all(|(id, score)| { *id < 10 && score.is_finite() && *score >= 0.0 }));
    assert!(dense_results
        .iter()
        .all(|(id, score)| { *id < 10 && score.is_finite() && *score >= 0.0 }));

    // Verify sorted descending
    for i in 1..bm25_results.len() {
        assert!(bm25_results[i - 1].1 >= bm25_results[i].1);
    }
    for i in 1..dense_results.len() {
        assert!(dense_results[i - 1].1 >= dense_results[i].1);
    }
}
