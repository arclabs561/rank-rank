//! Comprehensive integration tests for IVF-PQ with compression.

#[cfg(all(feature = "id-compression", feature = "ivf_pq"))]
mod tests {
    use rank_retrieve::dense::ivf_pq::{IVFPQIndex, IVFPQParams};
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
    fn test_ivf_compression_basic() {
        let params = IVFPQParams {
            num_clusters: 64,
            nprobe: 10,
            num_codebooks: 4,
            codebook_size: 256,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 10,
        };

        let mut index = IVFPQIndex::new(128, params).unwrap();

        // Add vectors
        for i in 0..1000 {
            let vector = generate_random_vector(128, i);
            index.add(i as u32, vector).unwrap();
        }

        index.build().unwrap();

        // Search
        let query = generate_random_vector(128, 9999);
        let results = index.search(&query, 10).unwrap();

        assert_eq!(results.len(), 10);
        assert!(results.iter().all(|(_, score)| score.is_finite()));
    }

    #[test]
    fn test_ivf_compression_vs_uncompressed() {
        let dimension = 128;
        let num_vectors = 2000;
        
        // Create vectors
        let vectors: Vec<Vec<f32>> = (0..num_vectors)
            .map(|i| generate_random_vector(dimension, i))
            .collect();

        // Build with compression
        let mut params_compressed = IVFPQParams {
            num_clusters: 128,
            nprobe: 20,
            num_codebooks: 4,
            codebook_size: 256,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 50,
        };
        let mut index_compressed = IVFPQIndex::new(dimension, params_compressed.clone()).unwrap();
        for (i, vec) in vectors.iter().enumerate() {
            index_compressed.add(i as u32, vec.clone()).unwrap();
        }
        index_compressed.build().unwrap();

        // Build without compression
        params_compressed.id_compression = None;
        let mut index_uncompressed = IVFPQIndex::new(dimension, params_compressed).unwrap();
        for (i, vec) in vectors.iter().enumerate() {
            index_uncompressed.add(i as u32, vec.clone()).unwrap();
        }
        index_uncompressed.build().unwrap();

        // Both should return same results (compression is lossless)
        let query = generate_random_vector(dimension, 9999);
        let results_compressed = index_compressed.search(&query, 10).unwrap();
        let results_uncompressed = index_uncompressed.search(&query, 10).unwrap();

        // Results should be identical (same doc IDs, same scores)
        assert_eq!(results_compressed.len(), results_uncompressed.len());
        for ((id1, score1), (id2, score2)) in results_compressed.iter().zip(results_uncompressed.iter()) {
            assert_eq!(id1, id2);
            assert!((score1 - score2).abs() < 1e-6);
        }
    }

    #[test]
    fn test_ivf_compression_threshold() {
        let params_below_threshold = IVFPQParams {
            num_clusters: 1000,  // Many clusters, small clusters
            nprobe: 50,
            num_codebooks: 4,
            codebook_size: 256,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 100,  // High threshold
        };

        let mut index = IVFPQIndex::new(128, params_below_threshold).unwrap();

        // Add few vectors per cluster
        for i in 0..500 {
            let vector = generate_random_vector(128, i);
            index.add(i as u32, vector).unwrap();
        }

        index.build().unwrap();

        // Should still work (small clusters won't be compressed)
        let query = generate_random_vector(128, 9999);
        let results = index.search(&query, 10).unwrap();
        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_ivf_compression_large_clusters() {
        let params = IVFPQParams {
            num_clusters: 10,  // Few clusters, large clusters
            nprobe: 5,
            num_codebooks: 4,
            codebook_size: 256,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 50,  // Low threshold
        };

        let mut index = IVFPQIndex::new(128, params).unwrap();

        // Add many vectors (will create large clusters)
        for i in 0..5000 {
            let vector = generate_random_vector(128, i);
            index.add(i as u32, vector).unwrap();
        }

        index.build().unwrap();

        let query = generate_random_vector(128, 9999);
        let results = index.search(&query, 10).unwrap();
        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_ivf_compression_empty_clusters() {
        let params = IVFPQParams {
            num_clusters: 100,
            nprobe: 10,
            num_codebooks: 4,
            codebook_size: 256,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 10,
        };

        let mut index = IVFPQIndex::new(128, params).unwrap();

        // Add only a few vectors (many empty clusters)
        for i in 0..50 {
            let vector = generate_random_vector(128, i);
            index.add(i as u32, vector).unwrap();
        }

        index.build().unwrap();

        let query = generate_random_vector(128, 9999);
        let results = index.search(&query, 10).unwrap();
        assert!(results.len() <= 10);
    }

    #[test]
    fn test_ivf_compression_multiple_searches() {
        let params = IVFPQParams {
            num_clusters: 64,
            nprobe: 10,
            num_codebooks: 4,
            codebook_size: 256,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 10,
        };

        let mut index = IVFPQIndex::new(128, params).unwrap();

        for i in 0..1000 {
            let vector = generate_random_vector(128, i);
            index.add(i as u32, vector).unwrap();
        }

        index.build().unwrap();

        // Multiple searches should work
        for _ in 0..10 {
            let query = generate_random_vector(128, rand::thread_rng().gen());
            let results = index.search(&query, 10).unwrap();
            assert_eq!(results.len(), 10);
        }
    }

    #[test]
    fn test_ivf_compression_various_k_values() {
        let params = IVFPQParams {
            num_clusters: 64,
            nprobe: 10,
            num_codebooks: 4,
            codebook_size: 256,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 10,
        };

        let mut index = IVFPQIndex::new(128, params).unwrap();

        for i in 0..1000 {
            let vector = generate_random_vector(128, i);
            index.add(i as u32, vector).unwrap();
        }

        index.build().unwrap();

        let query = generate_random_vector(128, 9999);

        // Test various k values
        for k in [1, 5, 10, 20, 50, 100] {
            let results = index.search(&query, k).unwrap();
            assert_eq!(results.len(), k.min(1000));
        }
    }

    #[test]
    fn test_ivf_compression_different_dimensions() {
        for dimension in [64, 128, 256, 512] {
            let params = IVFPQParams {
                num_clusters: 64,
                nprobe: 10,
                num_codebooks: 4,
                codebook_size: 256,
                id_compression: Some(IdCompressionMethod::Roc),
                compression_threshold: 10,
            };

            let mut index = IVFPQIndex::new(dimension, params).unwrap();

            for i in 0..500 {
                let vector = generate_random_vector(dimension, i);
                index.add(i as u32, vector).unwrap();
            }

            index.build().unwrap();

            let query = generate_random_vector(dimension, 9999);
            let results = index.search(&query, 10).unwrap();
            assert_eq!(results.len(), 10);
        }
    }

    #[test]
    fn test_ivf_compression_with_filtering() {
        let params = IVFPQParams {
            num_clusters: 64,
            nprobe: 10,
            num_codebooks: 4,
            codebook_size: 256,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 10,
        };

        let mut index = IVFPQIndex::with_filtering(128, params, "category").unwrap();

        // Add vectors with metadata
        for i in 0..1000 {
            let vector = generate_random_vector(128, i);
            index.add(i as u32, vector).unwrap();
            
            let mut metadata = rank_retrieve::filtering::DocumentMetadata::new();
            metadata.insert("category", (i % 10) as u32);
            index.add_metadata(i, metadata).unwrap();
        }

        index.build().unwrap();

        // Search with filter
        let query = generate_random_vector(128, 9999);
        let filter = rank_retrieve::filtering::FilterPredicate::Equals {
            field: "category",
            value: 5,
        };

        let results = index.search_with_filter(&query, 10, &filter).unwrap();
        assert!(results.len() <= 10);
        // All results should have category 5
        for (doc_id, _) in &results {
            let metadata = index.metadata.as_ref().unwrap().get(*doc_id).unwrap();
            assert_eq!(metadata.get("category"), Some(&5));
        }
    }
}
