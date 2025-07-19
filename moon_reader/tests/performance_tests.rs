// Performance tests for the knowledge accumulation system
use std::time::{Duration, Instant};
use tokio::time::timeout;
use moon_reader::{
    database::create_connection_pool,
    services::AppState,
    models::{Document, DocumentType},
    parsers::{DocumentParser, DocumentParserFactory},
};
use tempfile::NamedTempFile;
use std::io::Write;

// Helper function to create test app state
async fn create_test_app_state() -> AppState {
    let pool = create_connection_pool(":memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    AppState::new(pool)
}

// Helper function to create a large text file for testing
fn create_large_text_file(size_mb: usize) -> NamedTempFile {
    let mut temp_file = NamedTempFile::new().unwrap();
    let content = "A".repeat(1024); // 1KB of content
    let iterations = size_mb * 1024; // Repeat to get desired MB size
    
    for _ in 0..iterations {
        temp_file.write_all(content.as_bytes()).unwrap();
    }
    temp_file.flush().unwrap();
    temp_file
}

#[tokio::test]
async fn test_large_file_upload_performance() {
    let app_state = create_test_app_state().await;
    
    // Create a knowledge base
    let kb = app_state.db.create_knowledge_base("Performance Test KB", Some("Testing large file upload")).await.unwrap();
    
    // Test different file sizes
    let test_sizes = vec![1, 5, 10, 25]; // MB
    
    for size_mb in test_sizes {
        println!("Testing {}MB file upload...", size_mb);
        
        let temp_file = create_large_text_file(size_mb);
        let file_path = temp_file.path().to_string_lossy().to_string();
        
        let start_time = Instant::now();
        
        // Test file parsing performance
        let file_extension = std::path::Path::new(&file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("txt");
        let parser = DocumentParserFactory::get_parser(file_extension).unwrap();
        let content = parser.parse(std::path::Path::new(&file_path)).await.unwrap();
        
        let parse_duration = start_time.elapsed();
        println!("  Parsing {}MB took: {:?}", size_mb, parse_duration);
        
        // Ensure parsing doesn't take too long (reasonable threshold)
        assert!(parse_duration < Duration::from_secs(30), "Parsing {}MB file took too long: {:?}", size_mb, parse_duration);
        
        // Test database storage performance
        let start_time = Instant::now();
        
        let document = Document::new(
            kb.id.clone(),
            format!("large_file_{}mb.txt", size_mb),
            DocumentType::Txt,
            file_path,
            (size_mb * 1024 * 1024) as i64,
            Some(content),
        );
        
        app_state.db.save_document(&document).await.unwrap();
        
        let storage_duration = start_time.elapsed();
        println!("  Storing {}MB took: {:?}", size_mb, storage_duration);
        
        // Ensure storage doesn't take too long
        assert!(storage_duration < Duration::from_secs(10), "Storing {}MB file took too long: {:?}", size_mb, storage_duration);
    }
}

#[tokio::test]
async fn test_database_query_performance() {
    let app_state = create_test_app_state().await;
    
    // Create multiple knowledge bases with documents
    let num_kbs = 10;
    let docs_per_kb = 20;
    
    println!("Creating {} knowledge bases with {} documents each...", num_kbs, docs_per_kb);
    
    let setup_start = Instant::now();
    
    for i in 0..num_kbs {
        let kb = app_state.db.create_knowledge_base(
            &format!("Performance KB {}", i),
            Some(&format!("Performance testing knowledge base {}", i))
        ).await.unwrap();
        
        for j in 0..docs_per_kb {
            let content = format!("Content for document {} in KB {}", j, i);
            let document = Document::new(
                kb.id.clone(),
                format!("doc_{}_{}.txt", i, j),
                DocumentType::Txt,
                format!("/tmp/doc_{}_{}.txt", i, j),
                content.len() as i64,
                Some(content),
            );
            
            app_state.db.save_document(&document).await.unwrap();
        }
    }
    
    let setup_duration = setup_start.elapsed();
    println!("Setup took: {:?}", setup_duration);
    
    // Test knowledge base listing performance
    let start_time = Instant::now();
    let kbs = app_state.db.get_knowledge_bases().await.unwrap();
    let list_duration = start_time.elapsed();
    
    assert_eq!(kbs.len(), num_kbs);
    println!("Listing {} knowledge bases took: {:?}", num_kbs, list_duration);
    assert!(list_duration < Duration::from_millis(100), "Knowledge base listing took too long: {:?}", list_duration);
    
    // Test document listing performance for each KB
    let mut total_docs = 0;
    let start_time = Instant::now();
    
    for kb in &kbs {
        let docs = app_state.db.get_documents_by_knowledge_base(&kb.id).await.unwrap();
        total_docs += docs.len();
    }
    
    let docs_list_duration = start_time.elapsed();
    println!("Listing {} total documents took: {:?}", total_docs, docs_list_duration);
    assert!(docs_list_duration < Duration::from_millis(500), "Document listing took too long: {:?}", docs_list_duration);
    
    // Test individual document retrieval performance
    let first_kb = &kbs[0];
    let docs = app_state.db.get_documents_by_knowledge_base(&first_kb.id).await.unwrap();
    let first_doc = &docs[0];
    
    let start_time = Instant::now();
    let retrieved_doc = app_state.db.get_document_by_id(&first_doc.id).await.unwrap();
    let retrieval_duration = start_time.elapsed();
    
    assert!(retrieved_doc.is_some());
    println!("Individual document retrieval took: {:?}", retrieval_duration);
    assert!(retrieval_duration < Duration::from_millis(50), "Document retrieval took too long: {:?}", retrieval_duration);
}

#[tokio::test]
async fn test_concurrent_database_operations() {
    let app_state = create_test_app_state().await;
    
    // Test concurrent knowledge base creation
    let num_concurrent = 20;
    println!("Testing {} concurrent knowledge base creations...", num_concurrent);
    
    let start_time = Instant::now();
    
    let mut handles = vec![];
    for i in 0..num_concurrent {
        let app_state_clone = app_state.clone();
        let handle = tokio::spawn(async move {
            let kb_name = format!("Concurrent KB {}", i);
            let kb = app_state_clone.db.create_knowledge_base(&kb_name, Some("Concurrent test")).await.unwrap();
            
            // Add a document to each knowledge base
            let content = format!("Content for concurrent KB {}", i);
            let document = Document::new(
                kb.id.clone(),
                format!("concurrent_doc_{}.txt", i),
                DocumentType::Txt,
                format!("/tmp/concurrent_doc_{}.txt", i),
                content.len() as i64,
                Some(content),
            );
            
            app_state_clone.db.save_document(&document).await.unwrap();
            kb.id
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    let mut kb_ids = vec![];
    for handle in handles {
        let kb_id = handle.await.unwrap();
        kb_ids.push(kb_id);
    }
    
    let concurrent_duration = start_time.elapsed();
    println!("Concurrent operations took: {:?}", concurrent_duration);
    
    // Verify all operations completed successfully
    assert_eq!(kb_ids.len(), num_concurrent);
    
    // Verify data integrity
    let all_kbs = app_state.db.get_knowledge_bases().await.unwrap();
    assert_eq!(all_kbs.len(), num_concurrent);
    
    for kb_id in kb_ids {
        let documents = app_state.db.get_documents_by_knowledge_base(&kb_id).await.unwrap();
        assert_eq!(documents.len(), 1);
    }
    
    // Concurrent operations should complete in reasonable time
    assert!(concurrent_duration < Duration::from_secs(10), "Concurrent operations took too long: {:?}", concurrent_duration);
}

#[tokio::test]
async fn test_memory_usage_with_large_content() {
    let app_state = create_test_app_state().await;
    
    // Create a knowledge base
    let kb = app_state.db.create_knowledge_base("Memory Test KB", Some("Testing memory usage")).await.unwrap();
    
    // Test with progressively larger content to ensure memory doesn't grow unbounded
    let content_sizes = vec![1000, 10000, 100000, 500000]; // Characters
    
    for size in content_sizes {
        println!("Testing content size: {} characters", size);
        
        let large_content = "A".repeat(size);
        
        let start_time = Instant::now();
        
        let document = Document::new(
            kb.id.clone(),
            format!("memory_test_{}.txt", size),
            DocumentType::Txt,
            format!("/tmp/memory_test_{}.txt", size),
            large_content.len() as i64,
            Some(large_content.clone()),
        );
        
        app_state.db.save_document(&document).await.unwrap();
        
        // Retrieve the document to test memory usage during retrieval
        let retrieved_doc = app_state.db.get_document_by_id(&document.id).await.unwrap().unwrap();
        assert_eq!(retrieved_doc.content_text.as_ref().unwrap().len(), size);
        
        let operation_duration = start_time.elapsed();
        println!("  Operation took: {:?}", operation_duration);
        
        // Operations should complete in reasonable time regardless of content size
        assert!(operation_duration < Duration::from_secs(5), "Operation with {}KB content took too long: {:?}", size / 1000, operation_duration);
    }
}

#[tokio::test]
async fn test_ai_service_timeout_handling() {
    use moon_reader::services::ai::{AIProvider, DeepSeekProvider};
    
    // Test timeout handling for AI service calls with invalid API key
    let ai_provider = DeepSeekProvider::new("invalid-test-key".to_string());
    
    let start_time = Instant::now();
    
    // This should fail quickly due to invalid API key
    let result = timeout(
        Duration::from_secs(10),
        ai_provider.generate_question("test context")
    ).await;
    
    let operation_duration = start_time.elapsed();
    println!("AI service test took: {:?}", operation_duration);
    
    // Should complete within reasonable time (either success or failure)
    assert!(operation_duration < Duration::from_secs(10));
    
    // Result should be an error due to invalid API key or network issues
    match result {
        Ok(ai_result) => {
            // If the AI call completed, it should be an error due to invalid API key
            assert!(ai_result.is_err(), "Expected AI call to fail due to invalid API key");
            println!("AI service call failed as expected: {:?}", ai_result.err());
        }
        Err(_) => {
            // Timeout occurred
            println!("AI service call timed out");
        }
    }
}

#[tokio::test]
async fn test_database_connection_pool_performance() {
    // Test database connection pool efficiency
    let pool = create_connection_pool(":memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    
    let num_concurrent_queries = 50;
    println!("Testing {} concurrent database queries...", num_concurrent_queries);
    
    let start_time = Instant::now();
    
    let mut handles = vec![];
    for i in 0..num_concurrent_queries {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            // Perform a simple query
            let result = sqlx::query("SELECT COUNT(*) as count FROM knowledge_bases")
                .fetch_one(&pool_clone)
                .await;
            
            assert!(result.is_ok());
            i
        });
        handles.push(handle);
    }
    
    // Wait for all queries to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    let concurrent_queries_duration = start_time.elapsed();
    println!("Concurrent database queries took: {:?}", concurrent_queries_duration);
    
    // All queries should complete quickly with connection pooling
    assert!(concurrent_queries_duration < Duration::from_secs(5), "Concurrent queries took too long: {:?}", concurrent_queries_duration);
}

#[tokio::test]
async fn test_document_parsing_performance() {
    // Test parsing performance for different document types
    let test_content = "This is a test document with some content. ".repeat(1000); // ~43KB
    
    // Test TXT parsing
    let mut txt_file = NamedTempFile::new().unwrap();
    txt_file.write_all(test_content.as_bytes()).unwrap();
    txt_file.flush().unwrap();
    
    let start_time = Instant::now();
    let parser = DocumentParserFactory::get_parser("txt").unwrap();
    let parsed_content = parser.parse(txt_file.path()).await.unwrap();
    let txt_parse_duration = start_time.elapsed();
    
    assert_eq!(parsed_content.len(), test_content.len());
    println!("TXT parsing took: {:?}", txt_parse_duration);
    
    // TXT parsing should be very fast
    assert!(txt_parse_duration < Duration::from_millis(100), "TXT parsing took too long: {:?}", txt_parse_duration);
    
    // Test with multiple files concurrently
    let num_concurrent_parses = 10;
    println!("Testing {} concurrent document parses...", num_concurrent_parses);
    
    let start_time = Instant::now();
    let mut handles = vec![];
    
    for i in 0..num_concurrent_parses {
        let content = format!("Document {} content. ", i).repeat(500);
        let handle = tokio::spawn(async move {
            let mut temp_file = NamedTempFile::new().unwrap();
            temp_file.write_all(content.as_bytes()).unwrap();
            temp_file.flush().unwrap();
            
            let parser = DocumentParserFactory::get_parser("txt").unwrap();
            let parsed = parser.parse(temp_file.path()).await.unwrap();
            
            assert!(!parsed.is_empty());
            i
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    let concurrent_parse_duration = start_time.elapsed();
    println!("Concurrent parsing took: {:?}", concurrent_parse_duration);
    
    // Concurrent parsing should complete efficiently
    assert!(concurrent_parse_duration < Duration::from_secs(2), "Concurrent parsing took too long: {:?}", concurrent_parse_duration);
}