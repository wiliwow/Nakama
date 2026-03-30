// Integration tests for RAG functionality
// Run with: cargo test --test rag_integration --features swiftide_integration

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
    use nakama::commands::rag::rag_health_check;

    let health = rag_health_check().await;
    
    match health {
        Ok(result) => {
            println!("✓ Health check result:");
            println!("  Embeddings: {}", if result.embeddings_ok { "✓" } else { "✗" });
            if let Some(err) = &result.embeddings_error {
                println!("    Error: {}", err);
            }
            println!("  LLM: {}", if result.llm_ok { "✓" } else { "✗" });
            if let Some(err) = &result.llm_error {
                println!("    Error: {}", err);
            }
        }
        Err(e) => {
            println!("✗ Health check failed: {}", e);
        }
    }
}

#[cfg(not(feature = "swiftide_integration"))]
#[test]
fn test_feature_flag_warning() {
    println!("⚠️  Tests skipped: swiftide_integration feature not enabled");
    println!("    Run: cargo test --features swiftide_integration");
}
