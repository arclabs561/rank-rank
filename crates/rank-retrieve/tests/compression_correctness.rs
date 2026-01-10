//! Correctness tests ensuring compression doesn't affect search results.

#[cfg(all(feature = "id-compression", feature = "ivf_pq"))]
mod tests {
    use rank_retrieve::dense::ivf_pq::{IVFPQIndex, IVFPQParams};
    use rank_retrieve::compression::IdCompressionMethod;

    #[test]
    fn test_compression_preserves_search_results() {
        let dimension = 128;
        let num_vectors = 2000;
        
        // Create deterministic vectors
        let vectors: Vec<Vec<f32>> = (0..num_vectors)
            .map(|i| {
                (0..dimension)
                    .map(|j| ((i * dimension + j) as f32 / 1000.0).sin())
                    .collect()
            })
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

        // Test multiple queries
        for query_id in 0..10 {
            let query: Vec<f32> = (0..dimension)
                .map(|j| ((query_id * dimension + j) as f32 / 1000.0).sin())
                .collect();

            let results_compressed = index_compressed.search(&query, 20).unwrap();
            let results_uncompressed = index_uncompressed.search(&query, 20).unwrap();

            // Results should be identical (compression is lossless)
            assert_eq!(results_compressed.len(), results_uncompressed.len());
            
            for ((id1, score1), (id2, score2)) in results_compressed.iter().zip(results_uncompressed.iter()) {
                assert_eq!(id1, id2, "Doc IDs should match");
                assert!((score1 - score2).abs() < 1e-5, 
                    "Scores should match: {} vs {}", score1, score2);
            }
        }
    }

    #[test]
    fn test_compression_preserves_top_k_order() {
        let dimension = 128;
        let num_vectors = 5000;
        
        let vectors: Vec<Vec<f32>> = (0..num_vectors)
            .map(|i| {
                (0..dimension)
                    .map(|j| (i + j) as f32 / 1000.0)
                    .collect()
            })
            .collect();

        let mut params = IVFPQParams {
            num_clusters: 256,
            nprobe: 32,
            num_codebooks: 4,
            codebook_size: 256,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 50,
        };

        let mut index = IVFPQIndex::new(dimension, params.clone()).unwrap();
        for (i, vec) in vectors.iter().enumerate() {
            index.add(i as u32, vec.clone()).unwrap();
        }
        index.build().unwrap();

        let query: Vec<f32> = (0..dimension).map(|i| i as f32 / 1000.0).collect();

        // Results should be sorted by score
        let results = index.search(&query, 100).unwrap();
        for i in 1..results.len() {
            assert!(results[i-1].1 <= results[i].1, 
                "Results should be sorted: {} <= {}", results[i-1].1, results[i].1);
        }
    }

    #[test]
    fn test_compression_handles_all_cluster_sizes() {
        let dimension = 128;
        
        // Create index with varying cluster sizes
        let params = IVFPQParams {
            num_clusters: 1000,  // Many clusters
            nprobe: 50,
            num_codebooks: 4,
            codebook_size: 256,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 100,
        };

        let mut index = IVFPQIndex::new(dimension, params).unwrap();

        // Add vectors (will create clusters of varying sizes)
        for i in 0..5000 {
            let vector: Vec<f32> = (0..dimension).map(|j| (i + j) as f32 / 1000.0).collect();
            index.add(i, vector).unwrap();
        }

        index.build().unwrap();

        // Search should work regardless of cluster sizes
        let query: Vec<f32> = (0..dimension).map(|i| i as f32 / 1000.0).collect();
        let results = index.search(&query, 10).unwrap();
        assert_eq!(results.len(), 10);
    }
}
