//! Sparse vector representation and operations.
//!
//! Provides efficient sparse vector data structures for retrieval operations.
//!
//! Sparse vectors use parallel arrays of indices and values, where:
//! - Indices are sorted and unique (term IDs in vocabulary)
//! - Values are term weights (e.g., TF-IDF, BM25, SPLADE scores)
//!
//! This enables efficient operations for large vocabularies where most terms are zero.

/// A sparse vector representation using parallel arrays of indices and values.
/// Indices are assumed to be sorted for efficient operations.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SparseVector {
    pub indices: Vec<u32>,
    pub values: Vec<f32>,
}

impl SparseVector {
    /// Create a new SparseVector from sorted indices and values.
    /// Returns None if lengths don't match or indices are not sorted/unique.
    pub fn new(indices: Vec<u32>, values: Vec<f32>) -> Option<Self> {
        if indices.len() != values.len() {
            return None;
        }

        // Verify sorted and unique
        for i in 1..indices.len() {
            if indices[i] <= indices[i - 1] {
                return None;
            }
        }

        Some(Self { indices, values })
    }

    /// Create a new SparseVector without validation (unsafe if used incorrectly).
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - `indices.len() == values.len()`
    /// - `indices` are sorted and unique
    pub fn new_unchecked(indices: Vec<u32>, values: Vec<f32>) -> Self {
        Self { indices, values }
    }

    /// Prune the vector by removing values below a threshold (magnitude).
    pub fn prune(&self, threshold: f32) -> Self {
        let mut new_indices = Vec::with_capacity(self.indices.len());
        let mut new_values = Vec::with_capacity(self.values.len());

        for (i, &val) in self.values.iter().enumerate() {
            if val.abs() >= threshold {
                new_indices.push(self.indices[i]);
                new_values.push(val);
            }
        }

        Self {
            indices: new_indices,
            values: new_values,
        }
    }

    /// Keep only the top-k terms by absolute value.
    ///
    /// Useful for reducing memory usage while preserving the most important terms.
    /// Commonly used in SPLADE and learned sparse retrieval to reduce vector size.
    ///
    /// # Arguments
    ///
    /// * `k` - Number of top terms to keep
    ///
    /// # Returns
    ///
    /// New sparse vector with only top-k terms, sorted by absolute value descending
    pub fn top_k(&self, k: usize) -> Self {
        if k >= self.indices.len() {
            return self.clone();
        }

        // Create pairs of (index, absolute_value) and sort by absolute value
        let mut pairs: Vec<(usize, f32)> = self
            .values
            .iter()
            .enumerate()
            .map(|(i, &val)| (i, val.abs()))
            .collect();

        // Sort by absolute value descending (unstable sort is faster, stability not needed)
        pairs.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top-k and sort by index for consistency
        let mut top_indices = Vec::with_capacity(k);
        let mut top_values = Vec::with_capacity(k);

        for (i, _) in pairs.iter().take(k) {
            top_indices.push(self.indices[*i]);
            top_values.push(self.values[*i]);
        }

        // Sort by index to maintain sorted order
        let mut sorted_pairs: Vec<(u32, f32)> = top_indices
            .into_iter()
            .zip(top_values)
            .collect();
        sorted_pairs.sort_unstable_by_key(|(idx, _)| *idx);

        let (indices, values): (Vec<u32>, Vec<f32>) = sorted_pairs.into_iter().unzip();

        Self { indices, values }
    }

    /// Compute L2 norm of the sparse vector.
    ///
    /// # Returns
    ///
    /// L2 norm (Euclidean norm) of the vector
    pub fn norm(&self) -> f32 {
        self.values.iter().map(|v| v * v).sum::<f32>().sqrt()
    }

    /// Normalize the vector to unit length (L2 normalization).
    ///
    /// # Returns
    ///
    /// New sparse vector with L2 norm = 1.0, or empty vector if original norm is zero.
    pub fn normalize(&self) -> Self {
        let norm = self.norm();
        if norm < 1e-9 {
            // Zero vector, return empty
            return Self {
                indices: Vec::new(),
                values: Vec::new(),
            };
        }

        let normalized_values: Vec<f32> = self.values.iter().map(|v| v / norm).collect();

        Self {
            indices: self.indices.clone(),
            values: normalized_values,
        }
    }

    /// Get the number of non-zero elements.
    pub fn nnz(&self) -> usize {
        self.indices.len()
    }
}

/// Compute the dot product between two sparse vectors.
/// Assumes indices are sorted.
///
/// Uses SIMD-accelerated index comparison when available for better performance.
///
/// # Performance
///
/// - **Very sparse** (< 8 non-zeros): Scalar fallback (SIMD overhead not worth it)
/// - **Sparse** (8-64 non-zeros): SIMD-accelerated index comparison (2-4x speedup)
/// - **Dense** (> 64 non-zeros): Block-based SIMD processing
///
/// # Algorithm
///
/// Uses two-pointer merge algorithm: O(|a| + |b|) time complexity.
/// SIMD acceleration reduces branch mispredictions for larger vectors.
pub fn dot_product(a: &SparseVector, b: &SparseVector) -> f32 {
    #[cfg(all(feature = "sparse", any(feature = "dense", feature = "sparse")))]
    {
        use crate::simd::sparse_dot;
        sparse_dot(&a.indices, &a.values, &b.indices, &b.values)
    }
    #[cfg(not(all(feature = "sparse", any(feature = "dense", feature = "sparse"))))]
    {
        // Fallback to scalar implementation when SIMD is not available
        let mut i = 0;
        let mut j = 0;
        let mut result = 0.0;

        while i < a.indices.len() && j < b.indices.len() {
            if a.indices[i] < b.indices[j] {
                i += 1;
            } else if a.indices[i] > b.indices[j] {
                j += 1;
            } else {
                // Match found
                result += a.values[i] * b.values[j];
                i += 1;
                j += 1;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_product() {
        let v1 = SparseVector::new(vec![1, 3, 5], vec![1.0, 2.0, 3.0]).unwrap();
        let v2 = SparseVector::new(vec![1, 4, 5], vec![0.5, 2.0, 0.5]).unwrap();

        // Match at 1 (1.0 * 0.5 = 0.5) and 5 (3.0 * 0.5 = 1.5)
        // Total = 2.0
        let dot = dot_product(&v1, &v2);
        assert!((dot - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_prune() {
        let v = SparseVector::new(vec![1, 2, 3], vec![0.1, 0.9, -0.2]).unwrap();
        let pruned = v.prune(0.5);

        assert_eq!(pruned.indices, vec![2]);
        assert_eq!(pruned.values, vec![0.9]);
    }

    #[test]
    fn test_top_k() {
        let v = SparseVector::new(vec![1, 2, 3, 4], vec![0.1, 0.9, 0.3, 0.8]).unwrap();
        let top2 = v.top_k(2);

        // Top 2 by absolute value: 0.9 and 0.8
        assert_eq!(top2.indices.len(), 2);
        assert!(top2.indices.contains(&2)); // 0.9
        assert!(top2.indices.contains(&4)); // 0.8
    }

    #[test]
    fn test_norm() {
        let v = SparseVector::new(vec![1, 2], vec![3.0, 4.0]).unwrap();
        // L2 norm: sqrt(3^2 + 4^2) = sqrt(9 + 16) = sqrt(25) = 5.0
        assert!((v.norm() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize() {
        let v = SparseVector::new(vec![1, 2], vec![3.0, 4.0]).unwrap();
        let normalized = v.normalize();

        // Normalized: [3/5, 4/5] = [0.6, 0.8]
        assert!((normalized.norm() - 1.0).abs() < 1e-6);
        assert!((normalized.values[0] - 0.6).abs() < 1e-6);
        assert!((normalized.values[1] - 0.8).abs() < 1e-6);
    }
}
