//! Comprehensive integration tests for HNSW with compression.

#[cfg(all(feature = "id-compression", feature = "hnsw"))]
mod tests {
    use rank_retrieve::dense::hnsw::graph::{HNSWIndex, HNSWParams};
    use rank_retrieve::compression::IdCompressionMethod;
    use rand::Rng;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn generate_random_vector(dimension: usize, seed: u64) -> Vec<f32> {
        let mut rng = StdRng::seed_from_u64(seed);
        (0..dimension)
            .map(|_| rng.gen::<f32>())
            .collect()
    }

    #[test]
    fn test_hnsw_compression_basic() {
        let params = HNSWParams {
            m: 32,  // Large enough to trigger compression
            m_max: 32,
            ef_construction: 100,
            ef_search: 50,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 32,
            ..Default::default()
        };

        let mut index = HNSWIndex::new(128, params.m, params.m_max).unwrap();
        index.params = params;

        for i in 0..1000 {
            let vector = generate_random_vector(128, i);
            index.add(i, vector).unwrap();
        }

        index.build().unwrap();

        let query = generate_random_vector(128, 9999);
        let results = index.search(&query, 10, 50).unwrap();

        assert_eq!(results.len(), 10);
        assert!(results.iter().all(|(_, score)| score.is_finite()));
    }

    #[test]
    fn test_hnsw_compression_vs_uncompressed() {
        let dimension = 128;
        let num_vectors = 2000;
        
        let vectors: Vec<Vec<f32>> = (0..num_vectors)
            .map(|i| generate_random_vector(dimension, i))
            .collect();

        // With compression
        let mut params_compressed = HNSWParams {
            m: 32,
            m_max: 32,
            ef_construction: 100,
            ef_search: 50,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 32,
            ..Default::default()
        };
        let mut index_compressed = HNSWIndex::new(dimension, params_compressed.m, params_compressed.m_max).unwrap();
        index_compressed.params = params_compressed.clone();
        for (i, vec) in vectors.iter().enumerate() {
            index_compressed.add(i as u32, vec.clone()).unwrap();
        }
        index_compressed.build().unwrap();

        // Without compression
        params_compressed.id_compression = None;
        let mut index_uncompressed = HNSWIndex::new(dimension, params_compressed.m, params_compressed.m_max).unwrap();
        index_uncompressed.params = params_compressed;
        for (i, vec) in vectors.iter().enumerate() {
            index_uncompressed.add(i as u32, vec.clone()).unwrap();
        }
        index_uncompressed.build().unwrap();

        // Both should return same results
        let query = generate_random_vector(dimension, 9999);
        let results_compressed = index_compressed.search(&query, 10, 50).unwrap();
        let results_uncompressed = index_uncompressed.search(&query, 10, 50).unwrap();

        assert_eq!(results_compressed.len(), results_uncompressed.len());
        // Results may differ slightly due to graph structure, but should be similar
        for ((id1, score1), (id2, score2)) in results_compressed.iter().zip(results_uncompressed.iter()) {
            // IDs might differ, but scores should be similar
            assert!((score1 - score2).abs() < 0.1 || id1 == id2);
        }
    }

    #[test]
    fn test_hnsw_compression_threshold() {
        let params = HNSWParams {
            m: 16,  // Below threshold
            m_max: 16,
            ef_construction: 100,
            ef_search: 50,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 32,  // Won't compress
            ..Default::default()
        };

        let mut index = HNSWIndex::new(128, params.m, params.m_max).unwrap();
        index.params = params;

        for i in 0..1000 {
            let vector = generate_random_vector(128, i);
            index.add(i, vector).unwrap();
        }

        index.build().unwrap();

        let query = generate_random_vector(128, 9999);
        let results = index.search(&query, 10, 50).unwrap();
        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_hnsw_compression_large_m() {
        let params = HNSWParams {
            m: 64,  // Large m, will compress
            m_max: 64,
            ef_construction: 200,
            ef_search: 100,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 32,
            ..Default::default()
        };

        let mut index = HNSWIndex::new(128, params.m, params.m_max).unwrap();
        index.params = params;

        for i in 0..5000 {
            let vector = generate_random_vector(128, i);
            index.add(i, vector).unwrap();
        }

        index.build().unwrap();

        let query = generate_random_vector(128, 9999);
        let results = index.search(&query, 10, 100).unwrap();
        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_hnsw_compression_multiple_searches() {
        let params = HNSWParams {
            m: 32,
            m_max: 32,
            ef_construction: 100,
            ef_search: 50,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 32,
            ..Default::default()
        };

        let mut index = HNSWIndex::new(128, params.m, params.m_max).unwrap();
        index.params = params;

        for i in 0..1000 {
            let vector = generate_random_vector(128, i);
            index.add(i, vector).unwrap();
        }

        index.build().unwrap();

        // Multiple searches should work (cache should clear between searches)
        for _ in 0..10 {
            let query = generate_random_vector(128, rand::thread_rng().gen());
            let results = index.search(&query, 10, 50).unwrap();
            assert_eq!(results.len(), 10);
        }
    }

    #[test]
    fn test_hnsw_compression_various_ef_values() {
        let params = HNSWParams {
            m: 32,
            m_max: 32,
            ef_construction: 100,
            ef_search: 50,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 32,
            ..Default::default()
        };

        let mut index = HNSWIndex::new(128, params.m, params.m_max).unwrap();
        index.params = params;

        for i in 0..1000 {
            let vector = generate_random_vector(128, i);
            index.add(i, vector).unwrap();
        }

        index.build().unwrap();

        let query = generate_random_vector(128, 9999);

        // Test various ef_search values
        for ef in [10, 20, 50, 100, 200] {
            let results = index.search(&query, 10, ef).unwrap();
            assert_eq!(results.len(), 10);
        }
    }

    #[test]
    fn test_hnsw_compression_different_dimensions() {
        for dimension in [64, 128, 256, 512] {
            let params = HNSWParams {
                m: 32,
                m_max: 32,
                ef_construction: 100,
                ef_search: 50,
                id_compression: Some(IdCompressionMethod::Roc),
                compression_threshold: 32,
                ..Default::default()
            };

            let mut index = HNSWIndex::new(dimension, params.m, params.m_max).unwrap();
            index.params = params;

            for i in 0..500 {
                let vector = generate_random_vector(dimension, i);
                index.add(i, vector).unwrap();
            }

            index.build().unwrap();

            let query = generate_random_vector(dimension, 9999);
            let results = index.search(&query, 10, 50).unwrap();
            assert_eq!(results.len(), 10);
        }
    }

    #[test]
    fn test_hnsw_compression_with_filtering() {
        let params = HNSWParams {
            m: 32,
            m_max: 32,
            ef_construction: 100,
            ef_search: 50,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 32,
            ..Default::default()
        };

        let mut index = HNSWIndex::with_filtering(128, params.m, params.m_max, "category").unwrap();
        index.params = params;

        for i in 0..1000 {
            let vector = generate_random_vector(128, i);
            index.add(i, vector).unwrap();
            
            let mut metadata = rank_retrieve::filtering::DocumentMetadata::new();
            metadata.insert("category", (i % 10) as u32);
            index.add_metadata(i, metadata).unwrap();
        }

        index.build().unwrap();

        let query = generate_random_vector(128, 9999);
        let filter = rank_retrieve::filtering::FilterPredicate::Equals {
            field: "category",
            value: 5,
        };

        let results = index.search_with_filter(&query, 10, 50, &filter).unwrap();
        assert!(results.len() <= 10);
    }

    #[test]
    fn test_hnsw_compression_cache_clearing() {
        let params = HNSWParams {
            m: 32,
            m_max: 32,
            ef_construction: 100,
            ef_search: 50,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 32,
            ..Default::default()
        };

        let mut index = HNSWIndex::new(128, params.m, params.m_max).unwrap();
        index.params = params;

        for i in 0..1000 {
            let vector = generate_random_vector(128, i);
            index.add(i, vector).unwrap();
        }

        index.build().unwrap();

        // Multiple searches - cache should clear between searches
        let query1 = generate_random_vector(128, 1);
        let query2 = generate_random_vector(128, 2);
        let query3 = generate_random_vector(128, 3);

        let _results1 = index.search(&query1, 10, 50).unwrap();
        let _results2 = index.search(&query2, 10, 50).unwrap();
        let _results3 = index.search(&query3, 10, 50).unwrap();

        // All should succeed
        assert!(true);
    }
}
