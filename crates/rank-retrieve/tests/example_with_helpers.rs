//! Example tests using helper types
//!
//! This file demonstrates how the helper types in `test_helpers.rs` can
//! simplify test code and reduce duplication.

#[cfg(test)]
#[path = "test_helpers.rs"]
mod test_helpers;

#[cfg(test)]
mod tests {
    use super::test_helpers::*;
    #[cfg(feature = "bm25")]
    use rank_retrieve::{retrieve_bm25, batch::batch_retrieve_bm25};
    #[cfg(feature = "bm25")]
    use rank_retrieve::bm25::Bm25Params;
    use rank_eval::binary::ndcg_at_k;

    #[cfg(feature = "bm25")]
    #[test]
    fn example_using_test_collection() {
        // Before: Manual setup with repeated document/query creation
        // After: Use TestCollection helper
        
        let collection = TestCollection::machine_learning_collection();
        let scenario = TestScenario::new(collection.clone());
        
        // Get query and relevant documents
        let query = scenario.get_query("q1").unwrap();
        let relevant = scenario.get_relevant("q1").unwrap();
        
        // Retrieve
        let results = retrieve_bm25(&scenario.index, &query, 10, Bm25Params::default()).unwrap();
        
        // Evaluate
        let ranked = results.to_ranked_list();
        let ndcg = ndcg_at_k(&ranked, &relevant, 10);
        
        assert!(ndcg > 0.0);
    }

    #[cfg(feature = "bm25")]
    #[test]
    fn example_using_evaluation_results() {
        // Before: Manual metric calculation
        // After: Use EvaluationResults helper
        
        let collection = TestCollection::machine_learning_collection();
        let scenario = TestScenario::new(collection.clone());
        
        let query = scenario.get_query("q1").unwrap();
        let relevant = scenario.get_relevant("q1").unwrap();
        
        let results = retrieve_bm25(&scenario.index, &query, 10, Bm25Params::default()).unwrap();
        let ranked = results.to_ranked_list();
        
        // All metrics calculated at once
        let eval = EvaluationResults::from_ranked("q1".to_string(), &ranked, &relevant);
        
        // Check thresholds
        assert!(eval.meets_thresholds(0.1, 0.1, 0.1));
        assert!(eval.ndcg_at_10 > 0.0);
    }

    #[cfg(feature = "bm25")]
    #[test]
    fn example_using_query_types() {
        // Before: Manual query creation
        // After: Use TestQuery helper
        
        let collection = TestCollection::machine_learning_collection();
        let scenario = TestScenario::new(collection);
        
        // Create typed queries
        let lexical_query = TestQuery::lexical(vec!["machine".to_string(), "learning".to_string()]);
        let short_query = TestQuery::short(vec!["learning".to_string()]);
        
        // Use queries
        let lexical_results = retrieve_bm25(
            &scenario.index,
            &lexical_query.terms,
            10,
            Bm25Params::default(),
        ).unwrap();
        
        let short_results = retrieve_bm25(
            &scenario.index,
            &short_query.terms,
            10,
            Bm25Params::default(),
        ).unwrap();
        
        assert!(!lexical_results.is_empty());
        assert!(!short_results.is_empty());
    }

    #[cfg(feature = "bm25")]
    #[cfg(feature = "dense")]
    #[cfg(feature = "sparse")]
    #[test]
    fn example_using_multi_retriever_fixture() {
        // Before: Manual setup of all three retrievers
        // After: Use MultiRetrieverFixture
        
        let fixture = MultiRetrieverFixture::machine_learning();
        
        // All retrievers are ready to use
        let query = fixture.collection.queries[0].1.clone();
        let bm25_results = retrieve_bm25(
            &fixture.bm25_index,
            &query,
            10,
            Bm25Params::default(),
        ).unwrap();
        
        // Dense and sparse retrievers also available
        assert!(!bm25_results.is_empty());
    }
}

