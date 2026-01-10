//! Property-based tests for sparse vector operations.
//!
//! Verifies properties of sparse vector operations beyond basic dot product.

use proptest::prelude::*;
use rank_retrieve::sparse::SparseVector;

proptest! {
    #[test]
    fn test_sparse_vector_top_k_preserves_ordering(
        indices in prop::collection::vec(0u32..1000, 10..100),
        values in prop::collection::vec(
            (-1e10f32..1e10f32).prop_filter("finite", |&x| x.is_finite()),
            10..100
        ),
        k in 1usize..50
    ) {
        // Ensure indices and values have same length
        let len = indices.len().min(values.len());
        let indices = indices[..len].to_vec();
        let values = values[..len].to_vec();
        
        // Create sparse vector (must be sorted and unique)
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
        
        let k = k.min(vector.nnz());
        let top_k = vector.top_k(k);
        
        // Verify top_k has k elements (or fewer if original had fewer)
        prop_assert!(
            top_k.nnz() <= k,
            "top_k should have at most k elements: got {}, expected <= {}",
            top_k.nnz(),
            k
        );
        prop_assert!(
            top_k.nnz() <= vector.nnz(),
            "top_k should have at most original size: got {}, original {}",
            top_k.nnz(),
            vector.nnz()
        );
        
        // Verify top_k contains the k largest absolute values
        let mut original_abs: Vec<(u32, f32)> = vector
            .indices
            .iter()
            .zip(vector.values.iter())
            .map(|(&idx, &val)| (idx, val.abs()))
            .collect();
        original_abs.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        let top_k_abs: Vec<f32> = top_k.values.iter().map(|&v| v.abs()).collect();
        let expected_top_k_abs: Vec<f32> = original_abs.iter().take(k).map(|(_, abs)| *abs).collect();
        
        // Check that top_k contains the k largest absolute values
        for expected_abs in &expected_top_k_abs {
            prop_assert!(
                top_k_abs.contains(expected_abs) || top_k_abs.iter().any(|&a| (a - expected_abs).abs() < 1e-5),
                "top_k should contain largest absolute values"
            );
        }
    }

    #[test]
    fn test_sparse_vector_normalize_properties(
        indices in prop::collection::vec(0u32..100, 5..50),
        values in prop::collection::vec(
            (-1e10f32..1e10f32).prop_filter("finite", |&x| x.is_finite()),
            5..50
        )
    ) {
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
        
        let normalized = vector.normalize();
        let norm = normalized.norm();
        
        // Property: Normalized vector should have norm ≈ 1.0 (or 0.0 if original was zero)
        if vector.norm() > 1e-9 {
            prop_assert!(
                (norm - 1.0).abs() < 1e-4,
                "Normalized vector should have norm ≈ 1.0: got {}",
                norm
            );
        } else {
            prop_assert_eq!(
                normalized.nnz(),
                0,
                "Zero vector should normalize to empty vector"
            );
        }
    }

    #[test]
    fn test_sparse_vector_norm_properties(
        indices in prop::collection::vec(0u32..100, 5..50),
        values in prop::collection::vec(
            (-1e10f32..1e10f32).prop_filter("finite", |&x| x.is_finite()),
            5..50
        )
    ) {
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
        
        let norm = vector.norm();
        
        // Property: Norm should be non-negative
        prop_assert!(
            norm >= 0.0,
            "Norm should be non-negative: got {}",
            norm
        );
        
        // Property: Norm should be finite
        prop_assert!(
            norm.is_finite(),
            "Norm should be finite: got {}",
            norm
        );
        
        // Property: Norm of zero vector should be 0.0
        if vector.nnz() == 0 {
            prop_assert_eq!(norm, 0.0, "Zero vector should have norm 0.0");
        }
    }
}
