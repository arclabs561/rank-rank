//! Comprehensive edge case tests for index factory.
//!
//! Tests boundary conditions, error cases, and unusual inputs.

use rank_retrieve::dense::ann::factory::index_factory;
use rank_retrieve::RetrieveError;

#[test]
fn test_factory_empty_inputs() {
    // Empty string
    assert!(index_factory(128, "").is_err());
    
    // Whitespace only
    assert!(index_factory(128, "   ").is_err());
    assert!(index_factory(128, "\t").is_err());
    assert!(index_factory(128, "\n").is_err());
}

#[test]
fn test_factory_zero_dimension() {
    assert!(index_factory(0, "HNSW32").is_err());
    assert!(index_factory(0, "IVF1024,PQ8").is_err());
    assert!(index_factory(0, "SCANN256").is_err());
}

#[test]
fn test_factory_zero_parameters() {
    #[cfg(feature = "hnsw")]
    {
        assert!(index_factory(128, "HNSW0").is_err());
        assert!(index_factory(128, "HNSW0,32").is_err());
        assert!(index_factory(128, "HNSW32,0").is_err());
    }
    
    #[cfg(feature = "nsw")]
    {
        assert!(index_factory(128, "NSW0").is_err());
    }
    
    #[cfg(feature = "ivf_pq")]
    {
        assert!(index_factory(128, "IVF0,PQ8").is_err());
        assert!(index_factory(128, "IVF1024,PQ0").is_err());
    }
    
    #[cfg(feature = "scann")]
    {
        assert!(index_factory(128, "SCANN0").is_err());
    }
}

#[test]
fn test_factory_invalid_formats() {
    // Missing parameters
    assert!(index_factory(128, "HNSW").is_err());
    assert!(index_factory(128, "IVF").is_err());
    assert!(index_factory(128, "IVF1024").is_err());
    assert!(index_factory(128, "SCANN").is_err());
    
    // Invalid characters
    assert!(index_factory(128, "HNSWabc").is_err());
    assert!(index_factory(128, "HNSW-32").is_err());
    assert!(index_factory(128, "HNSW32.5").is_err());
    
    // Wrong order
    assert!(index_factory(128, "PQ8,IVF1024").is_err());
}

#[test]
fn test_factory_dimension_divisibility() {
    #[cfg(feature = "ivf_pq")]
    {
        // Valid: 128 % 8 = 0
        assert!(index_factory(128, "IVF1024,PQ8").is_ok());
        
        // Invalid: 100 % 8 != 0
        assert!(index_factory(100, "IVF1024,PQ8").is_err());
        
        // Valid: 100 % 4 = 0
        assert!(index_factory(100, "IVF1024,PQ4").is_ok());
        
        // Valid: 96 % 8 = 0
        assert!(index_factory(96, "IVF1024,PQ8").is_ok());
    }
}

#[test]
fn test_factory_very_large_parameters() {
    #[cfg(feature = "hnsw")]
    {
        // Very large m (should work if within limits)
        let result = index_factory(128, "HNSW128");
        // May succeed or fail depending on implementation limits
        let _ = result;  // Just check it doesn't panic
    }
    
    #[cfg(feature = "ivf_pq")]
    {
        // Very large clusters
        let result = index_factory(128, "IVF10000,PQ8");
        // May succeed or fail depending on implementation limits
        let _ = result;
    }
}

#[test]
fn test_factory_very_small_dimensions() {
    #[cfg(feature = "hnsw")]
    {
        // Minimum valid dimension
        assert!(index_factory(1, "HNSW32").is_ok());
    }
    
    #[cfg(feature = "ivf_pq")]
    {
        // Small dimension with matching codebooks
        assert!(index_factory(8, "IVF16,PQ1").is_ok());
        assert!(index_factory(8, "IVF16,PQ2").is_ok());
        assert!(index_factory(8, "IVF16,PQ4").is_ok());
        assert!(index_factory(8, "IVF16,PQ8").is_ok());
    }
}

#[test]
fn test_factory_case_sensitivity() {
    // Factory should be case-sensitive (following Faiss pattern)
    assert!(index_factory(128, "hnsw32").is_err());  // Lowercase
    assert!(index_factory(128, "Hnsw32").is_err());  // Mixed case
    assert!(index_factory(128, "HNSW32").is_ok());  // Correct case
}

#[test]
fn test_factory_special_characters() {
    // Should handle or reject special characters appropriately
    assert!(index_factory(128, "HNSW32!").is_err());
    assert!(index_factory(128, "HNSW@32").is_err());
    assert!(index_factory(128, "HNSW#32").is_err());
}

#[test]
fn test_factory_whitespace_variations() {
    #[cfg(feature = "hnsw")]
    {
        let base = index_factory(128, "HNSW32");
        let with_spaces = index_factory(128, "  HNSW32  ");
        let with_tabs = index_factory(128, "\tHNSW32\t");
        let with_newlines = index_factory(128, "\nHNSW32\n");
        
        // All should produce same result
        assert_eq!(base.is_ok(), with_spaces.is_ok());
        assert_eq!(base.is_ok(), with_tabs.is_ok());
        assert_eq!(base.is_ok(), with_newlines.is_ok());
    }
}

#[test]
fn test_factory_parameter_parsing_edge_cases() {
    #[cfg(feature = "hnsw")]
    {
        // Multiple commas (should fail gracefully)
        assert!(index_factory(128, "HNSW16,32,64").is_err());
        
        // Trailing comma
        assert!(index_factory(128, "HNSW32,").is_err());
        
        // Leading comma
        assert!(index_factory(128, ",HNSW32").is_err());
    }
    
    #[cfg(feature = "ivf_pq")]
    {
        // Missing PQ part
        assert!(index_factory(128, "IVF1024,").is_err());
        
        // Extra parts
        assert!(index_factory(128, "IVF1024,PQ8,Extra").is_err());
        
        // Invalid PQ format
        assert!(index_factory(128, "IVF1024,PQ8x").is_err());
        assert!(index_factory(128, "IVF1024,PQx8").is_err());
        assert!(index_factory(128, "IVF1024,PQ8x8x8").is_err());
    }
}

#[test]
fn test_factory_unicode_handling() {
    // Should reject or handle unicode appropriately
    assert!(index_factory(128, "HNSW32中文").is_err());
    assert!(index_factory(128, "HNSW32🚀").is_err());
    assert!(index_factory(128, "HNSW32é").is_err());
}

#[test]
fn test_factory_very_long_strings() {
    // Very long but valid string
    #[cfg(feature = "ivf_pq")]
    {
        let long_string = format!("IVF{},PQ{}", 999999, 32);
        let result = index_factory(128, &long_string);
        // May succeed or fail, but shouldn't panic
        let _ = result;
    }
}

#[test]
fn test_factory_repeated_calls() {
    #[cfg(feature = "hnsw")]
    {
        // Should be able to create multiple indexes
        for i in 0..10 {
            let result = index_factory(128, "HNSW32");
            assert!(result.is_ok(), "Should succeed on call {}", i);
        }
    }
}

#[test]
fn test_factory_error_message_quality() {
    // Test that error messages are helpful
    let result = index_factory(128, "INVALID");
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = format!("{}", err);
    
    // Error should mention what's wrong
    assert!(err_str.contains("Unsupported") || err_str.contains("INVALID"),
            "Error message should be helpful: {}", err_str);
    
    // Error should mention supported formats
    assert!(err_str.contains("HNSW") || err_str.contains("IVF") || err_str.contains("SCANN"),
            "Error message should mention supported formats: {}", err_str);
}

#[test]
fn test_factory_feature_gating_messages() {
    // When feature is disabled, error should tell user how to enable it
    #[cfg(not(feature = "hnsw"))]
    {
        let result = index_factory(128, "HNSW32");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = format!("{}", err);
        assert!(err_str.contains("feature") || err_str.contains("Cargo.toml"),
                "Error should tell user how to enable feature: {}", err_str);
    }
}
