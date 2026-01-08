//! Tests for the Backend trait interface.
//!
//! These tests verify that the Backend trait can be implemented correctly.

use rank_retrieve::integration::Backend;
use rank_retrieve::RetrieveError;

/// Simple test backend implementation.
struct TestBackend {
    documents: Vec<(u32, Vec<f32>)>,
    built: bool,
}

impl TestBackend {
    fn new() -> Self {
        Self {
            documents: Vec::new(),
            built: false,
        }
    }
}

impl Backend for TestBackend {
    fn retrieve(&self, query: &[f32], k: usize) -> Result<Vec<(u32, f32)>, RetrieveError> {
        if !self.built {
            return Err(RetrieveError::Other("Index not built".to_string()));
        }

        if query.is_empty() {
            return Err(RetrieveError::EmptyQuery);
        }

        let mut scored: Vec<(u32, f32)> = self.documents
            .iter()
            .map(|(doc_id, embedding)| {
                let score = query.iter()
                    .zip(embedding.iter())
                    .map(|(q, e)| q * e)
                    .sum::<f32>();
                (*doc_id, score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(scored.into_iter().take(k).collect())
    }

    fn add_document(&mut self, doc_id: u32, embedding: &[f32]) -> Result<(), RetrieveError> {
        if self.built {
            return Err(RetrieveError::Other("Cannot add after build".to_string()));
        }
        self.documents.push((doc_id, embedding.to_vec()));
        Ok(())
    }

    fn build(&mut self) -> Result<(), RetrieveError> {
        self.built = true;
        Ok(())
    }
}

#[test]
fn test_backend_trait_implementation() {
    let mut backend = TestBackend::new();
    
    backend.add_document(0, &vec![1.0, 0.0]).unwrap();
    backend.add_document(1, &vec![0.0, 1.0]).unwrap();
    backend.build().unwrap();
    
    let results = backend.retrieve(&vec![1.0, 0.0], 10).unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0, 0); // Should find doc 0 first
}

#[test]
fn test_backend_trait_error_handling() {
    let backend = TestBackend::new();
    
    // Should fail if not built
    let result = backend.retrieve(&vec![1.0, 0.0], 10);
    assert!(result.is_err());
    
    // Should fail on empty query
    let mut backend = TestBackend::new();
    backend.build().unwrap();
    let result = backend.retrieve(&vec![], 10);
    assert!(result.is_err());
}

