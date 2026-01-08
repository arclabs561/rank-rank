//! Test Helper Types and Utilities
//!
//! Common types and utilities for organizing test data and reducing duplication.
//! These helpers make tests more maintainable and easier to read.
//!
//! These helpers are designed to work across all rank-* crates:
//! - rank-retrieve: BM25, dense, sparse retrieval
//! - rank-fusion: Result fusion
//! - rank-rerank: Reranking
//! - rank-eval: Evaluation metrics
//!
//! The types are generic over ID types (String, &str, u32, u64) to work with
//! different crates' result formats.

#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "dense")]
use rank_retrieve::dense::DenseRetriever;
#[cfg(feature = "sparse")]
use rank_retrieve::sparse::{SparseRetriever, SparseVector};
use std::collections::HashSet;
use std::fmt::Display;

/// Test collection with documents and relevance judgments.
///
/// Encapsulates a labeled test collection for evaluation.
/// Generic over ID type to work with different crates (String, u32, etc.)
#[derive(Debug, Clone)]
pub struct TestCollection<ID = String>
where
    ID: Clone + Display + Eq + std::hash::Hash,
{
    /// Document ID to terms mapping
    pub documents: Vec<(ID, Vec<String>)>,
    /// Query ID to query terms mapping
    pub queries: Vec<(String, Vec<String>)>,
    /// Query ID to relevant document IDs mapping
    pub qrels: Vec<(String, HashSet<String>)>,
}

impl TestCollection<String> {
    /// Create a simple test collection for machine learning queries.
    pub fn machine_learning_collection() -> Self {
        let documents = vec![
            (0.to_string(), vec!["machine".to_string(), "learning".to_string(), "algorithms".to_string()]),
            (1.to_string(), vec!["deep".to_string(), "learning".to_string(), "neural".to_string()]),
            (2.to_string(), vec!["python".to_string(), "programming".to_string()]),
            (3.to_string(), vec!["rust".to_string(), "systems".to_string()]),
        ];
        
        let queries = vec![
            ("q1".to_string(), vec!["machine".to_string(), "learning".to_string()]),
            ("q2".to_string(), vec!["deep".to_string(), "learning".to_string()]),
        ];
        
        let qrels = vec![
            ("q1".to_string(), ["0", "1"].iter().map(|s| s.to_string()).collect()),
            ("q2".to_string(), ["1"].iter().map(|s| s.to_string()).collect()),
        ];
        
        Self { documents, queries, qrels }
    }
    
    /// Create a test collection with head and tail queries.
    pub fn head_tail_collection() -> Self {
        let mut documents = Vec::new();
        // Add common term to many documents (head)
        for i in 0..50 {
            let mut terms = vec![format!("term{}", i % 10)];
            if i % 2 == 0 {
                terms.push("common".to_string());
            }
            documents.push((i.to_string(), terms));
        }
        // Add rare term to one document (tail)
        documents.push((50.to_string(), vec!["rare_term_xyz".to_string()]));
        
        let queries = vec![
            ("head".to_string(), vec!["common".to_string()]),
            ("tail".to_string(), vec!["rare_term_xyz".to_string()]),
        ];
        
        let head_relevant: HashSet<String> = (0..50)
            .step_by(2)
            .map(|i| i.to_string())
            .collect();
        let tail_relevant: HashSet<String> = ["50"].iter().map(|s| s.to_string()).collect();
        
        let qrels = vec![
            ("head".to_string(), head_relevant),
            ("tail".to_string(), tail_relevant),
        ];
        
        Self { documents, queries, qrels }
    }
}

impl<ID> TestCollection<ID>
where
    ID: Clone + Display + Eq + std::hash::Hash,
{
    /// Get query terms by query ID.
    pub fn get_query(&self, query_id: &str) -> Option<Vec<String>> {
        self.queries
            .iter()
            .find(|(id, _)| id == query_id)
            .map(|(_, terms)| terms.clone())
    }
    
    /// Get relevant documents by query ID.
    pub fn get_relevant(&self, query_id: &str) -> Option<HashSet<String>> {
        self.qrels
            .iter()
            .find(|(id, _)| id == query_id)
            .map(|(_, relevant)| relevant.clone())
    }
}

/// Multi-retriever test fixture.
///
/// Encapsulates BM25, dense, and sparse retrievers with the same documents.
#[cfg(feature = "bm25")]
#[cfg(feature = "dense")]
#[cfg(feature = "sparse")]
pub struct MultiRetrieverFixture {
    pub bm25_index: InvertedIndex,
    pub dense_retriever: DenseRetriever,
    pub sparse_retriever: SparseRetriever,
    pub collection: TestCollection<String>,
}

#[cfg(feature = "bm25")]
#[cfg(feature = "dense")]
#[cfg(feature = "sparse")]
impl MultiRetrieverFixture {
    /// Create a fixture with machine learning test collection.
    pub fn machine_learning() -> Self {
        let collection = TestCollection::machine_learning_collection();
        
        let mut bm25_index = InvertedIndex::new();
        let mut dense_retriever = DenseRetriever::new();
        let mut sparse_retriever = SparseRetriever::new();
        
        // Add documents to all retrievers
        for (id_str, terms) in &collection.documents {
            let id: u32 = id_str.parse().unwrap_or(0);
            bm25_index.add_document(id, terms);
            
            // Create embeddings (simplified)
            let embedding: Vec<f32> = (0..64)
                .map(|i| (id as usize + i) as f32 / 200.0)
                .collect();
            dense_retriever.add_document(id, embedding);
            
            // Create sparse vectors
            let indices: Vec<u32> = (0..terms.len().min(10)).map(|i| i as u32).collect();
            let values: Vec<f32> = (0..terms.len().min(10)).map(|i| 1.0 / (i + 1) as f32).collect();
            let sparse_vec = SparseVector::new_unchecked(indices, values);
            sparse_retriever.add_document(id, sparse_vec);
        }
        
        Self {
            bm25_index,
            dense_retriever,
            sparse_retriever,
            collection,
        }
    }
}

/// Query type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    Lexical,
    Semantic,
    Short,  // 1-2 terms
    Long,   // 5+ terms
    Head,   // Common terms
    Tail,   // Rare terms
}

/// Query representation for testing.
#[derive(Debug, Clone)]
pub struct TestQuery {
    pub id: String,
    pub terms: Vec<String>,
    pub query_type: QueryType,
    pub dense_embedding: Option<Vec<f32>>,
    pub sparse_vector: Option<SparseVector>,
}

impl TestQuery {
    /// Create a lexical query (exact keyword match).
    pub fn lexical(terms: Vec<String>) -> Self {
        Self {
            id: format!("lexical_{}", terms.join("_")),
            query_type: QueryType::Lexical,
            terms,
            dense_embedding: None,
            sparse_vector: None,
        }
    }
    
    /// Create a short query (1-2 terms).
    pub fn short(terms: Vec<String>) -> Self {
        Self {
            id: format!("short_{}", terms.join("_")),
            query_type: QueryType::Short,
            terms,
            dense_embedding: None,
            sparse_vector: None,
        }
    }
    
    /// Create a long query (5+ terms).
    pub fn long(terms: Vec<String>) -> Self {
        Self {
            id: format!("long_{}", terms.join("_")),
            query_type: QueryType::Long,
            terms,
            dense_embedding: None,
            sparse_vector: None,
        }
    }
}

/// Evaluation results for a single query.
///
/// Works with any result type that implements ToRankedList.
#[derive(Debug, Clone)]
pub struct EvaluationResults {
    pub query_id: String,
    pub precision_at_1: f64,
    pub precision_at_5: f64,
    pub precision_at_10: f64,
    pub recall_at_5: f64,
    pub recall_at_10: f64,
    pub ndcg_at_5: f64,
    pub ndcg_at_10: f64,
    pub mrr: f64,
    pub map: f64,
}

impl EvaluationResults {
    /// Create evaluation results from ranked list and relevant set.
    pub fn from_ranked(
        query_id: String,
        ranked: &[String],
        relevant: &HashSet<String>,
    ) -> Self {
        use rank_eval::binary::{
            average_precision, mrr, ndcg_at_k, precision_at_k, recall_at_k,
        };
        
        Self {
            query_id,
            precision_at_1: precision_at_k(ranked, relevant, 1),
            precision_at_5: precision_at_k(ranked, relevant, 5),
            precision_at_10: precision_at_k(ranked, relevant, 10),
            recall_at_5: recall_at_k(ranked, relevant, 5),
            recall_at_10: recall_at_k(ranked, relevant, 10),
            ndcg_at_5: ndcg_at_k(ranked, relevant, 5),
            ndcg_at_10: ndcg_at_k(ranked, relevant, 10),
            mrr: mrr(ranked, relevant),
            map: average_precision(ranked, relevant),
        }
    }
    
    /// Check if results meet minimum thresholds.
    pub fn meets_thresholds(&self, min_precision: f64, min_recall: f64, min_ndcg: f64) -> bool {
        self.precision_at_10 >= min_precision
            && self.recall_at_10 >= min_recall
            && self.ndcg_at_10 >= min_ndcg
    }
}

/// Helper to convert retrieval results to ranked list for evaluation.
///
/// Works with any ID type that implements Display.
pub trait ToRankedList {
    fn to_ranked_list(&self) -> Vec<String>;
}

impl ToRankedList for Vec<(u32, f32)> {
    fn to_ranked_list(&self) -> Vec<String> {
        self.iter().map(|(id, _)| id.to_string()).collect()
    }
}

impl ToRankedList for Vec<(String, f32)> {
    fn to_ranked_list(&self) -> Vec<String> {
        self.iter().map(|(id, _)| id.clone()).collect()
    }
}

impl<'a> ToRankedList for Vec<(&'a str, f32)> {
    fn to_ranked_list(&self) -> Vec<String> {
        self.iter().map(|(id, _)| id.to_string()).collect()
    }
}

impl ToRankedList for Vec<(u64, f32)> {
    fn to_ranked_list(&self) -> Vec<String> {
        self.iter().map(|(id, _)| id.to_string()).collect()
    }
}

/// Helper to create relevance sets from document IDs.
pub fn relevant_set(ids: &[u32]) -> HashSet<String> {
    ids.iter().map(|id| id.to_string()).collect()
}

/// Helper to create relevance sets from string document IDs.
pub fn relevant_set_str(ids: &[&str]) -> HashSet<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

/// Test scenario builder for creating consistent test setups.
#[cfg(feature = "bm25")]
pub struct TestScenario {
    pub index: InvertedIndex,
    pub collection: TestCollection<String>,
}

#[cfg(feature = "bm25")]
impl TestScenario {
    /// Create a new test scenario from a collection.
    pub fn new(collection: TestCollection<String>) -> Self {
        let mut index = InvertedIndex::new();
        for (id_str, terms) in &collection.documents {
            let id: u32 = id_str.parse().unwrap_or(0);
            index.add_document(id, terms);
        }
        Self { index, collection }
    }
    
    /// Get query terms by query ID.
    pub fn get_query(&self, query_id: &str) -> Option<Vec<String>> {
        self.collection.get_query(query_id)
    }
    
    /// Get relevant documents by query ID.
    pub fn get_relevant(&self, query_id: &str) -> Option<HashSet<String>> {
        self.collection.get_relevant(query_id)
    }
}

/// Generic result type for fusion tests.
///
/// Works with any ID type that implements Display.
pub type FusionResult<ID> = Vec<(ID, f32)>;

/// Helper to create test results for fusion testing.
///
/// Works across rank-fusion, rank-retrieve, and other crates.
pub fn create_test_results<ID: Display + Clone>(
    ids_and_scores: &[(ID, f32)],
) -> Vec<(ID, f32)> {
    ids_and_scores.to_vec()
}

/// Helper to convert results to ranked list for evaluation.
///
/// Generic over ID type to work with different crates.
pub fn results_to_ranked_list<ID: Display>(results: &[(ID, f32)]) -> Vec<String> {
    results.iter().map(|(id, _)| id.to_string()).collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Mock Embedding Generators (for rank-rerank, rank-retrieve dense tests)
// ─────────────────────────────────────────────────────────────────────────────

/// Generate a normalized embedding from text (deterministic hash-based).
///
/// Useful for testing dense retrieval and reranking without actual models.
/// Same pattern as used in rank-rerank/tests/integration.rs
pub fn mock_dense_embed(text: &str, dim: usize) -> Vec<f32> {
    let mut embedding = vec![0.0; dim];
    for (i, c) in text.chars().enumerate() {
        embedding[i % dim] += (c as u32 as f32) / 1000.0;
    }
    // L2 normalize
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        embedding.iter_mut().for_each(|x| *x /= norm);
    }
    embedding
}

/// Generate token embeddings (one per word, simplified).
///
/// Useful for ColBERT-style reranking tests.
pub fn mock_token_embed(text: &str, dim: usize) -> Vec<Vec<f32>> {
    text.split_whitespace()
        .map(|word| mock_dense_embed(word, dim))
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// TREC Format Helpers (for rank-eval integration tests)
// ─────────────────────────────────────────────────────────────────────────────

/// TREC run entry for writing test data.
#[derive(Debug, Clone)]
pub struct TrecRunEntry {
    pub query_id: String,
    pub doc_id: String,
    pub rank: usize,
    pub score: f32,
    pub run_tag: String,
}

impl TrecRunEntry {
    /// Format as TREC run line: "query_id Q0 doc_id rank score run_tag"
    pub fn to_trec_line(&self) -> String {
        format!("{} Q0 {} {} {:.6} {}", self.query_id, self.doc_id, self.rank, self.score, self.run_tag)
    }
}

/// TREC qrel entry for writing test data.
#[derive(Debug, Clone)]
pub struct TrecQrelEntry {
    pub query_id: String,
    pub doc_id: String,
    pub relevance: u32,
}

impl TrecQrelEntry {
    /// Format as TREC qrel line: "query_id 0 doc_id relevance"
    pub fn to_trec_line(&self) -> String {
        format!("{} 0 {} {}", self.query_id, self.doc_id, self.relevance)
    }
}

/// Helper to create TREC runs from retrieval results.
pub fn results_to_trec_runs<ID: Display>(
    query_id: &str,
    results: &[(ID, f32)],
    run_tag: &str,
) -> Vec<TrecRunEntry> {
    results
        .iter()
        .enumerate()
        .map(|(rank, (doc_id, score))| TrecRunEntry {
            query_id: query_id.to_string(),
            doc_id: doc_id.to_string(),
            rank: rank + 1,
            score: *score,
            run_tag: run_tag.to_string(),
        })
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Property-Based Testing Helpers (for proptest integration)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "proptest")]
pub mod proptest_helpers {
    use super::*;
    use proptest::prelude::*;

    /// Strategy for generating retrieval results.
    ///
    /// Generates `Vec<(u32, f32)>` with IDs in range [0, max_id) and scores in [0.0, 1.0).
    pub fn arb_results(max_len: usize, max_id: u32) -> impl Strategy<Value = Vec<(u32, f32)>> {
        proptest::collection::vec((0u32..max_id, 0.0f32..1.0), 0..max_len)
    }

    /// Strategy for generating string-based retrieval results.
    ///
    /// Generates `Vec<(String, f32)>` with document IDs as strings.
    pub fn arb_string_results(max_len: usize, max_id: u32) -> impl Strategy<Value = Vec<(String, f32)>> {
        arb_results(max_len, max_id)
            .prop_map(|results| {
                results.into_iter()
                    .map(|(id, score)| (id.to_string(), score))
                    .collect()
            })
    }

    /// Strategy for generating test queries.
    pub fn arb_query(max_terms: usize) -> impl Strategy<Value = Vec<String>> {
        proptest::collection::vec("[a-z]{3,10}", 1..=max_terms)
    }

    /// Strategy for generating relevance sets.
    pub fn arb_relevant_set(max_docs: u32, max_relevant: usize) -> impl Strategy<Value = HashSet<String>> {
        proptest::collection::hash_set(0u32..max_docs, 0..=max_relevant)
            .prop_map(|ids| ids.into_iter().map(|id| id.to_string()).collect())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test Data Builders (builder pattern for complex test scenarios)
// ─────────────────────────────────────────────────────────────────────────────

/// Builder for creating test retrieval results.
pub struct ResultBuilder<ID> {
    results: Vec<(ID, f32)>,
}

impl<ID> ResultBuilder<ID> {
    pub fn new() -> Self {
        Self { results: Vec::new() }
    }
}

impl<ID: Clone> ResultBuilder<ID> {

    pub fn add(mut self, id: ID, score: f32) -> Self {
        self.results.push((id, score));
        self
    }

    pub fn add_many(mut self, items: Vec<(ID, f32)>) -> Self {
        self.results.extend(items);
        self
    }

    pub fn build(mut self) -> Vec<(ID, f32)> {
        // Sort by score descending
        self.results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        self.results
    }
}

impl<ID> Default for ResultBuilder<ID> {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating test collections.
pub struct TestCollectionBuilder {
    documents: Vec<(String, Vec<String>)>,
    queries: Vec<(String, Vec<String>)>,
    qrels: Vec<(String, HashSet<String>)>,
}

impl TestCollectionBuilder {
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
            queries: Vec::new(),
            qrels: Vec::new(),
        }
    }

    pub fn add_document(mut self, id: impl Into<String>, terms: Vec<String>) -> Self {
        self.documents.push((id.into(), terms));
        self
    }

    pub fn add_query(mut self, id: impl Into<String>, terms: Vec<String>) -> Self {
        self.queries.push((id.into(), terms));
        self
    }

    pub fn add_qrel(mut self, query_id: impl Into<String>, doc_ids: Vec<impl Into<String>>) -> Self {
        let relevant: HashSet<String> = doc_ids.into_iter().map(|id| id.into()).collect();
        self.qrels.push((query_id.into(), relevant));
        self
    }

    pub fn build(self) -> TestCollection<String> {
        TestCollection {
            documents: self.documents,
            queries: self.queries,
            qrels: self.qrels,
        }
    }
}

impl Default for TestCollectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}
