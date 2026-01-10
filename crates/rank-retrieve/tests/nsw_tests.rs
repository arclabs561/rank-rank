//! Tests for Flat Navigable Small World (NSW) implementation.

#[cfg(feature = "nsw")]
mod tests {
    use rank_retrieve::dense::nsw::{NSWIndex, NSWParams};
    use rank_retrieve::RetrieveError;

    #[test]
    fn test_nsw_basic_functionality() {
        let dimension = 128;
        let params = NSWParams::default();
        let mut index = NSWIndex::with_params(dimension, params).unwrap();

        // Add some vectors
        for i in 0..100 {
            let vector: Vec<f32> = (0..dimension)
                .map(|j| ((i * dimension + j) as f32) * 0.01)
                .collect();
            index.add(i as u32, vector).unwrap();
        }

        // Build index
        index.build().unwrap();

        // Search
        let query: Vec<f32> = (0..dimension).map(|i| (i as f32) * 0.01).collect();
        let results = index.search(&query, 10, 50).unwrap();

        assert_eq!(results.len(), 10);
        // Results should be sorted by distance
        for i in 1..results.len() {
            assert!(results[i].1 >= results[i-1].1, "Results should be sorted by distance");
        }
        // First vector should be very close (distance should be small)
        assert!(results[0].1 < 0.1, "First result should have small distance");
    }

    #[test]
    fn test_nsw_dimension_mismatch() {
        let dimension = 128;
        let params = NSWParams::default();
        let mut index = NSWIndex::with_params(dimension, params).unwrap();

        let wrong_dim_vector = vec![1.0; 64];
        let result = index.add(0, wrong_dim_vector);
        assert!(matches!(result, Err(RetrieveError::DimensionMismatch { .. })));
    }

    #[test]
    fn test_nsw_empty_index() {
        let dimension = 128;
        let params = NSWParams::default();
        let mut index = NSWIndex::with_params(dimension, params).unwrap();

        let result = index.build();
        assert!(matches!(result, Err(RetrieveError::EmptyIndex)));
    }

    #[test]
    fn test_nsw_search_before_build() {
        let dimension = 128;
        let params = NSWParams::default();
        let mut index = NSWIndex::with_params(dimension, params).unwrap();

        let vector: Vec<f32> = vec![1.0; dimension];
        index.add(0, vector).unwrap();

        let query = vec![1.0; dimension];
        let result = index.search(&query, 10, 50);
        assert!(matches!(result, Err(RetrieveError::Other(_))));
    }

    #[test]
    fn test_nsw_single_vector() {
        let dimension = 128;
        let params = NSWParams::default();
        let mut index = NSWIndex::with_params(dimension, params).unwrap();

        let vector: Vec<f32> = vec![1.0; dimension];
        index.add(0, vector).unwrap();
        index.build().unwrap();

        let query = vec![1.0; dimension];
        let results = index.search(&query, 10, 50).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 0);
    }

    #[test]
    fn test_nsw_cannot_add_after_build() {
        let dimension = 128;
        let params = NSWParams::default();
        let mut index = NSWIndex::with_params(dimension, params).unwrap();

        let vector: Vec<f32> = vec![1.0; dimension];
        index.add(0, vector).unwrap();
        index.build().unwrap();

        let vector2: Vec<f32> = vec![2.0; dimension];
        let result = index.add(1, vector2);
        assert!(matches!(result, Err(RetrieveError::Other(_))));
    }
}
