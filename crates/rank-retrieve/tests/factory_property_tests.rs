//! Property-based tests for index factory.
//!
//! These tests use property-based testing to validate that the factory
//! correctly handles various inputs and edge cases.

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rank_retrieve::dense::ann::factory::index_factory;
    use rank_retrieve::RetrieveError;

    /// Property: Factory should accept valid HNSW strings
    #[cfg(feature = "hnsw")]
    #[test]
    fn prop_factory_hnsw_valid() {
        proptest!(|(m in 1usize..128, m_max in 1usize..128, dim in 1usize..1024)| {
            let factory_str = if m == m_max {
                format!("HNSW{}", m)
            } else {
                format!("HNSW{},{}", m, m_max)
            };
            
            let result = index_factory(dim, &factory_str);
            prop_assert!(result.is_ok(), 
                "Factory should accept valid HNSW string: '{}' for dim={}", 
                factory_str, dim);
        });
    }

    /// Property: Factory should reject invalid HNSW strings
    #[test]
    fn prop_factory_hnsw_invalid() {
        proptest!(|(m in 0usize..0)| {
            // m = 0 should fail
            let factory_str = format!("HNSW{}", m);
            let result = index_factory(128, &factory_str);
            prop_assert!(result.is_err(), 
                "Factory should reject HNSW with m=0: '{}'", factory_str);
        });
    }

    /// Property: Factory should accept valid IVF-PQ strings
    #[cfg(feature = "ivf_pq")]
    #[test]
    fn prop_factory_ivf_pq_valid() {
        proptest!(|(
            num_clusters in 1usize..1024,
            num_codebooks in 1usize..32,
            dim in 8usize..1024
        )| {
            // Ensure dimension is divisible by num_codebooks
            let dim = (dim / num_codebooks) * num_codebooks;
            if dim == 0 || num_codebooks == 0 {
                return Ok(());
            }
            
            let factory_str = format!("IVF{},PQ{}", num_clusters, num_codebooks);
            let result = index_factory(dim, &factory_str);
            
            prop_assert!(result.is_ok(), 
                "Factory should accept valid IVF-PQ string: '{}' for dim={}", 
                factory_str, dim);
        });
    }

    /// Property: Factory should reject IVF-PQ when dimension not divisible by codebooks
    #[cfg(feature = "ivf_pq")]
    #[test]
    fn prop_factory_ivf_pq_dimension_mismatch() {
        proptest!(|(
            num_clusters in 1usize..1024,
            num_codebooks in 2usize..32,
            dim in 1usize..1024
        )| {
            // Ensure dimension is NOT divisible by num_codebooks
            let dim = if dim % num_codebooks == 0 {
                dim + 1
            } else {
                dim
            };
            
            let factory_str = format!("IVF{},PQ{}", num_clusters, num_codebooks);
            let result = index_factory(dim, &factory_str);
            
            prop_assert!(result.is_err(), 
                "Factory should reject IVF-PQ when dim ({}) not divisible by codebooks ({})", 
                dim, num_codebooks);
        });
    }

    /// Property: Factory should handle whitespace correctly
    #[cfg(feature = "hnsw")]
    #[test]
    fn prop_factory_whitespace_handling() {
        proptest!(|(m in 1usize..128, dim in 1usize..1024)| {
            let base_str = format!("HNSW{}", m);
            let with_whitespace = format!("  {}  ", base_str);
            
            let result1 = index_factory(dim, &base_str);
            let result2 = index_factory(dim, &with_whitespace);
            
            prop_assert_eq!(result1.is_ok(), result2.is_ok(),
                "Whitespace should not affect parsing: '{}' vs '{}'",
                base_str, with_whitespace);
        });
    }

    /// Property: Factory should reject zero dimension
    #[test]
    fn prop_factory_zero_dimension() {
        proptest!(|(factory_str in "[A-Za-z0-9,]+")| {
            // Only test if it's a potentially valid format
            if factory_str.contains("HNSW") || factory_str.contains("IVF") || 
               factory_str.contains("SCANN") || factory_str.contains("NSW") {
                let result = index_factory(0, &factory_str);
                prop_assert!(result.is_err(),
                    "Factory should reject zero dimension for: '{}'", factory_str);
            }
        });
    }

    /// Property: Factory should reject empty strings
    #[test]
    fn prop_factory_empty_string() {
        let result = index_factory(128, "");
        assert!(result.is_err(), "Factory should reject empty string");
        
        let result2 = index_factory(128, "   ");
        assert!(result.is_err(), "Factory should reject whitespace-only string");
    }

    /// Property: Factory creates indexes with correct dimensions
    #[cfg(feature = "hnsw")]
    #[test]
    fn prop_factory_dimension_correct() {
        proptest!(|(dim in 1usize..1024, m in 1usize..128)| {
            let factory_str = format!("HNSW{}", m);
            if let Ok(index) = index_factory(dim, &factory_str) {
                prop_assert_eq!(index.dimension(), dim,
                    "Index dimension should match factory input: expected {}, got {}",
                    dim, index.dimension());
            }
        });
    }

    /// Property: Factory-created indexes can add vectors
    #[cfg(feature = "hnsw")]
    #[test]
    fn prop_factory_add_vectors() {
        proptest!(|(dim in 1usize..256, m in 1usize..64, num_vectors in 1usize..100)| {
            let factory_str = format!("HNSW{}", m);
            if let Ok(mut index) = index_factory(dim, &factory_str) {
                for i in 0..num_vectors {
                    let vec = vec![0.1; dim];
                    let result = index.add(i as u32, vec);
                    prop_assert!(result.is_ok(),
                        "Should be able to add vectors to factory-created index");
                }
                
                prop_assert_eq!(index.num_vectors(), num_vectors,
                    "Index should have correct number of vectors: expected {}, got {}",
                    num_vectors, index.num_vectors());
            }
        });
    }
}
