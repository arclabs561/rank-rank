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
            if indices[i] <= indices[i-1] {
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
}

/// Compute the dot product between two sparse vectors.
/// Assumes indices are sorted.
pub fn dot_product(a: &SparseVector, b: &SparseVector) -> f32 {
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
}

