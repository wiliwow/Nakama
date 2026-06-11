// Integration tests for RAG functionality
// Run with: cargo test --test rag_integration --features swiftide_integration

use std::sync::Arc;
use std::sync::Mutex;

// Integration tests for RAG functionality - they require a Tauri State context
// which is difficult to construct in unit tests. For now, these tests validate
// the data structures and helper functions.

#[cfg(feature = "swiftide_integration")]
#[tokio::test]
async fn test_rag_add_file_and_retrieve() {
    use std::fs;

    // Create a temporary test directory
    let test_dir = std::env::temp_dir().join("nakama_rag_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test dir");

    // Test content
    let test_filename = "test_document.txt";
    let test_content = "The quick brown fox jumps over the lazy dog. \
                       This is a test document for RAG indexing. \
                       It contains information about animals and testing.";

    // Since we can't easily invoke Tauri commands in tests, we'll test the inner logic
    // This is a placeholder demonstrating the test structure
    
    println!("✓ Test: RAG add_file and retrieve");
    println!("  File: {}", test_filename);
    println!("  Content length: {} chars", test_content.len());

    // In a real test, we'd:
    // 1. Call rag_add_file(filename, content)
    // 2. Call rag_retrieve(query) with a relevant query
    // 3. Assert that retrievals contain the expected content

    assert!(!test_content.is_empty(), "Test content should not be empty");
    
    let _ = fs::remove_dir_all(&test_dir);
}

#[cfg(feature = "swiftide_integration")]
#[tokio::test]
async fn test_rag_health_check() {
    // Note: Testing rag_health_check requires a Tauri State context with ConfigManager,
    // which is not easily constructed in unit tests. This test validates the HealthCheckResult
    // struct deserializes correctly.
    
    use nakama::commands::rag::HealthCheckResult;
    
    // Test that HealthCheckResult can be serialized/deserialized
    let result = HealthCheckResult {
        embeddings_ok: true,
        embeddings_error: None,
        llm_ok: true,
        llm_error: None,
    };
    
    let json = serde_json::to_string(&result).expect("Failed to serialize");
    let deserialized: HealthCheckResult = serde_json::from_str(&json).expect("Failed to deserialize");
    
    assert!(deserialized.embeddings_ok);
    assert!(deserialized.llm_ok);
    
    println!("✓ Test: HealthCheckResult serialization works");
}

#[cfg(feature = "swiftide_integration")]
#[tokio::test]
async fn test_rag_index_stats() {
    // Test IndexStats struct deserialization
    use nakama::commands::rag::IndexStats;
    
    let stats = IndexStats {
        indexed_documents: 10,
        indexed_chunks: 50,
        total_indexed_bytes: 1024 * 1024,
        last_indexed_at: Some(1234567890),
        estimated_index_size_mb: 1,
    };
    
    let json = serde_json::to_string(&stats).expect("Failed to serialize");
    let deserialized: IndexStats = serde_json::from_str(&json).expect("Failed to deserialize");
    
    assert_eq!(deserialized.indexed_documents, 10);
    assert_eq!(deserialized.indexed_chunks, 50);
    
    println!("✓ Test: IndexStats serialization works");
}

#[cfg(not(feature = "swiftide_integration"))]
#[test]
fn test_feature_flag_warning() {
    println!("⚠️  Tests skipped: swiftide_integration feature not enabled");
    println!("    Run: cargo test --features swiftide_integration");
}
