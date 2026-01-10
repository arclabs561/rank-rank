//! Property-based tests for numerical stability.
//!
//! Verifies handling of extreme values, subnormal numbers, and edge cases.

#[cfg(feature = "dense")]
use proptest::prelude::*;

#[cfg(feature = "dense")]
mod dense_tests {
    use super::*;
    use rank_retrieve::dense::DenseRetriever;

    proptest! {
        #[test]
        fn test_extreme_value_handling(
            values in prop::collection::vec(
                (-1e10f32..1e10f32).prop_filter("finite", |&x| x.is_finite() && x.abs() > 1e-10),
                10..100
            )
        ) {
            // Property: Operations should handle extreme values without overflow/underflow
            // Constrain to reasonable range to avoid overflow in cosine similarity
            let mut retriever = DenseRetriever::new();
            retriever.add_document(0, values.clone());
            
            // Use a normalized query to avoid extreme dot products
            let query: Vec<f32> = values.iter().map(|&x| x.signum() * x.abs().min(1.0)).collect();
            let results = retriever.retrieve(&query, 10);
            
            // Should not panic
            prop_assert!(results.is_ok(), "Should handle extreme values without error");
            
            if let Ok(results) = results {
                // All scores should be finite
                for (doc_id, score) in &results {
                    prop_assert!(
                        score.is_finite(),
                        "Score should be finite for doc {}: {}",
                        doc_id,
                        score
                    );
                }
            }
        }

        #[test]
        fn test_very_large_vectors(
            dim in 100usize..1000
        ) {
            // Property: Operations should work with very large vectors
            let a: Vec<f32> = (0..dim).map(|i| (i as f32) * 0.001).collect();
            let b: Vec<f32> = (0..dim).map(|i| (i as f32) * 0.002).collect();
            
            // Use DenseRetriever which uses simd internally
            let mut retriever = DenseRetriever::new();
            retriever.add_document(0, a.clone());
            let result = retriever.retrieve(&b, 1);
            prop_assert!(
                result.is_ok(),
                "Retrieval should work with large vectors: dim={}",
                dim
            );
        }
    }

    #[test]
    fn test_subnormal_handling() {
        // Property: Subnormal numbers should be handled correctly
        // Values near f32::MIN_POSITIVE (~1.175e-38)
        let subnormal_values = vec![
            f32::MIN_POSITIVE,
            f32::MIN_POSITIVE * 2.0,
            f32::MIN_POSITIVE * 0.5,
        ];
        
        let normal_values = vec![1.0, 2.0, 3.0];
        
        // Test that operations work with subnormal values
        // Use DenseRetriever's cosine_similarity which uses simd internally
        let mut retriever = DenseRetriever::new();
        retriever.add_document(0, subnormal_values.clone());
        let result = retriever.retrieve(&normal_values, 1);
        assert!(
            result.is_ok(),
            "Retrieval should handle subnormal values"
        );
    }
}

#[cfg(feature = "sparse")]
mod sparse_tests {
    use super::*;
    use rank_retrieve::sparse::SparseVector;

    proptest! {
        #[test]
        fn test_sparse_extreme_values(
            indices in prop::collection::vec(0u32..1000, 10..100),
            values in prop::collection::vec(
                (-1e10f32..1e10f32).prop_filter("finite", |&x| x.is_finite()),
                10..100
            )
        ) {
            // Property: Sparse operations should handle extreme values
            let len = indices.len().min(values.len());
            let indices = indices[..len].to_vec();
            let values = values[..len].to_vec();
            
            // Create sparse vector
            let mut pairs: Vec<(u32, f32)> = indices.into_iter().zip(values.into_iter()).collect();
            pairs.sort_unstable_by_key(|(idx, _)| *idx);
            
            // Remove duplicates
            let mut unique_pairs = Vec::new();
            let mut seen = std::collections::HashSet::new();
            for (idx, val) in pairs {
                if seen.insert(idx) {
                    unique_pairs.push((idx, val));
                }
            }
            
            if unique_pairs.is_empty() {
                return Ok(());
            }
            
            let (indices, values) = unique_pairs.into_iter().unzip();
            let vector = SparseVector::new_unchecked(indices, values);
            
            // Test operations
            let norm = vector.norm();
            prop_assert!(
                norm.is_finite(),
                "Norm should be finite: {}",
                norm
            );
            
            let normalized = vector.normalize();
            prop_assert!(
                normalized.norm().is_finite(),
                "Normalized norm should be finite: {}",
                normalized.norm()
            );
        }
    }
}
