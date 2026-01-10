//! Filtering and lightweight faceting support for vector search.
//!
//! Provides:
//! - **Filter predicates**: Static constraints that narrow search (e.g., category=1)
//! - **Metadata storage**: Document metadata for filtering and faceting
//! - **Filter fusion**: Encoding metadata into embeddings
//! - **Lightweight faceting**: Value enumeration and counts for building filter UIs
//!
//! **Filters vs Facets**:
//! - **Filters**: Applied to queries to narrow search space (e.g., `FilterPredicate::equals("category", 1)`)
//! - **Facets**: Computed from metadata to discover available values and counts (e.g., `get_value_counts("category")`)
//!
//! **Scale and Limitations**:
//! - **Target scale**: 10K-100K documents (RAG systems, academic search)
//! - **Performance**: O(n) iteration - acceptable for target scale, slow for 1M+ documents (M-scale)
//! - **Limitations**: Single-field integrated filtering, static faceting (not from search results), categorical only
//! - **For larger scales**: Use production backends (Elasticsearch, Solr, Qdrant) for B-scale (billion+) or T-scale (trillion+)
//! - **Note**: Using FAISS scale terminology: M=million, B=billion, T=trillion
//!
//! For detailed analysis, see:
//! - [`docs/FACETS_VS_FILTERS.md`](../docs/FACETS_VS_FILTERS.md) - Conceptual differences and implementation
//! - [`docs/FACETS_VS_FILTERS_VALIDATION.md`](../docs/FACETS_VS_FILTERS_VALIDATION.md) - Deep validation against real-world evidence

use crate::RetrieveError;
use std::collections::HashMap;

/// Filter predicate for metadata-based filtering.
///
/// Supports categorical equality filters (e.g., category = "laptop").
/// For range queries or complex predicates, use filter fusion or external filtering.
#[derive(Clone, Debug)]
pub enum FilterPredicate {
    /// Equality filter: field must equal value
    Equals {
        field: String,
        value: u32, // Category ID (0-indexed)
    },
    /// Multiple equality filters (AND logic)
    And(Vec<FilterPredicate>),
    /// Multiple equality filters (OR logic)
    Or(Vec<FilterPredicate>),
}

impl FilterPredicate {
    /// Create an equality filter.
    pub fn equals(field: impl Into<String>, value: u32) -> Self {
        Self::Equals {
            field: field.into(),
            value,
        }
    }

    /// Check if a document matches this filter.
    pub fn matches(&self, metadata: &DocumentMetadata) -> bool {
        match self {
            Self::Equals { field, value } => {
                metadata.get(field).is_some_and(|&v| v == *value)
            }
            Self::And(predicates) => predicates.iter().all(|p| p.matches(metadata)),
            Self::Or(predicates) => predicates.iter().any(|p| p.matches(metadata)),
        }
    }
}

/// Document metadata storage.
///
/// Maps field names to category IDs (u32).
/// For categorical filters, use integer category IDs (0-indexed).
pub type DocumentMetadata = HashMap<String, u32>;

/// Metadata storage for a collection of documents.
///
/// Maps document IDs to their metadata.
#[derive(Debug)]
pub struct MetadataStore {
    /// Document ID -> Metadata
    metadata: HashMap<u32, DocumentMetadata>,
}

impl MetadataStore {
    /// Create a new metadata store.
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
        }
    }

    /// Add metadata for a document.
    pub fn add(&mut self, doc_id: u32, metadata: DocumentMetadata) {
        self.metadata.insert(doc_id, metadata);
    }

    /// Get metadata for a document.
    pub fn get(&self, doc_id: u32) -> Option<&DocumentMetadata> {
        self.metadata.get(&doc_id)
    }

    /// Check if a document matches a filter.
    pub fn matches(&self, doc_id: u32, filter: &FilterPredicate) -> bool {
        self.metadata
            .get(&doc_id)
            .is_some_and(|metadata| filter.matches(metadata))
    }

    /// Estimate filter selectivity (fraction of documents matching filter).
    ///
    /// Returns None if selectivity cannot be estimated (e.g., no metadata).
    pub fn estimate_selectivity(&self, filter: &FilterPredicate) -> Option<f32> {
        if self.metadata.is_empty() {
            return None;
        }

        let matching = self
            .metadata
            .iter()
            .filter(|(_, metadata)| filter.matches(metadata))
            .count();

        Some(matching as f32 / self.metadata.len() as f32)
    }

    /// Get all unique values for a field.
    ///
    /// Returns a sorted list of all category IDs present in the field.
    /// Useful for discovering available filter values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rank_retrieve::filtering::MetadataStore;
    ///
    /// let mut store = MetadataStore::new();
    /// let mut meta1 = std::collections::HashMap::new();
    /// meta1.insert("category".to_string(), 1);
    /// store.add(0, meta1);
    ///
    /// let mut meta2 = std::collections::HashMap::new();
    /// meta2.insert("category".to_string(), 2);
    /// store.add(1, meta2);
    ///
    /// let values = store.get_all_values("category");
    /// assert_eq!(values, vec![1, 2]);
    /// ```
    pub fn get_all_values(&self, field: &str) -> Vec<u32> {
        let mut values: std::collections::HashSet<u32> = std::collections::HashSet::new();
        for metadata in self.metadata.values() {
            if let Some(&value) = metadata.get(field) {
                values.insert(value);
            }
        }
        let mut result: Vec<u32> = values.into_iter().collect();
        result.sort();
        result
    }

    /// Get value counts for a field.
    ///
    /// Returns (value, count) pairs sorted by count descending.
    /// This is basic faceting - shows how many documents have each value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rank_retrieve::filtering::MetadataStore;
    ///
    /// let mut store = MetadataStore::new();
    /// // Add documents with category metadata...
    /// let counts = store.get_value_counts("category");
    /// // Returns: [(1, 10), (2, 5), (3, 2)] - category 1 has 10 docs, etc.
    /// ```
    pub fn get_value_counts(&self, field: &str) -> Vec<(u32, usize)> {
        let mut counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
        for metadata in self.metadata.values() {
            if let Some(&value) = metadata.get(field) {
                *counts.entry(value).or_insert(0) += 1;
            }
        }
        let mut result: Vec<(u32, usize)> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending
        result
    }

    /// Get value counts for documents matching a filter.
    ///
    /// Computes facet counts from a filtered subset of documents.
    /// This enables "filtered faceting" - showing available values in current results.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rank_retrieve::filtering::{MetadataStore, FilterPredicate};
    ///
    /// let store = MetadataStore::new();
    /// // ... add metadata ...
    ///
    /// // Get category counts for documents in region 1
    /// let region_filter = FilterPredicate::equals("region", 1);
    /// let counts = store.get_value_counts_filtered("category", &region_filter);
    /// ```
    pub fn get_value_counts_filtered(
        &self,
        field: &str,
        filter: &FilterPredicate,
    ) -> Vec<(u32, usize)> {
        let mut counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
        for metadata in self.metadata.values() {
            if filter.matches(metadata) {
                if let Some(&value) = metadata.get(field) {
                    *counts.entry(value).or_insert(0) += 1;
                }
            }
        }
        let mut result: Vec<(u32, usize)> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending
        result
    }
}

impl Default for MetadataStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Filter fusion: augment embeddings with metadata.
///
/// Appends weighted one-hot encodings to embeddings so standard ANN search
/// naturally respects categorical filters.
pub mod fusion {
    use super::*;

    /// Augment an embedding with categorical metadata.
    ///
    /// # Arguments
    ///
    /// * `embedding` - Original embedding vector
    /// * `category_id` - Category ID (0-indexed)
    /// * `num_categories` - Total number of categories
    /// * `weight` - Weight for metadata dimensions (higher = stricter filtering)
    ///
    /// # Returns
    ///
    /// Augmented embedding: `[original_embedding, weight * one_hot(category_id)]`
    ///
    /// # Example
    ///
    /// ```rust
    /// use rank_retrieve::filtering::fusion;
    ///
    /// let embedding = vec![0.1, 0.2, 0.3]; // 3-dim embedding
    /// let category_id = 1; // Category 1 (0-indexed)
    /// let num_categories = 3; // 3 total categories
    /// let weight = 10.0;
    ///
    /// let augmented = fusion::augment_embedding(&embedding, category_id, num_categories, weight);
    /// // Result: [0.1, 0.2, 0.3, 0.0, 10.0, 0.0] (3 original + 3 one-hot)
    /// ```
    pub fn augment_embedding(
        embedding: &[f32],
        category_id: u32,
        num_categories: usize,
        weight: f32,
    ) -> Result<Vec<f32>, RetrieveError> {
        if category_id as usize >= num_categories {
            return Err(RetrieveError::Other(format!(
                "Category ID {} >= num_categories {}",
                category_id, num_categories
            )));
        }

        let mut augmented = Vec::with_capacity(embedding.len() + num_categories);
        augmented.extend_from_slice(embedding);

        // Append one-hot encoding
        for i in 0..num_categories {
            if i == category_id as usize {
                augmented.push(weight);
            } else {
                augmented.push(0.0);
            }
        }

        Ok(augmented)
    }

    /// Augment a query embedding with desired category.
    ///
    /// Same as `augment_embedding` but for queries.
    pub fn augment_query(
        query: &[f32],
        desired_category: u32,
        num_categories: usize,
        weight: f32,
    ) -> Result<Vec<f32>, RetrieveError> {
        augment_embedding(query, desired_category, num_categories, weight)
    }

    /// Extract original embedding from augmented embedding.
    ///
    /// Removes the metadata dimensions appended by `augment_embedding`.
    pub fn extract_original(augmented: &[f32], original_dim: usize) -> Vec<f32> {
        augmented[..original_dim].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_predicate_equals() {
        let mut metadata = DocumentMetadata::new();
        metadata.insert("category".to_string(), 1);
        metadata.insert("region".to_string(), 2);

        let filter = FilterPredicate::equals("category", 1);
        assert!(filter.matches(&metadata));

        let filter = FilterPredicate::equals("category", 0);
        assert!(!filter.matches(&metadata));
    }

    #[test]
    fn test_filter_predicate_and() {
        let mut metadata = DocumentMetadata::new();
        metadata.insert("category".to_string(), 1);
        metadata.insert("region".to_string(), 2);

        let filter = FilterPredicate::And(vec![
            FilterPredicate::equals("category", 1),
            FilterPredicate::equals("region", 2),
        ]);
        assert!(filter.matches(&metadata));

        let filter = FilterPredicate::And(vec![
            FilterPredicate::equals("category", 1),
            FilterPredicate::equals("region", 0),
        ]);
        assert!(!filter.matches(&metadata));
    }

    #[test]
    fn test_metadata_store() {
        let mut store = MetadataStore::new();
        let mut metadata = DocumentMetadata::new();
        metadata.insert("category".to_string(), 1);
        store.add(0, metadata);

        let filter = FilterPredicate::equals("category", 1);
        assert!(store.matches(0, &filter));
        assert!(!store.matches(1, &filter)); // Document 1 doesn't exist
    }

    #[test]
    fn test_filter_fusion_augment() {
        let embedding = vec![0.1, 0.2, 0.3];
        let augmented = fusion::augment_embedding(&embedding, 1, 3, 10.0).unwrap();

        assert_eq!(augmented.len(), 6); // 3 original + 3 one-hot
        assert_eq!(augmented[0..3], embedding);
        assert_eq!(augmented[3], 0.0); // Category 0
        assert_eq!(augmented[4], 10.0); // Category 1 (desired)
        assert_eq!(augmented[5], 0.0); // Category 2
    }

    #[test]
    fn test_filter_fusion_extract() {
        let original = vec![0.1, 0.2, 0.3];
        let augmented = fusion::augment_embedding(&original, 1, 3, 10.0).unwrap();
        let extracted = fusion::extract_original(&augmented, 3);

        assert_eq!(extracted, original);
    }

    #[test]
    fn test_get_all_values() {
        let mut store = MetadataStore::new();
        let mut meta1 = DocumentMetadata::new();
        meta1.insert("category".to_string(), 1);
        store.add(0, meta1);

        let mut meta2 = DocumentMetadata::new();
        meta2.insert("category".to_string(), 2);
        store.add(1, meta2);

        let mut meta3 = DocumentMetadata::new();
        meta3.insert("category".to_string(), 1); // Duplicate
        store.add(2, meta3);

        let values = store.get_all_values("category");
        assert_eq!(values, vec![1, 2]);
    }

    #[test]
    fn test_get_value_counts() {
        let mut store = MetadataStore::new();
        for i in 0..10 {
            let mut meta = DocumentMetadata::new();
            meta.insert("category".to_string(), 1);
            store.add(i, meta);
        }
        for i in 10..15 {
            let mut meta = DocumentMetadata::new();
            meta.insert("category".to_string(), 2);
            store.add(i, meta);
        }

        let counts = store.get_value_counts("category");
        assert_eq!(counts.len(), 2);
        assert_eq!(counts[0], (1, 10)); // Category 1 has 10 docs
        assert_eq!(counts[1], (2, 5)); // Category 2 has 5 docs
    }

    #[test]
    fn test_get_value_counts_filtered() {
        let mut store = MetadataStore::new();
        // Add documents with category and region
        for i in 0..10 {
            let mut meta = DocumentMetadata::new();
            meta.insert("category".to_string(), 1);
            meta.insert("region".to_string(), 1);
            store.add(i, meta);
        }
        for i in 10..15 {
            let mut meta = DocumentMetadata::new();
            meta.insert("category".to_string(), 2);
            meta.insert("region".to_string(), 1);
            store.add(i, meta);
        }
        for i in 15..20 {
            let mut meta = DocumentMetadata::new();
            meta.insert("category".to_string(), 1);
            meta.insert("region".to_string(), 2); // Different region
            store.add(i, meta);
        }

        // Get category counts for region 1 only
        let region_filter = FilterPredicate::equals("region", 1);
        let counts = store.get_value_counts_filtered("category", &region_filter);
        assert_eq!(counts.len(), 2);
        assert_eq!(counts[0], (1, 10)); // Category 1: 10 docs in region 1
        assert_eq!(counts[1], (2, 5)); // Category 2: 5 docs in region 1
    }
}
